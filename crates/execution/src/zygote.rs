//! The zygote: a process that purba forks for one module. It loads the module
//! once, then forks a process for each test that purba names to it. Each process
//! writes what it found to purba as frames, on one pipe.
//!
//! When a Python thread other than the main one is alive after the load, the
//! zygote forks no test. purba then forks a zygote that loads nothing, and each
//! test's process loads the module before its test.

use std::fs::File;
use std::io::{self, ErrorKind, Read, Write};
use std::os::fd::{FromRawFd, OwnedFd};
use std::panic::AssertUnwindSafe;
use std::path::PathBuf;
use std::str::FromStr;
use std::sync::{Mutex, PoisonError};

use pyo3::Python;
use pyo3::types::PyAnyMethods;

use purba_assertion::{Readers, Replaced, Verdict};

use crate::fork::{Forked, fork};
use crate::order::{Called, Facts};
use crate::step::Gives;
use crate::{
    Error, Kind, Module, Outcome, Owner, Reason, Shown, Suppressed, Worker, frame, interrupt, panic,
};

/// Each message that crosses a pipe between purba, a zygote and a test's
/// process. Each one is one frame: the tag of its kind, then each of its
/// fields, as `put` writes it. A field holds any text, a NUL too.
#[derive(Debug)]
enum Message {
    /// purba names the test to run: the name of its function.
    Test { function: String },
    /// purba asks for a replay of the test `function` that reads the
    /// expectation at `index` alone.
    Replay { function: String, index: usize },
    /// The load ended, and the zygote forks each test from now on.
    Ready,
    /// A thread is alive after the load, and the zygote ends.
    Threaded,
    /// The load returned, and kept its module or met an interrupt.
    Loaded,
    /// The test's process started: its pid.
    Forked(libc::pid_t),
    /// The facts of the case, from which purba decides its outcome.
    Facts(Box<Facts>),
    /// The first interrupt landed in the test's step, so the case is
    /// `interrupted`. It comes after the facts, when the call ran.
    Interrupted,
    /// The error in place of the result of a load or the outcome of a case.
    Error(Error),
    /// The error of a thread that the test left, as text.
    TestThread(String),
    /// The error of a thread that the load left, as text.
    LoadThread(String),
    /// The first interrupt that the process kept: the number of its signal.
    Kept(i32),
    /// What a replay read of its expectation.
    Verdict(Verdict),
    /// The zygote could not fork a replay: its words.
    Refused(String),
    /// The test's process or the replay ended: its wait status.
    Ended(libc::c_int),
}

/// The tag of each kind of message.
mod tag {
    pub const TEST: u8 = b'n';
    pub const REPLAY: u8 = b'y';
    pub const READY: u8 = b'g';
    pub const THREADED: u8 = b't';
    pub const LOADED: u8 = b'l';
    pub const FORKED: u8 = b'c';
    pub const FACTS: u8 = b'a';
    pub const INTERRUPTED: u8 = b'i';
    pub const PROJECT: u8 = b'e';
    pub const INTERNAL: u8 = b'x';
    pub const TEST_THREAD: u8 = b'h';
    pub const LOAD_THREAD: u8 = b'H';
    pub const KEPT: u8 = b'k';
    pub const VERDICT: u8 = b'v';
    pub const REFUSED: u8 = b'r';
    pub const ENDED: u8 = b'w';
}

impl Message {
    /// Writes the message as one frame.
    fn write(&self, to: &mut impl Write) -> io::Result<()> {
        let mut text = String::new();
        let mut put = |field: &str| put(&mut text, field);
        let tag = match self {
            Self::Test { function } => {
                put(function);
                tag::TEST
            }
            Self::Replay { function, index } => {
                put(function);
                put(&index.to_string());
                tag::REPLAY
            }
            Self::Ready => tag::READY,
            Self::Threaded => tag::THREADED,
            Self::Loaded => tag::LOADED,
            Self::Forked(pid) => {
                put(&pid.to_string());
                tag::FORKED
            }
            Self::Facts(facts) => {
                put_facts(&mut put, facts);
                tag::FACTS
            }
            Self::Interrupted => tag::INTERRUPTED,
            Self::Error(Error(Reason::Project(project))) => {
                put(project);
                tag::PROJECT
            }
            Self::Error(Error(Reason::Internal(caught))) => {
                put(&caught.part);
                put(&caught.message);
                tag::INTERNAL
            }
            Self::TestThread(error) => {
                put(error);
                tag::TEST_THREAD
            }
            Self::LoadThread(error) => {
                put(error);
                tag::LOAD_THREAD
            }
            Self::Kept(signum) => {
                put(&signum.to_string());
                tag::KEPT
            }
            Self::Verdict(verdict) => {
                put(letter(*verdict));
                tag::VERDICT
            }
            Self::Refused(words) => {
                put(words);
                tag::REFUSED
            }
            Self::Ended(status) => {
                put(&status.to_string());
                tag::ENDED
            }
        };
        frame::write(to, tag, &text)
    }

