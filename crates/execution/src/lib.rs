//! Execution: loads a module, runs a test, and makes its outcome. It keeps the
//! first interrupt of the process.

#[cfg(not(any(target_os = "linux", target_os = "macos")))]
compile_error!("purba runs on Linux and macOS. On Windows, run it under WSL2.");

mod fork;
mod frame;
pub mod interrupt;
pub mod panic;
mod paths;
mod reader;
mod step;
mod thread;
mod whence;
mod zygote;

use std::any::Any;
use std::collections::HashMap;
use std::fmt;
use std::panic::AssertUnwindSafe;
use std::path::PathBuf;

/// Assertion's rewrite of a module, which a [`Module`] holds.
pub use purba_assertion::Rewrite;
use purba_assertion::{Loaded, Readings, Swallowed, Try, Verdict, Watched};
use pyo3::exceptions::PyAssertionError;
use pyo3::types::PyAnyMethods;
use pyo3::{Py, PyAny, PyErr, Python};
use serde::{Deserialize, Serialize};
use step::{Gives, Met};
pub use zygote::{Load, Zygote};

/// A module that dispatch hands a worker.
pub struct Module {
    path: PathBuf,
    rewrite: Rewrite,
}

impl Module {
    /// Makes a module from `path` as the tester typed it, and its rewrite.
    #[must_use]
    pub const fn new(path: PathBuf, rewrite: Rewrite) -> Self {
        Self { path, rewrite }
    }
}

/// What a run says about one test. Its field is private, so no part outside
/// execution changes it.
///
/// ```compile_fail,E0451
/// let outcome = purba_execution::Outcome { kind: purba_execution::Kind::Passed };
/// ```
///
/// ```compile_fail,E0616
/// fn forge(outcome: &mut purba_execution::Outcome) {
///     outcome.kind = purba_execution::Kind::Passed;
/// }
/// ```
#[derive(Debug, Serialize, Deserialize)]
pub struct Outcome {
    kind: Kind,
}

impl Outcome {
    /// Whether the case passed, failed, raised, or is `interrupted`.
    #[must_use]
    pub const fn kind(&self) -> &Kind {
        &self.kind
    }
}

/// Passed, failed, raised or interrupted.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Kind {
    /// Each expectation that the case read held, and the test returned.
    Passed,
    /// An expectation did not hold, and the test returned, or a marked assert
    /// did not hold, and the test raised `AssertionError`. `asserts` quotes each
    /// expectation that did not hold, in order, or the last marked assert that
    /// did not hold.
    Failed { asserts: Vec<Quote> },
    /// The test's call raised, and the outcome is not `Failed` or
    /// `Interrupted`. A test whose process ends before its outcome gives
    /// `Raised` too. `exception` is the type and the message of the exception,
    /// or how the process ended, as text. `whence` names the frame that the
    /// exception came from, when it is not the test's own.
    Raised {
        exception: String,
        whence: Option<String>,
    },
    /// The first interrupt landed during the test's step: its lookup or its
    /// call. The test has this outcome whether it then passed, failed or raised.
    Interrupted,
}

/// The text of an assert that did not hold, or of the `with` that raised, and
/// what an expectation that compares two values shows.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Quote {
    pub assert: String,
    pub shown: Option<Shown>,
}

/// What an expectation shows when it does not hold, as assertion gives it.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Shown {
    /// A check of one value: its expected value and its reading.
    Values { expected: String, reading: String },
    /// An `all()` or an `any()`: a heading, and a line for each item that a
    /// failure names.
    Items { heading: String, items: Vec<String> },
}

impl Quote {
    /// A quote of `text` that shows no reading.
    const fn alone(text: String) -> Self {
        Self {
            assert: text,
            shown: None,
        }
    }
}

/// Why `run` made no outcome: a project error or an internal error.
///
/// A project error is text, such as Python's error or a test that selection
/// refuses. Any part builds one with [`Error::project`].
/// An internal error is a panic, with the part that found it and its message.
/// Only execution makes an internal error.
#[derive(Debug)]
pub struct Error(Reason);

#[derive(Debug)]
enum Reason {
    Project(String),
    Internal(panic::Caught),
}

