//! Selection: names the tests of a parsed module, refuses each one that purba
//! cannot call, and imports nothing.

use std::collections::hash_map::Entry;
use std::collections::{HashMap, HashSet};
use std::fmt;
use std::path::{Path, PathBuf};

use purba_parsing::Module;
use ruff_python_ast::name::UnqualifiedName;
use ruff_python_ast::visitor::{
    Visitor, walk_except_handler, walk_expr, walk_interpolated_string_element, walk_pattern,
    walk_stmt,
};
use ruff_python_ast::{
    Alias, Comprehension, Decorator, ExceptHandler, Expr, ExprContext, Identifier,
    InterpolatedStringElement, ModModule, Parameters, Pattern, Stmt, StmtFunctionDef,
};
use ruff_text_size::{Ranged, TextSize};

/// A test's identity. Only selection constructs one.
///
/// ```compile_fail,E0451
/// let id = purba_selection::TestId { path: "test_a.py".into(), function: "test_one".into() };
/// ```
///
/// ```compile_fail,E0616
/// fn forge(id: &mut purba_selection::TestId) { id.path = "test_b.py".into(); }
/// ```
///
/// ```compile_fail,E0616
/// fn forge(id: &mut purba_selection::TestId) { id.function = "test_two".into(); }
/// ```
pub struct TestId {
    path: PathBuf,
    function: String,
}

impl TestId {
    /// The path of the test's module, as the tester typed it.
    #[must_use]
    pub fn path(&self) -> &Path {
        &self.path
    }

    /// The name of the test's function.
    #[must_use]
    pub fn function(&self) -> &str {
        &self.function
    }
}

impl fmt::Display for TestId {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{}::{}", self.path.display(), self.function)
    }
}

/// A test that purba cannot call, and why. Only selection constructs one. It
/// displays every reason, joined with `; `.
///
/// ```compile_fail,E0451
/// fn forge(test: purba_selection::TestId) -> purba_selection::Refused {
///     purba_selection::Refused { test, reasons: Vec::new() }
/// }
/// ```
pub struct Refused {
    test: TestId,
    reasons: Vec<Reason>,
}

impl Refused {
    /// The test that selection refuses.
    #[must_use]
    pub const fn test(&self) -> &TestId {
        &self.test
    }
}

impl fmt::Display for Refused {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        for (index, reason) in self.reasons.iter().enumerate() {
            if index > 0 {
                formatter.write_str("; ")?;
            }
            write!(formatter, "{reason}")?;
        }
        Ok(())
    }
}

enum Reason {
    Decorated(Vec<String>),
    Async,
    Generator,
    ReturnsValue,
    NoDefault(Vec<String>),
    Rebound(String, Vec<usize>),
}

impl fmt::Display for Reason {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Decorated(names) if names.len() == 1 => write!(
                formatter,
                "the decorator {} is not purba's, and purba cannot see what it does",
                names[0]
            ),
            Self::Decorated(names) => write!(
                formatter,
                "the decorators {} are not purba's, and purba cannot see what they do",
                names.join(", ")
            ),
            Self::Async => formatter.write_str("purba cannot run an async test yet"),
            Self::Generator => {
                formatter.write_str("the test is a generator, and purba does not iterate it")
            }
            Self::ReturnsValue => {
                formatter.write_str("the test returns a value, and purba does not read it")
            }
            Self::NoDefault(names) if names.len() == 1 => write!(
                formatter,
                "the parameter {} has no default, and purba cannot see which fixture builds it",
                names[0]
            ),
            Self::NoDefault(names) => write!(
                formatter,
                "the parameters {} have no default, and purba cannot see which fixtures build them",
                names.join(", ")
            ),
            Self::Rebound(name, lines) if lines.len() == 1 => write!(
                formatter,
                "line {} binds {name} again, and purba cannot see what the name then holds",
                lines[0]
            ),
            Self::Rebound(name, lines) => {
                let lines: Vec<String> = lines.iter().map(ToString::to_string).collect();
                write!(
                    formatter,
                    "lines {} bind {name} again, and purba cannot see what the name then holds",
                    lines.join(", ")
                )
            }
        }
    }
}

/// The tests of `module` that selection hands on, and each test it refuses.
///
/// Each list keeps source order. `path` is the module's path as the tester
/// typed it. A name defined twice is one test, at its first definition: import
/// binds the name there and binds it again at each later `def`, so selection
/// judges the last body.
#[must_use]
pub fn select(path: &Path, module: &Module) -> (Vec<TestId>, Vec<Refused>) {
    let mut last = Vec::new();
    let mut places = HashMap::new();
    for statement in &module.syntax().body {
        if let Stmt::FunctionDef(function) = statement
            && function.name.starts_with("test")
        {
            match places.entry(function.name.as_str()) {
                Entry::Occupied(place) => last[*place.get()] = function,
                Entry::Vacant(place) => {
                    place.insert(last.len());
                    last.push(function);
                }
            }
        }
    }
    let bindings = bindings(module.syntax());
    let mut tests = Vec::new();
    let mut refused = Vec::new();
    for function in last {
        let test = TestId {
            path: path.to_path_buf(),
            function: function.name.to_string(),
        };
        let reasons = reasons(function, module, &bindings);
        if reasons.is_empty() {
            tests.push(test);
        } else {
            refused.push(Refused { test, reasons });
        }
    }
    (tests, refused)
}

