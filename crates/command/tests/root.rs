use std::ffi::OsString;
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

/// Runs purba with `arguments` from `folder`, with no variable set. Gives what
/// `run_with` returned, and stdout.
fn run_in(folder: &Path, arguments: &[&str]) -> (Result<u8, Error>, String) {
    run_set(folder, arguments, &[])
}

/// Runs purba with `arguments` from `folder`, with each pair of `variables`
/// set. Gives what `run_with` returned, and stdout.
fn run_set(
    folder: &Path,
    arguments: &[&str],
    variables: &[(&str, OsString)],
) -> (Result<u8, Error>, String) {
    let mut argv = vec!["purba".to_owned()];
    argv.extend(arguments.iter().map(|&argument| argument.to_owned()));
    let variables: Vec<(OsString, OsString)> = variables
        .iter()
        .map(|(name, value)| (OsString::from(name), value.clone()))
        .collect();
    let mut out = Vec::new();
    let result = purba_command::run_with(
        &argv,
        folder,
        &variables,
        Ok(&mut out),
        purba_assertion::rewrite,
    );
    (
        result,
        String::from_utf8(out).expect("reporting writes UTF-8"),
    )
}

/// The home folder of the tests of an implicit root: the folder that holds
/// the folder of each test, as the walk names it.
fn home() -> PathBuf {
    let folder = PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join("root");
    fs::create_dir_all(&folder).expect("the test makes the folder of the tests");
    folder
        .canonicalize()
        .expect("the folder of the tests exists")
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
    let result = purba_command::run_with(&argv, &top, &[], Ok(&mut out), |facts, tests| {
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

/// The root that the notice on the first line of `stdout` names, as the line
/// shows it, or `None` when the first line is no notice of a root. Then the
/// lines after the notice.
fn noticed(stdout: &str) -> (Option<&str>, &str) {
    let (first, rest) = stdout.split_once('\n').unwrap_or((stdout, ""));
    match first
        .strip_prefix("⚠ root ")
        .and_then(|after| after.split_once(" is "))
    {
        Some((root, _)) => (Some(root), rest),
        None => (None, stdout),
    }
}

/// The variable that each line of an error of the environment names, in the
/// order of the lines, or `None` when `result` is no such error.
fn named(result: &Result<u8, Error>) -> Option<Vec<&str>> {
    match result {
        Err(Error::Environment(lines)) => Some(
            lines
                .iter()
                .map(|line| line.split_once(' ').map_or(line.as_str(), |(name, _)| name))
                .collect(),
        ),
        _ => None,
    }
}

/// `PURBA_IMPLICIT_ROOT` set to `value`, and `HOME` set to `home`.
fn implicit(value: &str, home: &Path) -> [(&'static str, OsString); 2] {
    [
        ("PURBA_IMPLICIT_ROOT", OsString::from(value)),
        ("HOME", home.as_os_str().to_owned()),
    ]
}

/// Runs purba from `folder` with `PURBA_IMPLICIT_ROOT=1`, and checks that the
/// notice names `root` and that `tests/test_a.py::test_add` passes from there.
fn takes_root(folder: &Path, root: &str) {
    let (result, stdout) = run_set(folder, &[], &implicit("1", &home()));
    let (shown, rest) = noticed(&stdout);
    assert_eq!(
        shown,
        Some(root),
        "the first line is the notice of the root, from {}: {stdout}",
        folder.display()
    );
    assert_eq!(
        kinds(rest),
        [("tests/test_a.py::test_add", "passed")],
        "each id starts at the root: {stdout}"
    );
    assert_eq!(result.ok(), Some(0), "the notice changes no exit code");
}

#[test]
fn with_the_variable_the_root_is_the_folder_of_the_pyproject_that_holds_project() {
    let top = repository("implicit_project");
    write(&top, "pyproject.toml", "[project]\nname = \"p\"\n");
    write(&top, "tests/test_a.py", PASSING);
    takes_root(&top, "~/implicit_project");
    takes_root(&top.join("tests"), "~/implicit_project");
}

#[test]
fn the_step_of_the_project_takes_the_nearest_project_below_git() {
    let top = repository("implicit_nearest");
    write(&top, "pyproject.toml", "[project]\nname = \"top\"\n");
    write(&top, "pkg/pyproject.toml", "[project]\nname = \"pkg\"\n");
    write(&top, "pkg/tests/test_a.py", PASSING);
    takes_root(&top.join("pkg/tests"), "~/implicit_nearest/pkg");

    let above = repository("implicit_above");
    write(&above, "pyproject.toml", "[project]\nname = \"above\"\n");
    fs::create_dir_all(above.join("repo/.git")).expect("the test makes a .git below");
    write(&above, "repo/tests/test_a.py", PASSING);
    takes_root(&above.join("repo"), "~/implicit_above/repo");
}

#[test]
fn with_the_variable_and_no_project_the_root_is_the_folder_that_holds_git() {
    let top = repository("implicit_git");
    write(&top, "pyproject.toml", "[tool.black]\n");
    write(&top, "tests/test_a.py", PASSING);
    takes_root(&top, "~/implicit_git");
    takes_root(&top.join("tests"), "~/implicit_git");

    let file = repository("implicit_git_file");
    write(&file, "worktree/.git", "gitdir: elsewhere\n");
    write(&file, "worktree/tests/test_a.py", PASSING);
    takes_root(&file.join("worktree"), "~/implicit_git_file/worktree");
}

#[test]
fn a_root_outside_the_home_folder_prints_in_full_and_the_home_folder_prints_tilde() {
    let top = repository("implicit_tilde");
    write(&top, "tests/test_a.py", PASSING);
    let canonical = top.canonicalize().expect("the folder of the test exists");
    let full = canonical.display().to_string();
    for (variables, shown) in [
        (
            vec![("PURBA_IMPLICIT_ROOT", OsString::from("1"))],
            full.as_str(),
        ),
        (implicit("1", Path::new("/nowhere/home")).to_vec(), &full),
        (implicit("1", &home().join("implicit_til")).to_vec(), &full),
        (implicit("1", &canonical).to_vec(), "~"),
    ] {
        let (result, stdout) = run_set(&top, &[], &variables);
        assert_eq!(
            noticed(&stdout).0,
            Some(shown),
            "~ stands in only for a HOME that holds the root, folder by folder: {variables:?}"
        );
        assert_eq!(result.ok(), Some(0), "the notice changes no exit code");
    }
}

#[test]
fn the_notice_comes_before_the_line_of_a_module_that_does_not_parse() {
    let top = repository("implicit_broken");
    write(&top, "tests/test_a.py", PASSING);
    write(&top, "tests/test_b.py", "def test_broken(:\n");
    let (result, stdout) = run_set(&top, &[], &implicit("1", &home()));
    let (shown, rest) = noticed(&stdout);
    assert_eq!(
        shown,
        Some("~/implicit_broken"),
        "the notice is the first line: {stdout}"
    );
    assert_eq!(
        kinds(rest),
        [
            ("tests/test_b.py", "project error"),
            ("tests/test_a.py::test_add", "passed"),
        ],
        "the error found before the run comes after the notice: {stdout}"
    );
    assert_eq!(result.ok(), Some(4), "a project error exits 4");
}

#[test]
fn a_root_that_tool_purba_gives_prints_no_line_with_the_variable_set() {
    let top = repository("implicit_tool_purba");
    write(&top, "pyproject.toml", "[tool.purba]\n");
    write(&top, "tests/test_a.py", PASSING);
    let (result, stdout) = run_set(&top, &[], &implicit("1", &home()));
    assert_eq!(noticed(&stdout).0, None, "no notice of a root: {stdout}");
    assert_eq!(
        kinds(&stdout),
        [("tests/test_a.py::test_add", "passed")],
        "only the line of the case: {stdout}"
    );
    assert_eq!(result.ok(), Some(0), "a passed case exits 0");
}

#[test]
fn zero_and_an_empty_value_leave_the_steps_off() {
    let top = repository("implicit_off");
    write(&top, "pyproject.toml", "[project]\nname = \"p\"\n");
    write(&top, "tests/test_a.py", PASSING);
    for value in ["0", ""] {
        let (result, stdout) = run_set(&top, &[], &implicit(value, &home()));
        assert!(
            matches!(
                &result,
                Err(Error::NoRoot(Place::Project(file))) if file == Path::new("pyproject.toml")
            ),
            "{value:?} refuses as an unset variable does: {result:?}"
        );
        assert_eq!(stdout, "", "no line and no count");
    }
}

#[test]
fn any_other_value_is_an_invocation_error_in_each_run() {
    for (name, project) in [
        ("implicit_yes", "[project]\nname = \"p\"\n"),
        ("implicit_yes_tool_purba", "[tool.purba]\n"),
    ] {
        let top = repository(name);
        write(&top, "pyproject.toml", project);
        write(&top, "tests/test_a.py", PASSING);
        for value in ["yes", "true"] {
            let (result, stdout) = run_set(&top, &[], &implicit(value, &home()));
            assert_eq!(
                named(&result),
                Some(vec!["PURBA_IMPLICIT_ROOT"]),
                "{value:?} is the variable's error, whatever the root: {project:?} {result:?}"
            );
            assert_eq!(stdout, "", "no line and no count");
        }
    }
}

#[test]
fn a_variable_near_the_name_is_an_invocation_error_and_a_far_one_is_ignored() {
    for (name, project) in [
        ("implicit_near", "[tool.purba]\n"),
        ("implicit_near_no_root", "[project]\nname = \"p\"\n"),
    ] {
        let top = repository(name);
        write(&top, "pyproject.toml", project);
        write(&top, "tests/test_a.py", PASSING);
        let (result, stdout) = run_set(&top, &[], &[("PURBA_IMLPICIT_ROOT", OsString::from("1"))]);
        assert_eq!(
            named(&result),
            Some(vec!["PURBA_IMLPICIT_ROOT"]),
            "one swap is near, whatever the walk finds: {project:?} {result:?}"
        );
        assert_eq!(stdout, "", "no line and no count");
    }

    let top = repository("implicit_far");
    write(&top, "pyproject.toml", "[tool.purba]\n");
    write(&top, "tests/test_a.py", PASSING);
    let (result, stdout) = run_set(&top, &[], &[("PURBA_SENTINEL", OsString::from("1"))]);
    assert_eq!(
        kinds(&stdout),
        [("tests/test_a.py::test_add", "passed")],
        "a name far from each dial changes nothing: {stdout}"
    );
    assert_eq!(result.ok(), Some(0), "a passed case exits 0");
}

#[test]
fn each_error_of_the_environment_comes_by_name_before_the_project_is_read() {
    let top = repository("implicit_errors");
    write(&top, "pyproject.toml", "[tool.purba]\nworkers = 4\n");
    write(&top, "tests/test_a.py", PASSING);
    let (result, stdout) = run_set(
        &top,
        &[],
        &[
            ("PURBA_IMPLICIT_ROOT", OsString::from("yes")),
            ("PURBA_IMLPICIT_ROOT", OsString::from("1")),
        ],
    );
    assert_eq!(
        named(&result),
        Some(vec!["PURBA_IMLPICIT_ROOT", "PURBA_IMPLICIT_ROOT"]),
        "one line for each error, by the name of its variable: {result:?}"
    );
    assert_eq!(
        stdout, "",
        "the key that no dial owns is not read, so no project error prints"
    );
}

#[test]
fn two_paths_are_the_usage_error_before_a_wrong_value() {
    let top = repository("implicit_usage");
    write(&top, "pyproject.toml", "[tool.purba]\n");
    write(&top, "tests/test_a.py", PASSING);
    let (result, stdout) = run_set(&top, &["a", "b"], &implicit("yes", &home()));
    assert!(
        matches!(result, Err(Error::Usage)),
        "the arguments come first: {result:?}"
    );
    assert_eq!(stdout, "", "no line and no count");
}
