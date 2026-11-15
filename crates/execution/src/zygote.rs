//! The zygote: a process that purba forks for one module. It loads the module
//! once, then forks a process for each test that purba names to it. Each process
//! writes what it found to purba as frames, on one pipe.
//!
//! When a Python thread other than the main one is alive after the load, the
//! zygote forks no test. purba then forks a zygote that loads nothing, and each
//! test's process loads the module before its test.

use std::fs::File;
use std::io::{self, ErrorKind};
use std::os::fd::{FromRawFd, OwnedFd};
use std::panic::AssertUnwindSafe;
use std::path::PathBuf;

use pyo3::Python;
use pyo3::types::PyAnyMethods;

use crate::fork::{Forked, fork};
use purba_assertion::{Readers, Verdict};

use crate::{Error, Kind, Module, Outcome, Owner, Reason, Worker, frame, interrupt, panic, reader};

/// The tags of the frames.
mod tag {
    /// purba names the test to run: the name of its function.
    pub const TEST: u8 = b'n';
    /// purba asks for a replay of a test: the name of its function, a NUL,
    /// and the index of the expectation that the replay reads alone.
    pub const REPLAY: u8 = b'y';
    /// Each later expectation that a replay must read alone: its index, `1`
    /// when it held in order or `0`, and its text, each after a NUL but the
    /// first.
    pub const LATER: u8 = b'q';
    /// The load ended, and the zygote forks each test from now on.
    pub const READY: u8 = b'g';
    /// A thread is alive after the load, and the zygote ends.
    pub const THREADED: u8 = b't';
    /// The load returned, and kept its module or met an interrupt.
    pub const LOADED: u8 = b'l';
    /// The test's process started: its pid.
    pub const FORKED: u8 = b'c';
    pub const PASSED: u8 = b'p';
    pub const INTERRUPTED: u8 = b'i';
    /// The text of each assert that did not hold, each after a NUL but the first.
    /// `compile` refuses a source that holds a NUL, so no text holds one:
    /// <https://docs.python.org/3.12/library/functions.html#compile>
    pub const FAILED: u8 = b'f';
    /// The exception, as text.
    pub const RAISED: u8 = b'r';
    /// A project error, as text.
    pub const PROJECT: u8 = b'e';
    /// An internal error: the part, a line feed, and the message.
    pub const INTERNAL: u8 = b'x';
    /// The error of a thread that the test left, as text.
    pub const TEST_THREAD: u8 = b'h';
    /// The error of a thread that the load left, as text.
    pub const LOAD_THREAD: u8 = b'H';
    /// The first interrupt that the process kept: the number of its signal.
    pub const KEPT: u8 = b'k';
    /// The test's process ended: its wait status.
    pub const ENDED: u8 = b'w';
}

/// A zygote that purba forked, seen from purba.
pub struct Zygote {
    pid: libc::pid_t,
    path: PathBuf,
    tests: Option<File>,
    frames: File,
}

/// What the load of a zygote gave: its result, the error of each thread that
/// the load left, and whether a thread is alive after it.
pub struct Load {
    pub result: Result<(), Error>,
    pub threads: Vec<Error>,
    pub threaded: bool,
}

/// The error of each thread that a test's process found, beside its owner.
type Threads<T> = Vec<(Owner<T>, Error)>;

impl<T> Worker<'_, T> {
    /// Forks a zygote for `module`. With `load`, the zygote loads the module
    /// once, and each test's process runs its test. Without it, each test's
    /// process loads the module, then runs its test.
    ///
    /// # Errors
    ///
    /// A project error when the operating system refused a pipe or the fork.
    pub fn zygote(&self, module: &Module, load: bool) -> Result<Zygote, Error> {
        let fork_error = |error: io::Error| Error::project(format!("purba cannot fork: {error}"));
        let (tests_in, tests_out) = pipe().map_err(fork_error)?;
        let (frames_in, frames_out) = pipe().map_err(fork_error)?;
        flush(self.py);
        match fork(self.py).map_err(fork_error)? {
            Forked::Child => {
                drop((tests_out, frames_in));
                // The life of the child unwinds a panic before it reaches this
                // frame, so the error goes on a copy of the pipe.
                let spare = frames_out.try_clone().map(File::from);
                let payload = std::panic::catch_unwind(AssertUnwindSafe(|| {
                    live_as_zygote(
                        self.py,
                        module,
                        load,
                        File::from(tests_in),
                        File::from(frames_out),
                    );
                }))
                .err();
                if let (Ok(mut spare), Some(payload)) = (spare, payload) {
                    let (tag, text) = error_frame(&Error::caught(payload.as_ref(), "execution"));
                    drop(frame::write(&mut spare, tag, &text));
                }
                exit(125)
            }
            Forked::Parent(pid) => {
                interrupt::follow_zygote(pid);
                Ok(Zygote {
                    pid,
                    path: module.path.clone(),
                    tests: Some(File::from(tests_out)),
                    frames: File::from(frames_in),
                })
            }
        }
    }
}

