//! Selection: names the tests of a parsed module, refuses each one that purba
//! cannot call or that breaks a rule of its code, and imports nothing.

use std::collections::hash_map::Entry;
use std::collections::{HashMap, HashSet};
use std::fmt;
use std::iter;
use std::path::{Path, PathBuf};

use purba_parsing::Module;
use ruff_python_ast::name::UnqualifiedName;
use ruff_python_ast::visitor::{
    Visitor, walk_except_handler, walk_expr, walk_interpolated_string_element, walk_pattern,
    walk_stmt,
};
use ruff_python_ast::{
    Alias, Comprehension, Decorator, ExceptHandler, Expr, ExprContext, Identifier,
    InterpolatedStringElement, ModModule, Parameters, Pattern, Stmt, StmtAssert, StmtFunctionDef,
};
use ruff_text_size::{Ranged, TextSize};

mod suppression;

pub use suppression::Suppression;

/// A test's identity. Only selection constructs one. It displays its path,
/// then each class on the way to the test, then its function, joined by `::`.
///
/// ```compile_fail,E0451
/// let id = purba_selection::TestId {
///     path: "test_a.py".into(),
///     classes: Vec::new(),
///     function: "test_one".into(),
///     suppressed: Vec::new(),
///     suppressions: Vec::new(),
/// };
/// ```
///
/// ```compile_fail,E0616
/// fn forge(id: &mut purba_selection::TestId) { id.path = "test_b.py".into(); }
/// ```
///
/// ```compile_fail,E0616
/// fn forge(id: &mut purba_selection::TestId) { id.function = "test_two".into(); }
/// ```
///
/// ```compile_fail,E0616
/// fn forge(id: &mut purba_selection::TestId) { id.classes.clear(); }
/// ```
#[derive(Clone)]
pub struct TestId {
    path: PathBuf,
    classes: Vec<String>,
    function: String,
    suppressed: Vec<Suppressed>,
    suppressions: Vec<Suppression>,
}

/// A violation of a strict check that a suppression turned into a warning: the
/// name of the check, its words, and the reason of the suppression.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Suppressed {
    pub check: &'static str,
    pub words: String,
    pub reason: String,
}

impl TestId {
    /// The test `function` of the module at `path`, in `classes`, with no
    /// suppression.
    fn new(path: &Path, classes: Vec<String>, function: String) -> Self {
        Self {
            path: path.to_path_buf(),
            classes,
            function,
            suppressed: Vec::new(),
            suppressions: Vec::new(),
        }
    }

    /// Each verdict of selection that a suppression turned into a warning, one
    /// for each check, in the order of its first reason.
    #[must_use]
    pub fn suppressed(&self) -> &[Suppressed] {
        &self.suppressed
    }

    /// Each suppression that reaches the test for a check that execution sees:
    /// its own, then the module's.
    #[must_use]
    pub fn suppressions(&self) -> &[Suppression] {
        &self.suppressions
    }

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
        write!(formatter, "{}", self.path.display())?;
        for class in &self.classes {
            write!(formatter, "::{class}")?;
        }
        write!(formatter, "::{}", self.function)
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
            if let Some(check) = reason.check() {
                write!(formatter, "[{check}] ")?;
            }
            write!(formatter, "{reason}")?;
        }
        Ok(())
    }
}

enum Reason {
    /// A fault of a suppression of the test.
    Fault(String),
    Decorated(Vec<String>),
    Async,
    Generator,
    ReturnsValue,
    NoDefault(Vec<String>),
    Rebound(String, Vec<usize>),
    NoDef(String, Vec<usize>),
    Under(&'static str),
    Method,
    NotOneComparison(usize),
    Computes(usize, String),
    Private(usize, String),
}

impl Reason {
    /// The name of the strict check that gives this reason, or `None` for a
    /// reason that says purba cannot call the test, or for a fault of a
    /// suppression.
    const fn check(&self) -> Option<&'static str> {
        Some(match self {
            Self::Decorated(_) => "foreign-decorator",
            Self::ReturnsValue => "returned-value",
            Self::Rebound(..) => "name-bound-again",
            Self::NoDef(..) => "name-without-def",
            Self::Under(_) => "def-under-block",
            Self::NotOneComparison(_) => "not-one-comparison",
            Self::Computes(..) => "computed-expected",
            Self::Private(..) => "private-name",
            Self::Fault(_) | Self::Async | Self::Generator | Self::NoDefault(_) | Self::Method => {
                return None;
            }
        })
    }

