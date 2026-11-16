//! Assertion: rewrites the asserts of a module, and loads the rewritten module.
//! An assert at the end of a test's body, or at the end of the body of a `with`
//! just before such asserts, is an expectation, which becomes a reading that
//! execution compares. Every other assert, except a tuple assert, keeps a mark
//! that records that it ran, and whether it held. A step follows each statement
//! of a test before its first expectation, and each item of each `with` of a
//! test is wrapped, so that the case sees its hooks and its `with`s.

use purba_parsing::Module;
use pyo3::basic::CompareOp;
use pyo3::exceptions::{PyIndexError, PyNameError, PyTypeError};
use pyo3::types::{
    PyAnyMethods, PyDictMethods, PyModule, PyModuleMethods, PyTuple, PyTupleMethods, PyTypeMethods,
};
use pyo3::{Bound, IntoPyObject, Py, PyAny, PyErr, PyResult, Python, pyclass, pymethods};
use ruff_python_ast::statement_visitor::{StatementVisitor, walk_stmt};
use ruff_python_ast::{CmpOp, ExceptHandler, Expr, ExprCompare, Stmt, StmtAssert, StmtFunctionDef};
use ruff_text_size::{Ranged, TextRange, TextSize};
use std::collections::{HashMap, HashSet};
use std::io;
use std::iter;
use std::ops::Range;
use std::panic::{self, AssertUnwindSafe};
use std::path::{Path, PathBuf};
use std::sync::{Mutex, MutexGuard, PoisonError};

/// The name that each marked assert calls.
const RAN: &str = "_purba_ran";
/// The name that each expectation calls with its reading.
const READ: &str = "_purba_read";
/// The name that each expectation calls first, to learn whether it reads.
const READS: &str = "_purba_reads";
/// The name that a test calls after each statement before its first
/// expectation, with the line of the statement.
const STEP: &str = "_purba_step";
/// The name that wraps each item of each `with` of a test, with the line of
/// the `with`.
const WITH: &str = "_purba_with";

/// The rewritten text of one module, the text and the line of each assert
/// that it marks, the text and the line of each expectation of each test, and
/// the statements of the body of each `with` of a test.
pub struct Rewrite {
    text: String,
    asserts: Vec<String>,
    marks: Vec<usize>,
    expectations: Vec<String>,
    checks: Vec<Option<Check>>,
    lines: Vec<usize>,
    tests: Vec<(String, Range<usize>)>,
    bodies: HashMap<usize, Body>,
}

/// The operator of an expectation that compares two values by one, and
/// whether its expected value is the left side: the left is a literal, and
/// the right is not.
#[derive(Clone, Copy, Debug)]
struct Check {
    op: CmpOp,
    expected_left: bool,
}

/// The two sides of `test` when it compares two values by one operator.
fn sides(test: &Expr) -> Option<(&ExprCompare, &Expr, &Expr)> {
    let Expr::Compare(compare) = test else {
        return None;
    };
    match (&*compare.ops, &*compare.operands) {
        ([_], [left, right]) => Some((compare, left, right)),
        _ => None,
    }
}

/// The body of a `with`: the line of each statement in it, at any depth, in
/// source order, and its last line.
#[derive(Clone, Debug, Default)]
struct Body {
    starts: Vec<usize>,
    last: usize,
}

impl Rewrite {
    /// The module's text, with each expectation rewritten, and each other
    /// assert marked except a tuple assert.
    #[must_use]
    pub fn text(&self) -> &str {
        &self.text
    }

    /// The text of each marked assert, without its comments and line continuations, in
    /// source order. The index of an assert here is the index that its mark
    /// records.
    #[must_use]
    pub fn asserts(&self) -> &[String] {
        &self.asserts
    }

    /// The text of each expectation of `test`, without its comments and line
    /// continuations, in source order. A function that is not a test of the
    /// rewrite has none.
    #[must_use]
    pub fn expectations(&self, test: &str) -> &[String] {
        self.tests
            .iter()
            .find(|(name, _)| name == test)
            .map_or(&[], |(_, range)| &self.expectations[range.clone()])
    }

    /// Parses `source`, the text of a module that no file holds, and rewrites
    /// it, with `tests` as the names of its tests.
    ///
    /// # Errors
    ///
    /// When `source` does not parse.
    pub fn of_source(source: &str, tests: &[&str]) -> Result<Self, purba_parsing::Error> {
        Ok(rewrite(
            &purba_parsing::parse_source(source.to_owned())?,
            tests,
        ))
    }
}

/// Whether `assert` tests a tuple with items, which is always true.
fn of_a_tuple(assert: &StmtAssert) -> bool {
    matches!(&*assert.test, Expr::Tuple(tuple) if !tuple.elts.is_empty())
}

/// Each assert of a module that the rewrite marks, in source order, at any
/// depth.
struct Asserts<'a> {
    expectations: &'a [&'a StmtAssert],
    found: Vec<&'a StmtAssert>,
}

impl<'a> StatementVisitor<'a> for Asserts<'a> {
    fn visit_stmt(&mut self, statement: &'a Stmt) {
        if let Stmt::Assert(assert) = statement
            && !of_a_tuple(assert)
            && !self
                .expectations
                .iter()
                .any(|expectation| std::ptr::eq(*expectation, assert))
        {
            self.found.push(assert);
        }
        walk_stmt(self, statement);
    }
}

/// The expectations at the end of `body`: the asserts at its end, with no other
/// statement after them, after those at the end of the body of a `with` just
/// before them, at any depth. Each comes in source order.
fn ending(body: &[Stmt]) -> Vec<&StmtAssert> {
    let mut segments: Vec<Vec<&StmtAssert>> = Vec::new();
    let mut current = body;
    loop {
        let mut asserts: Vec<&StmtAssert> = current
            .iter()
            .rev()
            .map_while(|statement| match statement {
                Stmt::Assert(assert) if !of_a_tuple(assert) => Some(assert),
                _ => None,
            })
            .collect();
        asserts.reverse();
        let before = current.len() - asserts.len();
        segments.push(asserts);
        match before.checked_sub(1).map(|last| &current[last]) {
            Some(Stmt::With(with)) if !with.is_async => current = &with.body,
            _ => break,
        }
    }
    segments.into_iter().rev().flatten().collect()
}

