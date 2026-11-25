//! Command: reads the arguments, finds the root, lends the parse of each test
//! file under the path they name to selection and assertion, starts a run, and
//! returns reporting's exit code.

use std::collections::HashMap;
use std::fmt;
use std::fs;
use std::io::{self, Write};
use std::ops::ControlFlow;
use std::path::{Path, PathBuf};

use purba_dispatch::{Handed, Module};
use purba_selection::{NoTestFile, Selected, TestId};

/// Why a run did not finish.
#[derive(Debug)]
pub enum Error {
    /// The arguments are more than one path.
    Usage,
    /// The working folder cannot be read.
    Folder(io::Error),
    /// No `pyproject.toml` on the walk up from the working folder holds
    /// `[tool.purba]`. It names where to add it.
    NoRoot(Place),
    /// The path, as the tester typed it, names no test file.
    NoTestFile(String, NoTestFile),
    /// stdout takes no write, so no report exists.
    Write(io::Error),
    /// stdout refused a write of the report. `code` is the exit code of what
    /// the run met by then: 125 after an internal error, and 3 otherwise.
    Refused { error: io::Error, code: u8 },
    /// An interrupt came before selection ended, so no count exists.
    Interrupted,
}

/// Where the walk up from the working folder says to add `[tool.purba]`, by
/// its path from the working folder.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Place {
    /// The nearest `pyproject.toml` that holds `[project]`.
    Project(PathBuf),
    /// The folder that holds `.git`, or the working folder when the walk met
    /// neither.
    Folder(PathBuf),
}

impl From<purba_reporting::Refused> for Error {
    fn from(purba_reporting::Refused { error, code }: purba_reporting::Refused) -> Self {
        Self::Refused { error, code }
    }
}

