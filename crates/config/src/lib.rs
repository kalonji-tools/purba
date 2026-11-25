//! The crate of the configuration concern: resolves each dial from its levels,
//! keeps the level of each value, and names each key that no dial owns.

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

/// The level that a value came from.
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum Level {
    /// No level sets the dial, so it takes its default.
    Default,
    /// The project's file, under `[tool.purba]`.
    Project,
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
