//! purba's signal handler. It keeps the first interrupt of the process.
//!
//! It raises `KeyboardInterrupt` inside a test's call or a load, and while
//! Python waits for threads after the command. After each call and each load,
//! purba gives each interrupt signal purba's signal handler again. The first
//! interrupt gives each interrupt signal its default action. The end of each
//! later call or load gives each interrupt signal its default action again. So
//! a second interrupt ends purba wherever it lands. Once Python finalizes,
//! purba dies by the interrupt that it kept.

use std::io::{self, Write};
use std::sync::atomic::{AtomicBool, AtomicI32, Ordering};
use std::sync::{Mutex, PoisonError};

use pyo3::exceptions::PyKeyboardInterrupt;
use pyo3::sync::PyOnceLock;
use pyo3::types::{PyAnyMethods, PyCFunction};
use pyo3::{Bound, Py, PyAny, PyResult, Python, pyclass, pymethods};

/// The number of the first signal that the signal handler took, or 0.
static KEPT: AtomicI32 = AtomicI32::new(0);

/// Whether a test's call or a load runs now.
static CALLING: AtomicBool = AtomicBool::new(false);

/// Whether the command returned, and purba's `atexit` function did not run yet.
static ENDED: AtomicBool = AtomicBool::new(false);

/// The number of each signal that `install` gave purba's signal handler.
static GIVEN: Mutex<Vec<i32>> = Mutex::new(Vec::new());

/// The names of the interrupt signals in Python's `signal` module.
#[cfg(not(windows))]
const NAMES: [&str; 3] = ["SIGINT", "SIGTERM", "SIGHUP"];
#[cfg(windows)]
const NAMES: [&str; 3] = ["SIGINT", "SIGTERM", "SIGBREAK"];

/// Takes the place of Python's own signal handler for each interrupt signal.
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
            .is_err()
        {
            // Two signals waited at once. Python took the second before purba
            // gave it the default action, so that action could not end purba.
            die_by(signum);
        }
        give_default_actions(py)?;
        let name = py
            .import("signal")?
            .getattr("Signals")?
            .call1((signum,))?
            .getattr("name")?
            .extract::<String>()?;
        // stderr may refuse the write. The run stops either way.
        let _ = writeln!(
            io::stderr(),
            "purba: interrupt: {}; a second interrupt exits at once",
            shown(&name)
        );
        if CALLING.load(Ordering::SeqCst) || ENDED.load(Ordering::SeqCst) {
            return Err(PyKeyboardInterrupt::new_err(()));
        }
        Ok(())
    }
}

static SIGNAL_HANDLER: PyOnceLock<Py<SignalHandler>> = PyOnceLock::new();

/// The numbers of the signals that `install` gave purba's signal handler.
fn given() -> Vec<i32> {
    GIVEN.lock().unwrap_or_else(PoisonError::into_inner).clone()
}

/// The name that purba's interrupt line on stderr gives a signal: the key that
/// sent it.
#[cfg(windows)]
fn shown(name: &str) -> &str {
    match name {
        "SIGINT" => "Ctrl-C",
        "SIGBREAK" => "Ctrl-Break",
        other => other,
    }
}

/// The name that purba's interrupt line on stderr gives a signal: Python's name
/// for it.
#[cfg(not(windows))]
const fn shown(name: &str) -> &str {
    name
}

/// Gives each interrupt signal purba's signal handler, unless purba's parent
/// ignores that signal.
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
        let handler = SIGNAL_HANDLER.get_or_try_init(py, || {
            // SAFETY: `die` runs no Python, so it may run after finalization.
            unsafe { pyo3::ffi::Py_AtExit(Some(die)) };
            Py::new(py, SignalHandler)
        })?;
        let ignored = signal.getattr("SIG_IGN")?;
        for name in NAMES {
            let signum = signal.getattr(name)?;
            if signal.getattr("getsignal")?.call1((&signum,))?.is(&ignored) {
                continue;
            }
            signal.getattr("signal")?.call1((&signum, handler))?;
            let signum = signum.extract::<i32>()?;
            let mut given = GIVEN.lock().unwrap_or_else(PoisonError::into_inner);
            if !given.contains(&signum) {
                given.push(signum);
            }
        }
        Ok(())
    })
}

/// Opens the window in which a first interrupt ends Python's wait for threads.
///
/// purba's `atexit` function closes it. Python waits for threads before it
/// calls any `atexit` function:
/// <https://github.com/python/cpython/blob/v3.15.2/Python/pylifecycle.c#L2284-L2300>
/// Python calls the `atexit` function registered last first:
/// <https://github.com/python/cpython/blob/v3.15.2/Modules/atexitmodule.c#L205-L206>
///
/// # Errors
///
/// purba cannot register its `atexit` function. The window then stays shut.
pub fn ended(py: Python<'_>) -> PyResult<()> {
    let close = PyCFunction::new_closure(py, None, None, |_args, _kwargs| {
        ENDED.store(false, Ordering::SeqCst);
    })?;
    py.import("atexit")?.getattr("register")?.call1((close,))?;
    ENDED.store(true, Ordering::SeqCst);
    Ok(())
}

