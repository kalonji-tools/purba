use std::fs;
use std::path::PathBuf;

fn rewrite(name: &str, text: &str) -> purba_assertion::Rewrite {
    let path = PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join(name);
    fs::write(&path, text).expect("the test writes its module");
    let module = purba_parsing::parse(&path).expect("the module parses");
    purba_assertion::rewrite(&module, &[])
}

#[test]
fn each_assert_records_its_index() {
    let rewritten = rewrite(
        "test_a.py",
        concat!(
            "def test_one():\n",
            "    assert 1 == 1\n",
            "\n",
            "def test_two():\n",
            "    assert 1 == 2\n",
            "\n",
            "def test_three():\n",
            "    pass\n",
        ),
    );

    assert_eq!(
        rewritten.text(),
        concat!(
            "def test_one():\n",
            "    assert _purba_ran(0, 1 == 1)\n",
            "\n",
            "def test_two():\n",
            "    assert _purba_ran(1, 1 == 2)\n",
            "\n",
            "def test_three():\n",
            "    pass\n",
        ),
        "each assert, marked with its index in source order"
    );
    assert_eq!(
        rewritten.asserts(),
        ["assert 1 == 1", "assert 1 == 2"],
        "the source text of each assert, at its index"
    );
}

#[test]
fn a_nested_assert_is_marked_too() {
    let rewritten = rewrite(
        "test_nested.py",
        concat!(
            "def helper():\n",
            "    assert True\n",
            "\n",
            "class TestK:\n",
            "    def test_m(self):\n",
            "        if True:\n",
            "            assert True, \"why\"\n",
        ),
    );

    assert_eq!(
        rewritten.text(),
        concat!(
            "def helper():\n",
            "    assert _purba_ran(0, True)\n",
            "\n",
            "class TestK:\n",
            "    def test_m(self):\n",
            "        if True:\n",
            "            assert _purba_ran(1, True), \"why\"\n",
        ),
        "an assert in a helper, a method and a branch, with its message kept"
    );
}

#[test]
fn a_tuple_assert_stays_as_written_and_takes_no_index() {
    let rewritten = rewrite(
        "test_tuple.py",
        concat!(
            "def test_t():\n",
            "    assert (1 == 2, \"msg\")\n",
            "    assert 1 == 1\n",
            "    assert ()\n",
        ),
    );

    assert_eq!(
        rewritten.text(),
        concat!(
            "def test_t():\n",
            "    assert (1 == 2, \"msg\")\n",
            "    assert _purba_ran(0, 1 == 1)\n",
            "    assert _purba_ran(1, ())\n",
        ),
        "a tuple with items stays as written, so CPython warns; an empty tuple is marked"
    );
    assert_eq!(
        rewritten.asserts(),
        ["assert 1 == 1", "assert ()"],
        "only the marked asserts, at their index"
    );
}

#[test]
fn a_yield_keeps_its_parentheses() {
    let rewritten = rewrite("test_yield.py", "def test_g():\n    assert (yield 1)\n");

    assert_eq!(
        rewritten.text(),
        "def test_g():\n    assert (_purba_ran(0, (yield 1)))\n",
        "a call refuses a bare yield as an argument"
    );
}

#[test]
fn the_text_of_an_assert_leaves_out_its_comments_and_line_continuations() {
    let rewritten = rewrite(
        "test_shapes.py",
        concat!(
            "def test_comment():\n",
            "    assert (1  # a comment\n",
            "            and 0)\n",
            "\n",
            "def test_backslash():\n",
            "    assert 1 and \\\n",
            "        0\n",
            "\n",
            "def test_hash_in_a_string():\n",
            "    assert \"a # b\" == \"c\"\n",
        ),
    );

    assert_eq!(
        rewritten.asserts(),
        [
            "assert (1  \n            and 0)",
            "assert 1 and \n        0",
            "assert \"a # b\" == \"c\"",
        ],
        "each assert without its comment and its backslash, and a # in a string kept"
    );
}

