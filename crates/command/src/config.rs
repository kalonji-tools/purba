//! Prototype: the levels of configuration, and the rule that resolves them.
//! This branch is never merged.

use std::collections::HashMap;
use std::io::Write;
use std::path::{Path, PathBuf};

use crate::Error;

/// Each dial, as a key of a file and as a flag after `--`.
const DIALS: [&str; 4] = ["timeout", "suppression-cap", "workers", "retries"];

/// The dials that can change an outcome, which the fence closes to the user
/// file and to the system file.
const OUTCOME: [&str; 3] = ["timeout", "suppression-cap", "retries"];

/// The dials that tighten a check as their value gets lower.
const TIGHTENS: [&str; 2] = ["timeout", "suppression-cap"];

#[derive(Clone, Copy, PartialEq, Eq)]
enum Rule {
    HigherOverwrites,
    CodeWins,
    StricterWins,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Fence {
    Open,
    Fenced,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum User {
    Merge,
    Drop,
}

#[derive(Clone, Debug)]
enum Level {
    CommandLine,
    Environment(String),
    Code(usize),
    Project(PathBuf),
    User(PathBuf),
    System(PathBuf),
}

impl Level {
    /// The place of the level in `rule`'s order: 0 wins over 1.
    const fn rank(&self, rule: Rule) -> u8 {
        match (self, rule) {
            (Self::Code(_), Rule::CodeWins) => 0,
            (Self::CommandLine, Rule::CodeWins) => 1,
            (Self::Environment(_), Rule::CodeWins) => 2,
            (Self::CommandLine, _) => 0,
            (Self::Environment(_), _) => 1,
            (Self::Code(_), _) => 2,
            (Self::Project(_), _) => 3,
            (Self::User(_), _) => 4,
            (Self::System(_), _) => 5,
        }
    }

    fn name(&self) -> String {
        match self {
            Self::CommandLine => "the command line".to_owned(),
            Self::Environment(variable) => format!("the environment ({variable})"),
            Self::Code(line) => format!("code (line {line})"),
            Self::Project(path) => format!("the project file {}", path.display()),
            Self::User(path) => format!("the user file {}", path.display()),
            Self::System(path) => format!("the system file {}", path.display()),
        }
    }
}

#[derive(Clone, Debug)]
struct Set {
    dial: &'static str,
    value: u32,
    level: Level,
}

/// What every level outside code sets, and the switches of the prototype.
pub struct Config {
    rule: Rule,
    fence: Fence,
    user: User,
    sets: Vec<Set>,
    notes: Vec<String>,
}

/// A timeout that code sets on one test.
pub struct Code {
    function: String,
    set: Set,
}

/// The path of the module, and what each level outside code sets.
///
/// # Errors
///
/// `Error::Usage` for arguments that are not flags then one `.py` file, and
/// `Error::Config` for a value, a key, a switch or a level the prototype
/// refuses.
pub fn read(arguments: &[String]) -> Result<(PathBuf, Config), Error> {
    let mut sets = Vec::new();
    let mut rest = arguments.iter().take(1).cloned().collect::<Vec<_>>();
    let mut given = arguments.iter().skip(1);
    while let Some(argument) = given.next() {
        if let Some(flag) = argument.strip_prefix("--") {
            let dial = dial(flag).ok_or(Error::Usage)?;
            let value = given.next().ok_or(Error::Usage)?;
            sets.push(Set {
                dial,
                value: number(value).ok_or_else(|| {
                    Error::Config(format!(
                        "--{dial} {value}: a value is a non-negative integer"
                    ))
                })?,
                level: Level::CommandLine,
            });
        } else {
            rest.push(argument.clone());
        }
    }
    let path = crate::path(&rest)?;
    let rule = match switch("PURBA_PROTO_RULE", "higher-overwrites").as_str() {
        "higher-overwrites" => Rule::HigherOverwrites,
        "code-wins" => Rule::CodeWins,
        "stricter-wins" => Rule::StricterWins,
        other => {
            return Err(Error::Config(format!(
                "PURBA_PROTO_RULE={other}: no such rule"
            )));
        }
    };
    let fence = match switch("PURBA_PROTO_FENCE", "open").as_str() {
        "open" => Fence::Open,
        "fenced" => Fence::Fenced,
        other => {
            return Err(Error::Config(format!(
                "PURBA_PROTO_FENCE={other}: no such fence"
            )));
        }
    };
    let user = match switch("PURBA_PROTO_USER", "merge").as_str() {
        "merge" => User::Merge,
        "drop" => User::Drop,
        other => {
            return Err(Error::Config(format!(
                "PURBA_PROTO_USER={other}: no such way"
            )));
        }
    };
    for dial in DIALS {
        let variable = format!("PURBA_{}", dial.to_uppercase().replace('-', "_"));
        if let Some(value) = std::env::var_os(&variable) {
            let value = value.to_string_lossy().into_owned();
            sets.push(Set {
                dial,
                value: number(&value).ok_or_else(|| {
                    Error::Config(format!(
                        "{variable}={value}: a value is a non-negative integer"
                    ))
                })?,
                level: Level::Environment(variable),
            });
        }
    }
    let mut notes = Vec::new();
    let project = project(&path)?;
    let found = project.is_some();
    if let Some((file, table)) = project {
        sets.extend(table_sets(&table, &Level::Project(file), fence)?);
    }
    if let Some(file) = user_file() {
        if user == User::Drop && found {
            notes.push(format!(
                "the user file {} is dropped, as a project file holds [tool.purba]",
                file.display()
            ));
        } else {
            let table = parse(&file)?;
            sets.extend(table_sets(&table, &Level::User(file), fence)?);
        }
    }
    let system = std::env::var_os("PURBA_PROTO_SYSTEM")
        .map_or_else(|| PathBuf::from("/etc/purba/purba.toml"), PathBuf::from);
    if system.is_file() {
        let table = parse(&system)?;
        sets.extend(table_sets(&table, &Level::System(system), fence)?);
    }
    Ok((
        path,
        Config {
            rule,
            fence,
            user,
            sets,
            notes,
        },
    ))
}

/// The timeout that code sets on each test, from the `# purba: set[...]`
/// comments of the module.
///
/// # Errors
///
/// `Error::Config` for a comment above no `def`, an unknown key, a dial other
/// than `timeout`, or a value that is no non-negative integer.
pub fn code(path: &Path, facts: &purba_parsing::Facts<'_>) -> Result<Vec<Code>, Error> {
    let (found, stray) = purba_selection::settings(facts);
    if let Some(line) = stray.first() {
        return Err(Error::Config(format!(
            "{} line {line}: # purba: set[...] sits above no def",
            path.display()
        )));
    }
    let mut codes = Vec::new();
    for setting in found {
        let at = format!("{} line {}", path.display(), setting.line);
        let Some(dial) = dial(&setting.key) else {
            return Err(Error::Config(unknown(&at, &setting.key)));
        };
        if dial != "timeout" {
            return Err(Error::Config(format!(
                "{at}: code sets {dial}, and code may set only timeout"
            )));
        }
        let value = number(&setting.value).ok_or_else(|| {
            Error::Config(format!(
                "{at}: set[{dial}] {}: a value is a non-negative integer",
                setting.value
            ))
        })?;
        codes.push(Code {
            function: setting.function,
            set: Set {
                dial,
                value,
                level: Level::Code(setting.line),
            },
        });
    }
    Ok(codes)
}

impl Config {
    /// The value of `dial` and the level it comes from, with `code` as the
    /// value that code sets on one test, or `None` when no level sets it.
    fn resolve(&self, dial: &str, code: Option<&Set>) -> Option<Set> {
        let candidates = self
            .sets
            .iter()
            .filter(|set| set.dial == dial)
            .chain(code)
            .cloned();
        if self.rule == Rule::StricterWins && TIGHTENS.contains(&dial) {
            candidates.min_by_key(|set| (set.value, set.level.rank(Rule::HigherOverwrites)))
        } else {
            candidates.min_by_key(|set| set.level.rank(self.rule))
        }
    }

    /// The resolved value of `dial` for every test that code sets nothing on.
    pub fn value(&self, dial: &str) -> Option<u32> {
        self.resolve(dial, None).map(|set| set.value)
    }

    /// The timeout of each test that code sets one on, by its function.
    pub fn timeouts(&self, codes: &[Code]) -> HashMap<String, u32> {
        codes
            .iter()
            .filter_map(|code| {
                self.resolve("timeout", Some(&code.set))
                    .map(|set| (code.function.clone(), set.value))
            })
            .collect()
    }

    /// Writes the header, then one line for each dial that some level sets.
    pub fn print(&self, codes: &[Code]) {
        let rule = match self.rule {
            Rule::HigherOverwrites => "higher-overwrites",
            Rule::CodeWins => "code-wins",
            Rule::StricterWins => "stricter-wins",
        };
        let fence = match self.fence {
            Fence::Open => "open",
            Fence::Fenced => "fenced",
        };
        let user = match self.user {
            User::Merge => "merge",
            User::Drop => "drop",
        };
        let mut lines = vec![format!(
            "purba: configuration, rule {rule}, fence {fence}, user file {user}"
        )];
        lines.extend(self.notes.iter().map(|note| format!("purba: {note}")));
        for dial in DIALS {
            if let Some(set) = self.resolve(dial, None) {
                let every = if dial == "timeout" && !codes.is_empty() {
                    ", for every other test"
                } else {
                    ""
                };
                lines.push(format!(
                    "purba: {dial} = {}, from {}{every}",
                    set.value,
                    set.level.name()
                ));
            }
            if dial == "timeout" {
                for code in codes {
                    if let Some(set) = self.resolve(dial, Some(&code.set)) {
                        lines.push(format!(
                            "purba: timeout = {}, from {}, for {}",
                            set.value,
                            set.level.name(),
                            code.function
                        ));
                    }
                }
            }
        }
        let mut stderr = std::io::stderr();
        for line in lines {
            drop(writeln!(stderr, "{line}"));
        }
    }
}

/// The dial named `name`, or `None`.
fn dial(name: &str) -> Option<&'static str> {
    DIALS.into_iter().find(|dial| *dial == name)
}

/// `text` as a non-negative integer, or `None`.
fn number(text: &str) -> Option<u32> {
    text.trim().parse().ok()
}

/// The value of the switch `variable`, or `default` when it is unset.
fn switch(variable: &str, default: &str) -> String {
    std::env::var_os(variable).map_or_else(
        || default.to_owned(),
        |value| value.to_string_lossy().into_owned(),
    )
}

/// The words for the unknown `key` at `at`, with the nearest dial when one is
/// close.
fn unknown(at: &str, key: &str) -> String {
    let nearest = DIALS
        .into_iter()
        .map(|dial| (distance(key, dial), dial))
        .min()
        .filter(|(distance, _)| *distance <= 2)
        .map_or_else(String::new, |(_, dial)| format!(", did you mean `{dial}`?"));
    format!("{at}: unknown key `{key}`{nearest}")
}

/// The edit distance from `a` to `b`.
fn distance(a: &str, b: &str) -> usize {
    let b: Vec<char> = b.chars().collect();
    let mut row: Vec<usize> = (0..=b.len()).collect();
    for (i, x) in a.chars().enumerate() {
        let mut previous = row[0];
        row[0] = i + 1;
        for (j, y) in b.iter().enumerate() {
            let above = row[j + 1];
            row[j + 1] = (previous + usize::from(x != *y))
                .min(row[j] + 1)
                .min(above + 1);
            previous = above;
        }
    }
    row[b.len()]
}

/// The `[tool.purba]` table of the first `pyproject.toml` that holds one, from
/// the module's directory up, and the path of its file.
fn project(module: &Path) -> Result<Option<(PathBuf, toml::Table)>, Error> {
    let module = std::path::absolute(module)
        .map_err(|error| Error::Config(format!("{}: {error}", module.display())))?;
    for directory in module.ancestors().skip(1) {
        let file = directory.join("pyproject.toml");
        if !file.is_file() {
            continue;
        }
        let table = parse(&file)?;
        if let Some(toml::Value::Table(purba)) = table
            .get("tool")
            .and_then(|tool| tool.get("purba"))
            .cloned()
        {
            return Ok(Some((file, purba)));
        }
    }
    Ok(None)
}

/// The user file: `$XDG_CONFIG_HOME/purba/purba.toml`, or
/// `$HOME/.config/purba/purba.toml`, when it exists.
fn user_file() -> Option<PathBuf> {
    let base = std::env::var_os("XDG_CONFIG_HOME")
        .filter(|value| !value.is_empty())
        .map(PathBuf::from)
        .or_else(|| std::env::var_os("HOME").map(|home| PathBuf::from(home).join(".config")))?;
    let file = base.join("purba").join("purba.toml");
    file.is_file().then_some(file)
}

fn parse(file: &Path) -> Result<toml::Table, Error> {
    let text = std::fs::read_to_string(file)
        .map_err(|error| Error::Config(format!("{}: {error}", file.display())))?;
    text.parse::<toml::Table>()
        .map_err(|error| Error::Config(format!("{}: {error}", file.display())))
}

/// What `table` sets at `level`, under `fence`.
fn table_sets(table: &toml::Table, level: &Level, fence: Fence) -> Result<Vec<Set>, Error> {
    let at = level.name();
    let mut sets = Vec::new();
    for (key, value) in table {
        let Some(dial) = dial(key) else {
            return Err(Error::Config(unknown(&at, key)));
        };
        let closed = matches!(level, Level::User(_) | Level::System(_));
        if fence == Fence::Fenced && closed && OUTCOME.contains(&dial) {
            return Err(Error::Config(format!(
                "{at} sets {dial}, and the fence closes this level to a dial \
                 that can change an outcome"
            )));
        }
        let value = value
            .as_integer()
            .and_then(|value| u32::try_from(value).ok())
            .ok_or_else(|| {
                Error::Config(format!(
                    "{at}: {dial} = {value}: a value is a non-negative integer"
                ))
            })?;
        sets.push(Set {
            dial,
            value,
            level: level.clone(),
        });
    }
    Ok(sets)
}