impl Zygote {
    /// Reads what the load gave. A zygote that loads nothing gives a load that
    /// returned.
    ///
    /// A zygote that ends before its load ends gives a project error that names
    /// the signal or the exit code, unless the first interrupt arrived as it
    /// loaded.
    pub fn loaded(&mut self) -> Load {
        let mut load = Load {
            result: Ok(()),
            threads: Vec::new(),
            threaded: false,
        };
        let before = interrupt::first();
        loop {
            match frame::read(&mut self.frames) {
                Ok(Some((tag::READY, _))) => return load,
                Ok(Some((tag::THREADED, _))) => {
                    load.threaded = true;
                    return load;
                }
                Ok(Some((tag::LOADED, _))) => load.result = Ok(()),
                Ok(Some((tag::LOAD_THREAD, text))) => load.threads.push(Error::project(text)),
                Ok(Some((tag::KEPT, signum))) => interrupt::adopt(signum.parse().unwrap_or(0)),
                Ok(Some((tag, text))) => {
                    load.result = Err(error_of(tag, text));
                }
                Ok(None) | Err(_) => {
                    let lost = lost(self.wait(), "its load ended");
                    if load.result.is_ok() && (before != 0 || interrupt::first() == 0) {
                        load.result = Err(Error::project(lost));
                    }
                    return load;
                }
            }
        }
    }

    /// Asks the zygote to fork a process for the test `function`, and reads its
    /// outcome and the error of each thread that it found. `owner` owns each
    /// thread that the test left. A process that ended with no outcome gives
    /// `Raised`, which names the signal or the exit code, or `Interrupted` when
    /// the first interrupt arrived as it ran. A process that an interrupt signal
    /// ended ends purba by that signal.
    pub fn run<T: Clone>(
        &mut self,
        function: &str,
        owner: T,
    ) -> (Result<Outcome, Error>, Threads<T>) {
        let mut threads = Vec::new();
        let asked = self.tests.as_mut().map_or_else(
            || Err(ErrorKind::BrokenPipe.into()),
            |tests| frame::write(tests, tag::TEST, function),
        );
        if asked.is_err() {
            let ended = ended(self.wait());
            return (
                Err(Error::project(format!("the module's process {ended}"))),
                threads,
            );
        }
        let before = interrupt::first();
        let mut result = None;
        let mut later = Vec::new();
        let status = loop {
            match frame::read(&mut self.frames) {
                Ok(Some((tag::ENDED, status))) => break status.parse().ok(),
                Ok(Some((tag::LATER, text))) => later = later_of(&text),
                Ok(Some((tag::FORKED, pid))) => {
                    interrupt::follow_test(pid.parse().unwrap_or(0));
                }
                Ok(Some((tag::TEST_THREAD, text))) => {
                    threads.push((Owner::Test(owner.clone()), Error::project(text)));
                }
                Ok(Some((tag::LOAD_THREAD, text))) => {
                    threads.push((Owner::Load(self.path.clone()), Error::project(text)));
                }
                Ok(Some((tag::KEPT, signum))) => interrupt::adopt(signum.parse().unwrap_or(0)),
                Ok(Some((tag, text))) => result = Some(outcome_of(tag, text)),
                Ok(None) | Err(_) => {
                    interrupt::follow_test(0);
                    let ended = ended(self.wait());
                    return (
                        result.unwrap_or_else(|| {
                            Err(Error::project(format!("the module's process {ended}")))
                        }),
                        threads,
                    );
                }
            }
        };
        interrupt::follow_test(0);
        if let Some(status) = status
            && libc::WIFSIGNALED(status)
        {
            interrupt::die_with(libc::WTERMSIG(status));
        }
        let result = result.unwrap_or_else(|| {
            let kind = if before == 0 && interrupt::first() != 0 {
                Kind::Interrupted
            } else {
                Kind::Raised {
                    exception: status.map_or_else(
                        || "its process ended before its outcome".to_owned(),
                        |status| lost(status, "its outcome"),
                    ),
                }
            };
            Ok(Outcome { kind })
        });
        let result = match result {
            Ok(outcome)
                if !later.is_empty()
                    && matches!(outcome.kind, Kind::Passed | Kind::Failed { .. }) =>
            {
                self.replays(function, &later, &owner, &mut threads)
                    .unwrap_or(Ok(outcome))
            }
            other => other,
        };
        (result, threads)
    }

