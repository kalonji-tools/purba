//! Selection: names the tests of a parsed module, refuses each one that purba
//! cannot call or that breaks a rule of its code, and imports nothing.

use std::collections::HashSet;
use std::fmt;
use std::path::{Path, PathBuf};

use purba_errors::Warning;
use purba_parsing::{Arms, Enter, Facts, Module, Node, Shape, TestFacts, origins, walk};
use ruff_python_ast::name::UnqualifiedName;
use ruff_python_ast::{
    Decorator, ExceptHandler, Expr, ExprContext, Parameters, Stmt, StmtFunctionDef,
};
use ruff_text_size::{Ranged, TextSize};

mod framework;
mod lookup;
mod patch;
pub mod proto;
mod suppression;

use lookup::{Lookup, Origin};
use suppression::Check;
pub use suppression::{Suppression, seen_by_the_run};

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
///     reaching: Vec::new(),
///     notices: Vec::new(),
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
    suppressed: Vec<Warning>,
    suppressions: Vec<Suppression>,
    reaching: Vec<Suppression>,
    notices: Vec<String>,
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
            reaching: Vec::new(),
            notices: Vec::new(),
        }
    }

    /// Each verdict of selection that a suppression turned into a warning, one
    /// for each check, in the order of its first reason.
    #[must_use]
    pub fn suppressed(&self) -> &[Warning] {
        &self.suppressed
    }

    /// Each notice of the test: a line that reads a private name, in source
    /// order. A notice refuses nothing.
    #[must_use]
    pub fn notices(&self) -> &[String] {
        &self.notices
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
                write!(formatter, "[{}] ", check.name)?;
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
    /// An `all()` or an `any()` of another shape: its line, its function, and
    /// the words of its shape.
    EachShape(usize, &'static str, &'static str),
    Computes(usize, String),
    Input(usize, String),
    Patched(usize, String),
    WithoutSpec(usize),
    Imported(usize, String),
    /// An `except` that only passes: its line, its type as written or `None`
    /// for a bare `except:`, and whether it is an `except*`.
    TypeAlone(usize, Option<String>, bool),
}

