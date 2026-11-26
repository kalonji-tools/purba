use std::fs;
use std::path::{Path, PathBuf};

use purba_selection::{NoTestFile, Pattern, Patterns, TestFiles};

/// Stage one with the patterns that apply when the project's file sets none.
fn test_files(root: &Path, path: Option<&Path>) -> Result<TestFiles, NoTestFile> {
    purba_selection::test_files(root, path, &Patterns::default())
}

/// Stage one with `*_test.py` alone, the one pattern that a hidden name can
/// match.
fn suffix_test_files(root: &Path, path: Option<&Path>) -> Result<TestFiles, NoTestFile> {
    purba_selection::test_files(root, path, &Patterns::from(vec![Pattern::TestSuffix]))
}

/// A new, empty folder for the test `name`, by its canonical path.
fn folder(name: &str) -> PathBuf {
    let path = PathBuf::from(env!("CARGO_TARGET_TMPDIR"))
        .join("stage_one")
        .join(name);
    if path.exists() {
        fs::remove_dir_all(&path).expect("the test removes its old folder");
    }
    fs::create_dir_all(&path).expect("the test makes its folder");
    path.canonicalize()
        .expect("the folder has a canonical path")
}

/// A project of git at a new folder for the test `name`. A folder named `.git`
/// is all that the walk reads of git.
fn project(name: &str) -> PathBuf {
    let root = folder(name);
    fs::create_dir(root.join(".git")).expect("the test makes .git");
    root
}

/// Writes `text` at `path` under `root`, and makes each folder on its way.
fn write(root: &Path, path: &str, text: &str) {
    let path = root.join(path);
    fs::create_dir_all(path.parent().expect("a file has a folder"))
        .expect("the test makes the folders");
    fs::write(path, text).expect("the test writes the file");
}

/// Each test file of `found`, as a string, where stage one read each folder.
fn shown(found: Result<TestFiles, NoTestFile>) -> Result<Vec<String>, NoTestFile> {
    found.map(|found| {
        assert_eq!(
            found.unread,
            Vec::new(),
            "stage one reads each folder of the project"
        );
        found
            .files
            .iter()
            .map(|path| path.display().to_string())
            .collect()
    })
}

#[test]
fn the_walk_gives_each_test_file_under_the_root_by_its_path_from_the_root() {
    let root = project("all");
    for path in [
        "tests/test_a.py",
        "test_top.py",
        "a/test_b.py",
        "a.py/test_c.py",
        "tests/deep/test_d.py",
    ] {
        write(&root, path, "");
    }
    assert_eq!(
        shown(test_files(&root, None)),
        Ok(vec![
            "a/test_b.py".to_owned(),
            "a.py/test_c.py".to_owned(),
            "test_top.py".to_owned(),
            "tests/deep/test_d.py".to_owned(),
            "tests/test_a.py".to_owned(),
        ]),
        "each test file, from the root, in the order of the parts of its path"
    );
}

#[test]
fn only_a_file_named_test_star_py_is_a_test_file() {
    let root = project("names");
    for path in [
        "test_a.py",
        "test_.py",
        "models_test.py",
        "tests.py",
        "test.py",
        "testing.py",
        "test_b.pyc",
        "test_c.PY",
        "test_d.py/test_e.txt",
    ] {
        write(&root, path, "");
    }
    assert_eq!(
        shown(test_files(&root, None)),
        Ok(vec!["test_.py".to_owned(), "test_a.py".to_owned()]),
        "test_*.py alone, and never a folder of that name"
    );
}

#[test]
fn the_walk_skips_each_hidden_folder_and_file() {
    let root = project("hidden");
    write(&root, "test_a.py", "");
    write(&root, ".venv/lib/test_installed.py", "");
    write(&root, "tests/.cache/test_c.py", "");
    assert_eq!(
        shown(test_files(&root, None)),
        Ok(vec!["test_a.py".to_owned()]),
        "no test file under a hidden folder"
    );
    write(&root, "y_test.py", "");
    write(&root, ".x_test.py", "");
    assert_eq!(
        shown(suffix_test_files(&root, None)),
        Ok(vec!["y_test.py".to_owned()]),
        "no hidden test file"
    );
}

