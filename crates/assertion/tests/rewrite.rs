use std::fs;
use std::path::PathBuf;

fn rewrite(name: &str, text: &str) -> purba_assertion::Rewrite {
    let path = PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join(name);
    fs::write(&path, text).expect("the test writes its module");
    let module = purba_parsing::parse(&path).expect("the module parses");
    purba_assertion::rewrite(&module)
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

    let rewritten = purba_assertion::Rewrite::of_source(text).expect("the text parses");

    assert_eq!(rewritten.text(), read.text(), "the same rewritten text");
    assert_eq!(rewritten.asserts(), read.asserts(), "the same asserts");
}

#[test]
fn a_text_that_does_not_parse_gives_the_syntax_error() {
    let error = purba_assertion::Rewrite::of_source("def test_one(:\n    pass\n").err();

    assert!(
        matches!(error, Some(purba_parsing::Error::Syntax(_))),
        "a syntax error, not {error:?}"
    );
}