impl fmt::Display for Error {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Usage => write!(formatter, "usage: purba [<path>]"),
            Self::Folder(error) => write!(formatter, "cannot read the working folder: {error}"),
            Self::NoRoot(place) => {
                write!(formatter, "no pyproject.toml holds [tool.purba]; ")?;
                match place {
                    Place::Project(file) => write!(formatter, "add it to {}", file.display()),
                    Place::Folder(folder) => {
                        write!(formatter, "add one with it to {}", folder.display())
                    }
                }
            }
            Self::NoTestFile(path, why) => write!(formatter, "{path} names no test file: {why}"),
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

/// Runs the tests of each test file under the path that `arguments` names, and
/// returns the exit code that reporting renders.
///
/// With no path, it runs each test file under the root. `arguments` is
/// `sys.argv`, so it starts with the name of the script. The working folder is
/// the folder of the process.
///
/// The line of each error found before the run, such as a folder that stage
/// one cannot read, a module that does not parse or a test that selection
/// refuses, prints before any test runs, in the order of the paths.
///
/// # Errors
///
/// Each stops the run. In the order the run meets them:
///
/// - The working folder cannot be read.
/// - The arguments are more than one path.
/// - `out` is an error.
/// - No `pyproject.toml` on the walk up from the working folder holds
///   `[tool.purba]`.
/// - The path names no test file.
/// - An interrupt comes before selection ends.
/// - `out` refuses a write.
///
/// Only a refused write comes once the tests start, and no test runs after it.
///
/// # Panics
///
/// In a build with the feature `fault`, when `PURBA_FAULT` is `command`.
pub fn run(arguments: &[String], out: io::Result<impl Write>) -> Result<u8, Error> {
    let folder = std::env::current_dir().map_err(Error::Folder)?;
    run_with(arguments, &folder, out, purba_assertion::rewrite)
}

/// [`run`] from the working folder `folder`, with `rewrite` in place of
/// assertion's rewrite of the facts of each module, so that a test can run in
/// its own folder, and put a signal inside the rewrite.
///
/// # Errors
///
/// Those of [`run`], save the working folder's.
///
/// # Panics
///
/// Where [`run`] panics.
#[doc(hidden)]
pub fn run_with(
    arguments: &[String],
    folder: &Path,
    out: io::Result<impl Write>,
    rewrite: impl Fn(&purba_parsing::Facts<'_>, &[&str]) -> purba_assertion::Rewrite,
) -> Result<u8, Error> {
    if cfg!(feature = "fault")
        && std::env::var_os("PURBA_FAULT").is_some_and(|setting| setting == "command")
    {
        panic!("PURBA_FAULT=command");
    }
    let typed = path(arguments)?;
    let out = out.map_err(Error::Write)?;
    let mut report = purba_reporting::Report::new(out);
    let root = match find_root(folder) {
        Rooted::Root(root) => root,
        Rooted::None(place) => return Err(Error::NoRoot(place)),
        Rooted::Unread(file, error) => {
            not_interrupted()?;
            report.error(&file.display(), &error)?;
            return Ok(report.finish()?);
        }
    };
    let found = purba_selection::test_files(&root, typed.as_deref().map(Path::new))
        .map_err(|why| Error::NoTestFile(typed.unwrap_or_default(), why))?;
    purba_execution::start_paths_at(root.clone());
    let (modules, mut kept) = before_the_run(&root, found, &mut report, &rewrite)?;
    hand(&root, modules, &mut kept, &mut report)?;
    for (path, selected) in &kept {
        for words in selected.unused() {
            report.error(&path.display(), &purba_errors::Error::project(words))?;
        }
    }
    Ok(report.finish()?)
}

/// What the run hands dispatch: each rewritten module, and what selection made
/// of each test file that it rewrote, by its path from the root.
type Prepared = (Vec<Module<TestId>>, Vec<(PathBuf, Selected)>);

/// Takes each test file of `found` in turn, by its path from `root`: parses
/// it, selects its tests and rewrites it, and drops its tree before it reads
/// the next. Once selection ends for every file, it prints, in the order of
/// the paths, the line of each folder that stage one cannot read and of each
/// error that a file met before the run. The error of one file or folder
/// stops only that one.
///
/// # Errors
///
/// An interrupt came before selection ended, or `report` refused a write.
fn before_the_run(
    root: &Path,
    found: purba_selection::TestFiles,
    report: &mut purba_reporting::Report<impl Write>,
    rewrite: &impl Fn(&purba_parsing::Facts<'_>, &[&str]) -> purba_assertion::Rewrite,
) -> Result<Prepared, Error> {
    if found.files.is_empty() {
        not_interrupted()?;
    }
    let mut before = Vec::with_capacity(found.files.len());
    for path in found.files {
        let stage = prepare(root, &path, rewrite)?;
        before.push((path, stage));
    }
    let mut unread = found.unread.into_iter().peekable();
    let mut modules = Vec::new();
    let mut kept = Vec::new();
    for (path, stage) in before {
        while let Some((folder, error)) = unread.next_if(|(folder, _)| *folder < path) {
            report.error(&folder.display(), &purba_errors::Error::project(error))?;
        }
        let shown = path.display();
        let (selected, rewritten) = match stage {
            Before::Stopped(error) => {
                report.error(&shown, &error)?;
                continue;
            }
            Before::Selected(selected, rewritten) => (selected, rewritten),
        };
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
        match rewritten {
            Found::Modules(found) => modules.extend(found),
            Found::NotRewritten(error, tests) => {
                report.stopped(&shown, &error, &[], &tests)?;
                continue;
            }
        }
        kept.push((path, selected));
    }
    for (folder, error) in unread {
        report.error(&folder.display(), &purba_errors::Error::project(error))?;
    }
    Ok((modules, kept))
}

/// Hands each of `modules` to dispatch in one call, and reports what dispatch
/// hands on. Each path that a line shows starts at `root`. `kept` judges the
/// suppressions of each test that ran.
///
/// # Errors
///
/// `report` refused a write.
fn hand(
    root: &Path,
    modules: Vec<Module<TestId>>,
    kept: &mut [(PathBuf, Selected)],
    report: &mut purba_reporting::Report<impl Write>,
) -> Result<(), Error> {
    let index: HashMap<PathBuf, usize> = kept
        .iter()
        .enumerate()
        .map(|(at, (path, _))| (path.clone(), at))
        .collect();
    let from_root = |path: &Path| -> PathBuf {
        path.strip_prefix(root)
            .map_or_else(|_| path.to_owned(), Path::to_path_buf)
    };
    let flow = purba_dispatch::hand(modules, |handed| {
        match handed {
            Handed::Test(test, result, suppressed) => {
                let checks: Vec<&str> = suppressed
                    .warnings
                    .iter()
                    .map(|warning| warning.check.as_str())
                    .collect();
                if let Some(&at) = index.get(test.path()) {
                    kept[at].1.judge(&test, &checks, suppressed.judged);
                }
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
            } => report.stopped(&from_root(&path).display(), &error, &threads, &tests),
            Handed::Ended {
                path,
                error,
                named,
                tests,
            } => report.ended(&from_root(&path).display(), &error, named, &tests),
            Handed::LoadThread(path, error) => report.error(&from_root(&path).display(), &error),
            Handed::TestThread(test, error) => report.error(&test, &error),
            Handed::Unreached(_) => {
                report.unreached();
                Ok(())
            }
        }
        .map_or_else(ControlFlow::Break, ControlFlow::Continue)
    });
    match flow {
        ControlFlow::Break(refused) => Err(refused.into()),
        ControlFlow::Continue(()) => Ok(()),
    }
}

/// Gives `Error::Interrupted` when purba's signal handler kept an interrupt.
fn not_interrupted() -> Result<(), Error> {
    if purba_execution::interrupted() {
        Err(Error::Interrupted)
    } else {
        Ok(())
    }
}

/// The path that `arguments` names after the script's name, if any.
fn path(arguments: &[String]) -> Result<Option<String>, Error> {
    match arguments {
        [] | [_] => Ok(None),
        [_, argument] => Ok(Some(argument.clone())),
        _ => Err(Error::Usage),
    }
}

/// What the walk up from the working folder finds.
enum Rooted {
    /// The folder of the nearest `pyproject.toml` that holds `[tool.purba]`.
    Root(PathBuf),
    /// No such file, and where to add `[tool.purba]`.
    None(Place),
    /// A `pyproject.toml` on the walk that is not TOML, by its path from the
    /// working folder, with its project error.
    Unread(PathBuf, purba_errors::Error),
}

/// Walks up from `folder` to the first `pyproject.toml` that holds
/// `[tool.purba]`, and stops after the folder that holds `.git`, a folder or
/// a file. The root is canonical.
fn find_root(folder: &Path) -> Rooted {
    let start = folder.canonicalize().unwrap_or_else(|_| folder.to_owned());
    let mut up = PathBuf::new();
    let mut project = None;
    for current in start.ancestors() {
        let file = current.join("pyproject.toml");
        if file.is_file() {
            let table = match read(&file) {
                Ok(table) => table,
                Err(error) => return Rooted::Unread(up.join("pyproject.toml"), error),
            };
            if holds(&table, &["tool", "purba"]) {
                return Rooted::Root(current.to_owned());
            }
            if project.is_none() && holds(&table, &["project"]) {
                project = Some(up.join("pyproject.toml"));
            }
        }
        if current.join(".git").symlink_metadata().is_ok() {
            return Rooted::None(project.map_or_else(|| Place::Folder(shown(&up)), Place::Project));
        }
        up.push("..");
    }
    Rooted::None(project.map_or_else(|| Place::Folder(PathBuf::from(".")), Place::Project))
}

/// `up` as a folder: `.` for the working folder.
fn shown(up: &Path) -> PathBuf {
    if up.as_os_str().is_empty() {
        PathBuf::from(".")
    } else {
        up.to_owned()
    }
}

/// The table that the `pyproject.toml` at `file` holds, or its project error.
fn read(file: &Path) -> Result<toml::Table, purba_errors::Error> {
    let text = fs::read_to_string(file)
        .map_err(|error| purba_errors::Error::project(error.to_string()))?;
    text.parse::<toml::Table>().map_err(|error| {
        let at = error.span().map_or(0, |span| span.start);
        let before = text.get(..at).unwrap_or_default();
        let line = before.matches('\n').count() + 1;
        let column = before
            .rsplit('\n')
            .next()
            .unwrap_or_default()
            .chars()
            .count()
            + 1;
        purba_errors::Error::project(format!(
            "TOML parse error at line {line}, column {column}: {}",
            error.message()
        ))
    })
}

/// Whether `table` holds a table at the keys of `keys`, one inside the other.
fn holds(table: &toml::Table, keys: &[&str]) -> bool {
    let mut table = table;
    for key in keys {
        match table.get(*key).and_then(toml::Value::as_table) {
            Some(inner) => table = inner,
            None => return false,
        }
    }
    true
}

/// What one test file leaves before the run, once its tree drops.
enum Before {
    /// The file does not parse, or selection panicked.
    Stopped(purba_errors::Error),
    /// What selection makes of the module, and what the rewrite leaves of it.
    Selected(Selected, Found),
}

/// Parses the test file at `path` from `root`, selects its tests, and
/// rewrites it. Its tree drops here.
///
/// # Errors
///
/// An interrupt came before its selection ended.
fn prepare(
    root: &Path,
    path: &Path,
    rewrite: &impl Fn(&purba_parsing::Facts<'_>, &[&str]) -> purba_assertion::Rewrite,
) -> Result<Before, Error> {
    let read = root.join(path);
    let module = match parse(&read) {
        Ok(module) => module,
        Err(error) => {
            not_interrupted()?;
            return Ok(Before::Stopped(error));
        }
    };
    let selection = purba_errors::catch("command", || {
        let facts = purba_parsing::Facts::of(&module);
        let selected = purba_selection::select(path, &facts);
        Ok((facts, selected))
    });
    not_interrupted()?;
    let (facts, selected) = match selection {
        Ok(selection) => selection,
        Err(error) => return Ok(Before::Stopped(error)),
    };
    let found = selected.fault().map_or_else(
        || rewritten(&read, facts, selected.tests().to_vec(), rewrite),
        |fault| {
            Found::NotRewritten(
                purba_errors::Error::project(fault.to_owned()),
                selected.tests().to_vec(),
            )
        },
    );
    Ok(Before::Selected(selected, found))
}

/// The parse of the module at `file`, or the error that stops it.
fn parse(file: &Path) -> Result<purba_parsing::Module, purba_errors::Error> {
    purba_errors::catch("command", || {
        purba_parsing::parse(file).map_err(|error| purba_errors::Error::project(error.to_string()))
    })
}

/// What selection and the rewrite leave of one module.
enum Found {
    /// The rewritten module with the tests that selection hands on, or no
    /// module when selection hands on no test.
    Modules(Vec<Module<TestId>>),
    /// The error that stops the module before it loads, and each test that
    /// selection hands on, which the error blocks: selection's fault, or the
    /// internal error of a panic in the rewrite.
    NotRewritten(purba_errors::Error, Vec<TestId>),
}

/// What the rewrite leaves of the module read from `path`, from its `facts`,
/// with the `tests` that selection hands on. A module with no test to hand on
/// is neither rewritten nor loaded. The facts drop here, before the module
/// drops.
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

    fn path_of(arguments: &[&str]) -> Result<Option<String>, String> {
        let mut argv = vec!["purba".to_owned()];
        argv.extend(arguments.iter().map(|&argument| argument.to_owned()));
        path(&argv).map_err(|error| error.to_string())
    }

    #[test]
    fn no_argument_or_one_after_the_scripts_name_is_the_path() {
        assert_eq!(path_of(&[]), Ok(None), "no argument names no path");
        assert_eq!(
            path_of(&["tests"]),
            Ok(Some("tests".to_owned())),
            "a folder"
        );
        assert_eq!(
            path_of(&["tests/test_a.py"]),
            Ok(Some("tests/test_a.py".to_owned())),
            "a path as the tester typed it"
        );
        assert_eq!(
            path_of(&["--help"]),
            Ok(Some("--help".to_owned())),
            "a flag is a path, which stage one finds no file at"
        );
    }

    #[test]
    fn two_or_more_arguments_are_the_usage_error() {
        let usage = Err("usage: purba [<path>]".to_owned());
        assert_eq!(path_of(&["a.py", "b.py"]), usage, "two arguments");
        assert_eq!(path_of(&["a", "b", "c"]), usage, "three arguments");
    }

    #[test]
    fn each_error_says_why() {
        assert_eq!(
            Error::NoRoot(Place::Project(PathBuf::from("../pyproject.toml"))).to_string(),
            "no pyproject.toml holds [tool.purba]; add it to ../pyproject.toml",
            "the refusal names the pyproject.toml of the project"
        );
        assert_eq!(
            Error::NoRoot(Place::Folder(PathBuf::from(".."))).to_string(),
            "no pyproject.toml holds [tool.purba]; add one with it to ..",
            "the refusal names the folder that holds .git"
        );
        assert_eq!(
            Error::NoTestFile("tests/x.py".to_owned(), NoTestFile::Name).to_string(),
            format!("tests/x.py names no test file: {}", NoTestFile::Name),
            "the refusal names the path as the tester typed it, and selection's reason"
        );
        assert_eq!(
            Error::Write(io::Error::other("boom")).to_string(),
            "cannot write to stdout: boom",
            "the write error names stdout"
        );
    }
}
