use std::fs;
use std::path::{Path, PathBuf};

use purba_config::Project;
use purba_selection::{Aliases, Patterns, QUERIES};

/// The words of a project's file that holds `fast = "!path(tests/slow/**)"`.
const FAST: &str = "[queries]\nfast = \"!path(tests/slow/**)\"\n";

/// The aliases that the table under `[tool.purba]`, as `text`, declares.
fn aliases(text: &str) -> Result<Aliases, Vec<String>> {
    let table: toml::Table = text.parse().expect("the test writes TOML");
    Project::new(table)
        .resolve(&QUERIES)
        .map(|resolved| resolved.value)
}

/// A project at a new, empty folder for the test `name`, by its canonical
/// path, with a test of each of `names` in each file of `files`.
fn project(name: &str, files: &[(&str, &[&str])]) -> PathBuf {
    let root = PathBuf::from(env!("CARGO_TARGET_TMPDIR"))
        .join("aliases")
        .join(name);
    if root.exists() {
        fs::remove_dir_all(&root).expect("the test removes its old folder");
    }
    fs::create_dir_all(root.join(".git")).expect("the test makes its folder and .git");
    for (path, names) in files {
        let path = root.join(path);
        fs::create_dir_all(path.parent().expect("a file has a folder"))
            .expect("the test makes the folders");
        let text: String = names
            .iter()
            .flat_map(|name| ["def ", name, "():\n    pass\n\n"])
            .collect();
        fs::write(path, text).expect("the test writes the module");
    }
    root.canonicalize()
        .expect("the folder has a canonical path")
}

/// The ids of the tests that `text` picks in `root` once the aliases of the
/// project's file `declared` expand it, or the words of the error that stops
/// the run.
fn picked(root: &Path, declared: &str, text: &str) -> Result<Vec<String>, String> {
    let aliases = aliases(declared).map_err(|words| words.join("\n"))?;
    let query = purba_selection::query(text)
        .and_then(|query| query.expand(&aliases))
        .map_err(|refusal| refusal.to_string())?;
    let reached = purba_selection::reach(root, Some(&query), &Patterns::default())
        .map_err(|unnamed| unnamed.to_string())?;
    let mut read = Vec::new();
    for path in &reached.files().files {
        let module = purba_parsing::parse(&root.join(path)).expect("the module parses");
        let selected = reached.select(path, &purba_parsing::Facts::of(&module));
        read.push((path.clone(), selected));
    }
    reached
        .check(
            read.iter()
                .map(|(path, selected)| (path.as_path(), Some(selected))),
        )
        .map_err(|unnamed| unnamed.to_string())?;
    Ok(read
        .iter()
        .flat_map(|(_, selected)| selected.tests().iter().map(ToString::to_string))
        .collect())
}

#[test]
fn with_no_key_the_project_declares_no_alias() {
    let declared = aliases("").expect("no key is taken");
    assert_eq!(
        declared.forms(),
        Vec::<&str>::new(),
        "no key declares no alias"
    );
}

#[test]
fn the_table_declares_each_alias_by_its_key() {
    let declared = aliases("[queries]\nfast = \"name(f)\"\n\"stable(x)\" = \"x\"\n")
        .expect("both aliases are taken");
    assert_eq!(
        declared.forms(),
        vec!["fast", "stable(x)"],
        "each alias in the form its key declares, sorted by name"
    );
}

#[test]
fn a_value_that_is_no_table_or_no_string_is_refused() {
    assert_eq!(
        aliases("queries = \"fast\"").map(|_| ()),
        Err(vec![
            "queries under [tool.purba] is a string; write a table, such as [tool.purba.queries]"
                .to_owned()
        ]),
        "the aliases sit in a table"
    );
    assert_eq!(
        aliases("[queries]\nfast = 1\nslow = \"path(tests/slow/**)\"").map(|_| ()),
        Err(vec![
            "fast under [tool.purba.queries] is a number; write a query in a string, \
             such as fast = \"!path(tests/slow/**)\""
                .to_owned()
        ]),
        "each alias holds a query in a string"
    );
}