    /// Asks the zygote for a replay of `function` for each of `later`, in
    /// order, and gives the result that the first replay whose reading differs
    /// from the one in order makes: a project error. A replay that the zygote
    /// could not fork gives a project error too, and an interrupt that lands
    /// during the replays gives `Interrupted`. Gives `None` when every replay
    /// agrees.
    fn replays<T: Clone>(
        &mut self,
        function: &str,
        later: &[(usize, bool, String)],
        owner: &T,
        threads: &mut Threads<T>,
    ) -> Option<Result<Outcome, Error>> {
        let before = interrupt::first();
        for (index, held, text) in later {
            let verdict = match self.replay(function, *index, owner, threads) {
                Ok(verdict) => verdict,
                Err(error) => return Some(Err(error)),
            };
            if before == 0 && interrupt::first() != 0 {
                return Some(Ok(Outcome {
                    kind: Kind::Interrupted,
                }));
            }
            if let Some(differ) = crate::differs(verdict, *held) {
                return Some(Err(Error::project(format!("{text} {differ}"))));
            }
        }
        None
    }

    /// Asks the zygote for one replay of `function` that reads the expectation
    /// at `index` alone, and gives its verdict.
    fn replay<T: Clone>(
        &mut self,
        function: &str,
        index: usize,
        owner: &T,
        threads: &mut Threads<T>,
    ) -> Result<Verdict, Error> {
        let asked = self.tests.as_mut().map_or_else(
            || Err(ErrorKind::BrokenPipe.into()),
            |tests| frame::write(tests, tag::REPLAY, &format!("{function}\0{index}")),
        );
        if asked.is_err() {
            let ended = ended(self.wait());
            return Err(Error::project(format!("the module's process {ended}")));
        }
        let mut refused = None;
        loop {
            match frame::read(&mut self.frames) {
                Ok(Some((tag::ENDED, status))) => {
                    interrupt::follow_test(0);
                    return refused.map_or_else(
                        || Ok(status.parse().map_or(Verdict::Raises, reader::verdict)),
                        |text| Err(Error::project(text)),
                    );
                }
                Ok(Some((tag::FORKED, pid))) => {
                    interrupt::follow_test(pid.parse().unwrap_or(0));
                }
                Ok(Some((tag::TEST_THREAD, text))) => {
                    threads.push((Owner::Test(owner.clone()), Error::project(text)));
                }
                Ok(Some((tag::LOAD_THREAD, text))) => {
                    threads.push((Owner::Load(self.path.clone()), Error::project(text)));
                }
                Ok(Some((tag::KEPT, signum))) => interrupt::adopt(signum.parse().unwrap_or(0)),
                Ok(Some((tag::PROJECT, text))) => refused = Some(text),
                Ok(Some(_)) => {}
                Ok(None) | Err(_) => {
                    interrupt::follow_test(0);
                    let ended = ended(self.wait());
                    return Err(Error::project(format!("the module's process {ended}")));
                }
            }
        }
    }

    /// Tells the zygote that no test is left, and waits for it to end.
    pub fn end(mut self) {
        self.wait();
    }

    /// Closes the pipe of test names, and waits for the zygote to end. Gives
    /// its wait status.
    fn wait(&mut self) -> libc::c_int {
        self.tests = None;
        let status = wait(self.pid);
        interrupt::follow_zygote(0);
        if libc::WIFSIGNALED(status) {
            interrupt::die_with(libc::WTERMSIG(status));
        }
        status
    }
}

/// What a process that ended before `what` gives, from its wait status: `its
/// process died by SIGABRT`, or `its process exited 3 before <what>`.
fn lost(status: libc::c_int, what: &str) -> String {
    if libc::WIFSIGNALED(status) {
        format!("its process {}", ended(status))
    } else {
        format!("its process {} before {what}", ended(status))
    }
}