/// Whether an error is a project error or an internal error, and the part
/// that found an internal error.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ErrorKind<'a> {
    /// An error in the tester's code.
    Project,
    /// An error in purba itself, found by `part`.
    Internal { part: &'a str },
}

impl Error {
    /// A project error of `text`.
    #[must_use]
    pub const fn project(text: String) -> Self {
        Self(Reason::Project(text))
    }

    /// Whether this is a project error or an internal error.
    #[must_use]
    pub fn kind(&self) -> ErrorKind<'_> {
        match &self.0 {
            Reason::Project(_) => ErrorKind::Project,
            Reason::Internal(caught) => ErrorKind::Internal { part: &caught.part },
        }
    }

    fn caught(payload: &(dyn Any + Send), catcher: &str) -> Self {
        Self(Reason::Internal(panic::caught(payload, catcher)))
    }
}

/// The text of a project error, or the message of an internal error.
impl fmt::Display for Error {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.0 {
            Reason::Project(text) => formatter.write_str(text),
            Reason::Internal(caught) => formatter.write_str(&caught.message),
        }
    }
}

impl std::error::Error for Error {}

fn error(err: &PyErr) -> Error {
    Error::project(err.to_string())
}

/// Runs `work`, and gives an internal error for a panic that crosses it. The
/// part comes from the note of the panic hook when the note holds the
/// panic's message, and is `catcher` otherwise.
///
/// # Errors
///
/// The error of `work`, or the internal error of its panic.
pub fn catch<R>(catcher: &str, work: impl FnOnce() -> Result<R, Error>) -> Result<R, Error> {
    std::panic::catch_unwind(AssertUnwindSafe(work))
        .unwrap_or_else(|payload| Err(Error::caught(payload.as_ref(), catcher)))
}

/// The load or the test that left a thread alive, which a thread's exception
/// charges. A load names the path of its module. A test is the one that the
/// caller handed to [`Worker::run`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Owner<T> {
    Load(PathBuf),
    Test(T),
}

/// Runs tests in this process. It loads each module once, and keeps it under its
/// path, or keeps that its load failed. It keeps the owner of each thread that a
/// load or a test left alive.
pub struct Worker<'py, T> {
    py: Python<'py>,
    loaded: HashMap<PathBuf, Option<Loaded<'py>>>,
    hook: &'static Py<thread::Hook>,
    owners: Vec<(Py<PyAny>, Owner<T>)>,
    last: Option<Owner<T>>,
    replays: Vec<(usize, bool, String)>,
    suppressed: Suppressed,
}

/// What a load or a call meets, from before its first line of Python: whether
/// purba kept an interrupt as it opened, and the threads alive then.
struct Window {
    kept: bool,
    alive: Vec<Py<PyAny>>,
}

impl Window {
    /// Whether the first interrupt landed since the window opened.
    fn landed(&self) -> bool {
        !self.kept && interrupt::kept_now()
    }
}

/// Gives `work` a worker for as long as it runs.
///
/// Before the first worker, it hooks `threading.excepthook` and
/// `sys.unraisablehook`, for the life of the process. So each run imports
/// `threading`.
///
/// # Panics
///
/// The hook could not take the place of `threading.excepthook` or
/// `sys.unraisablehook`.
pub fn attach<T, R>(work: impl for<'py> FnOnce(&mut Worker<'py, T>) -> R) -> R {
    Python::initialize();
    Python::attach(|py| work(&mut Worker::new(py)))
}

impl<'py, T> Worker<'py, T> {
    /// A worker that has loaded nothing.
    fn new(py: Python<'py>) -> Self {
        whence::keep_the_folder();
        Self {
            py,
            loaded: HashMap::new(),
            hook: thread::hook(py)
                .expect("the hook takes the place of threading.excepthook and sys.unraisablehook"),
            owners: Vec::new(),
            last: None,
            replays: Vec::new(),
            suppressed: Suppressed::default(),
        }
    }
}