    /// Reads one message. Gives `None` when the pipe ends before a frame
    /// starts. A frame that holds no message gives an internal error of
    /// execution in its place.
    fn read(from: &mut impl Read) -> io::Result<Option<Result<Self, Error>>> {
        Ok(frame::read(from)?.map(|(tag, text)| {
            Self::of(tag, &text).ok_or_else(|| {
                Error(Reason::Internal(panic::Caught {
                    part: "execution".to_owned(),
                    message: format!("the frame {:?} holds no message", char::from(tag)),
                }))
            })
        }))
    }

    /// The message that the frame of `tag` and `text` holds, or `None` when it
    /// holds none.
    fn of(tag: u8, text: &str) -> Option<Self> {
        let mut fields = Fields(text);
        let message = match tag {
            tag::TEST => Self::Test {
                function: fields.text()?,
            },
            tag::REPLAY => Self::Replay {
                function: fields.text()?,
                index: fields.number()?,
            },
            tag::READY => Self::Ready,
            tag::THREADED => Self::Threaded,
            tag::LOADED => Self::Loaded,
            tag::FORKED => Self::Forked(fields.number()?),
            tag::FACTS => Self::Facts(Box::new(fields.facts()?)),
            tag::INTERRUPTED => Self::Interrupted,
            tag::PROJECT => Self::Error(Error::project(fields.text()?)),
            tag::INTERNAL => Self::Error(Error(Reason::Internal(panic::Caught {
                part: fields.text()?,
                message: fields.text()?,
            }))),
            tag::TEST_THREAD => Self::TestThread(fields.text()?),
            tag::LOAD_THREAD => Self::LoadThread(fields.text()?),
            tag::KEPT => Self::Kept(fields.number()?),
            tag::VERDICT => Self::Verdict(fields.verdict()?),
            tag::REFUSED => Self::Refused(fields.text()?),
            tag::ENDED => Self::Ended(fields.number()?),
            _ => return None,
        };
        fields.0.is_empty().then_some(message)
    }
}

/// The fields of a frame that are left to read.
struct Fields<'a>(&'a str);

impl Fields<'_> {
    /// Takes the next field, which `put` wrote.
    fn text(&mut self) -> Option<String> {
        let (length, rest) = self.0.split_once('\0')?;
        let length: usize = length.parse().ok()?;
        let field = rest.get(..length)?;
        self.0 = &rest[length..];
        Some(field.to_owned())
    }

    /// Takes the next field as a number.
    fn number<N: FromStr>(&mut self) -> Option<N> {
        self.text()?.parse().ok()
    }

    /// Takes the next field as a verdict, which `letter` wrote.
    fn verdict(&mut self) -> Option<Verdict> {
        Some(match self.text()?.as_str() {
            "h" => Verdict::Holds,
            "f" => Verdict::Fails,
            "r" => Verdict::Raises,
            "x" => Verdict::Refused,
            _ => return None,
        })
    }

    /// Takes the next field as a flag, which `put_facts` wrote as `1` or `0`.
    fn flag(&mut self) -> Option<bool> {
        match self.text()?.as_str() {
            "1" => Some(true),
            "0" => Some(false),
            _ => None,
        }
    }

    /// Takes a count, then that many items, each of which `item` takes.
    fn list<I>(&mut self, mut item: impl FnMut(&mut Self) -> Option<I>) -> Option<Vec<I>> {
        let count: usize = self.number()?;
        (0..count).map(|_| item(self)).collect()
    }

    /// Takes an option, which `put_facts` wrote as a list of no field or one.
    /// Gives the list, which `pop` turns into the option.
    fn optional(&mut self) -> Option<Vec<String>> {
        let list = self.list(Self::text)?;
        (list.len() <= 1).then_some(list)
    }

    /// Takes the facts of a case, which `put_facts` wrote.
    fn facts(&mut self) -> Option<Facts> {
        let pair = |fields: &mut Self| Some((fields.text()?, fields.text()?));
        Some(Facts {
            form: self.list(pair)?,
            called: if self.flag()? {
                Called::Raised {
                    exception: self.text()?,
                    whence: self.optional()?.pop(),
                    assertion: self.flag()?,
                }
            } else {
                Called::Returned
            },
            with: self.optional()?.pop(),
            mark: if self.flag()? {
                Some((self.text()?, self.flag()?))
            } else {
                None
            },
            path: self.list(pair)?,
            in_another_try: self.flag()?,
            refused: self.optional()?.pop(),
            in_order: self.list(|fields| Some((fields.number()?, fields.flag()?)))?,
            shown: self.list(|fields| Some((fields.number()?, fields.shown()?)))?,
            alone: self.list(|fields| Some((fields.number()?, fields.verdict()?)))?,
            later: self.list(Self::number)?,
            texts: self.list(|fields| Some((fields.number()?, fields.text()?)))?,
            missed: self.list(Self::text)?,
            stated: self.flag()?,
        })
    }

    /// Takes what an expectation that did not hold shows.
    fn shown(&mut self) -> Option<Shown> {
        Some(match self.text()?.as_str() {
            "1" => Shown::Values {
                expected: self.text()?,
                reading: self.text()?,
            },
            "2" => Shown::Items {
                heading: self.text()?,
                items: self.list(Self::text)?,
            },
            _ => return None,
        })
    }
}

