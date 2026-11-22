//! Execution: loads a module, runs a test, and makes its outcome. It keeps the
//! first interrupt of the process.

#[cfg(not(any(target_os = "linux", target_os = "macos")))]
compile_error!("purba runs on Linux and macOS. On Windows, run it under WSL2.");

mod fork;
mod frame;
mod interrupt;
mod order;
pub mod panic;
mod paths;
mod step;
mod thread;
mod whence;
mod zygote;

use std::any::Any;
use std::fmt;
use std::panic::AssertUnwindSafe;
use std::path::PathBuf;

use order::{Called, Facts};
/// Assertion's rewrite of a module, which a [`Module`] holds.
pub use purba_assertion::Rewrite;
/// What an expectation that did not hold shows, which a [`Quote`] holds.
pub use purba_assertion::Shown;
use purba_assertion::{Loaded, Readings, Watched};
use pyo3::exceptions::PyAssertionError;
use pyo3::types::PyAnyMethods;
use pyo3::{Bound, Py, PyAny, PyErr, Python};
use serde::{Deserialize, Serialize};
use step::{Gives, Met};
pub use zygote::{End, Stop, Zygote};

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

    /// The line of this error: `project error: <text>`, or
    /// `internal error: <part>: <message>`. It keeps each break of the text.
    #[must_use]
    pub fn line(&self) -> String {
        self.line_with(str::to_owned)
    }

    /// The line of this error, with its text or its message as `text` gives
    /// it.
    #[must_use]
    pub fn line_with(&self, text: impl FnOnce(&str) -> String) -> String {
        match &self.0 {
            Reason::Project(own) => format!("project error: {}", text(own)),
            Reason::Internal(caught) => {
                format!("internal error: {}: {}", caught.part, text(&caught.message))
            }
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

/// Names what a case replaced since the filters were last held: a hook of the
/// process or the warning filters, as the hook names them, or else the events
/// of the watch of the paths. Gives `None` when it replaced none.
fn replaced(py: Python<'_>) -> Option<purba_assertion::Replaced> {
    thread::replaced(py).or_else(paths::replaced)
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
/// caller handed to [`Zygote::run`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Owner<T> {
    Load(PathBuf),
    Test(T),
}

/// Runs tests in this process.
///
/// It loads at most one module, the one of its process, and keeps it and its
/// path, or keeps only its path when its load failed. It keeps the owner of
/// each thread that a load or a test left alive.
pub struct Worker<'py, T> {
    py: Python<'py>,
    loaded: Option<Loaded<'py>>,
    path: Option<PathBuf>,
    hook: &'static Py<thread::Hook>,
    owners: Vec<(Py<PyAny>, Owner<T>)>,
    last: Option<Owner<T>>,
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
            loaded: None,
            path: None,
            hook: thread::hook(py)
                .expect("the hook takes the place of threading.excepthook and sys.unraisablehook"),
            owners: Vec::new(),
            last: None,
        }
    }
}

impl<'py, T: Clone> Worker<'py, T> {
    /// Calls `function` of the module that `load` loaded, and gathers the
    /// facts of its case, when the call ran. Also gives what the step gives
    /// besides them: `Interrupted` when the first interrupt landed in the
    /// step, and when it landed before the test's call, the call does not
    /// run. Each thread that the step leaves alive gets `test` as its owner,
    /// unless it has one.
    fn gather(&mut self, function: &str, test: T) -> (Option<Facts>, Result<Gives<()>, Error>) {
        let window = self.open();
        let (own, interrupted) = self.call(&window, function);
        let met = self.close(&window, Owner::Test(test), interrupted);
        let (facts, own) = match own {
            Ok(facts) => (facts, Ok(())),
            Err(error) => (None, Err(error)),
        };
        (facts, step::decide(own, met))
    }

    /// Loads `module`, the one module of its process, and keeps it, or keeps
    /// no module when its load fails. The load takes the note of a panic, so
    /// no later test takes it. Each thread that the load leaves alive gets the
    /// load as its owner. When the first interrupt landed in the load, it gives
    /// no project error, and keeps no module.
    ///
    /// # Errors
    ///
    /// The load raised. An internal error when the module caught a panic and the
    /// panic hook noted it. A later call names the module that did not load.
    pub(crate) fn load(&mut self, module: &Module) -> Result<(), Error> {
        self.path = Some(module.path.clone());
        let window = self.open();
        let py = self.py;
        let (own, interrupted) = if window.landed() {
            (Ok(None), true)
        } else {
            thread::hold_filters(py);
            let (own, landed) = interrupt::calling(py, || {
                paths::load(py, || {
                    let loaded = purba_assertion::load(py, &module.path, &module.rewrite)
                        .map_err(|err| error(&err))?;
                    if let Some(replaced) = thread::replaced(py) {
                        return Err(Error::project(format!(
                            "[hook-replaced] the module replaced {} as it loaded, and {}",
                            replaced.what(),
                            replaced.rule()
                        )));
                    }
                    Ok(loaded)
                })
            });
            (own.map(Some), landed)
        };
        let met = self.close(&window, Owner::Load(module.path.clone()), interrupted);
        let (kept, result) = match step::decide(own, met) {
            Ok(Gives::Own(loaded)) => (loaded, Ok(())),
            Ok(Gives::Interrupted) => (None, Ok(())),
            Err(error) => (None, Err(error)),
        };
        self.loaded = kept;
        result
    }

