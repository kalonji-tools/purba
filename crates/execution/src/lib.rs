//! Execution: runs a test, and makes its outcome.

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
    /// Whether the test passed, failed or is `unknown`.
    #[must_use]
    pub const fn kind(&self) -> &Kind {
        &self.kind
    }
}

/// Passed, failed or unknown.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Kind {
    /// An assert ran, and the test returned.
    Passed,
    /// The test raised `AssertionError` after an assert ran. `assert` is the
    /// source text of the last assert that ran.
    Failed { assert: String },
    /// No assert ran, and the test returned.
    Unknown,
}

/// Why `run` made no outcome, as text: Python's error, or that the module did
/// not load.
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

impl Worker<'_> {
    /// Calls `function` of `module`, and makes its outcome.
    ///
    /// # Errors
    ///
    /// The module did not load, it has no `function`, or the call raised an
    /// exception other than `AssertionError`, or raised `AssertionError` before
    /// any assert ran. Also, the guard recorded a call to `os._exit` since the
    /// last test, whoever saw its error.
    pub fn run(&mut self, module: &Module, function: &str) -> Result<Outcome, Error> {
        let result = self.call(module, function);
        self.guard.get().take().map_or(result, |call| {
            Err(Error(format!("ExitInsideATest: {call}")))
        })
    }

    fn call(&mut self, module: &Module, function: &str) -> Result<Outcome, Error> {
        if !self.loaded.contains_key(&module.path) {
            let loaded = purba_assertion::load(self.py, &module.path, &module.text);
            let failed = loaded.as_ref().err().map(error);
            self.loaded.insert(module.path.clone(), loaded.ok());
            if let Some(failed) = failed {
                return Err(failed);
            }
        }
        let loaded = self.loaded[&module.path]
            .as_ref()
            .ok_or_else(|| Error(format!("{} did not load", module.path.display())))?;
        let _ = loaded.take();
        let test = loaded
            .module()
            .getattr(function)
            .map_err(|err| error(&err))?;
        let purba_fixtures::Inputs {} = purba_fixtures::inputs(function);
        let kind = match (test.call0(), loaded.take()) {
            (Ok(_), Some(_)) => Kind::Passed,
            (Ok(_), None) => Kind::Unknown,
            (Err(err), Some(index)) if err.is_instance_of::<PyAssertionError>(self.py) => {
                let assert = module.asserts.get(index).ok_or_else(|| error(&err))?;
                Kind::Failed {
                    assert: assert.clone(),
                }
            }
            (Err(err), _) => return Err(error(&err)),
        };
        Ok(Outcome { kind })
    }
}

/// How many tests of a run passed, failed, are `unknown`, or gave an error in
/// place of an outcome.
#[derive(Debug, Default, PartialEq, Eq)]
pub struct Tally {
    pub passed: usize,
    pub failed: usize,
    pub unknown: usize,
    pub errors: usize,
}

impl Tally {
    /// Counts one test by its kind, or as an error in place of an outcome.
    pub const fn add<E>(&mut self, result: &Result<&Kind, E>) {
        match result {
            Ok(Kind::Passed) => self.passed += 1,
            Ok(Kind::Failed { .. }) => self.failed += 1,
            Ok(Kind::Unknown) => self.unknown += 1,
            Err(_) => self.errors += 1,
        }
    }
}