/// Writes the fields of `facts` through `put`, in the order that
/// [`Fields::facts`] takes them. A list is its count, then its items, and an
/// option is a list of no item or one.
fn put_facts(put: &mut impl FnMut(&str), facts: &Facts) {
    let flag = |set: bool| if set { "1" } else { "0" };
    let option = |put: &mut dyn FnMut(&str), value: Option<&str>| {
        put(flag(value.is_some()));
        if let Some(value) = value {
            put(value);
        }
    };
    let pairs = |put: &mut dyn FnMut(&str), pairs: &[(String, String)]| {
        put(&pairs.len().to_string());
        for (first, second) in pairs {
            put(first);
            put(second);
        }
    };
    pairs(put, &facts.form);
    match &facts.called {
        Called::Returned => put("0"),
        Called::Raised {
            exception,
            whence,
            assertion,
        } => {
            put("1");
            put(exception);
            option(put, whence.as_deref());
            put(flag(*assertion));
        }
    }
    option(put, facts.with.as_deref());
    match &facts.mark {
        Some((assert, held)) => {
            put("1");
            put(assert);
            put(flag(*held));
        }
        None => put("0"),
    }
    pairs(put, &facts.path);
    put(flag(facts.in_another_try));
    option(put, facts.refused.as_deref());
    put(&facts.in_order.len().to_string());
    for (index, held) in &facts.in_order {
        put(&index.to_string());
        put(flag(*held));
    }
    put(&facts.shown.len().to_string());
    for (index, shown) in &facts.shown {
        put(&index.to_string());
        match shown {
            Shown::Values { expected, reading } => {
                put("1");
                put(expected);
                put(reading);
            }
            Shown::Items { heading, items } => {
                put("2");
                put(heading);
                put(&items.len().to_string());
                for item in items {
                    put(item);
                }
            }
        }
    }
    put(&facts.alone.len().to_string());
    for (index, verdict) in &facts.alone {
        put(&index.to_string());
        put(letter(*verdict));
    }
    put(&facts.later.len().to_string());
    for index in &facts.later {
        put(&index.to_string());
    }
    put(&facts.texts.len().to_string());
    for (index, text) in &facts.texts {
        put(&index.to_string());
        put(text);
    }
    put(&facts.missed.len().to_string());
    for words in &facts.missed {
        put(words);
    }
    put(flag(facts.stated));
}

/// The letter of `verdict` in a frame.
const fn letter(verdict: Verdict) -> &'static str {
    match verdict {
        Verdict::Holds => "h",
        Verdict::Fails => "f",
        Verdict::Raises => "r",
        Verdict::Refused => "x",
    }
}

/// Writes `field` after its length in bytes and a NUL, so that a field holds
/// any text, a NUL too.
fn put(frame: &mut String, field: &str) {
    frame.push_str(&field.len().to_string());
    frame.push('\0');
    frame.push_str(field);
}

/// A zygote that purba forked, seen from purba.
pub struct Zygote {
    pid: libc::pid_t,
    path: PathBuf,
    tests: Option<File>,
    frames: File,
    /// The wait status of the zygote, once purba reaped it.
    reaped: Option<libc::c_int>,
    /// The end of the module's process that the last case met, until
    /// `ended` takes it.
    end: Option<End>,
}

/// The end of the module's process, which a case met: its error, and whether
/// the line of the case names it.
#[derive(Debug)]
pub struct End {
    pub error: Error,
    pub named: bool,
}

