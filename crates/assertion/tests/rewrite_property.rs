use proptest::prelude::*;
use proptest::test_runner::RngSeed;
use purba_parsing::Module;
use ruff_python_ast::statement_visitor::{StatementVisitor, walk_stmt};
use ruff_python_ast::{Expr, Stmt, StmtAssert};
use ruff_text_size::Ranged;

/// An expression that can be the condition of an assert. Each operand that
/// an operator takes sits in parentheses, so each case parses.
fn condition() -> impl Strategy<Value = String> {
    let leaf = prop_oneof![
        Just("x"),
        Just("1"),
        Just("True"),
        Just("None"),
        Just("'a # b'"),
        Just("\"q\\\"\""),
        Just("b'\\x00'"),
        Just("..."),
    ]
    .prop_map(str::to_owned);
    leaf.prop_recursive(4, 32, 3, |inner| {
        let pair = (inner.clone(), inner.clone());
        prop_oneof![
            pair.clone().prop_map(|(a, b)| format!("({a}) == ({b})")),
            pair.clone().prop_map(|(a, b)| format!("({a}) and ({b})")),
            pair.clone().prop_map(|(a, b)| format!("({a}) + ({b})")),
            pair.prop_map(|(a, b)| format!("({a}) in ({b})")),
            inner.clone().prop_map(|a| format!("not ({a})")),
            inner.clone().prop_map(|a| format!("({a})")),
            inner.clone().prop_map(|a| format!("f({a})")),
            inner.clone().prop_map(|a| format!("x[{a}]")),
            inner.clone().prop_map(|a| format!("({a}).real")),
            inner.clone().prop_map(|a| format!("[{a}, {a}]")),
            inner.clone().prop_map(|a| format!("{{{a}: {a}}}")),
            inner.clone().prop_map(|a| format!("({a},)")),
            inner.clone().prop_map(|a| format!("(n := {a})")),
            inner.clone().prop_map(|a| format!("(lambda v: ({a}))")),
            inner.clone().prop_map(|a| format!("[v for v in {a}]")),
            inner.clone().prop_map(|a| format!("f\"{{ ({a}) }}\"")),
            inner
                .clone()
                .prop_map(|a| format!("({a}  # a comment\n        )")),
            inner.clone().prop_map(|a| format!("(yield ({a}))")),
            inner.clone().prop_map(|a| format!("(await ({a}))")),
            (inner.clone(), inner.clone(), inner)
                .prop_map(|(a, b, c)| format!("(({a}) if ({b}) else ({c}))")),
        ]
    })
}

/// A module whose one async test holds one to three asserts, each with or
/// without a message.
fn module() -> impl Strategy<Value = String> {
    prop::collection::vec((condition(), any::<bool>()), 1..4).prop_map(|asserts| {
        let mut text = String::from("async def test_case():\n");
        for (condition, message) in asserts {
            text.push_str("    assert ");
            text.push_str(&condition);
            if message {
                text.push_str(", 'why'");
            }
            text.push('\n');
        }
        text
    })
}

#[derive(Default)]
struct Asserts<'a> {
    found: Vec<&'a StmtAssert>,
}

impl<'a> StatementVisitor<'a> for Asserts<'a> {
    fn visit_stmt(&mut self, statement: &'a Stmt) {
        if let Stmt::Assert(assert) = statement {
            self.found.push(assert);
        }
        walk_stmt(self, statement);
    }
}

/// Each assert of `module`, in source order.
fn asserts(module: &Module) -> Vec<&StmtAssert> {
    let mut visitor = Asserts::default();
    visitor.visit_body(&module.syntax().body);
    visitor.found
}

fn parse(text: &str, what: &str) -> Result<Module, TestCaseError> {
    purba_parsing::parse_source(text.to_owned())
        .map_err(|error| TestCaseError::fail(format!("{what} parses: {error:?}\n{text}")))
}

/// Checks that the condition of `assert` is `_purba_ran(index, condition)`.
fn check_marked(
    rewritten: &Module,
    assert: &StmtAssert,
    index: usize,
    condition: &str,
) -> Result<(), TestCaseError> {
    let text = |node: &dyn Ranged| &rewritten.source()[node.range()];
    let Expr::Call(call) = &*assert.test else {
        return Err(TestCaseError::fail(format!(
            "assert {index} calls no mark: {}",
            text(&*assert.test)
        )));
    };
    prop_assert_eq!(
        text(&*call.func),
        "_purba_ran",
        "assert {} calls the mark",
        index
    );
    prop_assert!(
        call.arguments.keywords.is_empty(),
        "assert {} names no keyword",
        index
    );
    let [first, second] = &*call.arguments.args else {
        return Err(TestCaseError::fail(format!(
            "assert {index} passes two arguments"
        )));
    };
    prop_assert_eq!(
        text(first),
        index.to_string(),
        "assert {} passes its index",
        index
    );
    prop_assert_eq!(
        text(second),
        condition,
        "assert {} passes its condition",
        index
    );
    Ok(())
}

