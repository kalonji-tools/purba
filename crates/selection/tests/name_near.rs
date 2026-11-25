use std::fs;
use std::path::{Path, PathBuf};

fn write(name: &str, text: &str) -> PathBuf {
    let path = PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join(name);
    fs::write(&path, text).expect("the test writes its module");
    path
}

/// Each line that selection gives before the run, in its order, for the module
/// that `text` holds: a refused test beside its reasons, and a name near `test`
/// beside its words.
fn lines(name: &str, text: &str) -> Vec<String> {
    let file = write(name, text);
    let module = purba_parsing::parse(&file).expect("the module parses");
    let selected =
        purba_selection::select(Path::new("test_m.py"), &purba_parsing::Facts::of(&module));
    selected
        .before_the_run()
        .iter()
        .map(|line| match line {
            purba_selection::Before::Refused(refused) => format!("{} {refused}", refused.test()),
            purba_selection::Before::NameNear(near) => {
                format!("{}::{} {near}", near.path().display(), near.name())
            }
        })
        .collect()
}

/// The line of a name near `test`, for the function `name`.
fn near(name: &str) -> String {
    format!(
        "test_m.py::{name} [name-near] {name} is not a test, and its name is near the prefix test"
    )
}

#[test]
fn a_function_whose_first_word_is_one_edit_from_test_is_named_and_the_tests_stay() {
    let file = write(
        "test_near_kept.py",
        "def test_add():\n    assert 1 + 1 == 2\n\ndef tset_sub():\n    assert 2 - 1 == 1\n",
    );
    let module = purba_parsing::parse(&file).expect("the module parses");
    let selected =
        purba_selection::select(Path::new("test_m.py"), &purba_parsing::Facts::of(&module));

    let tests: Vec<String> = selected.tests().iter().map(ToString::to_string).collect();

    assert_eq!(
        tests,
        ["test_m.py::test_add"],
        "the error of a name stops no test"
    );
    assert_eq!(
        lines("test_near_record.py", "def tset_sub():\n    pass\n"),
        [near("tset_sub")],
        "the record's own example"
    );
}

#[test]
fn each_kind_of_edit_makes_a_name_near() {
    let cases = [
        ("text_format", "a changed letter"),
        ("tesst_a", "an added letter"),
        ("qtest_b", "an added letter at the start"),
        ("tst_c", "a dropped letter"),
        ("tset_d", "two neighbours swapped"),
        ("tset", "a name with no `_` is its own first word"),
    ];
    for (name, why) in cases {
        assert_eq!(
            lines(
                &format!("test_edit_{name}.py"),
                &format!("def {name}():\n    pass\n")
            ),
            [near(name)],
            "{why}: {name}"
        );
    }
}

#[test]
fn the_check_ignores_case() {
    for name in ["Test_add", "TEST_add", "Tset_add", "tEXT_add"] {
        assert_eq!(
            lines(
                &format!("test_case_{name}.py"),
                &format!("def {name}():\n    pass\n")
            ),
            [near(name)],
            "{name} starts with no `test`, and its first word is near `test` once case is ignored"
        );
    }
}

#[test]
fn a_name_two_edits_from_test_or_with_test_in_another_word_gives_no_line() {
    let cases = [
        (
            "format_text",
            "`test` is one edit from its last word, not its first",
        ),
        ("tasty_snack", "the first word is two edits from `test`"),
        ("toast", "two edits, and no `_`"),
        ("_tset", "the first word is empty"),
        ("helper", "a name far from `test`"),
        ("testing", "a name that starts with `test` is a test"),
    ];
    for (name, why) in cases {
        assert_eq!(
            lines(
                &format!("test_far_{name}.py"),
                &format!("def {name}():\n    pass\n")
            ),
            Vec::<String>::new(),
            "{why}: {name}"
        );
    }
}

#[test]
fn the_check_reads_each_def_at_the_modules_level_and_in_a_block_there() {
    assert_eq!(
        lines(
            "test_places.py",
            concat!(
                "async def tset_async():\n    pass\n\n",
                "if True:\n    def tset_under():\n        pass\n\n",
                "class Tset:\n    def tset_method(self):\n        pass\n\n",
                "def helper():\n    def tset_inner():\n        pass\n\n",
                "tset_bound = 1\n",
            ),
        ),
        [near("tset_async"), near("tset_under")],
        "an async def and a def under a statement at the module's level; \
         no method, no def in a def, no class and no binding with no def"
    );
}

#[test]
fn a_name_defined_twice_gives_one_line_at_its_first_def() {
    assert_eq!(
        lines(
            "test_near_twice.py",
            concat!(
                "def tset_a():\n    pass\n\n",
                "async def test_wait():\n    ...\n\n",
                "def tset_a():\n    pass\n",
            ),
        ),
        [
            near("tset_a"),
            "test_m.py::test_wait purba cannot run an async test yet".to_owned(),
        ],
        "one line for the name, above the refused test between its two defs"
    );
}

#[test]
fn the_line_sorts_in_source_order_among_the_refused_tests() {
    assert_eq!(
        lines(
            "test_near_order.py",
            concat!(
                "async def test_wait():\n    ...\n\n",
                "def tset_sub():\n    pass\n\n",
                "def test_mul(x):\n    ...\n\n",
                "def test_add():\n    assert 1 + 1 == 2\n",
            ),
        ),
        [
            "test_m.py::test_wait purba cannot run an async test yet".to_owned(),
            near("tset_sub"),
            "test_m.py::test_mul the parameter x has no default, \
             and purba cannot see which fixture builds it"
                .to_owned(),
        ],
        "a refused test above the name, the name, then a refused test below it"
    );
}

#[test]
fn a_suppression_above_a_name_near_test_sits_above_no_test() {
    let file = write(
        "test_near_suppressed.py",
        concat!(
            "# purba: suppress[foreign-decorator] a helper\n",
            "def tset_sub():\n    pass\n\n",
            "def test_add():\n    assert 1 + 1 == 2\n",
        ),
    );
    let module = purba_parsing::parse(&file).expect("the module parses");
    let selected =
        purba_selection::select(Path::new("test_m.py"), &purba_parsing::Facts::of(&module));

    assert_eq!(
        selected.fault(),
        Some("line 1 suppresses above tset_sub, which is no test"),
        "the suppression is a fault of the module"
    );
    assert_eq!(
        lines(
            "test_near_suppressed_again.py",
            concat!(
                "# purba: suppress[foreign-decorator] a helper\n",
                "def tset_sub():\n    pass\n",
            ),
        ),
        [near("tset_sub")],
        "no suppression reaches the error of the name"
    );
}

#[test]
fn a_suppression_of_the_module_does_not_reach_a_name_near_test() {
    assert_eq!(
        lines(
            "test_near_module_suppressed.py",
            concat!(
                "# purba: suppress-module[foreign-decorator] helpers\n",
                "import json\n\n",
                "def tset_sub():\n    pass\n",
            ),
        ),
        [near("tset_sub")],
        "the error of the name stays"
    );
}