    /// Whether this reason says that purba cannot call the test.
    const fn cannot_call(&self) -> bool {
        matches!(
            self,
            Self::Async | Self::Generator | Self::NoDefault(_) | Self::Method
        )
    }
}

/// `reasons` without each reason that a suppression of `suppressions` turns
/// into a warning, and the warnings that it gives. A test that purba cannot call
/// keeps every reason, since no suppression reaches it.
fn suppress(reasons: Vec<Reason>, suppressions: &[Suppression]) -> (Vec<Reason>, Vec<Suppressed>) {
    if reasons.iter().any(Reason::cannot_call) {
        return (reasons, Vec::new());
    }
    let mut kept = Vec::new();
    let mut suppressed: Vec<Suppressed> = Vec::new();
    for reason in reasons {
        let found = reason
            .check()
            .and_then(|check| suppressions.iter().find(|named| named.check == check));
        let Some(suppression) = found else {
            kept.push(reason);
            continue;
        };
        let words = reason.to_string();
        match suppressed
            .iter_mut()
            .find(|verdict| verdict.check == suppression.check)
        {
            Some(verdict) => {
                verdict.words.push_str("; ");
                verdict.words.push_str(&words);
            }
            None => suppressed.push(Suppressed {
                check: suppression.check,
                words,
                reason: suppression.reason.clone(),
            }),
        }
    }
    (kept, suppressed)
}

impl fmt::Display for Reason {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Fault(words) => formatter.write_str(words),
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
            Self::NoDef(name, lines) if lines.len() == 1 => write!(
                formatter,
                "line {} binds {name} with no def, and purba cannot see what the name holds",
                lines[0]
            ),
            Self::NoDef(name, lines) => {
                let lines: Vec<String> = lines.iter().map(ToString::to_string).collect();
                write!(
                    formatter,
                    "lines {} bind {name} with no def, and purba cannot see what the name holds",
                    lines.join(", ")
                )
            }
            Self::Method => formatter.write_str("purba cannot call a method of a class"),
            Self::Under(keyword) => write!(
                formatter,
                "the def sits under `{keyword}`, and purba cannot see whether import binds it"
            ),
            Self::NotOneComparison(line) => write!(
                formatter,
                "line {line} is not one comparison, \
                 and an expectation compares a reading by one operator"
            ),
            Self::Computes(line, text) => write!(
                formatter,
                "line {line} computes {text}, and an expected value is a literal"
            ),
            Self::Private(line, name) => write!(
                formatter,
                "line {line} reads the private name {name}, and a test reads no private name"
            ),
        }
    }
}

