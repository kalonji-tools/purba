//! Execution: runs a test, and makes its outcome.

use std::collections::HashMap;
use std::fmt;
use std::path::PathBuf;

use purba_assertion::Loaded;
use pyo3::exceptions::PyAssertionError;
use pyo3::types::PyAnyMethods;
use pyo3::{PyErr, Python};
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
}

/// Gives `work` a worker for as long as it runs.
pub fn attach<R>(work: impl for<'py> FnOnce(&mut Worker<'py>) -> R) -> R {
    Python::initialize();
    Python::attach(|py| {
        work(&mut Worker {
            py,
            loaded: HashMap::new(),
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
    /// any assert ran.
    pub fn run(&mut self, module: &Module, function: &str) -> Result<Outcome, Error> {
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
