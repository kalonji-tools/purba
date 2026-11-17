//! Selection: names the tests of a parsed module, refuses each one that purba
//! cannot call or that breaks a rule of its code, and imports nothing.

use std::collections::hash_map::Entry;
use std::collections::{HashMap, HashSet};
use std::fmt;
use std::iter;
use std::path::{Path, PathBuf};

use purba_parsing::{Each, Module};
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

mod framework;
mod patch;
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
    suppressed: Vec<Suppressed>,
    suppressions: Vec<Suppression>,
    reaching: Vec<Suppression>,
    notices: Vec<String>,
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
            reaching: Vec::new(),
            notices: Vec::new(),
        }
    }

    /// Each verdict of selection that a suppression turned into a warning, one
    /// for each check, in the order of its first reason.
    #[must_use]
    pub fn suppressed(&self) -> &[Suppressed] {
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
            Self::NotOneComparison(_) | Self::EachShape(..) => "not-one-comparison",
            Self::Computes(..) | Self::Input(..) => "computed-expected",
            Self::Patched(..) => "patched-name",
            Self::WithoutSpec(_) => "mock-without-spec",
            Self::Imported(..) => "foreign-framework",
            Self::TypeAlone(..) => "type-alone",
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

/// The tests of `module` that selection hands on, and each test it refuses.
///
/// Each list keeps source order. `path` is the module's path as the tester
/// typed it. A name defined twice is one test at its first `def`. Selection
/// judges the last `def` of the name at the top, or the last `def` of the name
/// under a statement when no `def` at the top binds the name. A `def` under a
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
pub fn select(path: &Path, module: &Module) -> (Vec<TestId>, Vec<Refused>) {
    if cfg!(feature = "fault")
        && std::env::var_os("PURBA_FAULT").is_some_and(|setting| setting == "selection")
    {
        panic!("PURBA_FAULT=selection");
    }
    let Found { last, places, defs } = found(module);
    let sites = suppression::sites(module, &test_defs(module, &defs));
    let bindings = bindings(module.syntax());
    let imported = imports(module.syntax());
    let outside = outside_tests(module.syntax(), &imported, &bindings);
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
        reasons.extend(faults_of(function, module, &imported, &outside));
        let suppressions = sites.of(at);
        let (reasons, suppressed) = suppress(reasons, &suppressions);
        test.suppressed = suppressed;
        test.reaching.clone_from(&suppressions);
        test.suppressions = suppressions
            .into_iter()
            .filter(Suppression::seen_by_execution)
            .collect();
        if reasons.is_empty() {
            test.notices = notices_of(function, module, &imported);
            tests.push(test);
        } else {
            refused.push((first, Refused { test, reasons }));
        }
    }
    refused.extend(refused_methods(path, module, &imported, &outside, &sites));
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

/// Each suppression of a module, beside how many tests it reaches, at how many
/// of them purba judged it, and whether purba counts it as used.
pub struct Ledger {
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
    /// The suppressions of `module`, each beside how many tests of `tests` and
    /// `refused` it reaches.
    #[must_use]
    pub fn new(module: &Module, tests: &[TestId], refused: &[Refused]) -> Self {
        let Found { defs, .. } = found(module);
        let mut entries: Vec<Line> = suppression::sites(module, &test_defs(module, &defs))
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

    /// Judges each suppression that reaches `test`, one of a check that execution
    /// sees only when `judged`. A check that selection sees counts as used when
    /// the test suppressed it, and a check that execution sees when `warned`
    /// names it.
    pub fn judge(&mut self, test: &TestId, warned: &[&str], judged: bool) {
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
    pub fn refused(&mut self, refused: &Refused) {
        if !refused.reasons.iter().any(Reason::cannot_call) {
            self.judge(&refused.test, &[], false);
        }
    }

    /// The words of each unused suppression that purba judged at each test it
    /// reaches, in source order.
    #[must_use]
    pub fn unused(&self) -> Vec<String> {
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
    /// For each name, the block of the first `def` that `found` keeps, the
    /// `def` that selection judges, and the offset of the first `def` of the
    /// name.
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
    let mut firsts: HashMap<&str, TextSize> = HashMap::new();
    for statement in statements {
        let found = match (statement, keyword(statement)) {
            (Stmt::FunctionDef(function), _) => vec![(None, function)],
            (_, Some(keyword)) => defs_under(statement)
                .into_iter()
                .map(|function| (Some(keyword), function))
                .collect(),
            (_, None) => Vec::new(),
        };
        for (under, function) in found {
            let name = function.name.as_str();
            if !name.starts_with("test") {
                continue;
            }
            let first = *firsts.entry(name).or_insert_with(|| function.range.start());
            if under.is_some() && top.contains(name) {
                continue;
            }
            defs.entry(name).or_default().push(function.range.start());
            match places.entry(name) {
                Entry::Occupied(place) => last[*place.get()].1 = function,
                Entry::Vacant(place) => {
                    place.insert(last.len());
                    last.push((under, function, first));
                }
            }
        }
    }
    // A name enters `last` at the first `def` that the loop keeps, which is a
    // `def` at the top when the name has one. The sort moves each name to its
    // first `def`, which can sit under a statement before that `def` at the top.
    last.sort_unstable_by_key(|&(_, _, first)| first);
    for (place, (_, function, _)) in last.iter().enumerate() {
        places.insert(function.name.as_str(), place);
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
    outside: &Outside<'_>,
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
            reasons.extend(faults_of(function, module, imported, outside));
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

/// Each line that binds the name of `function` after the first `def` of that
/// name, or under `global` in any function, before or after it. A later `def`
/// is such a binding. In source order.
fn rebound(
    function: &StmtFunctionDef,
    module: &Module,
    bindings: &HashMap<&str, Vec<Binding>>,
) -> Vec<usize> {
    let name = function.name.as_str();
    let first_end = module
        .syntax()
        .body
        .iter()
        .flat_map(|statement| match statement {
            Stmt::FunctionDef(top) => vec![top],
            _ => defs_under(statement),
        })
        .find(|def| def.name.as_str() == name)
        .unwrap_or(function)
        .range
        .end();
    let mut offsets: Vec<TextSize> = bindings
        .get(name)
        .into_iter()
        .flatten()
        .filter(|binding| binding.in_function || binding.offset >= first_end)
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

/// The name that an `import` of `name` binds: `import a.b` binds `a`.
fn bound(name: &Identifier) -> &str {
    name.as_str().split('.').next().unwrap_or_default()
}

/// Each name that an `import` binds at the module's level or in a block there,
/// with where it comes from.
fn imports(module: &ModModule) -> HashMap<&str, String> {
    let mut found = HashMap::new();
    let mut pending: Vec<&Stmt> = module.body.iter().rev().collect();
    while let Some(statement) = pending.pop() {
        match statement {
            Stmt::Import(_) | Stmt::ImportFrom(_) => {
                for (name, origin) in origins(statement) {
                    found.insert(bound(name), origin);
                }
            }
            Stmt::FunctionDef(_) | Stmt::ClassDef(_) => {}
            other => pending.extend(blocks(other).into_iter().flatten().rev()),
        }
    }
    found
}

/// Each fault of the rules that selection reads from the body of `function`,
/// in source order: an expectation that is not one comparison, an `all()` or an
/// `any()` of another shape, an expected value that is not a literal, a patch,
/// a mock with no spec, an import of another test framework, in the body or in
/// `outside`, and an `except` that only passes. The check of each item of an
/// `all()` or an `any()` keeps the rule of an expected value. Each patch comes
/// once for each target of a line, each mock with no spec once for each line,
/// and each import once for each module of a line.
fn faults_of(
    function: &StmtFunctionDef,
    module: &Module,
    imported: &HashMap<&str, String>,
    outside: &Outside<'_>,
) -> Vec<Reason> {
    let line = |offset: TextSize| module.position(offset).line();
    let mut faults: Vec<(TextSize, Reason)> = Vec::new();
    let inputs = parameters_of(function);
    let input = |side: &Expr| match side {
        Expr::Name(name) if inputs.contains(name.id.as_str()) => Some(name.id.to_string()),
        _ => None,
    };
    for assert in expectations(&function.body) {
        let start = assert.start();
        let check = match purba_parsing::each(&assert.test) {
            Some(Each::One { check, .. }) => check,
            Some(Each::Other { function, shape }) => {
                faults.push((start, Reason::EachShape(line(start), function, shape)));
                continue;
            }
            None => &*assert.test,
        };
        match check {
            Expr::Compare(compare) if compare.ops.len() == 1 => {
                let [left, right] = &*compare.operands else {
                    continue;
                };
                if purba_parsing::is_literal(left) || purba_parsing::is_literal(right) {
                    continue;
                }
                let reason = input(right).or_else(|| input(left)).map_or_else(
                    || Reason::Computes(line(start), on_one_line(&module.source()[right.range()])),
                    |name| Reason::Input(line(start), name),
                );
                faults.push((start, reason));
            }
            test if purba_parsing::read(test).is_some() => {}
            _ => faults.push((start, Reason::NotOneComparison(line(start)))),
        }
    }
    let mut targets = HashSet::new();
    let locals = locals_of(function);
    let mut mocks = HashSet::new();
    for (offset, fault) in patch::faults(&function.body, &locals, imported, module.source()) {
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
    for (offset, name) in framework::imports(&function.body, &locals, imported, &outside.bound)
        .into_iter()
        .chain(outside.imports.iter().cloned())
    {
        let at = line(offset);
        if modules.insert((at, name.clone())) {
            faults.push((offset, Reason::Imported(at, name)));
        }
    }
    for (offset, kind, star) in passing_excepts(&function.body, module.source()) {
        faults.push((offset, Reason::TypeAlone(line(offset), kind, star)));
    }
    faults.sort_by_key(|(offset, _)| *offset);
    faults.into_iter().map(|(_, fault)| fault).collect()
}

/// Each `except` in `body` whose body only passes, by its offset, with its type
/// as `source` writes it, or `None` for a bare `except:`, and whether it is an
/// `except*`. A tuple gains the parentheses that `as` needs after it. Each
/// block counts, at any depth, but not a nested `def` or `class`.
fn passing_excepts(body: &[Stmt], source: &str) -> Vec<(TextSize, Option<String>, bool)> {
    let mut found = Vec::new();
    let mut pending = vec![body];
    while let Some(block) = pending.pop() {
        for statement in block {
            if let Stmt::Try(attempt) = statement {
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
            pending.extend(blocks(statement));
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

/// Each import of another test framework in `module` outside every test, by
/// its offset: at the module's level, in a block there, in a helper, in a
/// class, and in what runs when a `def` at the module's level is made. A test
/// here is a `def` at the module's level with a `test` name.
fn outside_tests<'a>(
    module: &'a ModModule,
    imported: &HashMap<&'a str, String>,
    bindings: &HashMap<&'a str, Vec<Binding>>,
) -> Outside<'a> {
    let bound: HashSet<&str> = bindings.keys().copied().collect();
    let none = HashSet::new();
    let mut found = Vec::new();
    for statement in &module.body {
        match statement {
            Stmt::FunctionDef(function) => {
                found.extend(framework::made(function, imported, &bound));
                if !function.name.starts_with("test") {
                    found.extend(framework::imports(
                        &function.body,
                        &locals_of(function),
                        imported,
                        &bound,
                    ));
                }
            }
            other => found.extend(framework::imports(
                std::slice::from_ref(other),
                &none,
                imported,
                &bound,
            )),
        }
    }
    Outside {
        imports: found,
        bound,
    }
}

/// The name of each parameter of `function`.
fn parameters_of(function: &StmtFunctionDef) -> HashSet<&str> {
    let parameters = &function.parameters;
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

/// Each name that `function` binds: its parameters, and each name its body binds.
fn locals_of(function: &StmtFunctionDef) -> HashSet<&str> {
    let mut locals: HashSet<&str> = Scope::of(&function.body)
        .names
        .into_iter()
        .map(|(name, _)| name)
        .collect();
    locals.extend(parameters_of(function));
    locals
}

/// The notice of each line of the body of `function` that reads a private
/// name, in source order: `line N reads the private name X`, once for each
/// name of a line.
fn notices_of(
    function: &StmtFunctionDef,
    module: &Module,
    imported: &HashMap<&str, String>,
) -> Vec<String> {
    let line = |offset: TextSize| module.position(offset).line();
    let locals = locals_of(function);
    let mut found = Names::of(&function.body, &locals, imported);
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
                    && of_a_private_path(origin)
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
