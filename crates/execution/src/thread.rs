//! The hook on `threading.excepthook` and `sys.unraisablehook`. It keeps each
//! exception that a thread raises until a worker takes it.

use std::sync::{Mutex, PoisonError};

use purba_assertion::Replaced;
use pyo3::exceptions::PySystemExit;
use pyo3::sync::PyOnceLock;
use pyo3::types::PyAnyMethods;
use pyo3::{Bound, Py, PyAny, PyErr, PyResult, Python, pyclass, pymethods};

/// An exception that a thread raised: the thread, when Python gives one, and
/// `thread <name> raised <type>: <message>`.
pub struct Raised {
    pub thread: Option<Py<PyAny>>,
    pub text: String,
}

/// Takes the place of `threading.excepthook`. It keeps each exception but
/// `SystemExit` itself, then calls `threading.__excepthook__`, which prints the
/// traceback. It also holds the real `threading.enumerate`, so a test that replaces it
/// changes no owner.
///
/// It also takes the place of `sys.unraisablehook`, where `CPython` sends the
/// exception of a thread that `_thread` starts. It keeps that entry alone, and
/// then calls `sys.__unraisablehook__`, which prints each entry.
#[pyclass(frozen)]
pub struct Hook {
    default: Py<PyAny>,
    unraisable_default: Py<PyAny>,
    enumerate: Py<PyAny>,
    raised: Mutex<Vec<Raised>>,
}

#[pymethods]
impl Hook {
    fn __call__(&self, args: &Bound<'_, PyAny>) -> PyResult<()> {
        let kept = (|| {
            self.keep(
                &args.getattr("exc_type")?,
                &args.getattr("exc_value")?,
                args.getattr("thread")?,
            )
        })();
        self.default.call1(args.py(), (args,))?;
        kept
    }

    /// `CPython` names the entry of a thread that `_thread` starts by this
    /// message, followed by the function in 3.13 and later:
    /// <https://github.com/python/cpython/blob/v3.12.0/Modules/_threadmodule.c#L1088>
    /// <https://github.com/python/cpython/blob/v3.12.0/Python/errors.c#L1646>
    /// <https://github.com/python/cpython/blob/v3.13.0/Modules/_threadmodule.c#L343-L344>
    /// <https://github.com/python/cpython/blob/v3.14.0/Modules/_threadmodule.c#L365-L366>
    fn unraisable(&self, args: &Bound<'_, PyAny>) -> PyResult<()> {
        let py = args.py();
        let kept = (|| {
            let message = args.getattr("err_msg")?;
            if message.is_none()
                || !message
                    .str()?
                    .to_string()
                    .starts_with("Exception ignored in thread started by")
            {
                return Ok(());
            }
            self.keep(
                &args.getattr("exc_type")?,
                &args.getattr("exc_value")?,
                py.None().into_bound(py),
            )
        })();
        self.unraisable_default.call1(py, (args,))?;
        kept
    }
}

impl Hook {
    /// Python's own hook prints nothing for `SystemExit` alone, and prints a
    /// subclass of it:
    /// <https://github.com/python/cpython/blob/v3.14.0/Modules/_threadmodule.c#L2283-L2287>
    fn keep(
        &self,
        kind: &Bound<'_, PyAny>,
        value: &Bound<'_, PyAny>,
        thread: Bound<'_, PyAny>,
    ) -> PyResult<()> {
        let py = kind.py();
        if kind.is(py.get_type::<PySystemExit>()) {
            return Ok(());
        }
        let name = if thread.is_none() {
            py.import("threading")?
                .getattr("get_ident")?
                .call0()?
                .str()?
                .to_string()
        } else {
            thread.getattr("name")?.str()?.to_string()
        };
        let exception = if value.is_none() {
            kind.getattr("__qualname__")?.str()?.to_string()
        } else {
            PyErr::from_value(value.clone()).to_string()
        };
        self.raised
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .push(Raised {
                thread: (!thread.is_none()).then(|| thread.unbind()),
                text: format!("thread {name} raised {exception}"),
            });
        Ok(())
    }

    /// Each exception kept since the last take, in the order threads raised them.
    pub fn take(&self) -> Vec<Raised> {
        std::mem::take(&mut *self.raised.lock().unwrap_or_else(PoisonError::into_inner))
    }

    /// The threads that are alive now.
    ///
    /// # Panics
    ///
    /// `threading.enumerate` did not give a list.
    pub fn alive(&self, py: Python<'_>) -> Vec<Py<PyAny>> {
        self.enumerate
            .call0(py)
            .and_then(|threads| threads.extract::<Vec<Py<PyAny>>>(py))
            .expect("threading.enumerate gives a list of threads")
    }
}

