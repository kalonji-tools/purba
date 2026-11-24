//! Command: reads the arguments, lends the parse of the module they name to
//! selection and assertion, starts a run, and returns reporting's exit code.

use std::fmt;
use std::io::{self, Write};
use std::ops::ControlFlow;
use std::path::{Path, PathBuf};

use purba_dispatch::{Handed, Module};
use purba_selection::TestId;

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
    /// The query does not parse, names no alias, or selects no test.
    /// Prototype of #1230.
    Query(String),
}

impl From<purba_reporting::Refused> for Error {
    fn from(purba_reporting::Refused { error, code }: purba_reporting::Refused) -> Self {
        Self::Refused { error, code }
    }
}

impl fmt::Display for Error {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Usage => write!(formatter, "usage: purba <query>"),
            Self::Query(words) => formatter.write_str(words),
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
    let text = query_text(arguments)?;
    let out = out.map_err(Error::Write)?;
    let options = proto_options()?;
    let aliases = aliases(Path::new("pyproject.toml"))?;
    let query = purba_selection::query::parse(&text, &aliases, options).map_err(Error::Query)?;

    // Stage one: every test file under the root, and the query's verdict on its
    // path alone. A file is parsed unless the verdict is no.
    let files = discover(Path::new("."));
    let to_parse: Vec<&PathBuf> = files
        .iter()
        .filter(|file| query.file(&slashed(file)) != Some(false))
        .collect();
    // PROTOTYPE: this line only under PURBA_PROTO_LIST, so a rater's run
    // shows only the forms.
    if std::env::var_os("PURBA_PROTO_LIST").is_some() {
        eprintln!(
            "purba: proto 1230: parser {}, depth limit {}; stage one: {} files found, {} parsed",
            options.parser.name(),
            if options.limit {
                purba_selection::query::LIMIT.to_string()
            } else {
                "off".to_owned()
            },
            files.len(),
            to_parse.len()
        );
    }

    // Stage two: parse and select each file that stage one kept.
    let mut read: Vec<Read> = Vec::new();
    for path in to_parse {
        let path = path.clone();
        match purba_errors::catch("command", || Ok(purba_parsing::parse(&path))) {
            Ok(Ok(module)) => {
                // PROTOTYPE: the name-near notice of a5a7b530, on stderr, for
                // each file that stage one kept.
                let facts = purba_parsing::Facts::of(&module);
                for (line, words) in purba_selection::proto::near_misses(&facts) {
                    eprintln!("purba: name-near: {}:{line}: {words}", path.display());
                }
                match purba_errors::catch("command", || Ok(select(&path, &module))) {
                    Ok(selected) => read.push(Read {
                        path,
                        module: Some(module),
                        selected: Ok(selected),
                    }),
                    Err(error) => read.push(Read {
                        path,
                        module: None,
                        selected: Err(error),
                    }),
                }
            }
            Ok(Err(purba_parsing::Error::Read(error))) => return Err(Error::Read(path, error)),
            Ok(Err(error)) => read.push(Read {
                path,
                module: None,
                selected: Err(purba_errors::Error::project(error.to_string())),
            }),
            Err(error) => read.push(Read {
                path,
                module: None,
                selected: Err(error),
            }),
        }
    }
    not_interrupted()?;
    let parsed = read.len();
    let mut picked = 0;
    for one in &read {
        match &one.selected {
            Ok(selected) => {
                picked += selected.tests().iter().filter(|t| picks(&query, t)).count();
                picked += selected
                    .refused()
                    .iter()
                    .filter(|refused| picks(&query, refused.test()))
                    .count();
            }
            Err(_) => picked += 1,
        }
    }
    if picked == 0 {
        return Err(Error::Query(empty(&query, &read, &files, parsed)));
    }
    if std::env::var_os("PURBA_PROTO_LIST").is_some() {
        for one in &read {
            match &one.selected {
                Ok(selected) => {
                    for test in selected.tests().iter().filter(|t| picks(&query, t)) {
                        println!("{test}");
                    }
                    for refused in selected.refused() {
                        if picks(&query, refused.test()) {
                            println!("{} (refused)", refused.test());
                        }
                    }
                }
                Err(error) => println!("{} (error: {error})", one.path.display()),
            }
        }
        eprintln!("purba: proto 1230: {picked} tests picked");
        return Ok(0);
    }