impl<'py, T: Clone> Worker<'py, T> {
    /// Calls `function` of `module`, which `load` loaded, and makes its
    /// outcome. When the first interrupt landed in the step, the outcome is
    /// `Interrupted`, and when it landed before the test's call, the call does
    /// not run. Each thread that the step leaves alive gets `test` as its owner,
    /// unless it has one.
    ///
    /// `suppressions` names each check that a suppression of the test reaches,
    /// beside its reason. A fault of such a check gives a warning in place of
    /// its project error. [`Worker::warnings`] then holds the warning. The case
    /// gets the outcome that it would get without that check.
    ///
    /// # Errors
    ///
    /// An internal error when `module` was never given to `load`, when a panic
    /// crossed the test's call, or when the test caught a panic and the panic
    /// hook noted it. A project error when the module did not load, it has no
    /// `function`, or `function` is not callable.
    pub fn run(
        &mut self,
        module: &Module,
        function: &str,
        suppressions: &[(&str, &str)],
        test: T,
    ) -> Result<Outcome, Error> {
        let window = self.open();
        let (own, interrupted) = self.call(&window, module, function, suppressions);
        let met = self.close(&window, Owner::Test(test), interrupted);
        step::decide(own, met).map(|gives| Outcome {
            kind: match gives {
                Gives::Own(kind) => kind,
                Gives::Interrupted => Kind::Interrupted,
            },
        })
    }

    /// Loads `module` once, and keeps it, or keeps that its load failed. The
    /// load takes the note of a panic, so no later test takes it. Each thread that the load
    /// leaves alive gets the load as its owner. When the first interrupt
    /// landed in the load, it gives no project error, and keeps no module.
    ///
    /// # Errors
    ///
    /// The load raised. An internal error when the module caught a panic and the
    /// panic hook noted it. A later call names the module that did not load.
    pub fn load(&mut self, module: &Module) -> Result<(), Error> {
        if self.loaded.contains_key(&module.path) {
            return self.module(module).map(|_| ());
        }
        let window = self.open();
        let py = self.py;
        let holders = paths::holders(py);
        let (own, interrupted) = if window.landed() {
            (Ok(None), true)
        } else if !paths::tried(py)
            && let [(_, Some(third)), (_, Some(fourth))] = &holders
        {
            let own = Err(Error::project(format!(
                "sys.monitoring holds the tool ids 3 and 4 for {third} and {fourth}, \
                 and execution needs one of them to watch"
            )));
            (own, false)
        } else {
            thread::hold_filters(py);
            let free: Vec<u8> = holders
                .iter()
                .filter(|(_, holder)| holder.is_none())
                .map(|(id, _)| *id)
                .collect();
            let (loaded, landed) = interrupt::calling(py, || {
                paths::wrap(py)?;
                purba_assertion::load(py, &module.path, &module.rewrite)
            });
            let own = loaded.map_err(|err| error(&err)).and_then(|loaded| {
                if let Some(replaced) = thread::replaced(py) {
                    return Err(Error::project(format!(
                        "[hook-replaced] the module replaced {} as it loaded, and {}",
                        replaced.what(),
                        replaced.rule()
                    )));
                }
                if !paths::arm(loaded.code()).map_err(|err| error(&err))? {
                    let took = match free.as_slice() {
                        [id] => format!("the tool id {id}"),
                        _ => "the tool ids 3 and 4".to_owned(),
                    };
                    return Err(Error::project(format!(
                        "[hook-replaced] the module took {took} of sys.monitoring as it \
                         loaded, and a test keeps purba's hooks"
                    )));
                }
                Ok(Some(loaded))
            });
            (own, landed)
        };
        let met = self.close(&window, Owner::Load(module.path.clone()), interrupted);
        let (kept, result) = match step::decide(own, met) {
            Ok(Gives::Own(loaded)) => (loaded, Ok(())),
            Ok(Gives::Interrupted) => (None, Ok(())),
            Err(error) => (None, Err(error)),
        };
        self.loaded.insert(module.path.clone(), kept);
        result
    }