/// What a child of the zygote sent, up to its end: the facts of a test, the
/// verdict of a replay, the zygote's refusal to fork it, and what takes the
/// place of the outcome. Its end is the wait status of the child, or the wait
/// status of the module's process, which ended first.
struct Sent {
    facts: Option<Facts>,
    verdict: Option<Verdict>,
    refused: Option<String>,
    result: Option<Result<Outcome, Error>>,
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

/// The error of each thread that a test's process found, beside its owner.
type Threads<T> = Vec<(Owner<T>, Error)>;

impl<T> Worker<'_, T> {
    /// Forks the zygote that forks a process for each test of `module`. Gives
    /// the zygote and the error of each thread that the load took, or gives
    /// what stopped the module.
    ///
    /// The zygote loads the module. When a Python thread is alive after the
    /// load, that zygote ends. A zygote that loads nothing then forks the
    /// processes. Each of them loads the module again before its test.
    ///
    /// # Errors
    ///
    /// The module stops when a zygote cannot fork, when the load fails, or
    /// when a zygote ends before it is ready.
    pub fn zygote(&self, module: &Module) -> Result<(Zygote, Vec<Error>), Stop> {
        let mut zygote = self.forked(module, true).map_err(|error| Stop {
            error,
            threads: Vec::new(),
        })?;
        let Load {
            result,
            threads,
            threaded,
        } = zygote.loaded();
        if let Err(error) = result {
            zygote.end();
            return Err(Stop { error, threads });
        }
        if !threaded {
            return Ok((zygote, threads));
        }
        zygote.end();
        let mut zygote = match self.forked(module, false) {
            Ok(zygote) => zygote,
            Err(error) => return Err(Stop { error, threads }),
        };
        match zygote.loaded().result {
            Ok(()) => Ok((zygote, threads)),
            Err(error) => {
                zygote.end();
                Err(Stop { error, threads })
            }
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
                        self.py,
                        module,
                        load,
                        File::from(tests_in),
                        File::from(frames_out),
                    );
                }))
                .err();
                if let (Ok(mut spare), Some(payload)) = (spare, payload) {
                    let error = Error::caught(payload.as_ref(), "execution");
                    drop(Message::Error(error).write(&mut spare));
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
                    reaped: None,
                    end: None,
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
        let before = interrupt::first();
        let mut threads: Threads<()> = Vec::new();
        let end = loop {
            match self.own(None, &mut threads) {
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
            } else if load.result.is_ok() && !interrupt::landed_since(before) {
                load.result = Err(Error::project(lost(status, "its load ended")));
            }
        }
        load
    }

    /// Reads the next message that is the caller's own. On the way, it takes
    /// each message that every caller takes alike. They are an interrupt that
    /// a child kept, the pid of a test's process that the zygote forked, and
    /// the error of a thread. The error of a thread goes into `threads`, with
    /// its owner. `owner` owns the error of a test's thread. The module owns
    /// the error of a load's thread, or of a test's thread with no `owner`.
    ///
    /// When no message comes, it calls `abandon`. It then gives the error of
    /// the frame and the wait status. The error is `None` when the pipe ended
    /// or failed.
    fn own<T: Clone>(
        &mut self,
        owner: Option<&T>,
        threads: &mut Threads<T>,
    ) -> Result<Message, (Option<Error>, libc::c_int)> {
        loop {
            match self.next() {
                Ok(Message::Kept(signum)) => interrupt::adopt(signum),
                Ok(Message::Forked(pid)) => interrupt::follow_test(pid),
                Ok(Message::TestThread(text)) => {
                    let owner = owner.map_or_else(
                        || Owner::Load(self.path.clone()),
                        |owner| Owner::Test(owner.clone()),
                    );
                    threads.push((owner, Error::project(text)));
                }
                Ok(Message::LoadThread(text)) => {
                    threads.push((Owner::Load(self.path.clone()), Error::project(text)));
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
    /// thread that it found. `owner` owns each thread that the test left. When
    /// the facts name later expectations, it asks for a replay of each, and
    /// then decides the outcome and the warnings of the case in one order,
    /// with the readings alone that a test's process that forked its readers
    /// would have.
    ///
    /// The process can give `Interrupted` or an error in place of the outcome.
    /// A process that ended with no outcome gives `Interrupted` when the first
    /// interrupt arrived as it ran. Otherwise it gives `Raised`, which names
    /// the signal or the exit code.
    ///
    /// When the module's process ends before each replay sent its reading, the
    /// case gets its end as a refusal, as when the zygote cannot fork a
    /// replay. Once the module's process ended, `ended` gives that end.
    pub fn run<T: Clone>(
        &mut self,
        function: &str,
        suppressions: &[(&str, &str)],
        owner: T,
    ) -> (Result<Outcome, Error>, Suppressed, Threads<T>) {
        let mut threads = Vec::new();
        let test = Message::Test {
            function: function.to_owned(),
        };
        let (result, suppressed) = match self.ask(&test) {
            Ok(()) => self.case(function, suppressions, &owner, &mut threads),
            Err(gone) => (Err(Error::project(gone)), Suppressed::default()),
        };
        if let Some(status) = self.reaped {
            let words = gone(status);
            let named = matches!(&result, Err(Error(Reason::Project(line))) if *line == words);
            self.end = Some(End {
                error: Error::project(words),
                named,
            });
        }
        (result, suppressed, threads)
    }

    /// The end of the module's process that the last case met. Gives it once.
    pub const fn ended(&mut self) -> Option<End> {
        self.end.take()
    }

    /// Reads the case of the test `function`, which the zygote forked, and
    /// asks for its replays. Gives the outcome or what takes its place, and
    /// what the suppressions of the case gave.
    fn case<T: Clone>(
        &mut self,
        function: &str,
        suppressions: &[(&str, &str)],
        owner: &T,
        threads: &mut Threads<T>,
    ) -> (Result<Outcome, Error>, Suppressed) {
        let before = interrupt::first();
        let Sent {
            mut facts,
            mut result,
            end,
            ..
        } = self.sent(owner, threads);
        if let Some(facts) = facts.as_mut()
            && result.is_none()
        {
            match end {
                Ok(_) => result = self.replays(function, facts, owner, threads),
                Err(status) if !facts.later.is_empty() => {
                    facts.refused.get_or_insert_with(|| gone(status));
                }
                Err(_) => {}
            }
        }
        let decided = facts.map(|facts| crate::order::order(&facts, suppressions));
        let (own, suppressed) = decided.map_or_else(
            || (None, Suppressed::default()),
            |(own, suppressed)| (Some(own.map(|kind| Outcome { kind })), suppressed),
        );
        let result = result.or(own).unwrap_or_else(|| match end {
            Ok(_) if interrupt::landed_since(before) => Ok(Outcome {
                kind: Kind::Interrupted,
            }),
            Ok(status) => Ok(Outcome {
                kind: Kind::Raised {
                    exception: lost(status, "its outcome"),
                    whence: None,
                },
            }),
            Err(status) => Err(Error::project(gone(status))),
        });
        (result, suppressed)
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

    /// Reads what a child of the zygote sends, until it ends. `owner` owns
    /// each thread that the child's test left.
    fn sent<T: Clone>(&mut self, owner: &T, threads: &mut Threads<T>) -> Sent {
        let mut sent = Sent {
            facts: None,
            verdict: None,
            refused: None,
            result: None,
            end: Ok(0),
        };
        sent.end = loop {
            match self.own(Some(owner), threads) {
                Ok(Message::Ended(status)) => {
                    interrupt::follow_test(0);
                    break Ok(status);
                }
                Ok(Message::Facts(found)) => sent.facts = Some(*found),
                Ok(Message::Verdict(verdict)) => sent.verdict = Some(verdict),
                Ok(Message::Refused(words)) => sent.refused = Some(words),
                Ok(Message::Interrupted) => {
                    sent.result = Some(Ok(Outcome {
                        kind: Kind::Interrupted,
                    }));
                }
                Ok(Message::Error(error)) => sent.result = Some(Err(error)),
                Ok(_) => {}
                Err((unreadable, status)) => {
                    if let Some(error) = unreadable {
                        sent.result = Some(Err(error));
                    }
                    break Err(status);
                }
            }
        };
        sent
    }

    /// Asks the zygote for a replay of `function` for each later expectation
    /// that `facts` name, in order, and adds each verdict to the readings alone
    /// of `facts`. When the zygote cannot fork a replay, or the module's
    /// process ends before a replay sent its reading, it puts the refusal in
    /// `facts` and asks for no later replay, as a test's process does when it
    /// cannot fork a reader. Either way, it gives `None`, and the order decides
    /// the case. It gives a result in place of the outcome only when a replay
    /// sends an error, a frame holds no message, or an interrupt lands during
    /// the replays.
    fn replays<T: Clone>(
        &mut self,
        function: &str,
        facts: &mut Facts,
        owner: &T,
        threads: &mut Threads<T>,
    ) -> Option<Result<Outcome, Error>> {
        let before = interrupt::first();
        let mut alone = Vec::new();
        for &index in &facts.later {
            let replayed = match self.replay(function, index, owner, threads) {
                Ok(replayed) => replayed,
                Err(error) => return Some(Err(error)),
            };
            if interrupt::landed_since(before) {
                return Some(Ok(Outcome {
                    kind: Kind::Interrupted,
                }));
            }
            match replayed {
                Ok(verdict) => alone.push((index, verdict)),
                Err(refused) => {
                    facts.refused = Some(refused);
                    break;
                }
            }
        }
        facts.alone.extend(alone);
        None
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
    fn replay<T: Clone>(
        &mut self,
        function: &str,
        index: usize,
        owner: &T,
        threads: &mut Threads<T>,
    ) -> Result<Result<Verdict, String>, Error> {
        let replay = Message::Replay {
            function: function.to_owned(),
            index,
        };
        if let Err(gone) = self.ask(&replay) {
            return Ok(Err(gone));
        }
        let Sent {
            verdict,
            refused,
            result,
            end,
            ..
        } = self.sent(owner, threads);
        if let Some(Err(error)) = result {
            return Err(error);
        }
        Ok(match (refused, verdict, end) {
            (Some(refused), _, _) => Err(refused),
            (None, Some(verdict), _) => Ok(verdict),
            (None, None, Ok(_)) => Ok(Verdict::Raises),
            (None, None, Err(status)) => Err(gone(status)),
        })
    }

    /// Tells the zygote that no test is left, and waits for it to end.
    pub fn end(mut self) {
        self.wait();
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

/// The words of the module's process that ended with the wait status
/// `status`: `the module's process died by SIGKILL`.
fn gone(status: libc::c_int) -> String {
    format!("the module's process {}", ended(status))
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

/// Whether a Python thread other than the main one is alive.
fn threaded(py: Python<'_>) -> bool {
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

/// The life of a child of the zygote: a test's process, or a replay, as `way`
/// says. Each child sends its pid, loads the module when `children` says so,
/// and sends what it found and the first interrupt that it kept. A process
/// that the test forks and that returns from the test ends with no frame.
///
/// A test's process runs the test `function`, sends its facts, joins its
/// threads, runs its `atexit` functions, and sends the error of each thread.
///
/// A replay runs the test again, and reads only the expectation at its index.
/// It ends at that expectation, and `REPLAYS` sends its verdict. It writes
/// nothing to stdout or stderr, since the case wrote what the test writes
/// once. It sends no error of a thread, since the case sent them once.
fn live_as_child(
    py: Python<'_>,
    worker: &mut Worker<'_, String>,
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
    let result = crate::catch("execution", || {
        if children.loads {
            worker.load(children.module)?;
            // The load in the first zygote already handed on the errors of these
            // threads.
            drop(worker.threads());
            if interrupt::kept_now() {
                return Ok(Gives::Interrupted);
            }
        }
        match way {
            Way::Test => {
                let (gathered, gives) = worker.gather(function, function.to_owned());
                facts = gathered;
                gives
            }
            Way::Replay(index) => worker.replay(function, index).map(Gives::Own),
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
        (Way::Replay(_), Ok(Gives::Own(()))) => REPLAYS.end(Verdict::Raises),
        (_, Ok(Gives::Own(()))) | (Way::Replay(_), Ok(Gives::Interrupted)) => {}
        (Way::Test, Ok(Gives::Interrupted)) => send(&mut frames, &Message::Interrupted),
        (_, Err(error)) => send(&mut frames, &Message::Error(error)),
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
        send(frames, &Message::Kept(signum));
    }
}

/// Writes the error of each thread that the worker took.
fn send_threads(worker: &Worker<'_, String>, frames: &mut File) {
    for (owner, error) in worker.threads() {
        let message = match owner {
            Owner::Load(_) => Message::LoadThread(error.to_string()),
            Owner::Test(_) => Message::TestThread(error.to_string()),
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

    use std::sync::mpsc;
    use std::time::Duration;

    /// One message of each variant, with a NUL, a line feed and text that is
    /// not ASCII in its fields, an empty field, and each shape of a field that
    /// is optional.
    fn each_variant() -> Vec<Message> {
        let text = |words: &str| format!("{words}\0\n é");
        vec![
            Message::Test {
                function: text("test_one"),
            },
            Message::Replay {
                function: text("test_one"),
                index: 2,
            },
            Message::Ready,
            Message::Threaded,
            Message::Loaded,
            Message::Forked(4321),
            Message::Facts(Box::new(Facts {
                form: vec![
                    (text("iterator-read"), text("words")),
                    (String::new(), String::new()),
                ],
                called: Called::Raised {
                    exception: text("ValueError: no"),
                    whence: Some(text("helper")),
                    assertion: true,
                },
                with: Some(text("with raises(ValueError):")),
                mark: Some((text("assert x == 1"), false)),
                path: vec![(text("second-path"), text("line 3"))],
                in_another_try: true,
                refused: Some(text("purba cannot fork")),
                in_order: vec![(1, true), (2, false)],
                shown: vec![
                    (
                        2,
                        Shown::Values {
                            expected: text("1"),
                            reading: text("2"),
                        },
                    ),
                    (
                        3,
                        Shown::Items {
                            heading: text("the first item that breaks"),
                            items: vec![text("0"), String::new()],
                        },
                    ),
                ],
                alone: vec![
                    (2, Verdict::Holds),
                    (3, Verdict::Fails),
                    (4, Verdict::Raises),
                    (5, Verdict::Refused),
                ],
                later: vec![2, 3],
                texts: vec![(1, text("assert x == 1")), (2, String::new())],
                missed: vec![text("try: x()")],
                stated: true,
            })),
            Message::Facts(Box::new(Facts {
                called: Called::Raised {
                    exception: text("ValueError: no"),
                    whence: None,
                    assertion: false,
                },
                ..Facts::default()
            })),
            Message::Facts(Box::default()),
            Message::Interrupted,
            Message::Error(Error::project(text("[no-expectation] no expectation ran"))),
            Message::Error(Error(Reason::Internal(panic::Caught {
                part: text("assertion"),
                message: text("a panic"),
            }))),
            Message::TestThread(text("thread A raised ValueError")),
            Message::LoadThread(text("thread L raised ValueError")),
            Message::Kept(libc::SIGTERM),
            Message::Verdict(Verdict::Holds),
            Message::Verdict(Verdict::Fails),
            Message::Verdict(Verdict::Raises),
            Message::Verdict(Verdict::Refused),
            Message::Refused(text("purba cannot fork to read an expectation alone")),
            Message::Ended(256),
        ]
    }

    /// The number of arms of `variant`, one for each variant of `Message`.
    const VARIANTS: usize = 15;

    /// The name of the variant of `message`. A variant that this match lacks
    /// fails to compile. The count above it must then grow, and
    /// `each_variant` must send the new one.
    const fn variant(message: &Message) -> &'static str {
        match message {
            Message::Test { .. } => "Test",
            Message::Replay { .. } => "Replay",
            Message::Ready => "Ready",
            Message::Threaded => "Threaded",
            Message::Loaded => "Loaded",
            Message::Forked(_) => "Forked",
            Message::Facts(_) => "Facts",
            Message::Interrupted => "Interrupted",
            Message::Error(_) => "Error",
            Message::TestThread(_) => "TestThread",
            Message::LoadThread(_) => "LoadThread",
            Message::Kept(_) => "Kept",
            Message::Verdict(_) => "Verdict",
            Message::Refused(_) => "Refused",
            Message::Ended(_) => "Ended",
        }
    }

    #[test]
    fn each_message_reads_back_as_itself() {
        let sent = each_variant();
        let mut pipe = Vec::new();
        for message in &sent {
            message.write(&mut pipe).expect("a Vec takes the write");
        }
        let mut from = pipe.as_slice();
        for message in &sent {
            let read = Message::read(&mut from)
                .expect("a whole frame reads")
                .expect("a frame is left")
                .expect("the frame holds a message");
            assert_eq!(
                format!("{read:?}"),
                format!("{message:?}"),
                "a {} message reads back as itself",
                variant(message)
            );
        }
        assert!(
            Message::read(&mut from).expect("the end reads").is_none(),
            "the pipe ends after the last message"
        );
        let mut names: Vec<&str> = sent.iter().map(variant).collect();
        names.sort_unstable();
        names.dedup();
        assert_eq!(names.len(), VARIANTS, "each variant is sent: {names:?}");
    }

    /// The internal error of a frame of `tag` that holds no message.
    fn no_message(tag: char) -> String {
        format!(
            "{:?}",
            Error(Reason::Internal(panic::Caught {
                part: "execution".to_owned(),
                message: format!("the frame {tag:?} holds no message"),
            }))
        )
    }

    #[test]
    fn a_frame_that_holds_no_message_reads_as_an_internal_error() {
        let frames: [(u8, &str, &str); 8] = [
            (tag::FACTS, "9\0cut", "a field longer than the frame"),
            (b'?', "", "a tag of no message"),
            (tag::FORKED, "1\0x", "a pid that is no number"),
            (tag::REPLAY, "1\0a", "a replay with one of its two fields"),
            (tag::ENDED, "1\0x", "a wait status that is no number"),
            (tag::KEPT, "1\0x", "a signal that is no number"),
            (tag::READY, "1\0x", "a field left over"),
            (
                tag::FACTS,
                "1\x0001\x002",
                "a flag of the facts that is neither 0 nor 1",
            ),
        ];
        let mut pipe = Vec::new();
        for (tag, text, _) in frames {
            frame::write(&mut pipe, tag, text).expect("a Vec takes the write");
        }
        let mut from = pipe.as_slice();
        for (tag, _, what) in frames {
            let read = Message::read(&mut from)
                .expect("a whole frame reads")
                .expect("a frame is left");
            assert_eq!(
                read.map_err(|error| format!("{error:?}"))
                    .map(|message| format!("{message:?}")),
                Err(no_message(char::from(tag))),
                "{what}"
            );
        }
    }

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

    /// Runs the case `test_one` on `frames`, as `Zygote::run` reads them.
    ///
    /// A thread stands in for the zygote and the test's process. It writes
    /// each frame, then holds the pipe of frames open until the pipe of test
    /// names closes, as a zygote does. The `Zygote` lives on after the run, as
    /// dispatch keeps it for the next test, until `release` drops. A run that
    /// waits for a frame that does not come fails the case after 10 seconds.
    fn run_on(frames: Vec<(u8, String)>) -> Ran {
        run_ending(frames, Ends::Never)
    }

    /// Runs the case `test_one` on `frames`, as `run_on` does, from a stand-in
    /// that ends as `ends` says.
    fn run_ending(frames: Vec<(u8, String)>, ends: Ends) -> Ran {
        let (mut zygote, sent) = zygote_on(frames, ends);
        let (done, outcome) = mpsc::channel();
        let (release, released) = mpsc::channel::<()>();
        std::thread::spawn(move || {
            let (result, _, _) = zygote.run("test_one", &[], ());
            drop(
                done.send(
                    result
                        .map(|outcome| outcome.kind().clone())
                        .map_err(|error| format!("{error:?}")),
                ),
            );
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
    }

    /// A `Zygote` of the module `test_one.py` that reads `frames`, and a
    /// receiver that hears when their sender ends, and holds the pipe of
    /// frames that it hands over.
    ///
    /// A thread stands in for the zygote and the test's process. It writes
    /// each frame, and ends as `ends` says.
    fn zygote_on(frames: Vec<(u8, String)>, ends: Ends) -> (Zygote, mpsc::Receiver<Option<File>>) {
        let (frames_in, frames_out) = pipe().expect("a pipe opens");
        let (tests_in, tests_out) = pipe().expect("a pipe opens");
        let (sender_done, sent) = mpsc::channel();
        std::thread::spawn(move || {
            let mut asks = File::from(tests_in);
            let mut sending = File::from(frames_out);
            let mut write_each = || {
                for (tag, text) in &frames {
                    if frame::write(&mut sending, *tag, text).is_err() {
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
                    drop(frame::read(&mut asks));
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
            };
            sender_done.send(held).ok();
        });
        let zygote = Zygote {
            // No child has this pid, so a wait for it returns at once.
            pid: libc::pid_t::MAX,
            path: PathBuf::from("test_one.py"),
            tests: Some(File::from(tests_out)),
            frames: File::from(frames_in),
            reaped: None,
            end: None,
        };
        (zygote, sent)
    }

    /// Replays the expectation at index 1 of `test_one` on `frames`. It gives
    /// the `Debug` text of the verdict or the error, beside the `Debug` text of
    /// each thread's owner and error. A replay that waits for a frame that does
    /// not come fails the case after 10 seconds.
    fn replay_on(frames: Vec<(u8, String)>) -> (Result<String, String>, Vec<String>) {
        let (mut zygote, _sent) = zygote_on(frames, Ends::Never);
        let (done, replayed) = mpsc::channel();
        std::thread::spawn(move || {
            let mut threads = Vec::new();
            let verdict = zygote
                .replay("test_one", 1, &"test_one", &mut threads)
                .map(|verdict| format!("{verdict:?}"))
                .map_err(|error| format!("{error:?}"));
            let threads = threads
                .iter()
                .map(|(owner, error)| format!("{owner:?} {error:?}"))
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
                    Owner::Test("test_one"),
                    Error::project("of the test".to_owned())
                ),
                format!(
                    "{:?} {:?}",
                    Owner::<&str>::Load(PathBuf::from("test_one.py")),
                    Error::project("of the load".to_owned())
                ),
            ],
            "a replay charges a test's thread to the test, and a load's thread to the module, \
             as a case does"
        );
    }

    #[test]
    fn a_frame_that_holds_no_message_ends_the_replay_with_its_error() {
        let (verdict, threads) = replay_on(vec![(tag::ENDED, "1\0x".to_owned())]);
        assert_eq!(
            verdict,
            Err(no_message('w')),
            "the internal error takes the place of the verdict, as it does for a case"
        );
        assert!(threads.is_empty(), "no thread is charged: {threads:?}");
    }

    /// The frame of `message`.
    fn frame_of(message: &Message) -> (u8, String) {
        let mut pipe = Vec::new();
        message.write(&mut pipe).expect("a Vec takes the write");
        frame::read(&mut pipe.as_slice())
            .expect("a whole frame reads")
            .expect("a frame is left")
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
        let frames = vec![frame_of(&passed()), (tag::ENDED, "1\0x".to_owned())];
        assert_eq!(
            run_on(frames).outcome,
            Err(no_message('w')),
            "the internal error takes the place of the outcome, and the run waits for no \
             frame that its process does not send"
        );
    }

    #[test]
    fn a_frame_that_holds_no_message_reads_each_frame_after_it() {
        // More than a pipe holds: a sender that nobody reads waits on it.
        let after = frame_of(&Message::TestThread("x".repeat(1024)));
        let frames = std::iter::once((tag::ENDED, "1\0x".to_owned()))
            .chain(std::iter::repeat_n(after, 256))
            .collect();
        let ran = run_on(frames);
        assert_eq!(
            ran.outcome,
            Err(no_message('w')),
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
    fn case(facts: Facts) -> [(u8, String); 2] {
        [
            frame_of(&Message::Facts(Box::new(facts))),
            frame_of(&Message::Ended(0)),
        ]
    }

    /// The frames that the zygote sends when it cannot fork a replay.
    fn refused_replay() -> [(u8, String); 2] {
        [
            frame_of(&Message::Refused(REFUSAL.to_owned())),
            frame_of(&Message::Ended(0)),
        ]
    }

    /// The frames of a replay that sends `verdict`, and ends.
    fn replayed(verdict: Verdict) -> [(u8, String); 3] {
        [
            frame_of(&Message::Forked(0)),
            frame_of(&Message::Verdict(verdict)),
            frame_of(&Message::Ended(0)),
        ]
    }

    /// The line that `order` gives for `facts`, when a test's process with no
    /// thread alive sent them, as `Ran::outcome` holds it.
    fn forked(facts: &Facts) -> Result<Kind, String> {
        crate::order::order(facts, &[])
            .0
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
                let (result, _, _) = zygote.run("test_one", &[], ());
                drop(
                    done.send(
                        result
                            .map(|outcome| outcome.kind().clone())
                            .map_err(|error| format!("{error:?}")),
                    ),
                );
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
        let frames = case(sent)
            .into_iter()
            .chain([
                frame_of(&Message::Forked(0)),
                (tag::ENDED, "1\0x".to_owned()),
            ])
            .collect();
        let ran = run_on(frames);
        assert_eq!(
            ran.outcome,
            Err(no_message('w')),
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
    /// them. Gives the outcome, or the `Debug` text of the error, beside what
    /// `ended` then gives: the line of the end, and whether the case named it.
    fn run_to_the_end(frames: Vec<(u8, String)>) -> (Result<Kind, String>, Option<(String, bool)>) {
        let (mut zygote, _sent) = zygote_on(frames, Ends::AfterItsFrames);
        let (done, ran) = mpsc::channel();
        std::thread::spawn(move || {
            let (result, _, _) = zygote.run("test_one", &[], ());
            let end = zygote.ended().map(|end| (end.error.line(), end.named));
            drop(
                done.send((
                    result
                        .map(|outcome| outcome.kind().clone())
                        .map_err(|error| format!("{error:?}")),
                    end,
                )),
            );
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
    fn replay_with(error: Error) -> [(u8, String); 3] {
        [
            frame_of(&Message::Forked(0)),
            frame_of(&Message::Error(error)),
            frame_of(&Message::Ended(125 << 8)),
        ]
    }

    #[test]
    fn a_replay_that_sends_an_internal_error_gives_the_case_that_error() {
        let payload: Box<dyn std::any::Any + Send> = Box::new("a panic in the replay");
        let internal = Error::caught(payload.as_ref(), "execution");
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
