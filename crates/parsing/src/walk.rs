//! One walk of ruff's tree that does not overflow the stack.

use ruff_python_ast::visitor::source_order::{self, SourceOrderVisitor};
use ruff_python_ast::{
    Alias, Comprehension, ExceptHandler, Expr, InterpolatedStringElement, Pattern, Stmt,
};

/// A node that can hold other nodes at any depth.
#[derive(Clone, Copy)]
pub enum Node<'a> {
    Stmt(&'a Stmt),
    Expr(&'a Expr),
    Pattern(&'a Pattern),
    Element(&'a InterpolatedStringElement),
}

/// How far the walk enters a node.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Enter {
    /// Each node it holds.
    All,
    /// Of a `def`, a `class` or a `lambda`: its decorators, defaults,
    /// annotations and bases, and not its body or its type parameters. `CPython`
    /// 3.12 and 3.13 run each of them where the `def` or the `class` stands,
    /// except an annotation under `from __future__ import annotations`. Of a
    /// comprehension: its iterables and conditions, and not its targets. Of any
    /// other node: each node it holds.
    Head,
    /// No node it holds.
    Nothing,
}

/// The arms of one visitor: what it does at each node that the walk meets.
/// Each arm that a visitor does not give enters each node.
pub trait Arms<'a> {
    /// Meets `statement`, and says how far the walk enters it.
    fn stmt(&mut self, _statement: &'a Stmt) -> Enter {
        Enter::All
    }

    /// Meets `expr`, and says how far the walk enters it.
    fn expr(&mut self, _expr: &'a Expr) -> Enter {
        Enter::All
    }

    /// Meets `comprehension`, and says how far the walk enters it.
    fn comprehension(&mut self, _comprehension: &'a Comprehension) -> Enter {
        Enter::All
    }

    /// Meets `pattern`. The walk enters each node it holds.
    fn pattern(&mut self, _pattern: &'a Pattern) {}

    /// Meets `handler`. The walk enters each node it holds.
    fn except_handler(&mut self, _handler: &'a ExceptHandler) {}

    /// Meets `alias`.
    fn alias(&mut self, _alias: &'a Alias) {}
}

/// Walks from `roots`, and gives each statement, expression and pattern to
/// `arms` in source order.
///
/// The arm of a comprehension, a handler or an alias runs when the walk enters
/// the node that holds it, before the nodes that come before it in that node.
///
/// A walk that recurses once for each level of nesting overflows the stack on
/// a module that nests deeper than the stack. This walk keeps the nodes still
/// to meet on a stack of its own, and ruff's visitor goes one level at a time.
pub fn walk<'a>(roots: impl IntoIterator<Item = Node<'a>>, arms: &mut impl Arms<'a>) {
    let mut pending: Vec<Node<'a>> = roots.into_iter().collect();
    pending.reverse();
    let mut children = Children {
        arms,
        found: Vec::new(),
    };
    while let Some(node) = pending.pop() {
        children.enter(node);
        pending.extend(children.found.drain(..).rev());
    }
}

/// The nodes that one node holds, as far as its arm enters it.
struct Children<'a, 'r, A> {
    arms: &'r mut A,
    found: Vec<Node<'a>>,
}

impl<'a, A: Arms<'a>> Children<'a, '_, A> {
    fn enter(&mut self, node: Node<'a>) {
        match node {
            Node::Stmt(statement) => match (self.arms.stmt(statement), statement) {
                (Enter::Nothing, _) => {}
                (Enter::Head, Stmt::FunctionDef(function)) => {
                    for decorator in &function.decorator_list {
                        self.visit_decorator(decorator);
                    }
                    self.visit_parameters(&function.parameters);
                    if let Some(returns) = &function.returns {
                        self.visit_annotation(returns);
                    }
                }
                (Enter::Head, Stmt::ClassDef(class)) => {
                    for decorator in &class.decorator_list {
                        self.visit_decorator(decorator);
                    }
                    if let Some(arguments) = &class.arguments {
                        self.visit_arguments(arguments);
                    }
                }
                _ => source_order::walk_stmt(self, statement),
            },
            Node::Expr(expr) => match (self.arms.expr(expr), expr) {
                (Enter::Nothing, _) => {}
                (Enter::Head, Expr::Lambda(lambda)) => {
                    if let Some(parameters) = &lambda.parameters {
                        self.visit_parameters(parameters);
                    }
                }
                _ => source_order::walk_expr(self, expr),
            },
            Node::Pattern(pattern) => {
                self.arms.pattern(pattern);
                source_order::walk_pattern(self, pattern);
            }
            Node::Element(element) => source_order::walk_interpolated_string_element(self, element),
        }
    }
}

// Each visit of a node that can nest pushes it, in place of a recursion.
impl<'a, A: Arms<'a>> SourceOrderVisitor<'a> for Children<'a, '_, A> {
    fn visit_stmt(&mut self, statement: &'a Stmt) {
        self.found.push(Node::Stmt(statement));
    }

    fn visit_expr(&mut self, expr: &'a Expr) {
        self.found.push(Node::Expr(expr));
    }

    fn visit_pattern(&mut self, pattern: &'a Pattern) {
        self.found.push(Node::Pattern(pattern));
    }

    fn visit_interpolated_string_element(&mut self, element: &'a InterpolatedStringElement) {
        self.found.push(Node::Element(element));
    }

    fn visit_comprehension(&mut self, comprehension: &'a Comprehension) {
        match self.arms.comprehension(comprehension) {
            Enter::All => source_order::walk_comprehension(self, comprehension),
            Enter::Head => {
                self.visit_expr(&comprehension.iter);
                for condition in &comprehension.ifs {
                    self.visit_expr(condition);
                }
            }
            Enter::Nothing => {}
        }
    }

    fn visit_except_handler(&mut self, handler: &'a ExceptHandler) {
        self.arms.except_handler(handler);
        source_order::walk_except_handler(self, handler);
    }

    fn visit_alias(&mut self, alias: &'a Alias) {
        self.arms.alias(alias);
    }
}
