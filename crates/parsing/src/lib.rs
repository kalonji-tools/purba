//! Parsing: reads a test module and turns its text into ruff's AST.

use std::fs;
use std::io;
use std::path::Path;

use ruff_python_ast::{ModModule, PySourceType};
use ruff_python_parser::Parsed;

/// One module: the text that was read, and ruff's tree of that text.
pub struct Module {
    source: String,
    parsed: Parsed<ModModule>,
}

impl Module {
    /// The text of the module, as it was read.
    #[must_use]
    pub fn source(&self) -> &str {
        &self.source
    }

    /// ruff's tree of the text.
    #[must_use]
    pub fn syntax(&self) -> &ModModule {
        self.parsed.syntax()
    }
}

/// Reads the file at `path` and parses it.
///
/// A syntax error does not fail the parse: ruff still returns a tree.
///
/// # Errors
///
/// When the file cannot be read.
#[expect(
    clippy::disallowed_methods,
    reason = "purba-parsing is the one caller of ruff's parser"
)]
pub fn parse(path: &Path) -> io::Result<Module> {
    let source = fs::read_to_string(path)?;
    let parsed = ruff_python_parser::parse_unchecked_source(&source, PySourceType::Python);
    Ok(Module { source, parsed })
}
