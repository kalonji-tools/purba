//! The hook on `threading.excepthook`. It keeps each exception that a thread
//! raises until a worker takes it.

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
/// `SystemExit` itself, then calls `threading.__excepthook__`, which prints the traceback.
/// It also holds the real `threading.enumerate`, so a test that replaces it
/// changes no owner.
#[pyclass(frozen)]
pub struct Hook {
    default: Py<PyAny>,
    enumerate: Py<PyAny>,
    raised: Mutex<Vec<Raised>>,
}

#[pymethods]
impl Hook {
    fn __call__(&self, args: &Bound<'_, PyAny>) -> PyResult<()> {
        let kept = self.keep(args);
        self.default.call1(args.py(), (args,))?;
        kept
    }
}

impl Hook {
    /// Python's own hook prints nothing for `SystemExit` alone, and prints a
    /// subclass of it:
    /// <https://github.com/python/cpython/blob/v3.14.0/Modules/_threadmodule.c#L2283-L2287>
    fn keep(&self, args: &Bound<'_, PyAny>) -> PyResult<()> {
        let py = args.py();
        let kind = args.getattr("exc_type")?;
        if kind.is(py.get_type::<PySystemExit>()) {
            return Ok(());
        }
        let thread = args.getattr("thread")?;
        let name = if thread.is_none() {
            py.import("threading")?
                .getattr("get_ident")?
                .call0()?
                .str()?
                .to_string()
        } else {
            thread.getattr("name")?.str()?.to_string()
        };
        let value = args.getattr("exc_value")?;
        let exception = if value.is_none() {
            kind.getattr("__qualname__")?.str()?.to_string()
        } else {
            PyErr::from_value(value).to_string()
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

/// The hook, which takes the place of `threading.excepthook` once per process.
pub fn hook(py: Python<'_>) -> PyResult<&'static Py<Hook>> {
    HOOK.get_or_try_init(py, || {
        let threading = py.import("threading")?;
        let hook = Py::new(
            py,
            Hook {
                default: threading.getattr("__excepthook__")?.unbind(),
                enumerate: threading.getattr("enumerate")?.unbind(),
                raised: Mutex::new(Vec::new()),
            },
        )?;
        threading.setattr("excepthook", &hook)?;
        Ok(hook)
    })
}
