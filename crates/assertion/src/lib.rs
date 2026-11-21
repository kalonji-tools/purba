//! Assertion: rewrites the asserts of a module, and loads the rewritten module.
//! An assert at the end of a test's body, or at the end of the body of a `with`
//! just before such asserts, is an expectation. A tuple assert is never an
//! expectation. An expectation becomes a reading that execution compares.
//! Every other assert, except a tuple assert, keeps a mark that records that it
//! ran, and whether it held. A step follows each statement of a test before its first expectation,
//! and each item of each `with` of a test is wrapped, so that the case sees its
//! hooks and its `with`s.

use purba_parsing::{Module, Reads, Shape};
use pyo3::basic::CompareOp;
use pyo3::exceptions::{PyImportError, PyIndexError, PyNameError};
use pyo3::types::{
    PyAnyMethods, PyDict, PyDictMethods, PyModule, PyModuleMethods, PyTuple, PyTupleMethods,
    PyTypeMethods,
};
use pyo3::{Bound, IntoPyObject, Py, PyAny, PyErr, PyResult, Python, pyclass, pymethods};
use ruff_python_ast::statement_visitor::{StatementVisitor, walk_stmt};
use ruff_python_ast::visitor::{Visitor, walk_expr};
use ruff_python_ast::{
    CmpOp, ExceptHandler, Expr, ExprContext, Parameters, Stmt, StmtAssert, StmtFunctionDef, StmtTry,
};
use ruff_text_size::{Ranged, TextRange, TextSize};
use std::collections::{HashMap, HashSet};
use std::io;
use std::iter;
use std::ops::{Range, RangeInclusive};
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
/// The name that the one statement of a `try` of one path calls when it did
/// not raise, with the index of the `try`.
const MISSED: &str = "_purba_missed";
/// The name of the parameter of the function that gives each item of an
/// `all()` or an `any()` with the parts of its check.
const ITEMS: &str = "_purba_items";
/// The name that holds each item of an `all()` or an `any()` before its target
/// unpacks it, so that the rewrite writes the target once.
const ITEM: &str = "_purba_item";

/// The rewritten text of one module.
///
/// It holds the text and the line of each assert that it marks, and the text,
/// the line and the names that it reads of each expectation of each test. It
/// holds the statements of the body of each `with` of a test.
pub struct Rewrite {
    text: String,
    asserts: Vec<String>,
    marks: Vec<usize>,
    expectations: Vec<String>,
    checks: Vec<Option<Check>>,
    lines: Vec<usize>,
    names: Vec<Vec<String>>,
    tests: Vec<(String, Range<usize>)>,
    bodies: HashMap<usize, Body>,
    missed: Vec<String>,
    tries: Vec<(String, Vec<Try>)>,
}

/// A `try` of a test: its lines, and the lines of its `except` when it is a
/// `try` of one path, which holds one simple statement and one
/// `except SomeError:`, with no `else` and no `finally`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Try {
    pub lines: RangeInclusive<usize>,
    pub except: Option<RangeInclusive<usize>>,
}

/// How an expectation checks its reading: by the operator of a comparison of
/// two values, with whether its expected value is the left side, since the left
/// is a literal and the right is not; by one of the four reads, with the text
/// of the type of `isinstance`; or by the check of each item of an `all()` or
/// an `any()`.
#[derive(Clone, Debug)]
enum Check {
    Compare { op: CmpOp, expected_left: bool },
    Read { read: Reads, kind: String },
    Each(Box<Items>),
}

/// How an `all()` or an `any()` checks each item: by the check of one item,
/// with the text of that check, of its side that reads the item, and of the
/// target. `bare` says that this side is the target itself. `empty` holds the words of an `all()`
/// whose reading holds no item.
#[derive(Clone, Debug)]
struct Items {
    any: bool,
    item: Check,
    label: String,
    side: String,
    target: String,
    bare: bool,
    empty: String,
}

/// The reading of `check`, of its parts `first` and `second`: the right side of
/// a comparison whose expected value is the left, and else the first part.
fn reading_side<T>(check: &Check, first: T, second: Option<T>) -> T {
    match (check, second) {
        (
            Check::Compare {
                expected_left: true,
                ..
            },
            Some(right),
        ) => right,
        _ => first,
    }
}

