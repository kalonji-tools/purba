//! Selection: names the tests of a parsed module, and imports nothing.

use std::collections::HashSet;
use std::fmt;
use std::path::{Path, PathBuf};

use purba_parsing::Module;
use ruff_python_ast::Stmt;

/// A test's identity. Only selection constructs one.
///
/// ```compile_fail,E0451
/// let id = purba_selection::TestId { path: "test_a.py".into(), function: "test_one".into() };
/// ```
///
/// ```compile_fail,E0616
/// fn forge(id: &mut purba_selection::TestId) { id.path = "test_b.py".into(); }
/// ```
///
/// ```compile_fail,E0616
/// fn forge(id: &mut purba_selection::TestId) { id.function = "test_two".into(); }
/// ```
pub struct TestId {
    path: PathBuf,
    function: String,
}

impl TestId {
    /// The path of the test's module, as the tester typed it.
    #[must_use]
    pub fn path(&self) -> &Path {
        &self.path
    }

    /// The name of the test's function.
    #[must_use]
    pub fn function(&self) -> &str {
        &self.function
    }
}

impl fmt::Display for TestId {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{}::{}", self.path.display(), self.function)
    }
}

/// Each test of `module`, in source order. `path` is the module's path as the
/// tester typed it. A name defined twice is one test, at its first definition:
/// import binds the name there and keeps the last body.
#[must_use]
pub fn select(path: &Path, module: &Module) -> Vec<TestId> {
    let mut seen = HashSet::new();
    module
        .syntax()
        .body
        .iter()
        .filter_map(|statement| match statement {
            Stmt::FunctionDef(function)
                if function.name.starts_with("test") && seen.insert(function.name.as_str()) =>
            {
                Some(TestId {
                    path: path.to_path_buf(),
                    function: function.name.to_string(),
                })
            }
            _ => None,
        })
        .collect()
}
