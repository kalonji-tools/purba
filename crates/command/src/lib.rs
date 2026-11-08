//! Command: reads the arguments, lends the parse of the module they name to
//! selection and assertion, starts a run, and returns reporting's exit code.

use std::fmt;
use std::io::{self, Write};
use std::ops::ControlFlow;
use std::path::{Path, PathBuf};

use purba_dispatch::{Charged, Handed, Module};
use purba_selection::{Refused, TestId};

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
    if cfg!(feature = "fault")
        && std::env::var_os("PURBA_FAULT").is_some_and(|setting| setting == "command")
    {
        panic!("PURBA_FAULT=command");
    }
    let path = path(arguments)?;
    let out = out.map_err(Error::Write)?;
    let mut report = purba_reporting::Report::new(out);
    let parsed = match purba_execution::catch("command", || Ok(purba_parsing::parse(&path))) {
        Ok(Ok(module)) => module,
        Ok(Err(purba_parsing::Error::Read(error))) => return Err(Error::Read(path, error)),
        Ok(Err(purba_parsing::Error::Syntax(error))) => {
            let error = purba_execution::Error::project(error.to_string());
            return stopped(report, &path, &error);
        }
        Err(error) => return stopped(report, &path, &error),
    };
    let (found, refused_tests) = match modules(&path, &parsed) {
        Ok(found) => found,
        Err(error) => return stopped(report, &path, &error),
    };
    not_interrupted()?;
    drop(parsed);
    for refused in &refused_tests {
        report.line(
            refused.test(),
            &Err(purba_execution::Error::project(refused.to_string())),
        )?;
    }
    let modules = match found {
        Found::Modules(modules) => modules,
        Found::NotRewritten(error, tests) => {
            report.error(&path.display(), &error)?;
            for test in &tests {
                report.blocked(test, &path.display())?;
            }
            return Ok(report.finish()?);
        }
    };
    let flow = purba_dispatch::hand(modules, |handed| {
        match handed {
            Handed::Test(test, result) => report.line(&test, &result),
            Handed::NotLoaded(path, error) | Handed::Thread(Charged::Module(path), error) => {
                report.error(&path.display(), &error)
            }
            Handed::Blocked(test, path) => report.blocked(&test, &path.display()),
            Handed::Thread(Charged::Test(test), error) => report.error(&test, &error),
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
    Ok(report.finish()?)
}

/// Prints the line of a module whose error stops the run before any test, then
/// the count. After an interrupt, it prints neither.
fn stopped(
    mut report: purba_reporting::Report<impl Write>,
    path: &Path,
    error: &purba_execution::Error,
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
    NotRewritten(purba_execution::Error, Vec<TestId>),
}

/// What selection and the rewrite leave of the module at `path`, and each test
/// that selection refuses. A module with no test to hand on is neither
/// rewritten nor loaded.
///
/// # Errors
///
/// The internal error of a panic in selection.
fn modules(
    path: &Path,
    module: &purba_parsing::Module,
) -> Result<(Found, Vec<Refused>), purba_execution::Error> {
    let (tests, refused) =
        purba_execution::catch("command", || Ok(purba_selection::select(path, module)))?;
    if tests.is_empty() {
        return Ok((Found::Modules(Vec::new()), refused));
    }
    let found = match purba_execution::catch("command", || Ok(purba_assertion::rewrite(module))) {
        Ok(rewrite) => Found::Modules(vec![Module::new(path.to_owned(), rewrite, tests)]),
        Err(error) => Found::NotRewritten(error, tests),
    };
    Ok((found, refused))
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