    /// The module that `load` loaded for `module`.
    fn module(&self, module: &Module) -> Result<&Loaded<'py>, Error> {
        match self.loaded.get(&module.path) {
            None => Err(Error(Reason::Internal(panic::Caught {
                part: "execution".to_owned(),
                message: format!("{} was never given to load", module.path.display()),
            }))),
            Some(None) => Err(Error::project(format!(
                "{} did not load",
                module.path.display()
            ))),
            Some(Some(loaded)) => Ok(loaded),
        }
    }

    /// Opens the window of a step, before its first line of Python.
    fn open(&self) -> Window {
        let kept = interrupt::kept_now();
        Window {
            kept,
            alive: self.hook.get().alive(self.py),
        }
    }

    /// Closes the window of a step that `owner` ran. Gives `owner` each thread
    /// that the step left alive, and takes the note of a panic.
    fn close(&mut self, window: &Window, owner: Owner<T>, interrupted: bool) -> Met {
        self.own(&window.alive, owner);
        Met {
            panic: panic::take(),
            interrupted,
        }
    }

    /// Takes each exception that a thread raised since the last take, in the
    /// order threads raised them, as a project error with its owner. A thread
    /// with no owner charges the load or the test that ran last. Before any
    /// load or test, it takes nothing.
    pub fn threads(&mut self) -> Vec<(Owner<T>, Error)> {
        let Some(last) = &self.last else {
            return Vec::new();
        };
        self.hook
            .get()
            .take()
            .into_iter()
            .map(|raised| {
                let owner = raised
                    .thread
                    .and_then(|thread| {
                        self.owners
                            .iter()
                            .find(|(owned, _)| owned.is(&thread))
                            .map(|(_, owner)| owner.clone())
                    })
                    .unwrap_or_else(|| last.clone());
                (owner, Error::project(raised.text))
            })
            .collect()
    }

    /// Gives `owner` each thread that is alive now and was not in `before`, and
    /// has no owner yet. `owner` then ran last.
    fn own(&mut self, before: &[Py<PyAny>], owner: Owner<T>) {
        let new: Vec<Py<PyAny>> = self
            .hook
            .get()
            .alive(self.py)
            .into_iter()
            .filter(|thread| {
                !before
                    .iter()
                    .chain(self.owners.iter().map(|(owned, _)| owned))
                    .any(|known| known.is(thread))
            })
            .collect();
        self.owners
            .extend(new.into_iter().map(|thread| (thread, owner.clone())));
        self.last = Some(owner);
    }

    /// Looks `function` up in `module`. Calls it, unless the first interrupt
    /// landed since `window` opened, and watches the paths of the call. Also
    /// says whether the first interrupt landed in the step. When a thread was
    /// alive at the first expectation and the outcome comes from the
    /// expectations, it keeps each later expectation for a replay.
    fn call(
        &mut self,
        window: &Window,
        module: &Module,
        function: &str,
        suppressions: &[(&str, &str)],
    ) -> (Result<Kind, Error>, bool) {
        let py = self.py;
        self.replays.clear();
        self.suppressed = Suppressed::default();
        let loaded = match self.module(module) {
            Ok(loaded) => loaded,
            Err(error) => return (Err(error), window.landed()),
        };
        let test = match loaded.module().getattr(function) {
            Ok(test) => test,
            Err(err) => return (Err(error(&err)), window.landed()),
        };
        // A call of an object that is not callable runs no code, and gives
        // CPython's own `TypeError`.
        if !test.is_callable()
            && let Err(err) = test.call0()
        {
            return (Err(error(&err)), window.landed());
        }
        let purba_fixtures::Inputs {} = purba_fixtures::inputs(function);
        if window.landed() {
            return (Ok(Kind::Interrupted), true);
        }
        let before = loaded.first_expectation(function).unwrap_or(usize::MAX);
        thread::hold_filters(py);
        if let Ok(code) = test.getattr("__code__") {
            paths::start(py, &code, before);
        }
        let mut landed = false;
        let watched = loaded.watch(&reader::READERS, || {
            let (called, inside) = interrupt::calling(py, || {
                std::panic::catch_unwind(AssertUnwindSafe(|| test.call0()))
            });
            landed = inside;
            Ok(called)
        });
        let path = paths::finish(py);
        let Watched {
            result,
            mark,
            readings,
            form,
            raised,
            swallowed,
            missed,
        } = watched;
        let (path, in_another_try) = path_faults(path, &swallowed, loaded.tries(function));
        let result = match result {
            Ok(Ok(called)) => called.map(drop),
            Ok(Err(payload)) => {
                return (Err(Error::caught(payload.as_ref(), "execution")), landed);
            }
            Err(err) => Err(err),
        };
        let with = result.as_ref().err().and_then(|err| {
            let value = err.value(py);
            raised
                .iter()
                .find(|(exception, _)| exception.bind(py).is(value))
                .map(|(_, text)| text.clone())
        });
        let whence = result
            .as_ref()
            .err()
            .and_then(|err| whence::of(py, err, &test));
        let seen = Seen {
            form,
            with,
            whence,
            path,
            in_another_try,
            missed,
            suppressions,
        };
        let decided = decide(py, result, mark, &readings, &seen, loaded, function);
        let replays = if decided.1 {
            readings
                .replay
                .iter()
                .filter_map(|index| {
                    let (_, held) = readings.in_order.iter().find(|(read, _)| read == index)?;
                    Some((*index, *held, loaded.expectation(*index)?.to_owned()))
                })
                .collect()
        } else {
            Vec::new()
        };
        self.replays = replays;
        self.suppressed = decided.2;
        (decided.0, landed)
    }

    /// Runs `function` of `module` again as a replay that reads only the
    /// expectation at `index`, and ends the process with its verdict. It
    /// returns only when `module` did not load or has no `function`.
    pub fn replay(&self, module: &Module, function: &str, index: usize) {
        let py = self.py;
        let Ok(loaded) = self.module(module) else {
            return;
        };
        let Ok(test) = loaded.module().getattr(function) else {
            return;
        };
        let _ = interrupt::calling(py, || {
            loaded.replay(&reader::READERS, index, || test.call0())
        });
    }

    /// Each warning of the last case: a fault of a check that a suppression
    /// reached.
    #[must_use]
    pub fn warnings(&self) -> &[Warning] {
        &self.suppressed.warnings
    }

    /// Whether execution read the paths of the last case, and so judged each
    /// suppression of a check that it sees.
    #[must_use]
    pub const fn judged(&self) -> bool {
        self.suppressed.judged
    }

    /// Each later expectation of the last case that a replay must read alone:
    /// its index, whether it held when read in order, and its text.
    #[must_use]
    pub fn replays(&self) -> &[(usize, bool, String)] {
        &self.replays
    }
}