/// The tests of `module` that selection hands on, and each test it refuses.
///
/// Each list keeps source order. `path` is the module's path as the tester
/// typed it. A name defined twice is one test, at its first definition: import
/// binds the name there and binds it again at each later `def`, so selection
/// judges the last body. A `def` under a statement at the module's level is a
/// test that selection refuses, unless a `def` at the top binds the name.
/// A `test` name that the module binds with no `def` is a test that selection
/// refuses, where a binding of it can hold a function. A refused test sits at
/// its first `def`, or else at its first binding that can hold one.
///
/// A `def` with a `test` name in a class is a test that selection refuses,
/// since purba calls a test by the name of one function. A class counts at the
/// module's level, in a block there, or in such a class. The test's identity
/// names each class on the way. Its line gives the reasons of its `def` and its
/// body, but none for `self`. A binding of its name at the module's level and a
/// statement above its class give no reason either.
///
/// # Panics
///
/// In a build with the feature `fault`, when `PURBA_FAULT` is `selection`.
#[must_use]
pub fn select(path: &Path, module: &Module) -> (Vec<TestId>, Vec<Refused>) {
    if cfg!(feature = "fault")
        && std::env::var_os("PURBA_FAULT").is_some_and(|setting| setting == "selection")
    {
        panic!("PURBA_FAULT=selection");
    }
    let Found { last, places, defs } = found(module);
    let sites = suppression::sites(module, &test_defs(module, &defs));
    let bindings = bindings(module.syntax());
    let imported = private_imports(module.syntax());
    let mut tests = Vec::new();
    let mut refused = Vec::new();
    for (under, function, first) in last {
        let mut test = TestId::new(path, Vec::new(), function.name.to_string());
        let at = defs
            .get(function.name.as_str())
            .map_or(&[][..], Vec::as_slice);
        let mut reasons: Vec<Reason> = sites.faults_of(at).into_iter().map(Reason::Fault).collect();
        reasons.extend(under.map(Reason::Under));
        reasons.extend(reasons_of(function, module, &bindings));
        reasons.extend(faults_of(function, module, &imported));
        let suppressions = sites.of(at);
        let (reasons, suppressed) = suppress(reasons, &suppressions);
        test.suppressed = suppressed;
        test.suppressions = suppressions
            .into_iter()
            .filter(Suppression::seen_by_execution)
            .collect();
        if reasons.is_empty() {
            tests.push(test);
        } else {
            refused.push((first, Refused { test, reasons }));
        }
    }
    refused.extend(refused_methods(path, module, &imported, &sites));
    let never = never_a_function(module.syntax());
    for (name, found) in &bindings {
        if !name.starts_with("test") || places.contains_key(name) {
            continue;
        }
        let mut offsets: Vec<TextSize> = found
            .iter()
            .map(|binding| binding.offset)
            .filter(|offset| !never.contains(offset))
            .collect();
        if offsets.is_empty() {
            continue;
        }
        offsets.sort_unstable();
        let mut lines: Vec<usize> = offsets
            .iter()
            .map(|offset| module.position(*offset).line())
            .collect();
        lines.dedup();
        let test = TestId::new(path, Vec::new(), (*name).to_string());
        refused.push((
            offsets[0],
            Refused {
                test,
                reasons: vec![Reason::NoDef((*name).to_string(), lines)],
            },
        ));
    }
    refused.sort_by_key(|(offset, _)| *offset);
    (
        tests,
        refused.into_iter().map(|(_, refused)| refused).collect(),
    )
}

/// The fault of the module's suppressions, or `None` when there is none.
///
/// It holds each fault of a suppression of the module, and each `# purba:`
/// comment that sits above no test, joined by `; `.
#[must_use]
pub fn module_fault(module: &Module) -> Option<String> {
    let Found { defs, .. } = found(module);
    suppression::sites(module, &test_defs(module, &defs)).module_fault()
}

/// The tests of a module at its top or under a statement there.
struct Found<'a> {
    /// For each name, the block it sits under, the `def` that import binds
    /// last, and the offset of its first `def`.
    last: Vec<(Option<&'static str>, &'a StmtFunctionDef, TextSize)>,
    /// The place of each name in `last`.
    places: HashMap<&'a str, usize>,
    /// The offset of each `def` of each name.
    defs: HashMap<&'a str, Vec<TextSize>>,
}

/// The tests of `module` at its top or under a statement there. A `def` under a
/// statement is a test only when no `def` at the top binds its name.
fn found(module: &Module) -> Found<'_> {
    let statements = &module.syntax().body;
    let top: HashSet<&str> = statements
        .iter()
        .filter_map(|statement| match statement {
            Stmt::FunctionDef(function) => Some(function.name.as_str()),
            _ => None,
        })
        .collect();
    let mut last: Vec<(Option<&'static str>, &StmtFunctionDef, TextSize)> = Vec::new();
    let mut places: HashMap<&str, usize> = HashMap::new();
    let mut defs: HashMap<&str, Vec<TextSize>> = HashMap::new();
    for statement in statements {
        let found = match (statement, keyword(statement)) {
            (Stmt::FunctionDef(function), _) => vec![(None, function)],
            (_, Some(keyword)) => defs_under(statement)
                .into_iter()
                .filter(|function| !top.contains(function.name.as_str()))
                .map(|function| (Some(keyword), function))
                .collect(),
            (_, None) => Vec::new(),
        };
        for (under, function) in found {
            if !function.name.starts_with("test") {
                continue;
            }
            defs.entry(function.name.as_str())
                .or_default()
                .push(function.range.start());
            match places.entry(function.name.as_str()) {
                Entry::Occupied(place) => last[*place.get()].1 = function,
                Entry::Vacant(place) => {
                    place.insert(last.len());
                    last.push((under, function, function.range.start()));
                }
            }
        }
    }
    Found { last, places, defs }
}

