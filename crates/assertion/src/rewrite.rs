//! The rewrite: each assert of a module, except a tuple assert, becomes a mark
//! or a reading, and each test gains its steps and the wraps of its `with`s.

use purba_parsing::{Arms, Enter, Facts, Module, Node, Reads, Shape, Side, TestFacts, walk};
use ruff_python_ast::{
    CmpOp, ExceptHandler, Expr, ExprContext, Stmt, StmtAssert, StmtFunctionDef, StmtTry,
};
use ruff_text_size::{Ranged, TextRange, TextSize};
use std::collections::{HashMap, HashSet};
use std::iter;
use std::ops::{Range, RangeInclusive};

use crate::{MISSED, RAN, READ, READS, STEP, WITH};

/// The name of the parameter of the function that gives each item of an
/// `all()` or an `any()` with the parts of its check.
const ITEMS: &str = "_purba_items";
/// The name that holds each item of an `all()` or an `any()` before its target
/// unpacks it, so that the rewrite writes the target once.
const ITEM: &str = "_purba_item";

/// The rewritten text of one module.
///
/// It holds each assert that it marks, each expectation of each test, and each
/// test. It holds the line of each statement in the body of each `with` of a
/// test.
pub struct Rewrite {
    pub(crate) text: String,
    /// Each marked assert, by the index that its mark records.
    pub(crate) marks: Vec<Mark>,
    /// Each expectation, by the index that its reading records.
    pub(crate) expectations: Vec<Expectation>,
    /// Each test, in the order of the `def` that binds it.
    pub(crate) tests: Vec<Tested>,
    pub(crate) bodies: HashMap<usize, Body>,
    pub(crate) missed: Vec<String>,
}

/// An assert that the rewrite marks: its text, without its comments and line
/// continuations, and its line.
#[derive(Clone, Debug)]
pub struct Mark {
    pub(crate) text: String,
    pub(crate) line: usize,
}

/// An expectation: its text, without its comments and line continuations, how
/// it checks its reading, and its line. It also holds each name that it reads
/// and that is no parameter of its test, and the indexes of the expectations of
/// its test.
#[derive(Clone, Debug)]
pub struct Expectation {
    pub(crate) text: String,
    pub(crate) check: Option<Check>,
    pub(crate) line: usize,
    pub(crate) names: Vec<String>,
    pub(crate) test: Range<usize>,
}

/// A test of the rewrite: its name, the indexes of its expectations, and each
/// `try` of it, in source order.
#[derive(Clone, Debug)]
pub struct Tested {
    pub(crate) name: String,
    pub(crate) expectations: Range<usize>,
    pub(crate) tries: Vec<Try>,
}

/// A `try` of a test: its lines, and the lines of its `except` when it is a
/// `try` of one path, which holds one simple statement and one
/// `except SomeError:`, with no `else` and no `finally`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Try {
    pub lines: RangeInclusive<usize>,
    pub except: Option<RangeInclusive<usize>>,
}

/// How an expectation checks its reading: by the operator of a comparison of
/// two values, with whether its expected value is the left side, since the left
/// is a literal and the right is not; by one of the four reads, with the text
/// of the type of `isinstance`; or by the check of each item of an `all()` or
/// an `any()`.
#[derive(Clone, Debug)]
pub enum Check {
    Compare { op: CmpOp, expected_left: bool },
    Read { read: Reads, kind: String },
    Each(Box<Items>),
}

/// How an `all()` or an `any()` checks each item: by the check of one item,
/// with the text of that check, of its side that reads the item, and of the
/// target. `bare` says that this side is the target itself. `empty` holds the words of an `all()`
/// whose reading holds no item.
#[derive(Clone, Debug)]
pub struct Items {
    pub(crate) any: bool,
    pub(crate) item: Check,
    pub(crate) label: String,
    pub(crate) side: String,
    pub(crate) target: String,
    pub(crate) bare: bool,
    pub(crate) empty: String,
}