/// The `def` of each test of `module` that `tests` names, in source order, at
/// the top of the module. When two `def`s bind one name, the last binds it.
fn tests_of<'a>(module: &'a Module, tests: &[&str]) -> Vec<&'a StmtFunctionDef> {
    let mut found: Vec<&StmtFunctionDef> = Vec::new();
    for statement in &module.syntax().body {
        let Stmt::FunctionDef(function) = statement else {
            continue;
        };
        if function.is_async || !tests.contains(&function.name.as_str()) {
            continue;
        }
        found.retain(|seen| seen.name != function.name);
        found.push(function);
    }
    found
}

/// Each statement of `body`, at any depth, in source order. The body of a
/// `def` or a `class` is not part of it.
fn statements(body: &[Stmt]) -> Vec<&Stmt> {
    let mut found = Vec::new();
    // A walk that recurses once for each level of nesting overflows the stack
    // on statements that nest deeper than the stack.
    let mut pending: Vec<&Stmt> = body.iter().rev().collect();
    while let Some(statement) = pending.pop() {
        found.push(statement);
        let blocks: Vec<&[Stmt]> = match statement {
            Stmt::If(branch) => iter::once(branch.body.as_slice())
                .chain(
                    branch
                        .elif_else_clauses
                        .iter()
                        .map(|clause| clause.body.as_slice()),
                )
                .collect(),
            Stmt::For(each) => vec![&each.body, &each.orelse],
            Stmt::While(each) => vec![&each.body, &each.orelse],
            Stmt::With(with) => vec![&with.body],
            Stmt::Try(attempt) => iter::once(attempt.body.as_slice())
                .chain(attempt.handlers.iter().map(|handler| {
                    let ExceptHandler::ExceptHandler(clause) = handler;
                    clause.body.as_slice()
                }))
                .chain([attempt.orelse.as_slice(), attempt.finalbody.as_slice()])
                .collect(),
            Stmt::Match(choice) => choice
                .cases
                .iter()
                .map(|case| case.body.as_slice())
                .collect(),
            _ => Vec::new(),
        };
        pending.extend(
            blocks
                .into_iter()
                .rev()
                .flat_map(|block| block.iter().rev()),
        );
    }
    found
}

/// Whether `statement` is simple: it holds no other statement.
const fn is_simple(statement: &Stmt) -> bool {
    !matches!(
        statement,
        Stmt::If(_)
            | Stmt::For(_)
            | Stmt::While(_)
            | Stmt::With(_)
            | Stmt::Try(_)
            | Stmt::Match(_)
            | Stmt::FunctionDef(_)
            | Stmt::ClassDef(_)
    )
}

/// One change of the rewrite: the text that takes the place of the source
/// from `start` to `end`. An insertion starts where it ends.
struct Edit {
    start: usize,
    end: usize,
    text: String,
}

impl Edit {
    const fn insert(at: usize, text: String) -> Self {
        Self {
            start: at,
            end: at,
            text,
        }
    }
}

/// Rewrites each assert of `module`, where `tests` names its tests.
///
/// An expectation `assert x` becomes `_purba_reads(i) and _purba_read(i, x)`,
/// where `i` counts the expectations of the module in source order, from 0. An
/// expectation that compares two values by one operator, `assert a < b`, becomes
/// `_purba_reads(i) and _purba_read(i, (a), (b))`, and the hook compares them by
/// that operator. Each keeps each line break of the assert inside the call, so
/// each later line keeps its number, and drops the assert's message.
///
/// Every other assert `assert x` becomes `assert _purba_ran(i, x)`, where `i`
/// counts the marked asserts of the module in source order, from 0. An assert
/// of a tuple with items stays as written, so that the compiler warns that it
/// is always true.
///
/// In a test, each simple statement that is no expectation gains
/// `; _purba_step(n)`, and each item `m` of a `with` becomes
/// `_purba_with(n, m)`, where `n` is the line of the statement. The body of a
/// `def` or a `class` in a test gains neither. Each line keeps its number.
///
/// # Panics
///
/// In a build with the feature `fault`, when `PURBA_FAULT` is `rewrite`.
#[must_use]
pub fn rewrite(module: &Module, tests: &[&str]) -> Rewrite {
    if cfg!(feature = "fault")
        && std::env::var_os("PURBA_FAULT").is_some_and(|setting| setting == "rewrite")
    {
        panic!("PURBA_FAULT=rewrite");
    }
    let source = module.source();
    let line = |offset: TextSize| module.position(offset).line();
    let by_test: Vec<(&StmtFunctionDef, Vec<&StmtAssert>)> = tests_of(module, tests)
        .into_iter()
        .map(|function| (function, ending(&function.body)))
        .collect();
    let all: Vec<&StmtAssert> = by_test
        .iter()
        .flat_map(|(_, asserts)| asserts.iter().copied())
        .collect();
    let expected: HashSet<*const StmtAssert> = all
        .iter()
        .map(|assert| std::ptr::from_ref(*assert))
        .collect();
    let mut visitor = Asserts {
        expectations: &all,
        found: Vec::new(),
    };
    visitor.visit_body(&module.syntax().body);

    let mut edits: Vec<Edit> = Vec::new();
    for (index, assert) in visitor.found.iter().enumerate() {
        let condition = assert.test.range();
        let mut text = format!("{RAN}({index}, ");
        push_condition(&mut text, source, assert);
        text.push(')');
        edits.push(Edit {
            start: condition.start().to_usize(),
            end: condition.end().to_usize(),
            text,
        });
    }
    for (index, assert) in all.iter().enumerate() {
        edits.push(read_edit(index, assert, source));
    }
    let bodies = watch_each_test(module, &by_test, &expected, &mut edits);
    edits.sort_by_key(|edit| (edit.start, edit.end));

    let mut rewritten = String::with_capacity(source.len());
    let mut copied = 0;
    for edit in &edits {
        rewritten.push_str(&source[copied..edit.start]);
        rewritten.push_str(&edit.text);
        copied = edit.end;
    }
    rewritten.push_str(&source[copied..]);

    let mut ranges = Vec::with_capacity(by_test.len());
    let mut start = 0;
    for (function, asserts) in &by_test {
        ranges.push((function.name.to_string(), start..start + asserts.len()));
        start += asserts.len();
    }
    Rewrite {
        text: rewritten,
        asserts: visitor
            .found
            .iter()
            .map(|assert| without_trivia(module, assert.range()))
            .collect(),
        marks: visitor
            .found
            .iter()
            .map(|assert| line(assert.start()))
            .collect(),
        expectations: all
            .iter()
            .map(|assert| without_trivia(module, assert.range()))
            .collect(),
        checks: all
            .iter()
            .map(|assert| {
                sides(&assert.test).map(|(compare, left, right)| Check {
                    op: compare.ops[0],
                    expected_left: purba_parsing::is_literal(left)
                        && !purba_parsing::is_literal(right),
                })
            })
            .collect(),
        lines: all.iter().map(|assert| line(assert.start())).collect(),
        tests: ranges,
        bodies,
    }
}

