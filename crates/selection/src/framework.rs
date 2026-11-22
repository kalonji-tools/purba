//! The imports of another test framework in a test module.

use purba_parsing::{Arms, Enter, Node, walk};
use ruff_python_ast::{Expr, Stmt};
use ruff_text_size::{Ranged, TextSize};

use super::lookup::Lookup;

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
/// that names it. A name resolves through `lookup`.
pub fn imports<'a>(statements: &'a [Stmt], lookup: &Lookup<'a, '_>) -> Vec<(TextSize, String)> {
    let mut found = Found {
        lookup,
        head: false,
        imports: Vec::new(),
    };
    walk(statements.iter().map(Node::Stmt), &mut found);
    found.imports
}

/// Each import of another test framework that runs when `function` is made:
/// in a decorator, a default or an annotation. A name resolves through
/// `lookup`, which holds no import and no local of its own.
pub fn made<'a>(function: &'a Stmt, lookup: &Lookup<'a, '_>) -> Vec<(TextSize, String)> {
    let mut found = Found {
        lookup,
        head: true,
        imports: Vec::new(),
    };
    walk([Node::Stmt(function)], &mut found);
    found.imports
}

struct Found<'a, 'l> {
    lookup: &'l Lookup<'a, 'l>,
    /// Whether the walk enters only what runs where a `def` stands.
    head: bool,
    imports: Vec<(TextSize, String)>,
}

impl<'a> Arms<'a> for Found<'a, '_> {
    fn stmt(&mut self, statement: &'a Stmt) -> Enter {
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
            Stmt::FunctionDef(_) if self.head => return Enter::Head,
            _ => {}
        }
        Enter::All
    }

    fn expr(&mut self, expr: &'a Expr) -> Enter {
        if let Expr::Call(call) = expr
            && let Some(Expr::StringLiteral(first)) = call.arguments.args.first()
            && let Some(function) = self.lookup.qualified(&call.func)
            && CALLS.contains(&function.as_str())
        {
            let value = first.value.to_str();
            if names_a_framework(value)
                && (!DOTTED.contains(&function.as_str()) || value.contains('.'))
            {
                self.imports.push((expr.start(), value.to_owned()));
            }
        }
        Enter::All
    }
}
