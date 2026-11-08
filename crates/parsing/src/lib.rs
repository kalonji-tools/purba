//! Parsing: reads a test module and turns its text into ruff's AST.

use std::cell::RefCell;
use std::fmt;
use std::fs;
use std::io;
use std::mem;
use std::path::Path;

use ruff_python_ast::token::{Token, TokenKind};
use ruff_python_ast::visitor::transformer::{
    Transformer, walk_expr, walk_interpolated_string_element, walk_pattern, walk_stmt,
};
use ruff_python_ast::{
    AtomicNodeIndex, Expr, ExprNoneLiteral, InterpolatedStringElement,
    InterpolatedStringLiteralElement, ModModule, Pattern, PatternMatchAs, PySourceType, Stmt,
    StmtPass,
};
use ruff_source_file::LineIndex;
use ruff_text_size::{Ranged, TextLen, TextRange, TextSize};

const BYTE_ORDER_MARK: char = '\u{feff}';

/// One module: its text, ruff's tree of that text, and where its trivia is.
pub struct Module {
    source: String,
    syntax: ModModule,
    trivia: Vec<TextRange>,
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
    pub const fn syntax(&self) -> &ModModule {
        &self.syntax
    }

    /// The range of each comment, and of the backslash of each line
    /// continuation, in source order. ruff's tree holds neither.
    #[must_use]
    pub fn trivia(&self) -> &[TextRange] {
        &self.trivia
    }

    /// The place of `offset`, a byte offset into the text.
    #[must_use]
    pub fn position(&self, offset: TextSize) -> Position {
        let place = LineIndex::from_source_text(&self.source).line_column(offset, &self.source);
        Position {
            line: place.line.get(),
            column: place.column.get(),
        }
    }
}

impl Drop for Module {
    // The drop glue of ruff's tree recurses once for each level of nesting,
    // and a module can nest deeper than the stack.
    // https://doc.rust-lang.org/reference/destructors.html
    fn drop(&mut self) {
        let detached = Detached::default();
        detached
            .stmts
            .borrow_mut()
            .extend(mem::take(&mut self.syntax.body));
        while detached.free_last(&detached.exprs, walk_expr)
            || detached.free_last(&detached.stmts, walk_stmt)
            || detached.free_last(&detached.patterns, walk_pattern)
            || detached.free_last(&detached.elements, walk_interpolated_string_element)
        {}
    }
}

/// The nodes taken out of a tree that is being freed.
#[derive(Default)]
struct Detached {
    exprs: RefCell<Vec<Expr>>,
    stmts: RefCell<Vec<Stmt>>,
    patterns: RefCell<Vec<Pattern>>,
    elements: RefCell<Vec<InterpolatedStringElement>>,
}

impl Detached {
    /// Moves the children of the last node of `nodes` onto the lists, and frees
    /// that node. Gives `false` when `nodes` is empty.
    fn free_last<T>(&self, nodes: &RefCell<Vec<T>>, walk: fn(&Self, &mut T)) -> bool {
        let last = nodes.borrow_mut().pop();
        let Some(mut node) = last else {
            return false;
        };
        walk(self, &mut node);
        true
    }
}

impl Transformer for Detached {
    fn visit_expr(&self, expr: &mut Expr) {
        let leaf = Expr::NoneLiteral(ExprNoneLiteral::default());
        self.exprs.borrow_mut().push(mem::replace(expr, leaf));
    }

    fn visit_stmt(&self, stmt: &mut Stmt) {
        let leaf = Stmt::Pass(StmtPass {
            node_index: AtomicNodeIndex::default(),
            range: TextRange::default(),
        });
        self.stmts.borrow_mut().push(mem::replace(stmt, leaf));
    }

    fn visit_pattern(&self, pattern: &mut Pattern) {
        let leaf = Pattern::MatchAs(PatternMatchAs {
            node_index: AtomicNodeIndex::default(),
            range: TextRange::default(),
            pattern: None,
            name: None,
        });
        self.patterns.borrow_mut().push(mem::replace(pattern, leaf));
    }

    fn visit_interpolated_string_element(&self, element: &mut InterpolatedStringElement) {
        let leaf = InterpolatedStringElement::Literal(InterpolatedStringLiteralElement {
            range: TextRange::default(),
            node_index: AtomicNodeIndex::default(),
            value: Box::default(),
        });
        self.elements.borrow_mut().push(mem::replace(element, leaf));
    }
}

/// Why a module gives no tree.
#[derive(Debug)]
pub enum Error {
    /// The file cannot be read.
    Read(io::Error),
    /// ruff found a syntax error in the text.
    Syntax(SyntaxError),
}

