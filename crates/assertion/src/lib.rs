//! Assertion: rewrites each assert of a module so that it records that it ran,
//! and whether it held. It also loads the rewritten module.

use purba_parsing::Module;
use pyo3::exceptions::{PyIndexError, PyNameError};
use pyo3::types::{PyAnyMethods, PyDictMethods, PyModule, PyModuleMethods};
use pyo3::{Bound, PyAny, PyErr, PyResult, Python, pyclass, pymethods};
use ruff_python_ast::statement_visitor::{StatementVisitor, walk_stmt};
use ruff_python_ast::{Expr, Stmt, StmtAssert};
use ruff_text_size::Ranged;
use std::path::{Path, PathBuf};
use std::sync::{Mutex, PoisonError};

/// The name that each marked assert calls.
const RAN: &str = "_purba_ran";

// PROTOTYPE (#715), never merge. PURBA_CATCH picks where a panic stops:
// none (today), dispatch, execution, hook, record. PURBA_FAULT injects a fault.
/// PROTOTYPE (#715): the catch mode.
#[must_use]
pub fn mode() -> String {
    std::env::var("PURBA_CATCH").unwrap_or_else(|_| "none".to_owned())
}

/// PROTOTYPE (#715): the fault switch.
#[must_use]
pub fn fault() -> String {
    std::env::var("PURBA_FAULT").unwrap_or_default()
}

/// PROTOTYPE (#715): the text of a panic's payload.
#[must_use]
pub fn panic_message(payload: &(dyn std::any::Any + Send)) -> String {
    payload
        .downcast_ref::<String>()
        .cloned()
        .or_else(|| {
            payload
                .downcast_ref::<&str>()
                .map(|text| (*text).to_owned())
        })
        .unwrap_or_else(|| "a panic with no message".to_owned())
}

mod internal {
    pyo3::create_exception!(
        purba,
        InternalErrorInsideATest,
        pyo3::exceptions::PyBaseException
    );
}

/// PROTOTYPE (#715): the message of purba's own exception, when `err` is one.
#[must_use]
pub fn internal_message(py: Python<'_>, err: &PyErr) -> Option<String> {
    err.is_instance_of::<internal::InternalErrorInsideATest>(py)
        .then(|| err.value(py).to_string())
}

/// PROTOTYPE (#715): why `load` failed, sorted by Q3's line.
#[derive(Debug)]
pub enum LoadError {
    /// The tester's text did not compile or run.
    Project(PyErr),
    /// A call that purba makes on its own objects failed.
    Internal(PyErr),
}

fn own<T>(result: PyResult<T>) -> Result<T, LoadError> {
    result.map_err(LoadError::Internal)
}

/// The rewritten text of one module, and the source text of each of its asserts.
pub struct Rewrite {
    text: String,
    asserts: Vec<String>,
}

impl Rewrite {
    /// The module's text, with each assert marked.
    #[must_use]
    pub fn text(&self) -> &str {
        &self.text
    }

    /// The source text of each assert, in source order. The index of an assert
    /// here is the index that its mark records.
    #[must_use]
    pub fn asserts(&self) -> &[String] {
        &self.asserts
    }

    /// The rewritten text and the source text of each assert, moved out.
    #[must_use]
    pub fn into_parts(self) -> (String, Vec<String>) {
        (self.text, self.asserts)
    }
}

/// Each assert of a module, in source order, at any depth.
struct Asserts<'a> {
    found: Vec<&'a StmtAssert>,
}

impl<'a> StatementVisitor<'a> for Asserts<'a> {
    fn visit_stmt(&mut self, statement: &'a Stmt) {
        if let Stmt::Assert(assert) = statement {
            self.found.push(assert);
        }
        walk_stmt(self, statement);
    }
}

/// Marks each assert of `module`: `assert x` becomes `assert _purba_ran(i, x)`,
/// where `i` counts the asserts of the module in source order, from 0.
#[must_use]
pub fn rewrite(module: &Module) -> Rewrite {
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
        asserts.push(source[assert.range()].to_owned());
    }
    rewritten.push_str(&source[copied..]);
    Rewrite {
        text: rewritten,
        asserts,
    }
}

/// The source text of each assert of one loaded module, the index of the last
/// one that ran, and whether it held.
#[pyclass(frozen)]
struct Ran {
    asserts: Vec<String>,
    last: Mutex<Option<(usize, bool)>>,
    // PROTOTYPE (#715): hook mode records its own panic, so that a test that
    // catches BaseException cannot hide it.
    panicked: Mutex<Option<String>>,
}

#[pymethods]
impl Ran {
    /// Tests `value` once, and records `index` and whether `value` held. Gives
    /// that back, so that the assert tests a `bool`.
    ///
    /// # Errors
    ///
    /// `IndexError` when no assert has `index`, or the error that testing
    /// `value` raised. Either records nothing.
    fn __call__(&self, index: usize, value: &Bound<'_, PyAny>) -> PyResult<bool> {
        if mode() == "hook" {
            return match std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                self.call(index, value)
            })) {
                Ok(result) => result,
                Err(payload) => {
                    let message = panic_message(&*payload);
                    *self.panicked.lock().unwrap_or_else(PoisonError::into_inner) =
                        Some(message.clone());
                    Err(internal::InternalErrorInsideATest::new_err(message))
                }
            };
        }
        self.call(index, value)
    }
}

