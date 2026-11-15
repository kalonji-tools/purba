//! Assertion: rewrites the asserts of a module, and loads the rewritten module.
//! An assert at the end of a test's body is an expectation, which becomes a
//! reading that execution compares. Every other assert, except a tuple assert,
//! keeps a mark that records that it ran, and whether it held.

use purba_parsing::Module;
use pyo3::exceptions::{PyIndexError, PyNameError};
use pyo3::types::{PyAnyMethods, PyDictMethods, PyModule, PyModuleMethods};
use pyo3::{Bound, PyAny, PyErr, PyResult, Python, pyclass, pymethods};
use ruff_python_ast::statement_visitor::{StatementVisitor, walk_stmt};
use ruff_python_ast::{Expr, Stmt, StmtAssert};
use ruff_text_size::{Ranged, TextRange};
use std::io;
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

/// The rewritten text of one module, the text of each assert that it marks,
/// and the text of each expectation of each test.
pub struct Rewrite {
    text: String,
    asserts: Vec<String>,
    expectations: Vec<String>,
    tests: Vec<(String, Range<usize>)>,
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

/// The expectations of each test of `module` that `tests` names, in source
/// order: the asserts at the end of the body of its `def` at the top of the
/// module, with no other statement after them. When two `def`s bind one name,
/// the last binds it.
fn expectations<'a>(module: &'a Module, tests: &[&str]) -> Vec<(&'a str, Vec<&'a StmtAssert>)> {
    let mut found: Vec<(&str, Vec<&StmtAssert>)> = Vec::new();
    for statement in &module.syntax().body {
        let Stmt::FunctionDef(function) = statement else {
            continue;
        };
        let name = function.name.as_str();
        if function.is_async || !tests.contains(&name) {
            continue;
        }
        let mut asserts: Vec<&StmtAssert> = function
            .body
            .iter()
            .rev()
            .map_while(|statement| match statement {
                Stmt::Assert(assert) if !of_a_tuple(assert) => Some(assert),
                _ => None,
            })
            .collect();
        asserts.reverse();
        found.retain(|(seen, _)| *seen != name);
        found.push((name, asserts));
    }
    found
}

/// What the rewrite puts in place of one assert.
enum Edit<'a> {
    Mark(usize, &'a StmtAssert),
    Expect(usize, &'a StmtAssert),
}

impl Edit<'_> {
    fn start(&self) -> usize {
        match self {
            Self::Mark(_, assert) | Self::Expect(_, assert) => assert.range().start().to_usize(),
        }
    }
}

