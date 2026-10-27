//! Dispatch: hands each selected test to one worker in-process, and hands
//! the test on, beside its outcome or error, as the worker returns it.

use std::ops::ControlFlow;
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

/// Hands each test of `modules` to one worker, in order, and calls `each` as
/// the worker returns the test.
///
/// `each` takes the test and its outcome, or the error that it gave in place
/// of one. Once `each` breaks, no further test runs, and `hand` returns the
/// break.
pub fn hand<T: Test, B>(
    modules: Vec<Module<T>>,
    mut each: impl FnMut(T, Result<Outcome, Error>) -> ControlFlow<B>,
) -> ControlFlow<B> {
    purba_execution::attach(|worker| {
        for module in modules {
            let executable = purba_execution::Module::new(module.path, module.text, module.asserts);
            for test in module.tests {
                let result = worker.run(&executable, test.function());
                each(test, result)?;
            }
        }
        ControlFlow::Continue(())
    })
}
