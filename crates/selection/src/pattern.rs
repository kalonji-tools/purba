//! The patterns of a test file's name, and `test-files`, the dial that sets
//! them.

use std::ffi::OsStr;
use std::fmt;

use purba_config::{Dial, nearest};

/// A pattern of the closed list that a test file's name may match.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Pattern {
    /// `test_*.py`: a name that starts with `test_` and ends with `.py`.
    TestPrefix,
    /// `*_test.py`: a name that ends with `_test.py`, after at least one
    /// letter.
    TestSuffix,
    /// `tests.py`.
    Tests,
    /// `test.py`.
    Test,
}

impl Pattern {
    /// The closed list, in the order its errors name it.
    const ALL: [Self; 4] = [Self::TestPrefix, Self::TestSuffix, Self::Tests, Self::Test];

    /// The pattern as the project's file writes it.
    const fn text(self) -> &'static str {
        match self {
            Self::TestPrefix => "test_*.py",
            Self::TestSuffix => "*_test.py",
            Self::Tests => "tests.py",
            Self::Test => "test.py",
        }
    }

    /// Whether the file name `name` matches the pattern.
    #[must_use]
    pub fn matches(self, name: &OsStr) -> bool {
        let name = name.as_encoded_bytes();
        match self {
            Self::TestPrefix => {
                name.len() >= b"test_.py".len()
                    && name.starts_with(b"test_")
                    && name.ends_with(b".py")
            }
            Self::TestSuffix => name.len() > b"_test.py".len() && name.ends_with(b"_test.py"),
            Self::Tests | Self::Test => name == self.text().as_bytes(),
        }
    }
}

impl fmt::Display for Pattern {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.text())
    }
}

/// The patterns that a run takes: a test file's name matches one of them.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Patterns(Vec<Pattern>);

impl Patterns {
    /// Whether the file name `name` matches a pattern of the list.
    pub(crate) fn matches(&self, name: &OsStr) -> bool {
        self.0.iter().any(|pattern| pattern.matches(name))
    }

    /// Each pattern of the list once, in the order of the list.
    pub(crate) fn each(&self) -> Vec<Pattern> {
        let mut each = Vec::with_capacity(self.0.len());
        for &pattern in &self.0 {
            if !each.contains(&pattern) {
                each.push(pattern);
            }
        }
        each
    }
}

impl Default for Patterns {
    /// `test_*.py` alone.
    fn default() -> Self {
        Self(vec![Pattern::TestPrefix])
    }
}

impl From<Vec<Pattern>> for Patterns {
    fn from(patterns: Vec<Pattern>) -> Self {
        Self(patterns)
    }
}

/// `test-files` under `[tool.purba]`: the patterns of a test file's name.
pub static TEST_FILES: Dial<Patterns> = Dial::new("test-files", read, Patterns::default);

/// The words that `items` hold, joined by `, ` and a last ` and `.
pub fn listed(items: &[impl fmt::Display]) -> String {
    let mut text = String::new();
    for (at, item) in items.iter().enumerate() {
        if at > 0 {
            text.push_str(if at + 1 == items.len() { " and " } else { ", " });
        }
        text.push_str(&item.to_string());
    }
    text
}

/// The patterns that the value of `test-files` lists, or the words of each
/// error of the value.
fn read(value: &toml::Value) -> Result<Patterns, Vec<String>> {
    if let toml::Value::Array(items) = value {
        return read_list(items);
    }
    Err(vec![format!(
        r#"test-files under [tool.purba] is {}; write a list, such as ["*_test.py"]"#,
        kind(value)
    )])
}

/// The kind of `value`, in the words of an error of a dial.
pub const fn kind(value: &toml::Value) -> &'static str {
    match value {
        toml::Value::Array(_) => "a list",
        toml::Value::String(_) => "a string",
        toml::Value::Integer(_) | toml::Value::Float(_) => "a number",
        toml::Value::Boolean(_) => "a boolean",
        toml::Value::Datetime(_) => "a date",
        toml::Value::Table(_) => "a table",
    }
}

/// The patterns that `items` name, or the words of each item outside the
/// closed list, or of an empty list.
fn read_list(items: &[toml::Value]) -> Result<Patterns, Vec<String>> {
    let closed = listed(&Pattern::ALL);
    if items.is_empty() {
        return Err(vec![format!(
            "test-files under [tool.purba] is an empty list, so no file is a test file; \
             list at least one of {closed}"
        )]);
    }
    let texts = Pattern::ALL.map(Pattern::text);
    let mut patterns = Vec::with_capacity(items.len());
    let mut errors = Vec::new();
    for item in items {
        let named = item.as_str().and_then(|text| {
            Pattern::ALL
                .into_iter()
                .find(|pattern| pattern.text() == text)
        });
        if let Some(pattern) = named {
            patterns.push(pattern);
            continue;
        }
        let shown = item
            .as_str()
            .map_or_else(|| item.to_string(), str::to_owned);
        let near = nearest(&shown, &texts).unwrap_or_default();
        errors.push(format!(
            "test-files under [tool.purba] holds {shown}, which is none of {closed}; \
             the nearest is {near}"
        ));
    }
    if errors.is_empty() {
        Ok(Patterns(patterns))
    } else {
        Err(errors)
    }
}