/// The first syntax error that ruff lists for a module.
#[derive(Debug)]
pub struct SyntaxError {
    message: String,
    start: Position,
    end: Position,
}

impl SyntaxError {
    /// ruff's words for the error.
    #[must_use]
    pub fn message(&self) -> &str {
        &self.message
    }

    /// Where the error starts.
    #[must_use]
    pub const fn start(&self) -> Position {
        self.start
    }

    /// Where the error ends: the first position after it.
    #[must_use]
    pub const fn end(&self) -> Position {
        self.end
    }
}

impl fmt::Display for SyntaxError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "SyntaxError: {}", self.message)
    }
}

/// A place in the text of a module, after a leading byte order mark.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Position {
    line: usize,
    column: usize,
}

impl Position {
    /// The line, from 1.
    #[must_use]
    pub const fn line(self) -> usize {
        self.line
    }

    /// The column, from 1. It counts Unicode scalars.
    #[must_use]
    pub const fn column(self) -> usize {
        self.column
    }
}

/// Reads the file at `path` and parses it.
///
/// # Errors
///
/// When the file cannot be read, or declares an encoding other than UTF-8,
/// the error is [`Error::Read`]. When ruff finds a syntax error in the file,
/// the error is [`Error::Syntax`], and it holds the first one that ruff lists.
///
/// # Panics
///
/// In a build with the feature `fault`, when `PURBA_FAULT` is `parse`.
pub fn parse(path: &Path) -> Result<Module, Error> {
    if cfg!(feature = "fault")
        && std::env::var_os("PURBA_FAULT").is_some_and(|setting| setting == "parse")
    {
        panic!("PURBA_FAULT=parse");
    }
    let bytes = fs::read(path).map_err(Error::Read)?;
    let marked = bytes.starts_with(b"\xef\xbb\xbf");
    if let Some(name) = declared_encoding(&bytes).filter(|name| !means_utf_8(name, marked)) {
        return Err(Error::Read(io::Error::new(
            io::ErrorKind::InvalidData,
            format!("it declares the encoding {name}, and purba reads only UTF-8"),
        )));
    }
    let source = String::from_utf8(bytes).map_err(|_| {
        Error::Read(io::Error::new(
            io::ErrorKind::InvalidData,
            "stream did not contain valid UTF-8",
        ))
    })?;
    parse_source(source)
}

/// The encoding that the module declares, if it declares one.
//
// CPython reads a declaration only in a line that holds only a comment. It
// reads line 1, and line 2 when line 1 is blank or holds a comment with no
// declaration. A line ends at `\r\n`, `\r` or `\n`.
// https://github.com/python/cpython/blob/v3.14.0/Parser/tokenizer/helpers.c#L212-L239
// https://github.com/python/cpython/blob/v3.14.0/Parser/tokenizer/helpers.c#L334-L380
// https://github.com/python/cpython/blob/v3.14.0/Parser/tokenizer/helpers.c#L386-L434
fn declared_encoding(bytes: &[u8]) -> Option<String> {
    let text = bytes.strip_prefix(b"\xef\xbb\xbf").unwrap_or(bytes);
    for line in lines(text).take(2) {
        let start = skip(line, b" \t\x0c");
        match start.first() {
            Some(b'#') => {
                if let Some(name) = coding_name(start) {
                    return Some(name);
                }
            }
            None => {}
            Some(_) => return None,
        }
    }
    None
}

/// The lines of `text`, each without its end: `\r\n`, `\r` or `\n`.
fn lines(mut text: &[u8]) -> impl Iterator<Item = &[u8]> {
    std::iter::from_fn(move || {
        if text.is_empty() {
            return None;
        }
        let end = text
            .iter()
            .position(|&byte| byte == b'\n' || byte == b'\r')
            .unwrap_or(text.len());
        let (line, rest) = text.split_at(end);
        text = rest
            .strip_prefix(b"\r\n")
            .or_else(|| rest.get(1..))
            .unwrap_or_default();
        Some(line)
    })
}

/// `bytes` after each leading byte that is one of `these`.
fn skip<'a>(bytes: &'a [u8], these: &[u8]) -> &'a [u8] {
    let start = bytes
        .iter()
        .position(|byte| !these.contains(byte))
        .unwrap_or(bytes.len());
    &bytes[start..]
}

/// The name after the first `coding:` or `coding=` of a comment.
fn coding_name(comment: &[u8]) -> Option<String> {
    (0..comment.len()).find_map(|at| {
        let after = comment[at..].strip_prefix(b"coding")?;
        let (&mark, rest) = after.split_first()?;
        if mark != b':' && mark != b'=' {
            return None;
        }
        let rest = skip(rest, b" \t");
        let end = rest
            .iter()
            .position(|&byte| !(byte.is_ascii_alphanumeric() || b"-_.".contains(&byte)))
            .unwrap_or(rest.len());
        (end > 0).then(|| String::from_utf8_lossy(&rest[..end]).into_owned())
    })
}

