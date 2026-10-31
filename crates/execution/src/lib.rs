//! Execution: loads a module, runs a test, and makes its outcome.

pub mod panic;

use std::collections::HashMap;
use std::fmt;
use std::path::PathBuf;
use std::sync::{Mutex, PoisonError};

use purba_assertion::Loaded;
use pyo3::exceptions::PyAssertionError;
use pyo3::sync::PyOnceLock;
use pyo3::types::PyAnyMethods;
use pyo3::{Bound, Py, PyAny, PyErr, PyResult, Python, pyclass, pymethods};
use serde::{Deserialize, Serialize};

/// A module that dispatch hands a worker.
pub struct Module {
    path: PathBuf,
    text: String,
    asserts: Vec<String>,
}

impl Module {
    /// Makes a module from `path` as the tester typed it, the rewritten text,
    /// and the source text of each assert at its index.
    #[must_use]
    pub const fn new(path: PathBuf, text: String, asserts: Vec<String>) -> Self {
        Self {
            path,
            text,
            asserts,
        }
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

/// Why `run` made no outcome, as text: Python's error, that the module did not
/// load, or that the test is not callable.
#[derive(Debug)]
pub struct Error(String);

impl fmt::Display for Error {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.0)
    }
}

impl std::error::Error for Error {}

fn error(err: &PyErr) -> Error {
    Error(err.to_string())
}

/// Runs tests in this process. It loads each module once, and keeps it under its
/// path, or keeps that its load failed.
pub struct Worker<'py> {
    py: Python<'py>,
    loaded: HashMap<PathBuf, Option<Loaded<'py>>>,
    guard: &'static Py<Guard>,
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
/// guards `os._exit` for the life of the process.
///
/// # Panics
///
/// The guard could not take the place of `os._exit`.
pub fn attach<R>(work: impl for<'py> FnOnce(&mut Worker<'py>) -> R) -> R {
    Python::initialize();
    Python::attach(|py| {
        work(&mut Worker {
            py,
            loaded: HashMap::new(),
            guard: guard(py).expect("the guard takes the place of os._exit"),
        })
    })
}

impl<'py> Worker<'py> {
    /// Calls `function` of `module`, and makes its outcome. When the guard
    /// recorded a call to `os._exit` since the last test, the outcome is
    /// `Raised`, and names that call to `os._exit`. An error before the test's
    /// call stays as it is.
    ///
    /// # Errors
    ///
    /// The module did not load, it has no `function`, or `function` is not
    /// callable.
    pub fn run(&mut self, module: &Module, function: &str) -> Result<Outcome, Error> {
        let result = self.call(module, function);
        // PROTOTYPE: #729 reads the note here, after the test's call.
        let _noted = crate::panic::take();
        match (self.guard.get().take(), result) {
            (Some(call), Ok(_)) => Ok(Outcome {
                kind: Kind::Raised {
                    exception: format!("ExitInsideATest: {call}"),
                },
            }),
            (_, result) => result,
        }
    }

    /// Loads `module` once, and keeps it, or keeps that its load failed.
    ///
    /// # Errors
    ///
    /// The load raised. A later call names the module that did not load.
    pub fn load(&mut self, module: &Module) -> Result<(), Error> {
        self.loaded(module).map(|_| ())
    }

    fn loaded(&mut self, module: &Module) -> Result<&Loaded<'py>, Error> {
        if !self.loaded.contains_key(&module.path) {
            let loaded =
                purba_assertion::load(self.py, &module.path, &module.text, module.asserts.clone());
            let failed = loaded.as_ref().err().map(error);
            self.loaded.insert(module.path.clone(), loaded.ok());
            if let Some(failed) = failed {
                return Err(failed);
            }
        }
        self.loaded[&module.path]
            .as_ref()
            .ok_or_else(|| Error(format!("{} did not load", module.path.display())))
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
        let kind = match loaded.watch(|| test.call0()) {
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

/// How many tests of a run passed, failed, raised, are `unknown`, or are
/// blocked, and how many errors the run met. Each test counts once, and each
/// error once.
#[derive(Debug, Default, PartialEq, Eq)]
pub struct Tally {
    pub passed: usize,
    pub failed: usize,
    pub raised: usize,
    pub unknown: usize,
    pub blocked: usize,
    pub errors: usize,
}

impl Tally {
    /// Counts one test by its kind. An error in place of an outcome blocked
    /// that test, so it counts one blocked test and one error.
    pub const fn add<E>(&mut self, result: &Result<&Kind, E>) {
        match result {
            Ok(Kind::Passed) => self.passed += 1,
            Ok(Kind::Failed { .. }) => self.failed += 1,
            Ok(Kind::Raised { .. }) => self.raised += 1,
            Ok(Kind::Unknown) => self.unknown += 1,
            Err(_) => {
                self.blocked += 1;
                self.errors += 1;
            }
        }
    }

    /// Counts one test that an error outside the test blocked.
    pub const fn add_blocked(&mut self) {
        self.blocked += 1;
    }

    /// Counts one error that belongs to no test.
    pub const fn add_error(&mut self) {
        self.errors += 1;
    }
}
