use std::fs;
use std::path::{Path, PathBuf};

use purba_command::{Error, Place};

/// A new, empty folder for the test `name`, with a folder `.git` in it, so
/// that the walk up stops there.
fn repository(name: &str) -> PathBuf {
    let folder = PathBuf::from(env!("CARGO_TARGET_TMPDIR"))
        .join("root")
        .join(name);
    if folder.exists() {
        fs::remove_dir_all(&folder).expect("the test removes its old folder");
    }
    fs::create_dir_all(folder.join(".git")).expect("the test makes its folder");
    folder
}

/// Writes `text` at `path` under `folder`, and makes each folder on its way.
fn write(folder: &Path, path: &str, text: &str) {
    let path = folder.join(path);
    fs::create_dir_all(path.parent().expect("a file has a folder"))
        .expect("the test makes the folders");
    fs::write(path, text).expect("the test writes the file");
}

/// Runs purba with `arguments` from `folder`. Gives what `run_with` returned,
/// and stdout.
fn run_in(folder: &Path, arguments: &[&str]) -> (Result<u8, Error>, String) {
    let mut argv = vec!["purba".to_owned()];
    argv.extend(arguments.iter().map(|&argument| argument.to_owned()));
    let mut out = Vec::new();
    let result = purba_command::run_with(&argv, folder, Ok(&mut out), purba_assertion::rewrite);
    (
        result,
        String::from_utf8(out).expect("reporting writes UTF-8"),
    )
}

/// Each kind that a line names after its subject, in reporting's words.
const KINDS: [&str; 6] = [
    "passed",
    "failed",
    "raised",
    "blocked",
    "project error",
    "internal error",
];

/// Each line of `stdout` but the count, the last, as its subject and its kind.
/// An indented line under a failed case is not such a line.
fn kinds(stdout: &str) -> Vec<(&str, &'static str)> {
    let mut lines: Vec<&str> = stdout.lines().collect();
    lines.pop();
    lines
        .into_iter()
        .filter(|line| !line.starts_with(' '))
        .map(|line| {
            // No path of these projects holds a space.
            let (subject, rest) = line
                .split_once(' ')
                .unwrap_or_else(|| panic!("a line names its subject and its kind: {line}"));
            let kind = KINDS
                .into_iter()
                .find(|kind| {
                    rest.strip_prefix(kind)
                        .is_some_and(|after| after.is_empty() || after.starts_with(':'))
                })
                .unwrap_or_else(|| panic!("a line names one of reporting's kinds: {line}"));
            (subject, kind)
        })
        .collect()
}

const PASSING: &str = "def test_add():\n    assert 1 + 1 == 2\n";

#[test]
fn the_root_is_the_nearest_folder_whose_pyproject_holds_tool_purba() {
    let top = repository("nearest");
    write(&top, "pyproject.toml", "[project]\nname = \"p\"\n");
    write(&top, "project/pyproject.toml", "[tool.purba]\n");
    write(&top, "project/tests/test_a.py", PASSING);
    write(&top, "project/test_top.py", PASSING);
    let (result, stdout) = run_in(&top.join("project/tests"), &[]);
    assert_eq!(
        kinds(&stdout),
        [
            ("test_top.py::test_add", "passed"),
            ("tests/test_a.py::test_add", "passed"),
        ],
        "each id starts at the root, whatever the working folder: {stdout}"
    );
    assert_eq!(result.ok(), Some(0), "two passed cases exit 0");
}

#[test]
fn a_path_starts_at_the_root() {
    let top = repository("path");
    write(&top, "pyproject.toml", "[tool.purba]\n");
    write(&top, "tests/test_a.py", PASSING);
    write(&top, "other/test_b.py", PASSING);
    let (result, stdout) = run_in(&top.join("other"), &["tests"]);
    assert_eq!(
        kinds(&stdout),
        [("tests/test_a.py::test_add", "passed")],
        "the path names the folder under the root, not under the working folder: {stdout}"
    );
    assert_eq!(result.ok(), Some(0), "a passed case exits 0");
}

#[test]
fn with_no_tool_purba_the_refusal_names_where_to_add_it() {
    let project = repository("project");
    write(&project, "pyproject.toml", "[project]\nname = \"p\"\n");
    write(&project, "tests/sub/test_a.py", PASSING);
    let (result, stdout) = run_in(&project.join("tests/sub"), &[]);
    assert!(
        matches!(
            &result,
            Err(Error::NoRoot(Place::Project(file))) if file == Path::new("../../pyproject.toml")
        ),
        "the pyproject.toml of the project, from the working folder: {result:?}"
    );
    assert_eq!(stdout, "", "no line and no count");

    let bare = repository("bare");
    write(&bare, "tests/test_a.py", PASSING);
    let (result, _) = run_in(&bare.join("tests"), &[]);
    assert!(
        matches!(&result, Err(Error::NoRoot(Place::Folder(folder))) if folder == Path::new("..")),
        "the folder that holds .git, from the working folder: {result:?}"
    );
}

#[test]
fn the_walk_stops_at_a_git_file() {
    let above = repository("git_file");
    write(&above, "pyproject.toml", "[tool.purba]\n");
    write(&above, "worktree/.git", "gitdir: elsewhere\n");
    write(&above, "worktree/test_a.py", PASSING);
    let (result, _) = run_in(&above.join("worktree"), &[]);
    assert!(
        matches!(&result, Err(Error::NoRoot(Place::Folder(folder))) if folder == Path::new(".")),
        "the [tool.purba] above the .git file is not the root: {result:?}"
    );
}

