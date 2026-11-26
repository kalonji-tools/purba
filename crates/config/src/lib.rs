//! The crate of the configuration concern: walks to the project's file,
//! resolves each dial from its levels, keeps the level of each value, and
//! names each key that no dial owns and each variable whose name is near a
//! dial's.

use std::collections::BTreeMap;
use std::ffi::{OsStr, OsString};
use std::fs;
use std::path::{Path, PathBuf};

/// A setting that a part declares: its key under `[tool.purba]`, the function
/// of the part that reads a value, and its default.
pub struct Dial<T> {
    key: &'static str,
    read: fn(&toml::Value) -> Result<T, Vec<String>>,
    default: fn() -> T,
}

impl<T> Dial<T> {
    /// The dial under `key`, whose value `read` reads, and which takes
    /// `default` when no level sets it. `read` gives the words of each error of
    /// a value it refuses.
    pub const fn new(
        key: &'static str,
        read: fn(&toml::Value) -> Result<T, Vec<String>>,
        default: fn() -> T,
    ) -> Self {
        Self { key, read, default }
    }

    /// The key of the dial under `[tool.purba]`.
    #[must_use]
    pub const fn key(&self) -> &'static str {
        self.key
    }
}

/// A setting that a part declares on the level of the environment: the name
/// of its `PURBA_*` variable, the function of the part that reads a value, and
/// its default.
pub struct Variable<T> {
    name: &'static str,
    read: fn(&OsStr) -> Result<T, Vec<String>>,
    default: fn() -> T,
}

impl<T> Variable<T> {
    /// The dial of the variable `name`, whose value `read` reads, and which
    /// takes `default` when the variable is not set. `read` gives the words of
    /// each error of a value it refuses.
    pub const fn new(
        name: &'static str,
        read: fn(&OsStr) -> Result<T, Vec<String>>,
        default: fn() -> T,
    ) -> Self {
        Self {
            name,
            read,
            default,
        }
    }

    /// The name of the variable.
    #[must_use]
    pub const fn name(&self) -> &'static str {
        self.name
    }
}

/// The level that a value came from.
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum Level {
    /// No level sets the dial, so it takes its default.
    Default,
    /// A `PURBA_*` variable of the environment.
    Environment,
    /// The project's file, under `[tool.purba]`.
    Project,
}

/// The level of the environment: each variable whose name starts with
/// `PURBA_`, sorted by name.
#[derive(Debug, Clone, Default)]
pub struct Environment {
    variables: BTreeMap<String, OsString>,
}

/// The most edits from a dial's variable at which a `PURBA_*` variable is
/// near it.
const NEAR: usize = 2;

impl Environment {
    /// The level that the pairs of `variables`, a name and a value, set. It
    /// keeps a variable only when its name is UTF-8 and starts with `PURBA_`.
    pub fn new(variables: impl IntoIterator<Item = (OsString, OsString)>) -> Self {
        Self {
            variables: variables
                .into_iter()
                .filter_map(|(name, value)| {
                    let name = name.into_string().ok()?;
                    name.starts_with("PURBA_").then_some((name, value))
                })
                .collect(),
        }
    }

    /// The value of `variable`, and its level.
    ///
    /// # Errors
    ///
    /// The words of each error of the value that the environment sets.
    pub fn resolve<T>(&self, variable: &Variable<T>) -> Result<Resolved<T>, Vec<String>> {
        self.variables.get(variable.name).map_or_else(
            || {
                Ok(Resolved {
                    value: (variable.default)(),
                    level: Level::Default,
                })
            },
            |value| {
                (variable.read)(value).map(|value| Resolved {
                    value,
                    level: Level::Environment,
                })
            },
        )
    }

    /// The words of each variable whose name is one or two edits from a name
    /// of `names`, the variables of the dials, and is none of them, sorted by
    /// name, with the nearest.
    #[must_use]
    pub fn near(&self, names: &[&str]) -> Vec<String> {
        self.variables
            .keys()
            .filter(|name| !names.contains(&name.as_str()))
            .filter_map(|name| {
                let dial = nearest(name, names)?;
                (distance(name, dial) <= NEAR)
                    .then(|| format!("{name} is no variable of purba; the nearest is {dial}"))
            })
            .collect()
    }
}

/// The value of a dial, and the level that it came from.
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Resolved<T> {
    /// The value.
    pub value: T,
    /// The level that it came from.
    pub level: Level,
}

/// The project's level: the table under `[tool.purba]` of the project's file.
#[derive(Debug, Clone, Default)]
pub struct Project {
    table: toml::Table,
}

impl Project {
    /// The project's level that `table`, the table under `[tool.purba]`, holds.
    #[must_use]
    pub const fn new(table: toml::Table) -> Self {
        Self { table }
    }

    /// The value of `dial`, and its level.
    ///
    /// # Errors
    ///
    /// The words of each error of the value that the project's file sets.
    pub fn resolve<T>(&self, dial: &Dial<T>) -> Result<Resolved<T>, Vec<String>> {
        self.table.get(dial.key).map_or_else(
            || {
                Ok(Resolved {
                    value: (dial.default)(),
                    level: Level::Default,
                })
            },
            |value| {
                (dial.read)(value).map(|value| Resolved {
                    value,
                    level: Level::Project,
                })
            },
        )
    }

