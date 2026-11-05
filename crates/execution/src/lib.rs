//! Execution: loads a module, runs a test, and makes its outcome. It keeps the
//! first interrupt of the process.

pub mod interrupt;
pub mod panic;
mod thread;

use std::any::Any;
use std::collections::HashMap;
use std::fmt;
use std::panic::AssertUnwindSafe;
use std::path::PathBuf;
use std::sync::{Mutex, PoisonError};

use purba_assertion::Loaded;
/// Assertion's rewrite of a module, which a [`Module`] holds.
pub use purba_assertion::Rewrite;
use pyo3::exceptions::PyAssertionError;
use pyo3::sync::PyOnceLock;
use pyo3::types::PyAnyMethods;
use pyo3::{Bound, Py, PyAny, PyErr, PyResult, Python, pyclass, pymethods};
use serde::{Deserialize, Serialize};

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
    /// Whether the test passed, failed, raised or is `unknown`.
    #[must_use]
    pub const fn kind(&self) -> &Kind {
        &self.kind
    }
}

/// Passed, failed, raised or unknown.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Kind {
    /// An assert ran, and the test returned.
    Passed,
    /// The last assert that ran did not hold, and the test raised
    /// `AssertionError`. `assert` is the source text of that assert.
    Failed { assert: String },
    /// The test's call raised, and the outcome is not `Failed`. A call to
    /// `os._exit` during the call gives `Raised` too, even when the test caught
    /// its error. `exception` is the type and the message of the exception, as
    /// text.
    Raised { exception: String },
    /// No assert ran, and the test returned.
    Unknown,
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
/// charges. A load names the path of its module, and a test its path and the
/// name of its function.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Owner {
    Load(PathBuf),
    Test(PathBuf, String),
}

/// Runs tests in this process. It loads each module once, and keeps it under its
/// path, or keeps that its load failed. It keeps the owner of each thread that a
/// load or a test left alive.
pub struct Worker<'py> {
    py: Python<'py>,
    loaded: HashMap<PathBuf, Option<Loaded<'py>>>,
    guard: &'static Py<Guard>,
    hook: &'static Py<thread::Hook>,
    owners: Vec<(Py<PyAny>, Owner)>,
    last: Option<Owner>,
}

mod exit {
    pyo3::create_exception!(purba, ExitInsideATest, pyo3::exceptions::PyBaseException);
}

use exit::ExitInsideATest;

/// Takes the place of `os._exit`. In the process that installed it, it keeps its
/// first call until a worker takes it, and raises `ExitInsideATest`. In a process
/// that a test forks, it calls the real one.
#[pyclass(frozen)]
struct Guard {
    exit: Py<PyAny>,
    process: u32,
    called: Mutex<Option<String>>,
}

#[pymethods]
impl Guard {
    fn __call__(&self, status: &Bound<'_, PyAny>) -> PyResult<()> {
        if std::process::id() == self.process {
            let call = format!("os._exit({status})");
            self.called
                .lock()
                .unwrap_or_else(PoisonError::into_inner)
                .get_or_insert_with(|| call.clone());
            return Err(ExitInsideATest::new_err(call));
        }
        self.exit.call1(status.py(), (status,))?;
        Ok(())
    }
}

impl Guard {
    fn take(&self) -> Option<String> {
        self.called
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .take()
    }
}

static GUARD: PyOnceLock<Py<Guard>> = PyOnceLock::new();

fn guard(py: Python<'_>) -> PyResult<&'static Py<Guard>> {
    GUARD.get_or_try_init(py, || {
        let os = py.import("os")?;
        let guard = Py::new(
            py,
            Guard {
                exit: os.getattr("_exit")?.unbind(),
                process: std::process::id(),
                called: Mutex::new(None),
            },
        )?;
        os.setattr("_exit", &guard)?;
        Ok(guard)
    })
}

/// Gives `work` a worker for as long as it runs. Before the first worker, it
/// guards `os._exit`, and hooks `threading.excepthook`, for the life of the
/// process. So each run imports `threading`.
///
/// # Panics
///
/// The guard could not take the place of `os._exit`, or the hook the place of
/// `threading.excepthook`.
pub fn attach<R>(work: impl for<'py> FnOnce(&mut Worker<'py>) -> R) -> R {
    Python::initialize();
    Python::attach(|py| {
        work(&mut Worker {
            py,
            loaded: HashMap::new(),
            guard: guard(py).expect("the guard takes the place of os._exit"),
            hook: thread::hook(py).expect("the hook takes the place of threading.excepthook"),
            owners: Vec::new(),
            last: None,
        })
    })
}

impl<'py> Worker<'py> {
    /// Whether purba's signal handler kept an interrupt. A signal that waits
    /// runs the signal handler before this check.
    #[must_use]
    pub fn interrupted(&self) -> bool {
        interrupt::kept(self.py)
    }

    /// Calls `function` of `module`, and makes its outcome. When the guard
    /// recorded a call to `os._exit` since the worker last took one, the
    /// outcome is `Raised`, and names that call to `os._exit`. An error before
    /// the test's call stays as it is. Each thread that the call leaves alive
    /// gets the test as its owner, unless it has one.
    ///
    /// # Errors
    ///
    /// The module did not load, it has no `function`, or `function` is not
    /// callable. An internal error when a panic crossed the test's call, or
    /// when the test caught a panic and the panic hook noted it.
    pub fn run(&mut self, module: &Module, function: &str) -> Result<Outcome, Error> {
        let before = self.hook.get().alive(self.py);
        let result = self.call(module, function);
        self.own(
            &before,
            Owner::Test(module.path.clone(), function.to_owned()),
        );
        let exit = self.guard.get().take();
        if let Some(noted) = panic::take() {
            return Err(Error(Reason::Internal(noted.caught_by("execution"))));
        }
        match (exit, result) {
            (Some(call), Ok(_)) => Ok(Outcome {
                kind: Kind::Raised {
                    exception: format!("ExitInsideATest: {call}"),
                },
            }),
            (_, result) => result,
        }
    }