/// The offset of each `def` of `module` that is a test: each of `defs`, and
/// each test in a class.
fn test_defs(module: &Module, defs: &HashMap<&str, Vec<TextSize>>) -> Vec<TextSize> {
    defs.values()
        .flatten()
        .copied()
        .chain(
            methods_of(&module.syntax().body)
                .into_iter()
                .map(|(_, function)| function.range.start()),
        )
        .collect()
}

/// The offset of each binding at the module's level, or in a block there, that
/// never holds a function: an assignment to the name alone of a literal, a
/// comprehension or a generator expression; an `import` with no `from`; a
/// `class` with no decorator; a `del`.
fn never_a_function(module: &ModModule) -> HashSet<TextSize> {
    let mut found = HashSet::new();
    let mut pending: Vec<&Stmt> = module.body.iter().rev().collect();
    while let Some(statement) = pending.pop() {
        match statement {
            Stmt::Assign(assign) if is_never_a_function(&assign.value) => {
                for target in &assign.targets {
                    if let Expr::Name(name) = target {
                        found.insert(name.start());
                    }
                }
            }
            Stmt::AnnAssign(assign) if assign.value.as_deref().is_some_and(is_never_a_function) => {
                if let Expr::Name(name) = &*assign.target {
                    found.insert(name.start());
                }
            }
            Stmt::Import(import) => {
                for alias in &import.names {
                    found.insert(
                        alias
                            .asname
                            .as_ref()
                            .map_or_else(|| alias.name.start(), Ranged::start),
                    );
                }
            }
            // A decorator can bind the name of its class to a function.
            // https://docs.python.org/3/reference/compound_stmts.html#class-definitions
            Stmt::ClassDef(class) if class.decorator_list.is_empty() => {
                found.insert(class.name.start());
            }
            Stmt::Delete(delete) => {
                for target in &delete.targets {
                    if let Expr::Name(name) = target {
                        found.insert(name.start());
                    }
                }
            }
            Stmt::FunctionDef(_) => {}
            other => pending.extend(blocks(other).into_iter().flatten().rev()),
        }
    }
    found
}

/// Whether `value` is never a function.
const fn is_never_a_function(value: &Expr) -> bool {
    matches!(
        value,
        Expr::List(_)
            | Expr::Tuple(_)
            | Expr::Dict(_)
            | Expr::Set(_)
            | Expr::StringLiteral(_)
            | Expr::BytesLiteral(_)
            | Expr::FString(_)
            | Expr::NumberLiteral(_)
            | Expr::BooleanLiteral(_)
            | Expr::NoneLiteral(_)
            | Expr::EllipsisLiteral(_)
            | Expr::ListComp(_)
            | Expr::SetComp(_)
            | Expr::DictComp(_)
            | Expr::Generator(_)
    )
}

/// The keyword of a statement at the module's level whose blocks import may
/// run or skip.
const fn keyword(statement: &Stmt) -> Option<&'static str> {
    match statement {
        Stmt::If(_) => Some("if"),
        Stmt::Try(_) => Some("try"),
        Stmt::With(_) => Some("with"),
        Stmt::For(_) => Some("for"),
        Stmt::While(_) => Some("while"),
        Stmt::Match(_) => Some("match"),
        _ => None,
    }
}