/// Whether a module that declares `name` is decoded with UTF-8. `marked` says
/// whether the file starts with a byte order mark.
//
// The tokenizer of CPython takes `utf-8` and `utf-8-…`, in any case and with
// `_` for `-`. After a mark, it refuses any other name. Without a mark, it
// gives any other name to `codecs`. The names that follow are the codecs
// `utf_8` and `utf_8_sig`, which both decode UTF-8, and the aliases of `utf_8`.
// https://github.com/python/cpython/blob/v3.14.0/Parser/tokenizer/helpers.c#L304-L320
// https://github.com/python/cpython/blob/v3.14.0/Parser/tokenizer/helpers.c#L423-L431
// https://github.com/python/cpython/blob/v3.14.0/Lib/encodings/aliases.py#L540-L546
fn means_utf_8(name: &str, marked: bool) -> bool {
    let tokenizer: String = name
        .chars()
        .take(12)
        .map(|c| {
            if c == '_' {
                '-'
            } else {
                c.to_ascii_lowercase()
            }
        })
        .collect();
    if tokenizer == "utf-8" || tokenizer.starts_with("utf-8-") {
        return true;
    }
    if marked {
        return false;
    }
    // `codecs` lowers the case, and turns each run of characters other than
    // letters, digits and `.` into one `_`. It looks up an alias as that name,
    // or with `.` as `_`, and a module only as that name, with no `.`.
    // https://github.com/python/cpython/blob/v3.14.0/Objects/unicodeobject.c#L3596-L3640
    // https://github.com/python/cpython/blob/v3.14.0/Lib/encodings/__init__.py#L71-L95
    let mut codec = String::new();
    let mut gap = false;
    for c in name.chars() {
        if c.is_ascii_alphanumeric() || c == '.' {
            if gap && !codec.is_empty() {
                codec.push('_');
            }
            gap = false;
            codec.push(c.to_ascii_lowercase());
        } else {
            gap = true;
        }
    }
    let alias = |key: &str| {
        matches!(
            key,
            "u8" | "utf" | "utf8" | "utf8_ucs2" | "utf8_ucs4" | "cp65001"
        )
    };
    matches!(codec.as_str(), "utf_8" | "utf_8_sig")
        || alias(&codec)
        || alias(&codec.replace('.', "_"))
}

/// Parses `source`, the text of a module, as [`parse`] parses the text of a
/// file.
///
/// # Errors
///
/// When ruff finds a syntax error in it.
#[expect(
    clippy::disallowed_methods,
    reason = "purba-parsing is the one caller of ruff's parser"
)]
pub fn parse_source(mut source: String) -> Result<Module, Error> {
    // CPython ignores this mark at the start of a file that is UTF-8.
    // https://docs.python.org/3.12/reference/lexical_analysis.html#encoding-declarations
    if source.starts_with(BYTE_ORDER_MARK) {
        source.remove(0);
    }
    // `parse_module` drops the tree it recovered inside ruff, and that drop
    // recurses once for each level of nesting. `Module`'s drop does not.
    // https://github.com/astral-sh/ruff/blob/0.17.0/crates/ruff_python_parser/src/lib.rs#L477-L483
    let parsed = ruff_python_parser::parse_unchecked_source(&source, PySourceType::Python);
    let first = parsed
        .errors()
        .first()
        .map(|error| (error.error.to_string(), error.location));
    let trivia = trivia(&source, parsed.tokens());
    let module = Module {
        syntax: parsed.into_syntax(),
        source,
        trivia,
    };
    match first {
        None => Ok(module),
        Some((message, location)) => Err(Error::Syntax(SyntaxError {
            message,
            start: module.position(location.start()),
            end: module.position(location.end()),
        })),
    }
}

/// The range of each comment token, and of each backslash between two tokens.
fn trivia(source: &str, tokens: &[Token]) -> Vec<TextRange> {
    // Outside a token, a backslash can only start a line continuation.
    // https://docs.python.org/3.12/reference/lexical_analysis.html#explicit-line-joining
    let mut trivia = Vec::new();
    let mut end = TextSize::default();
    for token in tokens {
        let gap = TextRange::new(end, token.start());
        trivia.extend(source[gap].match_indices('\\').map(|(at, _)| {
            TextRange::at(
                gap.start() + TextSize::try_from(at).unwrap_or_default(),
                '\\'.text_len(),
            )
        }));
        if token.kind() == TokenKind::Comment {
            trivia.push(token.range());
        }
        end = token.end();
    }
    trivia
}