impl Reason {
    /// The strict check that gives this reason, or `None` for a reason that
    /// says purba cannot call the test, or for a fault of a suppression.
    const fn check(&self) -> Option<&'static Check> {
        Some(match self {
            Self::Decorated(_) => &suppression::FOREIGN_DECORATOR,
            Self::ReturnsValue => &suppression::RETURNED_VALUE,
            Self::Rebound(..) => &suppression::NAME_BOUND_AGAIN,
            Self::NoDef(..) => &suppression::NAME_WITHOUT_DEF,
            Self::Under(_) => &suppression::DEF_UNDER_BLOCK,
            Self::NotOneComparison(_) | Self::EachShape(..) => &suppression::NOT_ONE_COMPARISON,
            Self::Computes(..) | Self::Input(..) => &suppression::COMPUTED_EXPECTED,
            Self::Patched(..) => &suppression::PATCHED_NAME,
            Self::WithoutSpec(_) => &suppression::MOCK_WITHOUT_SPEC,
            Self::Imported(..) => &suppression::FOREIGN_FRAMEWORK,
            Self::TypeAlone(..) => &suppression::TYPE_ALONE,
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
fn suppress(reasons: Vec<Reason>, suppressions: &[Suppression]) -> (Vec<Reason>, Vec<Warning>) {
    if reasons.iter().any(Reason::cannot_call) {
        return (reasons, Vec::new());
    }
    let mut kept = Vec::new();
    let mut suppressed: Vec<Warning> = Vec::new();
    for reason in reasons {
        let found = reason
            .check()
            .and_then(|check| suppressions.iter().find(|named| named.check == check.name));
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
            None => suppressed.push(Warning {
                check: suppression.check.to_owned(),
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
                 and an expectation compares a reading by one operator, \
                 or reads not x, isinstance(x, T), x.startswith(y) or x.endswith(y)"
            ),
            Self::EachShape(line, function, shape) => write!(
                formatter,
                "line {line} is an {function}() {shape}, and an {function}() takes one for, no if"
            ),
            Self::Computes(line, text) => write!(
                formatter,
                "line {line} computes {text}, and an expected value is a literal"
            ),
            Self::Input(line, name) => write!(
                formatter,
                "line {line} compares to the input {name}, \
                 and an expected value is a literal until purba has cases"
            ),
            Self::Patched(line, target) => write!(
                formatter,
                "line {line} patches {target}, and a stand-in enters a test as an argument"
            ),
            Self::WithoutSpec(line) => write!(
                formatter,
                "line {line} makes a mock with no spec, \
                 and a mock has the spec of what it stands for"
            ),
            Self::Imported(line, module) => write!(
                formatter,
                "line {line} imports {module}, and a test module imports no other test framework"
            ),
            Self::TypeAlone(line, kind, star) => {
                type_alone(formatter, *line, kind.as_deref(), *star)
            }
        }
    }
}

/// Writes the words of an `except` that only passes, at `line`, of the type
/// `kind`, or of a bare `except:` when `kind` is `None`.
fn type_alone(
    formatter: &mut fmt::Formatter<'_>,
    line: usize,
    kind: Option<&str>,
    star: bool,
) -> fmt::Result {
    let star = if star { "*" } else { "" };
    match kind {
        Some(kind) => write!(
            formatter,
            "line {line} expects {kind} by its type alone; \
             bind except{star} {kind} as e: and check str(e)"
        ),
        None => write!(
            formatter,
            "line {line} expects any raise; bind except{star} SomeError as e: and check str(e)"
        ),
    }
}

/// The tests of `module` that selection hands on, each test it refuses, the
/// fault of its suppressions, and what became of each suppression.
///
/// Selection reads the module's suppressions once. It judges them at each test
/// that it refuses and that purba can call. Each list keeps source order.
/// `path` is the module's path as the tester typed it. A name defined twice is
/// one test at its first `def`. Selection judges the last `def` of the name at
/// the top, or the last `def` of the name under a statement when no `def` at
/// the top binds the name. A `def` under a
/// statement at the module's level is a test that selection refuses, unless a
/// `def` at the top binds the name.
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
pub fn select(path: &Path, facts: &Facts<'_>) -> Selected {
    if cfg!(feature = "fault")
        && std::env::var_os("PURBA_FAULT").is_some_and(|setting| setting == "selection")
    {
        panic!("PURBA_FAULT=selection");
    }
    let module = facts.module();
    let test_defs: Vec<TextSize> = facts
        .tests()
        .iter()
        .flat_map(|test| test.defs.iter().map(Ranged::start))
        .collect();
    let sites = suppression::sites(facts, &test_defs);
    let bindings = facts.bindings();
    let imported = facts.imports();
    let outside = outside_tests(facts);
    let mut tests = Vec::new();
    let mut refused = Vec::new();
    let mut places: HashSet<&str> = HashSet::new();
    for found in facts.tests().iter().filter(|test| test.classes.is_empty()) {
        let Some(function) = found.defs.last() else {
            continue;
        };
        places.insert(function.name.as_str());
        let first = found.first;
        let mut test = TestId::new(path, Vec::new(), function.name.to_string());
        let at: Vec<TextSize> = found.defs.iter().map(Ranged::start).collect();
        let mut reasons: Vec<Reason> = sites
            .faults_of(&at)
            .into_iter()
            .map(Reason::Fault)
            .collect();
        reasons.extend(found.under.map(Reason::Under));
        reasons.extend(reasons_of(function, facts, first));
        let own = found.facts();
        let lookup = Lookup::new(&function.body, &own.locals, imported, &outside.bound);
        reasons.extend(faults_of(function, own, module, &lookup, &outside));
        let suppressions = sites.of(&at);
        let (reasons, suppressed) = suppress(reasons, &suppressions);
        test.suppressed = suppressed;
        test.reaching.clone_from(&suppressions);
        test.suppressions = suppressions
            .into_iter()
            .filter(Suppression::seen_by_execution)
            .collect();
        if reasons.is_empty() {
            test.notices = notices_of(function, module, &lookup);
            tests.push(test);
        } else {
            refused.push((first, Refused { test, reasons }));
        }
    }
    refused.extend(refused_methods(path, facts, &outside, &sites));
    let never = never_a_function(facts.statements());
    for (name, found) in bindings {
        if !purba_parsing::is_test_name(name) || places.contains(name) {
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
    let refused: Vec<Refused> = refused.into_iter().map(|(_, refused)| refused).collect();
    let mut ledger = Ledger::new(&sites, &tests, &refused);
    for test in &refused {
        ledger.refused(test);
    }
    Selected {
        tests,
        refused,
        fault: sites.module_fault(),
        ledger,
    }
}

/// What selection made of a module: its tests, the tests that it refuses, the
/// fault of its suppressions, and what became of each suppression.
pub struct Selected {
    tests: Vec<TestId>,
    refused: Vec<Refused>,
    fault: Option<String>,
    ledger: Ledger,
}

impl Selected {
    /// The tests that selection selects, in source order.
    #[must_use]
    pub fn tests(&self) -> &[TestId] {
        &self.tests
    }

    /// The tests that selection refuses, in source order.
    #[must_use]
    pub fn refused(&self) -> &[Refused] {
        &self.refused
    }

    /// The fault of the module's suppressions, or `None` when there is none.
    ///
    /// It holds each fault of a suppression of the module, and each `# purba:`
    /// comment that sits above no test, joined by `; `.
    #[must_use]
    pub fn fault(&self) -> Option<&str> {
        self.fault.as_deref()
    }

    /// Judges each suppression that reaches `test`, one of a check that
    /// execution sees only when `judged`. A check that selection sees counts
    /// as used when the test suppressed it, and a check that execution sees
    /// when `warned` names it.
    pub fn judge(&mut self, test: &TestId, warned: &[&str], judged: bool) {
        self.ledger.judge(test, warned, judged);
    }

    /// The words of each unused suppression that purba judged at each test it
    /// reaches, in source order.
    #[must_use]
    pub fn unused(&self) -> Vec<String> {
        self.ledger.unused()
    }
}

/// Each suppression of a module, beside how many tests it reaches, at how many
/// of them purba judged it, and whether purba counts it as used.
struct Ledger {
    entries: Vec<Line>,
}

/// One suppression of a [`Ledger`], and what the run made of it.
struct Line {
    suppression: Suppression,
    reaches: usize,
    judged: usize,
    used: bool,
}

impl Ledger {
    /// The suppressions of `sites`, each beside how many tests of `tests` and
    /// `refused` it reaches.
    fn new(sites: &suppression::Sites, tests: &[TestId], refused: &[Refused]) -> Self {
        let mut entries: Vec<Line> = sites
            .all()
            .into_iter()
            .map(|suppression| Line {
                suppression,
                reaches: 0,
                judged: 0,
                used: false,
            })
            .collect();
        for test in tests.iter().chain(refused.iter().map(Refused::test)) {
            for reaching in &test.reaching {
                if let Some(entry) = entries
                    .iter_mut()
                    .find(|entry| entry.suppression.line == reaching.line)
                {
                    entry.reaches += 1;
                }
            }
        }
        Self { entries }
    }

    /// Judges each suppression that reaches `test`, as [`Selected::judge`] says.
    fn judge(&mut self, test: &TestId, warned: &[&str], judged: bool) {
        for reaching in &test.reaching {
            let Some(entry) = self
                .entries
                .iter_mut()
                .find(|entry| entry.suppression.line == reaching.line)
            else {
                continue;
            };
            let used = if reaching.seen_by_execution() {
                if !judged {
                    continue;
                }
                warned.contains(&reaching.check)
            } else {
                test.suppressed
                    .iter()
                    .any(|verdict| verdict.check == reaching.check)
            };
            entry.judged += 1;
            entry.used |= used;
        }
    }

    /// Judges each suppression of a check that selection sees that reaches the
    /// refused test, unless purba cannot call the test.
    fn refused(&mut self, refused: &Refused) {
        if !refused.reasons.iter().any(Reason::cannot_call) {
            self.judge(&refused.test, &[], false);
        }
    }

    /// The words of each unused suppression, as [`Selected::unused`] says.
    fn unused(&self) -> Vec<String> {
        self.entries
            .iter()
            .filter(|entry| !entry.used && entry.judged == entry.reaches)
            .map(|entry| {
                let Suppression {
                    check, line, test, ..
                } = &entry.suppression;
                test.as_ref().map_or_else(
                    || {
                        format!(
                            "line {line} suppresses {check}, which no test of the module breaks"
                        )
                    },
                    |test| format!("line {line} suppresses {check}, which {test} does not break"),
                )
            })
            .collect()
    }
}

/// The offset of each binding at the module's level, or in a block there, that
/// never holds a function: an assignment to the name alone of a literal, a
/// comprehension or a generator expression; an `import` with no `from`; a
/// `class` with no decorator; a `del`. `statements` holds each statement at the
/// module's level and in a block there.
fn never_a_function(statements: &[&Stmt]) -> HashSet<TextSize> {
    let mut found = HashSet::new();
    for statement in statements {
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
            _ => {}
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

/// Each test in a class of the module of `facts`, refused, beside the offset
/// of its first `def`. A name defined twice in one class is one test, and
/// selection judges its last `def`.
fn refused_methods(
    path: &Path,
    facts: &Facts<'_>,
    outside: &Outside<'_>,
    sites: &suppression::Sites,
) -> Vec<(TextSize, Refused)> {
    let module = facts.module();
    facts
        .tests()
        .iter()
        .filter(|test| !test.classes.is_empty())
        .filter_map(|found| {
            let function = found.defs.last()?;
            let test = TestId::new(
                path,
                found
                    .classes
                    .iter()
                    .map(|class| (*class).to_owned())
                    .collect(),
                function.name.to_string(),
            );
            let at = [found.first, function.start()];
            let mut reasons: Vec<Reason> = sites
                .faults_of(&at)
                .into_iter()
                .map(Reason::Fault)
                .collect();
            reasons.push(Reason::Method);
            reasons.extend(own_reasons(function, module, Some("self")));
            let own = found.facts();
            let lookup = Lookup::new(&function.body, &own.locals, facts.imports(), &outside.bound);
            reasons.extend(faults_of(function, own, module, &lookup, outside));
            Some((found.first, Refused { test, reasons }))
        })
        .collect()
}

/// Why purba cannot call `function`, from its `def` and its body. `first` is
/// where the first `def` of its name starts.
fn reasons_of(function: &StmtFunctionDef, facts: &Facts<'_>, first: TextSize) -> Vec<Reason> {
    let mut reasons = own_reasons(function, facts.module(), None);
    let lines = rebound(function, facts, first);
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

/// Each line that binds the name of `function` after the first `def` of that
/// name, which starts at `first`, or under `global` in any function, before or
/// after it. A later `def` is such a binding. In source order.
fn rebound(function: &StmtFunctionDef, facts: &Facts<'_>, first: TextSize) -> Vec<usize> {
    let name = function.name.as_str();
    let first_end = facts
        .defs()
        .iter()
        .find(|def| def.start() == first)
        .map_or(function, |def| def)
        .range
        .end();
    let mut offsets: Vec<TextSize> = facts
        .bindings()
        .get(name)
        .into_iter()
        .flatten()
        .filter(|binding| binding.in_function || binding.offset >= first_end)
        .map(|binding| binding.offset)
        .collect();
    offsets.sort_unstable();
    let mut lines: Vec<usize> = offsets
        .into_iter()
        .map(|offset| facts.module().position(offset).line())
        .collect();
    lines.dedup();
    lines
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
/// body of a nested `def`, `lambda` or `class` is not its own, but its head is,
/// as [`Enter::Head`] gives it.
#[derive(Default)]
struct Body {
    yields: bool,
    returns_value: bool,
}

impl Body {
    fn of(statements: &[Stmt]) -> Self {
        let mut body = Self::default();
        walk(statements.iter().map(Node::Stmt), &mut body);
        body
    }
}

impl<'a> Arms<'a> for Body {
    fn stmt(&mut self, statement: &'a Stmt) -> Enter {
        match statement {
            Stmt::FunctionDef(_) | Stmt::ClassDef(_) => return Enter::Head,
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
        Enter::All
    }

    fn expr(&mut self, expr: &'a Expr) -> Enter {
        match expr {
            Expr::Lambda(_) => return Enter::Head,
            Expr::Yield(_) | Expr::YieldFrom(_) => self.yields = true,
            _ => {}
        }
        Enter::All
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

/// Each fault of the rules that selection reads from the body of `function`,
/// in source order: an expectation that is not one comparison, an `all()` or an
/// `any()` of another shape, an expected value that is not a literal, a patch,
/// a mock with no spec, an import of another test framework, in the body or in
/// `outside`, and an `except` that only passes. The check of each item of an
/// `all()` or an `any()` keeps the rule of an expected value. Each patch comes
/// once for each target of a line, each mock with no spec once for each line,
/// and each import once for each module of a line.
fn faults_of<'a>(
    function: &'a StmtFunctionDef,
    own: &TestFacts<'a>,
    module: &Module,
    lookup: &Lookup<'a, '_>,
    outside: &Outside<'_>,
) -> Vec<Reason> {
    let line = |offset: TextSize| module.position(offset).line();
    let mut faults: Vec<(TextSize, Reason)> = Vec::new();
    let input = |side: &Expr| match side {
        Expr::Name(name) if own.parameters.contains(name.id.as_str()) => Some(name.id.to_string()),
        _ => None,
    };
    for (assert, shape) in &own.ending {
        let start = assert.start();
        let check = match *shape {
            Shape::Check(check) | Shape::Each { check, .. } => check,
            Shape::OtherEach { function, shape } => {
                faults.push((start, Reason::EachShape(line(start), function, shape)));
                continue;
            }
            Shape::Other => {
                faults.push((start, Reason::NotOneComparison(line(start))));
                continue;
            }
        };
        if let purba_parsing::Check::Compare {
            left,
            right,
            expected: None,
            ..
        } = check
        {
            let reason = input(right).or_else(|| input(left)).map_or_else(
                || Reason::Computes(line(start), on_one_line(&module.source()[right.range()])),
                |name| Reason::Input(line(start), name),
            );
            faults.push((start, reason));
        }
    }
    let mut targets = HashSet::new();
    let mut mocks = HashSet::new();
    for (offset, fault) in patch::faults(&function.body, lookup, module.source()) {
        let at = line(offset);
        match fault {
            patch::Fault::Patched(target) if targets.insert((at, target.clone())) => {
                faults.push((offset, Reason::Patched(at, target)));
            }
            patch::Fault::WithoutSpec if mocks.insert(at) => {
                faults.push((offset, Reason::WithoutSpec(at)));
            }
            _ => {}
        }
    }
    let mut modules = HashSet::new();
    for (offset, name) in framework::imports(&function.body, lookup)
        .into_iter()
        .chain(outside.imports.iter().cloned())
    {
        let at = line(offset);
        if modules.insert((at, name.clone())) {
            faults.push((offset, Reason::Imported(at, name)));
        }
    }
    for (offset, kind, star) in passing_excepts(own, module.source()) {
        faults.push((offset, Reason::TypeAlone(line(offset), kind, star)));
    }
    faults.sort_by_key(|(offset, _)| *offset);
    faults.into_iter().map(|(_, fault)| fault).collect()
}

/// Each `except` of each `try` of a test, as `own` holds them, whose body only
/// passes, by its offset, with its type as `source` writes it, or `None` for a
/// bare `except:`, and whether it is an `except*`. A tuple gains the
/// parentheses that `as` needs after it.
fn passing_excepts(own: &TestFacts<'_>, source: &str) -> Vec<(TextSize, Option<String>, bool)> {
    let mut found = Vec::new();
    for attempt in own.tries() {
        for handler in &attempt.handlers {
            let ExceptHandler::ExceptHandler(clause) = handler;
            if only_passes(&clause.body) {
                let kind = clause.type_.as_deref().map(|kind| {
                    let text = on_one_line(&source[kind.range()]);
                    match kind {
                        Expr::Tuple(tuple) if !tuple.parenthesized => format!("({text})"),
                        _ => text,
                    }
                });
                found.push((clause.start(), kind, attempt.is_star));
            }
        }
    }
    found
}

/// Whether each statement of `body` is `pass`, `...` or a string.
fn only_passes(body: &[Stmt]) -> bool {
    body.iter().all(|statement| match statement {
        Stmt::Pass(_) => true,
        Stmt::Expr(expression) => matches!(
            &*expression.value,
            Expr::EllipsisLiteral(_) | Expr::StringLiteral(_)
        ),
        _ => false,
    })
}

/// What a test reads of another test framework outside its own body.
struct Outside<'a> {
    /// Each import outside every test, by its offset.
    imports: Vec<(TextSize, String)>,
    /// Each name that the module binds, which is then no builtin.
    bound: HashSet<&'a str>,
}

/// Each import of another test framework in the module of `facts` outside
/// every test, by its offset: at the module's level, in a block there, in a
/// helper, in a class, and in what runs when a `def` at the module's level is
/// made. A test here is a `def` at the module's level with a `test` name.
fn outside_tests<'a>(facts: &Facts<'a>) -> Outside<'a> {
    let imported = facts.imports();
    let bound: HashSet<&str> = facts.bindings().keys().copied().collect();
    let none = HashSet::new();
    let mut found = Vec::new();
    for statement in &facts.module().syntax().body {
        match statement {
            Stmt::FunctionDef(function) => {
                let made = Lookup::new(&[], &none, imported, &bound);
                found.extend(framework::made(statement, &made));
                if !purba_parsing::is_test_name(&function.name) {
                    let locals = purba_parsing::locals(function);
                    let lookup = Lookup::new(&function.body, &locals, imported, &bound);
                    found.extend(framework::imports(&function.body, &lookup));
                }
            }
            other => {
                let statements = std::slice::from_ref(other);
                let lookup = Lookup::new(statements, &none, imported, &bound);
                found.extend(framework::imports(statements, &lookup));
            }
        }
    }
    Outside {
        imports: found,
        bound,
    }
}

/// The notice of each line of the body of `function` that reads a private
/// name, in source order: `line N reads the private name X`, once for each
/// name of a line.
fn notices_of<'a>(
    function: &'a StmtFunctionDef,
    module: &Module,
    lookup: &Lookup<'a, '_>,
) -> Vec<String> {
    let line = |offset: TextSize| module.position(offset).line();
    let mut found = Names::of(&function.body, lookup);
    found.sort_by_key(|(offset, _)| *offset);
    let mut seen = HashSet::new();
    found
        .into_iter()
        .filter(|(offset, name)| seen.insert((line(*offset), name.clone())))
        .map(|(offset, name)| format!("line {} reads the private name {name}", line(offset)))
        .collect()
}

/// Each private name that a test's body reads, by its offset: an attribute, a
/// name that a private import at the module's level binds and the body does
/// not, and a private origin of an `import` in the body. The body of a nested
/// `def`, `lambda` or `class` is the test's too.
struct Names<'a, 'l> {
    lookup: &'l Lookup<'a, 'l>,
    found: Vec<(TextSize, String)>,
}

impl<'a> Names<'a, '_> {
    fn of(statements: &'a [Stmt], lookup: &Lookup<'a, '_>) -> Vec<(TextSize, String)> {
        let mut names = Names {
            lookup,
            found: Vec::new(),
        };
        walk(statements.iter().map(Node::Stmt), &mut names);
        names.found
    }
}

impl<'a> Arms<'a> for Names<'a, '_> {
    fn stmt(&mut self, statement: &'a Stmt) -> Enter {
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
        Enter::All
    }

    fn expr(&mut self, expr: &'a Expr) -> Enter {
        match expr {
            Expr::Attribute(attribute) if is_private(&attribute.attr) => {
                self.found
                    .push((attribute.attr.start(), attribute.attr.to_string()));
            }
            Expr::Name(name) if name.ctx == ExprContext::Load => {
                if let Origin::Imported(origin) = self.lookup.origin(name.id.as_str())
                    && of_a_private_path(origin)
                {
                    self.found.push((name.start(), origin.to_owned()));
                }
            }
            _ => {}
        }
        Enter::All
    }
}