    /// The words of each key of the table that none of `dials`, the keys of
    /// the dials, owns, sorted by key, with the nearest dial.
    #[must_use]
    pub fn unknown(&self, dials: &[&str]) -> Vec<String> {
        self.table
            .keys()
            .filter(|key| !dials.contains(&key.as_str()))
            .map(|key| {
                let line = format!("{key} under [tool.purba] is no setting of purba");
                match nearest(key, dials) {
                    Some(dial) => format!("{line}; the nearest is {dial}"),
                    None => line,
                }
            })
            .collect()
    }
}

/// The edits from `one` to `other`, letter by letter. An edit changes, adds or
/// drops one letter, or swaps two neighbours, and no letter moves twice. Case
/// counts.
#[must_use]
pub fn distance(one: &str, other: &str) -> usize {
    let one: Vec<char> = one.chars().collect();
    let other: Vec<char> = other.chars().collect();
    // `rows[i][j]` is the distance from the first `i` letters of `one` to the
    // first `j` letters of `other`.
    let mut rows: Vec<Vec<usize>> = (0..=one.len())
        .map(|i| {
            (0..=other.len())
                .map(|j| if i == 0 { j } else { i })
                .collect()
        })
        .collect();
    for (i, &letter) in one.iter().enumerate() {
        for (j, &against) in other.iter().enumerate() {
            let mut fewest = (rows[i][j + 1] + 1)
                .min(rows[i + 1][j] + 1)
                .min(rows[i][j] + usize::from(letter != against));
            if i > 0 && j > 0 && letter == other[j - 1] && one[i - 1] == against {
                fewest = fewest.min(rows[i - 1][j - 1] + 1);
            }
            rows[i + 1][j + 1] = fewest;
        }
    }
    rows[one.len()][other.len()]
}

/// The first of `names` that the fewest edits of [`distance`] reach from
/// `name`, or `None` when `names` is empty.
#[must_use]
pub fn nearest<'a>(name: &str, names: &[&'a str]) -> Option<&'a str> {
    names
        .iter()
        .copied()
        .min_by_key(|candidate| distance(name, candidate))
}

/// What the walk up from the working folder to the project's file finds.
#[derive(Debug, Clone, PartialEq)]
pub enum Walk {
    /// The nearest `pyproject.toml` that holds `[tool.purba]`, and the table
    /// under `[tool.purba]`.
    Found {
        /// The file, and its folder, the root.
        file: Met,
        /// The table under `[tool.purba]`.
        settings: toml::Table,
    },
    /// A `pyproject.toml` on the walk that is not TOML.
    Unread {
        /// The file, by its path from the working folder.
        file: PathBuf,
        /// The words of its error.
        words: String,
    },
    /// No `pyproject.toml` on the walk holds `[tool.purba]`.
    Missing {
        /// The nearest `pyproject.toml` that holds `[project]`.
        project: Option<Met>,
        /// The folder that holds `.git`, where the walk stopped.
        git: Option<Met>,
        /// The working folder, canonical.
        start: PathBuf,
    },
}

/// A file or a folder that the walk met.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Met {
    /// Its path from the working folder: `.` for the working folder itself.
    pub path: PathBuf,
    /// Its folder, canonical: the folder itself, or the folder of the file.
    pub folder: PathBuf,
}

/// Walks up from `folder`, the working folder, to the first `pyproject.toml`
/// that holds `[tool.purba]`, and stops after the folder that holds `.git`, a
/// folder or a file.
#[must_use]
pub fn walk(folder: &Path) -> Walk {
    let start = folder.canonicalize().unwrap_or_else(|_| folder.to_owned());
    let mut up = PathBuf::new();
    let mut project: Option<Met> = None;
    for current in start.ancestors() {
        let file = current.join("pyproject.toml");
        if file.is_file() {
            let table = match read(&file) {
                Ok(table) => table,
                Err(words) => {
                    return Walk::Unread {
                        file: up.join("pyproject.toml"),
                        words,
                    };
                }
            };
            let met = || Met {
                path: up.join("pyproject.toml"),
                folder: current.to_owned(),
            };
            if let Some(settings) = inner(&table, &["tool", "purba"]) {
                return Walk::Found {
                    file: met(),
                    settings: settings.clone(),
                };
            }
            if project.is_none() && inner(&table, &["project"]).is_some() {
                project = Some(met());
            }
        }
        if current.join(".git").symlink_metadata().is_ok() {
            return Walk::Missing {
                project,
                git: Some(Met {
                    path: shown(&up),
                    folder: current.to_owned(),
                }),
                start,
            };
        }
        up.push("..");
    }
    Walk::Missing {
        project,
        git: None,
        start,
    }
}

/// `up` as a folder: `.` for the working folder.
fn shown(up: &Path) -> PathBuf {
    if up.as_os_str().is_empty() {
        PathBuf::from(".")
    } else {
        up.to_owned()
    }
}

/// The table that the TOML file at `file` holds, or the words of its error.
fn read(file: &Path) -> Result<toml::Table, String> {
    let text = fs::read_to_string(file).map_err(|error| error.to_string())?;
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
        format!(
            "TOML parse error at line {line}, column {column}: {}",
            error.message()
        )
    })
}

/// The table that `table` holds at the keys of `keys`, one inside the other.
fn inner<'a>(table: &'a toml::Table, keys: &[&str]) -> Option<&'a toml::Table> {
    let mut table = table;
    for key in keys {
        table = table.get(*key).and_then(toml::Value::as_table)?;
    }
    Some(table)
}