    /// Loads `module` once, and keeps it, or keeps that its load failed. The
    /// load takes the call to `os._exit` that the guard recorded and the note
    /// of a panic, so no later test takes them. Each thread that the load
    /// leaves alive gets the load as its owner.
    ///
    /// # Errors
    ///
    /// The load raised, or it called `os._exit`, even when the module caught
    /// the call. An internal error when the module caught a panic and the
    /// panic hook noted it. A later call names the module that did not load.
    pub fn load(&mut self, module: &Module) -> Result<(), Error> {
        self.loaded(module).map(|_| ())
    }

    fn loaded(&mut self, module: &Module) -> Result<&Loaded<'py>, Error> {
        if !self.loaded.contains_key(&module.path) {
            let before = self.hook.get().alive(self.py);
            let py = self.py;
            let loaded =
                interrupt::calling(|| purba_assertion::load(py, &module.path, &module.rewrite));
            self.own(&before, Owner::Load(module.path.clone()));
            let exit = self.guard.get().take();
            let failed = match (panic::take(), exit, &loaded) {
                (Some(noted), _, _) => Some(Error(Reason::Internal(noted.caught_by("execution")))),
                (None, Some(call), Ok(_)) => {
                    Some(Error::project(format!("ExitInsideATest: {call}")))
                }
                (None, _, loaded) => loaded.as_ref().err().map(error),
            };
            self.loaded.insert(
                module.path.clone(),
                loaded.ok().filter(|_| failed.is_none()),
            );
            if let Some(failed) = failed {
                return Err(failed);
            }
        }
        self.loaded[&module.path]
            .as_ref()
            .ok_or_else(|| Error::project(format!("{} did not load", module.path.display())))
    }

    /// Takes each exception that a thread raised since the last take, in the
    /// order threads raised them, as a project error with its owner. A thread
    /// with no owner charges the load or the test that ran last. Before any
    /// load or test, it takes nothing.
    pub fn threads(&mut self) -> Vec<(Owner, Error)> {
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
    fn own(&mut self, before: &[Py<PyAny>], owner: Owner) {
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

    fn call(&mut self, module: &Module, function: &str) -> Result<Outcome, Error> {
        let py = self.py;
        let loaded = self.loaded(module)?;
        let test = loaded
            .module()
            .getattr(function)
            .map_err(|err| error(&err))?;
        // A call of an object that is not callable runs no code, and gives
        // CPython's own `TypeError`.
        if !test.is_callable()
            && let Err(err) = test.call0()
        {
            return Err(error(&err));
        }
        let purba_fixtures::Inputs {} = purba_fixtures::inputs(function);
        let (result, last) = loaded.watch(|| {
            Ok(interrupt::calling(|| {
                std::panic::catch_unwind(AssertUnwindSafe(|| test.call0()))
            }))
        });
        let result = match result {
            Ok(Ok(called)) => called,
            Ok(Err(payload)) => return Err(Error::caught(payload.as_ref(), "execution")),
            Err(err) => Err(err),
        };
        let kind = match (result, last) {
            (Ok(_), Some(_)) => Kind::Passed,
            (Ok(_), None) => Kind::Unknown,
            (Err(err), Some((assert, false))) if err.is_instance_of::<PyAssertionError>(py) => {
                Kind::Failed {
                    assert: assert.to_owned(),
                }
            }
            (Err(err), _) => Kind::Raised {
                exception: err.to_string(),
            },
        };
        Ok(Outcome { kind })
    }
}

/// How many tests of a run passed, failed, raised, are `unknown`, blocked or
/// unreached, and how many errors of each kind it met. Each test counts once,
/// and each error once, by its kind.
#[derive(Debug, Default, PartialEq, Eq)]
pub struct Tally {
    pub passed: usize,
    pub failed: usize,
    pub raised: usize,
    pub unknown: usize,
    pub blocked: usize,
    /// Tests that the run never reached, because an interrupt stopped the run.
    pub unreached: usize,
    pub project_errors: usize,
    pub internal_errors: usize,
    /// Writes that stdout refused. The run stops at the first.
    pub invocation_errors: usize,
}

impl Tally {
    /// Counts one test by its kind. An error in place of an outcome blocked
    /// that test, so it counts one blocked test and one error of its kind.
    pub fn add(&mut self, result: Result<&Kind, &Error>) {
        match result {
            Ok(Kind::Passed) => self.passed += 1,
            Ok(Kind::Failed { .. }) => self.failed += 1,
            Ok(Kind::Raised { .. }) => self.raised += 1,
            Ok(Kind::Unknown) => self.unknown += 1,
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

/// Whether purba's signal handler kept an interrupt, outside any worker.
///
/// A signal that waits runs the signal handler before this check. Before Python
/// starts, no signal handler of purba's exists, so none kept one.
#[must_use]
pub fn interrupted() -> bool {
    Python::try_attach(interrupt::kept).unwrap_or(false)
}