/// The edit that makes `assert`, the expectation at `index`, a call of the
/// hook. A comparison of two values by one operator hands the hook each side.
fn read_edit(index: usize, assert: &StmtAssert, source: &str) -> Edit {
    let whole = assert.range();
    let mut text = format!("{READS}({index}) and {READ}({index}, ");
    let mut copied = whole.start();
    if let Some((_, left, right)) = sides(&assert.test) {
        for (side, start) in [(left, ""), (right, ", ")] {
            push_breaks(&mut text, &source[TextRange::new(copied, side.start())]);
            text.push_str(start);
            push_side(&mut text, source, side);
            copied = side.end();
        }
    } else {
        let condition = assert.test.range();
        push_breaks(
            &mut text,
            &source[TextRange::new(copied, condition.start())],
        );
        push_condition(&mut text, source, assert);
        copied = condition.end();
    }
    push_breaks(&mut text, &source[TextRange::new(copied, whole.end())]);
    text.push(')');
    Edit {
        start: whole.start().to_usize(),
        end: whole.end().to_usize(),
        text,
    }
}

/// Writes the condition of `assert`.
fn push_condition(rewritten: &mut String, source: &str, assert: &StmtAssert) {
    // ruff's range of `(yield x)` leaves out the parentheses, and a yield
    // inside an expression is a `yield_atom`, which needs them.
    // https://docs.python.org/3.12/reference/expressions.html#yield-expressions
    let bare = matches!(*assert.test, Expr::Yield(_) | Expr::YieldFrom(_));
    let (open, close) = if bare { ("(", ")") } else { ("", "") };
    rewritten.push_str(open);
    rewritten.push_str(&source[assert.test.range()]);
    rewritten.push_str(close);
}

/// Writes `side`, one side of a comparison, in parentheses. ruff's range of
/// a side leaves out the parentheses around it, and the side may need them.
fn push_side(rewritten: &mut String, source: &str, side: &Expr) {
    rewritten.push('(');
    rewritten.push_str(&source[side.range()]);
    rewritten.push(')');
}

/// Writes a line feed for each line break in `text`.
fn push_breaks(rewritten: &mut String, text: &str) {
    for _ in text.matches('\n') {
        rewritten.push('\n');
    }
}

/// The text of `range` in `module`, without its comments and the backslash of
/// each line continuation.
fn without_trivia(module: &Module, range: TextRange) -> String {
    let source = module.source();
    let mut text = String::with_capacity(range.len().to_usize());
    let mut start = range.start();
    for trivia in module
        .trivia()
        .iter()
        .filter(|trivia| range.contains_range(**trivia))
    {
        text.push_str(&source[TextRange::new(start, trivia.start())]);
        start = trivia.end();
    }
    text.push_str(&source[TextRange::new(start, range.end())]);
    text
}

/// The panic of the fault `assert`, in a build with the feature `fault`.
fn fault() {
    if cfg!(feature = "fault")
        && std::env::var_os("PURBA_FAULT").is_some_and(|setting| setting == "assert")
    {
        panic!("PURBA_FAULT=assert");
    }
}

/// Whether `left` and `right` hold by `op`, as Python's own operator decides.
fn holds(op: CmpOp, left: &Bound<'_, PyAny>, right: &Bound<'_, PyAny>) -> PyResult<bool> {
    let by = |op| left.rich_compare(right, op)?.is_truthy();
    match op {
        CmpOp::Eq => by(CompareOp::Eq),
        CmpOp::NotEq => by(CompareOp::Ne),
        CmpOp::Lt => by(CompareOp::Lt),
        CmpOp::LtE => by(CompareOp::Le),
        CmpOp::Gt => by(CompareOp::Gt),
        CmpOp::GtE => by(CompareOp::Ge),
        CmpOp::In => right.contains(left),
        CmpOp::NotIn => right.contains(left).map(|contains| !contains),
        CmpOp::Is => Ok(left.is(right)),
        CmpOp::IsNot => Ok(!left.is(right)),
    }
}

/// What an expectation that compares `left` and `right` by `check` shows.
fn shown(check: Check, left: &Bound<'_, PyAny>, right: &Bound<'_, PyAny>) -> Shown {
    let (expected, reading) = if check.expected_left {
        (left, right)
    } else {
        (right, left)
    };
    let text = |value: &Bound<'_, PyAny>| {
        value
            .repr()
            .map(|text| text.to_string())
            .map_err(|err| format!("(repr raised {err})"))
    };
    let (mut expected_text, mut reading_text) = (text(expected), text(reading));
    if let (Ok(expected_text), Ok(reading_text)) = (&mut expected_text, &mut reading_text)
        && expected_text == reading_text
    {
        for (text, value) in [(expected_text, expected), (reading_text, reading)] {
            let class = value
                .get_type()
                .name()
                .map_or_default(|name| name.to_string());
            text.push_str(" (");
            text.push_str(&class);
            text.push(')');
        }
    }
    Shown {
        expected: rule(check, expected_text.unwrap_or_else(|text| text)),
        reading: reading_text.unwrap_or_else(|text| text),
    }
}