/// The text of `expr`, with each run of whitespace as one space.
fn spaced(expr: &Expr, source: &str) -> String {
    source[expr.range()]
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}

/// How `check`, one check that parsing names, checks its reading.
fn checked(check: purba_parsing::Check<'_>, source: &str) -> Check {
    match check {
        purba_parsing::Check::Compare { op, left, right } => Check::Compare {
            op,
            expected_left: purba_parsing::is_literal(left) && !purba_parsing::is_literal(right),
        },
        purba_parsing::Check::Read { read, other, .. } => Check::Read {
            read,
            kind: other.map_or_else(String::new, |other| spaced(other, source)),
        },
    }
}

/// How `test`, the condition of the assert at `line`, checks its reading, or
/// `None` when parsing names no check in it.
fn check_of(test: &Expr, source: &str, line: usize) -> Option<Check> {
    match purba_parsing::shape(test) {
        Shape::Check(check) => Some(checked(check, source)),
        Shape::Each {
            any,
            check,
            item: label,
            target,
            reading: over,
        } => {
            let item = checked(check, source);
            let over = spaced(over, source);
            let (first, second) = check.values();
            let side = reading_side(&item, first, second);
            let (side, target) = (spaced(side, source), spaced(target, source));
            Some(Check::Each(Box::new(Items {
                any,
                item,
                label: spaced(label, source),
                bare: side == target,
                side,
                target,
                empty: format!("line {line} checks each item of {over}, and {over} holds no item"),
            })))
        }
        Shape::OtherEach { .. } | Shape::Other => None,
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

/// The expectations of `function`: the asserts at the end of its body, except
/// each tuple assert, which the rewrite leaves as written.
fn expectations_of(function: &StmtFunctionDef) -> Vec<&StmtAssert> {
    purba_parsing::ending(&function.body)
        .into_iter()
        .filter(|assert| !of_a_tuple(assert))
        .collect()
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
        pending.extend(
            purba_parsing::blocks(statement)
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
/// that operator. An expectation `assert all(c for t in r)`, or `any(…)`, of
/// one check `c` becomes `_purba_reads(i) and _purba_read(i, (r), f)`, where
/// `f` gives each item of `r` with the parts of `c`, and the hook checks each
/// item by `c`. Each keeps each line break of the assert inside the call, so
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
        .map(|function| (function, expectations_of(function)))
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
    let (missed, tries) = mark_each_try(module, &by_test, &mut edits);
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
            .map(|assert| check_of(&assert.test, source, line(assert.start())))
            .collect(),
        lines: all.iter().map(|assert| line(assert.start())).collect(),
        names: by_test
            .iter()
            .flat_map(|(function, asserts)| {
                let parameters = parameters_of(&function.parameters);
                asserts.iter().map(move |assert| {
                    read_names(&assert.test)
                        .into_iter()
                        .filter(|name| !parameters.contains(name.as_str()))
                        .collect()
                })
            })
            .collect(),
        tests: ranges,
        bodies,
        missed,
        tries,
    }
}

/// The edit that makes `assert`, the expectation at `index`, a call of the
/// hook. A comparison of two values by one operator hands the hook each side,
/// and one of the four reads hands its reading and the type or the argument
/// that it takes. An `all()` or an `any()` of one check hands its reading, and
/// a function of that reading that gives each item with the parts of its
/// check.
fn read_edit(index: usize, assert: &StmtAssert, source: &str) -> Edit {
    let whole = assert.range();
    let mut text = format!("{READS}({index}) and {READ}({index}, ");
    let shape = purba_parsing::shape(&assert.test);
    if let Shape::Each {
        check,
        target,
        reading,
        ..
    } = shape
    {
        let (first, second) = check.values();
        push_side(&mut text, source, reading);
        text.push_str(", lambda ");
        text.push_str(ITEMS);
        text.push_str(": ((");
        text.push_str(ITEM);
        for part in iter::once(first).chain(second) {
            text.push_str(", ");
            push_side(&mut text, source, part);
        }
        text.push_str(") for ");
        text.push_str(ITEM);
        text.push_str(" in ");
        text.push_str(ITEMS);
        text.push_str(" for ");
        push_side(&mut text, source, target);
        text.push_str(" in (");
        text.push_str(ITEM);
        text.push_str(",))");
        let breaks = source[whole].matches('\n').count();
        for _ in text.matches('\n').count()..breaks {
            text.push('\n');
        }
        text.push(')');
        return Edit {
            start: whole.start().to_usize(),
            end: whole.end().to_usize(),
            text,
        };
    }
    let mut copied = whole.start();
    if let Shape::Check(check) = shape {
        let (first, second) = check.values();
        for (side, start) in iter::once((first, "")).chain(second.map(|second| (second, ", "))) {
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

/// Whether `value`, and `other` when the expectation hands one, hold by
/// `check`, as Python decides. With no check, `value` holds when it is true.
/// It is `None` when the values do not match the check. Then no check or `not`
/// gets two values, a check of two values gets one, or the check is of an
/// `all()` or an `any()`.
fn held(
    check: Option<&Check>,
    value: &Bound<'_, PyAny>,
    other: Option<&Bound<'_, PyAny>>,
) -> PyResult<Option<bool>> {
    let read = |method: &str, argument: &Bound<'_, PyAny>| {
        value.call_method1(method, (argument,))?.is_truthy()
    };
    match (check, other) {
        (None, None) => value.is_truthy(),
        (Some(Check::Compare { op, .. }), Some(right)) => holds(*op, value, right),
        (
            Some(Check::Read {
                read: Reads::Not, ..
            }),
            None,
        ) => value.is_truthy().map(|truthy| !truthy),
        (
            Some(Check::Read {
                read: Reads::Instance,
                ..
            }),
            Some(kind),
        ) => value.is_instance(kind),
        (
            Some(Check::Read {
                read: Reads::Start, ..
            }),
            Some(argument),
        ) => read("startswith", argument),
        (
            Some(Check::Read {
                read: Reads::End, ..
            }),
            Some(argument),
        ) => read("endswith", argument),
        _ => return Ok(None),
    }
    .map(Some)
}

/// Whether `value` is an iterator that one reading empties: its type has
/// `__next__`, `iter(value)` is `value`, and it is no `file`, an `io.IOBase`.
fn is_iterator(value: &Bound<'_, PyAny>, file: &Bound<'_, PyAny>) -> bool {
    value.get_type().hasattr("__next__").unwrap_or(false)
        && !value.is_instance(file).unwrap_or(true)
        && value
            .try_iter()
            .is_ok_and(|iterator| iterator.as_any().is(value))
}

/// The name of each parameter of `parameters`.
fn parameters_of(parameters: &Parameters) -> HashSet<&str> {
    parameters
        .posonlyargs
        .iter()
        .chain(&parameters.args)
        .chain(&parameters.kwonlyargs)
        .map(|parameter| parameter.parameter.name.as_str())
        .chain(parameters.vararg.iter().map(|vararg| vararg.name.as_str()))
        .chain(parameters.kwarg.iter().map(|kwarg| kwarg.name.as_str()))
        .collect()
}

/// Each name that `test` reads and binds nowhere in it, such as a target of a
/// comprehension or a parameter of a `lambda` in it, in source order, once.
fn read_names(test: &Expr) -> Vec<String> {
    let mut names = Names::default();
    names.visit_expr(test);
    while let Some(expr) = names.pending.pop() {
        walk_expr(&mut names, expr);
    }
    let Names {
        mut read, bound, ..
    } = names;
    read.sort_by_key(|(offset, _)| *offset);
    let mut seen = HashSet::new();
    read.into_iter()
        .filter(|(_, name)| !bound.contains(name) && seen.insert(*name))
        .map(|(_, name)| name.to_owned())
        .collect()
}

/// The names of an expression: each name it reads, by its offset, and each
/// name it binds. Each visit pushes its expression, and [`read_names`] walks
/// each expression it pushed, so a deep expression does not overflow the
/// stack.
#[derive(Default)]
struct Names<'a> {
    read: Vec<(TextSize, &'a str)>,
    bound: HashSet<&'a str>,
    pending: Vec<&'a Expr>,
}

impl<'a> Visitor<'a> for Names<'a> {
    fn visit_expr(&mut self, expr: &'a Expr) {
        match expr {
            Expr::Name(name) if name.ctx == ExprContext::Load => {
                self.read.push((name.start(), name.id.as_str()));
            }
            Expr::Name(name) => {
                self.bound.insert(name.id.as_str());
            }
            Expr::Lambda(lambda) => {
                if let Some(parameters) = &lambda.parameters {
                    self.bound.extend(parameters_of(parameters));
                }
            }
            _ => {}
        }
        self.pending.push(expr);
    }
}

/// The `repr` of `value`, or the words of its raise.
fn text_of(value: &Bound<'_, PyAny>) -> Result<String, String> {
    value
        .repr()
        .map(|text| text.to_string())
        .map_err(|err| format!("(repr raised {err})"))
}

/// The name of the class of `value`.
fn class_of(value: &Bound<'_, PyAny>) -> String {
    value
        .get_type()
        .name()
        .map_or_default(|name| name.to_string())
}

/// What an expectation that checks `value`, and `other` when it hands one, by
/// `check` shows.
fn shown(
    check: &Check,
    value: &Bound<'_, PyAny>,
    other: Option<&Bound<'_, PyAny>>,
) -> Option<Shown> {
    let (op, expected_left, right) = match (check, other) {
        (Check::Compare { op, expected_left }, Some(right)) => (*op, *expected_left, right),
        (Check::Read { read, kind }, other) => {
            let argument = || {
                other.map_or_else(String::new, |other| {
                    text_of(other).unwrap_or_else(|text| text)
                })
            };
            let reading = text_of(value).unwrap_or_else(|text| text);
            let (expected, reading) = match read {
                Reads::Not => ("empty or false".to_owned(), reading),
                Reads::Instance => (
                    format!("an instance of {kind}"),
                    format!("{reading} ({})", class_of(value)),
                ),
                Reads::Start => (format!("to start with {}", argument()), reading),
                Reads::End => (format!("to end with {}", argument()), reading),
            };
            return Some(Shown::Values { expected, reading });
        }
        (Check::Compare { .. }, None) | (Check::Each(_), _) => return None,
    };
    let (expected, reading) = if expected_left {
        (value, right)
    } else {
        (right, value)
    };
    let (mut expected_text, mut reading_text) = (text_of(expected), text_of(reading));
    if let (Ok(expected_text), Ok(reading_text)) = (&mut expected_text, &mut reading_text)
        && expected_text == reading_text
    {
        for (text, value) in [(expected_text, expected), (reading_text, reading)] {
            text.push_str(" (");
            text.push_str(&class_of(value));
            text.push(')');
        }
    }
    Some(Shown::Values {
        expected: rule(op, expected_left, expected_text.unwrap_or_else(|text| text)),
        reading: reading_text.unwrap_or_else(|text| text),
    })
}

/// One item of an `all()` or an `any()`: its index, whether it holds the
/// check, the item, and the value of the side that the check reads.
type Row<'py> = (usize, bool, Bound<'py, PyAny>, Bound<'py, PyAny>);

/// Whether the `all()` or the `any()` of `items` holds over `reading`, the row
/// of each item that a failure names, and how many items it read. `make` gives
/// each item of `reading` with the parts of its check. An `any()` stops at its
/// first item that holds, and an `all()` at its first item that breaks, as
/// Python's own do. An `any()` of no item fails, and an `all()` of no item
/// holds. It takes no `repr`, so a reader ends with its verdict before any.
/// It gives `None` when the parts of an item do not match the check of an item.
fn read_each<'py>(
    items: &Items,
    reading: &Bound<'py, PyAny>,
    make: &Bound<'py, PyAny>,
) -> PyResult<Option<(bool, Vec<Row<'py>>, usize)>> {
    let row_of = |index: usize, row: Bound<'py, PyAny>| -> PyResult<Option<Row<'py>>> {
        let (first, second) = match row.len()? {
            2 => (row.get_item(1)?, None),
            3 => (row.get_item(1)?, Some(row.get_item(2)?)),
            _ => return Ok(None),
        };
        let Some(holds) = held(Some(&items.item), &first, second.as_ref())? else {
            return Ok(None);
        };
        let value = reading_side(&items.item, first, second);
        Ok(Some((index, holds, row.get_item(0)?, value)))
    };
    let mut rows: Vec<Row<'py>> = Vec::new();
    let mut read = 0;
    for (index, row) in make.call1((reading,))?.try_iter()?.enumerate() {
        let Some(row) = row_of(index, row?)? else {
            return Ok(None);
        };
        read += 1;
        let holds = row.1;
        if items.any || !holds {
            rows.push(row);
        }
        if items.any == holds {
            break;
        }
    }
    // Each reading stops at the row that decides it, so the last row gives the
    // verdict, and a reading of no row gives Python's verdict of no item.
    let held = rows.last().map_or(!items.any, |row| row.1);
    Ok(Some((held, rows, read)))
}

/// What a failed `all()` or `any()` of `items` shows from its `rows`, after it
/// read `total` items: a heading, then a line for the first item that breaks
/// the check of an `all()`, or for each item of an `any()`.
fn shown_items(items: &Items, rows: &[Row<'_>], total: usize) -> Shown {
    let heading = if items.any {
        let noun = if total == 1 { "item" } else { "items" };
        format!("none of {total} {noun} holds  {}", items.label)
    } else {
        format!("the first item that breaks  {}", items.label)
    };
    let lines = rows
        .iter()
        .map(|(index, _, item, value)| {
            let item = text_of(item).unwrap_or_else(|text| text);
            if items.bare {
                format!("[{index}]  {item}")
            } else {
                format!(
                    "[{index}]  {}  {}     {}  {item}",
                    items.side,
                    text_of(value).unwrap_or_else(|text| text),
                    items.target
                )
            }
        })
        .collect();
    Shown::Items {
        heading,
        items: lines,
    }
}

/// The text of `expected` under the rule of `op`, read from the side of the
/// reading, where `expected_left` says whether the expected value is the left
/// side: `== 2` shows `2`, `< 2` shows `< 2`, `2 < x` shows `> 2`, and
/// `'z' in x` shows `to contain 'z'`.
fn rule(op: CmpOp, expected_left: bool, expected: String) -> String {
    let words = match (op, expected_left) {
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
    Ok(match reach(py, file)? {
        None => format!("line {line}"),
        Some((reaches, name)) => format!("line {reaches} reaches line {line} of {name}, which"),
    })
}

/// The words of an assert at `line` that ran in the act, where `file` is the
/// module's file, at the place that [`place`] names.
fn in_the_act(py: Python<'_>, line: usize, file: &Bound<'_, PyAny>) -> PyResult<String> {
    const RULE: &str = "one test checks one state; check this state in a test of its own";
    Ok(match reach(py, file)? {
        None => format!("line {line}: an assert ran in the act; {RULE}"),
        Some((reaches, name)) => format!(
            "line {reaches} reaches line {line} of {name}, which ran an assert in the act; {RULE}"
        ),
    })
}

/// The line that the test runs and the function of the innermost frame, or
/// `None` when the innermost frame is the test's own. The test is the
/// outermost frame whose code comes from `file`, the module's file.
fn reach(py: Python<'_>, file: &Bound<'_, PyAny>) -> PyResult<Option<(usize, String)>> {
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
        return Ok(None);
    };
    let reaches: usize = test.getattr("f_lineno")?.extract()?;
    let name: String = innermost.getattr("f_code")?.getattr("co_name")?.extract()?;
    Ok(Some((reaches, name)))
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
                in_the_act(py, line, self.file.bind(py))?,
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

    /// Names what the case replaced: a hook of the process, which it puts back,
    /// the warning filters, or the events of the watch of the paths. Gives
    /// `None` when the case replaced none.
    fn replaced(&self, _py: Python<'_>) -> Option<Replaced> {
        None
    }

    /// Hears that the case reached its first expectation.
    fn expecting(&self, _py: Python<'_>) {}
}

/// What a case or a module replaced that a test keeps: the first hook of the
/// process that it replaced, by its name, the warning filters, or the events
/// that the watch of the paths set.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Replaced {
    Hook(&'static str),
    Filters,
    Events,
}

impl Replaced {
    /// The name of what was replaced.
    #[must_use]
    pub const fn what(self) -> &'static str {
        match self {
            Self::Hook(name) => name,
            Self::Filters => "the warning filters",
            Self::Events => "the events of sys.monitoring that purba watches with",
        }
    }

    /// The rule that a replacement breaks.
    #[must_use]
    pub const fn rule(self) -> &'static str {
        match self {
            Self::Hook(_) | Self::Events => "a test keeps purba's hooks",
            Self::Filters => "a test keeps the filters that the run began with",
        }
    }
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

/// What an expectation that did not hold shows.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum Shown {
    /// A check of one value: its expected value and its reading. Each is the
    /// `repr` of its value, beside the name of its class when the two `repr`s
    /// are the same. The expected value reads by the rule of its operator.
    Values { expected: String, reading: String },
    /// An `all()` or an `any()`: a heading, and a line for each item that a
    /// failure names.
    Items { heading: String, items: Vec<String> },
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
    missed: Vec<String>,
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
    lines: Vec<usize>,
    names: Vec<Vec<String>>,
    bodies: HashMap<usize, Body>,
    missed: Vec<String>,
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

    /// Closes the reading of the expectation at `index`, whose call handed
    /// values that do not match its check. A reader ends with `Raises`. Any
    /// other process panics, since the rewrite writes no such call.
    fn mismatched(&self, index: usize) -> ! {
        self.state().reading = false;
        if let Some(readers) = self.reader() {
            readers.end(Verdict::Raises);
        }
        panic!("{READ}({index}): the values do not match the check of the expectation");
    }

    /// Records `text` as a fault of the form, by the name of its `check`, unless
    /// a fault of that check is on record.
    fn fault(&self, check: &'static str, text: String) {
        let mut state = self.state();
        if !state.form.iter().any(|(named, _)| *named == check) {
            state.form.push((check, text));
        }
    }

    /// Records a fault of the form when a name that the expectation at `index`
    /// reads holds an iterator in the frame of the test, while a case runs. A
    /// parameter of the test is no such name. The fault names the first one.
    ///
    /// # Errors
    ///
    /// The error of reading the frame of the test.
    fn iterators(&self, py: Python<'_>, index: usize) -> PyResult<()> {
        let names = match self.names.get(index) {
            Some(names) if !names.is_empty() && self.state().readers.is_some() => names,
            _ => return Ok(()),
        };
        let locals = py
            .import("sys")?
            .getattr("_getframe")?
            .call0()?
            .getattr("f_locals")?;
        let file = py.import("io")?.getattr("IOBase")?;
        if let Some(name) = names.iter().find(|name| {
            locals
                .get_item(name.as_str())
                .is_ok_and(|value| is_iterator(&value, &file))
        }) {
            self.fault(
                "iterator-read",
                format!(
                    "line {} reads {name}, an iterator that one reading empties; \
                     make it a list in the act",
                    self.lines.get(index).copied().unwrap_or_default()
                ),
            );
        }
        Ok(())
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
        if reads {
            self.iterators(py, index)?;
        }
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
    /// `IndexError` when no expectation has `index`, or the error that the check
    /// of `value` raised.
    ///
    /// # Panics
    ///
    /// It panics when the values do not match the check of the expectation. A
    /// reader ends with `Raises` instead. It also panics in a build with the
    /// feature `fault`, when `PURBA_FAULT` is `assert`.
    #[pyo3(signature = (index, value, *rest))]
    fn __call__(
        &self,
        index: usize,
        value: &Bound<'_, PyAny>,
        rest: &Bound<'_, PyTuple>,
    ) -> PyResult<()> {
        self.known(index)?;
        let check = self.checks.get(index).cloned().flatten();
        let other = match rest.len() {
            0 => None,
            1 => Some(rest.get_item(0)?),
            _ => self.mismatched(index),
        };
        let (held, rows, read) = match (&check, &other) {
            (Some(Check::Each(items)), Some(make)) => match read_each(items, value, make) {
                Ok(Some((held, rows, read))) => {
                    if read == 0 && !items.any {
                        self.fault("no-item", items.empty.clone());
                    }
                    (Ok(Some(held)), rows, read)
                }
                Ok(None) => (Ok(None), Vec::new(), 0),
                Err(err) => (Err(err), Vec::new(), 0),
            },
            _ => (held(check.as_ref(), value, other.as_ref()), Vec::new(), 0),
        };
        let (reader, readers) = {
            let mut state = self.state();
            state.reading = false;
            (state.reader, state.readers)
        };
        if let (Some(_), Some(readers)) = (reader, readers) {
            readers.end(match held {
                Ok(Some(true)) => Verdict::Holds,
                Ok(Some(false)) => Verdict::Fails,
                Ok(None) | Err(_) => Verdict::Raises,
            });
        }
        fault();
        let Some(held) = held? else {
            self.mismatched(index)
        };
        let shown = |check: &Check| match check {
            Check::Each(items) => Some(shown_items(items, &rows, read)),
            check => shown(check, value, other.as_ref()),
        };
        if !held && let Some(shown) = check.as_ref().and_then(shown) {
            self.state().readings.shown.push((index, shown));
        }
        self.state().readings.in_order.push((index, held));
        Ok(())
    }

    /// Records the `try` at `index` as missed: its one statement did not raise.
    ///
    /// # Errors
    ///
    /// `IndexError` when no `try` of one path has `index`.
    fn missed(&self, index: usize) -> PyResult<()> {
        let words = self.missed.get(index).ok_or_else(|| {
            PyIndexError::new_err(format!(
                "{MISSED}({index}): the module has {} tries of one path",
                self.missed.len()
            ))
        })?;
        self.state().missed.push(words.clone());
        Ok(())
    }

    /// Runs after each statement of a test before its first expectation, at
    /// `line`. Records what the case replaced as the fault of the form, and the
    /// process puts back a hook that it replaced.
    fn step(&self, py: Python<'_>, line: usize) {
        let readers = {
            let state = self.state();
            if state.started {
                return;
            }
            state.readers
        };
        if let Some(replaced) = readers.and_then(|readers| readers.replaced(py)) {
            self.fault(
                "hook-replaced",
                format!(
                    "line {line} replaced {}, and {}",
                    replaced.what(),
                    replaced.rule()
                ),
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
    tries: Vec<(String, Vec<Try>)>,
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
    /// an assert inside a reading, a hook that the case replaced, an `all()`
    /// whose reading holds no item, or a name of an expectation that holds an
    /// iterator.
    pub form: Vec<(&'static str, String)>,
    /// Each exception that the entry or the cleanup of a `with` raised, in the
    /// order they raised, beside the text that names that `with`.
    pub raised: Vec<(Py<PyAny>, String)>,
    /// Each `with` that swallowed a raise and skipped a line.
    pub swallowed: Vec<Swallowed>,
    /// The words of each `try` of one path whose statement did not raise, in
    /// the order they ran.
    pub missed: Vec<String>,
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

    /// Each `try` of `test`, in source order. A function that is not a test of
    /// the rewrite has none.
    #[must_use]
    pub fn tries(&self, test: &str) -> &[Try] {
        self.tries
            .iter()
            .find(|(name, _)| name == test)
            .map_or(&[], |(_, tries)| tries)
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
            missed: state.missed,
        }
    }
}

/// Each name of a hook, beside the hook that it holds.
fn hooks<'py>(
    ran: &Bound<'py, Ran>,
    read: &Bound<'py, Read>,
) -> PyResult<[(&'static str, Bound<'py, PyAny>); 6]> {
    Ok([
        (RAN, ran.as_any().clone()),
        (READ, read.as_any().clone()),
        (READS, read.getattr("reads")?),
        (STEP, read.getattr("step")?),
        (WITH, read.getattr("wrap")?),
        (MISSED, read.getattr("missed")?),
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
/// 1. names the module after the file of `path`, without `.py`, refuses the
///    name when `sys.modules` holds it, and sets the module's `__file__` to
///    `path`;
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
/// `ImportError` when `sys.modules` holds the module's name. The error that
/// step 4 raises, such as one from `compile()` or the module's body.
/// `NameError` when the body bound a name of a hook to anything but the hook.
///
/// # Panics
///
/// A panic of step 4 or 5 goes on after step 6.
pub fn load<'py>(py: Python<'py>, path: &Path, rewrite: &Rewrite) -> PyResult<Loaded<'py>> {
    let text = rewrite.text.as_str();
    let name = path.file_stem().unwrap_or_default().to_string_lossy();
    let modules = py.import("sys")?.getattr("modules")?;
    if modules.contains(&*name)? {
        return Err(PyImportError::new_err(format!(
            "sys.modules already holds the name {name}"
        )));
    }
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
            lines: rewrite.lines.clone(),
            names: rewrite.names.clone(),
            bodies: rewrite.bodies.clone(),
            missed: rewrite.missed.clone(),
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
        tries: rewrite.tries.clone(),
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
    let keywords = PyDict::new(py);
    keywords.set_item("optimize", 0)?;
    let code = builtins
        .getattr("compile")?
        .call((text, path.as_os_str(), "exec"), Some(&keywords))?;
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
            for hook in [RAN, READ, READS, STEP, WITH, MISSED] {
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
        MISSED => "try hook",
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

/// The text of the type that `attempt` expects when it is a `try` of one path:
/// one simple statement, and one `except SomeError:` whose type is a name or a
/// dotted name, with or without `as`, and no `else` and no `finally`. Else
/// `None`.
fn one_path<'a>(attempt: &StmtTry, source: &'a str) -> Option<&'a str> {
    let ([statement], [ExceptHandler::ExceptHandler(handler)]) =
        (&*attempt.body, &*attempt.handlers)
    else {
        return None;
    };
    if attempt.is_star
        || !is_simple(statement)
        || !attempt.orelse.is_empty()
        || !attempt.finalbody.is_empty()
    {
        return None;
    }
    let kind = handler.type_.as_deref()?;
    let mut part = kind;
    loop {
        match part {
            Expr::Name(_) => return Some(&source[kind.range()]),
            Expr::Attribute(attribute) => part = &attribute.value,
            _ => return None,
        }
    }
}

/// Adds to `edits` a call of the hook of a missed `try` after the step of the
/// one statement of each `try` of one path of each test of `by_test`. Gives the
/// words of each such `try`, by its index, and each `try` of each test.
fn mark_each_try(
    module: &Module,
    by_test: &[(&StmtFunctionDef, Vec<&StmtAssert>)],
    edits: &mut Vec<Edit>,
) -> (Vec<String>, Vec<(String, Vec<Try>)>) {
    let line = |offset: TextSize| module.position(offset).line();
    let mut missed = Vec::new();
    let mut tries = Vec::new();
    for (function, _) in by_test {
        let mut found = Vec::new();
        for statement in statements(&function.body) {
            let Stmt::Try(attempt) = statement else {
                continue;
            };
            let at = line(attempt.start());
            let lines = at..=line(attempt.end());
            let Some(kind) = one_path(attempt, module.source()) else {
                found.push(Try {
                    lines,
                    except: None,
                });
                continue;
            };
            edits.push(Edit::insert(
                attempt.body[0].end().to_usize(),
                format!("; {MISSED}({})", missed.len()),
            ));
            missed.push(format!(
                "the try at line {at} expects {kind}, and the statement did not raise it"
            ));
            let handler = &attempt.handlers[0];
            found.push(Try {
                lines,
                except: Some(line(handler.start())..=line(handler.end())),
            });
        }
        tries.push((function.name.to_string(), found));
    }
    (missed, tries)
}
