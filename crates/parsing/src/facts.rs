//! The facts of a module: its tests, its imports and the names it binds. Each
//! test also holds its own facts, which parsing finds when a part first asks
//! for them.

use std::cell::OnceCell;
use std::collections::hash_map::Entry;
use std::collections::{HashMap, HashSet};

use ruff_python_ast::{
    Alias, Comprehension, ExceptHandler, Expr, ExprContext, Identifier, Parameters, Pattern, Stmt,
    StmtAssert, StmtFunctionDef, StmtTry,
};
use ruff_text_size::{Ranged, TextSize};

use crate::walk::{Arms, Enter, Node, walk};
use crate::{Module, Shape, blocks, ending, keyword, shape};

/// The facts of a module: its tests, its imports and the names it binds.
/// They borrow the module, and drop before it.
pub struct Facts<'m> {
    module: &'m Module,
    statements: Vec<&'m Stmt>,
    defs: Vec<&'m StmtFunctionDef>,
    tests: Vec<Test<'m>>,
    imports: HashMap<&'m str, String>,
    bindings: HashMap<&'m str, Vec<Binding>>,
}

/// A name with a `test` name that a `def` binds at the top of a module, under
/// a statement there, or in a class there.
pub struct Test<'m> {
    /// The classes on the way, outermost first. Empty at the module's level.
    pub classes: Vec<&'m str>,
    /// The keyword of the statement at the top that the first `def` that binds
    /// the name is under, or `None`.
    pub under: Option<&'static str>,
    /// Each `def` of the name, in source order. The last one binds it.
    pub defs: Vec<&'m StmtFunctionDef>,
    /// Where the first `def` of the name starts.
    pub first: TextSize,
    facts: OnceCell<TestFacts<'m>>,
}

impl<'m> Test<'m> {
    /// The facts of the `def` that binds the name, the last of `defs`. The
    /// first call reads them, and each later call gives the same facts.
    pub fn facts(&self) -> &TestFacts<'m> {
        self.facts.get_or_init(|| {
            self.defs
                .last()
                .map_or_else(TestFacts::default, |function| TestFacts::of(function))
        })
    }

    /// The facts of the test, after a part asks for them.
    #[must_use]
    pub fn facts_if_read(&self) -> Option<&TestFacts<'m>> {
        self.facts.get()
    }
}

/// The facts of the `def` of one test.
#[derive(Default)]
pub struct TestFacts<'m> {
    /// The name of each parameter.
    pub parameters: HashSet<&'m str>,
    /// Each name that the test binds: its parameters, and each name that its
    /// body binds.
    pub locals: HashSet<&'m str>,
    /// The asserts at the end of the body, as [`ending`] gives them, each with
    /// its shape. A tuple assert can be one of them.
    pub ending: Vec<(&'m StmtAssert, Shape<'m>)>,
    /// Each statement of the body at any depth, in source order. The list
    /// leaves out the body of each `def` or `class` in the body.
    pub statements: Vec<&'m Stmt>,
}

impl<'m> TestFacts<'m> {
    fn of(function: &'m StmtFunctionDef) -> Self {
        Self {
            parameters: parameters(&function.parameters),
            locals: locals(function),
            ending: ending(&function.body)
                .into_iter()
                .map(|assert| (assert, shape(&assert.test)))
                .collect(),
            statements: statements(&function.body),
        }
    }

    /// Each `try` among the statements, in source order.
    pub fn tries(&self) -> impl Iterator<Item = &'m StmtTry> + '_ {
        self.statements
            .iter()
            .filter_map(|statement| match statement {
                Stmt::Try(attempt) => Some(attempt),
                _ => None,
            })
    }
}

/// The name of each parameter of `parameters`, of a `def` or of a `lambda`.
#[must_use]
pub fn parameters(parameters: &Parameters) -> HashSet<&str> {
    parameters
        .posonlyargs
        .iter()
        .chain(&parameters.args)
        .chain(&parameters.kwonlyargs)
        .map(|parameter| parameter.parameter.name.as_str())
        .chain(parameters.vararg.iter().map(|vararg| vararg.name.as_str()))
        .chain(parameters.kwarg.iter().map(|kwarg| kwarg.name.as_str()))
        .collect()
}

