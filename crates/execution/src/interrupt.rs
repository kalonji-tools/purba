//! purba's signal handler. It keeps the first interrupt of the process.
//!
//! It raises `KeyboardInterrupt` inside a test's call or a load, and while
//! Python waits for threads after the command. After each call and each load,
//! purba gives each interrupt signal purba's signal handler again, or the ignore
//! that the parent gave it. It then gives each interrupt signal the place in the
//! thread's mask that it had when `install` ran. The first interrupt gives each
//! interrupt signal that has purba's signal handler its default action. A native
//! handler sits in front of the C handler of `CPython`. As the first interrupt
//! signal arrives, it gives each interrupt signal that has the native handler its
//! default action, and prints purba's interrupt line, even while purba or a test
//! runs C code. The end of each later call or load gives each interrupt signal
//! that has purba's signal handler its default action again.
//! Once Python finalizes, purba dies by the interrupt that it kept, or else by
//! the one that arrived.
//!
//! `docs/decisions/a-process-that-a-test-forks-has-the-signal-handlers-it-has-without-purba.md`
//! says what a process that a test forks gets for a signal.
//!
//! `docs/decisions/an-interrupt-stops-the-run-and-a-second-one-exits-at-once.md`
//! says what a second interrupt does.

use std::io::{self, Write};
use std::sync::atomic::{AtomicBool, AtomicI32, AtomicUsize, Ordering};
use std::sync::{Mutex, PoisonError};

use pyo3::exceptions::PyKeyboardInterrupt;
use pyo3::sync::PyOnceLock;
use pyo3::types::{PyAnyMethods, PyCFunction, PyDict, PyDictMethods};
use pyo3::{Bound, Py, PyAny, PyResult, Python, pyclass, pymethods};

/// The number of the first signal that the signal handler took, or 0.
static KEPT: AtomicI32 = AtomicI32::new(0);

/// The number of the first interrupt signal that arrived, or 0. The native
/// handler sets it as the signal arrives. `KEPT` waits for purba's signal
/// handler, which runs at the next line of Python.
static ARRIVED: AtomicI32 = AtomicI32::new(0);

/// Whether a handler printed purba's interrupt line.
static PRINTED: AtomicBool = AtomicBool::new(false);

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
static UNMASKED: Mutex<Vec<i32>> = Mutex::new(Vec::new());

/// The number of each interrupt signal that the thread's mask held when
/// `install` ran.
static MASKED: Mutex<Vec<i32>> = Mutex::new(Vec::new());

/// The Python signal handler that each interrupt signal that `install` did not
/// find ignored had before `install` gave it purba's.
static BEFORE: Mutex<Vec<(i32, Py<PyAny>)>> = Mutex::new(Vec::new());

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

/// The interrupt signals.
const SIGNALS: [libc::c_int; 3] = [libc::SIGINT, libc::SIGTERM, libc::SIGHUP];

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
        // A signal that only the C handler of `CPython` took is noted as
        // arrived here.
        let _ = ARRIVED.compare_exchange(0, signum, Ordering::SeqCst, Ordering::SeqCst);
        if KEPT
            .compare_exchange(0, signum, Ordering::SeqCst, Ordering::SeqCst)
            .is_err()
        {
            // Two signals waited at once. Python took the second before purba
            // gave it the default action, so that action could not end purba.
            die_by(signum);
        }
        give_default_actions(py)?;
        if !PRINTED.swap(true, Ordering::SeqCst) {
            // stderr may refuse the write. The run stops either way.
            let _ = io::stderr().write_all(interrupt_line(signum));
        }
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
fn change_mask(how: libc::c_int, signals: &[i32]) {
    // SAFETY: each number is one of `SIGNALS`, or a signal that the kernel sent,
    // and `sigemptyset` initialized `set`.
    unsafe {
        let mut set = std::mem::zeroed::<libc::sigset_t>();
        libc::sigemptyset(&raw mut set);
        for &signum in signals {
            libc::sigaddset(&raw mut set, signum);
        }
        libc::pthread_sigmask(how, &raw const set, std::ptr::null_mut());
    }
}

