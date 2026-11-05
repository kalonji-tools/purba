//! purba's signal handler. It keeps the first interrupt of the process. It
//! raises `KeyboardInterrupt` only inside a test's call or a load. Once Python
//! finalizes, purba dies by the interrupt that it kept.

use std::io::{self, Write};
use std::sync::atomic::{AtomicBool, AtomicI32, Ordering};

use pyo3::exceptions::PyKeyboardInterrupt;
use pyo3::sync::PyOnceLock;
use pyo3::types::PyAnyMethods;
use pyo3::{Bound, Py, PyAny, PyResult, Python, pyclass, pymethods};

/// The number of the first signal that the signal handler took, or 0.
static KEPT: AtomicI32 = AtomicI32::new(0);

/// Whether a test's call or a load runs now.
static CALLING: AtomicBool = AtomicBool::new(false);

/// Takes the place of Python's own signal handler for SIGINT.
#[pyclass(frozen)]
struct SignalHandler;

#[pymethods]
impl SignalHandler {
    #[expect(
        clippy::unused_self,
        reason = "Python calls the signal handler as an object"
    )]
    fn __call__(&self, py: Python<'_>, signum: i32, _frame: &Bound<'_, PyAny>) -> PyResult<()> {
        if KEPT
            .compare_exchange(0, signum, Ordering::SeqCst, Ordering::SeqCst)
            .is_ok()
        {
            let name = py
                .import("signal")?
                .getattr("Signals")?
                .call1((signum,))?
                .getattr("name")?
                .extract::<String>()?;
            // stderr may refuse the write. The run stops either way.
            let _ = writeln!(io::stderr(), "purba: interrupt: {name}");
        }
        if CALLING.load(Ordering::SeqCst) {
            return Err(PyKeyboardInterrupt::new_err(()));
        }
        Ok(())
    }
}

static SIGNAL_HANDLER: PyOnceLock<Py<SignalHandler>> = PyOnceLock::new();

/// Gives SIGINT purba's signal handler, unless purba's parent ignores SIGINT.
///
/// A parent that starts purba in the background ignores SIGINT for it. The first
/// install also registers the function that makes purba die by the signal it
/// kept, once Python finalizes.
///
/// # Errors
///
/// `signal` refused the signal handler.
pub fn install() -> PyResult<()> {
    Python::initialize();
    Python::attach(|py| {
        let signal = py.import("signal")?;
        let sigint = signal.getattr("SIGINT")?;
        if signal
            .getattr("getsignal")?
            .call1((&sigint,))?
            .is(&signal.getattr("SIG_IGN")?)
        {
            return Ok(());
        }
        let handler = SIGNAL_HANDLER.get_or_try_init(py, || {
            // SAFETY: `die` runs no Python, so it may run after finalization.
            unsafe { pyo3::ffi::Py_AtExit(Some(die)) };
            Py::new(py, SignalHandler)
        })?;
        signal.getattr("signal")?.call1((sigint, handler))?;
        Ok(())
    })
}

/// Whether the signal handler kept a signal. A signal that waits runs the
/// signal handler before this check.
pub(crate) fn kept(py: Python<'_>) -> bool {
    // purba's signal handler raises nothing outside a call. A signal handler
    // that a test installed may raise here. Its error has no test to charge, so
    // `kept` drops it.
    drop(py.check_signals());
    KEPT.load(Ordering::SeqCst) != 0
}

/// Runs `work` as a test's call or a load, in which the signal handler raises
/// `KeyboardInterrupt`.
pub(crate) fn calling<R>(work: impl FnOnce() -> R) -> R {
    struct Close;
    impl Drop for Close {
        fn drop(&mut self) {
            CALLING.store(false, Ordering::SeqCst);
        }
    }
    CALLING.store(true, Ordering::SeqCst);
    let _close = Close;
    work()
}

/// Runs after finalization gives each signal its default action:
/// <https://github.com/python/cpython/blob/v3.15.2/Python/pylifecycle.c#L2465-L2466>
/// <https://github.com/python/cpython/blob/v3.15.2/Python/pylifecycle.c#L2598>
extern "C" fn die() {
    let signum = KEPT.load(Ordering::SeqCst);
    if signum == 0 {
        return;
    }
    #[cfg(unix)]
    // SAFETY: `signal` and `raise` take a signal number that the kernel sent.
    unsafe {
        libc::signal(signum, libc::SIG_DFL);
        libc::raise(signum);
    }
    // CPython exits with this code after a Ctrl-C:
    // <https://github.com/python/cpython/blob/v3.15.2/Modules/main.c#L793-L815>
    #[cfg(windows)]
    {
        const STATUS_CONTROL_C_EXIT: u32 = 0xC000_013A;
        #[expect(
            clippy::cast_possible_wrap,
            reason = "Windows reads the code as the unsigned value"
        )]
        std::process::exit(STATUS_CONTROL_C_EXIT as i32);
    }
}
