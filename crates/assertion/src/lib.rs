//! Assertion: rewrites each assert of a module so that it records that it ran,
//! and whether it held. It also loads the rewritten module.

use purba_parsing::Module;
use pyo3::exceptions::PyIndexError;
use pyo3::types::{PyAnyMethods, PyModule, PyModuleMethods};
use pyo3::{Bound, PyAny, PyResult, Python, pyclass, pymethods};
use ruff_python_ast::statement_visitor::{StatementVisitor, walk_stmt};
use ruff_python_ast::{Expr, Stmt, StmtAssert};
use ruff_text_size::Ranged;
use std::path::Path;
use std::sync::{Mutex, PoisonError};

/// The name that each marked assert calls.
const RAN: &str = "_purba_ran";

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
    pub fn watch<R>(&self, call: impl FnOnce() -> R) -> (R, Option<(&str, bool)>) {
        let ran = self.ran.get();
        let _ = ran.take();
        let result = call();
        let last = ran
            .take()
            .and_then(|(index, held)| Some((ran.asserts.get(index)?.as_str(), held)));
        (result, last)
    }
}

/// Loads `text` as the module at `path`, in this order:
///
/// 1. names the module after the file of `path`, without `.py`, and sets its
///    `__file__` to `path`;
/// 2. gives it `_purba_ran`, which holds `asserts`, the source text of each
///    assert at the index that its mark records;
/// 3. puts it in `sys.modules` under that name;
/// 4. compiles `text`, and runs it in the module's namespace.
///
/// # Errors
///
/// The error that `compile()` or the module's body raises.
pub fn load<'py>(
    py: Python<'py>,
    path: &Path,
    text: &str,
    asserts: Vec<String>,
) -> PyResult<Loaded<'py>> {
    let name = path.file_stem().unwrap_or_default().to_string_lossy();
    let module = PyModule::new(py, &name)?;
    module.setattr("__file__", path.as_os_str())?;
    let ran = Bound::new(
        py,
        Ran {
            asserts,
            last: Mutex::new(None),
        },
    )?;
    module.setattr(RAN, &ran)?;
    let modules = py.import("sys")?.getattr("modules")?;
    let before = modules.get_item(&*name).ok();
    modules.set_item(&*name, &module)?;
    let run = || -> PyResult<()> {
        let builtins = py.import("builtins")?;
        let code = builtins
            .getattr("compile")?
            .call1((text, path.as_os_str(), "exec"))?;
        builtins.getattr("exec")?.call1((code, module.dict()))?;
        Ok(())
    };
    if let Err(err) = run() {
        // PROTOTYPE, never merge: PURBA_PROTO_658 picks what the entry holds
        // after a load fails: keep (main), remove, restore or none.
        match std::env::var("PURBA_PROTO_658").as_deref() {
            Ok("remove") => {
                let _ = modules.del_item(&*name);
            }
            Ok("restore") => {
                let _ = match &before {
                    Some(old) => modules.set_item(&*name, old),
                    None => modules.del_item(&*name),
                };
            }
            Ok("none") => {
                let _ = modules.set_item(&*name, py.None());
            }
            _ => {}
        }
        return Err(err);
    }
    Ok(Loaded { module, ran })
}
