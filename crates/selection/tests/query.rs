use std::fs;
use std::path::{Path, PathBuf};

use purba_selection::Patterns;

/// A project of git at a new, empty folder for the test `name`, by its
/// canonical path. A folder named `.git` is all that the walk reads of git.
fn project(name: &str) -> PathBuf {
    let root = PathBuf::from(env!("CARGO_TARGET_TMPDIR"))
        .join("query")
        .join(name);
    if root.exists() {
        fs::remove_dir_all(&root).expect("the test removes its old folder");
    }
    fs::create_dir_all(root.join(".git")).expect("the test makes its folder and .git");
    root.canonicalize()
        .expect("the folder has a canonical path")
}

/// Writes a module at `path` under `root` that holds a test of each of
/// `names`, and makes each folder on its way.
fn module(root: &Path, path: &str, names: &[&str]) {
    let path = root.join(path);
    fs::create_dir_all(path.parent().expect("a file has a folder"))
        .expect("the test makes the folders");
    let text: String = names
        .iter()
        .flat_map(|name| ["def ", name, "():\n    pass\n\n"])
        .collect();
    fs::write(path, text).expect("the test writes the module");
}

/// What a run of `text` in `root` gives: each file that stage two parses, and
/// each test that selection picks, refused ones among them, or the words of the
/// error that stops the run.
fn picked(root: &Path, text: &str) -> Result<(Vec<String>, Vec<String>), String> {
    let query = purba_selection::query(text).map_err(|refusal| refusal.to_string())?;
    let reached = purba_selection::reach(root, Some(&query), &Patterns::default())
        .map_err(|unnamed| unnamed.to_string())?;
    assert_eq!(
        reached.files().unread,
        Vec::new(),
        "stage one reads each folder of the project"
    );
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
    let files = read
        .iter()
        .map(|(path, _)| path.display().to_string())
        .collect();
    let tests = read
        .iter()
        .flat_map(|(_, selected)| {
            selected.tests().iter().map(ToString::to_string).chain(
                selected
                    .refused()
                    .iter()
                    .map(|refused| refused.test().to_string()),
            )
        })
        .collect();
    Ok((files, tests))
}

/// A project with `tests/test_a.py`, which holds `test_add` and `test_sub`, and
/// `tests/test_b.py`, which holds `test_mul`.
fn two_files(name: &str) -> PathBuf {
    let root = project(name);
    module(&root, "tests/test_a.py", &["test_add", "test_sub"]);
    module(&root, "tests/test_b.py", &["test_mul"]);
    root
}

fn strings(each: &[&str]) -> Vec<String> {
    each.iter().map(|&one| one.to_owned()).collect()
}

#[test]
fn the_id_of_a_test_parses_its_file_alone_and_picks_that_test() {
    let root = two_files("id_of_a_test");
    assert_eq!(
        picked(&root, "tests/test_a.py::test_add"),
        Ok((
            strings(&["tests/test_a.py"]),
            strings(&["tests/test_a.py::test_add"])
        )),
        "stage two parses only the file of the id"
    );
}

#[test]
fn a_folder_and_a_file_pick_each_test_under_them() {
    let root = two_files("folder_and_file");
    assert_eq!(
        picked(&root, "tests/test_b.py"),
        Ok((
            strings(&["tests/test_b.py"]),
            strings(&["tests/test_b.py::test_mul"])
        )),
        "a file picks each of its tests"
    );
    assert_eq!(
        picked(&root, "./tests"),
        Ok((
            strings(&["tests/test_a.py", "tests/test_b.py"]),
            strings(&[
                "tests/test_a.py::test_add",
                "tests/test_a.py::test_sub",
                "tests/test_b.py::test_mul"
            ])
        )),
        "a folder picks each test under it, and ./ names the same folder"
    );
}

#[test]
fn an_id_matches_only_whole_parts() {
    let root = two_files("whole_parts");
    assert_eq!(
        picked(&root, "tests/test_a"),
        Err("tests/test_a names no test; the nearest is tests/test_a.py".to_owned()),
        "a part of a file's name names no file"
    );
}

#[test]
fn name_reaches_each_file_and_picks_each_test_whose_name_holds_it() {
    let root = two_files("name");
    assert_eq!(
        picked(&root, "name(u)"),
        Ok((
            strings(&["tests/test_a.py", "tests/test_b.py"]),
            strings(&["tests/test_a.py::test_sub", "tests/test_b.py::test_mul"])
        )),
        "name() reads no path, so stage two parses each file"
    );
}