/// Rewrites each assert of `module`, where `tests` names its tests.
///
/// An expectation `assert x` becomes `_purba_reads(i) and _purba_read(i, x)`,
/// where `i` counts the expectations of the module in source order, from 0. It
/// keeps each line break of the assert inside the call, so each later line
/// keeps its number, and drops the assert's message.
///
/// Every other assert `assert x` becomes `assert _purba_ran(i, x)`, where `i`
/// counts the marked asserts of the module in source order, from 0. An assert
/// of a tuple with items stays as written, so that the compiler warns that it
/// is always true.
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
    let by_test = expectations(module, tests);
    let all: Vec<&StmtAssert> = by_test
        .iter()
        .flat_map(|(_, asserts)| asserts.iter().copied())
        .collect();
    let mut visitor = Asserts {
        expectations: &all,
        found: Vec::new(),
    };
    visitor.visit_body(&module.syntax().body);

    let mut edits: Vec<Edit<'_>> = visitor
        .found
        .iter()
        .enumerate()
        .map(|(index, assert)| Edit::Mark(index, assert))
        .chain(
            all.iter()
                .enumerate()
                .map(|(index, assert)| Edit::Expect(index, assert)),
        )
        .collect();
    edits.sort_by_key(Edit::start);

    let mut rewritten = String::with_capacity(source.len());
    let mut copied = 0;
    for edit in &edits {
        match edit {
            Edit::Mark(index, assert) => {
                let condition = assert.test.range();
                rewritten.push_str(&source[copied..condition.start().to_usize()]);
                rewritten.push_str(RAN);
                rewritten.push('(');
                rewritten.push_str(&index.to_string());
                rewritten.push_str(", ");
                push_condition(&mut rewritten, source, assert);
                rewritten.push(')');
                copied = condition.end().to_usize();
            }
            Edit::Expect(index, assert) => {
                let whole = assert.range();
                let condition = assert.test.range();
                rewritten.push_str(&source[copied..whole.start().to_usize()]);
                let index = index.to_string();
                for part in [READS, "(", &index, ") and ", READ, "(", &index, ", "] {
                    rewritten.push_str(part);
                }
                let before = &source[TextRange::new(whole.start(), condition.start())];
                let after = &source[TextRange::new(condition.end(), whole.end())];
                push_breaks(&mut rewritten, before);
                push_condition(&mut rewritten, source, assert);
                push_breaks(&mut rewritten, after);
                rewritten.push(')');
                copied = whole.end().to_usize();
            }
        }
    }
    rewritten.push_str(&source[copied..]);

    let mut ranges = Vec::with_capacity(by_test.len());
    let mut start = 0;
    for (name, asserts) in &by_test {
        ranges.push(((*name).to_owned(), start..start + asserts.len()));
        start += asserts.len();
    }
    Rewrite {
        text: rewritten,
        asserts: visitor
            .found
            .iter()
            .map(|assert| without_trivia(module, assert.range()))
            .collect(),
        expectations: all
            .iter()
            .map(|assert| without_trivia(module, assert.range()))
            .collect(),
        tests: ranges,
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

/// The text of each marked assert of one loaded module, and the index of the
/// last one that did not hold, or else of the last one that ran, beside
/// whether it held.
#[pyclass(frozen)]
struct Ran {
    asserts: Vec<String>,
    kept: Mutex<Option<(usize, bool)>>,
}

#[pymethods]
impl Ran {
    /// Tests `value` once, and records `index` and whether `value` held, unless
    /// `value` held and an assert that did not hold is on record. Gives back
    /// whether `value` held, so that the assert tests a `bool`.
    ///
    /// # Errors
    ///
    /// `IndexError` when no assert has `index`, or the error that testing
    /// `value` raised. Either records nothing.
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

/// What a reading of one expectation gave: it held, it did not hold, or it
/// raised.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Verdict {
    Holds,
    Fails,
    Raises,
}

/// What reading each later expectation alone needs from the process that runs
/// a case.
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
}

/// What the expectations of one case gave: the verdict of each that the case
/// read in order, the verdict of each that a reader read alone, and why no
/// reader was forked when the fork failed.
#[derive(Debug, Default, PartialEq, Eq)]
pub struct Readings {
    pub in_order: Vec<(usize, bool)>,
    pub alone: Vec<(usize, Verdict)>,
    pub refused: Option<String>,
}

#[derive(Default)]
struct State {
    readers: Option<&'static dyn Readers>,
    reader: Option<usize>,
    started: bool,
    readings: Readings,
}

/// The hook of the expectations of one loaded module. For each expectation, it
/// holds the range of the expectations of its test.
#[pyclass(frozen)]
struct Read {
    tests: Vec<Range<usize>>,
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
}