/// The text of `expected` under the rule of the operator of `check`, read from
/// the side of the reading: `== 2` shows `2`, `< 2` shows `< 2`, `2 < x` shows
/// `> 2`, and `'z' in x` shows `to contain 'z'`.
fn rule(check: Check, expected: String) -> String {
    let words = match (check.op, check.expected_left) {
        (CmpOp::Eq, _) => return expected,
        (CmpOp::NotEq, _) => "!=",
        (CmpOp::Lt, false) | (CmpOp::Gt, true) => "<",
        (CmpOp::LtE, false) | (CmpOp::GtE, true) => "<=",
        (CmpOp::Gt, false) | (CmpOp::Lt, true) => ">",
        (CmpOp::GtE, false) | (CmpOp::LtE, true) => ">=",
        (CmpOp::In, false) => "in",
        (CmpOp::NotIn, false) => "not in",
        (CmpOp::In, true) => "to contain",
        (CmpOp::NotIn, true) => "not to contain",
        (CmpOp::Is, _) => "is",
        (CmpOp::IsNot, _) => "is not",
    };
    format!("{words} {expected}")
}

/// Where the assert at `line` runs. It is `line N` when the innermost frame
/// is the test's own frame. Else it is `line T reaches line N of F, which`.
/// `T` is the line that the test runs, and `F` names the function of the
/// innermost frame. The test is the outermost frame whose code comes from
/// `file`, the module's file.
fn place(py: Python<'_>, line: usize, file: &Bound<'_, PyAny>) -> PyResult<String> {
    let innermost = py.import("sys")?.getattr("_getframe")?.call0()?;
    let mut test = None;
    let mut frame = innermost.clone();
    while !frame.is_none() {
        if frame.getattr("f_code")?.getattr("co_filename")?.eq(file)? {
            test = Some(frame.clone());
        }
        frame = frame.getattr("f_back")?;
    }
    let Some(test) = test.filter(|test| !test.is(&innermost)) else {
        return Ok(format!("line {line}"));
    };
    let reaches: usize = test.getattr("f_lineno")?.extract()?;
    let name: String = innermost.getattr("f_code")?.getattr("co_name")?.extract()?;
    Ok(format!(
        "line {reaches} reaches line {line} of {name}, which"
    ))
}

/// Whether a frame of the call that runs now, from the innermost out, runs an
/// `__exit__`: the cleanup of a `with`.
fn in_cleanup(py: Python<'_>) -> PyResult<bool> {
    let mut frame = py.import("sys")?.getattr("_getframe")?.call0()?;
    while !frame.is_none() {
        if frame
            .getattr("f_code")?
            .getattr("co_name")?
            .eq("__exit__")?
        {
            return Ok(true);
        }
        frame = frame.getattr("f_back")?;
    }
    Ok(false)
}

/// The text of each marked assert of one loaded module, and the index of the
/// last one that did not hold, or else of the last one that ran, beside
/// whether it held. It also holds the line of each, and the hook of the
/// expectations, which says whether a case runs before its first expectation
/// or inside a reading, and the file of the module.
#[pyclass(frozen)]
struct Ran {
    asserts: Vec<String>,
    marks: Vec<usize>,
    read: Py<Read>,
    file: Py<PyAny>,
    kept: Mutex<Option<(usize, bool)>>,
}

#[pymethods]
impl Ran {
    /// Tests `value` once, and records `index` and whether `value` held, unless
    /// `value` held and an assert that did not hold is on record. Gives back
    /// whether `value` held, so that the assert tests a `bool`.
    ///
    /// While a case runs before its first expectation, it also records the
    /// assert as a fault of the form, unless it runs in the cleanup of a
    /// `with`. Inside a reading, it records the assert as a fault of the form,
    /// and a reader or a replay ends with `Verdict::Refused`.
    ///
    /// # Errors
    ///
    /// `IndexError` when no assert has `index`, or the error that testing
    /// `value` raised. Either records no index.
    ///
    /// # Panics
    ///
    /// In a build with the feature `fault`, when `PURBA_FAULT` is `assert`.
    fn __call__(&self, index: usize, value: &Bound<'_, PyAny>) -> PyResult<bool> {
        fault();
        if index >= self.asserts.len() {
            return Err(PyIndexError::new_err(format!(
                "{RAN}({index}): the module has {} marked asserts",
                self.asserts.len()
            )));
        }
        let py = value.py();
        let read = self.read.get();
        let line = self.marks[index];
        if read.inside_a_reading() {
            if let Some(readers) = read.reader() {
                readers.end(Verdict::Refused);
            }
            read.fault(
                "assert-in-reading",
                format!(
                    "{} asserts inside a reading, and only an expectation asserts",
                    place(py, line, self.file.bind(py))?
                ),
            );
        } else if read.before_the_first_expectation() && !in_cleanup(py)? {
            read.fault(
                "assert-before-expectation",
                format!(
                    "{} asserts before the first expectation, and only an expectation asserts",
                    place(py, line, self.file.bind(py))?
                ),
            );
        }
        let held = value.is_truthy()?;
        let mut kept = self.kept.lock().unwrap_or_else(PoisonError::into_inner);
        if !held || !matches!(*kept, Some((_, false))) {
            *kept = Some((index, held));
        }
        drop(kept);
        Ok(held)
    }
}

impl Ran {
    fn take(&self) -> Option<(usize, bool)> {
        self.kept
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .take()
    }
}

/// What a reading of one expectation gave: it held, it did not hold, it
/// raised, or an assert ran inside it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Verdict {
    Holds,
    Fails,
    Raises,
    Refused,
}