/// The C handler of `CPython`, which marks a signal as waiting for its Python
/// handler, or 0 before `install`.
static CPYTHON_HANDLER: AtomicUsize = AtomicUsize::new(0);

/// The signals that `install` gave purba's signal handler, where the native
/// handler can read them. A slot that holds 0 holds no signal.
static NATIVE_GIVEN: [AtomicI32; SIGNALS.len()] = [const { AtomicI32::new(0) }; SIGNALS.len()];

/// The handler that the operating system runs for `signum`.
fn action(signum: i32) -> libc::sighandler_t {
    // SAFETY: `sigaction` with no new action only reads the current one into
    // `current`, which `zeroed` initialized.
    unsafe {
        let mut current = std::mem::zeroed::<libc::sigaction>();
        libc::sigaction(signum, std::ptr::null(), &raw mut current);
        current.sa_sigaction
    }
}

/// Gives `signum` the handler `handler` in the operating system, with the flags
/// that `CPython` gives its own C handler:
/// <https://github.com/python/cpython/blob/v3.15.2/Python/pylifecycle.c#L4002-L4006>
/// While `handler` runs, the operating system masks each interrupt signal. Two
/// signals that wait at once then reach it one after the other.
fn set_action(signum: i32, handler: libc::sighandler_t) {
    // SAFETY: `sigemptyset` initializes the mask of `new`, and each number is one
    // of `SIGNALS`, or a signal that the kernel sent.
    unsafe {
        let mut new = std::mem::zeroed::<libc::sigaction>();
        libc::sigemptyset(&raw mut new.sa_mask);
        for given in &NATIVE_GIVEN {
            let given = given.load(Ordering::SeqCst);
            if given != 0 {
                libc::sigaddset(&raw mut new.sa_mask, given);
            }
        }
        new.sa_flags = libc::SA_ONSTACK;
        new.sa_sigaction = handler;
        libc::sigaction(signum, &raw const new, std::ptr::null_mut());
    }
}

/// The native handler, as the operating system holds it.
fn native() -> libc::sighandler_t {
    native_handler as extern "C" fn(libc::c_int) as libc::sighandler_t
}

/// purba's interrupt line for `signum`, which the native handler and purba's
/// signal handler write.
const fn interrupt_line(signum: i32) -> &'static [u8] {
    match signum {
        libc::SIGINT => b"purba: interrupt: SIGINT; a second interrupt exits at once\n",
        libc::SIGTERM => b"purba: interrupt: SIGTERM; a second interrupt exits at once\n",
        libc::SIGHUP => b"purba: interrupt: SIGHUP; a second interrupt exits at once\n",
        _ => b"",
    }
}

/// Runs as an interrupt signal arrives, even while purba or a test runs C code,
/// and then calls the C handler of `CPython`. It runs no Python, and calls only
/// functions that a signal handler may call.
///
/// The C handler of `CPython` only marks the signal as waiting, and purba's
/// signal handler runs at the next line of Python:
/// <https://github.com/python/cpython/blob/v3.15.2/Modules/signalmodule.c#L338-L369>
/// So on the first interrupt, it gives each interrupt signal that has the native
/// handler its default action, and a second one ends purba at once. It gives the
/// interrupted code back its `errno`, as the C handler of `CPython` does.
extern "C" fn native_handler(signum: libc::c_int) {
    let errno = errno();
    // SAFETY: `errno` is the address of this thread's `errno`.
    let saved = unsafe { *errno };
    if ARRIVED
        .compare_exchange(0, signum, Ordering::SeqCst, Ordering::SeqCst)
        .is_err()
    {
        set_action(signum, libc::SIG_DFL);
        change_mask(libc::SIG_UNBLOCK, &[signum]);
        // SAFETY: `raise` takes a signal number that the kernel sent. `exit`
        // may not run in a signal handler, and `_exit` may.
        unsafe {
            libc::raise(signum);
            libc::_exit(128 + signum)
        }
    }
    for given in &NATIVE_GIVEN {
        let given = given.load(Ordering::SeqCst);
        if given != 0 && action(given) == native() {
            set_action(given, libc::SIG_DFL);
        }
    }
    if !PRINTED.swap(true, Ordering::SeqCst) {
        let line = interrupt_line(signum);
        // SAFETY: `write` reads `line.len()` bytes of `line`. stderr may refuse
        // the write, and the run stops either way.
        unsafe { libc::write(libc::STDERR_FILENO, line.as_ptr().cast(), line.len()) };
    }
    let cpython = CPYTHON_HANDLER.load(Ordering::SeqCst);
    if cpython != 0 {
        // SAFETY: `install` read this address from `sigaction`, where `CPython`
        // had put its C handler, which takes the number of the signal.
        let cpython = unsafe { std::mem::transmute::<usize, extern "C" fn(libc::c_int)>(cpython) };
        cpython(signum);
    }
    // SAFETY: `errno` is the address of this thread's `errno`.
    unsafe { *errno = saved };
}

