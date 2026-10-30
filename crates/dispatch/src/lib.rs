//! Dispatch: loads each module on one worker in-process, hands it each
//! selected test, and hands on what the worker returns, or the error of a
//! module that did not load.

use std::ops::ControlFlow;
use std::panic::{AssertUnwindSafe, catch_unwind};
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

/// What dispatch hands on: a test beside its outcome or its own error, the
/// error of a module that did not load, or a test of that module.
pub enum Handed<T> {
    /// A test that ran, beside its outcome, or the error that it gave in
    /// place of one.
    Test(T, Result<Outcome, Error>),
    /// A module that did not load, by its path, and the load's error.
    NotLoaded(PathBuf, Error),
    /// A test that no call reached, because its module, by its path, did not
    /// load.
    Blocked(T, PathBuf),
}

/// Loads each module of `modules` that has a test on one worker, hands each
/// of its tests to it, in order, and calls `each` with what the worker returns.
///
/// When a module does not load, `each` takes its error once, then each of its
/// tests as blocked, and no test of it runs. Once `each` breaks, nothing
/// further runs, and `hand` returns the break.
pub fn hand<T: Test, B>(
    modules: Vec<Module<T>>,
    mut each: impl FnMut(Handed<T>) -> ControlFlow<B>,
) -> ControlFlow<B> {
    purba_execution::attach(|worker| {
        for module in modules {
            if module.tests.is_empty() {
                continue;
            }
            let path = module.path.clone();
            let executable = purba_execution::Module::new(module.path, module.text, module.asserts);
            let loaded = if catches() {
                catch_unwind(AssertUnwindSafe(|| worker.load(&executable)))
                    .unwrap_or_else(|payload| Err(Error::internal("dispatch", message(&*payload))))
            } else {
                worker.load(&executable)
            };
            if let Err(error) = loaded {
                each(Handed::NotLoaded(path.clone(), error))?;
                for test in module.tests {
                    each(Handed::Blocked(test, path.clone()))?;
                }
                continue;
            }
            for test in module.tests {
                let result = if catches() {
                    catch_unwind(AssertUnwindSafe(|| {
                        worker.run(&executable, test.function())
                    }))
                    .unwrap_or_else(|payload| Err(Error::internal("dispatch", message(&*payload))))
                } else {
                    worker.run(&executable, test.function())
                };
                each(Handed::Test(test, result))?;
            }
        }
        ControlFlow::Continue(())
    })
}

// PROTOTYPE (#715), never merge: every mode but `none` catches here too.
fn catches() -> bool {
    std::env::var("PURBA_CATCH").is_ok_and(|mode| mode != "none")
}

fn message(payload: &(dyn std::any::Any + Send)) -> String {
    payload
        .downcast_ref::<String>()
        .cloned()
        .or_else(|| {
            payload
                .downcast_ref::<&str>()
                .map(|text| (*text).to_owned())
        })
        .unwrap_or_else(|| "a panic with no message".to_owned())
}
