//! Dispatch: forks for each module a zygote that loads it once, and runs each
//! selected test in a process that the zygote forks. It hands on what each test's
//! process gives, and the error of a module that did not load or of a thread. It
//! also hands on each test that an interrupt left unreached.

use std::ops::ControlFlow;
use std::path::PathBuf;

use purba_execution::{Error, Load, Outcome, Owner, Rewrite, Worker, Zygote};
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

/// Forks a zygote for each module of `modules` that has a test, and runs each
/// of its tests, in order, in a process that the zygote forks.
///
/// The zygote loads the module once. `each` takes what each test's process
/// gives.
///
/// When a module does not load, `each` takes its error once, then each of its
/// tests as blocked, and no test of it runs. Once `each` breaks, nothing
/// further runs, and `hand` returns the break.
///
/// After each load and each test, `each` takes the error of each thread that
/// the load or the test's process took, beside the test or the module that it
/// charges. After a load, they come before the first test or the first blocked
/// test.
///
/// When a Python thread is alive after the load, purba forks a zygote that loads
/// nothing, and each test's process loads the module again before its test.
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
    purba_execution::attach(|worker: &mut Worker<'_, T>| {
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
            let started = worker.zygote(&executable, true).map(|mut zygote| {
                let load = zygote.loaded();
                (zygote, load)
            });
            let (mut zygote, load) = match started {
                Ok(started) => started,
                Err(error) => {
                    each(Handed::NotLoaded(path.clone(), error))?;
                    for test in module.tests {
                        each(Handed::Blocked(test, path.clone()))?;
                    }
                    continue;
                }
            };
            let Load {
                result,
                threads,
                threaded,
            } = load;
            if let Err(error) = result {
                zygote.end();
                each(Handed::NotLoaded(path.clone(), error))?;
                for error in threads {
                    each(Handed::Thread(Charged::Module(path.clone()), error))?;
                }
                for test in module.tests {
                    each(Handed::Blocked(test, path.clone()))?;
                }
                continue;
            }
            for error in threads {
                each(Handed::Thread(Charged::Module(path.clone()), error))?;
            }
            if threaded {
                zygote.end();
                zygote = match worker.zygote(&executable, false) {
                    Ok(mut zygote) => {
                        zygote.loaded();
                        zygote
                    }
                    Err(error) => {
                        let text = error.to_string();
                        for test in module.tests {
                            each(Handed::Test(test, Err(Error::project(text.clone()))))?;
                        }
                        continue;
                    }
                };
            }
            let flow = run(&mut zygote, module.tests, &mut each);
            zygote.end();
            flow?;
        }
        ControlFlow::Continue(())
    })
}

/// Runs each of `tests` in a process that `zygote` forks, and hands `each` what
/// it gives.
fn run<T: Test + Clone, B>(
    zygote: &mut Zygote,
    tests: Vec<T>,
    each: &mut impl FnMut(Handed<T>) -> ControlFlow<B>,
) -> ControlFlow<B> {
    for test in tests {
        if purba_execution::interrupted() {
            each(Handed::Unreached(test))?;
            continue;
        }
        let (result, threads) = zygote.run(test.function(), test.clone());
        each(Handed::Test(test, result))?;
        for (owner, error) in threads {
            each(Handed::Thread(charged(owner), error))?;
        }
    }
    ControlFlow::Continue(())
}

/// The test or the module that `owner` names.
fn charged<T>(owner: Owner<T>) -> Charged<T> {
    match owner {
        Owner::Test(test) => Charged::Test(test),
        Owner::Load(path) => Charged::Module(path),
    }
}
