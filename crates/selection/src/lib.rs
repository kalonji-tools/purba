//! Selection: names the tests of a parsed module, refuses each one that purba
//! cannot call, and imports nothing.

use std::collections::HashMap;
use std::collections::hash_map::Entry;
use std::fmt;
use std::path::{Path, PathBuf};

use purba_parsing::Module;
use ruff_python_ast::visitor::{
    Visitor, walk_expr, walk_interpolated_string_element, walk_pattern, walk_stmt,
};
use ruff_python_ast::{
    Expr, InterpolatedStringElement, Parameters, Pattern, Stmt, StmtFunctionDef,
};

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
    Async,
    Generator,
    ReturnsValue,
    NoDefault(Vec<String>),
}

impl fmt::Display for Reason {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
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
    let mut tests = Vec::new();
    let mut refused = Vec::new();
    for function in last {
        let test = TestId {
            path: path.to_path_buf(),
            function: function.name.to_string(),
        };
        let reasons = reasons(function);
        if reasons.is_empty() {
            tests.push(test);
        } else {
            refused.push(Refused { test, reasons });
        }
    }
    (tests, refused)
}

/// Why purba cannot call `function`. A decorator can change what the call
/// takes and returns, so selection judges no decorated function.
fn reasons(function: &StmtFunctionDef) -> Vec<Reason> {
    if !function.decorator_list.is_empty() {
        return Vec::new();
    }
    let body = Body::of(&function.body);
    let mut reasons = Vec::new();
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
    reasons
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
            match node {
                Node::Stmt(statement) => walk_stmt(&mut body, statement),
                Node::Expr(expr) => walk_expr(&mut body, expr),
                Node::Pattern(pattern) => walk_pattern(&mut body, pattern),
                Node::Element(element) => walk_interpolated_string_element(&mut body, element),
            }
        }
        body
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