#[test]
fn an_alias_that_names_itself_is_refused_by_the_dial() {
    assert_eq!(
        aliases("[queries]\nloop = \"loop\"\nfast = \"name(f)\"").map(|_| ()),
        Err(vec![
            "loop under [tool.purba.queries] names itself".to_owned()
        ]),
        "the dial reads every alias, and fast is sound"
    );
}

#[test]
fn an_alias_picks_the_tests_its_query_picks() {
    let root = project(
        "fast",
        &[
            ("tests/test_a.py", &["test_add", "test_sub"]),
            ("tests/slow/test_s.py", &["test_slow"]),
        ],
    );
    assert_eq!(
        picked(&root, FAST, "fast"),
        Ok(vec![
            "tests/test_a.py::test_add".to_owned(),
            "tests/test_a.py::test_sub".to_owned(),
        ]),
        "fast leaves out tests/slow"
    );
}

#[test]
fn a_bare_word_that_names_an_alias_and_a_folder_runs_the_alias() {
    let root = project(
        "alias_and_folder",
        &[
            ("tests/test_a.py", &["test_add", "test_sub"]),
            ("fast/test_f.py", &["test_f"]),
        ],
    );
    assert_eq!(
        picked(
            &root,
            "[queries]\nfast = \"tests/test_a.py::test_add\"",
            "fast"
        ),
        Ok(vec!["tests/test_a.py::test_add".to_owned()]),
        "the alias hides the folder fast"
    );
    assert_eq!(
        picked(&root, "", "fast"),
        Ok(vec!["fast/test_f.py::test_f".to_owned()]),
        "with no alias, fast is the folder"
    );
}

#[test]
fn a_call_hands_its_argument_to_the_alias() {
    let root = project(
        "stable",
        &[
            ("tests/test_a.py", &["test_add"]),
            ("tests/slow/test_a.py", &["test_slow"]),
            ("tests/test_b.py", &["test_mul"]),
        ],
    );
    assert_eq!(
        picked(
            &root,
            "[queries]\n\"stable(x)\" = \"x & !path(tests/slow/**)\"",
            "stable(name(test_))"
        ),
        Ok(vec![
            "tests/test_a.py::test_add".to_owned(),
            "tests/test_b.py::test_mul".to_owned(),
        ]),
        "stable keeps what its argument picks outside tests/slow"
    );
}

#[test]
fn a_bare_word_that_names_nothing_names_the_nearer_of_an_id_and_an_alias() {
    let root = project("nearest", &[("tests/test_a.py", &["test_add"])]);
    assert_eq!(
        picked(&root, "", "fsat"),
        Err("fsat names no test; the nearest is tests".to_owned()),
        "with no alias, the words stay those of an id"
    );
    assert_eq!(
        picked(&root, FAST, "fsat"),
        Err("fsat names no test or alias; the nearest alias is fast".to_owned()),
        "fast is 1 edit from fsat, and tests is more"
    );
    assert_eq!(
        picked(&root, FAST, "tset"),
        Err("tset names no test or alias; the nearest is tests".to_owned()),
        "tests is 2 edits from tset, and fast is 3"
    );
    assert_eq!(
        picked(&root, "[queries]\ntsetxy = \"name(add)\"", "tset"),
        Err("tset names no test or alias; the nearest alias is tsetxy".to_owned()),
        "tsetxy and tests are each 2 edits from tset, and the tie goes to the alias"
    );
    assert_eq!(
        picked(&root, FAST, "tests/test_b.py"),
        Err("tests/test_b.py names no test; the nearest is tests/test_a.py".to_owned()),
        "a / holds no name of an alias, so the words stay those of an id"
    );
}

#[test]
fn a_call_that_names_no_alias_is_refused_with_the_nearest_alias() {
    let root = project("no_alias", &[("tests/test_a.py", &["test_add"])]);
    assert_eq!(
        picked(
            &root,
            "[queries]\nfast = \"name(f)\"\n\"stable(x)\" = \"x\"",
            "stale(tests)"
        ),
        Err("column 1: stale( is no term; the nearest alias is stable(x)".to_owned()),
        "the dial hands the nearest by edits, not the first alias"
    );
}