#[test]
fn path_matches_a_glob_part_by_part() {
    let root = project("path");
    module(&root, "tests/test_a.py", &["test_add"]);
    module(&root, "tests/db/test_db.py", &["test_query"]);
    assert_eq!(
        picked(&root, "path(tests/**/test_*.py)"),
        Ok((
            strings(&["tests/db/test_db.py", "tests/test_a.py"]),
            strings(&[
                "tests/db/test_db.py::test_query",
                "tests/test_a.py::test_add"
            ])
        )),
        "** matches no part, or many"
    );
    assert_eq!(
        picked(&root, "path(tests/*.py)"),
        Ok((
            strings(&["tests/test_a.py"]),
            strings(&["tests/test_a.py::test_add"])
        )),
        "* stays inside one part"
    );
    assert_eq!(
        picked(&root, "path(tests/test_?.py)"),
        Ok((
            strings(&["tests/test_a.py"]),
            strings(&["tests/test_a.py::test_add"])
        )),
        "? matches one letter"
    );
}

#[test]
fn not_and_and_or_combine_the_picks() {
    let root = two_files("combine");
    assert_eq!(
        picked(&root, "tests/test_a.py & !name(sub)"),
        Ok((
            strings(&["tests/test_a.py"]),
            strings(&["tests/test_a.py::test_add"])
        )),
        "& keeps what both pick, and ! drops what its term picks"
    );
    assert_eq!(
        picked(&root, "name(mul) | name(add) & name(sub)"),
        Ok((
            strings(&["tests/test_a.py", "tests/test_b.py"]),
            strings(&["tests/test_b.py::test_mul"])
        )),
        "& binds tighter than |, so name(add) & name(sub) picks nothing"
    );
}

#[test]
fn a_query_whose_terms_each_name_something_may_pick_no_test() {
    let root = two_files("empty_pick");
    assert_eq!(
        picked(&root, "tests/test_a.py::test_add & name(sub)"),
        Ok((strings(&["tests/test_a.py"]), Vec::new())),
        "each term names a test, so no error comes"
    );
}

#[test]
fn a_refused_test_comes_only_when_the_query_picks_it() {
    let root = project("refused");
    let path = root.join("tests/test_a.py");
    fs::create_dir_all(path.parent().expect("a file has a folder")).expect("the test makes tests");
    fs::write(
        &path,
        "def test_mul(x):\n    pass\n\ndef test_add():\n    pass\n",
    )
    .expect("the test writes the module");
    assert_eq!(
        picked(&root, "tests/test_a.py::test_add"),
        Ok((
            strings(&["tests/test_a.py"]),
            strings(&["tests/test_a.py::test_add"])
        )),
        "the query does not pick test_mul, so its refusal does not come"
    );
    assert_eq!(
        picked(&root, "name(mul)"),
        Ok((
            strings(&["tests/test_a.py"]),
            strings(&["tests/test_a.py::test_mul"])
        )),
        "the query picks test_mul, so its refusal comes"
    );
}

#[test]
fn a_test_that_is_not_in_its_file_names_the_nearest_test() {
    let root = two_files("nearest_test");
    assert_eq!(
        picked(&root, "tests/test_a.py::test_ad"),
        Err(
            "tests/test_a.py::test_ad names no test; the nearest is tests/test_a.py::test_add"
                .to_owned()
        ),
        "the file holds test_add, one edit away"
    );
    assert_eq!(
        picked(&root, "tests/test_a.py::test_add::neg"),
        Err(
            "tests/test_a.py::test_add::neg names no test; the nearest is tests/test_a.py::test_add"
                .to_owned()
        ),
        "no case has an id yet, so a third part names nothing"
    );
}

#[test]
fn the_nearest_id_is_found_at_the_first_part_that_differs() {
    let root = project("part_by_part");
    module(&root, "tests/math/test_math.py", &["test_add"]);
    module(&root, "tests/store/test_store.py", &["test_migrate"]);
    assert_eq!(
        picked(&root, "tests/mat/test_store.py"),
        Err("tests/mat/test_store.py names no test; the nearest is tests/math".to_owned()),
        "the folder mat differs first, and math is nearest to it, though the whole id is \
         nearer to tests/store/test_store.py"
    );
}

#[test]
fn a_path_that_names_nothing_in_a_project_with_no_test_file_names_no_nearest() {
    let root = project("no_test_file");
    assert_eq!(
        picked(&root, "nowhere"),
        Err("nowhere names no test".to_owned()),
        "the project holds no test file to name"
    );
}

#[test]
fn a_name_that_no_test_holds_names_the_nearest_name() {
    let root = two_files("nearest_name");
    assert_eq!(
        picked(&root, "name(tset_sub)"),
        Err("name(tset_sub) names no test; the nearest name is test_sub".to_owned()),
        "test_sub is one swap from tset_sub"
    );
}