/// The text of how a process ended, from its wait status: `died by SIGABRT`,
/// or `exited 3`.
fn ended(status: libc::c_int) -> String {
    if libc::WIFSIGNALED(status) {
        format!("died by {}", signal_name(libc::WTERMSIG(status)))
    } else {
        format!("exited {}", libc::WEXITSTATUS(status))
    }
}

/// The name of `signum`, as `kill -l` gives it with `SIG` in front.
fn signal_name(signum: libc::c_int) -> String {
    let name = match signum {
        libc::SIGABRT => "SIGABRT",
        libc::SIGALRM => "SIGALRM",
        libc::SIGBUS => "SIGBUS",
        libc::SIGFPE => "SIGFPE",
        libc::SIGHUP => "SIGHUP",
        libc::SIGILL => "SIGILL",
        libc::SIGINT => "SIGINT",
        libc::SIGKILL => "SIGKILL",
        libc::SIGPIPE => "SIGPIPE",
        libc::SIGQUIT => "SIGQUIT",
        libc::SIGSEGV => "SIGSEGV",
        libc::SIGSYS => "SIGSYS",
        libc::SIGTERM => "SIGTERM",
        libc::SIGTRAP => "SIGTRAP",
        libc::SIGUSR1 => "SIGUSR1",
        libc::SIGUSR2 => "SIGUSR2",
        _ => return format!("signal {signum}"),
    };
    name.to_owned()
}

/// The error that the frame of `tag` and `text` carries.
fn error_of(tag: u8, text: String) -> Error {
    if tag == tag::INTERNAL {
        let (part, message) = text.split_once('\n').unwrap_or(("execution", &text));
        Error(Reason::Internal(panic::Caught {
            part: part.to_owned(),
            message: message.to_owned(),
        }))
    } else {
        Error::project(text)
    }
}

/// The outcome or the error that the frame of `tag` and `text` carries.
fn outcome_of(tag: u8, text: String) -> Result<Outcome, Error> {
    let kind = match tag {
        tag::PASSED => Kind::Passed,
        tag::INTERRUPTED => Kind::Interrupted,
        tag::FAILED => Kind::Failed {
            asserts: text.split('\0').map(str::to_owned).collect(),
        },
        tag::RAISED => Kind::Raised { exception: text },
        _ => return Err(error_of(tag, text)),
    };
    Ok(Outcome { kind })
}

/// The frame of an error.
fn error_frame(error: &Error) -> (u8, String) {
    match &error.0 {
        Reason::Project(text) => (tag::PROJECT, text.clone()),
        Reason::Internal(caught) => (
            tag::INTERNAL,
            format!("{}\n{}", caught.part, caught.message),
        ),
    }
}

/// The frame of an outcome or of the error in its place.
fn outcome_frame(result: &Result<Kind, Error>) -> (u8, String) {
    match result {
        Ok(Kind::Passed) => (tag::PASSED, String::new()),
        Ok(Kind::Interrupted) => (tag::INTERRUPTED, String::new()),
        Ok(Kind::Failed { asserts }) => (tag::FAILED, asserts.join("\0")),
        Ok(Kind::Raised { exception }) => (tag::RAISED, exception.clone()),
        Err(error) => error_frame(error),
    }
}

/// A pipe whose ends a process closes as it runs another program.
fn pipe() -> io::Result<(OwnedFd, OwnedFd)> {
    let mut ends = [0; 2];
    // SAFETY: `pipe` writes two descriptors into `ends`.
    if unsafe { libc::pipe(ends.as_mut_ptr()) } != 0 {
        return Err(io::Error::last_os_error());
    }
    // SAFETY: `pipe` opened both descriptors, and nothing else owns them.
    let ends = unsafe { (OwnedFd::from_raw_fd(ends[0]), OwnedFd::from_raw_fd(ends[1])) };
    for end in [&ends.0, &ends.1] {
        use std::os::fd::AsRawFd;
        // SAFETY: `end` is an open descriptor.
        if unsafe { libc::fcntl(end.as_raw_fd(), libc::F_SETFD, libc::FD_CLOEXEC) } != 0 {
            return Err(io::Error::last_os_error());
        }
    }
    Ok(ends)
}

/// Waits for the child `pid` to end, and gives its wait status.
pub fn wait(pid: libc::pid_t) -> libc::c_int {
    let mut status = 0;
    loop {
        // SAFETY: `waitpid` writes the status of `pid` into `status`.
        let waited = unsafe { libc::waitpid(pid, &raw mut status, 0) };
        if waited == pid {
            return status;
        }
        if waited < 0 && io::Error::last_os_error().kind() != ErrorKind::Interrupted {
            return status;
        }
    }
}