/// The reading of `check`, of its parts `first` and `second`: the right side of
/// a comparison whose expected value is the left, and else the first part.
pub fn reading_side<T>(check: &Check, first: T, second: Option<T>) -> T {
    match (check, second) {
        (
            Check::Compare {
                expected_left: true,
                ..
            },
            Some(right),
        ) => right,
        _ => first,
    }
}

/// The text of `expr`, with each run of whitespace as one space.
fn spaced(expr: &Expr, source: &str) -> String {
    source[expr.range()]
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}

/// How `check`, one check that parsing names, checks its reading.
fn checked(check: purba_parsing::Check<'_>, source: &str) -> Check {
    match check {
        purba_parsing::Check::Compare { op, expected, .. } => Check::Compare {
            op,
            expected_left: expected == Some(Side::Left),
        },
        purba_parsing::Check::Read { read, other, .. } => Check::Read {
            read,
            kind: other.map_or_else(String::new, |other| spaced(other, source)),
        },
    }
}

/// How an assert of `shape` at `line` checks its reading, or `None` when
/// parsing names no check in it.
fn check_of(shape: Shape<'_>, source: &str, line: usize) -> Option<Check> {
    match shape {
        Shape::Check(check) => Some(checked(check, source)),
        Shape::Each {
            any,
            check,
            item: label,
            target,
            reading: over,
        } => {
            let item = checked(check, source);
            let over = spaced(over, source);
            let (first, second) = check.values();
            let side = reading_side(&item, first, second);
            let (side, target) = (spaced(side, source), spaced(target, source));
            Some(Check::Each(Box::new(Items {
                any,
                item,
                label: spaced(label, source),
                bare: side == target,
                side,
                target,
                empty: format!("line {line} checks each item of {over}, and {over} holds no item"),
            })))
        }
        Shape::OtherEach { .. } | Shape::Other => None,
    }
}

/// The body of a `with`: the line of each statement in it, at any depth, in
/// source order, and its last line.
#[derive(Clone, Debug, Default)]
pub struct Body {
    pub(crate) starts: Vec<usize>,
    pub(crate) last: usize,
}

impl Rewrite {
    /// The module's text, with each expectation rewritten, and each other
    /// assert marked except a tuple assert.
    #[must_use]
    pub fn text(&self) -> &str {
        &self.text
    }

    /// The text of each marked assert, without its comments and line continuations, in
    /// source order. The index of an assert here is the index that its mark
    /// records.
    #[must_use]
    pub fn asserts(&self) -> Vec<&str> {
        self.marks.iter().map(|mark| mark.text.as_str()).collect()
    }

    /// The text of each expectation of `test`, without its comments and line
    /// continuations, in source order. A function that is not a test of the
    /// rewrite has none.
    #[must_use]
    pub fn expectations(&self, test: &str) -> Vec<&str> {
        self.tests
            .iter()
            .find(|tested| tested.name == test)
            .map_or_else(Vec::new, |tested| {
                self.expectations[tested.expectations.clone()]
                    .iter()
                    .map(|expectation| expectation.text.as_str())
                    .collect()
            })
    }

    /// Parses `source`, the text of a module that no file holds, reads its
    /// facts, and rewrites it, with `tests` as the names of its tests.
    ///
    /// # Errors
    ///
    /// When `source` does not parse.
    pub fn of_source(source: &str, tests: &[&str]) -> Result<Self, purba_parsing::Error> {
        let module = purba_parsing::parse_source(source.to_owned())?;
        Ok(rewrite(&Facts::of(&module), tests))
    }
}

/// Whether `assert` tests a tuple with items, which is always true.
fn of_a_tuple(assert: &StmtAssert) -> bool {
    matches!(&*assert.test, Expr::Tuple(tuple) if !tuple.elts.is_empty())
}

/// Each assert of a module that the rewrite marks, in source order, at any
/// depth.
struct Asserts<'a> {
    expectations: &'a [&'a StmtAssert],
    found: Vec<&'a StmtAssert>,
}