/// What the run of a case saw besides its result, its mark and its readings.
/// It holds the first fault of each check of the form, each beside the name of
/// its check. It holds the text of a `with` whose entry or cleanup raised what
/// the call raised, and the frame that the call's raise came from. It holds
/// the faults of the paths, each beside the name of its check, and whether one
/// lies in a `try` of another shape than one path.
/// It holds the words of each missed `try`. It also holds each check that a
/// suppression of the test reaches, beside its reason.
struct Seen<'a> {
    form: Vec<(&'static str, String)>,
    with: Option<String>,
    whence: Option<String>,
    path: Vec<(&'static str, String)>,
    in_another_try: bool,
    missed: Vec<String>,
    suppressions: &'a [(&'a str, &'a str)],
}

/// The checks that execution sees, of those that a suppression turns into a
/// warning.
const SUPPRESSIBLE: [&str; 4] = [
    "second-path",
    "assert-before-expectation",
    "swallowed-raise",
    "iterator-read",
];

/// A violation of a check that execution sees, which a suppression turned
/// into a warning: the name of the check, its words, and the reason of the
/// suppression.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Warning {
    pub check: String,
    pub words: String,
    pub reason: String,
}

/// What the suppressions of a case gave: each warning, and whether execution
/// read the paths of the case, and so judged each suppression of its checks.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Suppressed {
    pub warnings: Vec<Warning>,
    pub judged: bool,
}

/// The suppressions of a case, and the warnings that they gave.
struct Suppressing<'a> {
    suppressions: &'a [(&'a str, &'a str)],
    warnings: Vec<Warning>,
    judged: bool,
}