#[test]
fn a_term_that_names_nothing_counts_the_files_that_stage_one_left_out() {
    let root = two_files("left_out");
    assert_eq!(
        picked(&root, "tests/test_a.py & name(test_mul)"),
        Err(
            "name(test_mul) names no test; the nearest name is test_sub; \
             the query reaches 1 of 2 test files"
                .to_owned()
        ),
        "test_mul sits in tests/test_b.py, which the query does not reach"
    );
}

#[test]
fn a_glob_that_matches_no_test_file_names_the_nearest_file() {
    let root = two_files("nearest_file");
    assert_eq!(
        picked(&root, "path(tests/*_x.py)"),
        Err("path(tests/*_x.py) names no test file; the nearest is tests/test_a.py".to_owned()),
        "the part before * measures, and the first file in the order of the paths wins a tie"
    );
}

#[test]
fn an_id_in_a_folder_that_stage_one_skips_keeps_its_reason() {
    let root = two_files("skipped");
    module(&root, "tests/.cache/test_c.py", &["test_c"]);
    assert_eq!(
        picked(&root, "tests/.cache/test_c.py"),
        Err("tests/.cache/test_c.py names no test file: \
             stage one skips tests/.cache, a hidden folder"
            .to_owned()),
        "the reason of stage one stays"
    );
}

#[test]
fn a_query_that_does_not_parse_gives_the_words_of_query() {
    assert_eq!(
        purba_selection::query("tag(slow)")
            .map(|_| ())
            .map_err(|refusal| refusal.to_string()),
        purba_query::parse("tag(slow)")
            .map(|_| ())
            .map_err(|refusal| refusal.to_string()),
        "selection hands on the refusal of query, in its words"
    );
}

#[test]
fn or_of_two_ids_reaches_the_file_of_each() {
    let root = two_files("or_of_ids");
    assert_eq!(
        picked(&root, "tests/test_a.py::test_add | tests/test_b.py"),
        Ok((
            strings(&["tests/test_a.py", "tests/test_b.py"]),
            strings(&["tests/test_a.py::test_add", "tests/test_b.py::test_mul"])
        )),
        "| keeps each file that either term reaches"
    );
}

#[test]
fn not_of_an_or_leaves_out_a_file_that_one_term_reaches_whole() {
    let root = two_files("not_of_an_or");
    assert_eq!(
        picked(&root, "!(tests/test_a.py::test_add | tests/test_b.py)"),
        Ok((
            strings(&["tests/test_a.py"]),
            strings(&["tests/test_a.py::test_sub"])
        )),
        "tests/test_b.py picks each of its tests, so the ! of the | picks none of them"
    );
}

#[test]
fn not_of_a_file_reaches_each_other_file() {
    let root = two_files("not_of_a_file");
    assert_eq!(
        picked(&root, "!tests/test_b.py"),
        Ok((
            strings(&["tests/test_a.py"]),
            strings(&["tests/test_a.py::test_add", "tests/test_a.py::test_sub"])
        )),
        "! of a file picks no test of it, so stage two does not parse it"
    );
}

#[test]
fn question_mark_matches_one_letter_and_two_stars_cross_parts() {
    let root = project("glob_marks");
    module(&root, "tests/test_a.py", &["test_add"]);
    module(&root, "tests/test_ab.py", &["test_abs"]);
    module(&root, "tests/db/deep/test_deep.py", &["test_deep"]);
    assert_eq!(
        picked(&root, "path(tests/test_?.py)"),
        Ok((
            strings(&["tests/test_a.py"]),
            strings(&["tests/test_a.py::test_add"])
        )),
        "? matches one letter, so test_ab.py stays out"
    );
    assert_eq!(
        picked(&root, "path(tests/**/test_deep.py)"),
        Ok((
            strings(&["tests/db/deep/test_deep.py"]),
            strings(&["tests/db/deep/test_deep.py::test_deep"])
        )),
        "** crosses two parts"
    );
}

#[test]
fn the_nearest_file_of_a_glob_measures_the_part_before_its_first_star() {
    let root = project("glob_measure");
    module(&root, "tests/test_b_long_suffix.py", &["test_b"]);
    module(&root, "tests/x/test_c.py", &["test_c"]);
    assert_eq!(
        picked(&root, "path(tests/test_a*_long_suffix.py)"),
        Err("path(tests/test_a*_long_suffix.py) names no test file; \
             the nearest is tests/x/test_c.py"
            .to_owned()),
        "tests/test_a is nearer to tests/x/test_c.py, though the whole glob is nearer to \
         tests/test_b_long_suffix.py"
    );
}