/// Each `def` in the blocks of `statement`, at any depth, in source order. The
/// body of a `def` or a `class` is not one of its blocks.
fn defs_under(statement: &Stmt) -> Vec<&StmtFunctionDef> {
    let mut found = Vec::new();
    // A walk that recurses once for each level of nesting overflows the stack
    // on statements that nest deeper than the stack.
    let mut pending = vec![statement];
    while let Some(statement) = pending.pop() {
        if let Stmt::FunctionDef(function) = statement {
            found.push(function);
        } else {
            pending.extend(blocks(statement).into_iter().flatten().rev());
        }
    }
    found
}

/// The blocks of `statement` that import may run or skip, in source order. The
/// body of a `def` or a `class` is not one of them.
fn blocks(statement: &Stmt) -> Vec<&[Stmt]> {
    match statement {
        Stmt::If(branch) => iter::once(branch.body.as_slice())
            .chain(
                branch
                    .elif_else_clauses
                    .iter()
                    .map(|clause| clause.body.as_slice()),
            )
            .collect(),
        Stmt::Try(attempt) => iter::once(attempt.body.as_slice())
            .chain(attempt.handlers.iter().map(|handler| {
                let ExceptHandler::ExceptHandler(clause) = handler;
                clause.body.as_slice()
            }))
            .chain([attempt.orelse.as_slice(), attempt.finalbody.as_slice()])
            .collect(),
        Stmt::With(with) => vec![with.body.as_slice()],
        Stmt::For(each) => vec![each.body.as_slice(), each.orelse.as_slice()],
        Stmt::While(each) => vec![each.body.as_slice(), each.orelse.as_slice()],
        Stmt::Match(choice) => choice
            .cases
            .iter()
            .map(|case| case.body.as_slice())
            .collect(),
        _ => Vec::new(),
    }
}

/// Each test in a class of `module`, refused, beside the offset of its first
/// `def`. A name defined twice in one class is one test, and selection judges
/// its last `def`.
fn refused_methods(
    path: &Path,
    module: &Module,
    imported: &HashMap<&str, String>,
    sites: &suppression::Sites,
) -> Vec<(TextSize, Refused)> {
    let mut methods: Vec<(Vec<&str>, &StmtFunctionDef, TextSize)> = Vec::new();
    let mut places: HashMap<(Vec<&str>, &str), usize> = HashMap::new();
    for (classes, function) in methods_of(&module.syntax().body) {
        match places.entry((classes.clone(), function.name.as_str())) {
            Entry::Occupied(place) => methods[*place.get()].1 = function,
            Entry::Vacant(place) => {
                place.insert(methods.len());
                methods.push((classes, function, function.range.start()));
            }
        }
    }
    methods
        .into_iter()
        .map(|(classes, function, first)| {
            let test = TestId::new(
                path,
                classes.into_iter().map(str::to_owned).collect(),
                function.name.to_string(),
            );
            let at = [first, function.range.start()];
            let mut reasons: Vec<Reason> = sites
                .faults_of(&at)
                .into_iter()
                .map(Reason::Fault)
                .collect();
            reasons.push(Reason::Method);
            reasons.extend(own_reasons(function, module, Some("self")));
            reasons.extend(faults_of(function, module, imported));
            (first, Refused { test, reasons })
        })
        .collect()
}

/// Each `def` with a `test` name in the body of a class, beside the name of
/// each class on the way, in source order. A class counts at the module's
/// level, in a block there, or in such a class. A class in a `def` does not
/// count.
fn methods_of(statements: &[Stmt]) -> Vec<(Vec<&str>, &StmtFunctionDef)> {
    let mut found = Vec::new();
    // A walk that recurses once for each level of nesting overflows the stack
    // on statements that nest deeper than the stack.
    let mut pending: Vec<(&Stmt, Vec<&str>)> = statements
        .iter()
        .rev()
        .map(|statement| (statement, Vec::new()))
        .collect();
    while let Some((statement, classes)) = pending.pop() {
        match statement {
            Stmt::FunctionDef(function) => {
                if !classes.is_empty() && function.name.starts_with("test") {
                    found.push((classes, function));
                }
            }
            Stmt::ClassDef(class) => {
                let mut inner = classes;
                inner.push(class.name.as_str());
                pending.extend(
                    class
                        .body
                        .iter()
                        .rev()
                        .map(|statement| (statement, inner.clone())),
                );
            }
            _ => pending.extend(
                blocks(statement)
                    .into_iter()
                    .flatten()
                    .rev()
                    .map(|statement| (statement, classes.clone())),
            ),
        }
    }
    found
}

