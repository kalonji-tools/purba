//! purba's signal handler. It keeps the first interrupt of the process.
//!
//! It raises `KeyboardInterrupt` inside a test's call or a load, and while
//! Python waits for threads after the command. After each call and each load,
//! purba gives each interrupt signal purba's signal handler again, or the ignore
//! that the parent gave it. On Unix, it then gives each interrupt signal the
//! place in the thread's mask that it had when `install` ran. The first
//! interrupt gives each interrupt signal its default action. The end of each
//! later call or load gives each interrupt signal its default action again. Once
//! Python finalizes, purba dies by the interrupt that it kept.
//!
//! `docs/decisions/an-interrupt-stops-the-run-and-a-second-one-exits-at-once.md`
//! says what a second interrupt does.

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

/// The number of each interrupt signal that purba's parent ignores.
static IGNORED: Mutex<Vec<i32>> = Mutex::new(Vec::new());

/// The number of each interrupt signal that the thread's mask did not hold
/// when `install` ran.
#[cfg(unix)]
static UNMASKED: Mutex<Vec<i32>> = Mutex::new(Vec::new());

/// The number of each interrupt signal that the thread's mask held when
/// `install` ran.
#[cfg(unix)]
static MASKED: Mutex<Vec<i32>> = Mutex::new(Vec::new());

/// The functions and the ignore of `_signal` that `install` found.
/// `signal.signal` calls `_signal.signal` by its name in the module `signal`, so
/// a test that patches either module reaches it, but not these:
/// <https://github.com/python/cpython/blob/v3.15.2/Lib/signal.py#L56-L59>
struct Functions {
    signal: Py<PyAny>,
    getsignal: Py<PyAny>,
    ignore: Py<PyAny>,
}

static FUNCTIONS: PyOnceLock<Functions> = PyOnceLock::new();

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

/// Adds `signum` to `list`, once.
fn remember(list: &Mutex<Vec<i32>>, signum: i32) {
    let mut list = list.lock().unwrap_or_else(PoisonError::into_inner);
    if !list.contains(&signum) {
        list.push(signum);
    }
}

/// Whether the thread's mask holds `signum`.
#[cfg(unix)]
fn masked(signum: i32) -> bool {
    // SAFETY: `pthread_sigmask` with no new set only reads the mask into `mask`,
    // which `sigemptyset` initialized.
    unsafe {
        let mut mask = std::mem::zeroed::<libc::sigset_t>();
        libc::sigemptyset(&raw mut mask);
        libc::pthread_sigmask(libc::SIG_BLOCK, std::ptr::null(), &raw mut mask);
        libc::sigismember(&raw const mask, signum) == 1
    }
}

/// Gives each interrupt signal the place in the thread's mask that it had when
/// `install` ran. A signal that only a test's mask held then reaches its signal
/// handler.
#[cfg(unix)]
fn put_back_mask() {
    let read = |signals: &Mutex<Vec<i32>>| {
        signals
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .clone()
    };
    change_mask(libc::SIG_BLOCK, &read(&MASKED));
    change_mask(libc::SIG_UNBLOCK, &read(&UNMASKED));
}

