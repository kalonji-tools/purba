//! The check of a name near `test`.

use std::collections::HashSet;
use std::fmt;
use std::path::{Path, PathBuf};

use purba_parsing::Facts;
use ruff_python_ast::Stmt;
use ruff_text_size::{Ranged, TextSize};

/// A function whose name is near `test`, which is a project error.
///
/// Its name does not start with `test`, and its first word is at most one edit
/// from `test`. Only selection constructs one. It displays its kind and its
/// words.
///
/// ```compile_fail,E0451
/// let near = purba_selection::NameNear { path: "test_a.py".into(), name: "tset".into() };
/// ```
pub struct NameNear {
    path: PathBuf,
    name: String,
}

impl NameNear {
    /// The path of the function's module, from the root.
    #[must_use]
    pub fn path(&self) -> &Path {
        &self.path
    }

    /// The name of the function.
    #[must_use]
    pub fn name(&self) -> &str {
        &self.name
    }
}

impl fmt::Display for NameNear {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "[name-near] {} is not a test, and its name is near the prefix test",
            self.name
        )
    }
}

/// Each name near `test` of the module at `path`, at its first `def`, in
/// source order. A `def` counts at the module's level, or in a block of a
/// statement there. A method and a `def` in a `def` do not count.
pub fn names_near(path: &Path, facts: &Facts<'_>) -> Vec<(TextSize, NameNear)> {
    let mut seen: HashSet<&str> = HashSet::new();
    facts
        .statements()
        .iter()
        .filter_map(|statement| match statement {
            Stmt::FunctionDef(function) => Some(function),
            _ => None,
        })
        .filter(|function| is_near(&function.name) && seen.insert(function.name.as_str()))
        .map(|function| {
            (
                function.start(),
                NameNear {
                    path: path.to_path_buf(),
                    name: function.name.to_string(),
                },
            )
        })
        .collect()
}

/// Whether `name` does not start with `test`, and its first word, which ends
/// at its first `_`, is at most one edit from `test` when case is ignored.
fn is_near(name: &str) -> bool {
    let word = name.split('_').next().unwrap_or_default();
    !name.starts_with("test") && purba_config::distance(&word.to_lowercase(), "test") <= 1
}
