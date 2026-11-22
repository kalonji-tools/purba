//! The patches and the mocks with no spec in the body of a test.

use purba_parsing::{Arms, Enter, Node, walk};
use ruff_python_ast::{Arguments, Expr, ExprCall, ExprName, Stmt};
use ruff_text_size::{Ranged, TextSize};

use super::lookup::{Lookup, Origin};
use super::on_one_line;

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

/// Each patch and each mock with no spec in `body`, by its offset, in source
/// order. A name resolves through `lookup`. The body of a nested `def`,
/// `lambda` or `class` is the test's too.
pub fn faults<'a>(
    body: &'a [Stmt],
    lookup: &Lookup<'a, '_>,
    source: &'a str,
) -> Vec<(TextSize, Fault)> {
    let mut stand_ins = StandIns {
        lookup,
        source,
        found: Vec::new(),
    };
    walk(body.iter().map(Node::Stmt), &mut stand_ins);
    stand_ins.found
}

struct StandIns<'a, 'l> {
    lookup: &'l Lookup<'a, 'l>,
    source: &'a str,
    found: Vec<(TextSize, Fault)>,
}

impl StandIns<'_, '_> {
    /// Where the import that binds `name` comes from, or `None` when no import
    /// binds it.
    fn origin(&self, name: &str) -> Option<&str> {
        match self.lookup.origin(name) {
            Origin::Own(origin) | Origin::Imported(origin) => Some(origin),
            Origin::Local | Origin::Module | Origin::Builtin => None,
        }
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
        self.lookup.qualified(expr).as_deref() == Some("unittest.mock.DEFAULT")
    }

    fn call(&mut self, call: &ExprCall) {
        let at = call.start();
        let arguments = &call.arguments;
        let Some(name) = self.lookup.qualified(&call.func) else {
            return;
        };
        match name.as_str() {
            "builtins.setattr" | "builtins.delattr" => self.builtin(call),
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

    /// A call of the builtin `setattr` or `delattr`: a patch when its object is
    /// an imported name.
    fn builtin(&mut self, call: &ExprCall) {
        let Some(object) = call.arguments.find_positional(0) else {
            return;
        };
        if self.through_imports(object).is_some() {
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

impl<'a> Arms<'a> for StandIns<'a, '_> {
    fn stmt(&mut self, statement: &'a Stmt) -> Enter {
        match statement {
            Stmt::Assign(assign) => assign.targets.iter().for_each(|target| self.store(target)),
            Stmt::AugAssign(assign) => self.store(&assign.target),
            Stmt::AnnAssign(assign) if assign.value.is_some() => self.store(&assign.target),
            Stmt::Delete(delete) => delete.targets.iter().for_each(|target| self.store(target)),
            _ => {}
        }
        Enter::All
    }

    fn expr(&mut self, expr: &'a Expr) -> Enter {
        if let Expr::Call(call) = expr {
            self.call(call);
        }
        Enter::All
    }
}