/// Masks or unmasks, by `how`, each signal in `signals`.
#[cfg(unix)]
fn change_mask(how: libc::c_int, signals: &[i32]) {
    // SAFETY: each number is a signal that Python's `_signal` module named, or
    // that the kernel sent, and `sigemptyset` initialized `set`.
    unsafe {
        let mut set = std::mem::zeroed::<libc::sigset_t>();
        libc::sigemptyset(&raw mut set);
        for &signum in signals {
            libc::sigaddset(&raw mut set, signum);
        }
        libc::pthread_sigmask(how, &raw const set, std::ptr::null_mut());
    }
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
/// kept, once Python finalizes. `install` keeps what `put_back` gives back: the
/// signals that the parent ignores and, on Unix, the place of each signal in the
/// thread's mask.
///
/// # Errors
///
/// `signal` refused the signal handler.
pub fn install() -> PyResult<()> {
    Python::initialize();
    Python::attach(|py| {
        let signal = py.import("_signal")?;
        let handler = SIGNAL_HANDLER.get_or_try_init(py, || {
            // SAFETY: `die` runs no Python, so it may run after finalization.
            unsafe { pyo3::ffi::Py_AtExit(Some(die)) };
            Py::new(py, SignalHandler)
        })?;
        let functions = FUNCTIONS.get_or_try_init(py, || -> PyResult<_> {
            Ok(Functions {
                signal: signal.getattr("signal")?.unbind(),
                getsignal: signal.getattr("getsignal")?.unbind(),
                ignore: signal.getattr("SIG_IGN")?.unbind(),
            })
        })?;
        let ignore = functions.ignore.bind(py);
        for name in NAMES {
            let signum = signal.getattr(name)?.extract::<i32>()?;
            #[cfg(unix)]
            remember(if masked(signum) { &MASKED } else { &UNMASKED }, signum);
            if functions.getsignal.bind(py).call1((signum,))?.is(ignore) {
                remember(&IGNORED, signum);
                continue;
            }
            functions.signal.bind(py).call1((signum, handler))?;
            remember(&GIVEN, signum);
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
/// Then gives each interrupt signal back what `put_back` names, if the test or
/// the module changed it.
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

/// Gives purba's signal handler back to each signal that `install` gave it, and
/// the ignore back to each signal that the parent ignores, when the signal has
/// another signal handler now. On Unix, it then gives each signal the place in
/// the mask that it had at `install`. After the first signal, it then gives each
/// signal its default action again.
fn put_back(py: Python<'_>) {
    let (Some(handler), Some(functions)) = (SIGNAL_HANDLER.get(py), FUNCTIONS.get(py)) else {
        return;
    };
    let ignore = functions.ignore.bind(py);
    let given = given();
    let ignored = IGNORED
        .lock()
        .unwrap_or_else(PoisonError::into_inner)
        .clone();
    let give = || -> PyResult<()> {
        let wanted = given
            .iter()
            .map(|&signum| (signum, handler.bind(py).as_any()))
            .chain(ignored.iter().map(|&signum| (signum, ignore)));
        for (signum, wanted) in wanted {
            if !functions.getsignal.bind(py).call1((signum,))?.is(wanted) {
                functions.signal.bind(py).call1((signum, wanted))?;
            }
        }
        Ok(())
    };
    // `_signal.signal` first runs a signal that waits, through the signal
    // handler that the signal has, and that one may raise:
    // <https://github.com/python/cpython/blob/v3.15.2/Modules/signalmodule.c#L522-L525>
    // The signal no longer waits then, so a second try gives each signal back
    // what it had. Its error has no test to charge.
    if give().is_err() {
        drop(give());
    }
    #[cfg(unix)]
    put_back_mask();
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
    let (Some(handler), Some(functions)) = (SIGNAL_HANDLER.get(py), FUNCTIONS.get(py)) else {
        return Ok(());
    };
    for signum in given() {
        if functions.getsignal.bind(py).call1((signum,))?.is(handler) {
            // SAFETY: `signal` takes a number that Python's `_signal` module named.
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
        // SAFETY: `signal` takes a signal number that the kernel sent.
        unsafe { libc::signal(signum, libc::SIG_DFL) };
        // A mask can hold the signal: the parent's, which `put_back` gave back,
        // or one that a test or its `atexit` function set. So purba unmasks it
        // before `raise`.
        #[cfg(unix)]
        change_mask(libc::SIG_UNBLOCK, &[signum]);
        // SAFETY: `raise` takes a signal number that the kernel sent.
        unsafe { libc::raise(signum) };
        // `raise` returns only when the unmask failed. purba then exits with the
        // code that a shell reports for a death by the signal.
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