proptest! {
    #![proptest_config(ProptestConfig {
        cases: 4096,
        rng_seed: RngSeed::Fixed(849),
        failure_persistence: None,
        ..ProptestConfig::default()
    })]

    #[test]
    fn the_rewrite_parses_and_marks_each_condition_but_a_tuple_as_it_was_written(text in module()) {
        let original = parse(&text, "the module")?;
        let conditions: Vec<(&str, bool)> = asserts(&original)
            .iter()
            .map(|assert| {
                let tuple = matches!(&*assert.test, Expr::Tuple(tuple) if !tuple.elts.is_empty());
                (&original.source()[assert.test.range()], tuple)
            })
            .collect();

        let rewrite = purba_assertion::rewrite(&original, &[]);
        let rewritten = parse(rewrite.text(), "the rewrite")?;

        let found = asserts(&rewritten);
        prop_assert_eq!(found.len(), conditions.len(), "the rewrite holds as many asserts");
        let mut index = 0;
        for (assert, (condition, tuple)) in found.iter().zip(&conditions) {
            if *tuple {
                prop_assert_eq!(
                    &rewritten.source()[assert.test.range()],
                    *condition,
                    "an assert of a tuple with items stays as written"
                );
            } else {
                check_marked(&rewritten, assert, index, condition)?;
                index += 1;
            }
        }
    }
}

/// Each operator of one comparison.
const OPERATORS: [&str; 10] = [
    "==", "!=", "<", "<=", ">", ">=", "in", "not in", "is", "is not",
];

/// A module whose one test ends with an expectation that compares two
/// conditions by one operator. A test that is not async cannot `await`, so a
/// call takes the place of each `await`.
fn compared() -> impl Strategy<Value = String> {
    (
        condition(),
        prop::sample::select(&OPERATORS[..]),
        condition(),
    )
        .prop_map(|(left, op, right)| {
            format!("def test_case():\n    assert ({left}) {op} ({right})\n")
                .replace("(await ", "(id ")
        })
}

proptest! {
    #![proptest_config(ProptestConfig {
        cases: 4096,
        rng_seed: RngSeed::Fixed(1024),
        failure_persistence: None,
        ..ProptestConfig::default()
    })]

    #[test]
    fn the_rewrite_hands_each_side_of_a_comparison_as_it_was_written(text in compared()) {
        let original = parse(&text, "the module")?;
        let [assert] = &*asserts(&original) else {
            return Err(TestCaseError::fail("the module holds one assert"));
        };
        let Expr::Compare(compare) = &*assert.test else {
            return Err(TestCaseError::fail("the assert compares"));
        };
        let [left, right] = &*compare.operands else {
            return Err(TestCaseError::fail("the comparison has two sides"));
        };
        let left = &original.source()[left.range()];
        let right = &original.source()[right.range()];

        let rewrite = purba_assertion::rewrite(&original, &["test_case"]);
        let rewritten = parse(rewrite.text(), "the rewrite")?;
        prop_assert_eq!(
            rewritten.source().lines().count(),
            original.source().lines().count(),
            "the rewrite keeps each line"
        );
        let Some(Stmt::FunctionDef(test)) = rewritten.syntax().body.first() else {
            return Err(TestCaseError::fail("the rewrite holds the test"));
        };
        let Some(Stmt::Expr(statement)) = test.body.last() else {
            return Err(TestCaseError::fail("the expectation is an expression"));
        };
        let Expr::BoolOp(both) = &*statement.value else {
            return Err(TestCaseError::fail("the expectation asks the hook whether it reads"));
        };
        let Some(Expr::Call(call)) = both.values.last() else {
            return Err(TestCaseError::fail("the expectation calls the hook"));
        };
        let text = |node: &dyn Ranged| &rewritten.source()[node.range()];
        prop_assert_eq!(text(&*call.func), "_purba_read", "the hook reads");
        let [index, first, second] = &*call.arguments.args else {
            return Err(TestCaseError::fail("the hook takes its index and two sides"));
        };
        prop_assert_eq!(text(index), "0", "the index of the one expectation");
        prop_assert_eq!(text(first), left, "the left side, as written");
        prop_assert_eq!(text(second), right, "the right side, as written");
    }
}

