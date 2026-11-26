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
use std::sync::{Mutex, PoisonError};

use pyo3::Python;
use pyo3::types::PyAnyMethods;

use purba_assertion::{Readers, Replaced, Verdict};
use purba_errors::catch;

use crate::fork::{Forked, fork};
use crate::frame::Message;
use crate::interrupt::Window;
use crate::order::{Case, Decided, Facts};
use crate::step::Met;
use crate::{Error, Module, Outcome, Purba, Step, Suppressed, Worker, interrupt, outcome, thread};

/// A zygote that purba forked, seen from purba. A dropped zygote ends, and
/// purba waits for it.
pub struct Zygote {
    pid: libc::pid_t,
    tests: Option<File>,
    frames: File,
    /// The wait status of the zygote, once purba reaped it.
    reaped: Option<libc::c_int>,
    /// Whether the line of the last case names the end of the module's
    /// process.
    named: bool,
}

/// The end of the module's process, which a case met: its error, and whether
/// the line of the case names it.
#[derive(Debug)]
pub struct End {
    pub error: Error,
    pub named: bool,
}

/// What `Purba::zygote` gives for a module.
pub enum Started {
    /// The zygote, and the error of each thread that the load took.
    Loaded(Zygote, Vec<Error>),
    /// What stopped the module before its tests.
    Stopped(Stop),
    /// An interrupt is kept, so nothing loads.
    Unreached,
}

/// What `Zygote::run` gives for one test.
#[derive(Debug)]
pub enum Gave {
    /// The test ran.
    Ran(Ran),
    /// An interrupt is kept, so the test does not run.
    Unreached,
    /// The module's process ended at an earlier test, so the test does not run.
    Ended(End),
}

/// A test that ran: its outcome, or the error in its place, what its
/// suppressions gave, and the error of each thread, in the order that the
/// threads raised them.
#[derive(Debug)]
pub struct Ran {
    pub result: Result<Outcome, Error>,
    pub suppressed: Suppressed,
    pub threads: Vec<Thread>,
}

/// The error of a thread's exception, beside the step that it charges.
#[derive(Debug)]
pub enum Thread {
    /// The test that the test's process ran.
    Test(Error),
    /// The load of the module.
    Load(Error),
}

/// What a child of the zygote sent, up to its end: the facts of a test, the
/// verdict of a replay, the zygote's refusal to fork it, each error that it
/// sent or that a frame held, and whether the first interrupt landed in its
/// step. Its end is the wait status of the child, or the wait status of the
/// module's process, which ended first.
struct Sent {
    facts: Option<Facts>,
    verdict: Option<Verdict>,
    refused: Option<String>,
    errors: Vec<Error>,
    landed: bool,
    end: Result<libc::c_int, libc::c_int>,
}

/// What the load of a zygote gave: its result, the error of each thread that
/// the load left, and whether a thread is alive after it.
pub struct Load {
    pub result: Result<(), Error>,
    pub threads: Vec<Error>,
    pub threaded: bool,
}

/// What stopped a module before its tests: the error, and the error of each
/// thread that its load took.
#[derive(Debug)]
pub struct Stop {
    pub error: Error,
    pub threads: Vec<Error>,
}

/// The error of each thread that a test's process found, beside its step.
type Threads = Vec<(Step, Error)>;

/// Why no replay read an expectation alone: the words of the zygote's refusal
/// to fork it, or of the end of the module's process.
enum Refusal {
    Fork(String),
    End(String),
}

impl Refusal {
    /// The words of the refusal.
    fn words(self) -> String {
        let (Self::Fork(words) | Self::End(words)) = self;
        words
    }
}