/// The address of this thread's `errno`.
fn errno() -> *mut libc::c_int {
    #[cfg(target_os = "linux")]
    // SAFETY: the function only returns the address of this thread's `errno`.
    return unsafe { libc::__errno_location() };
    #[cfg(target_os = "macos")]
    // SAFETY: the function only returns the address of this thread's `errno`.
    return unsafe { libc::__error() };
}

/// Puts the native handler in front of the C handler of `CPython`, for each
/// signal that has purba's signal handler in Python's table. It replaces what
/// the operating system holds for that signal: the C handler of `CPython`, a
/// handler or an ignore that C code gave the signal, or the flag that
/// `signal.siginterrupt` gave it. A test that calls `signal.signal` replaces the
/// native handler.
fn put_native_in_front(py: Python<'_>) {
    let (Some(handler), Some(functions)) = (SIGNAL_HANDLER.get(py), FUNCTIONS.get(py)) else {
        return;
    };
    let cpython = CPYTHON_HANDLER.load(Ordering::SeqCst);
    if cpython == 0 {
        return;
    }
    for signum in given() {
        let purbas = functions
            .getsignal
            .bind(py)
            .call1((signum,))
            .is_ok_and(|current| current.is(handler));
        if purbas {
            set_action(signum, native());
        }
    }
}

