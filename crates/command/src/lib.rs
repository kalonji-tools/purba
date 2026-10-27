//! Command: reads the arguments, lends the parse of the module they name to
//! selection and assertion, starts a run, and returns reporting's exit code.

use std::fmt;
use std::io::{self, Write};
use std::ops::ControlFlow;
use std::path::{Path, PathBuf};

use purba_dispatch::Module;
use purba_selection::TestId;

/// Why a run did not finish.
#[derive(Debug)]
pub enum Error {
    /// The arguments are not one path to one `.py` file.
    Usage,
    /// The module cannot be read.
    Read(PathBuf, io::Error),
    /// stdout refused a write, or takes none.
    Write(io::Error),
}

impl fmt::Display for Error {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Usage => write!(formatter, "usage: purba <path to one .py file>"),
            Self::Read(path, error) => write!(formatter, "cannot read {}: {error}", path.display()),
            Self::Write(error) => write!(formatter, "cannot write to stdout: {error}"),
        }
    }
}

/// Runs the tests of the one module that `arguments` names, and returns the
/// exit code that reporting renders. `arguments` is `sys.argv`, so it starts
/// with the name of the script.
///
/// # Errors
///
/// In the order the run meets them: when the arguments are not one path to
/// one `.py` file, when `out` is an error, when the module cannot be read,
/// and when `out` refuses a write. Each stops the run. Only a refused write
/// comes once the tests start, and no test runs after it.
pub fn run(arguments: &[String], out: io::Result<impl Write>) -> Result<u8, Error> {
    let path = path(arguments)?;
    let out = out.map_err(Error::Write)?;
    let modules = modules(path)?;
    let mut report = purba_reporting::Report::new(out);
    let flow = purba_dispatch::hand(modules, |test, result| {
        report
            .line(&test, &result)
            .map_or_else(ControlFlow::Break, ControlFlow::Continue)
    });
    if let ControlFlow::Break(error) = flow {
        return Err(Error::Write(error));
    }
    report.finish().map_err(Error::Write)
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

/// The module at `path`, with its selected tests. The parse ends here, before
/// the run.
fn modules(path: PathBuf) -> Result<Vec<Module<TestId>>, Error> {
    let module = match purba_parsing::parse(&path) {
        Ok(module) => module,
        Err(error) => return Err(Error::Read(path, error)),
    };
    let tests = purba_selection::select(&path, &module);
    if tests.is_empty() {
        return Ok(Vec::new());
    }
    let (text, asserts) = purba_assertion::rewrite(&module).into_parts();
    Ok(vec![Module::new(path, text, asserts, tests)])
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
