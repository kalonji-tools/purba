//! The patches and the mocks with no spec in the body of a test.

use std::collections::{HashMap, HashSet};

use ruff_python_ast::visitor::Visitor;
use ruff_python_ast::{
    Arguments, Expr, ExprCall, ExprName, InterpolatedStringElement, Pattern, Stmt,
};
use ruff_text_size::{Ranged, TextSize};

use super::{Node, on_one_line, origins, walk_node};

/// A fault that a patch or a mock with no spec gives.
pub enum Fault {
    /// A patch of its target.
    Patched(String),
    /// A mock with no spec.
    WithoutSpec,
}

/// The classes of `unittest.mock` that make a mock.
/// <https://docs.python.org/3.12/library/unittest.mock.html#the-mock-class>
const MOCKS: [&str; 5] = [
    "unittest.mock.Mock",
    "unittest.mock.MagicMock",
    "unittest.mock.NonCallableMock",
    "unittest.mock.NonCallableMagicMock",
    "unittest.mock.AsyncMock",
];

/// The keywords of `patch.multiple` that name no attribute.
/// <https://docs.python.org/3.12/library/unittest.mock.html#patch-multiple>
const OF_MULTIPLE: [&str; 6] = [
    "spec",
    "create",
    "spec_set",
    "autospec",
    "new_callable",
    "unsafe",
];

/// Each patch and each mock with no spec in `body`, by its offset, in the order
/// the walk finds them. A name resolves through an import of the body, then
/// through `imported` unless the body binds it. The body of a nested `def`,
/// `lambda` or `class` is the test's too.
pub fn faults<'a>(
    body: &'a [Stmt],
    locals: &HashSet<&'a str>,
    imported: &HashMap<&'a str, String>,
    source: &'a str,
) -> Vec<(TextSize, Fault)> {
    let mut stand_ins = StandIns {
        own: own_imports(body),
        locals,
        imported,
        source,
        found: Vec::new(),
        pending: Vec::new(),
    };
    for statement in body {
        stand_ins.visit_stmt(statement);
    }
    while let Some(node) = stand_ins.pending.pop() {
        walk_node(&mut stand_ins, node);
    }
    stand_ins.found
}

/// Where each name that an `import` in `body` binds comes from, at any depth.
fn own_imports(body: &[Stmt]) -> HashMap<&str, String> {
    let mut own = HashMap::new();
    let mut pending: Vec<&Stmt> = body.iter().collect();
    while let Some(statement) = pending.pop() {
        for (name, origin) in origins(statement) {
            own.insert(super::bound(name), origin);
        }
        pending.extend(super::blocks(statement).into_iter().flatten());
        if let Stmt::FunctionDef(function) = statement {
            pending.extend(&function.body);
        } else if let Stmt::ClassDef(class) = statement {
            pending.extend(&class.body);
        }
    }
    own
}

struct StandIns<'a, 'b> {
    own: HashMap<&'a str, String>,
    locals: &'b HashSet<&'a str>,
    imported: &'b HashMap<&'a str, String>,
    source: &'a str,
    found: Vec<(TextSize, Fault)>,
    pending: Vec<Node<'a>>,
}

impl StandIns<'_, '_> {
    /// Where the import that binds `name` comes from, or `None` when no import
    /// binds it.
    fn origin(&self, name: &str) -> Option<&str> {
        if let Some(origin) = self.own.get(name) {
            return Some(origin);
        }
        if self.locals.contains(name) {
            return None;
        }
        self.imported.get(name).map(String::as_str)
    }

