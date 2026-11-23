//! Command: reads the arguments, lends the parse of the module they name to
//! selection and assertion, starts a run, and returns reporting's exit code.

use std::fmt;
use std::io::{self, Write};
use std::ops::ControlFlow;
use std::path::{Path, PathBuf};

use purba_dispatch::{Handed, Module};
use purba_selection::TestId;

mod config;

/// Why a run did not finish.
#[derive(Debug)]
pub enum Error {
    /// The arguments are not one path to one `.py` file.
    Usage,
    /// The module cannot be read.
    Read(PathBuf, io::Error),
    /// stdout takes no write, so no report exists.
    Write(io::Error),
    /// stdout refused a write of the report. `code` is the exit code of what
    /// the run met by then: 125 after an internal error, and 3 otherwise.
    Refused { error: io::Error, code: u8 },
    /// An interrupt came before selection ended, so no count exists.
    Interrupted,
    /// Prototype: a level of configuration holds what the prototype refuses.
    Config(String),
}

impl From<purba_reporting::Refused> for Error {
    fn from(purba_reporting::Refused { error, code }: purba_reporting::Refused) -> Self {
        Self::Refused { error, code }
    }
}

impl fmt::Display for Error {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Usage => write!(formatter, "usage: purba <path to one .py file>"),
            Self::Read(path, error) => write!(formatter, "cannot read {}: {error}", path.display()),
            Self::Write(error) | Self::Refused { error, .. } => {
                write!(formatter, "cannot write to stdout: {error}")
            }
            Self::Interrupted => write!(
                formatter,
                "an interrupt stopped the run before selection ended"
            ),
            Self::Config(words) => write!(formatter, "configuration: {words}"),
        }
    }
}

/// Runs the tests of the one module that `arguments` names, and returns the
/// exit code that reporting renders. `arguments` is `sys.argv`, so it starts
/// with the name of the script.
///
/// The line of each test that selection refuses prints before any test runs.
///
/// # Errors
///
/// Each stops the run. In the order the run meets them:
///
/// - The arguments are not one path to one `.py` file.
/// - `out` is an error.
/// - purba cannot read the module.
/// - An interrupt comes before selection ends.
/// - `out` refuses a write.
///
/// Only a refused write comes once the tests start, and no test runs after it.
///
/// # Panics
///
/// In a build with the feature `fault`, when `PURBA_FAULT` is `command`.
pub fn run(arguments: &[String], out: io::Result<impl Write>) -> Result<u8, Error> {
    run_with(arguments, out, purba_assertion::rewrite)
}

/// [`run`], with `rewrite` in place of assertion's rewrite of the facts of the
/// module, so that a test can put a signal inside the rewrite.
///
/// # Errors
///
/// Those of [`run`].
///
/// # Panics
///
/// Where [`run`] panics.
#[doc(hidden)]
pub fn run_with(
    arguments: &[String],
    out: io::Result<impl Write>,
    rewrite: impl Fn(&purba_parsing::Facts<'_>, &[&str]) -> purba_assertion::Rewrite,
) -> Result<u8, Error> {
    if cfg!(feature = "fault")
        && std::env::var_os("PURBA_FAULT").is_some_and(|setting| setting == "command")
    {
        panic!("PURBA_FAULT=command");
    }
    let (path, config) = config::read(arguments)?;
    let out = out.map_err(Error::Write)?;
    let mut report = purba_reporting::Report::new(out);
    let parsed = match purba_errors::catch("command", || Ok(purba_parsing::parse(&path))) {
        Ok(Ok(module)) => module,
        Ok(Err(purba_parsing::Error::Read(error))) => return Err(Error::Read(path, error)),
        Ok(Err(error)) => {
            let error = purba_errors::Error::project(error.to_string());
            return stopped(report, &path, &error);
        }
        Err(error) => return stopped(report, &path, &error),
    };
    let (facts, mut selected) = match purba_errors::catch("command", || Ok(select(&path, &parsed)))
    {
        Ok(read) => read,
        Err(error) => return stopped(report, &path, &error),
    };
    let codes = config::code(&path, &facts)?;
    config.print(&codes);
    purba_execution::set_timeouts(config.value("timeout"), config.timeouts(&codes));
    not_interrupted()?;
    for refused in selected.refused() {
        let test = refused.test();
        report.case(
            test,
            &Err(purba_errors::Error::project(refused.to_string())),
            test.notices(),
            test.suppressed(),
            &[],
        )?;
    }
    if let Some(fault) = selected.fault() {
        report.stopped(
            &path.display(),
            &purba_errors::Error::project(fault.to_owned()),
            &[],
            selected.tests(),
        )?;
        return Ok(report.finish()?);
    }
    if let Some(cap) = config.value("suppression-cap")
        && selected.suppressions() > cap as usize
    {
        report.stopped(
            &path.display(),
            &purba_errors::Error::project(format!(
                "{} suppressions, and the cap is {cap}",
                selected.suppressions()
            )),
            &[],
            selected.tests(),
        )?;
        return Ok(report.finish()?);
    }
    let found = rewritten(&path, facts, selected.tests().to_vec(), &rewrite);
    drop(parsed);
    let modules = match found {
        Found::Modules(modules) => modules,
        Found::NotRewritten(error, tests) => {
            report.stopped(&path.display(), &error, &[], &tests)?;
            return Ok(report.finish()?);
        }
    };
    let flow = purba_dispatch::hand(modules, |handed| {
        match handed {
            Handed::Test(test, result, suppressed) => {
                let checks: Vec<&str> = suppressed
                    .warnings
                    .iter()
                    .map(|warning| warning.check.as_str())
                    .collect();
                selected.judge(&test, &checks, suppressed.judged);
                report.case(
                    &test,
                    &result,
                    test.notices(),
                    test.suppressed(),
                    &suppressed.warnings,
                )
            }
            Handed::Stopped {
                path,
                error,
                threads,
                tests,
            } => report.stopped(&path.display(), &error, &threads, &tests),
            Handed::Ended {
                path,
                error,
                named,
                tests,
            } => report.ended(&path.display(), &error, named, &tests),
            Handed::LoadThread(path, error) => report.error(&path.display(), &error),
            Handed::TestThread(test, error) => report.error(&test, &error),
            Handed::Unreached(_) => {
                report.unreached();
                Ok(())
            }
        }
        .map_or_else(ControlFlow::Break, ControlFlow::Continue)
    });
    if let ControlFlow::Break(refused) = flow {
        return Err(refused.into());
    }
    for words in selected.unused() {
        report.error(&path.display(), &purba_errors::Error::project(words))?;
    }
    Ok(report.finish()?)
}

/// Prints the line of a module whose error stops the run before any test, then
/// the count. After an interrupt, it prints neither.
fn stopped(
    mut report: purba_reporting::Report<impl Write>,
    path: &Path,
    error: &purba_errors::Error,
) -> Result<u8, Error> {
    not_interrupted()?;
    report.error(&path.display(), error)?;
    Ok(report.finish()?)
}

/// Gives `Error::Interrupted` when purba's signal handler kept an interrupt.
fn not_interrupted() -> Result<(), Error> {
    if purba_execution::interrupted() {
        Err(Error::Interrupted)
    } else {
        Ok(())
    }
}

/// The one path that `arguments` names after the script's name.
fn path(arguments: &[String]) -> Result<PathBuf, Error> {
    match arguments {
        [_, argument]
            if Path::new(argument)
                .extension()
                .is_some_and(|extension| extension == "py") =>
        {
            Ok(PathBuf::from(argument))
        }
        _ => Err(Error::Usage),
    }
}

/// What selection and the rewrite leave of one module.
enum Found {
    /// The rewritten module with the tests that selection hands on, or no
    /// module when selection hands on no test.
    Modules(Vec<Module<TestId>>),
    /// The internal error of a panic in the rewrite, and each test that
    /// selection hands on, which the panic blocks.
    NotRewritten(purba_errors::Error, Vec<TestId>),
}

/// The facts of `module`, and what selection makes of the module at `path` from
/// them.
fn select<'m>(
    path: &Path,
    module: &'m purba_parsing::Module,
) -> (purba_parsing::Facts<'m>, purba_selection::Selected) {
    let facts = purba_parsing::Facts::of(module);
    let selected = purba_selection::select(path, &facts);
    (facts, selected)
}