/// Why purba cannot call `function`.
fn reasons(
    function: &StmtFunctionDef,
    module: &Module,
    bindings: &HashMap<&str, Vec<Binding>>,
) -> Vec<Reason> {
    let body = Body::of(&function.body);
    let mut reasons = Vec::new();
    if !function.decorator_list.is_empty() {
        reasons.push(Reason::Decorated(
            function
                .decorator_list
                .iter()
                .map(|decorator| decorator_name(decorator, module.source()))
                .collect(),
        ));
    }
    if function.is_async {
        reasons.push(Reason::Async);
    } else if body.yields {
        reasons.push(Reason::Generator);
    }
    if body.returns_value {
        reasons.push(Reason::ReturnsValue);
    }
    let without_default = without_default(&function.parameters);
    if !without_default.is_empty() {
        reasons.push(Reason::NoDefault(without_default));
    }
    let lines = rebound(function, module, bindings);
    if !lines.is_empty() {
        reasons.push(Reason::Rebound(function.name.to_string(), lines));
    }
    reasons
}

/// A decorator's dotted name, without a call's arguments. Any other form is
/// its text, on one line.
fn decorator_name(decorator: &Decorator, source: &str) -> String {
    let mut expr = &decorator.expression;
    while let Expr::Call(call) = expr {
        expr = &call.func;
    }
    UnqualifiedName::from_expr(expr).map_or_else(
        || {
            source[expr.range()]
                .lines()
                .map(str::trim)
                .collect::<Vec<_>>()
                .join(" ")
        },
        |name| name.to_string(),
    )
}

/// Each line that binds the name of `function` after its `def`, or under
/// `global` in any function, before or after it. In source order.
fn rebound(
    function: &StmtFunctionDef,
    module: &Module,
    bindings: &HashMap<&str, Vec<Binding>>,
) -> Vec<usize> {
    let mut offsets: Vec<TextSize> = bindings
        .get(function.name.as_str())
        .into_iter()
        .flatten()
        .filter(|binding| binding.in_function || binding.offset >= function.range.end())
        .map(|binding| binding.offset)
        .collect();
    offsets.sort_unstable();
    let mut lines: Vec<usize> = offsets
        .into_iter()
        .map(|offset| module.position(offset).line())
        .collect();
    lines.dedup();
    lines
}

/// A place where the module binds a name. A binding in a function binds the
/// name whenever the function runs.
struct Binding {
    offset: TextSize,
    in_function: bool,
}

/// Each place where the module binds a name, by name: at the module's level,
/// and under `global` in the body of a `def` or a `class`.
fn bindings(module: &ModModule) -> HashMap<&str, Vec<Binding>> {
    let mut found: HashMap<&str, Vec<Binding>> = HashMap::new();
    let mut scopes = vec![(module.body.as_slice(), true, false)];
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

/// Each parameter with no default. `*args` and `**kwargs` take nothing that
/// purba must pass.
fn without_default(parameters: &Parameters) -> Vec<String> {
    parameters
        .posonlyargs
        .iter()
        .chain(&parameters.args)
        .chain(&parameters.kwonlyargs)
        .filter(|parameter| parameter.default.is_none())
        .map(|parameter| parameter.parameter.name.to_string())
        .collect()
}

/// Whether a function's own body yields, and whether it returns a value. The
/// body of a nested `def`, `lambda` or `class` is not its own.
#[derive(Default)]
struct Body<'a> {
    yields: bool,
    returns_value: bool,
    pending: Vec<Node<'a>>,
}

