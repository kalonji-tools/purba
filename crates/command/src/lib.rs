//! Command: reads the arguments, finds the root, lends the parse of each test
//! file under the path they name to selection and assertion, starts a run, and
//! returns reporting's exit code.

use std::collections::HashMap;
use std::ffi::{OsStr, OsString};
use std::fmt;
use std::io::{self, Write};
use std::ops::ControlFlow;
use std::path::{Path, PathBuf};

use purba_config::Walk;
use purba_dispatch::{Handed, Module};
use purba_selection::{Reached, Selected, TestId, Unnamed, Unparsed};

/// Why a run did not finish.
#[derive(Debug)]
pub enum Error {
    /// The arguments are more than one query.
    Usage,
    /// The query does not parse, or its aliases do not expand.
    Query(Unparsed),
    /// The words of each error of the `PURBA_*` variables, sorted by the name
    /// of the variable.
    Environment(Vec<String>),
    /// The working folder cannot be read.
    Folder(io::Error),
    /// No `pyproject.toml` on the walk up from the working folder holds
    /// `[tool.purba]`. It names where to add it.
    NoRoot(Place),
    /// A term of the query names nothing that purba read.
    Unnamed(Unnamed),
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
            Self::Usage => write!(formatter, "usage: purba [<query>]"),
            Self::Query(why) => write!(formatter, "{why}"),
            Self::Environment(lines) => write!(formatter, "{}", lines.join("; ")),
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
            Self::Unnamed(why) => write!(formatter, "{why}"),
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

/// Runs the tests that the query of `arguments` names, and returns the exit
/// code that reporting renders.
///
/// With no query, it runs each test file under the root. `arguments` is
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
/// - The arguments are more than one query.
/// - The query does not parse.
/// - A `PURBA_*` variable is near `PURBA_IMPLICIT_ROOT`, or
///   `PURBA_IMPLICIT_ROOT` holds a value other than `1`, `0` or nothing.
/// - `out` is an error.
/// - No `pyproject.toml` on the walk up from the working folder holds
///   `[tool.purba]`, and `PURBA_IMPLICIT_ROOT` is not `1`.
/// - A call of the query names no alias, hands an alias the wrong count of
///   arguments, or nests deeper than 255 levels as the aliases expand.
/// - An id of the query names no test file.
/// - An interrupt comes before selection ends.
/// - A term of the query names nothing that purba read.
/// - `out` refuses a write.
///
/// Only a refused write comes once the tests start, and no test runs after it.
///
/// # Panics
///
/// In a build with the feature `fault`, when `PURBA_FAULT` is `command`.
pub fn run(arguments: &[String], out: io::Result<impl Write>) -> Result<u8, Error> {
    let folder = std::env::current_dir().map_err(Error::Folder)?;
    let variables: Vec<(OsString, OsString)> = std::env::vars_os().collect();
    run_with(
        arguments,
        &folder,
        &variables,
        out,
        purba_assertion::rewrite,
    )
}

/// [`run`] from the working folder `folder`, with `variables` in place of the
/// variables of the process, and `rewrite` in place of assertion's rewrite of
/// the facts of each module, so that a test can run in its own folder, set its
/// own variables, and put a signal inside the rewrite.
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
    variables: &[(OsString, OsString)],
    out: io::Result<impl Write>,
    rewrite: impl Fn(&purba_parsing::Facts<'_>, &[&str]) -> purba_assertion::Rewrite,
) -> Result<u8, Error> {
    if cfg!(feature = "fault")
        && std::env::var_os("PURBA_FAULT").is_some_and(|setting| setting == "command")
    {
        panic!("PURBA_FAULT=command");
    }
    let query = typed(arguments)?
        .as_deref()
        .map(purba_selection::query)
        .transpose()
        .map_err(Error::Query)?;
    let implicit = implicit_root(variables)?;
    let out = out.map_err(Error::Write)?;
    let mut report = purba_reporting::Report::new(out);
    // An implicit root has no project's file, so its table of dials is empty
    // and no dial refuses it. Its notice prints once selection ends.
    let mut notice = None;
    let (root, (file, settings)) = match purba_config::walk(folder) {
        Walk::Found { file, settings } => (file.folder, (file.path, settings)),
        Walk::Unread { file, words } => {
            not_interrupted()?;
            report.error(&file.display(), &purba_errors::Error::project(words))?;
            return Ok(report.finish()?);
        }
        Walk::Missing {
            project,
            git,
            start,
        } => {
            let (place, step) = match (project, git) {
                (Some(project), _) => (Place::Project(project.path), Step::Project(project.folder)),
                (None, Some(git)) => (Place::Folder(git.path), Step::Git(git.folder)),
                (None, None) => (Place::Folder(PathBuf::from(".")), Step::Working(start)),
            };
            if !implicit {
                return Err(Error::NoRoot(place));
            }
            notice = Some(step.line(home(variables).as_deref()));
            (step.into_folder(), Default::default())
        }
    };
    let (patterns, aliases) = match dials(settings) {
        Ok(dials) => dials,
        Err(errors) => {
            not_interrupted()?;
            for words in errors {
                report.error(&file.display(), &purba_errors::Error::project(words))?;
            }
            return Ok(report.finish()?);
        }
    };
    let query = query
        .map(|query| query.expand(&aliases))
        .transpose()
        .map_err(Error::Query)?;
    let reached =
        purba_selection::reach(&root, query.as_ref(), &patterns).map_err(Error::Unnamed)?;
    purba_execution::start_paths_at(root.clone(), folder.to_path_buf());
    let (modules, mut kept) =
        before_the_run(&root, &reached, notice.as_deref(), &mut report, &rewrite)?;
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

/// Takes each test file that the query of `reached` reaches in turn, by its
/// path from `root`: parses it, selects its tests and rewrites it, and drops
/// its tree before it reads the next. Once selection ends for every file, it
/// checks that each term of the query names something, then prints `notice`,
/// the notice of an implicit root, if any. Then it prints, in the order of the
/// paths, the line of each folder that stage one cannot read and of each
/// error that a file met before the run. The error of one file or folder
/// stops only that one.
///
/// # Errors
///
/// An interrupt came before selection ended, a term of the query names
/// nothing, or `report` refused a write.
fn before_the_run(
    root: &Path,
    reached: &Reached,
    notice: Option<&str>,
    report: &mut purba_reporting::Report<impl Write>,
    rewrite: &impl Fn(&purba_parsing::Facts<'_>, &[&str]) -> purba_assertion::Rewrite,
) -> Result<Prepared, Error> {
    let found = reached.files();
    if found.files.is_empty() {
        not_interrupted()?;
    }
    let mut before = Vec::with_capacity(found.files.len());
    for path in &found.files {
        let stage = prepare(root, path, reached, rewrite)?;
        before.push((path.clone(), stage));
    }
    reached
        .check(before.iter().map(|(path, stage)| {
            let selected = match stage {
                Before::Selected(selected, _) => Some(&**selected),
                Before::Stopped(_) => None,
            };
            (path.as_path(), selected)
        }))
        .map_err(Error::Unnamed)?;
    if let Some(words) = notice {
        report.notice(words)?;
    }
    let mut unread = found.unread.iter().cloned().peekable();
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
            Before::Selected(selected, rewritten) => (*selected, rewritten),
        };
        for line in selected.before_the_run() {
            match line {
                purba_selection::Before::Refused(refused) => {
                    let test = refused.test();
                    report.case(
                        test,
                        &Err(purba_errors::Error::project(refused.to_string())),
                        test.notices(),
                        test.suppressed(),
                        &[],
                    )?;
                }
                purba_selection::Before::NameNear(near) => {
                    report.error(
                        &format_args!("{}::{}", near.path().display(), near.name()),
                        &purba_errors::Error::project(near.to_string()),
                    )?;
                }
            }
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

/// The query that `arguments` holds after the script's name, if any.
fn typed(arguments: &[String]) -> Result<Option<String>, Error> {
    match arguments {
        [] | [_] => Ok(None),
        [_, argument] => Ok(Some(argument.clone())),
        _ => Err(Error::Usage),
    }
}

/// The patterns of a test file and the aliases that the table `settings` under
/// `[tool.purba]` sets, or the words of each error of a dial, and then of each
/// key that no dial owns.
fn dials(
    settings: toml::Table,
) -> Result<(purba_selection::Patterns, purba_selection::Aliases), Vec<String>> {
    let project = purba_config::Project::new(settings);
    let patterns = project.resolve(&purba_selection::TEST_FILES);
    let aliases = project.resolve(&purba_selection::QUERIES);
    let mut errors = project.unknown(&[
        purba_selection::TEST_FILES.key(),
        purba_selection::QUERIES.key(),
    ]);
    match (patterns, aliases) {
        (Ok(patterns), Ok(aliases)) if errors.is_empty() => Ok((patterns.value, aliases.value)),
        (patterns, aliases) => {
            let mut words = Vec::new();
            words.extend(patterns.err().unwrap_or_default());
            words.extend(aliases.err().unwrap_or_default());
            words.append(&mut errors);
            Err(words)
        }
    }
}

/// The step to a root that `PURBA_IMPLICIT_ROOT` takes when no
/// `pyproject.toml` holds `[tool.purba]`, with the folder it gives.
enum Step {
    /// The folder of the nearest `pyproject.toml` with `[project]`.
    Project(PathBuf),
    /// The folder that holds `.git`.
    Git(PathBuf),
    /// The working folder.
    Working(PathBuf),
}

impl Step {
    /// The folder that the step gives.
    fn into_folder(self) -> PathBuf {
        match self {
            Self::Project(folder) | Self::Git(folder) | Self::Working(folder) => folder,
        }
    }

    /// The words of the notice that names the root and the step, with `~` for
    /// the start of a root under `home`.
    fn line(&self, home: Option<&Path>) -> String {
        let (folder, what) = match self {
            Self::Project(folder) => (
                folder,
                "the folder of the pyproject.toml that holds [project]",
            ),
            Self::Git(folder) => (folder, "the folder that holds .git"),
            Self::Working(folder) => (folder, "the working folder"),
        };
        let mut line = format!(
            "root {} is {what}, because PURBA_IMPLICIT_ROOT is set and no pyproject.toml \
             holds [tool.purba]",
            tilde(folder, home)
        );
        if matches!(self, Self::Working(_)) {
            line.push_str("; the id of each test starts here, and moves with the working folder");
        }
        line
    }
}

/// `folder`, with `~` in place of `home` where `folder` starts there.
fn tilde(folder: &Path, home: Option<&Path>) -> String {
    match home.and_then(|home| folder.strip_prefix(home).ok()) {
        Some(rest) if rest.as_os_str().is_empty() => "~".to_owned(),
        Some(rest) => format!("~/{}", rest.display()),
        None => folder.display().to_string(),
    }
}

/// The home folder that `HOME` in `variables` names, as the walk names a
/// folder, or `None` when `HOME` is unset or empty.
fn home(variables: &[(OsString, OsString)]) -> Option<PathBuf> {
    let (_, home) = variables
        .iter()
        .find(|(name, value)| name == "HOME" && !value.is_empty())?;
    let home = PathBuf::from(home);
    Some(home.canonicalize().unwrap_or(home))
}

/// `PURBA_IMPLICIT_ROOT`: whether the walk takes a step to a root when no
/// `pyproject.toml` holds `[tool.purba]`.
static IMPLICIT_ROOT: purba_config::Variable<bool> =
    purba_config::Variable::new("PURBA_IMPLICIT_ROOT", read_implicit_root, || false);

/// `1` takes the steps; `0` and an empty value do not.
fn read_implicit_root(value: &OsStr) -> Result<bool, Vec<String>> {
    match value.to_str() {
        Some("1") => Ok(true),
        Some("0" | "") => Ok(false),
        _ => Err(vec![format!(
            "PURBA_IMPLICIT_ROOT is {}; set 1 or 0",
            value.display()
        )]),
    }
}

/// Whether `variables` turn the steps of `PURBA_IMPLICIT_ROOT` on.
///
/// # Errors
///
/// [`Error::Environment`], with the words of each `PURBA_*` variable near
/// `PURBA_IMPLICIT_ROOT` and of a value that it refuses.
fn implicit_root(variables: &[(OsString, OsString)]) -> Result<bool, Error> {
    let environment = purba_config::Environment::new(variables.iter().cloned());
    let mut errors = environment.near(&[IMPLICIT_ROOT.name()]);
    let implicit = match environment.resolve(&IMPLICIT_ROOT) {
        Ok(resolved) => resolved.value,
        Err(words) => {
            errors.extend(words);
            false
        }
    };
    if errors.is_empty() {
        Ok(implicit)
    } else {
        // Each line starts with the name of its variable.
        errors.sort();
        Err(Error::Environment(errors))
    }
}

/// What one test file leaves before the run, once its tree drops.
enum Before {
    /// The file does not parse, or selection panicked.
    Stopped(purba_errors::Error),
    /// What selection makes of the module, and what the rewrite leaves of it.
    Selected(Box<Selected>, Found),
}

/// Parses the test file at `path` from `root`, selects the tests of it that
/// the query of `reached` picks, and rewrites it. Its tree drops here.
///
/// # Errors
///
/// An interrupt came before its selection ended.
fn prepare(
    root: &Path,
    path: &Path,
    reached: &Reached,
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
        let selected = reached.select(path, &facts);
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
    Ok(Before::Selected(Box::new(selected), found))
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

    fn query_of(arguments: &[&str]) -> Result<Option<String>, String> {
        let mut argv = vec!["purba".to_owned()];
        argv.extend(arguments.iter().map(|&argument| argument.to_owned()));
        typed(&argv).map_err(|error| error.to_string())
    }

    #[test]
    fn no_argument_or_one_after_the_scripts_name_is_the_query() {
        assert_eq!(query_of(&[]), Ok(None), "no argument names no query");
        assert_eq!(
            query_of(&["tests"]),
            Ok(Some("tests".to_owned())),
            "a folder"
        );
        assert_eq!(
            query_of(&["tests/test_a.py & !name(sub)"]),
            Ok(Some("tests/test_a.py & !name(sub)".to_owned())),
            "a query as the tester typed it"
        );
        assert_eq!(
            query_of(&["--help"]),
            Ok(Some("--help".to_owned())),
            "a flag is a query, which names nothing"
        );
    }

    #[test]
    fn two_or_more_arguments_are_the_usage_error() {
        let usage = Err("usage: purba [<query>]".to_owned());
        assert_eq!(query_of(&["a.py", "b.py"]), usage, "two arguments");
        assert_eq!(query_of(&["a", "b", "c"]), usage, "three arguments");
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
            Error::Write(io::Error::other("boom")).to_string(),
            "cannot write to stdout: boom",
            "the write error names stdout"
        );
    }
}
