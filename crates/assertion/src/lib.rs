//! Assertion: rewrites each assert of a module so that it records that it ran.

use purba_parsing::Module;
use ruff_python_ast::statement_visitor::{StatementVisitor, walk_stmt};
use ruff_python_ast::{Expr, Stmt, StmtAssert};
use ruff_text_size::Ranged;

/// The name that each marked assert calls.
const RAN: &str = "_purba_ran";

/// The rewritten text of one module, and the source text of each of its asserts.
pub struct Rewrite {
    text: String,
    asserts: Vec<String>,
}

impl Rewrite {
    /// The module's text, with each assert marked.
    #[must_use]
    pub fn text(&self) -> &str {
        &self.text
    }

    /// The source text of each assert, in source order. The index of an assert
    /// here is the index that its mark records.
    #[must_use]
    pub fn asserts(&self) -> &[String] {
        &self.asserts
    }
}

/// Each assert of a module, in source order, at any depth.
struct Asserts<'a> {
    found: Vec<&'a StmtAssert>,
}

impl<'a> StatementVisitor<'a> for Asserts<'a> {
    fn visit_stmt(&mut self, statement: &'a Stmt) {
        if let Stmt::Assert(assert) = statement {
            self.found.push(assert);
        }
        walk_stmt(self, statement);
    }
}

/// Marks each assert of `module`: `assert x` becomes `assert _purba_ran(i, x)`,
/// where `i` counts the asserts of the module in source order, from 0.
#[must_use]
pub fn rewrite(module: &Module) -> Rewrite {
    let source = module.source();
    let mut visitor = Asserts { found: Vec::new() };
    visitor.visit_body(&module.syntax().body);

    let mut rewritten = String::with_capacity(source.len());
    let mut asserts = Vec::with_capacity(visitor.found.len());
    let mut copied = 0;
    for (index, assert) in visitor.found.iter().enumerate() {
        let condition = assert.test.range();
        // ruff's range of `(yield x)` leaves out the parentheses, and a yield
        // inside an expression is a `yield_atom`, which needs them.
        // https://docs.python.org/3.12/reference/expressions.html#yield-expressions
        let bare = matches!(*assert.test, Expr::Yield(_) | Expr::YieldFrom(_));
        let (open, close) = if bare { ("(", ")") } else { ("", "") };
        rewritten.push_str(&source[copied..condition.start().to_usize()]);
        rewritten.push_str(RAN);
        rewritten.push('(');
        rewritten.push_str(&index.to_string());
        rewritten.push_str(", ");
        rewritten.push_str(open);
        rewritten.push_str(&source[condition]);
        rewritten.push_str(close);
        rewritten.push(')');
        copied = condition.end().to_usize();
        asserts.push(source[assert.range()].to_owned());
    }
    rewritten.push_str(&source[copied..]);
    Rewrite {
        text: rewritten,
        asserts,
    }
}