    /// The dotted name of `expr` through the imports, such as
    /// `unittest.mock.patch` for `mock.patch`.
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
        let mut qualified = self.origin(&name.id)?.to_owned();
        for part in parts.iter().rev() {
            qualified.push('.');
            qualified.push_str(part);
        }
        Some(qualified)
    }

    /// `expr` with the name at its root replaced by where its import comes
    /// from, such as `os.environ["EDITOR"]` for `environ["EDITOR"]`, or `None`
    /// when no import binds that name.
    fn through_imports(&self, expr: &Expr) -> Option<String> {
        let root = root(expr)?;
        let origin = self.origin(&root.id)?;
        let rest = &self.source[usize::from(root.end())..usize::from(expr.end())];
        Some(format!("{origin}{}", on_one_line(rest)))
    }

    /// The target that `expr` names: a string's value, or the text of `expr`
    /// through the imports.
    fn target(&self, expr: &Expr) -> String {
        if let Expr::StringLiteral(string) = expr {
            return string.value.to_str().to_owned();
        }
        self.through_imports(expr)
            .unwrap_or_else(|| on_one_line(&self.source[expr.range()]))
    }

    /// The target of `object`, then the name in `attribute` when that name is a
    /// string, or else the target of `object` alone.
    fn attribute_of(&self, object: &Expr, attribute: Option<&Expr>) -> String {
        let object = self.target(object);
        match attribute {
            Some(Expr::StringLiteral(name)) => format!("{object}.{}", name.value.to_str()),
            _ => object,
        }
    }

    fn is_default(&self, expr: &Expr) -> bool {
        self.qualified(expr).as_deref() == Some("unittest.mock.DEFAULT")
    }

    fn call(&mut self, call: &ExprCall) {
        let at = call.start();
        let arguments = &call.arguments;
        let Some(name) = self.qualified(&call.func) else {
            self.builtin(call);
            return;
        };
        match name.as_str() {
            "unittest.mock.patch" => {
                let target = arguments
                    .find_argument_value("target", 0)
                    .map(|target| self.target(target));
                self.patch(at, target, arguments, [1, 2, 4, 5, 6]);
            }
            "unittest.mock.patch.object" => {
                let target = arguments.find_argument_value("target", 0).map(|object| {
                    self.attribute_of(object, arguments.find_argument_value("attribute", 1))
                });
                self.patch(at, target, arguments, [2, 3, 5, 6, 7]);
            }
            "unittest.mock.patch.dict" => {
                if let Some(target) = arguments.find_argument_value("in_dict", 0) {
                    let target = self.target(target);
                    self.found.push((at, Fault::Patched(target)));
                }
            }
            "unittest.mock.patch.multiple" => self.multiple(at, arguments),
            mock if MOCKS.contains(&mock) => {
                let spec = arguments.find_argument_value("spec", 0).is_some_and(given)
                    || arguments
                        .find_keyword("spec_set")
                        .is_some_and(|k| given(&k.value));
                if !spec && !unseen(arguments) {
                    self.found.push((at, Fault::WithoutSpec));
                }
            }
            _ => {}
        }
    }

    /// A call of `patch` or `patch.object`, whose arguments `new`, `spec`,
    /// `spec_set`, `autospec` and `new_callable` sit at `positions`.
    fn patch(
        &mut self,
        at: TextSize,
        target: Option<String>,
        arguments: &Arguments,
        [new, spec, spec_set, autospec, new_callable]: [usize; 5],
    ) {
        let Some(target) = target else {
            return;
        };
        self.found.push((at, Fault::Patched(target)));
        let value = arguments
            .find_argument_value("new", new)
            .is_some_and(|value| !self.is_default(value))
            || arguments
                .find_argument_value("new_callable", new_callable)
                .is_some_and(given);
        let spec = [
            ("spec", spec),
            ("spec_set", spec_set),
            ("autospec", autospec),
        ]
        .into_iter()
        .any(|(name, position)| {
            arguments
                .find_argument_value(name, position)
                .is_some_and(given)
        });
        if !value && !spec && !unseen(arguments) {
            self.found.push((at, Fault::WithoutSpec));
        }
    }

    /// A call of `patch.multiple`: a patch of each name that it gives a value.
    fn multiple(&mut self, at: TextSize, arguments: &Arguments) {
        let Some(object) = arguments.find_argument_value("target", 0) else {
            return;
        };
        let object = self.target(object);
        let named: Vec<_> = arguments
            .keywords
            .iter()
            .filter_map(|keyword| {
                keyword
                    .arg
                    .as_ref()
                    .map(|arg| (arg.as_str(), &keyword.value))
            })
            .filter(|(arg, _)| !OF_MULTIPLE.contains(arg))
            .collect();
        if named.is_empty() {
            self.found.push((at, Fault::Patched(object)));
            return;
        }
        for (name, _) in &named {
            self.found
                .push((at, Fault::Patched(format!("{object}.{name}"))));
        }
        let spec = ["spec", "spec_set", "autospec", "new_callable"]
            .into_iter()
            .any(|name| {
                arguments
                    .find_keyword(name)
                    .is_some_and(|k| given(&k.value))
            });
        if !spec && !unseen(arguments) && named.iter().any(|(_, value)| self.is_default(value)) {
            self.found.push((at, Fault::WithoutSpec));
        }
    }

    /// A call of the builtin `setattr` or `delattr` on an imported name.
    fn builtin(&mut self, call: &ExprCall) {
        let Expr::Name(name) = &*call.func else {
            return;
        };
        let builtin = matches!(name.id.as_str(), "setattr" | "delattr")
            && !self.own.contains_key(name.id.as_str())
            && !self.locals.contains(name.id.as_str())
            && !self.imported.contains_key(name.id.as_str());
        let Some(object) = call.arguments.find_positional(0) else {
            return;
        };
        if builtin && self.through_imports(object).is_some() {
            let target = self.attribute_of(object, call.arguments.find_positional(1));
            self.found.push((call.start(), Fault::Patched(target)));
        }
    }

    /// An assignment to, or a `del` of, `target`: a patch when it is an
    /// attribute or an item of an imported name.
    fn store(&mut self, target: &Expr) {
        match target {
            Expr::Tuple(tuple) => tuple.elts.iter().for_each(|item| self.store(item)),
            Expr::List(list) => list.elts.iter().for_each(|item| self.store(item)),
            Expr::Starred(starred) => self.store(&starred.value),
            Expr::Attribute(_) | Expr::Subscript(_) => {
                if let Some(text) = self.through_imports(target) {
                    self.found.push((target.start(), Fault::Patched(text)));
                }
            }
            _ => {}
        }
    }
}