/// Writes what Python buffered for stdout and stderr, so that no child writes
/// it again. A stream that is gone or that refuses the write keeps its buffer.
pub fn flush(py: Python<'_>) {
    if let Ok(sys) = py.import("sys") {
        for name in ["stdout", "stderr"] {
            if let Ok(stream) = sys.getattr(name) {
                drop(stream.call_method0("flush"));
            }
        }
    }
}

/// Writes a frame, and ends the process when the pipe refuses it: purba no
/// longer reads.
fn send(frames: &mut File, (tag, text): (u8, String)) {
    if frame::write(frames, tag, &text).is_err() {
        exit(0);
    }
}

/// Calls `function` of the module `atexit`. Its error has no test to charge.
fn call_atexit(py: Python<'_>, function: &str) {
    drop(
        py.import("atexit")
            .and_then(|atexit| atexit.getattr(function)?.call0()),
    );
}

/// Ends the zygote: runs each `atexit` function that its load registered, and
/// writes what Python buffered.
fn end(py: Python<'_>) -> ! {
    call_atexit(py, "_run_exitfuncs");
    flush(py);
    exit(0)
}

/// Ends the process now, with no `atexit` function and no wait for threads.
fn exit(code: libc::c_int) -> ! {
    // SAFETY: `_exit` takes any code, and ends the process.
    unsafe { libc::_exit(code) }
}

/// The zygote's life, in the child of the fork.
fn live_as_zygote(
    py: Python<'_>,
    module: &Module,
    load: bool,
    mut tests: File,
    mut frames: File,
) -> ! {
    let mut worker = Worker::<String>::new(py);
    call_atexit(py, "_clear");
    if load {
        let result = crate::catch("execution", || worker.load(module));
        match &result {
            Ok(()) => send(&mut frames, (tag::LOADED, String::new())),
            Err(error) => send(&mut frames, error_frame(error)),
        }
        send_threads(&mut worker, &mut frames);
        send_kept(&mut frames);
        if result.is_err() {
            send(&mut frames, (tag::READY, String::new()));
            end(py);
        }
        if threaded(py) {
            send(&mut frames, (tag::THREADED, String::new()));
            end(py);
        }
    }
    send(&mut frames, (tag::READY, String::new()));
    while let Ok(Some((tag, text))) = frame::read(&mut tests) {
        let (function, replay) = match tag {
            tag::TEST => (text, None),
            tag::REPLAY => match text.split_once('\0') {
                Some((function, index)) => (function.to_owned(), index.parse().ok()),
                None => break,
            },
            _ => break,
        };
        flush(py);
        match fork(py) {
            Ok(Forked::Child) => {
                drop(tests);
                match replay {
                    Some(index) => {
                        live_as_replay(py, &mut worker, module, !load, &function, index, &frames)
                    }
                    None => live_as_test(py, &mut worker, module, !load, &function, frames),
                }
            }
            Ok(Forked::Parent(pid)) => {
                let status = wait(pid);
                send(&mut frames, (tag::ENDED, status.to_string()));
            }
            Err(error) => {
                let text = if replay.is_some() {
                    format!("purba cannot fork to read an expectation alone: {error}")
                } else {
                    format!("purba cannot fork: {error}")
                };
                send(&mut frames, (tag::PROJECT, text));
                send(&mut frames, (tag::ENDED, "0".to_owned()));
            }
        }
    }
    end(py)
}

/// Whether a Python thread other than the main one is alive.
pub fn threaded(py: Python<'_>) -> bool {
    let Ok(main) = py
        .import("threading")
        .and_then(|threading| threading.getattr("main_thread")?.call0())
    else {
        return false;
    };
    let Ok(hook) = crate::thread::hook(py) else {
        return false;
    };
    hook.get()
        .alive(py)
        .iter()
        .any(|thread| !thread.bind(py).is(&main))
}