/// What the expectations and the steps of one case need from the process that
/// runs it.
pub trait Readers: Sync {
    /// Whether a Python thread other than the main one is alive.
    fn threaded(&self, py: Python<'_>) -> bool;

    /// Forks a reader. Gives `None` in the reader, and the reader's pid in the
    /// process that forked it.
    ///
    /// # Errors
    ///
    /// The operating system refused the fork.
    fn fork(&self, py: Python<'_>) -> io::Result<Option<i32>>;

    /// Waits for the reader `pid` to end, and gives its verdict.
    fn wait(&self, pid: i32) -> Verdict;

    /// Ends a reader, with `verdict`.
    fn end(&self, verdict: Verdict) -> !;

    /// Puts back each hook of the process that the case replaced, and names
    /// the first, or gives `None` when the case replaced none.
    fn replaced(&self, _py: Python<'_>) -> Option<&'static str> {
        None
    }

    /// Hears that the case reached its first expectation.
    fn expecting(&self, _py: Python<'_>) {}
}

/// What the expectations of one case gave.
///
/// The verdict of each that the case read in order, what each that did not hold
/// shows, the verdict of each that a reader read alone, why no reader was forked
/// when the fork failed, and each later expectation that a replay must read
/// alone, since a thread was alive at the first expectation.
#[derive(Debug, Default, PartialEq, Eq)]
pub struct Readings {
    pub in_order: Vec<(usize, bool)>,
    pub shown: Vec<(usize, Shown)>,
    pub alone: Vec<(usize, Verdict)>,
    pub refused: Option<String>,
    pub replay: Vec<usize>,
}

/// What a comparison that did not hold shows: its expected value and its reading.
///
/// Each is the `repr` of its value, beside the name of its class when the two
/// `repr`s are the same. The expected value reads by the rule of its operator.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Shown {
    pub expected: String,
    pub reading: String,
}

#[derive(Default)]
struct State {
    readers: Option<&'static dyn Readers>,
    reader: Option<usize>,
    started: bool,
    reading: bool,
    readings: Readings,
    form: Vec<(&'static str, String)>,
    raised: Vec<(Py<PyAny>, String)>,
    swallowed: Vec<Swallowed>,
}

/// A `with` that swallowed a raise, and the lines that the raise skipped.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Swallowed {
    /// The line of the `with`.
    pub with: usize,
    /// The first line of its body after the line that raised.
    pub skipped: usize,
    /// The last line of its body.
    pub last: usize,
}

/// The hook of the expectations of one loaded module. For each expectation, it
/// holds the range of the expectations of its test. It also holds the body of
/// each `with` of a test, under the line of the `with`.
#[pyclass(frozen)]
struct Read {
    tests: Vec<Range<usize>>,
    checks: Vec<Option<Check>>,
    bodies: HashMap<usize, Body>,
    state: Mutex<State>,
}

impl Read {
    fn state(&self) -> MutexGuard<'_, State> {
        self.state.lock().unwrap_or_else(PoisonError::into_inner)
    }

    fn known(&self, index: usize) -> PyResult<Range<usize>> {
        self.tests.get(index).cloned().ok_or_else(|| {
            PyIndexError::new_err(format!(
                "{READ}({index}): the module has {} expectations",
                self.tests.len()
            ))
        })
    }

    /// Whether a case runs, and has not reached its first expectation.
    fn before_the_first_expectation(&self) -> bool {
        let state = self.state();
        state.readers.is_some() && !state.started
    }

    /// Whether a case runs inside a reading.
    fn inside_a_reading(&self) -> bool {
        let state = self.state();
        state.readers.is_some() && state.reading
    }

    /// The readers, when this process is a reader or a replay.
    fn reader(&self) -> Option<&'static dyn Readers> {
        let state = self.state();
        state.reader.and(state.readers)
    }

    /// Records `text` as a fault of the form, by the name of its `check`, unless
    /// a fault of that check is on record.
    fn fault(&self, check: &'static str, text: String) {
        let mut state = self.state();
        if !state.form.iter().any(|(named, _)| *named == check) {
            state.form.push((check, text));
        }
    }

    /// Forks the readers of a case, and gives whether this process reads the
    /// expectation at `index`. The first call of a case forks a reader for each
    /// later expectation of its test, one at a time, and waits for each. When a
    /// Python thread other than the main one is alive, it forks none, and
    /// records each later expectation for a replay. A reader reads only its own
    /// expectation.
    ///
    /// # Errors
    ///
    /// `IndexError` when no expectation has `index`.
    fn forks(&self, py: Python<'_>, index: usize) -> PyResult<bool> {
        let test = self.known(index)?;
        let mut state = self.state();
        if let Some(reader) = state.reader {
            return Ok(index == reader);
        }
        if state.started {
            return Ok(true);
        }
        state.started = true;
        let Some(readers) = state.readers else {
            return Ok(true);
        };
        // A child of the fork inherits each lock, so none is held across it.
        drop(state);
        readers.expecting(py);
        if readers.threaded(py) {
            self.state().readings.replay = (index + 1..test.end).collect();
            return Ok(true);
        }
        for later in index + 1..test.end {
            match readers.fork(py) {
                Ok(None) => {
                    self.state().reader = Some(later);
                    return Ok(false);
                }
                Ok(Some(pid)) => {
                    let verdict = readers.wait(pid);
                    self.state().readings.alone.push((later, verdict));
                }
                Err(error) => {
                    self.state().readings.refused = Some(format!(
                        "purba cannot fork to read an expectation alone: {error}"
                    ));
                    return Ok(true);
                }
            }
        }
        Ok(true)
    }
}

#[pymethods]
impl Read {
    /// Whether this process reads the expectation at `index`. When it does, a
    /// reading opens.
    ///
    /// # Errors
    ///
    /// `IndexError` when no expectation has `index`.
    fn reads(&self, py: Python<'_>, index: usize) -> PyResult<bool> {
        let reads = self.forks(py, index)?;
        self.state().reading = reads;
        Ok(reads)
    }

