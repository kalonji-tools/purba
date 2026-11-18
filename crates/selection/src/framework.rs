//! The imports of another test framework in a test module.

use std::collections::{HashMap, HashSet};

use ruff_python_ast::visitor::Visitor;
use ruff_python_ast::{Expr, InterpolatedStringElement, Pattern, Stmt, StmtFunctionDef};
use ruff_text_size::{Ranged, TextSize};

use super::lookup::{Lookup, Origin};
use super::{Node, walk_node};

/// The modules of another test framework: pytest, and the package that holds
/// its code.
const FRAMEWORKS: [&str; 2] = ["pytest", "_pytest"];

/// Each function that imports the module that its first argument names, by
/// its name through the imports.
/// <https://docs.python.org/3.12/library/functions.html#import__>
/// <https://docs.python.org/3.12/library/importlib.html#importlib.import_module>
/// <https://docs.python.org/3.12/library/importlib.html#importlib.util.find_spec>
/// <https://docs.python.org/3.12/library/pkgutil.html#pkgutil.resolve_name>
/// <https://docs.python.org/3.12/library/runpy.html#runpy.run_module>
/// <https://docs.python.org/3.12/library/importlib.resources.html#importlib.resources.files>
/// <https://docs.python.org/3.12/library/unittest.mock.html#patch>
const CALLS: [&str; 8] = [
    "builtins.__import__",
    "importlib.__import__",
    "importlib.import_module",
    "importlib.util.find_spec",
    "pkgutil.resolve_name",
    "runpy.run_module",
    "importlib.resources.files",
    "unittest.mock.patch",
];

/// The calls of [`CALLS`] that import the parent of a dotted name, and nothing
/// for a name with no dot.
/// <https://docs.python.org/3.12/library/importlib.html#importlib.util.find_spec>
/// <https://docs.python.org/3.12/library/unittest.mock.html#where-to-patch>
const DOTTED: [&str; 2] = ["importlib.util.find_spec", "unittest.mock.patch"];

/// Whether `name` is a module of another test framework, or a module under one.
fn names_a_framework(name: &str) -> bool {
    let top = name.split(['.', ':']).next().unwrap_or_default();
    FRAMEWORKS.contains(&top)
}

/// Each import of another test framework in `statements`, at any depth, by its
/// offset, with the module as the code writes it: an `import` or a
/// `from … import`, and a call of [`CALLS`] whose first argument is a string
/// that names it. A name resolves through an import of `statements`, then
/// through `imported` unless `locals` binds it. A name that no import, no local
/// and no binding of the module in `bound` binds is a builtin.
pub fn imports<'a>(
    statements: &'a [Stmt],
    locals: &HashSet<&'a str>,
    imported: &HashMap<&'a str, String>,
    bound: &HashSet<&'a str>,
) -> Vec<(TextSize, String)> {
    let mut found = Found {
        lookup: Lookup::new(statements, locals, imported, bound),
        imports: Vec::new(),
        pending: Vec::new(),
    };
    for statement in statements {
        found.visit_stmt(statement);
    }
    while let Some(node) = found.pending.pop() {
        walk_node(&mut found, node);
    }
    found.imports
}

/// Each import of another test framework that runs when `function` is made:
/// in a decorator, a default or an annotation. A name resolves as in
/// [`imports`], with no import and no local of its own.
pub fn made<'a>(
    function: &'a StmtFunctionDef,
    imported: &HashMap<&'a str, String>,
    bound: &HashSet<&'a str>,
) -> Vec<(TextSize, String)> {
    let none = HashSet::new();
    let mut found = Found {
        lookup: Lookup::new(&[], &none, imported, bound),
        imports: Vec::new(),
        pending: Vec::new(),
    };
    for decorator in &function.decorator_list {
        found.visit_decorator(decorator);
    }
    found.visit_parameters(&function.parameters);
    if let Some(returns) = &function.returns {
        found.visit_expr(returns);
    }
    while let Some(node) = found.pending.pop() {
        walk_node(&mut found, node);
    }
    found.imports
}

struct Found<'a, 'b> {
    lookup: Lookup<'a, 'b>,
    imports: Vec<(TextSize, String)>,
    pending: Vec<Node<'a>>,
}

impl Found<'_, '_> {
    /// The dotted name of `expr` through the imports, such as
    /// `importlib.import_module` for `load` after
    /// `from importlib import import_module as load`.
    fn qualified(&self, expr: &Expr) -> Option<String> {
        let mut parts = Vec::new();
        let mut current = expr;
        while let Expr::Attribute(attribute) = current {
            parts.push(attribute.attr.as_str());
            current = &attribute.value;
        }
        let Expr::Name(name) = current else {
            return None;
        };
        let root = name.id.as_str();
        let origin = match self.lookup.origin(root) {
            Origin::Own(origin) | Origin::Imported(origin) => origin.to_owned(),
            Origin::Local | Origin::Module => return None,
            Origin::Builtin => format!("builtins.{root}"),
        };
        let mut qualified = origin;
        for part in parts.iter().rev() {
            qualified.push('.');
            qualified.push_str(part);
        }
        Some(qualified)
    }
}

// As in `Bindings`, each visit pushes its node.
impl<'a> Visitor<'a> for Found<'a, '_> {
    fn visit_stmt(&mut self, statement: &'a Stmt) {
        match statement {
            Stmt::Import(import) => {
                for alias in &import.names {
                    if names_a_framework(alias.name.as_str()) {
                        self.imports
                            .push((statement.start(), alias.name.to_string()));
                    }
                }
            }
            Stmt::ImportFrom(import) if import.level == 0 => {
                if let Some(module) = &import.module
                    && names_a_framework(module.as_str())
                {
                    self.imports.push((statement.start(), module.to_string()));
                }
            }
            _ => {}
        }
        self.pending.push(Node::Stmt(statement));
    }

    fn visit_expr(&mut self, expr: &'a Expr) {
        if let Expr::Call(call) = expr
            && let Some(Expr::StringLiteral(first)) = call.arguments.args.first()
            && let Some(function) = self.qualified(&call.func)
            && CALLS.contains(&function.as_str())
        {
            let value = first.value.to_str();
            if names_a_framework(value)
                && (!DOTTED.contains(&function.as_str()) || value.contains('.'))
            {
                self.imports.push((expr.start(), value.to_owned()));
            }
        }
        self.pending.push(Node::Expr(expr));
    }

    fn visit_pattern(&mut self, pattern: &'a Pattern) {
        self.pending.push(Node::Pattern(pattern));
    }

    fn visit_interpolated_string_element(&mut self, element: &'a InterpolatedStringElement) {
        self.pending.push(Node::Element(element));
    }
}
