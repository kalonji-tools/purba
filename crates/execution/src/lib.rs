//! Execution: loads a module, runs a test, and makes its outcome. It keeps the
//! first interrupt of the process.

#[cfg(not(unix))]
compile_error!("purba runs on Linux and macOS. On Windows, run it under WSL2.");

pub mod interrupt;
pub mod panic;
mod step;
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
use step::{Gives, Met};

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
    /// Whether the test passed, failed, raised, or is `unknown` or
    /// `interrupted`.
    #[must_use]
    pub const fn kind(&self) -> &Kind {
        &self.kind
    }
}

/// Passed, failed, raised, unknown or interrupted.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Kind {
    /// An assert ran, and the test returned.
    Passed,
    /// An assert that ran did not hold, and the test raised `AssertionError`.
    /// `assert` is the text of the last assert that did not hold.
    Failed { assert: String },
    /// The test's call raised, and the outcome is not `Failed` or
    /// `Interrupted`. A call to `os._exit` during the call gives `Raised` too,
    /// even when the test caught its error. `exception` is the type and the
    /// message of the exception, as text.
    Raised { exception: String },
    /// No assert ran, and the test returned.
    Unknown,
    /// The first interrupt landed during the test's step: its lookup or its
    /// call. The test has this outcome whether it then passed, failed, raised or
    /// called `os._exit`.
    Interrupted,
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
    guard: &'static Py<Guard>,
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

/// Gives `work` a worker for as long as it runs.
///
/// Before the first worker, it guards `os._exit`, and hooks
/// `threading.excepthook` and `sys.unraisablehook`, for the life of the
/// process. So each run imports `threading`.
///
/// # Panics
///
/// The guard could not take the place of `os._exit`, or the hook the place of
/// `threading.excepthook` or `sys.unraisablehook`.
pub fn attach<T, R>(work: impl for<'py> FnOnce(&mut Worker<'py, T>) -> R) -> R {
    Python::initialize();
    Python::attach(|py| {
        work(&mut Worker {
            py,
            loaded: HashMap::new(),
            guard: guard(py).expect("the guard takes the place of os._exit"),
            hook: thread::hook(py)
                .expect("the hook takes the place of threading.excepthook and sys.unraisablehook"),
            owners: Vec::new(),
            last: None,
        })
    })
}

impl<'py, T: Clone> Worker<'py, T> {
    /// Calls `function` of `module`, which `load` loaded, and makes its
    /// outcome. When the first interrupt landed in the step, the outcome is
    /// `Interrupted`, and when it landed before the test's call, the call does
    /// not run. When the guard recorded a call to `os._exit` since the worker
    /// last took one, the outcome is `Raised`, and names that call to
    /// `os._exit`, unless the step gave a project error of its own. Each thread
    /// that the step leaves alive gets `test` as its owner, unless it has one.
    ///
    /// # Errors
    ///
    /// An internal error when `module` was never given to `load`, when a panic
    /// crossed the test's call, or when the test caught a panic and the panic
    /// hook noted it. A project error when the module did not load, it has no
    /// `function`, or `function` is not callable.
    pub fn run(&mut self, module: &Module, function: &str, test: T) -> Result<Outcome, Error> {
        let window = self.open();
        let (own, interrupted) = self.call(&window, module, function);
        let met = self.close(&window, Owner::Test(test), interrupted);
        step::decide(own, met).map(|gives| Outcome {
            kind: match gives {
                Gives::Own(kind) => kind,
                Gives::Interrupted => Kind::Interrupted,
                Gives::Exit(call) => Kind::Raised {
                    exception: format!("ExitInsideATest: {call}"),
                },
            },
        })
    }

    /// Loads `module` once, and keeps it, or keeps that its load failed. The
    /// load takes the call to `os._exit` that the guard recorded and the note
    /// of a panic, so no later test takes them. Each thread that the load
    /// leaves alive gets the load as its owner. When the first interrupt
    /// landed in the load, it gives no project error, and keeps no module.
    ///
    /// # Errors
    ///
    /// The load raised, or it called `os._exit`, even when the module caught
    /// the call. An internal error when the module caught a panic and the
    /// panic hook noted it. A later call names the module that did not load.
    pub fn load(&mut self, module: &Module) -> Result<(), Error> {
        if self.loaded.contains_key(&module.path) {
            return self.module(module).map(|_| ());
        }
        let window = self.open();
        let py = self.py;
        let (own, interrupted) = if window.landed() {
            (Ok(None), true)
        } else {
            let (loaded, landed) = interrupt::calling(py, || {
                purba_assertion::load(py, &module.path, &module.rewrite)
            });
            (loaded.map(Some).map_err(|err| error(&err)), landed)
        };
        let met = self.close(&window, Owner::Load(module.path.clone()), interrupted);
        let (kept, result) = match step::decide(own, met) {
            Ok(Gives::Own(loaded)) => (loaded, Ok(())),
            Ok(Gives::Interrupted) => (None, Ok(())),
            Ok(Gives::Exit(call)) => (
                None,
                Err(Error::project(format!("ExitInsideATest: {call}"))),
            ),
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
    /// that the step left alive, and takes the call to `os._exit` and the note
    /// of a panic.
    fn close(&mut self, window: &Window, owner: Owner<T>, interrupted: bool) -> Met {
        self.own(&window.alive, owner);
        let exit = self.guard.get().take();
        Met {
            panic: panic::take(),
            interrupted,
            exit,
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
    /// landed since `window` opened. Also says whether the first interrupt
    /// landed in the step.
    fn call(
        &self,
        window: &Window,
        module: &Module,
        function: &str,
    ) -> (Result<Kind, Error>, bool) {
        let py = self.py;
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
        let mut landed = false;
        let (result, last) = loaded.watch(|| {
            let (called, inside) = interrupt::calling(py, || {
                std::panic::catch_unwind(AssertUnwindSafe(|| test.call0()))
            });
            landed = inside;
            Ok(called)
        });
        let result = match result {
            Ok(Ok(called)) => called,
            Ok(Err(payload)) => {
                return (Err(Error::caught(payload.as_ref(), "execution")), landed);
            }
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
        (Ok(kind), landed)
    }
}

/// How many tests of a run passed, failed, raised, are `unknown` or
/// `interrupted`, are blocked or unreached, and how many errors of each kind
/// it met.
///
/// Each test counts once, and each error once, by its kind.
#[derive(Debug, Default, PartialEq, Eq)]
pub struct Tally {
    pub passed: usize,
    pub failed: usize,
    pub raised: usize,
    pub unknown: usize,
    pub interrupted: usize,
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
