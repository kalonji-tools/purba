use std::ffi::OsStr;
use std::fs;
use std::path::PathBuf;

use purba_config::{Level, Project};
use purba_selection::{NoTestFile, Pattern, Patterns, TEST_FILES, test_files};

/// The four patterns of the closed list, in the order the errors name them.
const CLOSED: &str = "test_*.py, *_test.py, tests.py and test.py";

/// The words of an item `item` outside the closed list, whose nearest pattern
/// is `nearest`.
fn outside(item: &str, nearest: &str) -> String {
    format!(
        "test-files under [tool.purba] holds {item}, which is none of {CLOSED}; \
         the nearest is {nearest}"
    )
}

/// The words of a value of the kind `kind` in place of a list.
fn not_a_list(kind: &str) -> String {
    format!(r#"test-files under [tool.purba] is {kind}; write a list, such as ["*_test.py"]"#)
}

/// The table under `[tool.purba]` that `text` holds.
fn table(text: &str) -> toml::Table {
    text.parse().expect("the test writes TOML")
}

/// `test-files` as the project's file `text` sets it.
fn resolved(text: &str) -> Result<(Patterns, Level), Vec<String>> {
    Project::new(table(text))
        .resolve(&TEST_FILES)
        .map(|resolved| (resolved.value, resolved.level))
}

#[test]
fn with_no_key_the_list_is_test_prefix_alone() {
    assert_eq!(
        resolved(""),
        Ok((Patterns::from(vec![Pattern::TestPrefix]), Level::Default)),
        "no key gives test_*.py, from no level"
    );
    assert_eq!(
        Patterns::default(),
        Patterns::from(vec![Pattern::TestPrefix]),
        "the default list is test_*.py alone"
    );
}

#[test]
fn the_project_sets_each_pattern_of_the_closed_list_in_its_order() {
    assert_eq!(
        resolved(r#"test-files = ["test.py", "tests.py", "*_test.py", "test_*.py", "test.py"]"#),
        Ok((
            Patterns::from(vec![
                Pattern::Test,
                Pattern::Tests,
                Pattern::TestSuffix,
                Pattern::TestPrefix,
                Pattern::Test,
            ]),
            Level::Project
        )),
        "each item of the closed list is taken, a repeat too, from the project's level"
    );
}

#[test]
fn each_refused_value_reads_in_its_words() {
    for (text, words) in [
        (
            r#"test-files = ["test*.py"]"#,
            vec![outside("test*.py", "test_*.py")],
        ),
        (r#"test-files = "*_test.py""#, vec![not_a_list("a string")]),
        ("test-files = 5", vec![not_a_list("a number")]),
        ("test-files = 1.5", vec![not_a_list("a number")]),
        ("test-files = true", vec![not_a_list("a boolean")]),
        ("test-files = 1979-05-27", vec![not_a_list("a date")]),
        ("[test-files]", vec![not_a_list("a table")]),
        (
            "test-files = []",
            vec![format!(
                "test-files under [tool.purba] is an empty list, so no file is a test file; \
                 list at least one of {CLOSED}"
            )],
        ),
        (
            r#"test-files = ["tset_*.py", "*_test.py", "tests_*.py", 7, "*_Test.py", 1.5, true]"#,
            vec![
                outside("tset_*.py", "test_*.py"),
                outside("tests_*.py", "test_*.py"),
                outside("7", "test.py"),
                outside("*_Test.py", "*_test.py"),
                outside("1.5", "test.py"),
                outside("true", "test.py"),
            ],
        ),
    ] {
        assert_eq!(resolved(text), Err(words), "{text}");
    }
}

#[test]
fn each_pattern_matches_only_its_names() {
    for (pattern, matched, unmatched) in [
        (
            Pattern::TestPrefix,
            &["test_a.py", "test_.py"][..],
            &[
                "test.py",
                "tests.py",
                "a_test.py",
                "test_a.pyc",
                "Test_a.py",
            ][..],
        ),
        (
            Pattern::TestSuffix,
            &["a_test.py", "b_c_test.py"][..],
            &["_test.py", "test_a.py", "a_test.pyc", "atest.py"][..],
        ),
        (
            Pattern::Tests,
            &["tests.py"][..],
            &["test.py", "tests.pyc", "a_tests.py"][..],
        ),
        (
            Pattern::Test,
            &["test.py"][..],
            &["tests.py", "test_a.py", "a_test.py"][..],
        ),
    ] {
        for name in matched {
            assert!(
                pattern.matches(OsStr::new(name)),
                "{pattern} matches {name}"
            );
        }
        for name in unmatched {
            assert!(
                !pattern.matches(OsStr::new(name)),
                "{pattern} does not match {name}"
            );
        }
    }
}

#[test]
fn the_walk_gives_only_the_files_that_a_pattern_of_the_list_matches() {
    let root = PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join("test_files_dial");
    if root.exists() {
        fs::remove_dir_all(&root).expect("the test removes its old folder");
    }
    fs::create_dir_all(root.join(".git")).expect("the test makes .git");
    fs::create_dir_all(root.join("tests")).expect("the test makes tests");
    for name in ["test_a.py", "b_test.py", "tests.py", "_test.py"] {
        fs::write(root.join("tests").join(name), "").expect("the test writes a file");
    }
    let root = root
        .canonicalize()
        .expect("the folder has a canonical path");
    let patterns = Patterns::from(vec![Pattern::TestSuffix, Pattern::Tests]);

    let found = test_files(&root, None, &patterns).expect("the walk reads the root");

    assert_eq!(
        found.files,
        vec![
            PathBuf::from("tests/b_test.py"),
            PathBuf::from("tests/tests.py")
        ],
        "only *_test.py and tests.py, and not test_*.py"
    );
    assert_eq!(
        test_files(&root, Some(&PathBuf::from("tests/test_a.py")), &patterns),
        Err(NoTestFile::Name(patterns.clone())),
        "a named path that matches no pattern of the list names no test file"
    );
}

#[test]
fn the_reason_of_a_name_names_each_pattern_of_the_list() {
    for (patterns, words) in [
        (
            vec![Pattern::TestSuffix],
            "its name does not match *_test.py",
        ),
        (
            vec![Pattern::TestSuffix, Pattern::TestSuffix],
            "its name does not match *_test.py",
        ),
        (
            vec![Pattern::TestPrefix, Pattern::TestSuffix],
            "its name matches none of test_*.py and *_test.py",
        ),
        (
            vec![Pattern::TestPrefix, Pattern::Tests, Pattern::Test],
            "its name matches none of test_*.py, tests.py and test.py",
        ),
    ] {
        assert_eq!(
            NoTestFile::Name(Patterns::from(patterns.clone())).to_string(),
            words,
            "{patterns:?}"
        );
    }
}