#[test]
fn a_pyproject_toml_that_is_not_toml_is_a_project_error() {
    let top = repository("not_toml");
    write(&top, "pyproject.toml", "[tool.purba\n");
    write(&top, "tests/test_a.py", PASSING);
    let (result, stdout) = run_in(&top.join("tests"), &[]);
    assert_eq!(
        kinds(&stdout),
        [("../pyproject.toml", "project error")],
        "the line names the file from the working folder: {stdout}"
    );
    assert_eq!(result.ok(), Some(4), "a project error exits 4");
}

#[test]
fn a_refused_value_of_a_dial_stops_the_run_before_any_test() {
    let top = repository("refused_dial");
    write(&top, "pyproject.toml", "[tool.purba]\ntest-files = []\n");
    write(&top, "tests/test_a.py", PASSING);
    let (result, stdout) = run_in(&top.join("tests"), &[]);
    assert_eq!(
        kinds(&stdout),
        [("../pyproject.toml", "project error")],
        "the error names the file from the working folder, and no test runs: {stdout}"
    );
    assert_eq!(result.ok(), Some(4), "a project error exits 4");
}

#[test]
fn a_key_that_no_dial_owns_stops_the_run_before_any_test() {
    let top = repository("unknown_key");
    write(&top, "pyproject.toml", "[tool.purba]\nworkers = 4\n");
    write(&top, "tests/test_a.py", PASSING);
    let (result, stdout) = run_in(&top.join("tests"), &[]);
    assert_eq!(
        kinds(&stdout),
        [("../pyproject.toml", "project error")],
        "the error names the file from the working folder, and no test runs: {stdout}"
    );
    assert_eq!(result.ok(), Some(4), "a project error exits 4");
}

#[test]
fn a_module_that_does_not_parse_stops_only_itself_and_prints_before_the_run() {
    let top = repository("broken");
    write(&top, "pyproject.toml", "[tool.purba]\n");
    write(&top, "tests/test_a.py", PASSING);
    write(&top, "tests/test_b.py", "def test_broken(:\n");
    let (result, stdout) = run_in(&top, &[]);
    assert_eq!(
        kinds(&stdout),
        [
            ("tests/test_b.py", "project error"),
            ("tests/test_a.py::test_add", "passed"),
        ],
        "the error found before the run prints first, and the other file runs: {stdout}"
    );
    assert_eq!(result.ok(), Some(4), "a project error exits 4");
}

#[cfg(unix)]
#[test]
fn a_module_that_purba_cannot_read_stops_only_itself() {
    use std::os::unix::fs::PermissionsExt;

    let top = repository("unread");
    write(&top, "pyproject.toml", "[tool.purba]\n");
    write(&top, "tests/test_a.py", PASSING);
    write(&top, "tests/test_b.py", PASSING);
    let unread = top.join("tests/test_b.py");
    fs::set_permissions(&unread, fs::Permissions::from_mode(0o000))
        .expect("the test takes away each permission");
    if fs::read(&unread).is_ok() {
        eprintln!("skipped: root reads a file with no permission");
        return;
    }
    let (result, stdout) = run_in(&top, &[]);
    assert_eq!(
        kinds(&stdout),
        [
            ("tests/test_b.py", "project error"),
            ("tests/test_a.py::test_add", "passed"),
        ],
        "the module that purba cannot read has its error, and the other file runs: {stdout}"
    );
    assert_eq!(result.ok(), Some(4), "a project error exits 4");
}

#[test]
fn a_root_with_no_test_file_prints_only_the_count() {
    let top = repository("empty");
    write(&top, "pyproject.toml", "[tool.purba]\n");
    write(&top, "helper.py", PASSING);
    let (result, stdout) = run_in(&top, &[]);
    assert!(
        kinds(&stdout).is_empty(),
        "no line of a case or a module: {stdout}"
    );
    assert_eq!(stdout.lines().count(), 1, "only the count: {stdout}");
    assert_eq!(result.ok(), Some(2), "a run with no test exits 2");
}

#[test]
fn a_panic_in_the_rewrite_of_one_module_stops_only_that_module() {
    let top = repository("rewrite_panic");
    write(&top, "pyproject.toml", "[tool.purba]\n");
    write(&top, "tests/test_a.py", PASSING);
    write(
        &top,
        "tests/test_b.py",
        "def test_panics():\n    assert 1 + 1 == 2\n",
    );
    let argv = ["purba".to_owned()];
    let mut out = Vec::new();
    let result = purba_command::run_with(&argv, &top, Ok(&mut out), |facts, tests| {
        assert!(
            !tests.contains(&"test_panics"),
            "the rewrite of tests/test_b.py panics"
        );
        purba_assertion::rewrite(facts, tests)
    });
    let stdout = String::from_utf8(out).expect("reporting writes UTF-8");
    assert_eq!(
        kinds(&stdout),
        [
            ("tests/test_b.py", "internal error"),
            ("tests/test_b.py::test_panics", "blocked"),
            ("tests/test_a.py::test_add", "passed"),
        ],
        "the panicking module has its error before the run, and the other module runs: {stdout}"
    );
    assert_eq!(result.ok(), Some(125), "an internal error exits 125");
}
