//! Dispatch: asks execution for a loaded zygote for each module, and runs each
//! selected test in a process that the zygote forks. It hands on what each test's
//! process gives, each module that stopped before its tests, the tests that the
//! end of their module's process blocked, and the error of a thread. It also
//! hands on each test that an interrupt left unreached.

use std::ops::ControlFlow;
use std::path::PathBuf;

use purba_errors::{Error, Outcome, Suppressed};
use purba_execution::{End, Purba, Rewrite, Step, Stop, Zygote};
use purba_selection::TestId;

/// What dispatch reads of a selected test.
pub trait Test {
    /// The name of the test's function.
    fn function(&self) -> &str;

    /// The name of each check that execution sees and that a suppression of
    /// the test reaches, beside its reason. A test has none unless it says so.
    fn suppressions(&self) -> Vec<(&str, &str)> {
        Vec::new()
    }
}

impl Test for TestId {
    fn function(&self) -> &str {
        Self::function(self)
    }

    fn suppressions(&self) -> Vec<(&str, &str)> {
        Self::suppressions(self)
            .iter()
            .map(|suppression| (suppression.check, suppression.reason.as_str()))
            .collect()
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

/// What dispatch hands on: a test with what it gave, a module that stopped
/// before its tests, the error of a thread, or an unreached test.
pub enum Handed<T> {
    /// A test that ran, beside its outcome, or the error that it gave in
    /// place of one, and what the suppressions of its case gave.
    Test(T, Result<Outcome, Error>, Suppressed),
    /// A module that stopped before any test of it ran, by its path: the error
    /// that stopped it, the error of each thread that its load took, and each
    /// of its tests, which no call reached.
    Stopped {
        path: PathBuf,
        error: Error,
        threads: Vec<Error>,
        tests: Vec<T>,
    },
    /// The tests of a module that no call reached, because the module's process
    /// ended after a test of it ran, by its path: the error of that end,
    /// whether the line of that test named it, and each test left.
    Ended {
        path: PathBuf,
        error: Error,
        named: bool,
        tests: Vec<T>,
    },
    /// The error of a thread's exception that charges a test, beside the test.
    /// It blocks no test.
    TestThread(T, Error),
    /// The error of a thread's exception that charges the load of a module,
    /// beside the module's path. It blocks no test.
    LoadThread(PathBuf, Error),
    /// A test that the run never reached, because an interrupt stopped the run.
    Unreached(T),
}

/// Asks execution for a loaded zygote for each module of `modules` that has a
/// test. Runs each of its tests, in order, in a process that the zygote forks.
///
/// The zygote loads the module once. When a Python thread is alive after the
/// load, each test's process loads the module again. `each` takes what each
/// test's process gives.
///
/// When a module stops before its tests, `each` takes it once, with its error,
/// the error of each thread that its load took, and its tests, and no test of
/// it runs. A module stops when execution gives no zygote for it: see
/// [`Purba::zygote`]. Once `each` breaks, nothing further runs, and `hand`
/// returns the break.
///
/// When the module's process ends after a test of it ran, `each` takes the
/// tests left once, with that end, and no test left runs. See
/// [`Zygote::ended`].
///
/// After each load and each test, `each` takes the error of each thread that
/// the load or the test's process took, beside the test or the module that it
/// charges. After a load, they come before the first test.
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
    purba_execution::attach(|purba: &Purba<'_>| {
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
            let (mut zygote, threads) = match purba.zygote(&executable) {
                Ok(started) => started,
                Err(Stop { error, threads }) => {
                    each(Handed::Stopped {
                        path,
                        error,
                        threads,
                        tests: module.tests,
                    })?;
                    continue;
                }
            };
            for error in threads {
                each(Handed::LoadThread(path.clone(), error))?;
            }
            let flow = run(&mut zygote, path, module.tests, &mut each);
            zygote.end();
            flow?;
        }
        ControlFlow::Continue(())
    })
}

/// Runs each of `tests` of the module at `path` in a process that `zygote`
/// forks, and hands `each` what it gives. Once the module's process ended, it
/// hands each test left as blocked by that end, and runs none of them.
fn run<T: Test + Clone, B>(
    zygote: &mut Zygote,
    path: PathBuf,
    tests: Vec<T>,
    each: &mut impl FnMut(Handed<T>) -> ControlFlow<B>,
) -> ControlFlow<B> {
    let mut tests = tests.into_iter();
    while let Some(test) = tests.next() {
        if purba_execution::interrupted() {
            each(Handed::Unreached(test))?;
            continue;
        }
        let (result, suppressed, threads) = zygote.run(test.function(), &test.suppressions());
        each(Handed::Test(test.clone(), result, suppressed))?;
        for (step, error) in threads {
            each(match step {
                Step::Test => Handed::TestThread(test.clone(), error),
                Step::Load => Handed::LoadThread(path.clone(), error),
            })?;
        }
        if purba_execution::interrupted() {
            continue;
        }
        if let Some(End { error, named }) = zygote.ended() {
            let left: Vec<T> = tests.collect();
            if left.is_empty() {
                return ControlFlow::Continue(());
            }
            return each(Handed::Ended {
                path,
                error,
                named,
                tests: left,
            });
        }
    }
    ControlFlow::Continue(())
}