#[test]
fn a_file_that_does_not_parse_leaves_a_term_of_its_test_unjudged() {
    let root = two_files("unparsed");
    fs::write(root.join("tests/test_b.py"), "def (:\n").expect("the test writes the module");
    let query = purba_selection::query("tests/test_b.py::test_mul").expect("the query parses");
    let reached = purba_selection::reach(&root, Some(&query), &Patterns::default())
        .expect("the file of the id is a test file");
    assert_eq!(
        reached.files().files,
        vec![PathBuf::from("tests/test_b.py")],
        "the query reaches the file of the id alone"
    );
    assert_eq!(
        reached
            .check([(Path::new("tests/test_b.py"), None)])
            .map_err(|unnamed| unnamed.to_string()),
        Ok(()),
        "the module's project error shows, and no term is blamed for it"
    );
}

#[test]
fn the_line_of_a_name_near_test_comes_under_a_query_that_picks_no_test_of_its_file() {
    let root = project("near_under_a_query");
    let path = root.join("tests/test_a.py");
    fs::create_dir_all(path.parent().expect("a file has a folder")).expect("the test makes tests");
    fs::write(
        &path,
        "def test_add():\n    pass\n\ndef tset_sub():\n    pass\n",
    )
    .expect("the test writes the module");
    let query = purba_selection::query("name(add) & name(sub)").expect("the query parses");
    let reached = purba_selection::reach(&root, Some(&query), &Patterns::default())
        .expect("name() reaches each file");
    let module = purba_parsing::parse(&path).expect("the module parses");
    let selected = reached.select(
        Path::new("tests/test_a.py"),
        &purba_parsing::Facts::of(&module),
    );
    assert!(selected.tests().is_empty(), "the query picks no test");
    let near: Vec<String> = selected
        .before_the_run()
        .iter()
        .filter_map(|line| match line {
            purba_selection::Before::NameNear(near) => Some(near.name().to_owned()),
            purba_selection::Before::Refused(_) => None,
        })
        .collect();
    assert_eq!(
        near,
        strings(&["tset_sub"]),
        "stage two parsed the file, so its name near test comes"
    );
}

#[test]
fn a_suppression_of_a_test_that_the_query_does_not_pick_stays_unjudged() {
    let root = project("suppression_unpicked");
    let path = root.join("tests/test_a.py");
    fs::create_dir_all(path.parent().expect("a file has a folder")).expect("the test makes tests");
    fs::write(
        &path,
        concat!(
            "# purba: suppress[not-one-comparison] a range reads best as one chain\n",
            "def test_sub():\n",
            "    value = 1\n",
            "    assert 0 < value < 2\n\n",
            "def test_add():\n",
            "    assert 1 + 1 == 2\n",
        ),
    )
    .expect("the test writes the module");
    let query = purba_selection::query("tests/test_a.py::test_add").expect("the query parses");
    let reached = purba_selection::reach(&root, Some(&query), &Patterns::default())
        .expect("the file of the id is a test file");
    let module = purba_parsing::parse(&path).expect("the module parses");
    let mut selected = reached.select(
        Path::new("tests/test_a.py"),
        &purba_parsing::Facts::of(&module),
    );
    let tests = selected.tests().to_vec();
    for test in &tests {
        selected.judge(test, &[], true);
    }
    assert_eq!(
        selected.unused(),
        Vec::<String>::new(),
        "the run never judged test_sub, so its suppression is not unused"
    );
}

#[test]
fn name_leaves_its_term_unjudged_beside_a_file_that_does_not_parse() {
    let root = two_files("name_unparsed");
    let query = purba_selection::query("name(mul)").expect("the query parses");
    let reached = purba_selection::reach(&root, Some(&query), &Patterns::default())
        .expect("name() reaches each file");
    let module =
        purba_parsing::parse(&root.join("tests/test_a.py")).expect("tests/test_a.py parses");
    let selected = reached.select(
        Path::new("tests/test_a.py"),
        &purba_parsing::Facts::of(&module),
    );
    assert_eq!(
        reached
            .check([
                (Path::new("tests/test_a.py"), Some(&selected)),
                (Path::new("tests/test_b.py"), None),
            ])
            .map_err(|unnamed| unnamed.to_string()),
        Ok(()),
        "test_mul may sit in tests/test_b.py, which did not parse"
    );
}

#[test]
fn name_counts_the_case_of_letters() {
    let root = two_files("name_case");
    assert_eq!(
        picked(&root, "name(ADD)"),
        Err("name(ADD) names no test; the nearest name is test_add".to_owned()),
        "ADD is not add"
    );
}