impl<'a> Arms<'a> for Asserts<'a> {
    fn stmt(&mut self, statement: &'a Stmt) -> Enter {
        if let Stmt::Assert(assert) = statement
            && !of_a_tuple(assert)
            && !self
                .expectations
                .iter()
                .any(|expectation| std::ptr::eq(*expectation, assert))
        {
            self.found.push(assert);
        }
        Enter::All
    }

    fn expr(&mut self, _expr: &'a Expr) -> Enter {
        Enter::Nothing
    }
}

/// A test that the rewrite writes: the `def` that binds its name, its facts,
/// and its expectations, each with its shape.
struct Written<'m, 'f> {
    function: &'m StmtFunctionDef,
    own: &'f TestFacts<'m>,
    expectations: Vec<(&'m StmtAssert, Shape<'m>)>,
}

/// Each test of `facts` at the module's level, in no class and under no
/// statement, whose name `tests` names. They come in the order of the `def`
/// that binds each, and that `def` is no `async def`. Its expectations are the
/// asserts at the end of its body, except each tuple assert, which the rewrite
/// leaves as written.
fn written<'m, 'f>(facts: &'f Facts<'m>, tests: &[&str]) -> Vec<Written<'m, 'f>> {
    let mut found: Vec<Written<'m, 'f>> = facts
        .tests()
        .iter()
        .filter(|test| test.classes.is_empty() && test.under.is_none())
        .filter_map(|test| {
            let function = *test.defs.last()?;
            if function.is_async || !tests.contains(&function.name.as_str()) {
                return None;
            }
            let own = test.facts();
            let expectations = own
                .ending
                .iter()
                .filter(|(assert, _)| !of_a_tuple(assert))
                .copied()
                .collect();
            Some(Written {
                function,
                own,
                expectations,
            })
        })
        .collect();
    found.sort_by_key(|written| written.function.start());
    found
}

/// Whether `statement` is simple: it holds no other statement. A `def` and a
/// `class` hold a body, which is not one of the blocks that parsing names.
fn is_simple(statement: &Stmt) -> bool {
    purba_parsing::keyword(statement).is_none()
        && !matches!(statement, Stmt::FunctionDef(_) | Stmt::ClassDef(_))
}

/// One change of the rewrite: the text that takes the place of the source
/// from `start` to `end`. An insertion starts where it ends.
struct Edit {
    start: usize,
    end: usize,
    text: String,
}

impl Edit {
    const fn insert(at: usize, text: String) -> Self {
        Self {
            start: at,
            end: at,
            text,
        }
    }
}

