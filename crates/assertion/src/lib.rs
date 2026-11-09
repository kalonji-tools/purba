//! Assertion: rewrites each assert of a module, except a tuple assert, so that
//! it records that it ran, and whether it held. It also loads the rewritten module.

use purba_parsing::Module;
use pyo3::exceptions::{PyIndexError, PyNameError};
use pyo3::types::{PyAnyMethods, PyDictMethods, PyModule, PyModuleMethods};
use pyo3::{Bound, PyAny, PyErr, PyResult, Python, pyclass, pymethods};
use ruff_python_ast::statement_visitor::{StatementVisitor, walk_stmt};
use ruff_python_ast::{Expr, Stmt, StmtAssert};
use ruff_text_size::{Ranged, TextRange};
use std::panic::{self, AssertUnwindSafe};
use std::path::{Path, PathBuf};
use std::sync::{Mutex, PoisonError};

/// The name that each marked assert calls.
const RAN: &str = "_purba_ran";

/// The rewritten text of one module, and the text of each assert that it marks.
pub struct Rewrite {
    text: String,
    asserts: Vec<String>,
}

impl Rewrite {
    /// The module's text, with each assert marked except a tuple assert.
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

    /// Parses `source`, the text of a module that no file holds, and rewrites
    /// it.
    ///
    /// # Errors
    ///
    /// When `source` does not parse.
    pub fn of_source(source: &str) -> Result<Self, purba_parsing::Error> {
        Ok(rewrite(&purba_parsing::parse_source(source.to_owned())?))
    }
}

/// Each assert of a module that the rewrite marks, in source order, at any
/// depth.
struct Asserts<'a> {
    found: Vec<&'a StmtAssert>,
}

impl<'a> StatementVisitor<'a> for Asserts<'a> {
    fn visit_stmt(&mut self, statement: &'a Stmt) {
        if let Stmt::Assert(assert) = statement
            && !matches!(&*assert.test, Expr::Tuple(tuple) if !tuple.elts.is_empty())
        {
            self.found.push(assert);
        }
        walk_stmt(self, statement);
    }
}

/// Marks each assert of `module`: `assert x` becomes `assert _purba_ran(i, x)`,
/// where `i` counts the marked asserts of the module in source order, from 0.
///
/// An assert of a tuple with items stays as written, so that the compiler warns
/// that it is always true.
///
/// # Panics
///
/// In a build with the feature `fault`, when `PURBA_FAULT` is `rewrite`.
#[must_use]
pub fn rewrite(module: &Module) -> Rewrite {
    if cfg!(feature = "fault")
        && std::env::var_os("PURBA_FAULT").is_some_and(|setting| setting == "rewrite")
    {
        panic!("PURBA_FAULT=rewrite");
    }
    let source = module.source();
    let mut visitor = Asserts { found: Vec::new() };
    visitor.visit_body(&module.syntax().body);

    let mut rewritten = String::with_capacity(source.len());
    let mut asserts = Vec::with_capacity(visitor.found.len());
    let mut copied = 0;
    for (index, assert) in visitor.found.iter().enumerate() {
        let condition = assert.test.range();
        // ruff's range of `(yield x)` leaves out the parentheses, and a yield
        // inside an expression is a `yield_atom`, which needs them.
        // https://docs.python.org/3.12/reference/expressions.html#yield-expressions
        let bare = matches!(*assert.test, Expr::Yield(_) | Expr::YieldFrom(_));
        let (open, close) = if bare { ("(", ")") } else { ("", "") };
        rewritten.push_str(&source[copied..condition.start().to_usize()]);
        rewritten.push_str(RAN);
        rewritten.push('(');
        rewritten.push_str(&index.to_string());
        rewritten.push_str(", ");
        rewritten.push_str(open);
        rewritten.push_str(&source[condition]);
        rewritten.push_str(close);
        rewritten.push(')');
        copied = condition.end().to_usize();
        asserts.push(without_trivia(module, assert.range()));
    }
    rewritten.push_str(&source[copied..]);
    Rewrite {
        text: rewritten,
        asserts,
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
        if cfg!(feature = "fault")
            && std::env::var_os("PURBA_FAULT").is_some_and(|setting| setting == "assert")
        {
            panic!("PURBA_FAULT=assert");
        }
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

/// A module that `load` made from a rewritten text.
pub struct Loaded<'py> {
    path: PathBuf,
    module: Bound<'py, PyModule>,
    ran: Bound<'py, Ran>,
}

impl<'py> Loaded<'py> {
    /// The module, whose namespace stays in Python.
    #[must_use]
    pub const fn module(&self) -> &Bound<'py, PyModule> {
        &self.module
    }

    /// Runs `call` with no assert recorded. Gives its result, and the text of
    /// the last marked assert inside it that did not hold, or else of the last
    /// one that ran, beside whether it held.
    ///
    /// When `call` bound `_purba_ran` to anything but the hook, the result is
    /// the error that [`load`] gives such a module, and the hook is back.
    pub fn watch<T>(
        &self,
        call: impl FnOnce() -> PyResult<T>,
    ) -> (PyResult<T>, Option<(&str, bool)>) {
        let ran = self.ran.get();
        let _ = ran.take();
        let result = match (call(), check(&self.module, &self.ran, &self.path)) {
            (result, Ok(())) => result,
            (_, Err(err)) => self.module.dict().set_item(RAN, &self.ran).and(Err(err)),
        };
        let last = ran
            .take()
            .and_then(|(index, held)| Some((ran.asserts.get(index)?.as_str(), held)));
        (result, last)
    }
}