    /// The module that `load` loaded.
    fn module(&self) -> Result<&Loaded<'py>, Error> {
        self.loaded.as_ref().ok_or_else(|| {
            let path = self
                .path
                .as_deref()
                .unwrap_or_else(|| "the module".as_ref());
            Error::project(format!("{} did not load", path.display()))
        })
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
    pub(crate) fn threads(&self) -> Vec<(Owner<T>, Error)> {
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

    /// Looks `function` up in the module that `load` loaded. Calls it, unless
    /// the first interrupt landed since `window` opened, and watches the paths
    /// of the call. Also says whether the first interrupt landed in the step.
    /// Gives the facts of the case, or `None` when the first interrupt landed
    /// before the call. When a thread was alive at the first expectation, the
    /// facts hold each later expectation for a replay.
    fn call(&self, window: &Window, function: &str) -> (Result<Option<Facts>, Error>, bool) {
        let py = self.py;
        let loaded = match self.module() {
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
            return (Ok(None), true);
        }
        let before = loaded.first_expectation(function).unwrap_or(usize::MAX);
        thread::hold_filters(py);
        let watch = paths::Watch::start(py, test.getattr("__code__").ok().as_ref(), before);
        let mut landed = false;
        let watched = loaded.watch(&zygote::READERS, || {
            let (called, inside) = interrupt::calling(py, || {
                std::panic::catch_unwind(AssertUnwindSafe(|| test.call0()))
            });
            landed = inside;
            Ok(called)
        });
        let path = watch.finish(py);
        let Watched {
            result,
            mark,
            readings,
            form,
            raised,
            swallowed,
            missed,
        } = watched;
        let result = match result {
            Ok(Ok(called)) => called.map(drop),
            Ok(Err(payload)) => {
                return (Err(Error::caught(payload.as_ref(), "execution")), landed);
            }
            Err(err) => Err(err),
        };
        let (called, with) = match result {
            Ok(()) => (Called::Returned, None),
            Err(err) => raised_by(py, &err, &raised, &test),
        };
        let Readings {
            in_order,
            shown,
            alone,
            refused,
            replay,
        } = readings;
        let texts = texts(
            loaded,
            in_order
                .iter()
                .map(|(index, _)| *index)
                .chain(alone.iter().map(|(index, _)| *index))
                .chain(replay.iter().copied()),
        );
        let owned = |faults: Vec<(&'static str, String)>| {
            faults
                .into_iter()
                .map(|(check, words)| (check.to_owned(), words))
                .collect()
        };
        let facts = Facts {
            form: owned(form),
            called,
            with,
            mark: mark.map(|(assert, held)| (assert.to_owned(), held)),
            path,
            swallowed,
            tries: loaded.tries(function).to_vec(),
            refused,
            in_order,
            shown,
            alone,
            later: replay,
            texts,
            missed,
            stated: loaded.states_expectation(function),
        };
        (Ok(Some(facts)), landed)
    }

    /// Runs `function` of the module that `load` loaded again, as a replay
    /// that reads only the expectation at `index`. At that expectation,
    /// `REPLAYS` sends the verdict and ends the process. It returns when the
    /// call returned before it reached that expectation.
    ///
    /// # Errors
    ///
    /// The module did not load, or has no `function`.
    pub(crate) fn replay(&self, function: &str, index: usize) -> Result<(), Error> {
        let py = self.py;
        let loaded = self.module()?;
        let test = loaded
            .module()
            .getattr(function)
            .map_err(|err| error(&err))?;
        let _ = interrupt::calling(py, || {
            loaded.replay(&zygote::REPLAYS, index, || test.call0())
        });
        Ok(())
    }
}

/// What the call of `test` gave when it raised `err`, and the text of the
/// `with` whose entry or cleanup raised `err`, when one of `raised` did.
fn raised_by(
    py: Python<'_>,
    err: &PyErr,
    raised: &[(Py<PyAny>, String)],
    test: &Bound<'_, PyAny>,
) -> (Called, Option<String>) {
    let value = err.value(py);
    let with = raised
        .iter()
        .find(|(exception, _)| exception.bind(py).is(value))
        .map(|(_, text)| text.clone());
    let called = Called::Raised {
        exception: err.to_string(),
        whence: whence::of(py, err, test),
        assertion: err.is_instance_of::<PyAssertionError>(py),
    };
    (called, with)
}

/// The text of each expectation of `loaded` at one of `indexes`, once each, in
/// the order of the indexes.
fn texts(loaded: &Loaded<'_>, indexes: impl Iterator<Item = usize>) -> Vec<(usize, String)> {
    let mut indexes: Vec<usize> = indexes.collect();
    indexes.sort_unstable();
    indexes.dedup();
    indexes
        .into_iter()
        .filter_map(|index| Some((index, loaded.expectation(index)?.to_owned())))
        .collect()
}

/// The checks that execution sees, of those that a suppression turns into a
/// warning. Selection's table names the same checks, and a test of dispatch
/// compares the two.
pub const SUPPRESSIBLE: [&str; 4] = [
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

    /// How many cases the run has: each case that passed, failed, raised, is
    /// `interrupted`, is blocked or is unreached. An error and a suppression
    /// are no case.
    #[must_use]
    pub const fn cases(&self) -> usize {
        self.passed + self.failed + self.raised + self.interrupted + self.blocked + self.unreached
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

/// Sets each hook of the process that a run needs.
///
/// It wraps Rust's panic hook first, then gives each interrupt signal purba's
/// signal handler. A test of the signal handler makes this call, so it runs
/// the process that a run has.
///
/// # Errors
///
/// `signal` refused the signal handler, or `os.register_at_fork` refused the
/// function that gives a forked child back the signal handlers.
pub fn set_hooks() -> pyo3::PyResult<()> {
    panic::wrap_hook();
    interrupt::install()
}

/// Whether purba's signal handler kept an interrupt.
///
/// A signal that waits runs the signal handler before this check. Before Python
/// starts, no signal handler of purba's exists, so none kept one.
#[must_use]
pub fn interrupted() -> bool {
    Python::try_attach(interrupt::kept).unwrap_or(false)
}