/// Each name that `function` binds: its parameters, and each name that its
/// body binds.
#[must_use]
pub fn locals(function: &StmtFunctionDef) -> HashSet<&str> {
    let mut locals: HashSet<&str> = Scope::of(&function.body)
        .names
        .into_iter()
        .map(|(name, _)| name)
        .collect();
    locals.extend(parameters(&function.parameters));
    locals
}

/// Each statement of `body` at any depth, in source order. The body of a `def`
/// or a `class` is not part of it.
fn statements(body: &[Stmt]) -> Vec<&Stmt> {
    #[derive(Default)]
    struct Statements<'a>(Vec<&'a Stmt>);
    impl<'a> Arms<'a> for Statements<'a> {
        fn stmt(&mut self, statement: &'a Stmt) -> Enter {
            self.0.push(statement);
            match statement {
                Stmt::FunctionDef(_) | Stmt::ClassDef(_) => Enter::Nothing,
                _ => Enter::All,
            }
        }

        fn expr(&mut self, _expr: &'a Expr) -> Enter {
            Enter::Nothing
        }
    }
    let mut statements = Statements::default();
    walk(body.iter().map(Node::Stmt), &mut statements);
    statements.0
}

/// A place where a module binds a name. A binding in a function binds the name
/// whenever the function runs.
pub struct Binding {
    pub offset: TextSize,
    pub in_function: bool,
}

impl<'m> Facts<'m> {
    /// Reads the facts of `module`: one walk of its statements, and one walk
    /// of each scope for its bindings.
    #[must_use]
    pub fn of(module: &'m Module) -> Self {
        let mut facts = Self {
            module,
            statements: Vec::new(),
            defs: Vec::new(),
            tests: Vec::new(),
            imports: HashMap::new(),
            bindings: bindings(&module.syntax().body),
        };
        let mut candidates: Vec<(Place<'m>, &'m StmtFunctionDef)> = Vec::new();
        // A walk that recurses once for each level of nesting overflows the
        // stack on statements that nest deeper than the stack. Each statement
        // travels with its place, which `walk` does not give.
        let mut pending: Vec<(&'m Stmt, Place<'m>)> = module
            .syntax()
            .body
            .iter()
            .rev()
            .map(|statement| (statement, Place::Top))
            .collect();
        while let Some((statement, place)) = pending.pop() {
            if matches!(place, Place::Top | Place::Under(_)) {
                facts.statements.push(statement);
                for (name, origin) in origins(statement) {
                    facts.imports.insert(bound(name), origin);
                }
            }
            match statement {
                Stmt::FunctionDef(function) => {
                    facts.defs.push(function);
                    candidates.push((place, function));
                    pending.extend(function.body.iter().rev().map(|inner| (inner, Place::Def)));
                }
                Stmt::ClassDef(class) => {
                    let inner = match place {
                        Place::Top | Place::Under(_) => Place::Class(vec![class.name.as_str()]),
                        Place::Class(mut classes) => {
                            classes.push(class.name.as_str());
                            Place::Class(classes)
                        }
                        Place::Def => Place::Def,
                    };
                    pending.extend(class.body.iter().rev().map(|body| (body, inner.clone())));
                }
                _ => {
                    let inner = match (&place, keyword(statement)) {
                        (Place::Top, Some(keyword)) => Place::Under(keyword),
                        _ => place,
                    };
                    pending.extend(
                        blocks(statement)
                            .into_iter()
                            .flatten()
                            .rev()
                            .map(|block| (block, inner.clone())),
                    );
                }
            }
        }
        facts.tests = tests(candidates);
        facts
    }

    /// The module that the facts read.
    #[must_use]
    pub const fn module(&self) -> &'m Module {
        self.module
    }

    /// Each statement at the module's level, or in a block there, in source
    /// order.
    #[must_use]
    pub fn statements(&self) -> &[&'m Stmt] {
        &self.statements
    }

    /// Each `def` at any depth, in source order.
    #[must_use]
    pub fn defs(&self) -> &[&'m StmtFunctionDef] {
        &self.defs
    }

    /// Each name with a `test` name that a `def` binds at the top, under a
    /// statement there, or in a class there, in the order of its first `def`.
    /// A `def` under a statement does not count when a `def` at the top binds
    /// its name, though it keeps its place. A class in a `def` holds no test.
    #[must_use]
    pub fn tests(&self) -> &[Test<'m>] {
        &self.tests
    }

    /// Where each name that an import at the module's level, or in a block
    /// there, binds comes from. The last import of a name wins.
    #[must_use]
    pub const fn imports(&self) -> &HashMap<&'m str, String> {
        &self.imports
    }

    /// Each place where the module binds a name, by name: at the module's
    /// level, and under `global` in the body of a `def` or a `class`.
    #[must_use]
    pub const fn bindings(&self) -> &HashMap<&'m str, Vec<Binding>> {
        &self.bindings
    }
}

