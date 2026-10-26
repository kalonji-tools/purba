//! Dispatch: hands each selected test to one worker in-process, and puts its
//! outcome or error beside it.

use std::path::PathBuf;

use purba_execution::{Error, Outcome};
use purba_selection::TestId;

/// What dispatch reads of a selected test.
pub trait Test {
    /// The name of the test's function.
    fn function(&self) -> &str;
}

impl Test for TestId {
    fn function(&self) -> &str {
        Self::function(self)
    }
}

/// A module that command hands dispatch, with its selected tests.
pub struct Module<T> {
    path: PathBuf,
    text: String,
    asserts: Vec<String>,
    tests: Vec<T>,
}

impl<T> Module<T> {
    /// Makes a module from `path` as the tester typed it, the rewritten text,
    /// the source text of each assert at its index, and its selected tests.
    #[must_use]
    pub const fn new(path: PathBuf, text: String, asserts: Vec<String>, tests: Vec<T>) -> Self {
        Self {
            path,
            text,
            asserts,
            tests,
        }
    }
}

/// Hands each test of `modules` to one worker, in order, and puts each outcome
/// beside its test, or the error that it gave in place of an outcome.
#[must_use]
pub fn hand<T: Test>(modules: Vec<Module<T>>) -> Vec<(T, Result<Outcome, Error>)> {
    purba_execution::attach(|worker| {
        let mut results = Vec::new();
        for module in modules {
            let executable = purba_execution::Module::new(module.path, module.text, module.asserts);
            for test in module.tests {
                // PROTOTYPE (#657), never merge: a panic in one test stays in that test.
                let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                    worker.run(&executable, test.function())
                }))
                .unwrap_or_else(|_| Err(purba_execution::Error::internal("internal error: a panic during the test".to_owned())));
                results.push((test, result));
            }
        }
        results
    })
}