#[test]
fn a_module_with_no_assert_is_unchanged() {
    let text = "def test_three():\n    pass\n";
    let rewritten = rewrite("test_none.py", text);

    assert_eq!(rewritten.text(), text, "the text as it was read");
    assert!(rewritten.asserts().is_empty(), "no assert to quote");
}

#[test]
fn a_text_that_no_file_holds_gives_the_rewrite_of_a_file_that_holds_it() {
    let text = "def test_one():\n    assert 1 == 1\n";
    let read = rewrite("test_of_source.py", text);

    let rewritten = purba_assertion::Rewrite::of_source(text, &[]).expect("the text parses");

    assert_eq!(rewritten.text(), read.text(), "the same rewritten text");
    assert_eq!(rewritten.asserts(), read.asserts(), "the same asserts");
}

#[test]
fn a_text_that_does_not_parse_gives_the_syntax_error() {
    let error = purba_assertion::Rewrite::of_source("def test_one(:\n    pass\n", &[]).err();

    assert!(
        matches!(error, Some(purba_parsing::Error::Syntax(_))),
        "a syntax error, not {error:?}"
    );
}

#[test]
fn the_asserts_at_the_end_of_a_test_are_its_expectations() {
    let rewritten = purba_assertion::Rewrite::of_source(
        concat!(
            "def helper():\n",
            "    assert True\n",
            "\n",
            "def test_one():\n",
            "    assert 1 == 1\n",
            "    total = 2\n",
            "    assert total == 2\n",
            "    assert total > 1, \"why\"\n",
        ),
        &["test_one"],
    )
    .expect("the text parses");

    assert_eq!(
        rewritten.text(),
        concat!(
            "def helper():\n",
            "    assert _purba_ran(0, True)\n",
            "\n",
            "def test_one():\n",
            "    assert _purba_ran(1, 1 == 1); _purba_step(5)\n",
            "    total = 2; _purba_step(6)\n",
            "    _purba_reads(0) and _purba_read(0, (total), (2))\n",
            "    _purba_reads(1) and _purba_read(1, (total), (1))\n",
        ),
        "an assert of a function that is no test, and one before another statement, keep a mark"
    );
    assert_eq!(
        rewritten.expectations("test_one"),
        ["assert total == 2", "assert total > 1, \"why\""],
        "the text of each expectation, with its message"
    );
    assert_eq!(
        rewritten.expectations("helper"),
        [] as [&str; 0],
        "a function that is no test has no expectation"
    );
}

#[test]
fn an_expectation_keeps_the_number_of_each_later_line() {
    let rewritten = purba_assertion::Rewrite::of_source(
        concat!(
            "def test_one():\n",
            "    total = 2; assert \\\n",
            "        total == 2, (\n",
            "        \"why\")\n",
            "    assert total\n",
        ),
        &["test_one"],
    )
    .expect("the text parses");

    assert_eq!(
        rewritten.text(),
        concat!(
            "def test_one():\n",
            "    total = 2; _purba_step(2); _purba_reads(0) and _purba_read(0, \n",
            "(total), (2)\n",
            ")\n",
            "    _purba_reads(1) and _purba_read(1, total)\n",
        ),
        "an expectation after `;` stays an expression, and its line breaks stay inside the call"
    );
    assert_eq!(
        rewritten.expectations("test_one"),
        [
            "assert \n        total == 2, (\n        \"why\")",
            "assert total"
        ],
        "the text of each expectation, without its line continuation"
    );
}