/// Where a statement of a module stands.
#[derive(Clone)]
enum Place<'m> {
    /// At the module's level.
    Top,
    /// In a block of a statement at the module's level, with its keyword.
    Under(&'static str),
    /// In the body of each class on the way, outermost first.
    Class(Vec<&'m str>),
    /// In the body of a `def`.
    Def,
}

/// The tests among `candidates`, each `def` beside its place, in source order.
fn tests<'m>(candidates: Vec<(Place<'m>, &'m StmtFunctionDef)>) -> Vec<Test<'m>> {
    let top: HashSet<&str> = candidates
        .iter()
        .filter(|(place, _)| matches!(place, Place::Top))
        .map(|(_, function)| function.name.as_str())
        .collect();
    let mut tests: Vec<Test<'m>> = Vec::new();
    let mut places: HashMap<(Vec<&str>, &str), usize> = HashMap::new();
    let mut firsts: HashMap<&str, TextSize> = HashMap::new();
    for (place, function) in candidates {
        let name = function.name.as_str();
        if !name.starts_with("test") {
            continue;
        }
        let (classes, under) = match place {
            Place::Top => (Vec::new(), None),
            Place::Under(keyword) => (Vec::new(), Some(keyword)),
            Place::Class(classes) => (classes, None),
            Place::Def => continue,
        };
        let mut first = function.start();
        if classes.is_empty() {
            first = *firsts.entry(name).or_insert(first);
            if under.is_some() && top.contains(name) {
                continue;
            }
        }
        match places.entry((classes.clone(), name)) {
            Entry::Occupied(place) => tests[*place.get()].defs.push(function),
            Entry::Vacant(place) => {
                place.insert(tests.len());
                tests.push(Test {
                    classes,
                    under,
                    defs: vec![function],
                    first,
                    facts: OnceCell::new(),
                });
            }
        }
    }
    tests.sort_by_key(|test| test.first);
    tests
}

/// Each place where the module of `body` binds a name, by name: at the
/// module's level, and under `global` in the body of a `def` or a `class`.
fn bindings(body: &[Stmt]) -> HashMap<&str, Vec<Binding>> {
    let mut found: HashMap<&str, Vec<Binding>> = HashMap::new();
    let mut scopes = vec![(body, true, false)];
    while let Some((statements, module_level, in_function)) = scopes.pop() {
        let scope = Scope::of(statements);
        for (name, offset) in scope.names {
            if module_level || scope.globals.contains(name) {
                found.entry(name).or_default().push(Binding {
                    offset,
                    in_function,
                });
            }
        }
        scopes.extend(scope.functions.into_iter().map(|body| (body, false, true)));
        scopes.extend(
            scope
                .classes
                .into_iter()
                .map(|body| (body, false, in_function)),
        );
    }
    found
}

/// Where each name of an `import` comes from, by the name it binds. `import a.b`
/// binds `a`, which comes from `a`.
/// <https://docs.python.org/3.12/reference/simple_stmts.html#the-import-statement>
#[must_use]
pub fn origins(statement: &Stmt) -> Vec<(&Identifier, String)> {
    match statement {
        Stmt::Import(import) => import
            .names
            .iter()
            .map(|alias| {
                alias.asname.as_ref().map_or_else(
                    || {
                        (
                            &alias.name,
                            alias.name.split('.').next().unwrap_or_default().to_owned(),
                        )
                    },
                    |asname| (asname, alias.name.to_string()),
                )
            })
            .collect(),
        Stmt::ImportFrom(import) => {
            let mut from = ".".repeat(import.level as usize);
            from.push_str(import.module.as_ref().map_or("", Identifier::as_str));
            import
                .names
                .iter()
                .filter(|alias| alias.name.as_str() != "*")
                .map(|alias| {
                    let origin = if from.ends_with('.') || from.is_empty() {
                        format!("{from}{}", alias.name)
                    } else {
                        format!("{from}.{}", alias.name)
                    };
                    (alias.asname.as_ref().unwrap_or(&alias.name), origin)
                })
                .collect()
        }
        _ => Vec::new(),
    }
}

/// The name that an `import` of `name` binds: `import a.b` binds `a`.
#[must_use]
pub fn bound(name: &Identifier) -> &str {
    name.as_str().split('.').next().unwrap_or_default()
}

/// The names that one scope binds, and the names it declares `global`.
///
/// The body of a nested `def` or `class` is a scope of its own. A `lambda` and
/// the target of a comprehension bind nothing in this scope.
/// <https://docs.python.org/3.12/reference/executionmodel.html#binding-of-names>
/// <https://docs.python.org/3.12/reference/simple_stmts.html#annotated-assignment-statements>
#[derive(Default)]
pub struct Scope<'a> {
    /// Each name that the scope binds, by the offset of its binding.
    pub names: Vec<(&'a str, TextSize)>,
    /// Each name that the scope declares `global`.
    pub globals: HashSet<&'a str>,
    /// The body of each `def` in the scope.
    pub functions: Vec<&'a [Stmt]>,
    /// The body of each `class` in the scope.
    pub classes: Vec<&'a [Stmt]>,
}

impl<'a> Scope<'a> {
    /// The scope of `statements`.
    #[must_use]
    pub fn of(statements: &'a [Stmt]) -> Self {
        let mut scope = Self::default();
        walk(statements.iter().map(Node::Stmt), &mut scope);
        scope
    }

    fn bind(&mut self, name: &'a Identifier) {
        self.names.push((name.as_str(), name.start()));
    }
}

impl<'a> Arms<'a> for Scope<'a> {
    fn stmt(&mut self, statement: &'a Stmt) -> Enter {
        match statement {
            Stmt::FunctionDef(function) => {
                self.bind(&function.name);
                self.functions.push(&function.body);
                Enter::Head
            }
            Stmt::ClassDef(class) => {
                self.bind(&class.name);
                self.classes.push(&class.body);
                Enter::Head
            }
            Stmt::Global(global) => {
                self.globals
                    .extend(global.names.iter().map(Identifier::as_str));
                Enter::All
            }
            // `x: T` with no value binds nothing.
            Stmt::AnnAssign(assign) if assign.value.is_none() => {
                walk([Node::Expr(&assign.annotation)], self);
                Enter::Nothing
            }
            _ => Enter::All,
        }
    }

    fn expr(&mut self, expr: &'a Expr) -> Enter {
        match expr {
            Expr::Name(name) if matches!(name.ctx, ExprContext::Store | ExprContext::Del) => {
                self.names.push((name.id.as_str(), name.start()));
                Enter::All
            }
            Expr::Lambda(_) => Enter::Head,
            _ => Enter::All,
        }
    }

    fn comprehension(&mut self, _comprehension: &'a Comprehension) -> Enter {
        Enter::Head
    }

    fn except_handler(&mut self, handler: &'a ExceptHandler) {
        let ExceptHandler::ExceptHandler(clause) = handler;
        if let Some(name) = &clause.name {
            self.bind(name);
        }
    }

    fn alias(&mut self, alias: &'a Alias) {
        match &alias.asname {
            Some(asname) => self.bind(asname),
            // `import a.b` binds `a`.
            // https://docs.python.org/3.12/reference/simple_stmts.html#the-import-statement
            None if alias.name.as_str() != "*" => {
                let first = alias.name.split('.').next().unwrap_or_default();
                self.names.push((first, alias.name.start()));
            }
            None => {}
        }
    }

    fn pattern(&mut self, pattern: &'a Pattern) {
        let captured = match pattern {
            Pattern::MatchAs(capture) => capture.name.as_ref(),
            Pattern::MatchStar(star) => star.name.as_ref(),
            Pattern::MatchMapping(mapping) => mapping.rest.as_ref(),
            _ => None,
        };
        if let Some(name) = captured {
            self.bind(name);
        }
    }
}