/// Gives each interrupt signal purba's signal handler, unless purba's parent
/// ignores that signal.
///
/// A parent that starts purba in the background ignores SIGINT for it. The first
/// install also registers the function that makes purba die by the signal it
/// kept, once Python finalizes, and the function that gives a forked child back
/// the signal handlers that `install` kept. `install` keeps what `put_back` gives
/// back: the signals that the parent ignores, and the place of each signal in the
/// thread's mask.
///
/// # Errors
///
/// `signal` refused the signal handler, or `os.register_at_fork` refused the
/// function.
pub fn install() -> PyResult<()> {
    Python::initialize();
    Python::attach(|py| {
        let signal = py.import("_signal")?;
        let handler = SIGNAL_HANDLER.get_or_try_init(py, || {
            // SAFETY: `die` runs no Python, so it may run after finalization.
            unsafe { pyo3::ffi::Py_AtExit(Some(die)) };
            let in_child = PyCFunction::new_closure(py, None, None, |args, _kwargs| {
                give_back_in_child(args.py())
            })?;
            let kwargs = PyDict::new(py);
            kwargs.set_item("after_in_child", in_child)?;
            py.import("os")?
                .getattr("register_at_fork")?
                .call((), Some(&kwargs))?;
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
        for signum in SIGNALS {
            remember(if masked(signum) { &MASKED } else { &UNMASKED }, signum);
            let before = functions.getsignal.bind(py).call1((signum,))?;
            if before.is(ignore) {
                remember(&IGNORED, signum);
                continue;
            }
            if !before.is(handler) {
                let mut kept = BEFORE.lock().unwrap_or_else(PoisonError::into_inner);
                if !kept.iter().any(|(kept, _)| *kept == signum) {
                    kept.push((signum, before.unbind()));
                }
            }
            functions.signal.bind(py).call1((signum, handler))?;
            remember(&GIVEN, signum);
        }
        let given = given();
        for (slot, &signum) in NATIVE_GIVEN.iter().zip(&given) {
            slot.store(signum, Ordering::SeqCst);
        }
        if let Some(&signum) = given.first() {
            let _ = CPYTHON_HANDLER.compare_exchange(
                0,
                action(signum),
                Ordering::SeqCst,
                Ordering::SeqCst,
            );
        }
        put_native_in_front(py);
        Ok(())
    })
}

/// Runs in each process that `os.fork` makes. It gives each interrupt signal
/// that still has purba's signal handler the Python signal handler that the
/// signal had before `install`. `_signal.signal` also gives the operating system
/// the C handler of `CPython`, or the default action, so the operating system
/// drops purba's native handler too:
/// <https://github.com/python/cpython/blob/v3.15.2/Modules/signalmodule.c#L509-L529>
///
/// A signal whose handler Python did not install has no Python signal handler
/// to give back, and keeps purba's.
fn give_back_in_child(py: Python<'_>) -> PyResult<()> {
    let (Some(handler), Some(functions)) = (SIGNAL_HANDLER.get(py), FUNCTIONS.get(py)) else {
        return Ok(());
    };
    let before: Vec<(i32, Py<PyAny>)> = BEFORE
        .lock()
        .unwrap_or_else(PoisonError::into_inner)
        .iter()
        .map(|(signum, before)| (*signum, before.clone_ref(py)))
        .collect();
    for (signum, before) in before {
        if before.is_none(py) {
            continue;
        }
        if functions.getsignal.bind(py).call1((signum,))?.is(handler) {
            functions.signal.bind(py).call1((signum, before))?;
        }
    }
    Ok(())
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
/// another signal handler now. It also gives each signal that the parent ignores
/// the ignore in the operating system, where C code can give the signal a
/// handler that Python's table does not show. It then gives each signal the
/// place in the mask that it had at `install`. After the first signal, it then
/// gives each signal that has purba's signal handler its default action again.
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
    for &signum in &ignored {
        set_action(signum, libc::SIG_IGN);
    }
    put_back_mask();
    // A signal that arrived still waits for purba's signal handler, and it is
    // the first interrupt.
    if kept_now() || ARRIVED.load(Ordering::SeqCst) != 0 {
        // Its error has no test to charge either.
        drop(give_default_actions(py));
    } else {
        put_native_in_front(py);
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
            // SAFETY: `signal` takes one of `SIGNALS`.
            unsafe { libc::signal(signum, libc::SIG_DFL) };
        }
    }
    Ok(())
}

/// Runs after finalization gives each signal its default action:
/// <https://github.com/python/cpython/blob/v3.15.2/Python/pylifecycle.c#L2465-L2466>
/// <https://github.com/python/cpython/blob/v3.15.2/Python/pylifecycle.c#L2598>
/// It ends purba by the signal that purba's signal handler kept, or else by the
/// one that arrived.
extern "C" fn die() {
    let signum = match KEPT.load(Ordering::SeqCst) {
        0 => ARRIVED.load(Ordering::SeqCst),
        kept => kept,
    };
    if signum != 0 {
        die_by(signum);
    }
}

/// Ends purba by the signal `signum`.
fn die_by(signum: i32) -> ! {
    // SAFETY: `signal` takes a signal number that the kernel sent.
    unsafe { libc::signal(signum, libc::SIG_DFL) };
    // A mask can hold the signal: the parent's, which `put_back` gave back, or
    // one that a test or its `atexit` function set. So purba unmasks it before
    // `raise`.
    change_mask(libc::SIG_UNBLOCK, &[signum]);
    // SAFETY: `raise` takes a signal number that the kernel sent.
    unsafe { libc::raise(signum) };
    // `raise` returns only when the unmask failed. purba then exits with the
    // code that a shell reports for a death by the signal.
    std::process::exit(128 + signum)
}