/// Whether the signal handler kept a signal. A signal that waits runs the
/// signal handler before this check.
pub(crate) fn kept(py: Python<'_>) -> bool {
    // While the command runs, purba's signal handler raises nothing outside a
    // call. A signal handler that a test installed may raise here. Its error
    // has no test to charge, so `kept` drops it.
    drop(py.check_signals());
    kept_now()
}

/// Whether the signal handler kept a signal. It runs no Python, so a signal
/// that waits still waits after it.
pub(crate) fn kept_now() -> bool {
    KEPT.load(Ordering::SeqCst) != 0
}

/// Runs `work` as a test's call or a load, in which the signal handler raises
/// `KeyboardInterrupt`. Also says whether the first signal landed inside it.
/// Then gives each interrupt signal purba's signal handler again, if the test
/// or the module installed its own.
pub(crate) fn calling<R>(py: Python<'_>, work: impl FnOnce() -> R) -> (R, bool) {
    struct Close;
    impl Drop for Close {
        fn drop(&mut self) {
            CALLING.store(false, Ordering::SeqCst);
        }
    }
    let kept_before = kept_now();
    CALLING.store(true, Ordering::SeqCst);
    let close = Close;
    let result = work();
    let landed = !kept_before && kept_now();
    drop(close);
    put_back(py);
    (result, landed)
}

/// Gives purba's signal handler back to each signal that `install` gave it,
/// when the signal has another one now. After the first signal, it then gives
/// each signal its default action again.
fn put_back(py: Python<'_>) {
    let Some(handler) = SIGNAL_HANDLER.get(py) else {
        return;
    };
    let given = given();
    let give = || -> PyResult<()> {
        let signal = py.import("signal")?;
        for &signum in &given {
            if !signal.getattr("getsignal")?.call1((signum,))?.is(handler) {
                signal.getattr("signal")?.call1((signum, handler))?;
            }
        }
        Ok(())
    };
    // `signal.signal` first runs a signal that waits, through the signal
    // handler that the signal has, and that one may raise:
    // <https://github.com/python/cpython/blob/v3.15.2/Modules/signalmodule.c#L522-L525>
    // The signal no longer waits then, so a second try gives each signal
    // purba's signal handler. Its error has no test to charge.
    if give().is_err() {
        drop(give());
    }
    if kept_now() {
        // Its error has no test to charge either.
        drop(give_default_actions(py));
    }
}

/// Gives each signal that still has purba's signal handler its default action,
/// in the operating system only. Python's own table keeps purba's signal
/// handler, so a signal that waits already still reaches it. If purba gave the
/// default action through `signal.signal`, Python would find a waiting signal
/// with no handler. Python would then report an `OSError` in place of the
/// signal:
/// <https://github.com/python/cpython/blob/v3.15.2/Modules/signalmodule.c#L1838-L1855>
fn give_default_actions(py: Python<'_>) -> PyResult<()> {
    let Some(handler) = SIGNAL_HANDLER.get(py) else {
        return Ok(());
    };
    let getsignal = py.import("signal")?.getattr("getsignal")?;
    for signum in given() {
        if getsignal.call1((signum,))?.is(handler) {
            // SAFETY: `signal` takes a number that Python's `signal` module named.
            unsafe { libc::signal(signum, libc::SIG_DFL) };
        }
    }
    Ok(())
}

/// Runs after finalization gives each signal its default action:
/// <https://github.com/python/cpython/blob/v3.15.2/Python/pylifecycle.c#L2465-L2466>
/// <https://github.com/python/cpython/blob/v3.15.2/Python/pylifecycle.c#L2598>
extern "C" fn die() {
    let signum = KEPT.load(Ordering::SeqCst);
    if signum != 0 {
        die_by(signum);
    }
}

/// Ends purba by the signal `signum`.
fn die_by(signum: i32) -> ! {
    #[cfg(not(windows))]
    {
        // SAFETY: `signal` and `raise` take a signal number that the kernel sent.
        unsafe {
            libc::signal(signum, libc::SIG_DFL);
            libc::raise(signum);
        }
        // `raise` returns only while the thread blocks the signal. purba then
        // exits with the code that a shell reports for a death by the signal.
        std::process::exit(128 + signum)
    }
    // CPython exits with this code after a Ctrl-C:
    // <https://github.com/python/cpython/blob/v3.15.2/Modules/main.c#L793-L815>
    #[cfg(windows)]
    {
        const STATUS_CONTROL_C_EXIT: u32 = 0xC000_013A;
        // Windows ends purba by a code, the same for each signal.
        let _ = signum;
        #[expect(
            clippy::cast_possible_wrap,
            reason = "Windows reads the code as the unsigned value"
        )]
        std::process::exit(STATUS_CONTROL_C_EXIT as i32)
    }
}

#[cfg(all(test, windows))]
mod tests {
    use super::*;

    #[test]
    fn the_interrupt_line_names_a_ctrl_c_and_a_ctrl_break() {
        assert_eq!(
            [shown("SIGINT"), shown("SIGBREAK"), shown("SIGTERM")],
            ["Ctrl-C", "Ctrl-Break", "SIGTERM"],
            "on Windows, purba's interrupt line names the key that sent SIGINT or SIGBREAK"
        );
    }
}