    /// Records whether `value` held as the reading of the expectation at
    /// `index`, and closes the reading. A reader ends here, with its verdict.
    /// With one more value in `rest`, the expectation compares `value` and that
    /// value by its own operator, and when they do not hold, it records what it
    /// shows. A side can be `None`, so the right side comes in `rest`, where it
    /// cannot read as a missing argument.
    ///
    /// # Errors
    ///
    /// `IndexError` when no expectation has `index`, `TypeError` when `rest`
    /// holds a value for an expectation that compares no two values, or more
    /// than one, or the error that testing `value` raised.
    ///
    /// # Panics
    ///
    /// In a build with the feature `fault`, when `PURBA_FAULT` is `assert`.
    #[pyo3(signature = (index, value, *rest))]
    fn __call__(
        &self,
        index: usize,
        value: &Bound<'_, PyAny>,
        rest: &Bound<'_, PyTuple>,
    ) -> PyResult<()> {
        self.known(index)?;
        let compared = match rest.len() {
            0 => None,
            1 => {
                let right = rest.get_item(0)?;
                let check = self.checks.get(index).copied().flatten().ok_or_else(|| {
                    PyTypeError::new_err(format!(
                        "{READ}({index}): the expectation compares no two values"
                    ))
                })?;
                Some((check, right))
            }
            _ => {
                return Err(PyTypeError::new_err(format!(
                    "{READ}({index}): an expectation compares at most two values"
                )));
            }
        };
        let held = match &compared {
            Some((check, right)) => holds(check.op, value, right),
            None => value.is_truthy(),
        };
        let (reader, readers) = {
            let mut state = self.state();
            state.reading = false;
            (state.reader, state.readers)
        };
        if let (Some(_), Some(readers)) = (reader, readers) {
            readers.end(match held {
                Ok(true) => Verdict::Holds,
                Ok(false) => Verdict::Fails,
                Err(_) => Verdict::Raises,
            });
        }
        fault();
        let held = held?;
        if !held && let Some((check, right)) = &compared {
            let shown = shown(*check, value, right);
            self.state().readings.shown.push((index, shown));
        }
        self.state().readings.in_order.push((index, held));
        Ok(())
    }

    /// Runs after each statement of a test before its first expectation, at
    /// `line`. Records a hook of the process that the case replaced as the
    /// fault of the form, and the process puts it back.
    fn step(&self, py: Python<'_>, line: usize) {
        let readers = {
            let state = self.state();
            if state.started {
                return;
            }
            state.readers
        };
        if let Some(name) = readers.and_then(|readers| readers.replaced(py)) {
            self.fault(
                "hook-replaced",
                format!("line {line} replaced {name}, and a test keeps purba's hooks"),
            );
        }
    }

    /// Wraps `manager`, an item of the `with` at `line`, so that the case sees
    /// its entry and its cleanup. A `manager` whose type lacks `__enter__` or
    /// `__exit__` stays as it is, so that Python refuses it in its own words.
    fn wrap(slf: &Bound<'_, Self>, line: usize, manager: Bound<'_, PyAny>) -> PyResult<Py<PyAny>> {
        let py = slf.py();
        let kind = manager.get_type();
        let (Ok(enter), Ok(exit)) = (kind.getattr("__enter__"), kind.getattr("__exit__")) else {
            return Ok(manager.unbind());
        };
        Ok(Bound::new(
            py,
            With {
                manager: manager.unbind(),
                enter: enter.unbind(),
                exit: exit.unbind(),
                line,
                read: slf.clone().unbind(),
            },
        )?
        .into_any()
        .unbind())
    }
}

/// An item of a `with` of a test, which enters and leaves its manager. It keeps
/// a raise of the entry or of the cleanup, with the line of the `with`, and a
/// raise that the cleanup swallowed.
#[pyclass(frozen)]
struct With {
    manager: Py<PyAny>,
    enter: Py<PyAny>,
    exit: Py<PyAny>,
    line: usize,
    read: Py<Read>,
}

impl With {
    /// Keeps `err`, which the `what` of the `with` raised, and gives it back.
    fn keep(&self, py: Python<'_>, err: PyErr, what: &str) -> PyErr {
        let text = format!("the {what} of the with at line {} raised {err}", self.line);
        let value = err.value(py).clone().into_any().unbind();
        self.read.get().state().raised.push((value, text));
        err
    }
}

#[pymethods]
impl With {
    fn __enter__(&self, py: Python<'_>) -> PyResult<Py<PyAny>> {
        self.enter
            .call1(py, (&self.manager,))
            .map_err(|err| self.keep(py, err, "entry"))
    }

    fn __exit__(
        &self,
        py: Python<'_>,
        kind: &Bound<'_, PyAny>,
        value: &Bound<'_, PyAny>,
        traceback: &Bound<'_, PyAny>,
    ) -> PyResult<Py<PyAny>> {
        // A reading that raised ends here, inside the `with` that holds it.
        self.read.get().state().reading = false;
        let result = self
            .exit
            .call1(py, (&self.manager, kind, value, traceback))
            .map_err(|err| self.keep(py, err, "cleanup"))?;
        if !value.is_none() && !traceback.is_none() && result.bind(py).is_truthy()? {
            let raised: usize = traceback.getattr("tb_lineno")?.extract()?;
            let read = self.read.get();
            if let Some(body) = read.bodies.get(&self.line)
                && let Some(&skipped) = body.starts.iter().find(|start| **start > raised)
            {
                read.state().swallowed.push(Swallowed {
                    with: self.line,
                    skipped,
                    last: body.last,
                });
            }
        }
        Ok(result)
    }
}

/// A module that `load` made from a rewritten text.
pub struct Loaded<'py> {
    path: PathBuf,
    module: Bound<'py, PyModule>,
    code: Bound<'py, PyAny>,
    ran: Bound<'py, Ran>,
    read: Bound<'py, Read>,
    expectations: Vec<String>,
    lines: Vec<usize>,
    tests: Vec<(String, Range<usize>)>,
}