#[test]
fn a_path_to_a_hidden_file_says_that_it_is_hidden() {
    let root = project("hidden_file");
    write(&root, ".x_test.py", "");
    assert_eq!(
        suffix_test_files(&root, Some(Path::new(".x_test.py"))),
        Err(NoTestFile::Hidden(PathBuf::from(".x_test.py"))),
        "the file is hidden, and no .gitignore ignores it"
    );
}

#[test]
fn outside_git_a_path_to_a_hidden_file_says_that_it_is_hidden() {
    // The folder of cargo's temporary files lies in purba's own clone.
    let made = std::env::temp_dir().join(format!("purba-hidden-file-{}", std::process::id()));
    fs::create_dir_all(&made).expect("the test makes its folder");
    let root = made
        .canonicalize()
        .expect("the folder has a canonical path");
    write(&root, ".x_test.py", "");
    let found = suffix_test_files(&root, Some(Path::new(".x_test.py")));
    fs::remove_dir_all(&root).expect("the test removes its folder");
    assert_eq!(
        found,
        Err(NoTestFile::Hidden(PathBuf::from(".x_test.py"))),
        "with no .git, no .gitignore is read, and the file is hidden"
    );
}

#[test]
fn a_hidden_name_that_matches_no_pattern_keeps_its_reason() {
    let root = project("hidden_name_no_pattern");
    write(&root, ".test_a.py", "");
    assert_eq!(
        test_files(&root, Some(Path::new(".test_a.py"))),
        Err(NoTestFile::Name(Patterns::default())),
        "the name is told before the dot"
    );
}

#[test]
fn a_hidden_file_that_a_gitignore_ignores_is_hidden() {
    let root = project("hidden_ignored_file");
    write(&root, ".gitignore", ".x_test.py\n");
    write(&root, ".x_test.py", "");
    assert_eq!(
        suffix_test_files(&root, Some(Path::new(".x_test.py"))),
        Err(NoTestFile::Hidden(PathBuf::from(".x_test.py"))),
        "the leading dot is told first, as for a folder on the way"
    );
}

#[test]
fn a_hidden_file_in_a_hidden_folder_gets_the_reason_of_the_folder() {
    let root = project("hidden_file_in_hidden_folder");
    write(&root, ".cache/.x_test.py", "");
    assert_eq!(
        suffix_test_files(&root, Some(Path::new(".cache/.x_test.py"))),
        Err(NoTestFile::Hidden(PathBuf::from(".cache"))),
        "the folders on the way are told before the file"
    );
}

#[test]
fn a_root_that_is_hidden_is_walked() {
    let root = project(".hidden_root");
    write(&root, "test_a.py", "");
    assert_eq!(
        shown(test_files(&root, None)),
        Ok(vec!["test_a.py".to_owned()]),
        "only a file or folder below the root can be hidden"
    );
}

#[test]
fn the_walk_skips_what_a_gitignore_of_the_project_ignores() {
    let root = project("gitignore");
    write(&root, ".gitignore", "gen/\n");
    write(&root, "tests/.gitignore", "test_local_*.py\n");
    write(&root, "test_a.py", "");
    write(&root, "gen/test_gen.py", "");
    write(&root, "tests/test_b.py", "");
    write(&root, "tests/test_local_c.py", "");
    write(&root, "tests/gen/test_deep.py", "");
    assert_eq!(
        shown(test_files(&root, None)),
        Ok(vec!["test_a.py".to_owned(), "tests/test_b.py".to_owned()]),
        "each .gitignore of the project, at its own folder and below"
    );
}

#[test]
fn a_gitignore_above_the_folder_of_the_path_ignores_what_it_names_there() {
    let root = project("gitignore_above_path");
    write(&root, ".gitignore", "tests/gen/\n");
    write(&root, "tests/test_a.py", "");
    write(&root, "tests/gen/test_gen.py", "");
    assert_eq!(
        shown(test_files(&root, Some(Path::new("tests")))),
        Ok(vec!["tests/test_a.py".to_owned()]),
        "the .gitignore of the root counts below the folder of the path"
    );
}