/// The name at the root of a chain of attributes and items.
fn root(expr: &Expr) -> Option<&ExprName> {
    let mut current = expr;
    loop {
        match current {
            Expr::Attribute(attribute) => current = &attribute.value,
            Expr::Subscript(subscript) => current = &subscript.value,
            Expr::Name(name) => return Some(name),
            _ => return None,
        }
    }
}

/// Whether an argument gives something: it is not `None` and not `False`.
const fn given(value: &Expr) -> bool {
    !value.is_none_literal_expr()
        && !matches!(value, Expr::BooleanLiteral(literal) if !literal.value)
}

/// Whether a `*` or a `**` argument may give what syntax does not show.
fn unseen(arguments: &Arguments) -> bool {
    arguments.args.iter().any(Expr::is_starred_expr)
        || arguments
            .keywords
            .iter()
            .any(|keyword| keyword.arg.is_none())
}

// As in `Body`, each visit pushes its node, and `faults` walks each node it
// pushed.
impl<'a> Visitor<'a> for StandIns<'a, '_> {
    fn visit_stmt(&mut self, statement: &'a Stmt) {
        match statement {
            Stmt::Assign(assign) => assign.targets.iter().for_each(|target| self.store(target)),
            Stmt::AugAssign(assign) => self.store(&assign.target),
            Stmt::AnnAssign(assign) if assign.value.is_some() => self.store(&assign.target),
            Stmt::Delete(delete) => delete.targets.iter().for_each(|target| self.store(target)),
            _ => {}
        }
        self.pending.push(Node::Stmt(statement));
    }

    fn visit_expr(&mut self, expr: &'a Expr) {
        if let Expr::Call(call) = expr {
            self.call(call);
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
