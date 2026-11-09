//! The hook on `threading.excepthook` and `sys.unraisablehook`. It keeps each
//! exception that a thread raises until a worker takes it.

use std::sync::{Mutex, PoisonError};

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
/// `SystemExit` itself and `ExitInsideATest`, then calls
/// `threading.__excepthook__`, which prints the traceback. The guard on
/// `os._exit` keeps the first call, which gives the test that runs `raised`.
/// It also holds the real `threading.enumerate`, so a test that replaces it
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
        if kind.is(py.get_type::<PySystemExit>())
            || kind.is(py.get_type::<crate::ExitInsideATest>())
        {
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