    let mut report = purba_reporting::Report::new(out);
    let mut modules = Vec::new();
    let mut ledgers: Vec<(PathBuf, purba_selection::Selected)> = Vec::new();
    for one in read {
        let path = one.path;
        let selected = match one.selected {
            Ok(selected) => selected,
            Err(error) => {
                report.error(&path.display(), &error)?;
                continue;
            }
        };
        for refused in selected.refused() {
            let test = refused.test();
            if !picks(&query, test) {
                continue;
            }
            report.case(
                test,
                &Err(purba_errors::Error::project(refused.to_string())),
                test.notices(),
                test.suppressed(),
                &[],
            )?;
        }
        let tests: Vec<TestId> = selected
            .tests()
            .iter()
            .filter(|t| picks(&query, t))
            .cloned()
            .collect();
        if let Some(fault) = selected.fault() {
            report.stopped(
                &path.display(),
                &purba_errors::Error::project(fault.to_owned()),
                &[],
                &tests,
            )?;
            continue;
        }
        let module = one.module.expect("a module that selection read");
        let facts = purba_parsing::Facts::of(&module);
        match rewritten(&path, facts, tests, &rewrite) {
            Found::Modules(found) => modules.extend(found),
            Found::NotRewritten(error, tests) => {
                report.stopped(&path.display(), &error, &[], &tests)?;
            }
        }
        ledgers.push((path, selected));
    }
    let mut failures: Vec<String> = Vec::new();
    let flow = purba_dispatch::hand(modules, |handed| {
        match handed {
            Handed::Test(test, result, suppressed) => {
                let checks: Vec<&str> = suppressed
                    .warnings
                    .iter()
                    .map(|warning| warning.check.as_str())
                    .collect();
                if let Some((_, selected)) =
                    ledgers.iter_mut().find(|(path, _)| path == test.path())
                {
                    selected.judge(&test, &checks, suppressed.judged);
                }
                if !matches!(&result, Ok(outcome)
                    if *outcome.kind() == purba_errors::Kind::Passed)
                {
                    failures.push(test.to_string());
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
            } => {
                failures.extend(tests.iter().map(ToString::to_string));
                report.stopped(&path.display(), &error, &threads, &tests)
            }
            Handed::Ended {
                path,
                error,
                named,
                tests,
            } => {
                failures.extend(tests.iter().map(ToString::to_string));
                report.ended(&path.display(), &error, named, &tests)
            }
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
    for (path, selected) in &ledgers {
        for words in selected.unused() {
            report.error(&path.display(), &purba_errors::Error::project(words))?;
        }
    }
    let code = report.finish()?;
    if !failures.is_empty() {
        eprintln!("purba: the failures as one query: {}", failures.join(" | "));
    }
    Ok(code)
}

/// One file that stage one kept: its module and what selection made of it, or
/// the error that stopped its read.
struct Read {
    path: PathBuf,
    module: Option<purba_parsing::Module>,
    selected: Result<purba_selection::Selected, purba_errors::Error>,
}

/// `path` with `/` between its parts, and no leading `./`.
fn slashed(path: &Path) -> String {
    path.components()
        .filter_map(|part| match part {
            std::path::Component::Normal(name) => Some(name.to_string_lossy().into_owned()),
            _ => None,
        })
        .collect::<Vec<_>>()
        .join("/")
}

/// Whether `query` picks `test`.
fn picks(query: &purba_selection::query::Query, test: &TestId) -> bool {
    let path = slashed(test.path());
    let id = test.to_string();
    query.picks(&purba_selection::query::Test {
        path: &path,
        id: &id,
        function: test.function(),
        tags: test.tags(),
    })
}

/// The words of a query that picks no test of `read`.
fn empty(
    query: &purba_selection::query::Query,
    read: &[Read],
    files: &[PathBuf],
    parsed: usize,
) -> String {
    let owned: Vec<(String, String, &TestId)> = read
        .iter()
        .filter_map(|one| one.selected.as_ref().ok())
        .flat_map(|selected| {
            selected.tests().iter().chain(
                selected
                    .refused()
                    .iter()
                    .map(purba_selection::Refused::test),
            )
        })
        .map(|test| (slashed(test.path()), test.to_string(), test))
        .collect();
    let tests: Vec<purba_selection::query::Test<'_>> = owned
        .iter()
        .map(|(path, id, test)| purba_selection::query::Test {
            path,
            id,
            function: test.function(),
            tags: test.tags(),
        })
        .collect();
    let files: Vec<String> = files.iter().map(|file| slashed(file)).collect();
    query.empty(&tests, &files, parsed)
}

/// The one argument after the script's name: the query.
fn query_text(arguments: &[String]) -> Result<String, Error> {
    match arguments {
        [_, query] => Ok(query.clone()),
        _ => Err(Error::Usage),
    }
}

/// The prototype's switches: `PURBA_PROTO_PARSER` and `PURBA_PROTO_DEPTH`.
fn proto_options() -> Result<purba_selection::query::Options, Error> {
    use purba_selection::query::{Crate, Options};
    let parser = match std::env::var("PURBA_PROTO_PARSER").as_deref() {
        Err(_) | Ok("pest") => Crate::Pest,
        Ok("chumsky") => Crate::Chumsky,
        Ok(other) => {
            return Err(Error::Query(format!(
                "PURBA_PROTO_PARSER is pest or chumsky, not {other}"
            )));
        }
    };
    let limit = std::env::var("PURBA_PROTO_DEPTH").as_deref() != Ok("off");
    Ok(Options { parser, limit })
}

/// The aliases of `[tool.purba.queries]` in the file at `path`, if it exists.
fn aliases(path: &Path) -> Result<Vec<purba_selection::query::Alias>, Error> {
    let Ok(text) = std::fs::read_to_string(path) else {
        return Ok(Vec::new());
    };
    let table: toml::Table = text
        .parse()
        .map_err(|error| Error::Query(format!("{}: {error}", path.display())))?;
    let Some(queries) = table
        .get("tool")
        .and_then(|tool| tool.get("purba"))
        .and_then(|purba| purba.get("queries"))
        .and_then(toml::Value::as_table)
    else {
        return Ok(Vec::new());
    };
    queries
        .iter()
        .map(|(key, value)| {
            value
                .as_str()
                .map(|body| purba_selection::query::Alias::new(key, body))
                .ok_or_else(|| {
                    Error::Query(format!(
                        "the alias `{key}` in {} is not a string",
                        path.display()
                    ))
                })
        })
        .collect()
}

/// Each `test_*.py` and `*_test.py` under `root`, sorted by its path. It skips
/// a directory whose name starts with `.`, and `__pycache__`.
fn discover(root: &Path) -> Vec<PathBuf> {
    let mut found = Vec::new();
    let mut stack = vec![root.to_path_buf()];
    while let Some(directory) = stack.pop() {
        let Ok(entries) = std::fs::read_dir(&directory) else {
            continue;
        };
        for entry in entries.flatten() {
            let name = entry.file_name().to_string_lossy().into_owned();
            let Ok(kind) = entry.file_type() else {
                continue;
            };
            let path = entry.path();
            let path = path
                .strip_prefix(".")
                .map_or(path.clone(), Path::to_path_buf);
            if kind.is_dir() {
                if !name.starts_with('.') && name != "__pycache__" {
                    stack.push(path);
                }
            } else if kind.is_file()
                && Path::new(&name).extension().is_some_and(|e| e == "py")
                && (name.starts_with("test_") || name.ends_with("_test.py"))
            {
                found.push(path);
            }
        }
    }
    found.sort_by_key(|path| slashed(path));
    found
}

/// Gives `Error::Interrupted` when purba's signal handler kept an interrupt.
fn not_interrupted() -> Result<(), Error> {
    if purba_execution::interrupted() {
        Err(Error::Interrupted)
    } else {
        Ok(())
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

/// What selection makes of `module` at `path`.
fn select(path: &Path, module: &purba_parsing::Module) -> purba_selection::Selected {
    purba_selection::select(path, &purba_parsing::Facts::of(module))
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
