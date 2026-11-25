use purba_config::{Dial, Level, Project, distance, nearest};

/// A dial for the test: a whole number under `workers`, 1 by default.
static WORKERS: Dial<i64> = Dial::new("workers", read_workers, || 1);

fn read_workers(value: &toml::Value) -> Result<i64, Vec<String>> {
    value
        .as_integer()
        .ok_or_else(|| vec!["workers under [tool.purba] is not a number".to_owned()])
}

/// The table under `[tool.purba]` that `text` holds.
fn project(text: &str) -> Project {
    Project::new(text.parse().expect("the test writes TOML"))
}

#[test]
fn a_dial_with_no_key_takes_its_default_from_no_level() {
    let resolved = project("").resolve(&WORKERS).expect("no key is no error");

    assert_eq!(
        (resolved.value, resolved.level),
        (1, Level::Default),
        "the default, from no level"
    );
}

#[test]
fn a_dial_with_a_key_takes_its_value_from_the_project() {
    let resolved = project("workers = 4")
        .resolve(&WORKERS)
        .expect("a number is no error");

    assert_eq!(
        (resolved.value, resolved.level),
        (4, Level::Project),
        "the project's value, from the project's level"
    );
}

#[test]
fn a_value_that_the_dial_refuses_gives_the_dials_words() {
    assert_eq!(
        project(r#"workers = "four""#)
            .resolve(&WORKERS)
            .map(|resolved| resolved.value),
        Err(vec![
            "workers under [tool.purba] is not a number".to_owned()
        ]),
        "the words of the dial's part"
    );
}

#[test]
fn each_key_that_no_dial_owns_names_the_nearest_dial_sorted_by_key() {
    let project = project(
        r#"
worker = 4
workers = 2
test-files = []
[queries]
fast = "x"
"#,
    );
    let line = |key: &str| {
        format!("{key} under [tool.purba] is no setting of purba; the nearest is workers")
    };

    assert_eq!(
        project.unknown(&["workers", "test-files"]),
        vec![line("queries"), line("worker")],
        "one line for each key that no dial owns, a table too, and none for a dial's key"
    );
}

#[test]
fn a_project_whose_keys_each_belong_to_a_dial_has_no_unknown_key() {
    assert_eq!(
        project("workers = 2").unknown(&["workers"]),
        Vec::<String>::new(),
        "each key belongs to a dial"
    );
}

#[test]
fn with_no_dial_a_key_names_no_nearest_dial() {
    assert_eq!(
        project("worker = 1").unknown(&[]),
        vec!["worker under [tool.purba] is no setting of purba".to_owned()],
        "with no dial to name, the line ends at the key"
    );
}

#[test]
fn an_edit_changes_adds_or_drops_one_letter_or_swaps_two_neighbours() {
    for (one, other, edits) in [
        ("test", "test", 0),
        ("test", "best", 1),
        ("test", "tests", 1),
        ("test", "tst", 1),
        ("tset", "test", 1),
        ("abc", "", 3),
        ("", "abc", 3),
        ("ca", "abc", 3),
        ("Test", "test", 1),
    ] {
        assert_eq!(distance(one, other), edits, "{one} to {other}");
    }
}

#[test]
fn the_nearest_name_is_the_first_of_the_fewest_edits() {
    assert_eq!(
        nearest("tset-files", &["workers", "test-files"]),
        Some("test-files"),
        "the fewest edits"
    );
    assert_eq!(
        nearest("x", &["ab", "cd"]),
        Some("ab"),
        "a tie takes the first"
    );
    assert_eq!(nearest("x", &[]), None, "no name, no nearest");
}