#[test]
fn a_root_that_a_gitignore_above_it_ignores_is_walked() {
    let repository = project("ignored_root");
    write(&repository, ".gitignore", "ignored/\n");
    let root = repository.join("ignored/project");
    write(&root, "tests/test_a.py", "");
    assert_eq!(
        shown(test_files(&root, None)),
        Ok(vec!["tests/test_a.py".to_owned()]),
        "stage one never skips the root itself"
    );
    assert_eq!(
        shown(test_files(&root, Some(Path::new("tests")))),
        Ok(vec!["tests/test_a.py".to_owned()]),
        "nor a path under the root, for a folder above the root"
    );
}

#[test]
fn the_walk_reads_no_exclude_of_a_clone_and_no_ignore_file() {
    let root = project("exclude");
    write(&root, ".git/info/exclude", "mine/\n");
    write(&root, ".ignore", "other/\n");
    write(&root, "mine/test_m.py", "");
    write(&root, "other/test_o.py", "");
    assert_eq!(
        shown(test_files(&root, None)),
        Ok(vec![
            "mine/test_m.py".to_owned(),
            "other/test_o.py".to_owned()
        ]),
        "a file on one machine changes no outcome"
    );
}

#[test]
fn outside_git_a_gitignore_ignores_nothing() {
    // The folder of cargo's temporary files lies in purba's own clone.
    let made = std::env::temp_dir().join(format!("purba-stage-one-{}", std::process::id()));
    fs::create_dir_all(&made).expect("the test makes its folder");
    let root = made
        .canonicalize()
        .expect("the folder has a canonical path");
    write(&root, ".gitignore", "gen/\n");
    write(&root, "gen/test_gen.py", "");
    let found = shown(test_files(&root, None));
    fs::remove_dir_all(&root).expect("the test removes its folder");
    assert_eq!(
        found,
        Ok(vec!["gen/test_gen.py".to_owned()]),
        "with no .git, the walk ignores nothing"
    );
}

#[test]
fn a_gitignore_above_the_root_counts_up_to_the_folder_that_holds_git() {
    let above = folder("above");
    write(&above, ".gitignore", "test_far.py\n");
    let repository = above.join("repository");
    fs::create_dir_all(repository.join(".git")).expect("the test makes .git");
    write(&repository, ".gitignore", "gen/\n");
    let root = repository.join("project");
    write(&root, "gen/test_gen.py", "");
    write(&root, "test_far.py", "");
    write(&root, "test_a.py", "");
    assert_eq!(
        shown(test_files(&root, None)),
        Ok(vec!["test_a.py".to_owned(), "test_far.py".to_owned()]),
        "the .gitignore of the repository counts, and the one above it does not"
    );
}

#[cfg(unix)]
#[test]
fn the_walk_follows_no_symbolic_link() {
    let root = project("links");
    let far = folder("links_far");
    write(&far, "test_far.py", "");
    write(&root, "test_a.py", "");
    std::os::unix::fs::symlink(&far, root.join("linked")).expect("the test links a folder");
    std::os::unix::fs::symlink(root.join("test_a.py"), root.join("test_b.py"))
        .expect("the test links a file");
    assert_eq!(
        shown(test_files(&root, None)),
        Ok(vec!["test_a.py".to_owned()]),
        "neither a linked folder nor a linked file"
    );
}

#[test]
fn a_path_starts_at_the_root() {
    let root = project("path");
    write(&root, "tests/test_a.py", "");
    write(&root, "tests/deep/test_b.py", "");
    write(&root, "other/test_c.py", "");
    assert_eq!(
        shown(test_files(&root, Some(Path::new("tests")))),
        Ok(vec![
            "tests/deep/test_b.py".to_owned(),
            "tests/test_a.py".to_owned()
        ]),
        "a folder gives each test file under it"
    );
    assert_eq!(
        shown(test_files(&root, Some(Path::new("tests/test_a.py")))),
        Ok(vec!["tests/test_a.py".to_owned()]),
        "a file gives itself"
    );
    assert_eq!(
        shown(test_files(&root, Some(&root.join("tests/test_a.py")))),
        Ok(vec!["tests/test_a.py".to_owned()]),
        "an absolute path under the root gives its path from the root"
    );
    assert_eq!(
        shown(test_files(&root, Some(Path::new(".")))),
        shown(test_files(&root, None)),
        "the root as a path is the root"
    );
}