/// Loads the text of `rewrite` as the module at `path`, in this order:
///
/// 1. names the module after the file of `path`, without `.py`, and sets its
///    `__file__` to `path`;
/// 2. gives it `_purba_ran`, which holds the text of each assert of
///    `rewrite` at the index that its mark records;
/// 3. puts it in `sys.modules` under that name;
/// 4. compiles the text, checks that no function holds `_purba_ran` as a local
///    name, and runs it in the module's namespace;
/// 5. checks that `_purba_ran` still holds the hook;
/// 6. when step 4 or 5 raises or panics, puts back what `sys.modules` held
///    under that name, or takes the name out when it held nothing.
///
/// # Errors
///
/// The error that step 4 raises, such as one from `compile()` or the module's
/// body. `NameError` when the body bound `_purba_ran` to anything but the hook.
///
/// # Panics
///
/// A panic of step 4 or 5 goes on after step 6.
pub fn load<'py>(py: Python<'py>, path: &Path, rewrite: &Rewrite) -> PyResult<Loaded<'py>> {
    let (text, asserts) = (rewrite.text.as_str(), rewrite.asserts.clone());
    let name = path.file_stem().unwrap_or_default().to_string_lossy();
    let module = PyModule::new(py, &name)?;
    module.setattr("__file__", path.as_os_str())?;
    let ran = Bound::new(
        py,
        Ran {
            asserts,
            kept: Mutex::new(None),
        },
    )?;
    module.setattr(RAN, &ran)?;
    let modules = py.import("sys")?.getattr("modules")?;
    let before = modules.get_item(&*name).ok();
    modules.set_item(&*name, &module)?;
    let ran_body = panic::catch_unwind(AssertUnwindSafe(|| {
        run(py, path, text, &module).and_then(|()| check(&module, &ran, path))
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

/// Checks that `_purba_ran` in the namespace of `module` is `ran`.
///
/// # Errors
///
/// `NameError` that names the file at `path` when it is not.
fn check(module: &Bound<'_, PyModule>, ran: &Bound<'_, Ran>, path: &Path) -> PyResult<()> {
    if module
        .dict()
        .get_item(RAN)?
        .is_some_and(|held| held.is(ran))
    {
        return Ok(());
    }
    Err(bound(path))
}

/// Checks that no code object in `code`, at any depth, holds `_purba_ran` among
/// the local names of its function. Such a name hides the hook from each
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
            if current.getattr(names)?.contains(RAN)? {
                return Err(bound(path));
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

/// The error of a module at `path` that binds `_purba_ran`.
fn bound(path: &Path) -> PyErr {
    PyNameError::new_err(format!(
        "{} binds {RAN}, the name of purba's assert hook",
        path.display()
    ))
}