/// Why purba cannot call `function`, from its `def` and its body.
fn reasons_of(
    function: &StmtFunctionDef,
    module: &Module,
    bindings: &HashMap<&str, Vec<Binding>>,
) -> Vec<Reason> {
    let mut reasons = own_reasons(function, module, None);
    let lines = rebound(function, module, bindings);
    if !lines.is_empty() {
        reasons.push(Reason::Rebound(function.name.to_string(), lines));
    }
    reasons
}

/// Why purba cannot call `function`, from its `def` and its body alone, with
/// no reason for the parameter `aside`.
fn own_reasons(function: &StmtFunctionDef, module: &Module, aside: Option<&str>) -> Vec<Reason> {
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
    let mut without_default = without_default(&function.parameters);
    without_default.retain(|name| Some(name.as_str()) != aside);
    if !without_default.is_empty() {
        reasons.push(Reason::NoDefault(without_default));
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
        || on_one_line(&source[expr.range()]),
        |name| name.to_string(),
    )
}

/// `text` on one line: each line trimmed, joined by a space.
fn on_one_line(text: &str) -> String {
    text.lines().map(str::trim).collect::<Vec<_>>().join(" ")
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

/// Whether `name` is private: it starts with `_`, and is not a dunder such as
/// `__len__`.
fn is_private(name: &str) -> bool {
    name.starts_with('_') && !(name.len() > 4 && name.starts_with("__") && name.ends_with("__"))
}

/// Whether a part of the dotted `path` is private.
fn of_a_private_path(path: &str) -> bool {
    path.split('.').any(is_private)
}

/// Where each name of an `import` comes from, by the name it binds. `import a.b`
/// binds `a`, which comes from `a`.
/// <https://docs.python.org/3.12/reference/simple_stmts.html#the-import-statement>
fn origins(statement: &Stmt) -> Vec<(&Identifier, String)> {
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

/// Each name that an `import` at the module's level, or in a block there,
/// binds from a private origin, by the name, beside that origin.
fn private_imports(module: &ModModule) -> HashMap<&str, String> {
    let mut found = HashMap::new();
    let mut pending: Vec<&Stmt> = module.body.iter().rev().collect();
    while let Some(statement) = pending.pop() {
        match statement {
            Stmt::Import(_) | Stmt::ImportFrom(_) => {
                for (name, origin) in origins(statement) {
                    if of_a_private_path(&origin) {
                        found.insert(name.as_str(), origin);
                    }
                }
            }
            Stmt::FunctionDef(_) | Stmt::ClassDef(_) => {}
            other => pending.extend(blocks(other).into_iter().flatten().rev()),
        }
    }
    found
}

/// Each fault of the rules that selection reads from the body of `function`,
/// in source order: an expectation that is not one comparison, an expected
/// value that is not a literal, and a private name.
fn faults_of(
    function: &StmtFunctionDef,
    module: &Module,
    imported: &HashMap<&str, String>,
) -> Vec<Reason> {
    let line = |offset: TextSize| module.position(offset).line();
    let mut faults: Vec<(TextSize, Reason)> = Vec::new();
    for assert in expectations(&function.body) {
        let start = assert.start();
        match &*assert.test {
            Expr::Compare(compare) if compare.ops.len() == 1 => {
                let [left, right] = &*compare.operands else {
                    continue;
                };
                if !purba_parsing::is_literal(left) && !purba_parsing::is_literal(right) {
                    let text = on_one_line(&module.source()[right.range()]);
                    faults.push((start, Reason::Computes(line(start), text)));
                }
            }
            _ => faults.push((start, Reason::NotOneComparison(line(start)))),
        }
    }
    let mut locals: HashSet<&str> = Scope::of(&function.body)
        .names
        .into_iter()
        .map(|(name, _)| name)
        .collect();
    let parameters = &function.parameters;
    locals.extend(
        parameters
            .posonlyargs
            .iter()
            .chain(&parameters.args)
            .chain(&parameters.kwonlyargs)
            .map(|parameter| parameter.parameter.name.as_str())
            .chain(parameters.vararg.iter().map(|vararg| vararg.name.as_str()))
            .chain(parameters.kwarg.iter().map(|kwarg| kwarg.name.as_str())),
    );
    let mut seen = HashSet::new();
    for (offset, name) in Names::of(&function.body, &locals, imported) {
        if seen.insert((line(offset), name.clone())) {
            faults.push((offset, Reason::Private(line(offset), name)));
        }
    }
    faults.sort_by_key(|(offset, _)| *offset);
    faults.into_iter().map(|(_, fault)| fault).collect()
}

/// Each private name that a test's body reads, by its offset: an attribute, a
/// name that a private import at the module's level binds and the body does
/// not, and a private origin of an `import` in the body. The body of a nested
/// `def`, `lambda` or `class` is the test's too.
struct Names<'a, 'b> {
    locals: &'b HashSet<&'a str>,
    imported: &'b HashMap<&'a str, String>,
    found: Vec<(TextSize, String)>,
    pending: Vec<Node<'a>>,
}

impl<'a, 'b> Names<'a, 'b> {
    fn of(
        statements: &'a [Stmt],
        locals: &'b HashSet<&'a str>,
        imported: &'b HashMap<&'a str, String>,
    ) -> Vec<(TextSize, String)> {
        let mut names = Self {
            locals,
            imported,
            found: Vec::new(),
            pending: Vec::new(),
        };
        for statement in statements {
            names.visit_stmt(statement);
        }
        while let Some(node) = names.pending.pop() {
            walk_node(&mut names, node);
        }
        names.found
    }
}

// As in `Body`, each visit pushes its node, and `Names::of` walks each node it
// pushed.
impl<'a> Visitor<'a> for Names<'a, '_> {
    fn visit_stmt(&mut self, statement: &'a Stmt) {
        if let Stmt::Import(import) = statement {
            // In the body, `import a._b` names `a._b`, though it binds `a`.
            for alias in &import.names {
                if of_a_private_path(&alias.name) {
                    self.found.push((alias.start(), alias.name.to_string()));
                }
            }
        } else {
            for (name, origin) in origins(statement) {
                if of_a_private_path(&origin) {
                    self.found.push((name.start(), origin));
                }
            }
        }
        self.pending.push(Node::Stmt(statement));
    }

    fn visit_expr(&mut self, expr: &'a Expr) {
        match expr {
            Expr::Attribute(attribute) if is_private(&attribute.attr) => {
                self.found
                    .push((attribute.attr.start(), attribute.attr.to_string()));
            }
            Expr::Name(name) if name.ctx == ExprContext::Load => {
                if let Some(origin) = self.imported.get(name.id.as_str())
                    && !self.locals.contains(name.id.as_str())
                {
                    self.found.push((name.start(), origin.clone()));
                }
            }
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

/// The expectations at the end of `body`: the asserts at its end, with no other
/// statement after them, after those at the end of the body of a `with` just
/// before them, at any depth. Each comes in source order.
fn expectations(body: &[Stmt]) -> Vec<&StmtAssert> {
    let mut segments: Vec<Vec<&StmtAssert>> = Vec::new();
    let mut current = body;
    loop {
        let mut asserts: Vec<&StmtAssert> = current
            .iter()
            .rev()
            .map_while(|statement| match statement {
                Stmt::Assert(assert) => Some(assert),
                _ => None,
            })
            .collect();
        asserts.reverse();
        let before = current.len() - asserts.len();
        segments.push(asserts);
        match before.checked_sub(1).map(|last| &current[last]) {
            Some(Stmt::With(with)) if !with.is_async => current = &with.body,
            _ => break,
        }
    }
    segments.into_iter().rev().flatten().collect()
}