/// Rewrites each assert of the module of `facts`, where `tests` names its
/// tests.
///
/// An expectation `assert x` becomes `_purba_reads(i) and _purba_read(i, x)`,
/// where `i` counts the expectations of the module in source order, from 0. An
/// expectation that compares two values by one operator, `assert a < b`, becomes
/// `_purba_reads(i) and _purba_read(i, (a), (b))`, and the hook compares them by
/// that operator. An expectation `assert all(c for t in r)`, or `any(…)`, of
/// one check `c` becomes `_purba_reads(i) and _purba_read(i, (r), f)`, where
/// `f` gives each item of `r` with the parts of `c`, and the hook checks each
/// item by `c`. Each keeps each line break of the assert inside the call, so
/// each later line keeps its number, and drops the assert's message.
///
/// Every other assert `assert x` becomes `assert _purba_ran(i, x)`, where `i`
/// counts the marked asserts of the module in source order, from 0. An assert
/// of a tuple with items stays as written, so that the compiler warns that it
/// is always true.
///
/// In a test, each simple statement that is no expectation gains
/// `; _purba_step(n)`, and each item `m` of a `with` becomes
/// `_purba_with(n, m)`, where `n` is the line of the statement. The body of a
/// `def` or a `class` in a test gains neither. Each line keeps its number.
///
/// # Panics
///
/// In a build with the feature `fault`, when `PURBA_FAULT` is `rewrite`.
#[must_use]
pub fn rewrite(facts: &Facts<'_>, tests: &[&str]) -> Rewrite {
    crate::fault("rewrite");
    let module = facts.module();
    let source = module.source();
    let line = |offset: TextSize| module.position(offset).line();
    let by_test = written(facts, tests);
    let all: Vec<&StmtAssert> = by_test
        .iter()
        .flat_map(|written| written.expectations.iter().map(|(assert, _)| *assert))
        .collect();
    let expected: HashSet<*const StmtAssert> = all
        .iter()
        .map(|assert| std::ptr::from_ref(*assert))
        .collect();
    let mut visitor = Asserts {
        expectations: &all,
        found: Vec::new(),
    };
    walk(module.syntax().body.iter().map(Node::Stmt), &mut visitor);

    let mut edits: Vec<Edit> = Vec::new();
    for (index, assert) in visitor.found.iter().enumerate() {
        let condition = assert.test.range();
        let mut text = format!("{RAN}({index}, ");
        push_condition(&mut text, source, assert);
        text.push(')');
        edits.push(Edit {
            start: condition.start().to_usize(),
            end: condition.end().to_usize(),
            text,
        });
    }
    let shaped = by_test
        .iter()
        .flat_map(|written| written.expectations.iter().copied());
    for (index, (assert, shape)) in shaped.enumerate() {
        edits.push(read_edit(index, assert, shape, source));
    }
    let bodies = watch_each_test(module, &by_test, &expected, &mut edits);
    let (missed, tries) = mark_each_try(module, &by_test, &mut edits);
    edits.sort_by_key(|edit| (edit.start, edit.end));

    let mut rewritten = String::with_capacity(source.len());
    let mut copied = 0;
    for edit in &edits {
        rewritten.push_str(&source[copied..edit.start]);
        rewritten.push_str(&edit.text);
        copied = edit.end;
    }
    rewritten.push_str(&source[copied..]);

    let mut expectations = Vec::with_capacity(all.len());
    let mut tested = Vec::with_capacity(by_test.len());
    for (written, tries) in by_test.iter().zip(tries) {
        let start = expectations.len();
        let range = start..start + written.expectations.len();
        for (assert, shape) in &written.expectations {
            let at = line(assert.start());
            expectations.push(Expectation {
                text: without_trivia(module, assert.range()),
                check: check_of(*shape, source, at),
                line: at,
                names: read_names(&assert.test)
                    .into_iter()
                    .filter(|name| !written.own.parameters.contains(name.as_str()))
                    .collect(),
                test: range.clone(),
            });
        }
        tested.push(Tested {
            name: written.function.name.to_string(),
            expectations: range,
            tries,
        });
    }
    Rewrite {
        text: rewritten,
        marks: visitor
            .found
            .iter()
            .map(|assert| Mark {
                text: without_trivia(module, assert.range()),
                line: line(assert.start()),
            })
            .collect(),
        expectations,
        tests: tested,
        bodies,
        missed,
    }
}

/// The edit that makes `assert`, the expectation at `index` of `shape`, a call
/// of the hook. A comparison of two values by one operator hands the hook each
/// side, and one of the four reads hands its reading and the type or the
/// argument that it takes. An `all()` or an `any()` of one check hands its
/// reading, and a function of that reading that gives each item with the parts
/// of its check.
fn read_edit(index: usize, assert: &StmtAssert, shape: Shape<'_>, source: &str) -> Edit {
    let whole = assert.range();
    let mut text = format!("{READS}({index}) and {READ}({index}, ");
    if let Shape::Each {
        check,
        target,
        reading,
        ..
    } = shape
    {
        let (first, second) = check.values();
        push_side(&mut text, source, reading);
        text.push_str(", lambda ");
        text.push_str(ITEMS);
        text.push_str(": ((");
        text.push_str(ITEM);
        for part in iter::once(first).chain(second) {
            text.push_str(", ");
            push_side(&mut text, source, part);
        }
        text.push_str(") for ");
        text.push_str(ITEM);
        text.push_str(" in ");
        text.push_str(ITEMS);
        text.push_str(" for ");
        push_side(&mut text, source, target);
        text.push_str(" in (");
        text.push_str(ITEM);
        text.push_str(",))");
        let breaks = source[whole].matches('\n').count();
        for _ in text.matches('\n').count()..breaks {
            text.push('\n');
        }
        text.push(')');
        return Edit {
            start: whole.start().to_usize(),
            end: whole.end().to_usize(),
            text,
        };
    }
    let mut copied = whole.start();
    if let Shape::Check(check) = shape {
        let (first, second) = check.values();
        for (side, start) in iter::once((first, "")).chain(second.map(|second| (second, ", "))) {
            push_breaks(&mut text, &source[TextRange::new(copied, side.start())]);
            text.push_str(start);
            push_side(&mut text, source, side);
            copied = side.end();
        }
    } else {
        let condition = assert.test.range();
        push_breaks(
            &mut text,
            &source[TextRange::new(copied, condition.start())],
        );
        push_condition(&mut text, source, assert);
        copied = condition.end();
    }
    push_breaks(&mut text, &source[TextRange::new(copied, whole.end())]);
    text.push(')');
    Edit {
        start: whole.start().to_usize(),
        end: whole.end().to_usize(),
        text,
    }
}