/// The life of a test's process, in the child of the zygote. With `load`, it
/// loads the module first. A process that the test forks and that returns from
/// the test ends with no frame.
fn live_as_test(
    py: Python<'_>,
    worker: &mut Worker<'_, String>,
    module: &Module,
    load: bool,
    function: &str,
    mut frames: File,
) -> ! {
    // SAFETY: `getpid` only gives the pid of the process.
    let own = unsafe { libc::getpid() };
    send(&mut frames, (tag::FORKED, own.to_string()));
    call_atexit(py, "_clear");
    let result = crate::catch("execution", || {
        if load {
            worker.load(module)?;
            // The load in the first zygote already handed on the errors of these
            // threads.
            drop(worker.threads());
            if interrupt::kept_now() {
                return Ok(Kind::Interrupted);
            }
        }
        worker
            .run(module, function, function.to_owned())
            .map(|outcome| outcome.kind)
    });
    // SAFETY: `getpid` only gives the pid of the process.
    if unsafe { libc::getpid() } != own {
        exit(0);
    }
    if result.is_ok() && !worker.replays().is_empty() {
        let later: Vec<String> = worker
            .replays()
            .iter()
            .flat_map(|(index, held, text)| {
                [index.to_string(), u8::from(*held).to_string(), text.clone()]
            })
            .collect();
        send(&mut frames, (tag::LATER, later.join("\0")));
    }
    send(&mut frames, outcome_frame(&result));
    interrupt::calling(py, || join(py, worker));
    call_atexit(py, "_run_exitfuncs");
    send_threads(worker, &mut frames);
    send_kept(&mut frames);
    flush(py);
    exit(0)
}

/// Joins each thread that is alive and is not a daemon, until an interrupt
/// raises in a join.
fn join(py: Python<'_>, worker: &Worker<'_, String>) {
    let Ok(main) = py
        .import("threading")
        .and_then(|threading| threading.getattr("main_thread")?.call0())
    else {
        return;
    };
    for thread in worker.hook.get().alive(py) {
        let thread = thread.bind(py);
        let daemon = thread
            .getattr("daemon")
            .and_then(|daemon| daemon.is_truthy())
            .unwrap_or(true);
        if !thread.is(&main) && !daemon && thread.call_method0("join").is_err() {
            return;
        }
    }
}

/// Writes the first interrupt that the process kept, if it kept one.
fn send_kept(frames: &mut File) {
    let signum = interrupt::first();
    if signum != 0 {
        send(frames, (tag::KEPT, signum.to_string()));
    }
}

/// Writes the error of each thread that the worker took.
fn send_threads(worker: &mut Worker<'_, String>, frames: &mut File) {
    for (owner, error) in worker.threads() {
        let tag = match owner {
            Owner::Load(_) => tag::LOAD_THREAD,
            Owner::Test(_) => tag::TEST_THREAD,
        };
        send(frames, (tag, error.to_string()));
    }
}

/// Each later expectation that a `LATER` frame names: its index, whether it
/// held in order, and its text.
fn later_of(text: &str) -> Vec<(usize, bool, String)> {
    let parts: Vec<&str> = text.split('\0').collect();
    let (triples, _) = parts.as_chunks::<3>();
    triples
        .iter()
        .filter_map(|[index, held, text]| {
            Some((index.parse().ok()?, *held == "1", (*text).to_owned()))
        })
        .collect()
}

/// The life of a replay, in the child of the zygote: it runs the test
/// `function` again, and reads only the expectation at `index`. With `load`,
/// it loads the module first. It ends with the verdict of that expectation as
/// its exit code, and writes nothing to stdout or stderr: the case wrote what
/// the test writes once.
fn live_as_replay(
    py: Python<'_>,
    worker: &mut Worker<'_, String>,
    module: &Module,
    load: bool,
    function: &str,
    index: usize,
    frames: &File,
) -> ! {
    // SAFETY: `getpid` only gives the pid of the process.
    let own = unsafe { libc::getpid() };
    let mut frames = frames.try_clone().unwrap_or_else(|_| exit(0));
    send(&mut frames, (tag::FORKED, own.to_string()));
    drop(frames);
    if let Ok(null) = File::options().write(true).open("/dev/null") {
        use std::os::fd::AsRawFd;
        for target in [libc::STDOUT_FILENO, libc::STDERR_FILENO] {
            // SAFETY: both descriptors are open, and `dup2` only points
            // `target` at the file of `null`.
            unsafe { libc::dup2(null.as_raw_fd(), target) };
        }
    }
    call_atexit(py, "_clear");
    drop(crate::catch("execution", || {
        if load {
            worker.load(module)?;
            drop(worker.threads());
        }
        worker.replay(module, function, index);
        Ok(())
    }));
    // SAFETY: `getpid` only gives the pid of the process.
    if unsafe { libc::getpid() } != own {
        exit(0);
    }
    reader::READERS.end(Verdict::Raises)
}