#[test]
fn the_assert_of_each_named_test_of_test_a_is_its_expectation() {
    let rewritten = purba_assertion::Rewrite::of_source(
        concat!(
            "def test_one():\n",
            "    assert 1 == 1\n",
            "\n",
            "def test_two():\n",
            "    assert 1 == 2\n",
            "\n",
            "def test_three():\n",
            "    pass\n",
        ),
        &["test_one", "test_two", "test_three"],
    )
    .expect("the text parses");

    assert_eq!(
        rewritten.text(),
        concat!(
            "def test_one():\n",
            "    _purba_reads(0) and _purba_read(0, (1), (1))\n",
            "\n",
            "def test_two():\n",
            "    _purba_reads(1) and _purba_read(1, (1), (2))\n",
            "\n",
            "def test_three():\n",
            "    pass; _purba_step(8)\n",
        ),
        "each assert, as an expectation with its index in source order"
    );
    assert_eq!(
        rewritten.asserts(),
        [] as [&str; 0],
        "no assert keeps a mark"
    );
}

#[test]
fn a_step_follows_each_statement_of_a_test_before_its_first_expectation() {
    let rewritten = purba_assertion::Rewrite::of_source(
        concat!(
            "def test_one():\n",
            "    \"\"\"Why.\"\"\"\n",
            "    def inner():\n",
            "        return 1\n",
            "    total = inner()\n",
            "    if total:\n",
            "        total = 2; print(total)\n",
            "    assert total == 2\n",
            "\n",
            "def helper():\n",
            "    total = 1\n",
        ),
        &["test_one"],
    )
    .expect("the text parses");

    assert_eq!(
        rewritten.text(),
        concat!(
            "def test_one():\n",
            "    \"\"\"Why.\"\"\"; _purba_step(2)\n",
            "    def inner():\n",
            "        return 1\n",
            "    total = inner(); _purba_step(5)\n",
            "    if total:\n",
            "        total = 2; _purba_step(7); print(total); _purba_step(7)\n",
            "    _purba_reads(0) and _purba_read(0, (total), (2))\n",
            "\n",
            "def helper():\n",
            "    total = 1\n",
        ),
        "a step after each simple statement of the test, at any depth but a def's body, \
         each naming its line; a function that is no test takes none"
    );
}

#[test]
fn an_assert_at_the_end_of_the_body_of_a_with_is_an_expectation_and_each_with_is_watched() {
    let rewritten = purba_assertion::Rewrite::of_source(
        concat!(
            "def test_one():\n",
            "    with open(\"f\") as handle, lock:\n",
            "        total = 1\n",
            "        assert total == 1\n",
            "    assert handle.closed\n",
        ),
        &["test_one"],
    )
    .expect("the text parses");

    assert_eq!(
        rewritten.text(),
        concat!(
            "def test_one():\n",
            "    with _purba_with(2, open(\"f\")) as handle, _purba_with(2, lock):\n",
            "        total = 1; _purba_step(3)\n",
            "        _purba_reads(0) and _purba_read(0, (total), (1))\n",
            "    _purba_reads(1) and _purba_read(1, handle.closed)\n",
        ),
        "each item of the with names its line, and the asserts at the end of its body and \
         after it are expectations"
    );
    assert_eq!(
        rewritten.expectations("test_one"),
        ["assert total == 1", "assert handle.closed"],
        "the expectation inside the with comes first"
    );
}

#[test]
fn a_comparison_hands_each_side_in_parentheses_and_keeps_each_line() {
    let rewritten = purba_assertion::Rewrite::of_source(
        concat!(
            "def test_one():\n",
            "    got = 1\n",
            "    assert (\n",
            "        got  # the reading\n",
            "    ) in (\n",
            "        1, 2)\n",
            "    assert not got\n",
        ),
        &["test_one"],
    )
    .expect("the text parses");

    assert_eq!(
        rewritten.text(),
        concat!(
            "def test_one():\n",
            "    got = 1; _purba_step(2)\n",
            "    _purba_reads(0) and _purba_read(0, \n",
            "(got)\n",
            ", ((\n",
            "        1, 2)))\n",
            "    _purba_reads(1) and _purba_read(1, not got)\n",
        ),
        "each side of one comparison goes to the hook in parentheses, without the operator, \
         and the line breaks between them stay; any other expectation hands its condition"
    );
}