#[pymethods]
impl Read {
    /// Whether this process reads the expectation at `index`. The first call of
    /// a case forks a reader for each later expectation of its test, one at a
    /// time, and waits for each, unless a Python thread other than the main
    /// one is alive. A reader reads only its own expectation.
    ///
    /// # Errors
    ///
    /// `IndexError` when no expectation has `index`.
    fn reads(&self, py: Python<'_>, index: usize) -> PyResult<bool> {
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
        if readers.threaded(py) {
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

    /// Records whether `value` held as the reading of the expectation at
    /// `index`. A reader ends here, with its verdict.
    ///
    /// # Errors
    ///
    /// `IndexError` when no expectation has `index`, or the error that testing
    /// `value` raised.
    ///
    /// # Panics
    ///
    /// In a build with the feature `fault`, when `PURBA_FAULT` is `assert`.
    fn __call__(&self, index: usize, value: &Bound<'_, PyAny>) -> PyResult<()> {
        self.known(index)?;
        let held = value.is_truthy();
        let (reader, readers) = {
            let state = self.state();
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
        self.state().readings.in_order.push((index, held));
        Ok(())
    }
}

/// A module that `load` made from a rewritten text.
pub struct Loaded<'py> {
    path: PathBuf,
    module: Bound<'py, PyModule>,
    ran: Bound<'py, Ran>,
    read: Bound<'py, Read>,
    expectations: Vec<String>,
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
}

impl<'py> Loaded<'py> {
    /// The module, whose namespace stays in Python.
    #[must_use]
    pub const fn module(&self) -> &Bound<'py, PyModule> {
        &self.module
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

    /// Runs `call` with no assert and no reading recorded, with `readers` for
    /// the expectations. Gives its result, the last mark, and the readings.
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
        let ran = self.ran.get();
        let read = self.read.get();
        let _ = ran.take();
        *read.state() = State {
            readers: Some(readers),
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
        }
    }
}

/// Gives `module` each hook.
fn bind(
    module: &Bound<'_, PyModule>,
    ran: &Bound<'_, Ran>,
    read: &Bound<'_, Read>,
) -> PyResult<()> {
    module.setattr(RAN, ran)?;
    module.setattr(READ, read)?;
    module.setattr(READS, read.getattr("reads")?)
}

/// Loads the text of `rewrite` as the module at `path`, in this order:
///
/// 1. names the module after the file of `path`, without `.py`, and sets its
///    `__file__` to `path`;
/// 2. gives it `_purba_ran`, which holds the text of each assert of `rewrite`
///    at the index that its mark records, and `_purba_read` and
///    `_purba_reads`, the hook of its expectations;
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
    let ran = Bound::new(
        py,
        Ran {
            asserts: rewrite.asserts.clone(),
            kept: Mutex::new(None),
        },
    )?;
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
            state: Mutex::new(State::default()),
        },
    )?;
    bind(&module, &ran, &read)?;
    let modules = py.import("sys")?.getattr("modules")?;
    let before = modules.get_item(&*name).ok();
    modules.set_item(&*name, &module)?;
    let ran_body = panic::catch_unwind(AssertUnwindSafe(|| {
        run(py, path, text, &module).and_then(|()| check(&module, &ran, &read, path))
    }));
    if !matches!(ran_body, Ok(Ok(()))) {
        let _ = before.map_or_else(
            || modules.del_item(&*name),
            |before| modules.set_item(&*name, before),
        );
    }
    match ran_body {
        Ok(Ok(())) => {}
        Ok(Err(err)) => return Err(err),
        Err(payload) => panic::resume_unwind(payload),
    }
    Ok(Loaded {
        path: path.to_owned(),
        module,
        ran,
        read,
        expectations: rewrite.expectations.clone(),
        tests: rewrite.tests.clone(),
    })
}

/// Compiles `text`, and runs it in the namespace of `module`.
fn run(py: Python<'_>, path: &Path, text: &str, module: &Bound<'_, PyModule>) -> PyResult<()> {
    let builtins = py.import("builtins")?;
    let code = builtins
        .getattr("compile")?
        .call1((text, path.as_os_str(), "exec"))?;
    check_locals(&code, path)?;
    builtins.getattr("exec")?.call1((code, module.dict()))?;
    Ok(())
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
    let holds = |name: &str, hook: &Bound<'_, PyAny>| -> PyResult<bool> {
        Ok(match names.get_item(name)? {
            Some(held) if name == READS => held.eq(hook)?,
            Some(held) => held.is(hook),
            None => false,
        })
    };
    for (name, hook) in [
        (RAN, ran.as_any().clone()),
        (READ, read.as_any().clone()),
        (READS, read.getattr("reads")?),
    ] {
        if !holds(name, &hook)? {
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
            for hook in [RAN, READ, READS] {
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
    let hook = if name == RAN {
        "assert hook"
    } else {
        "expectation hook"
    };
    PyNameError::new_err(format!(
        "{} binds {name}, the name of purba's {hook}",
        path.display()
    ))
}