/// One value of a check: a literal, or the name `x`.
fn value() -> impl Strategy<Value = &'static str> {
    prop::sample::select(&["1", "'ab'", "None", "[1]", "(1, 2)", "x"][..])
}

/// One check: a comparison of two values by one operator, or one of the four
/// reads.
fn one_check() -> impl Strategy<Value = String> {
    prop_oneof![
        (value(), prop::sample::select(&OPERATORS[..]), value())
            .prop_map(|(left, op, right)| format!("{left} {op} {right}")),
        value().prop_map(|reading| format!("not {reading}")),
        value().prop_map(|reading| format!("isinstance({reading}, str)")),
        (value(), value()).prop_map(|(reading, start)| format!("({reading}).startswith({start})")),
        (value(), value()).prop_map(|(reading, end)| format!("({reading}).endswith({end})")),
    ]
}

/// The test of an assert in each shape that parsing names: one check, an
/// `all()` or an `any()` of one check, another `all()` or `any()`, and a test
/// that is no check.
fn any_shape() -> impl Strategy<Value = String> {
    let function = prop::sample::select(&["all", "any"][..]);
    prop_oneof![
        one_check(),
        (function.clone(), one_check())
            .prop_map(|(function, check)| format!("{function}({check} for x in [1, 'ab', None])")),
        (function.clone(), one_check())
            .prop_map(|(function, check)| format!("{function}({check} for x in x if x)")),
        function.prop_map(|function| format!("{function}([x])")),
        value().prop_map(|reading| format!("bool({reading})")),
        (value(), value()).prop_map(|(left, right)| format!("{left} == 1 == {right}")),
    ]
}

/// Readers that fork none, since a Python thread always seems alive.
struct NoReader;

static NO_READER: NoReader = NoReader;

impl purba_assertion::Readers for NoReader {
    fn threaded(&self, _py: pyo3::Python<'_>) -> bool {
        true
    }

    fn fork(&self, _py: pyo3::Python<'_>) -> std::io::Result<Option<i32>> {
        unreachable!("a process with a thread forks no reader")
    }

    fn wait(&self, _pid: i32) -> purba_assertion::Verdict {
        unreachable!("a process with a thread forks no reader")
    }

    fn end(&self, _verdict: purba_assertion::Verdict) -> ! {
        unreachable!("no process is a reader")
    }
}

proptest! {
    #![proptest_config(ProptestConfig {
        cases: 1024,
        rng_seed: RngSeed::Fixed(1103),
        failure_persistence: None,
        ..ProptestConfig::default()
    })]

    #[test]
    fn the_call_of_each_expectation_hands_the_hook_the_values_of_its_check(test in any_shape()) {
        use pyo3::types::PyAnyMethods;
        let text = format!("x = 'ab'\n\ndef test_case():\n    assert {test}\n");
        let rewrite = purba_assertion::Rewrite::of_source(&text, &["test_case"])
            .map_err(|error| TestCaseError::fail(format!("the module parses: {error:?}")))?;
        pyo3::Python::initialize();
        // The hook panics when the values that the call hands it do not match
        // the check, and PyO3 resumes that panic here.
        let read = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            pyo3::Python::attach(|py| {
                // CPython warns of `is` with a literal when it compiles the module.
                let quiet = c"import warnings; warnings.simplefilter('ignore', SyntaxWarning)";
                py.run(quiet, None, None).expect("the filter is set");
                // The case before loaded a module under the same name.
                let fresh = c"import sys; sys.modules.pop('test_shape', None)";
                py.run(fresh, None, None).expect("the name is free");
                let path = std::path::Path::new("test_shape.py");
                let loaded = purba_assertion::load(py, path, &rewrite).expect("the module loads");
                let case = loaded.module().getattr("test_case").expect("the test is bound");
                let watched = loaded.watch(&NO_READER, || case.call0());
                (watched.result.is_ok(), watched.readings.in_order.len())
            })
        }));
        let Ok((returned, read)) = read else {
            return Err(TestCaseError::fail(format!("the hook panicked on: assert {test}")));
        };
        prop_assert!(
            !returned || read == 1,
            "a test that returns read its one expectation: assert {}",
            test
        );
    }
}
