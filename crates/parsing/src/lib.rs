//! Parsing: reads a test module and turns its text into ruff's AST.

use std::cell::RefCell;
use std::fs;
use std::io;
use std::mem;
use std::path::Path;

use ruff_python_ast::visitor::transformer::{
    Transformer, walk_expr, walk_interpolated_string_element, walk_pattern, walk_stmt,
};
use ruff_python_ast::{
    AtomicNodeIndex, Expr, ExprNoneLiteral, InterpolatedStringElement,
    InterpolatedStringLiteralElement, ModModule, Pattern, PatternMatchAs, PySourceType, Stmt,
    StmtPass,
};
use ruff_text_size::TextRange;

const BYTE_ORDER_MARK: char = '\u{feff}';

/// One module: its text, and ruff's tree of that text.
pub struct Module {
    source: String,
    syntax: ModModule,
    has_syntax_error: bool,
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

    /// Whether ruff found a syntax error in the text.
    #[must_use]
    pub const fn has_syntax_error(&self) -> bool {
        self.has_syntax_error
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
    let has_syntax_error = !parsed.errors().is_empty();
    Ok(Module {
        source,
        syntax: parsed.into_syntax(),
        has_syntax_error,
    })
}