/// Writes the condition of `assert`.
fn push_condition(rewritten: &mut String, source: &str, assert: &StmtAssert) {
    // ruff's range of `(yield x)` leaves out the parentheses, and a yield
    // inside an expression is a `yield_atom`, which needs them.
    // https://docs.python.org/3.12/reference/expressions.html#yield-expressions
    let bare = matches!(*assert.test, Expr::Yield(_) | Expr::YieldFrom(_));
    let (open, close) = if bare { ("(", ")") } else { ("", "") };
    rewritten.push_str(open);
    rewritten.push_str(&source[assert.test.range()]);
    rewritten.push_str(close);
}

/// Writes `side`, one side of a comparison, in parentheses. ruff's range of
/// a side leaves out the parentheses around it, and the side may need them.
fn push_side(rewritten: &mut String, source: &str, side: &Expr) {
    rewritten.push('(');
    rewritten.push_str(&source[side.range()]);
    rewritten.push(')');
}

/// Writes a line feed for each line break in `text`.
fn push_breaks(rewritten: &mut String, text: &str) {
    for _ in text.matches('\n') {
        rewritten.push('\n');
    }
}

/// The text of `range` in `module`, without its comments and the backslash of
/// each line continuation.
fn without_trivia(module: &Module, range: TextRange) -> String {
    let source = module.source();
    let mut text = String::with_capacity(range.len().to_usize());
    let mut start = range.start();
    for trivia in module
        .trivia()
        .iter()
        .filter(|trivia| range.contains_range(**trivia))
    {
        text.push_str(&source[TextRange::new(start, trivia.start())]);
        start = trivia.end();
    }
    text.push_str(&source[TextRange::new(start, range.end())]);
    text
}

/// Each name that `test` reads and binds nowhere in it, such as a target of a
/// comprehension or a parameter of a `lambda` in it, in source order, once.
fn read_names(test: &Expr) -> Vec<String> {
    let mut names = Names::default();
    walk([Node::Expr(test)], &mut names);
    let Names { mut read, bound } = names;
    read.sort_by_key(|(offset, _)| *offset);
    let mut seen = HashSet::new();
    read.into_iter()
        .filter(|(_, name)| !bound.contains(name) && seen.insert(*name))
        .map(|(_, name)| name.to_owned())
        .collect()
}

/// The names of an expression: each name it reads, by its offset, and each
/// name it binds.
#[derive(Default)]
struct Names<'a> {
    read: Vec<(TextSize, &'a str)>,
    bound: HashSet<&'a str>,
}

impl<'a> Arms<'a> for Names<'a> {
    fn expr(&mut self, expr: &'a Expr) -> Enter {
        match expr {
            Expr::Name(name) if name.ctx == ExprContext::Load => {
                self.read.push((name.start(), name.id.as_str()));
            }
            Expr::Name(name) => {
                self.bound.insert(name.id.as_str());
            }
            Expr::Lambda(lambda) => {
                if let Some(parameters) = &lambda.parameters {
                    self.bound.extend(purba_parsing::parameters(parameters));
                }
            }
            _ => {}
        }
        Enter::All
    }
}

