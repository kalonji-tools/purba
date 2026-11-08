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
    fn the_rewrite_parses_and_marks_each_condition_as_it_was_written(text in module()) {
        let original = parse(&text, "the module")?;
        let conditions: Vec<&str> = asserts(&original)
            .iter()
            .map(|assert| &original.source()[assert.test.range()])
            .collect();

        let rewrite = purba_assertion::rewrite(&original);
        let rewritten = parse(rewrite.text(), "the rewrite")?;

        let marked = asserts(&rewritten);
        prop_assert_eq!(marked.len(), conditions.len(), "the rewrite holds as many asserts");
        for (index, (assert, condition)) in marked.iter().zip(&conditions).enumerate() {
            check_marked(&rewritten, assert, index, condition)?;
        }
    }
}