impl Ran {
    fn call(&self, index: usize, value: &Bound<'_, PyAny>) -> PyResult<bool> {
        // PROTOTYPE (#715): `assert "purba:panic"` makes the hook panic.
        if value
            .extract::<String>()
            .is_ok_and(|text| text == "purba:panic")
        {
            panic!("PROTOTYPE fault: the assert hook panicked");
        }
        if index >= self.asserts.len() {
            return Err(PyIndexError::new_err(format!(
                "{RAN}({index}): the module has {} asserts",
                self.asserts.len()
            )));
        }
        let held = value.is_truthy()?;
        *self.last.lock().unwrap_or_else(PoisonError::into_inner) = Some((index, held));
        Ok(held)
    }
}

impl Ran {
    fn take(&self) -> Option<(usize, bool)> {
        self.last
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

    /// Runs `call` with no assert recorded. Gives its result, and the source
    /// text of the last marked assert that ran inside it, beside whether it held.
    ///
    /// When `call` bound `_purba_ran` to anything but the hook, the result is
    /// the error that [`load`] gives such a module, and the hook is back.
    pub fn watch<T>(
        &self,
        call: impl FnOnce() -> PyResult<T>,
    ) -> (
        PyResult<T>,
        Option<(&str, bool)>,
        Option<(&'static str, String)>,
    ) {
        let ran = self.ran.get();
        let _ = ran.take();
        let mut internal = None;
        let result = match (call(), check(&self.module, &self.ran, &self.path)) {
            (result, Ok(())) => result,
            (_, Err(err)) => {
                // PROTOTYPE (#715): PURBA_FAULT=hook-back makes the put-back fail.
                let back = if fault() == "hook-back" {
                    Err(pyo3::exceptions::PyRuntimeError::new_err(
                        "PROTOTYPE fault: the hook could not be put back",
                    ))
                } else {
                    self.module.dict().set_item(RAN, &self.ran)
                };
                match back {
                    Ok(()) => Err(err),
                    Err(back) => {
                        internal = Some(("execution", back.to_string()));
                        Err(err)
                    }
                }
            }
        };
        let last = ran
            .take()
            .and_then(|(index, held)| Some((ran.asserts.get(index)?.as_str(), held)));
        // PROTOTYPE (#715): hook mode sees its own panic, though the test caught it.
        if let Some(message) = ran
            .panicked
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .take()
        {
            internal.get_or_insert(("assertion", message));
        }
        (result, last, internal)
    }
}

/// Loads `text` as the module at `path`, in this order:
///
/// 1. names the module after the file of `path`, without `.py`, and sets its
///    `__file__` to `path`;
/// 2. gives it `_purba_ran`, which holds `asserts`, the source text of each
///    assert at the index that its mark records;
/// 3. puts it in `sys.modules` under that name;
/// 4. compiles `text`, checks that no function holds `_purba_ran` as a local
///    name, and runs it in the module's namespace;
/// 5. checks that `_purba_ran` still holds the hook;
/// 6. when step 4 or 5 raises, puts back what `sys.modules` held under that
///    name, or takes the name out when it held nothing.
///
/// # Errors
///
/// The error that step 4 raises, such as one from `compile()` or the module's
/// body. `NameError` when the body bound `_purba_ran` to anything but the hook.
pub fn load<'py>(
    py: Python<'py>,
    path: &Path,
    text: &str,
    asserts: Vec<String>,
) -> Result<Loaded<'py>, LoadError> {
    let name = path.file_stem().unwrap_or_default().to_string_lossy();
    // PROTOTYPE (#715): PURBA_FAULT=load-own-call makes purba's own first call fail.
    if fault() == "load-own-call" {
        return Err(LoadError::Internal(
            pyo3::exceptions::PyRuntimeError::new_err("PROTOTYPE fault: PyModule::new failed"),
        ));
    }
    let module = own(PyModule::new(py, &name))?;
    own(module.setattr("__file__", path.as_os_str()))?;
    let ran = own(Bound::new(
        py,
        Ran {
            asserts,
            last: Mutex::new(None),
            panicked: Mutex::new(None),
        },
    ))?;
    own(module.setattr(RAN, &ran))?;
    let modules = own(py.import("sys").and_then(|sys| sys.getattr("modules")))?;
    let before = modules.get_item(&*name).ok();
    own(modules.set_item(&*name, &module))?;
    if let Err(err) = run(py, path, text, &module).and_then(|()| check(&module, &ran, path)) {
        let _ = before.map_or_else(
            || modules.del_item(&*name),
            |before| modules.set_item(&*name, before),
        );
        return Err(LoadError::Project(err));
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
