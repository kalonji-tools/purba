use std::fs;
use std::path::{Path, PathBuf};

use purba_config::{Met, Walk, walk};

/// A new, empty folder for the test `name`, with a folder `.git` in it, so
/// that the walk up stops there. Canonical, as the walk names a folder.
fn repository(name: &str) -> PathBuf {
    let folder = PathBuf::from(env!("CARGO_TARGET_TMPDIR"))
        .join("walk")
        .join(name);
    if folder.exists() {
        fs::remove_dir_all(&folder).expect("the test removes its old folder");
    }
    fs::create_dir_all(folder.join(".git")).expect("the test makes its folder");
    folder.canonicalize().expect("the test's folder exists")
}

/// Writes `text` at `path` under `folder`, and makes each folder on its way.
fn write(folder: &Path, path: &str, text: &str) {
    let path = folder.join(path);
    fs::create_dir_all(path.parent().expect("a file has a folder"))
        .expect("the test makes the folders");
    fs::write(path, text).expect("the test writes the file");
}

/// What the walk met at `path` from the working folder, in `folder`.
fn met(path: &str, folder: &Path) -> Met {
    Met {
        path: PathBuf::from(path),
        folder: folder.to_owned(),
    }
}

#[test]
fn with_no_tool_purba_the_walk_names_the_project_and_the_folder_that_holds_git() {
    let project = repository("project");
    write(&project, "pyproject.toml", "[project]\nname = \"p\"\n");
    let start = project.join("tests/sub");
    fs::create_dir_all(&start).expect("the test makes the working folder");
    assert_eq!(
        walk(&start),
        Walk::Missing {
            project: Some(met("../../pyproject.toml", &project)),
            git: Some(met("../..", &project)),
            start: start.clone(),
        },
        "the pyproject.toml of the project, and the folder that holds .git, from the working folder"
    );

    let bare = repository("bare");
    let start = bare.join("tests");
    fs::create_dir_all(&start).expect("the test makes the working folder");
    assert_eq!(
        walk(&start),
        Walk::Missing {
            project: None,
            git: Some(met("..", &bare)),
            start: start.clone(),
        },
        "no project, and the folder that holds .git, from the working folder"
    );
}

#[test]
fn the_walk_stops_at_a_git_file() {
    let above = repository("git_file");
    write(&above, "pyproject.toml", "[tool.purba]\n");
    write(&above, "worktree/.git", "gitdir: elsewhere\n");
    let start = above.join("worktree");
    assert_eq!(
        walk(&start),
        Walk::Missing {
            project: None,
            git: Some(met(".", &start)),
            start: start.clone(),
        },
        "the [tool.purba] above the .git file is not the root"
    );
}

#[test]
fn a_pyproject_toml_that_is_not_toml_gives_its_words() {
    let top = repository("not_toml");
    write(&top, "pyproject.toml", "[tool.purba\n");
    let start = top.join("tests");
    fs::create_dir_all(&start).expect("the test makes the working folder");
    assert_eq!(
        walk(&start),
        Walk::Unread {
            file: PathBuf::from("../pyproject.toml"),
            words: "TOML parse error at line 1, column 12: unclosed table, expected `]`".to_owned(),
        },
        "the file from the working folder, and the line and the column of its error"
    );
}