impl Suppressing<'_> {
    /// Gives `words`, a fault of `check`, back as a clause of a project error,
    /// or turns it into a warning when a suppression reaches `check`.
    fn clause(&mut self, check: &str, words: &str) -> Option<String> {
        let Some((_, reason)) = self
            .suppressions
            .iter()
            .find(|(named, _)| *named == check && SUPPRESSIBLE.contains(named))
        else {
            return Some(format!("[{check}] {words}"));
        };
        match self
            .warnings
            .iter_mut()
            .find(|warning| warning.check == check)
        {
            Some(warning) => {
                warning.words.push_str("; ");
                warning.words.push_str(words);
            }
            None => self.warnings.push(Warning {
                check: check.to_owned(),
                words: words.to_owned(),
                reason: (*reason).to_owned(),
            }),
        }
        None
    }
}

/// The shape that a fault of a line of a `try` of another shape names.
const ONE_PATH: &str =
    "a try of one path holds one statement and one except SomeError:, with no else and no finally";

/// The faults of the paths of a case, in the order of their lines, beside the
/// name of each check: each `with` that swallowed a raise and skipped a line,
/// and each fault of `path` that no such `with` explains. A fault of a line of
/// the test in the `except` of a `try` of one path of `tries` is no fault. It
/// also says whether a fault lies on a line of the test in another `try`.
fn path_faults(
    path: Vec<paths::Fault>,
    swallowed: &[Swallowed],
    tries: &[Try],
) -> (Vec<(&'static str, String)>, bool) {
    let mut faults: Vec<(usize, _, String, &'static str)> = path
        .into_iter()
        .filter(|(line, reached, _)| {
            reached.is_some()
                || !swallowed
                    .iter()
                    .any(|swallow| (swallow.skipped..=swallow.last).contains(line))
                    && !tries.iter().any(|attempt| {
                        attempt
                            .except
                            .as_ref()
                            .is_some_and(|except| except.contains(line))
                    })
        })
        .map(|(line, reached, fault)| (line, reached, fault, "second-path"))
        .chain(swallowed.iter().map(|swallow| {
            (
                swallow.with,
                None,
                format!(
                    "the with at line {} swallowed a raise, and line {} did not run",
                    swallow.with, swallow.skipped
                ),
                "swallowed-raise",
            )
        }))
        .collect();
    faults.sort();
    faults.dedup();
    let in_another_try = faults.iter().any(|(line, reached, _, _)| {
        reached.is_none()
            && tries
                .iter()
                .any(|attempt| attempt.except.is_none() && attempt.lines.contains(line))
    });
    (
        faults
            .into_iter()
            .map(|(_, _, fault, check)| (check, fault))
            .collect(),
        in_another_try,
    )
}

/// The line of two readings of the expectation `text` that differ, or of a
/// reading alone that an assert ran inside, with the name of its check, or
/// `None` when the two readings agree.
fn differ_line(text: &str, alone: Verdict, in_order: bool) -> Option<String> {
    let check = if alone == Verdict::Refused {
        "assert-in-reading"
    } else {
        "readings-differ"
    };
    differs(alone, in_order).map(|differ| format!("[{check}] {text} {differ}"))
}

/// The words of a reading alone that an assert ran inside.
const REFUSED_ALONE: &str = "reaches an assert when read alone, and only an expectation asserts";

/// The words of two readings of one expectation that differ, or of a reading
/// alone that an assert ran inside, or `None` when the two readings agree.
const fn differs(alone: Verdict, in_order: bool) -> Option<&'static str> {
    Some(match (alone, in_order) {
        (Verdict::Holds, false) => {
            "holds when read alone, and not after the expectations before it"
        }
        (Verdict::Fails, true) => "holds after the expectations before it, and not when read alone",
        (Verdict::Raises, true) => {
            "holds after the expectations before it, and raises when read alone"
        }
        (Verdict::Raises, false) => {
            "does not hold after the expectations before it, and raises when read alone"
        }
        (Verdict::Refused, _) => REFUSED_ALONE,
        (Verdict::Holds, true) | (Verdict::Fails, false) => return None,
    })
}

/// The outcome of the case of `function`, and whether its expectations gave
/// it. It also gives each warning of the case. It reads what the call gave,
/// the last marked assert, the readings of the expectations, and what the run
/// saw. The first of these wins:
///
/// 1. a project error for the first fault of each check of the form in turn,
///    then for a reading alone that an assert ran inside;
/// 2. `raised` when the call raised, unless a `with`'s entry or cleanup raised
///    it, or a marked assert that did not hold raised `AssertionError`, which
///    are `failed`;
/// 3. a project error for the faults of the paths, joined by `; `;
/// 4. a project error when a fork to read a later expectation alone failed, or
///    when the two readings of one expectation differ;
/// 5. `failed` when a `try` of one path missed, or an expectation did not
///    hold;
/// 6. a project error when no expectation ran;
/// 7. `passed`.
///
/// A fault of a check that a suppression of the test reaches gives a warning in
/// place of its project error, and the case goes on to the next of these. Each
/// project error of a strict check names its check in brackets before the
/// words of each fault. Execution judges each suppression of a check that it
/// sees only when `decide` reaches step 3.
fn decide(
    py: Python<'_>,
    result: Result<(), PyErr>,
    mark: Option<(&str, bool)>,
    readings: &Readings,
    seen: &Seen<'_>,
    loaded: &Loaded<'_>,
    function: &str,
) -> (Result<Kind, Error>, bool, Suppressed) {
    let mut suppressing = Suppressing {
        suppressions: seen.suppressions,
        warnings: Vec::new(),
        judged: false,
    };
    let (own, decided) = 'decide: {
        for (check, words) in &seen.form {
            if let Some(clause) = suppressing.clause(check, words) {
                break 'decide (Err(Error::project(clause)), false);
            }
        }
        if let Some((index, _)) = readings
            .alone
            .iter()
            .find(|(_, verdict)| *verdict == Verdict::Refused)
        {
            let text = loaded.expectation(*index).unwrap_or_default();
            break 'decide (
                Err(Error::project(format!(
                    "[assert-in-reading] {text} {REFUSED_ALONE}"
                ))),
                false,
            );
        }
        if let Err(err) = result {
            if let Some(with) = &seen.with {
                break 'decide (
                    Ok(Kind::Failed {
                        asserts: vec![Quote::alone(with.clone())],
                    }),
                    false,
                );
            }
            if let Some((assert, false)) = mark
                && err.is_instance_of::<PyAssertionError>(py)
            {
                break 'decide (
                    Ok(Kind::Failed {
                        asserts: vec![Quote::alone(assert.to_owned())],
                    }),
                    false,
                );
            }
            break 'decide (
                Ok(Kind::Raised {
                    exception: err.to_string(),
                    whence: seen.whence.clone(),
                }),
                false,
            );
        }
        suppressing.judged = true;
        let path: Vec<String> = seen
            .path
            .iter()
            .filter_map(|(check, words)| suppressing.clause(check, words))
            .collect();
        if !path.is_empty() {
            let mut line = path.join("; ");
            if seen.in_another_try {
                line.push_str("; ");
                line.push_str(ONE_PATH);
            }
            break 'decide (Err(Error::project(line)), false);
        }
        if let Some(refused) = &readings.refused {
            break 'decide (Err(Error::project(refused.clone())), false);
        }
        for &(index, alone) in &readings.alone {
            let Some(&(_, in_order)) = readings.in_order.iter().find(|(read, _)| *read == index)
            else {
                continue;
            };
            let text = loaded.expectation(index).unwrap_or_default();
            if let Some(line) = differ_line(text, alone, in_order) {
                break 'decide (Err(Error::project(line)), false);
            }
        }
        of_the_readings(readings, &seen.missed, loaded, function)
    };
    (
        own,
        decided,
        Suppressed {
            warnings: suppressing.warnings,
            judged: suppressing.judged,
        },
    )
}