#[derive(Clone, Copy)]
enum Node<'a> {
    Stmt(&'a Stmt),
    Expr(&'a Expr),
    Pattern(&'a Pattern),
    Element(&'a InterpolatedStringElement),
}

impl<'a> Body<'a> {
    fn of(statements: &'a [Stmt]) -> Self {
        let mut body = Self::default();
        for statement in statements {
            body.visit_stmt(statement);
        }
        while let Some(node) = body.pending.pop() {
            walk_node(&mut body, node);
        }
        body
    }
}

fn walk_node<'a, V: Visitor<'a>>(visitor: &mut V, node: Node<'a>) {
    match node {
        Node::Stmt(statement) => walk_stmt(visitor, statement),
        Node::Expr(expr) => walk_expr(visitor, expr),
        Node::Pattern(pattern) => walk_pattern(visitor, pattern),
        Node::Element(element) => walk_interpolated_string_element(visitor, element),
    }
}

// Each visit pushes its node, or skips a nested `def`, `lambda` or `class`, and
// `Body::of` walks each node it pushed. A walk that recurses once for each level
// of nesting overflows the stack on a body that nests deeper than the stack.
impl<'a> Visitor<'a> for Body<'a> {
    fn visit_stmt(&mut self, statement: &'a Stmt) {
        match statement {
            Stmt::FunctionDef(_) | Stmt::ClassDef(_) => return,
            Stmt::Return(ret)
                if ret
                    .value
                    .as_deref()
                    .is_some_and(|value| !value.is_none_literal_expr()) =>
            {
                self.returns_value = true;
            }
            _ => {}
        }
        self.pending.push(Node::Stmt(statement));
    }

    fn visit_expr(&mut self, expr: &'a Expr) {
        match expr {
            Expr::Lambda(_) => return,
            Expr::Yield(_) | Expr::YieldFrom(_) => self.yields = true,
            _ => {}
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

/// The names that one scope binds, and the names it declares `global`. The body
/// of a nested `def` or `class` is a scope of its own. A `lambda` and the target
/// of a comprehension bind nothing in this scope.
/// <https://docs.python.org/3.12/reference/executionmodel.html#binding-of-names>
/// <https://docs.python.org/3.12/reference/simple_stmts.html#annotated-assignment-statements>
#[derive(Default)]
struct Scope<'a> {
    names: Vec<(&'a str, TextSize)>,
    globals: HashSet<&'a str>,
    functions: Vec<&'a [Stmt]>,
    classes: Vec<&'a [Stmt]>,
    pending: Vec<Node<'a>>,
}

impl<'a> Scope<'a> {
    fn of(statements: &'a [Stmt]) -> Self {
        let mut scope = Self::default();
        for statement in statements {
            scope.visit_stmt(statement);
        }
        while let Some(node) = scope.pending.pop() {
            walk_node(&mut scope, node);
        }
        scope
    }

    fn bind(&mut self, name: &'a Identifier) {
        self.names.push((name.as_str(), name.start()));
    }
}

// As in `Body`, each visit pushes its node, or skips what binds nothing in this
// scope, and `Scope::of` walks each node it pushed.
impl<'a> Visitor<'a> for Scope<'a> {
    fn visit_stmt(&mut self, statement: &'a Stmt) {
        match statement {
            Stmt::FunctionDef(function) => {
                self.bind(&function.name);
                for decorator in &function.decorator_list {
                    self.visit_expr(&decorator.expression);
                }
                self.visit_parameters(&function.parameters);
                if let Some(returns) = &function.returns {
                    self.visit_expr(returns);
                }
                self.functions.push(&function.body);
                return;
            }
            Stmt::ClassDef(class) => {
                self.bind(&class.name);
                for decorator in &class.decorator_list {
                    self.visit_expr(&decorator.expression);
                }
                if let Some(arguments) = &class.arguments {
                    self.visit_arguments(arguments);
                }
                self.classes.push(&class.body);
                return;
            }
            Stmt::Global(global) => {
                self.globals
                    .extend(global.names.iter().map(Identifier::as_str));
            }
            // `x: T` with no value binds nothing.
            Stmt::AnnAssign(assign) if assign.value.is_none() => {
                self.visit_expr(&assign.annotation);
                return;
            }
            _ => {}
        }
        self.pending.push(Node::Stmt(statement));
    }

    fn visit_expr(&mut self, expr: &'a Expr) {
        match expr {
            Expr::Name(name) if matches!(name.ctx, ExprContext::Store | ExprContext::Del) => {
                self.names.push((name.id.as_str(), name.start()));
            }
            Expr::Lambda(lambda) => {
                if let Some(parameters) = &lambda.parameters {
                    self.visit_parameters(parameters);
                }
                return;
            }
            _ => {}
        }
        self.pending.push(Node::Expr(expr));
    }

    fn visit_comprehension(&mut self, comprehension: &'a Comprehension) {
        self.visit_expr(&comprehension.iter);
        for condition in &comprehension.ifs {
            self.visit_expr(condition);
        }
    }

    fn visit_except_handler(&mut self, handler: &'a ExceptHandler) {
        let ExceptHandler::ExceptHandler(clause) = handler;
        if let Some(name) = &clause.name {
            self.bind(name);
        }
        walk_except_handler(self, handler);
    }

    fn visit_alias(&mut self, alias: &'a Alias) {
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

    fn visit_pattern(&mut self, pattern: &'a Pattern) {
        let captured = match pattern {
            Pattern::MatchAs(capture) => capture.name.as_ref(),
            Pattern::MatchStar(star) => star.name.as_ref(),
            Pattern::MatchMapping(mapping) => mapping.rest.as_ref(),
            _ => None,
        };
        if let Some(name) = captured {
            self.bind(name);
        }
        self.pending.push(Node::Pattern(pattern));
    }

    fn visit_interpolated_string_element(&mut self, element: &'a InterpolatedStringElement) {
        self.pending.push(Node::Element(element));
    }
}