/// Adds to `edits` a step after each simple statement of each test of
/// `by_test` that is no expectation, and a wrap around each item of each of
/// its `with`s. Gives the body of each `with`, under its line. The statements
/// of a `with`'s body are those of the test that lie inside that body.
fn watch_each_test(
    module: &Module,
    by_test: &[Written<'_, '_>],
    expected: &HashSet<*const StmtAssert>,
    edits: &mut Vec<Edit>,
) -> HashMap<usize, Body> {
    let line = |offset: TextSize| module.position(offset).line();
    let mut bodies = HashMap::new();
    for written in by_test {
        let statements = &written.own.statements;
        for statement in statements {
            let at = line(statement.start());
            if let Stmt::With(with) = statement {
                for item in &with.items {
                    let context = item.context_expr.range();
                    edits.push(Edit::insert(
                        context.start().to_usize(),
                        format!("{WITH}({at}, "),
                    ));
                    edits.push(Edit::insert(context.end().to_usize(), ")".to_owned()));
                }
                let starts = match (with.body.first(), with.body.last()) {
                    (Some(first), Some(last)) => {
                        let body = TextRange::new(first.start(), last.end());
                        statements
                            .iter()
                            .filter(|inner| body.contains_range(inner.range()))
                            .map(|inner| line(inner.start()))
                            .collect()
                    }
                    _ => Vec::new(),
                };
                bodies.insert(
                    at,
                    Body {
                        starts,
                        last: with.body.last().map_or(at, |last| line(last.end())),
                    },
                );
            }
            let is_expectation = matches!(
                statement,
                Stmt::Assert(assert) if expected.contains(&std::ptr::from_ref(assert))
            );
            if is_simple(statement) && !is_expectation {
                edits.push(Edit::insert(
                    statement.end().to_usize(),
                    format!("; {STEP}({at})"),
                ));
            }
        }
    }
    bodies
}

/// The text of the type that `attempt` expects when it is a `try` of one path:
/// one simple statement, and one `except SomeError:` whose type is a name or a
/// dotted name, with or without `as`, and no `else` and no `finally`. Else
/// `None`.
fn one_path<'a>(attempt: &StmtTry, source: &'a str) -> Option<&'a str> {
    let ([statement], [ExceptHandler::ExceptHandler(handler)]) =
        (&*attempt.body, &*attempt.handlers)
    else {
        return None;
    };
    if attempt.is_star
        || !is_simple(statement)
        || !attempt.orelse.is_empty()
        || !attempt.finalbody.is_empty()
    {
        return None;
    }
    let kind = handler.type_.as_deref()?;
    let mut part = kind;
    loop {
        match part {
            Expr::Name(_) => return Some(&source[kind.range()]),
            Expr::Attribute(attribute) => part = &attribute.value,
            _ => return None,
        }
    }
}

/// Adds to `edits` a call of the hook of a missed `try` after the step of the
/// one statement of each `try` of one path of each test of `by_test`. Gives the
/// words of each such `try`, by its index, and each `try` of each test, in the
/// order of `by_test`.
fn mark_each_try(
    module: &Module,
    by_test: &[Written<'_, '_>],
    edits: &mut Vec<Edit>,
) -> (Vec<String>, Vec<Vec<Try>>) {
    let line = |offset: TextSize| module.position(offset).line();
    let mut missed = Vec::new();
    let mut tries = Vec::new();
    for written in by_test {
        let mut found = Vec::new();
        for attempt in written.own.tries() {
            let at = line(attempt.start());
            let lines = at..=line(attempt.end());
            let Some(kind) = one_path(attempt, module.source()) else {
                found.push(Try {
                    lines,
                    except: None,
                });
                continue;
            };
            edits.push(Edit::insert(
                attempt.body[0].end().to_usize(),
                format!("; {MISSED}({})", missed.len()),
            ));
            missed.push(format!(
                "the try at line {at} expects {kind}, and the statement did not raise it"
            ));
            let handler = &attempt.handlers[0];
            found.push(Try {
                lines,
                except: Some(line(handler.start())..=line(handler.end())),
            });
        }
        tries.push(found);
    }
    (missed, tries)
}