/// The outcome of the case of `function` from the readings of its expectations
/// in order and each `try` of one path that `missed`, and whether they gave it:
/// `failed`, with each missed `try` and then each expectation that did not
/// hold, a project error when no expectation ran, or else `passed`.
fn of_the_readings(
    readings: &Readings,
    missed: &[String],
    loaded: &Loaded<'_>,
    function: &str,
) -> (Result<Kind, Error>, bool) {
    let failed: Vec<Quote> = missed
        .iter()
        .map(|words| Quote::alone(words.clone()))
        .chain(
            readings
                .in_order
                .iter()
                .filter(|(_, held)| !held)
                .filter_map(|(index, _)| {
                    Some(Quote {
                        assert: loaded.expectation(*index)?.to_owned(),
                        shown: readings.shown.iter().find(|(shown, _)| shown == index).map(
                            |(_, shown)| match shown.clone() {
                                purba_assertion::Shown::Values { expected, reading } => {
                                    Shown::Values { expected, reading }
                                }
                                purba_assertion::Shown::Items { heading, items } => {
                                    Shown::Items { heading, items }
                                }
                            },
                        ),
                    })
                }),
        )
        .collect();
    if !failed.is_empty() {
        (Ok(Kind::Failed { asserts: failed }), true)
    } else if !readings.in_order.is_empty() {
        (Ok(Kind::Passed), true)
    } else if !loaded.states_expectation(function) {
        (
            Err(Error::project(
                "[no-expectation] no expectation stated".to_owned(),
            )),
            false,
        )
    } else {
        (
            Err(Error::project(
                "[no-expectation] no expectation ran".to_owned(),
            )),
            false,
        )
    }
}