/// What the rewrite leaves of the module at `path`, from its `facts`, with the
/// `tests` that selection hands on. A module with no test to hand on is neither
/// rewritten nor loaded. The facts drop here, before the module drops.
#[expect(
    clippy::needless_pass_by_value,
    reason = "the facts borrow the module, and must drop before it"
)]
fn rewritten(
    path: &Path,
    facts: purba_parsing::Facts<'_>,
    tests: Vec<TestId>,
    rewrite: &impl Fn(&purba_parsing::Facts<'_>, &[&str]) -> purba_assertion::Rewrite,
) -> Found {
    if tests.is_empty() {
        return Found::Modules(Vec::new());
    }
    let names: Vec<&str> = tests.iter().map(TestId::function).collect();
    match purba_errors::catch("command", || Ok(rewrite(&facts, &names))) {
        Ok(rewrite) => Found::Modules(vec![Module::new(path.to_owned(), rewrite, tests)]),
        Err(error) => Found::NotRewritten(error, tests),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn path_of(arguments: &[&str]) -> Result<PathBuf, String> {
        let mut argv = vec!["purba".to_owned()];
        argv.extend(arguments.iter().map(|&argument| argument.to_owned()));
        path(&argv).map_err(|error| error.to_string())
    }

    #[test]
    fn one_py_file_after_the_scripts_name_is_the_path() {
        assert_eq!(
            path_of(&["test_a.py"]),
            Ok(PathBuf::from("test_a.py")),
            "a file name"
        );
        assert_eq!(
            path_of(&["tests/test_a.py"]),
            Ok(PathBuf::from("tests/test_a.py")),
            "a path as the tester typed it"
        );
    }

    #[test]
    fn any_other_arguments_are_the_usage_error() {
        let usage = Err("usage: purba <path to one .py file>".to_owned());
        assert_eq!(path_of(&[]), usage, "no argument");
        assert_eq!(path_of(&["a.py", "b.py"]), usage, "two arguments");
        assert_eq!(path_of(&["test_a.txt"]), usage, "not a .py file");
        assert_eq!(path_of(&["--help"]), usage, "a flag");
        assert_eq!(path_of(&[".py"]), usage, "a name with no stem");
    }

    #[test]
    fn each_error_says_why() {
        assert_eq!(
            Error::Read(PathBuf::from("tests/test_x.py"), io::Error::other("boom")).to_string(),
            "cannot read tests/test_x.py: boom",
            "the read error names the path as the tester typed it"
        );
        assert_eq!(
            Error::Write(io::Error::other("boom")).to_string(),
            "cannot write to stdout: boom",
            "the write error names stdout"
        );
    }
}