static HOOK: PyOnceLock<Py<Hook>> = PyOnceLock::new();

/// The hook, which takes the place of `threading.excepthook` and
/// `sys.unraisablehook` once per process.
pub fn hook(py: Python<'_>) -> PyResult<&'static Py<Hook>> {
    HOOK.get_or_try_init(py, || {
        let threading = py.import("threading")?;
        let sys = py.import("sys")?;
        let hook = Py::new(
            py,
            Hook {
                default: threading.getattr("__excepthook__")?.unbind(),
                unraisable_default: sys.getattr("__unraisablehook__")?.unbind(),
                enumerate: threading.getattr("enumerate")?.unbind(),
                raised: Mutex::new(Vec::new()),
            },
        )?;
        threading.setattr("excepthook", &hook)?;
        sys.setattr("unraisablehook", hook.getattr(py, "unraisable")?)?;
        Ok(hook)
    })
}

/// The hook, once `hook` gave it, in this process or in a process that this
/// process forked from. Gives `None` before that.
pub fn taken(py: Python<'_>) -> Option<&'static Py<Hook>> {
    HOOK.get(py)
}

/// Names what the process replaced since the filters were last held: the first
/// place of `threading.excepthook` and `sys.unraisablehook` that holds anything
/// but the hook, which [`hooks_replaced`] puts back, or else the warning
/// filters, when they differ from those held, or hold them in another order.
/// Gives `None` when neither changed.
pub fn replaced(py: Python<'_>) -> Option<Replaced> {
    hooks_replaced(py)
        .map(Replaced::Hook)
        .or_else(|| filters_differ(py).then_some(Replaced::Filters))
}

/// The warning filters that a case or a load began with: a copy of
/// `warnings.filters`.
static FILTERS: Mutex<Option<Py<PyAny>>> = Mutex::new(None);

/// Holds a copy of `warnings.filters`, which [`replaced`] compares with the
/// filters it finds.
pub fn hold_filters(py: Python<'_>) {
    let held = filters(py)
        .and_then(|filters| filters.call_method0("copy"))
        .ok()
        .map(Bound::unbind);
    *FILTERS.lock().unwrap_or_else(PoisonError::into_inner) = held;
}

/// Whether `warnings.filters` differs from the filters held, by a filter or by
/// their order. No filters held is no difference.
fn filters_differ(py: Python<'_>) -> bool {
    let held = FILTERS
        .lock()
        .unwrap_or_else(PoisonError::into_inner)
        .as_ref()
        .map(|held| held.clone_ref(py));
    let Some(held) = held else {
        return false;
    };
    filters(py)
        .and_then(|filters| filters.eq(held.bind(py)))
        .is_ok_and(|same| !same)
}

/// The warning filters that apply now. From `CPython` 3.14, `catch_warnings` keeps
/// them in a context variable when `sys.flags.context_aware_warnings` is set,
/// which a free-threaded build sets by default, and `warnings.filters` then stays
/// as it was. `warnings._get_filters` gives the filters of the context:
/// <https://github.com/python/cpython/blob/v3.14.0/Lib/_py_warnings.py>
/// <https://docs.python.org/3.14/library/warnings.html#concurrent-safety-of-context-managers>
/// Before 3.14 no such function exists, and `warnings.filters` is the list.
fn filters(py: Python<'_>) -> PyResult<Bound<'_, PyAny>> {
    let warnings = py.import("warnings")?;
    warnings
        .getattr("_get_filters")
        .map_or_else(|_| warnings.getattr("filters"), |get| get.call0())
}

/// Puts the hook back in each place of `threading.excepthook` and
/// `sys.unraisablehook` that holds anything else, and names the first such
/// place. Gives `None` when both hold the hook, or before the hook takes them.
fn hooks_replaced(py: Python<'_>) -> Option<&'static str> {
    let hook = HOOK.get(py)?;
    let threading = py.import("threading").ok()?;
    let sys = py.import("sys").ok()?;
    let unraisable = hook.getattr(py, "unraisable").ok()?;
    let mut first = if threading
        .getattr("excepthook")
        .is_ok_and(|held| held.is(hook))
    {
        None
    } else {
        drop(threading.setattr("excepthook", hook));
        Some("threading.excepthook")
    };
    if !sys
        .getattr("unraisablehook")
        .and_then(|held| held.eq(&unraisable))
        .unwrap_or(false)
    {
        drop(sys.setattr("unraisablehook", &unraisable));
        first.get_or_insert("sys.unraisablehook");
    }
    first
}