#[test]
fn a_path_that_names_no_test_file_says_why() {
    let root = project("reasons");
    write(&root, ".gitignore", "gen/\nignored_*.py\ntest_ignored.py\n");
    write(&root, "gen/test_gen.py", "");
    write(&root, ".venv/lib/test_installed.py", "");
    write(&root, "tests/models_test.py", "");
    write(&root, "test_ignored.py", "");
    write(&root, "empty/helper.py", "");
    let outside = folder("reasons_outside");
    write(&outside, "test_far.py", "");
    for (path, why) in [
        (
            "tests/models_test.py",
            NoTestFile::Name(Patterns::default()),
        ),
        ("gen/test_gen.py", NoTestFile::Ignored(PathBuf::from("gen"))),
        ("gen", NoTestFile::Ignored(PathBuf::from("gen"))),
        (
            "test_ignored.py",
            NoTestFile::Ignored(PathBuf::from("test_ignored.py")),
        ),
        (
            ".venv/lib/test_installed.py",
            NoTestFile::Hidden(PathBuf::from(".venv")),
        ),
        (".venv/lib", NoTestFile::Hidden(PathBuf::from(".venv"))),
        ("nowhere", NoTestFile::Missing),
        ("tests/models_test.py/x", NoTestFile::Missing),
        ("../reasons_outside/test_far.py", NoTestFile::Outside),
    ] {
        assert_eq!(
            test_files(&root, Some(Path::new(path))),
            Err(why.clone()),
            "{path}: {why}"
        );
    }
    assert_eq!(
        test_files(&root, Some(&outside.join("test_far.py"))),
        Err(NoTestFile::Outside),
        "an absolute path outside the root"
    );
    assert_eq!(
        shown(test_files(&root, Some(Path::new("empty")))),
        Ok(Vec::new()),
        "a folder with no test file names something, and gives no test file"
    );
}

#[cfg(unix)]
#[test]
fn a_folder_that_stage_one_cannot_read_comes_with_its_error_beside_the_test_files() {
    use std::os::unix::fs::PermissionsExt;

    let root = project("unread");
    write(&root, "tests/test_a.py", "");
    write(&root, "secret/test_s.py", "");
    let secret = root.join("secret");
    fs::set_permissions(&secret, fs::Permissions::from_mode(0o000))
        .expect("the test takes the folder's permissions");
    // A user that reads a folder with no permission, such as root, reads it.
    let readable = fs::read_dir(&secret).is_ok();
    let found = test_files(&root, None);
    fs::set_permissions(&secret, fs::Permissions::from_mode(0o700))
        .expect("the test gives the folder's permissions back");
    if readable {
        return;
    }
    assert_eq!(
        found,
        Ok(TestFiles {
            files: vec![PathBuf::from("tests/test_a.py")],
            unread: vec![(
                PathBuf::from("secret"),
                "Permission denied (os error 13)".to_owned()
            )],
        }),
        "the test files that stage one found, and the folder it cannot read"
    );
}

#[test]
fn each_reason_reads_in_its_words() {
    for (why, words) in [
        (
            NoTestFile::Name(Patterns::default()),
            "its name does not match test_*.py",
        ),
        (
            NoTestFile::Ignored(PathBuf::from("gen")),
            "stage one skips gen, which a .gitignore ignores",
        ),
        (
            NoTestFile::Hidden(PathBuf::from(".venv")),
            "stage one skips .venv, a hidden file or folder",
        ),
        (NoTestFile::Missing, "no file or folder has that path"),
        (NoTestFile::Outside, "it is outside the root"),
    ] {
        assert_eq!(why.to_string(), words, "{why:?}");
    }
}