/// What [`Loaded::watch`] saw of one call.
pub struct Watched<'a, T> {
    /// The result of the call.
    pub result: PyResult<T>,
    /// The text of the last marked assert inside the call that did not hold, or
    /// else of the last one that ran, beside whether it held.
    pub mark: Option<(&'a str, bool)>,
    /// What the expectations gave.
    pub readings: Readings,
    /// The first fault of each check of the form, each beside the name of its
    /// check, in the order they came: an assert before the first expectation,
    /// an assert inside a reading, or a hook that the case replaced.
    pub form: Vec<(&'static str, String)>,
    /// Each exception that the entry or the cleanup of a `with` raised, in the
    /// order they raised, beside the text that names that `with`.
    pub raised: Vec<(Py<PyAny>, String)>,
    /// Each `with` that swallowed a raise and skipped a line.
    pub swallowed: Vec<Swallowed>,
}

impl<'py> Loaded<'py> {
    /// The module, whose namespace stays in Python.
    #[must_use]
    pub const fn module(&self) -> &Bound<'py, PyModule> {
        &self.module
    }

    /// The code object of the module's text, which holds the code of each
    /// function that the module defines.
    #[must_use]
    pub const fn code(&self) -> &Bound<'py, PyAny> {
        &self.code
    }

    /// The text of the expectation at `index`, as its reading records it.
    #[must_use]
    pub fn expectation(&self, index: usize) -> Option<&str> {
        self.expectations.get(index).map(String::as_str)
    }

    /// Whether `test` states an expectation.
    #[must_use]
    pub fn states_expectation(&self, test: &str) -> bool {
        self.tests
            .iter()
            .any(|(name, range)| name == test && !range.is_empty())
    }

    /// The line of the first expectation of `test`, when it states one.
    #[must_use]
    pub fn first_expectation(&self, test: &str) -> Option<usize> {
        self.tests
            .iter()
            .find(|(name, _)| name == test)
            .and_then(|(_, range)| self.lines.get(range.start).copied())
            .filter(|_| self.states_expectation(test))
    }

    /// Runs `call` with no assert and no reading recorded, with `readers` for
    /// the expectations. Gives its result, the last mark, the readings, and
    /// what the steps, the marks and the `with`s of the case saw.
    ///
    /// When `call` bound a name of a hook to anything but the hook, the result
    /// is the error that [`load`] gives such a module, and the hook is back.
    /// When this process is a reader and `call` ends, the reader ends with the
    /// verdict that its expectation raised.
    pub fn watch<T>(
        &self,
        readers: &'static dyn Readers,
        call: impl FnOnce() -> PyResult<T>,
    ) -> Watched<'_, T> {
        self.observe(readers, None, call)
    }

    /// Runs `call` as a replay that reads only the expectation at `index`, and
    /// ends the process with its verdict through `readers`. When `call` ends
    /// before that expectation, the replay ends with the verdict that it
    /// raised.
    pub fn replay<T>(
        &self,
        readers: &'static dyn Readers,
        index: usize,
        call: impl FnOnce() -> PyResult<T>,
    ) -> Watched<'_, T> {
        self.observe(readers, Some(index), call)
    }

    fn observe<T>(
        &self,
        readers: &'static dyn Readers,
        reader: Option<usize>,
        call: impl FnOnce() -> PyResult<T>,
    ) -> Watched<'_, T> {
        let ran = self.ran.get();
        let read = self.read.get();
        let _ = ran.take();
        *read.state() = State {
            readers: Some(readers),
            reader,
            started: reader.is_some(),
            ..State::default()
        };
        let result = match (
            call(),
            check(&self.module, &self.ran, &self.read, &self.path),
        ) {
            (result, Ok(())) => result,
            (_, Err(err)) => bind(&self.module, &self.ran, &self.read).and(Err(err)),
        };
        let state = std::mem::take(&mut *read.state());
        if state.reader.is_some() {
            readers.end(Verdict::Raises);
        }
        let mark = ran
            .take()
            .and_then(|(index, held)| Some((ran.asserts.get(index)?.as_str(), held)));
        Watched {
            result,
            mark,
            readings: state.readings,
            form: state.form,
            raised: state.raised,
            swallowed: state.swallowed,
        }
    }
}

/// Each name of a hook, beside the hook that it holds.
fn hooks<'py>(
    ran: &Bound<'py, Ran>,
    read: &Bound<'py, Read>,
) -> PyResult<[(&'static str, Bound<'py, PyAny>); 5]> {
    Ok([
        (RAN, ran.as_any().clone()),
        (READ, read.as_any().clone()),
        (READS, read.getattr("reads")?),
        (STEP, read.getattr("step")?),
        (WITH, read.getattr("wrap")?),
    ])
}

/// Gives `module` each hook.
fn bind(
    module: &Bound<'_, PyModule>,
    ran: &Bound<'_, Ran>,
    read: &Bound<'_, Read>,
) -> PyResult<()> {
    for (name, hook) in hooks(ran, read)? {
        module.setattr(name, hook)?;
    }
    Ok(())
}

