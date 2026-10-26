//! Parsing: reads a test module and turns its text into ruff's AST.

use std::fs;
use std::io;
use std::path::Path;

use ruff_python_ast::{ModModule, PySourceType};
use ruff_python_parser::Parsed;

const BYTE_ORDER_MARK: char = '\u{feff}';

/// One module: its text, and ruff's tree of that text.
pub struct Module {
    source: String,
    parsed: Parsed<ModModule>,
}

impl Module {
    /// The text of the module, after a leading byte order mark, if the file
    /// has one.
    #[must_use]
    pub fn source(&self) -> &str {
        &self.source
    }

    /// ruff's tree of the text.
    #[must_use]
    pub fn syntax(&self) -> &ModModule {
        self.parsed.syntax()
    }

    /// Whether ruff found a syntax error in the text.
    #[must_use]
    pub fn has_syntax_error(&self) -> bool {
        !self.parsed.errors().is_empty()
    }
}

/// Reads the file at `path` and parses it.
///
/// A syntax error does not fail the parse: ruff still returns a tree, and
/// [`Module::has_syntax_error`] says so.
///
/// # Errors
///
/// When the file cannot be read.
#[expect(
    clippy::disallowed_methods,
    reason = "purba-parsing is the one caller of ruff's parser"
)]
pub fn parse(path: &Path) -> io::Result<Module> {
    let mut source = fs::read_to_string(path)?;
    // CPython ignores this mark at the start of a file that is UTF-8.
    // https://docs.python.org/3.12/reference/lexical_analysis.html#encoding-declarations
    if source.starts_with(BYTE_ORDER_MARK) {
        source.remove(0);
    }
    let parsed = ruff_python_parser::parse_unchecked_source(&source, PySourceType::Python);
    Ok(Module { source, parsed })
}
