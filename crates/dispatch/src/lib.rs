//! Dispatch: loads each module on one worker in-process, and hands it each
//! selected test. It hands on what the worker returns, and the error of a module
//! that did not load or of a thread. It also hands on each test that an
//! interrupt left unreached.

use std::ops::ControlFlow;
use std::path::PathBuf;

use purba_execution::{Error, Outcome, Owner, Rewrite, Worker};
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
    rewrite: Rewrite,
    tests: Vec<T>,
}

impl<T> Module<T> {
    /// Makes a module from `path` as the tester typed it, its rewrite, and its
    /// selected tests.
    #[must_use]
    pub const fn new(path: PathBuf, rewrite: Rewrite, tests: Vec<T>) -> Self {
        Self {
            path,
            rewrite,
            tests,
        }
    }
}

/// What dispatch hands on: a test beside its outcome or its own error, the
/// error of a module that did not load, a test of that module, or the error
/// of a thread.
pub enum Handed<T> {
    /// A test that ran, beside its outcome, or the error that it gave in
    /// place of one.
    Test(T, Result<Outcome, Error>),
    /// A module that did not load, by its path, and the load's error, or the
    /// internal error of a panic during the load.
    NotLoaded(PathBuf, Error),
    /// A test that no call reached, because its module, by its path, did not
    /// load.
    Blocked(T, PathBuf),
    /// The error of a thread's exception, beside the test or the module that
    /// it charges. It blocks no test.
    Thread(Charged<T>, Error),
    /// A test that the run never reached, because an interrupt stopped the run.
    Unreached(T),
}

/// What the error of a thread charges: a test, or a module by its path.
pub enum Charged<T> {
    Test(T),
    Module(PathBuf),
}

/// Loads each module of `modules` that has a test on one worker, hands each
/// of its tests to it, in order, and calls `each` with what the worker returns.
///
/// When a module does not load, `each` takes its error once, then each of its
/// tests as blocked, and no test of it runs. Once `each` breaks, nothing
/// further runs, and `hand` returns the break.
///
/// After each load and each test, `each` takes the error of each thread that
/// the worker took, beside the test or the module that it charges. After a
/// load, they come before the first test or the first blocked test.
///
/// Before each load and each test, dispatch asks execution whether purba's
/// signal handler kept an interrupt. Once it did, `each` takes each test left,
/// of each module left, as unreached. Nothing further loads or runs. When the
/// interrupt lands in a load, `each` takes no project error of the load.
///
/// A panic that crosses the load of a module or the run of a test gives that
/// module or that test an internal error.
pub fn hand<T: Test + Clone, B>(
    modules: Vec<Module<T>>,
    mut each: impl FnMut(Handed<T>) -> ControlFlow<B>,
) -> ControlFlow<B> {
    purba_execution::attach(|worker| {
        for module in modules {
            if module.tests.is_empty() {
                continue;
            }
            if purba_execution::interrupted() {
                for test in module.tests {
                    each(Handed::Unreached(test))?;
                }
                continue;
            }
            let path = module.path.clone();
            let executable = purba_execution::Module::new(module.path, module.rewrite);
            let (loaded, threads) = taken(worker, |worker| worker.load(&executable));
            if let Err(error) = loaded {
                each(Handed::NotLoaded(path.clone(), error))?;
                for (owner, error) in threads {
                    each(Handed::Thread(charged(owner), error))?;
                }
                for test in module.tests {
                    each(Handed::Blocked(test, path.clone()))?;
                }
                continue;
            }
            for (owner, error) in threads {
                each(Handed::Thread(charged(owner), error))?;
            }
            for test in module.tests {
                if purba_execution::interrupted() {
                    each(Handed::Unreached(test))?;
                    continue;
                }
                let (result, threads) = taken(worker, |worker| {
                    worker.run(&executable, test.function(), test.clone())
                });
                each(Handed::Test(test, result))?;
                for (owner, error) in threads {
                    each(Handed::Thread(charged(owner), error))?;
                }
            }
        }
        ControlFlow::Continue(())
    })
}

/// The error of each thread that the worker took, beside its owner.
type Threads<T> = Vec<(Owner<T>, Error)>;

/// Runs `work`, then takes the errors of threads, inside one `catch`. A panic
/// gives the internal error of `work`, and takes no error of a thread.
fn taken<T: Clone, R>(
    worker: &mut Worker<'_, T>,
    work: impl FnOnce(&mut Worker<'_, T>) -> Result<R, Error>,
) -> (Result<R, Error>, Threads<T>) {
    purba_execution::catch("dispatch", || {
        let result = work(worker);
        Ok((result, worker.threads()))
    })
    .unwrap_or_else(|error| (Err(error), Vec::new()))
}

/// The test or the module that `owner` names.
fn charged<T>(owner: Owner<T>) -> Charged<T> {
    match owner {
        Owner::Test(test) => Charged::Test(test),
        Owner::Load(path) => Charged::Module(path),
    }
}