/// Loads the text of `rewrite` as the module at `path`, in this order:
///
/// 1. names the module after the file of `path`, without `.py`, and sets its
///    `__file__` to `path`;
/// 2. gives it `_purba_ran`, which holds the text and the line of each assert
///    of `rewrite` at the index that its mark records, and `_purba_read`,
///    `_purba_reads`, `_purba_step` and `_purba_with`, the hooks of its
///    expectations and of its tests;
/// 3. puts it in `sys.modules` under that name;
/// 4. compiles the text, checks that no function holds a name of a hook as a
///    local name, and runs it in the module's namespace;
/// 5. checks that each name of a hook still holds its hook;
/// 6. when step 4 or 5 raises or panics, puts back what `sys.modules` held
///    under that name, or takes the name out when it held nothing.
///
/// # Errors
///
/// The error that step 4 raises, such as one from `compile()` or the module's
/// body. `NameError` when the body bound a name of a hook to anything but the
/// hook.
///
/// # Panics
///
/// A panic of step 4 or 5 goes on after step 6.
pub fn load<'py>(py: Python<'py>, path: &Path, rewrite: &Rewrite) -> PyResult<Loaded<'py>> {
    let text = rewrite.text.as_str();
    let name = path.file_stem().unwrap_or_default().to_string_lossy();
    let module = PyModule::new(py, &name)?;
    module.setattr("__file__", path.as_os_str())?;
    let mut tests = vec![0..0; rewrite.expectations.len()];
    for (_, range) in &rewrite.tests {
        for index in range.clone() {
            tests[index] = range.clone();
        }
    }
    let read = Bound::new(
        py,
        Read {
            tests,
            checks: rewrite.checks.clone(),
            bodies: rewrite.bodies.clone(),
            state: Mutex::new(State::default()),
        },
    )?;
    let ran = Bound::new(
        py,
        Ran {
            asserts: rewrite.asserts.clone(),
            marks: rewrite.marks.clone(),
            read: read.clone().unbind(),
            file: path.as_os_str().into_pyobject(py)?.into_any().unbind(),
            kept: Mutex::new(None),
        },
    )?;
    bind(&module, &ran, &read)?;
    let modules = py.import("sys")?.getattr("modules")?;
    let before = modules.get_item(&*name).ok();
    modules.set_item(&*name, &module)?;
    let ran_body = panic::catch_unwind(AssertUnwindSafe(|| {
        let code = run(py, path, text, &module)?;
        check(&module, &ran, &read, path).map(|()| code)
    }));
    if !matches!(ran_body, Ok(Ok(_))) {
        let _ = before.map_or_else(
            || modules.del_item(&*name),
            |before| modules.set_item(&*name, before),
        );
    }
    let code = match ran_body {
        Ok(Ok(code)) => code,
        Ok(Err(err)) => return Err(err),
        Err(payload) => panic::resume_unwind(payload),
    };
    Ok(Loaded {
        path: path.to_owned(),
        module,
        code,
        ran,
        read,
        expectations: rewrite.expectations.clone(),
        lines: rewrite.lines.clone(),
        tests: rewrite.tests.clone(),
    })
}

/// Compiles `text`, runs it in the namespace of `module`, and gives its code.
fn run<'py>(
    py: Python<'py>,
    path: &Path,
    text: &str,
    module: &Bound<'py, PyModule>,
) -> PyResult<Bound<'py, PyAny>> {
    let builtins = py.import("builtins")?;
    let code = builtins
        .getattr("compile")?
        .call1((text, path.as_os_str(), "exec"))?;
    check_locals(&code, path)?;
    builtins.getattr("exec")?.call1((&code, module.dict()))?;
    Ok(code)
}

/// Checks that each name of a hook in the namespace of `module` holds its hook.
///
/// # Errors
///
/// `NameError` that names the file at `path` when one does not.
fn check(
    module: &Bound<'_, PyModule>,
    ran: &Bound<'_, Ran>,
    read: &Bound<'_, Read>,
    path: &Path,
) -> PyResult<()> {
    let names = module.dict();
    for (name, hook) in hooks(ran, read)? {
        // A bound method is a new object at each lookup, so it holds when it
        // equals the hook.
        let holds = match names.get_item(name)? {
            Some(held) if name == RAN || name == READ => held.is(&hook),
            Some(held) => held.eq(&hook)?,
            None => false,
        };
        if !holds {
            return Err(bound(path, name));
        }
    }
    Ok(())
}

/// Checks that no code object in `code`, at any depth, holds a name of a hook
/// among the local names of its function. Such a name hides the hook from each
/// assert of that function.
///
/// # Errors
///
/// `NameError` that names the file at `path` when one does.
fn check_locals(code: &Bound<'_, PyAny>, path: &Path) -> PyResult<()> {
    let code_type = code.get_type();
    let mut waiting = vec![code.clone()];
    while let Some(current) = waiting.pop() {
        for names in ["co_varnames", "co_cellvars", "co_freevars"] {
            let names = current.getattr(names)?;
            for hook in [RAN, READ, READS, STEP, WITH] {
                if names.contains(hook)? {
                    return Err(bound(path, hook));
                }
            }
        }
        for constant in current.getattr("co_consts")?.try_iter()? {
            let constant = constant?;
            if constant.is_instance(&code_type)? {
                waiting.push(constant);
            }
        }
    }
    Ok(())
}

/// The error of a module at `path` that binds `name`, the name of a hook.
fn bound(path: &Path, name: &str) -> PyErr {
    let hook = match name {
        RAN => "assert hook",
        STEP => "step hook",
        WITH => "with hook",
        _ => "expectation hook",
    };
    PyNameError::new_err(format!(
        "{} binds {name}, the name of purba's {hook}",
        path.display()
    ))
}

/// Adds to `edits` a step after each simple statement of each test of
/// `by_test` that is no expectation, and a wrap around each item of each of
/// its `with`s. Gives the body of each `with`, under its line.
fn watch_each_test(
    module: &Module,
    by_test: &[(&StmtFunctionDef, Vec<&StmtAssert>)],
    expected: &HashSet<*const StmtAssert>,
    edits: &mut Vec<Edit>,
) -> HashMap<usize, Body> {
    let line = |offset: TextSize| module.position(offset).line();
    let mut bodies = HashMap::new();
    for (function, _) in by_test {
        for statement in statements(&function.body) {
            let at = line(statement.start());
            if let Stmt::With(with) = statement {
                for item in &with.items {
                    let context = item.context_expr.range();
                    edits.push(Edit::insert(
                        context.start().to_usize(),
                        format!("{WITH}({at}, "),
                    ));
                    edits.push(Edit::insert(context.end().to_usize(), ")".to_owned()));
                }
                bodies.insert(
                    at,
                    Body {
                        starts: statements(&with.body)
                            .iter()
                            .map(|inner| line(inner.start()))
                            .collect(),
                        last: with.body.last().map_or(at, |last| line(last.end())),
                    },
                );
            }
            let is_expectation = matches!(
                statement,
                Stmt::Assert(assert) if expected.contains(&std::ptr::from_ref(assert))
            );
            if is_simple(statement) && !is_expectation {
                edits.push(Edit::insert(
                    statement.end().to_usize(),
                    format!("; {STEP}({at})"),
                ));
            }
        }
    }
    bodies
}