/// How many cases of a run passed, failed, raised, are `interrupted`, are
/// blocked or unreached, and how many errors of each kind it met.
///
/// Each case counts once, and each error once, by its kind. It also counts each
/// verdict that a suppression turned into a warning.
#[derive(Debug, Default, PartialEq, Eq)]
pub struct Tally {
    pub passed: usize,
    pub failed: usize,
    pub raised: usize,
    pub interrupted: usize,
    pub blocked: usize,
    /// Cases that the run never reached, because an interrupt stopped the run.
    pub unreached: usize,
    pub project_errors: usize,
    pub internal_errors: usize,
    /// Writes that stdout refused. The run stops at the first.
    pub invocation_errors: usize,
    /// Verdicts that a suppression turned into a warning: one for each check
    /// of each test.
    pub suppressed: usize,
}

impl Tally {
    /// Counts one test by its kind. An error in place of an outcome blocked
    /// that test, so it counts one blocked test and one error of its kind.
    pub fn add(&mut self, result: Result<&Kind, &Error>) {
        match result {
            Ok(Kind::Passed) => self.passed += 1,
            Ok(Kind::Failed { .. }) => self.failed += 1,
            Ok(Kind::Raised { .. }) => self.raised += 1,
            Ok(Kind::Interrupted) => self.interrupted += 1,
            Err(error) => {
                self.blocked += 1;
                self.add_error(error);
            }
        }
    }

    /// Counts one test that the error of its module blocked.
    pub const fn add_blocked(&mut self) {
        self.blocked += 1;
    }

    /// Counts one verdict that a suppression turned into a warning.
    pub const fn add_suppressed(&mut self) {
        self.suppressed += 1;
    }

    /// Counts one test that the run never reached.
    pub const fn add_unreached(&mut self) {
        self.unreached += 1;
    }

    /// Counts one write that stdout refused.
    pub const fn add_invocation_error(&mut self) {
        self.invocation_errors += 1;
    }

    /// Counts one error by its kind, and no test. The error of a module counts
    /// through it, and each test that the error blocked through
    /// [`Tally::add_blocked`]. The error of a thread counts through it alone.
    pub fn add_error(&mut self, error: &Error) {
        match error.kind() {
            ErrorKind::Project => self.project_errors += 1,
            ErrorKind::Internal { .. } => self.internal_errors += 1,
        }
    }
}

/// Whether purba's signal handler kept an interrupt.
///
/// A signal that waits runs the signal handler before this check. Before Python
/// starts, no signal handler of purba's exists, so none kept one.
#[must_use]
pub fn interrupted() -> bool {
    Python::try_attach(interrupt::kept).unwrap_or(false)
}