impl Purba<'_> {
    /// Forks the zygote that forks a process for each test of `module`. Gives
    /// the zygote and the error of each thread that the load took, or what
    /// stopped the module, or `Unreached` when purba's signal handler kept an
    /// interrupt before the fork.
    ///
    /// The zygote loads the module. When a Python thread is alive after the
    /// load, that zygote ends. A zygote that loads nothing then forks the
    /// processes. Each of them loads the module again before its test.
    ///
    /// The module stops when a zygote cannot fork, when the load fails, or
    /// when a zygote ends before it is ready.
    #[must_use]
    pub fn zygote(&self, module: &Module) -> Started {
        if interrupt::kept(self.py) {
            return Started::Unreached;
        }
        let mut zygote = match self.forked(module, true) {
            Ok(zygote) => zygote,
            Err(error) => {
                return Started::Stopped(Stop {
                    error,
                    threads: Vec::new(),
                });
            }
        };
        let Load {
            result,
            threads,
            threaded,
        } = zygote.loaded();
        if let Err(error) = result {
            return Started::Stopped(Stop { error, threads });
        }
        if !threaded {
            return Started::Loaded(zygote, threads);
        }
        drop(zygote);
        let mut zygote = match self.forked(module, false) {
            Ok(zygote) => zygote,
            Err(error) => return Started::Stopped(Stop { error, threads }),
        };
        match zygote.loaded().result {
            Ok(()) => Started::Loaded(zygote, threads),
            Err(error) => Started::Stopped(Stop { error, threads }),
        }
    }

    /// Forks a zygote for `module`. With `load`, the zygote loads the module
    /// once, and each test's process runs its test. Without it, each test's
    /// process loads the module, then runs its test.
    ///
    /// # Errors
    ///
    /// A project error when the operating system refused a pipe or the fork.
    fn forked(&self, module: &Module, load: bool) -> Result<Zygote, Error> {
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
                        Worker::new(self.py, self.hook),
                        module,
                        load,
                        File::from(tests_in),
                        File::from(frames_out),
                    );
                }))
                .err();
                if let (Ok(mut spare), Some(payload)) = (spare, payload) {
                    let error = crate::caught(payload.as_ref());
                    drop(Message::Error(error).write(&mut spare));
                }
                exit(125)
            }
            Forked::Parent(pid) => {
                interrupt::follow_zygote(pid);
                Ok(Zygote {
                    pid,
                    tests: Some(File::from(tests_out)),
                    frames: File::from(frames_in),
                    reaped: None,
                    named: false,
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
    pub(crate) fn loaded(&mut self) -> Load {
        let mut load = Load {
            result: Ok(()),
            threads: Vec::new(),
            threaded: false,
        };
        let window = Window::open();
        let mut threads = Vec::new();
        let end = loop {
            match self.own(Step::Load, &mut threads) {
                Ok(Message::Ready) => break None,
                Ok(Message::Threaded) => {
                    load.threaded = true;
                    break None;
                }
                Ok(Message::Loaded) => load.result = Ok(()),
                Ok(Message::Error(error)) => load.result = Err(error),
                Ok(_) => {}
                Err(end) => break Some(end),
            }
        };
        load.threads = threads.into_iter().map(|(_, error)| error).collect();
        if let Some((unreadable, status)) = end {
            if let Some(error) = unreadable {
                load.result = Err(error);
            } else if load.result.is_ok() && !window.arrived() {
                load.result = Err(Error::project(lost(status, "its load ended")));
            }
        }
        load
    }

    /// Reads the next message that is the caller's own. On the way, it takes
    /// each message that every caller takes alike. They are an interrupt that
    /// a child kept, the pid of a test's process that the zygote forked, and
    /// the error of a thread. The error of a thread goes into `threads`, with
    /// its step. A test's thread gets `running`, the step that runs: the load
    /// as the zygote loads, and the test in a case. A load's thread gets the
    /// load.
    ///
    /// When no message comes, it calls `abandon`. It then gives the error of
    /// the frame and the wait status. The error is `None` when the pipe ended
    /// or failed.
    fn own(
        &mut self,
        running: Step,
        threads: &mut Threads,
    ) -> Result<Message, (Option<Error>, libc::c_int)> {
        loop {
            match self.next() {
                Ok(Message::Kept(signum)) => interrupt::adopt(signum),
                Ok(Message::Forked(pid)) => interrupt::follow_test(pid),
                Ok(Message::TestThread(text)) => {
                    threads.push((running, Error::project(text)));
                }
                Ok(Message::LoadThread(text)) => {
                    threads.push((Step::Load, Error::project(text)));
                }
                Ok(message) => return Ok(message),
                Err(unreadable) => {
                    interrupt::follow_test(0);
                    return Err((unreadable, self.abandon()));
                }
            }
        }
    }

    /// Reads the next message from the zygote, a test's process or a replay.
    /// Gives an error when no message comes: the internal error of a frame that
    /// holds no message, or `None` when the pipe ended or failed. Either way, the
    /// reader then calls `abandon`, since no frame after it can be trusted.
    fn next(&mut self) -> Result<Message, Option<Error>> {
        match Message::read(&mut self.frames) {
            Ok(Some(read)) => read.map_err(Some),
            Ok(None) | Err(_) => Err(None),
        }
    }

    /// Closes the pipe of test names, reads each frame left until the pipe of
    /// frames ends, and waits for the zygote to end. Gives its wait status. A
    /// process that still writes frames then never waits on a full pipe.
    fn abandon(&mut self) -> libc::c_int {
        self.tests = None;
        drop(io::copy(&mut self.frames, &mut io::sink()));
        self.wait()
    }

    /// Asks the zygote to fork a process for the test `function`, which
    /// `suppressions` reach. Reads the facts of its case, and the error of each
    /// thread that it found, beside the step that left the thread. When
    /// the facts name later expectations, it asks for a replay of each, and
    /// then decides the outcome and the warnings of the case in one order,
    /// with the readings alone that a test's process that forked its readers
    /// would have.
    ///
    /// The process can give `Interrupted` or an error in place of the outcome.
    /// A case with no outcome gives a project error when the module's process
    /// ended, else `Interrupted` when the first interrupt arrived as it ran,
    /// else `Raised`, which names the signal or the exit code of the test's
    /// process.
    ///
    /// When the module's process ends before each replay sent its reading, the
    /// case gets its end as a refusal, as when the zygote cannot fork a
    /// replay.
    ///
    /// It runs no test when purba's signal handler kept an interrupt, and
    /// gives `Unreached`. Else, once the module's process ended, it runs no
    /// test, and gives that end.
    pub fn run(&mut self, function: &str, suppressions: &[(&str, &str)]) -> Gave {
        if crate::interrupted() {
            return Gave::Unreached;
        }
        if let Some(status) = self.reaped {
            return Gave::Ended(End {
                error: Error::project(gone(status)),
                named: self.named,
            });
        }
        let mut threads = Vec::new();
        let test = Message::Test {
            function: function.to_owned(),
        };
        let (result, suppressed, named) = match self.ask(&test) {
            Ok(()) => self.case(function, suppressions, &mut threads),
            Err(gone) => (Err(Error::project(gone)), Suppressed::default(), true),
        };
        self.named = named;
        Gave::Ran(Ran {
            result,
            suppressed,
            threads: threads
                .into_iter()
                .map(|(step, error)| match step {
                    Step::Test => Thread::Test(error),
                    Step::Load => Thread::Load(error),
                })
                .collect(),
        })
    }

    /// Reads the case of the test `function`, which the zygote forked, and
    /// asks for its replays. Gives what `order` decides from all that the case
    /// met: the outcome or what takes its place, what the suppressions of the
    /// case gave, and whether the end of the module's process took the place
    /// of the outcome.
    fn case(
        &mut self,
        function: &str,
        suppressions: &[(&str, &str)],
        threads: &mut Threads,
    ) -> (Result<Outcome, Error>, Suppressed, bool) {
        let window = Window::open();
        let Sent {
            facts,
            errors,
            landed,
            end,
            ..
        } = self.sent(threads);
        let mut case = Case {
            facts,
            errors,
            landed,
            arrived: false,
            end,
            ended_a_replay: false,
        };
        self.replays(function, &mut case, threads);
        case.arrived = window.arrived();
        let Decided {
            own,
            suppressed,
            ended,
        } = crate::order::order(case, suppressions);
        (own.map(outcome), suppressed, ended)
    }

    /// Writes `message` to the zygote. When the zygote no longer reads, it
    /// waits for it, and gives the words of its end.
    fn ask(&mut self, message: &Message) -> Result<(), String> {
        let asked = self.tests.as_mut().map_or_else(
            || Err(ErrorKind::BrokenPipe.into()),
            |tests| message.write(tests),
        );
        asked.map_err(|_| gone(self.wait()))
    }

    /// Reads what a child of the zygote sends, until it ends. The test owns
    /// each thread that the child's test left.
    fn sent(&mut self, threads: &mut Threads) -> Sent {
        let mut sent = Sent {
            facts: None,
            verdict: None,
            refused: None,
            errors: Vec::new(),
            landed: false,
            end: Ok(0),
        };
        sent.end = loop {
            match self.own(Step::Test, threads) {
                Ok(Message::Ended(status)) => {
                    interrupt::follow_test(0);
                    break Ok(status);
                }
                Ok(Message::Facts(found)) => sent.facts = Some(*found),
                Ok(Message::Verdict(verdict)) => sent.verdict = Some(verdict),
                Ok(Message::Refused(words)) => sent.refused = Some(words),
                Ok(Message::Interrupted) => sent.landed = true,
                Ok(Message::Error(error)) => sent.errors.push(error),
                Ok(_) => {}
                Err((unreadable, status)) => {
                    sent.errors.extend(unreadable);
                    break Err(status);
                }
            }
        };
        sent
    }

    /// Asks the zygote for a replay of `function` for each of `case.later()`,
    /// in order, and adds each verdict to the readings alone of the facts. When
    /// the zygote cannot fork a replay, or the module's process ends before a
    /// replay sent its reading, it puts the refusal in the facts and asks for
    /// no later replay, as a test's process does when it cannot fork a reader.
    /// When a replay sends an error or a frame holds no message, it adds the
    /// error to the case. When an interrupt lands during the replays, the case
    /// has `landed`. Either of these two keeps no verdict, and asks for no
    /// later replay.
    fn replays(&mut self, function: &str, case: &mut Case, threads: &mut Threads) {
        let later = case.later().to_vec();
        let window = Window::open();
        let mut alone = Vec::new();
        let mut refusal = None;
        for index in later {
            let replayed = match self.replay(function, index, threads) {
                Ok(replayed) => replayed,
                Err(error) => {
                    case.errors.push(error);
                    return;
                }
            };
            if window.arrived() {
                case.landed = true;
                return;
            }
            match replayed {
                Ok(verdict) => alone.push((index, verdict)),
                Err(refused) => {
                    refusal = Some(refused);
                    break;
                }
            }
        }
        if let Some(facts) = case.facts.as_mut() {
            facts.alone.extend(alone);
            if let Some(refused) = refusal {
                case.ended_a_replay = matches!(refused, Refusal::End(_));
                facts.refused = Some(refused.words());
            }
        }
    }

    /// Asks the zygote for one replay of `function` that reads the expectation
    /// at `index` alone, and gives its verdict, or the words of a refusal: the
    /// zygote could not fork it, or the module's process ended before its
    /// verdict came. A replay that ends with no verdict gives `Raises`.
    ///
    /// # Errors
    ///
    /// The error that the replay sent, or the error of a frame that holds no
    /// message.
    fn replay(
        &mut self,
        function: &str,
        index: usize,
        threads: &mut Threads,
    ) -> Result<Result<Verdict, Refusal>, Error> {
        let replay = Message::Replay {
            function: function.to_owned(),
            index,
        };
        if let Err(gone) = self.ask(&replay) {
            return Ok(Err(Refusal::End(gone)));
        }
        let Sent {
            verdict,
            refused,
            mut errors,
            end,
            ..
        } = self.sent(threads);
        if let Some(error) = errors.pop() {
            return Err(error);
        }
        Ok(match (refused, verdict, end) {
            (Some(refused), _, _) => Err(Refusal::Fork(refused)),
            (None, Some(verdict), _) => Ok(verdict),
            (None, None, Ok(_)) => Ok(Verdict::Raises),
            (None, None, Err(status)) => Err(Refusal::End(gone(status))),
        })
    }

    /// Closes the pipe of test names, and waits for the zygote to end. Gives
    /// its wait status. Once it reaped the zygote, it gives that status again.
    fn wait(&mut self) -> libc::c_int {
        self.tests = None;
        if let Some(status) = self.reaped {
            return status;
        }
        let status = wait(self.pid);
        self.reaped = Some(status);
        interrupt::follow_zygote(0);
        status
    }
}

impl Drop for Zygote {
    /// Tells the zygote that no test is left, and waits for it to end.
    fn drop(&mut self) {
        self.wait();
    }
}

/// The words of the module's process that ended with the wait status
/// `status`: `the module's process died by SIGKILL`.
pub fn gone(status: libc::c_int) -> String {
    format!("the module's process {}", ended(status))
}

/// What a process that ended before `what` gives, from its wait status: `its
/// process died by SIGABRT`, or `its process exited 3 before <what>`.
pub fn lost(status: libc::c_int, what: &str) -> String {
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
fn wait(pid: libc::pid_t) -> libc::c_int {
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
fn flush(py: Python<'_>) {
    if let Ok(sys) = py.import("sys") {
        for name in ["stdout", "stderr"] {
            if let Ok(stream) = sys.getattr(name) {
                drop(stream.call_method0("flush"));
            }
        }
    }
}

/// Writes `message`, and ends the process when the pipe refuses it: purba no
/// longer reads.
fn send(frames: &mut File, message: &Message) {
    if message.write(frames).is_err() {
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

/// The zygote's life, in the child of the fork, with `worker`.
fn live_as_zygote(
    mut worker: Worker<'_>,
    module: &Module,
    load: bool,
    mut tests: File,
    mut frames: File,
) -> ! {
    let py = worker.py;
    call_atexit(py, "_clear");
    if load {
        let result = catch("execution", || worker.load(module));
        let failed = result.is_err();
        send(
            &mut frames,
            &result.map_or_else(Message::Error, |()| Message::Loaded),
        );
        send_threads(&worker, &mut frames);
        send_kept(&mut frames);
        if failed {
            send(&mut frames, &Message::Ready);
            end(py);
        }
        if threaded(py) {
            send(&mut frames, &Message::Threaded);
            end(py);
        }
    }
    send(&mut frames, &Message::Ready);
    let children = Children {
        module,
        loads: !load,
    };
    while let Ok(Some(Ok(message))) = Message::read(&mut tests) {
        let (function, way) = match message {
            Message::Test { function } => (function, Way::Test),
            Message::Replay { function, index } => (function, Way::Replay(index)),
            _ => break,
        };
        flush(py);
        match fork(py) {
            Ok(Forked::Child) => {
                drop(tests);
                live_as_child(py, &mut worker, &children, &function, way, frames)
            }
            Ok(Forked::Parent(pid)) => {
                let status = wait(pid);
                send(&mut frames, &Message::Ended(status));
            }
            Err(error) => {
                let refusal = match way {
                    Way::Replay(_) => Message::Refused(format!(
                        "purba cannot fork to read an expectation alone: {error}"
                    )),
                    Way::Test => {
                        Message::Error(Error::project(format!("purba cannot fork: {error}")))
                    }
                };
                send(&mut frames, &refusal);
                send(&mut frames, &Message::Ended(0));
            }
        }
    }
    end(py)
}

/// What the zygote asks of a child: to run a test, or to replay it and read
/// the expectation at an index alone.
#[derive(Clone, Copy)]
enum Way {
    Test,
    Replay(usize),
}

/// What each child of a zygote starts from: the module, and whether the child
/// loads it. A child loads it when the zygote loaded nothing, since the first
/// zygote's load left a thread alive.
struct Children<'m> {
    module: &'m Module,
    loads: bool,
}

/// Whether a Python thread other than the main one is alive, as the hook that
/// purba took sees.
fn threaded(py: Python<'_>) -> bool {
    let Ok(main) = py
        .import("threading")
        .and_then(|threading| threading.getattr("main_thread")?.call0())
    else {
        return false;
    };
    let Some(hook) = thread::taken(py) else {
        return false;
    };
    hook.get()
        .alive(py)
        .iter()
        .any(|thread| !thread.bind(py).is(&main))
}

/// The life of a child of the zygote: a test's process, or a replay, as `way`
/// says. Each child sends its pid, loads the module when `children` says so,
/// and sends what it found and the first interrupt that it kept. A process
/// that the test forks and that returns from the test ends with no frame.
///
/// A test's process runs the test `function`, and sends each thing that its
/// step met: its facts, its own error, a panic that the hook noted, and
/// whether the first interrupt landed in the step. It decides nothing from
/// them. It then joins its threads, runs its `atexit` functions, and sends the
/// error of each thread.
///
/// A replay runs the test again, and reads only the expectation at its index.
/// It ends at that expectation, and `REPLAYS` sends its verdict. It writes
/// nothing to stdout or stderr, since the case wrote what the test writes
/// once. It sends no error of a thread, since the case sent them once.
fn live_as_child(
    py: Python<'_>,
    worker: &mut Worker<'_>,
    children: &Children<'_>,
    function: &str,
    way: Way,
    mut frames: File,
) -> ! {
    // SAFETY: `getpid` only gives the pid of the process.
    let own = unsafe { libc::getpid() };
    send(&mut frames, &Message::Forked(own));
    if let Way::Replay(_) = way {
        silence();
        REPLAYS.hold(own, &frames);
    }
    call_atexit(py, "_clear");
    let mut facts = None;
    let mut met = Met {
        panic: None,
        interrupted: false,
    };
    let result = catch("execution", || {
        if children.loads {
            worker.load(children.module)?;
            // The load in the first zygote already handed on the errors of these
            // threads.
            drop(worker.threads());
            if interrupt::kept_now() {
                met.interrupted = true;
                return Ok(());
            }
        }
        match way {
            Way::Test => {
                let (gathered, own, step) = worker.gather(function);
                facts = gathered;
                met = step;
                own
            }
            Way::Replay(index) => worker.replay(function, index),
        }
    });
    // SAFETY: `getpid` only gives the pid of the process.
    if unsafe { libc::getpid() } != own {
        exit(0);
    }
    if let Some(facts) = facts {
        send(&mut frames, &Message::Facts(Box::new(facts)));
    }
    match (way, result) {
        (Way::Replay(_), Ok(())) if !met.interrupted => REPLAYS.end(Verdict::Raises),
        (_, Ok(())) => {}
        (_, Err(error)) => send(&mut frames, &Message::Error(error)),
    }
    if let Some(noted) = met.panic {
        let panicked = crate::internal(noted.caught_by("execution"));
        send(&mut frames, &Message::Error(panicked));
    }
    if matches!(way, Way::Test) && met.interrupted {
        send(&mut frames, &Message::Interrupted);
    }
    if matches!(way, Way::Test) {
        interrupt::calling(py, || join(py, worker));
        call_atexit(py, "_run_exitfuncs");
        send_threads(worker, &mut frames);
    }
    send_kept(&mut frames);
    flush(py);
    exit(0)
}

/// Points stdout and stderr at `/dev/null`.
fn silence() {
    if let Ok(null) = File::options().write(true).open("/dev/null") {
        use std::os::fd::AsRawFd;
        for target in [libc::STDOUT_FILENO, libc::STDERR_FILENO] {
            // SAFETY: both descriptors are open, and `dup2` only points
            // `target` at the file of `null`.
            unsafe { libc::dup2(null.as_raw_fd(), target) };
        }
    }
}

/// Joins each thread that is alive and is not a daemon, until an interrupt
/// raises in a join.
fn join(py: Python<'_>, worker: &Worker<'_>) {
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
        send(frames, &Message::Kept(signum));
    }
}

/// Writes the error of each thread that the worker took.
fn send_threads(worker: &Worker<'_>, frames: &mut File) {
    for (step, error) in worker.threads() {
        let message = match step {
            Step::Load => Message::LoadThread(error.to_string()),
            Step::Test => Message::TestThread(error.to_string()),
        };
        send(frames, &message);
    }
}

/// The readers of a case: a process that the case's process forks for each
/// expectation after the first, before any reading, to read that expectation
/// alone. A reader ends with its verdict as its exit code. The steps of a case
/// also ask here what it replaced, and the first expectation stops the watch
/// of its paths.
pub struct Forks;

pub static READERS: Forks = Forks;

const HOLDS: libc::c_int = 0;
const FAILS: libc::c_int = 1;
const RAISES: libc::c_int = 2;
const REFUSED: libc::c_int = 3;

impl Readers for Forks {
    fn threaded(&self, py: Python<'_>) -> bool {
        threaded(py)
    }

    fn fork(&self, py: Python<'_>) -> io::Result<Option<i32>> {
        flush(py);
        Ok(match fork(py)? {
            Forked::Child => None,
            Forked::Parent(pid) => Some(pid),
        })
    }

    fn wait(&self, pid: i32) -> Verdict {
        verdict(wait(pid))
    }

    fn end(&self, verdict: Verdict) -> ! {
        exit(match verdict {
            Verdict::Holds => HOLDS,
            Verdict::Fails => FAILS,
            Verdict::Raises => RAISES,
            Verdict::Refused => REFUSED,
        })
    }

    fn replaced(&self, py: Python<'_>) -> Option<Replaced> {
        crate::replaced(py)
    }

    fn expecting(&self, _py: Python<'_>) {
        crate::paths::stop();
    }
}

/// The readers of a replay. A replay reads one expectation, and forks no
/// reader. It ends by sending its verdict and the first interrupt that it
/// kept to purba, as frames, on the pipe that it holds. A process that the
/// test forks in the replay ends as a reader does.
pub struct Replays {
    frames: Mutex<Option<(libc::pid_t, File)>>,
}

pub static REPLAYS: Replays = Replays {
    frames: Mutex::new(None),
};

impl Replays {
    /// Holds a copy of the pipe of frames of the replay `pid`. Without one,
    /// the replay ends as a reader does.
    fn hold(&self, pid: libc::pid_t, frames: &File) {
        if let Ok(copy) = frames.try_clone() {
            *self.frames.lock().unwrap_or_else(PoisonError::into_inner) = Some((pid, copy));
        }
    }
}

impl Readers for Replays {
    fn threaded(&self, py: Python<'_>) -> bool {
        READERS.threaded(py)
    }

    fn fork(&self, py: Python<'_>) -> io::Result<Option<i32>> {
        READERS.fork(py)
    }

    fn wait(&self, pid: i32) -> Verdict {
        READERS.wait(pid)
    }

    fn end(&self, verdict: Verdict) -> ! {
        let held = self
            .frames
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .take();
        // SAFETY: `getpid` only gives the pid of the process.
        let own = unsafe { libc::getpid() };
        if let Some((pid, mut frames)) = held
            && pid == own
        {
            send(&mut frames, &Message::Verdict(verdict));
            send_kept(&mut frames);
            exit(0)
        }
        READERS.end(verdict)
    }

    fn replaced(&self, py: Python<'_>) -> Option<Replaced> {
        READERS.replaced(py)
    }

    fn expecting(&self, py: Python<'_>) {
        READERS.expecting(py);
    }
}

/// The verdict of a reader that ended with the wait status
/// `status`. One that a signal ends gives `Raises`, as does any exit code but
/// the four of a verdict.
const fn verdict(status: libc::c_int) -> Verdict {
    if !libc::WIFEXITED(status) {
        return Verdict::Raises;
    }
    match libc::WEXITSTATUS(status) {
        HOLDS => Verdict::Holds,
        FAILS => Verdict::Fails,
        REFUSED => Verdict::Refused,
        _ => Verdict::Raises,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use std::io::Write;
    use std::sync::mpsc;
    use std::time::Duration;

    use crate::Kind;
    use crate::frame::tests::holding_no_message;

    /// What a run of the case `test_one` gave, and how to end it.
    struct Ran {
        /// The outcome, or the `Debug` text of the error.
        outcome: Result<Kind, String>,
        /// Hears when the sender of the frames ends, and holds the pipe of
        /// frames that it hands over.
        sent: mpsc::Receiver<Option<File>>,
        /// Lets the `Zygote` drop.
        release: mpsc::Sender<()>,
    }

    /// The outcome of a test that `Zygote::run` ran, or the `Debug` text of
    /// its error.
    fn outcome_of(gave: Gave) -> Result<Kind, String> {
        match gave {
            Gave::Ran(ran) => ran
                .result
                .map(|outcome| outcome.kind().clone())
                .map_err(|error| format!("{error:?}")),
            other => panic!("the test runs: {other:?}"),
        }
    }

    /// Runs the case `test_one` on `frames`, as `Zygote::run` reads them.
    ///
    /// A thread stands in for the zygote and the test's process. It writes
    /// each frame, then holds the pipe of frames open until the pipe of test
    /// names closes, as a zygote does. The `Zygote` lives on after the run, as
    /// dispatch keeps it for the next test, until `release` drops. A run that
    /// waits for a frame that does not come fails the case after 10 seconds.
    fn run_on(frames: Vec<Vec<u8>>) -> Ran {
        run_ending(frames, Ends::Never)
    }

    /// Runs the case `test_one` on `frames`, as `run_on` does, from a stand-in
    /// that ends as `ends` says.
    fn run_ending(frames: Vec<Vec<u8>>, ends: Ends) -> Ran {
        let (mut zygote, sent) = zygote_on(frames, ends);
        let (done, outcome) = mpsc::channel();
        let (release, released) = mpsc::channel::<()>();
        std::thread::spawn(move || {
            drop(done.send(outcome_of(zygote.run("test_one", &[]))));
            released.recv().ok();
        });
        let outcome = outcome
            .recv_timeout(Duration::from_secs(10))
            .expect("the run returns without a frame that does not come");
        Ran {
            outcome,
            sent,
            release,
        }
    }

    /// When the stand-in for the zygote and the test's process ends, as its
    /// pipes show it.
    enum Ends {
        /// Never: it holds the pipe of frames open until the pipe of test
        /// names closes, as a zygote does.
        Never,
        /// Before the ask for a replay: it reads the ask for the case, closes
        /// the pipe of test names, writes each frame, and hands the pipe of
        /// frames to the receiver, which holds it open, as a process that the
        /// test forked can. The ask for a replay then fails, and a read for a
        /// frame waits.
        BeforeTheReplay,
        /// After its frames: it closes the pipe of frames after the last
        /// frame, and reads each ask until the pipe of test names closes. A
        /// read for a frame of the replay then finds the end.
        AfterItsFrames,
        /// Before the ask for the case: it closes both pipes, and writes no
        /// frame, as a zygote that died between two tests does. Once the
        /// receiver hears it end, the ask for the case fails.
        BeforeTheAsk,
    }

    /// A `Zygote` of the module `test_one.py` that reads `frames`, and a
    /// receiver that hears when their sender ends, and holds the pipe of
    /// frames that it hands over.
    ///
    /// A thread stands in for the zygote and the test's process. It writes
    /// each frame, and ends as `ends` says.
    fn zygote_on(frames: Vec<Vec<u8>>, ends: Ends) -> (Zygote, mpsc::Receiver<Option<File>>) {
        let (frames_in, frames_out) = pipe().expect("a pipe opens");
        let (tests_in, tests_out) = pipe().expect("a pipe opens");
        let (sender_done, sent) = mpsc::channel();
        std::thread::spawn(move || {
            let mut asks = File::from(tests_in);
            let mut sending = File::from(frames_out);
            let mut write_each = || {
                for frame in &frames {
                    if sending.write_all(frame).is_err() {
                        break;
                    }
                }
            };
            let held = match ends {
                Ends::Never => {
                    write_each();
                    drop(io::copy(&mut asks, &mut io::sink()));
                    drop(sending);
                    None
                }
                Ends::BeforeTheReplay => {
                    drop(Message::read(&mut asks));
                    drop(asks);
                    write_each();
                    Some(sending)
                }
                Ends::AfterItsFrames => {
                    write_each();
                    drop(sending);
                    drop(io::copy(&mut asks, &mut io::sink()));
                    None
                }
                Ends::BeforeTheAsk => {
                    drop(asks);
                    drop(sending);
                    None
                }
            };
            sender_done.send(held).ok();
        });
        let zygote = Zygote {
            // No child has this pid, so a wait for it returns at once.
            pid: libc::pid_t::MAX,
            tests: Some(File::from(tests_out)),
            frames: File::from(frames_in),
            reaped: None,
            named: false,
        };
        (zygote, sent)
    }

    /// Replays the expectation at index 1 of `test_one` on `frames`. It gives
    /// the `Debug` text of the verdict or the error, beside the `Debug` text of
    /// each thread's step and error. A replay that waits for a frame that does
    /// not come fails the case after 10 seconds.
    fn replay_on(frames: Vec<Vec<u8>>) -> (Result<String, String>, Vec<String>) {
        let (mut zygote, _sent) = zygote_on(frames, Ends::Never);
        let (done, replayed) = mpsc::channel();
        std::thread::spawn(move || {
            let mut threads = Vec::new();
            let verdict = zygote
                .replay("test_one", 1, &mut threads)
                .map(|verdict| format!("{:?}", verdict.map_err(Refusal::words)))
                .map_err(|error| format!("{error:?}"));
            let threads = threads
                .iter()
                .map(|(step, error)| format!("{step:?} {error:?}"))
                .collect::<Vec<_>>();
            drop(done.send((verdict, threads)));
        });
        replayed
            .recv_timeout(Duration::from_secs(10))
            .expect("the replay returns without a frame that does not come")
    }

    #[test]
    fn a_replay_charges_each_thread_to_its_owner_and_reads_its_verdict() {
        let frames = vec![
            frame_of(&Message::Forked(0)),
            frame_of(&Message::TestThread("of the test".to_owned())),
            frame_of(&Message::LoadThread("of the load".to_owned())),
            frame_of(&Message::Verdict(Verdict::Holds)),
            frame_of(&Message::Ended(0)),
        ];
        let (verdict, threads) = replay_on(frames);
        assert_eq!(
            verdict,
            Ok(format!("{:?}", Ok::<_, String>(Verdict::Holds))),
            "a replay that sends that its expectation holds reads that it holds"
        );
        assert_eq!(
            threads,
            vec![
                format!(
                    "{:?} {:?}",
                    Step::Test,
                    Error::project("of the test".to_owned())
                ),
                format!(
                    "{:?} {:?}",
                    Step::Load,
                    Error::project("of the load".to_owned())
                ),
            ],
            "a replay charges a test's thread to the test, and a load's thread to the module, \
             as a case does"
        );
    }

    #[test]
    fn a_frame_that_holds_no_message_ends_the_replay_with_its_error() {
        let (no_message, error) = holding_no_message();
        let (verdict, threads) = replay_on(vec![no_message]);
        assert_eq!(
            verdict,
            Err(error),
            "the internal error takes the place of the verdict, as it does for a case"
        );
        assert!(threads.is_empty(), "no thread is charged: {threads:?}");
    }

    /// The bytes of the frame of `message`.
    fn frame_of(message: &Message) -> Vec<u8> {
        let mut pipe = Vec::new();
        message.write(&mut pipe).expect("a Vec takes the write");
        pipe
    }

    /// The facts of a case whose one expectation held, so that it passed.
    fn passed() -> Message {
        Message::Facts(Box::new(Facts {
            in_order: vec![(1, true)],
            stated: true,
            ..Facts::default()
        }))
    }

    #[test]
    fn a_case_skips_a_message_that_comes_out_of_its_turn() {
        let frames = vec![
            frame_of(&passed()),
            frame_of(&Message::Ready),
            frame_of(&Message::Loaded),
            frame_of(&Message::Ended(0)),
        ];
        assert_eq!(
            run_on(frames).outcome,
            Ok(Kind::Passed),
            "a message of the load, after the outcome of a case, changes nothing"
        );
    }

    #[test]
    fn a_frame_that_holds_no_message_ends_the_case_with_its_error() {
        let (no_message, error) = holding_no_message();
        let frames = vec![frame_of(&passed()), no_message];
        assert_eq!(
            run_on(frames).outcome,
            Err(error),
            "the internal error takes the place of the outcome, and the run waits for no \
             frame that its process does not send"
        );
    }

    #[test]
    fn a_frame_that_holds_no_message_reads_each_frame_after_it() {
        // More than a pipe holds: a sender that nobody reads waits on it.
        let after = frame_of(&Message::TestThread("x".repeat(1024)));
        let (no_message, error) = holding_no_message();
        let frames = std::iter::once(no_message)
            .chain(std::iter::repeat_n(after, 256))
            .collect();
        let ran = run_on(frames);
        assert_eq!(
            ran.outcome,
            Err(error),
            "the internal error takes the place of the outcome"
        );
        assert!(
            ran.sent.recv_timeout(Duration::from_secs(10)).is_ok(),
            "the run reads each frame after it, so its sender never waits on a full pipe"
        );
        drop(ran.release);
    }

    /// The words of a fork that the operating system refused.
    const REFUSAL: &str = "purba cannot fork to read an expectation alone: EAGAIN";

    /// The frames of a test's process that sent `facts`, and ended.
    fn case(facts: Facts) -> [Vec<u8>; 2] {
        [
            frame_of(&Message::Facts(Box::new(facts))),
            frame_of(&Message::Ended(0)),
        ]
    }

    /// The frames that the zygote sends when it cannot fork a replay.
    fn refused_replay() -> [Vec<u8>; 2] {
        [
            frame_of(&Message::Refused(REFUSAL.to_owned())),
            frame_of(&Message::Ended(0)),
        ]
    }

    /// The frames of a replay that sends `verdict`, and ends.
    fn replayed(verdict: Verdict) -> [Vec<u8>; 3] {
        [
            frame_of(&Message::Forked(0)),
            frame_of(&Message::Verdict(verdict)),
            frame_of(&Message::Ended(0)),
        ]
    }

    /// The line that `order` gives for `facts`, when a test's process with no
    /// thread alive sent them, as `Ran::outcome` holds it.
    fn forked(facts: &Facts) -> Result<Kind, String> {
        crate::order::order(Case::of(facts.clone()), &[])
            .own
            .map_err(|error| format!("{error:?}"))
    }

    /// The facts of a case whose call raised after its first expectation held.
    fn raised() -> Facts {
        Facts {
            called: crate::order::Called::Raised {
                exception: "ValueError: boom".to_owned(),
                whence: None,
                assertion: false,
            },
            in_order: vec![(0, true)],
            stated: true,
            ..Facts::default()
        }
    }

    #[test]
    fn a_replay_that_the_zygote_cannot_fork_leaves_the_case_to_the_order() {
        let sent = Facts {
            later: vec![1, 2],
            ..raised()
        };
        let frames = case(sent).into_iter().chain(refused_replay()).collect();
        let ran = run_on(frames);
        let without_a_thread = Facts {
            refused: Some(REFUSAL.to_owned()),
            ..raised()
        };
        assert_eq!(
            ran.outcome,
            forked(&without_a_thread),
            "a case that raised gives raised, as it does when no thread is alive"
        );
        drop(ran.release);
    }

    #[test]
    fn a_replay_that_the_zygote_cannot_fork_gives_its_refusal_at_the_step_of_a_fork() {
        let held = Facts {
            in_order: vec![(0, true)],
            stated: true,
            ..Facts::default()
        };
        let sent = Facts {
            later: vec![1],
            ..held.clone()
        };
        let frames = case(sent).into_iter().chain(refused_replay()).collect();
        let ran = run_on(frames);
        let without_a_thread = Facts {
            refused: Some(REFUSAL.to_owned()),
            ..held
        };
        assert_eq!(
            ran.outcome,
            forked(&without_a_thread),
            "a case that would pass gives the refusal as a project error, as it does when no \
             thread is alive"
        );
        drop(ran.release);
    }

    #[test]
    fn a_replay_that_the_zygote_cannot_fork_keeps_each_verdict_before_it() {
        let held = Facts {
            in_order: vec![(0, true), (1, true), (2, true)],
            texts: vec![(1, "assert x == 1".to_owned())],
            stated: true,
            ..Facts::default()
        };
        let sent = Facts {
            later: vec![1, 2],
            ..held.clone()
        };
        let frames = case(sent)
            .into_iter()
            .chain(replayed(Verdict::Refused))
            .chain(refused_replay())
            .collect();
        let ran = run_on(frames);
        let without_a_thread = Facts {
            alone: vec![(1, Verdict::Refused)],
            refused: Some(REFUSAL.to_owned()),
            ..held
        };
        assert_eq!(
            ran.outcome,
            forked(&without_a_thread),
            "an assert that the replay before the refusal reached beats the refusal, as a \
             reader's does when no thread is alive"
        );
        drop(ran.release);
    }

    #[test]
    fn a_replay_that_the_zygote_cannot_fork_leaves_no_frame_for_the_next_case() {
        let sent = Facts {
            later: vec![1],
            ..raised()
        };
        let frames = case(sent)
            .into_iter()
            .chain(refused_replay())
            .chain([frame_of(&passed()), frame_of(&Message::Ended(0))])
            .collect();
        let (mut zygote, _sent) = zygote_on(frames, Ends::Never);
        let (done, outcomes) = mpsc::channel();
        std::thread::spawn(move || {
            for _ in 0..2 {
                drop(done.send(outcome_of(zygote.run("test_one", &[]))));
            }
        });
        let next = || {
            outcomes
                .recv_timeout(Duration::from_secs(10))
                .expect("each run returns without a frame that does not come")
        };
        let first = next();
        assert_eq!(
            next(),
            Ok(Kind::Passed),
            "the case after a refused replay reads its own frames; the first case gave {first:?}"
        );
    }

    #[test]
    fn a_replay_whose_frame_holds_no_message_takes_the_place_of_the_outcome() {
        let sent = Facts {
            later: vec![1],
            ..raised()
        };
        let (no_message, error) = holding_no_message();
        let frames = case(sent)
            .into_iter()
            .chain([frame_of(&Message::Forked(0)), no_message])
            .collect();
        let ran = run_on(frames);
        assert_eq!(
            ran.outcome,
            Err(error),
            "the internal error of the replay takes the place of the line that the order gives"
        );
        drop(ran.release);
    }

    /// The words of the stand-in's process that ended. No child has its pid,
    /// so its wait status reads 0.
    const ENDED: &str = "the module's process exited 0";

    /// The facts of a case whose two expectations held in order.
    fn held() -> Facts {
        Facts {
            in_order: vec![(0, true), (1, true)],
            stated: true,
            ..Facts::default()
        }
    }

    #[test]
    fn a_replay_whose_module_process_ends_before_its_ask_leaves_a_case_that_raised_to_the_order() {
        let sent = Facts {
            later: vec![1],
            ..raised()
        };
        let ran = run_ending(case(sent).into(), Ends::BeforeTheReplay);
        let refused = Facts {
            refused: Some(ENDED.to_owned()),
            ..raised()
        };
        assert_eq!(
            ran.outcome,
            forked(&refused),
            "a case that raised gives raised, since the end is a refusal, as a fork that \
             failed is"
        );
        drop(ran.release);
    }

    #[test]
    fn a_replay_whose_module_process_ends_before_its_ask_gives_the_refusal_of_its_end() {
        let sent = Facts {
            later: vec![1],
            ..held()
        };
        let ran = run_ending(case(sent).into(), Ends::BeforeTheReplay);
        let refused = Facts {
            refused: Some(ENDED.to_owned()),
            ..held()
        };
        assert_eq!(
            ran.outcome,
            forked(&refused),
            "a case whose expectation no replay read alone gives the end of the module's \
             process, and does not pass"
        );
        drop(ran.release);
    }

    /// The facts of a case whose three expectations held in order, and the
    /// text of the second.
    fn held_three() -> Facts {
        Facts {
            in_order: vec![(0, true), (1, true), (2, true)],
            texts: vec![(1, "assert x == 1".to_owned())],
            stated: true,
            ..Facts::default()
        }
    }

    /// Runs a case whose replay of the second expectation sends `verdict`,
    /// and whose module's process then ends before the replay of the third.
    fn ended_after_a_replay_of(verdict: Verdict) -> Ran {
        let sent = Facts {
            later: vec![1, 2],
            ..held_three()
        };
        let frames = case(sent).into_iter().chain(replayed(verdict)).collect();
        run_ending(frames, Ends::AfterItsFrames)
    }

    #[test]
    fn a_replay_whose_module_process_ends_before_its_frames_gives_the_refusal_of_its_end() {
        let ran = ended_after_a_replay_of(Verdict::Holds);
        let refused = Facts {
            alone: vec![(1, Verdict::Holds)],
            refused: Some(ENDED.to_owned()),
            ..held_three()
        };
        assert_eq!(
            ran.outcome,
            forked(&refused),
            "a case whose last expectation no replay read alone gives the end of the \
             module's process, and does not pass"
        );
        drop(ran.release);
    }

    #[test]
    fn a_replay_whose_module_process_ends_before_its_frames_keeps_each_verdict_before_it() {
        let ran = ended_after_a_replay_of(Verdict::Refused);
        let refused = Facts {
            alone: vec![(1, Verdict::Refused)],
            refused: Some(ENDED.to_owned()),
            ..held_three()
        };
        assert_eq!(
            ran.outcome,
            forked(&refused),
            "an assert that the replay before the end reached beats the end, as it beats a \
             fork that failed"
        );
        drop(ran.release);
    }

    #[test]
    fn a_replay_that_ends_with_no_verdict_reads_as_one_that_raised() {
        let (verdict, _) = replay_on(vec![
            frame_of(&Message::Forked(0)),
            frame_of(&Message::Ended(0)),
        ]);
        assert_eq!(
            verdict,
            Ok(format!("{:?}", Ok::<_, String>(Verdict::Raises))),
            "a replay that exits 0 with no verdict did not read its expectation"
        );
    }

    /// Runs the case `test_one` on `frames`, from a stand-in that ends after
    /// them. Gives the outcome, or the `Debug` text of the error, beside the
    /// end that the run of the next test then gives: the line of the end, and
    /// whether the case named it.
    fn run_to_the_end(frames: Vec<Vec<u8>>) -> (Result<Kind, String>, Option<(String, bool)>) {
        to_the_end(zygote_on(frames, Ends::AfterItsFrames))
    }

    /// Runs the case `test_one` on the stand-in `zygote`, whose receiver it
    /// holds, and gives what `run_to_the_end` gives.
    fn to_the_end(
        (mut zygote, _sent): (Zygote, mpsc::Receiver<Option<File>>),
    ) -> (Result<Kind, String>, Option<(String, bool)>) {
        let (done, ran) = mpsc::channel();
        std::thread::spawn(move || {
            let outcome = outcome_of(zygote.run("test_one", &[]));
            let end = match zygote.run("test_two", &[]) {
                Gave::Ended(end) => Some((end.error.line(), end.named)),
                Gave::Ran(_) | Gave::Unreached => None,
            };
            drop(done.send((outcome, end)));
        });
        ran.recv_timeout(Duration::from_secs(10))
            .expect("the run returns without a frame that does not come")
    }

    #[test]
    fn a_module_process_that_ends_after_a_case_with_no_replay_leaves_it_its_outcome() {
        let (outcome, end) = run_to_the_end(vec![frame_of(&passed())]);
        assert_eq!(
            outcome,
            Ok(Kind::Passed),
            "each reading came, so the end is no refusal of the case"
        );
        assert_eq!(
            end,
            Some((format!("project error: {ENDED}"), false)),
            "the end is given once, and the case's line does not name it"
        );
    }

    #[test]
    fn a_module_process_that_ends_before_a_replay_gives_an_end_that_the_case_named() {
        let sent = Facts {
            later: vec![1],
            ..held()
        };
        let (_, end) = run_to_the_end(vec![frame_of(&Message::Facts(Box::new(sent)))]);
        assert_eq!(
            end,
            Some((format!("project error: {ENDED}"), true)),
            "the case's line is the end, so it names it"
        );
    }

    /// The outcome that a case whose line is the end gives, as `to_the_end`
    /// gives it.
    fn the_end() -> Result<Kind, String> {
        Err(format!("{:?}", Error::project(ENDED.to_owned())))
    }

    #[test]
    fn a_module_process_that_ends_before_the_facts_gives_an_end_that_the_case_named() {
        assert_eq!(
            run_to_the_end(Vec::new()),
            (the_end(), Some((format!("project error: {ENDED}"), true))),
            "a case with no facts takes the end as its line, so it names it"
        );
    }

    #[test]
    fn a_module_process_that_ends_before_the_ask_of_a_replay_gives_an_end_that_the_case_named() {
        let sent = Facts {
            later: vec![1],
            ..held()
        };
        assert_eq!(
            to_the_end(zygote_on(case(sent).into(), Ends::BeforeTheReplay)),
            (the_end(), Some((format!("project error: {ENDED}"), true))),
            "the refusal of the replay is the end, and it is the case's line, so it names it"
        );
    }

    #[test]
    fn a_module_process_that_no_longer_reads_the_ask_of_a_test_gives_an_end_that_the_case_named() {
        let (zygote, sent) = zygote_on(Vec::new(), Ends::BeforeTheAsk);
        sent.recv_timeout(Duration::from_secs(10))
            .expect("the stand-in closes its pipes");
        assert_eq!(
            to_the_end((zygote, sent)),
            (the_end(), Some((format!("project error: {ENDED}"), true))),
            "the ask of the test finds the end, which is the case's line, so it names it"
        );
    }

    #[test]
    fn a_module_process_that_ends_after_the_facts_gives_the_refusal_of_its_end() {
        let sent = Facts {
            later: vec![1],
            ..held()
        };
        let ran = run_ending(
            vec![frame_of(&Message::Facts(Box::new(sent)))],
            Ends::AfterItsFrames,
        );
        let refused = Facts {
            refused: Some(ENDED.to_owned()),
            ..held()
        };
        assert_eq!(
            ran.outcome,
            forked(&refused),
            "a case whose expectation no replay read alone gives the end of the module's \
             process, and does not pass"
        );
        drop(ran.release);
    }

    /// The frames of a replay that sends `error`, and that the zygote then
    /// sees end with the exit code of a panic.
    fn replay_with(error: Error) -> [Vec<u8>; 3] {
        [
            frame_of(&Message::Forked(0)),
            frame_of(&Message::Error(error)),
            frame_of(&Message::Ended(125 << 8)),
        ]
    }

    #[test]
    fn a_replay_that_sends_an_internal_error_gives_the_case_that_error() {
        let payload: Box<dyn std::any::Any + Send> = Box::new("a panic in the replay");
        let internal = crate::caught(payload.as_ref());
        let expected = format!("{internal:?}");
        let sent = Facts {
            later: vec![1],
            ..held()
        };
        let frames = case(sent)
            .into_iter()
            .chain(replay_with(internal))
            .collect();
        let ran = run_on(frames);
        assert_eq!(
            ran.outcome,
            Err(expected),
            "the internal error of the replay takes the place of the line that the order gives, \
             as it does for a case"
        );
        drop(ran.release);
    }

    #[test]
    fn a_replay_that_sends_a_project_error_gives_the_case_that_error() {
        let error = Error::project("test_one.py did not load".to_owned());
        let expected = format!("{error:?}");
        let sent = Facts {
            later: vec![1],
            ..raised()
        };
        let frames = case(sent).into_iter().chain(replay_with(error)).collect();
        let ran = run_on(frames);
        assert_eq!(
            ran.outcome,
            Err(expected),
            "an error that the replay's own process sends takes the place of the line that the \
             order gives, and is no refusal of the zygote"
        );
        drop(ran.release);
    }
}
