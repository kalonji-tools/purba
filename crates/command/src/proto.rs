//! PROTOTYPE for #1229, never merge: stage one, which picks the test files
//! under a folder from their paths alone, by the strategy that
//! `PURBA_PROTO_MARKERS` names.

use std::path::{Component, Path, PathBuf};

/// A line of a check: the path, the kind of check, and its words.
pub type Line = (PathBuf, &'static str, String);

/// What stage one picks under a root, and each line of its check.
pub struct Picked {
    /// Each file picked, as a path the tester could type, in path order.
    pub files: Vec<PathBuf>,
    /// Each line of a check: the path, the kind of check, and its words.
    pub lines: Vec<Line>,
}

/// Each `.py` file under `here`, relative to `root`, in path order. A folder or
/// a file whose name starts with `.` is skipped, and so is a linked folder.
fn walk(root: &Path, here: &Path, found: &mut Vec<PathBuf>) {
    let Ok(entries) = std::fs::read_dir(here) else {
        return;
    };
    let mut entries: Vec<PathBuf> = entries
        .filter_map(|entry| entry.ok().map(|entry| entry.path()))
        .collect();
    entries.sort();
    for path in entries {
        let name = path.file_name().unwrap_or_default().to_string_lossy();
        if name.starts_with('.') {
            continue;
        }
        if path.is_dir() && !path.is_symlink() {
            walk(root, &path, found);
        } else if path.extension().is_some_and(|extension| extension == "py") && path.is_file() {
            found.push(path.strip_prefix(root).unwrap_or(&path).to_path_buf());
        }
    }
}

/// The stem of `path`, the file name with no `.py`.
fn stem(path: &Path) -> String {
    path.file_stem()
        .unwrap_or_default()
        .to_string_lossy()
        .into_owned()
}

/// Whether the name of `path` matches `test_*.py` or `*_test.py`.
#[must_use]
pub fn named_as_test(path: &Path) -> bool {
    let stem = stem(path);
    stem.starts_with("test_") || stem.ends_with("_test")
}

/// Whether the first or the last `_` part of the stem of `path` is within one
/// edit of `test`, case ignored, when the name does not match.
fn near_the_name(path: &Path) -> bool {
    let stem = stem(path);
    let parts: Vec<&str> = stem.split('_').filter(|part| !part.is_empty()).collect();
    !named_as_test(path)
        && (parts
            .first()
            .is_some_and(|part| purba_selection::proto::near_test(part))
            || parts
                .last()
                .is_some_and(|part| purba_selection::proto::near_test(part)))
}

/// Whether a folder named `test`, `tests` or `testing` stands on `root`
/// joined with the folder of `relative`.
fn in_a_test_folder(root: &Path, relative: &Path) -> bool {
    let folders = root
        .components()
        .chain(relative.parent().into_iter().flat_map(Path::components));
    folders
        .filter_map(|component| match component {
            Component::Normal(name) => Some(name.to_string_lossy().into_owned()),
            _ => None,
        })
        .any(|name| matches!(name.as_str(), "test" | "tests" | "testing"))
}

/// The folders that `tests = [...]` lists under `[tool.purba]`, in the file
/// that `PURBA_PROTO_LIST` names, or else in `root/pyproject.toml`. It reads
/// one line `tests = ["a", "b"]`. `None` when no such line exists.
fn listed(root: &Path) -> Option<Vec<String>> {
    let file = std::env::var_os("PURBA_PROTO_LIST")
        .map_or_else(|| root.join("pyproject.toml"), PathBuf::from);
    let text = std::fs::read_to_string(file).ok()?;
    let mut in_table = false;
    for line in text.lines() {
        let line = line.trim();
        if line.starts_with('[') && !line.starts_with("[[") && !line.contains('=') {
            in_table = line == "[tool.purba]";
            continue;
        }
        if !in_table {
            continue;
        }
        let Some((key, value)) = line.split_once('=') else {
            continue;
        };
        if key.trim() != "tests" {
            continue;
        }
        let entries = value
            .trim()
            .trim_start_matches('[')
            .trim_end_matches(']')
            .split(',')
            .map(|entry| {
                entry
                    .trim()
                    .trim_matches('"')
                    .trim_end_matches('/')
                    .to_owned()
            })
            .filter(|entry| !entry.is_empty())
            .collect();
        return Some(entries);
    }
    None
}

/// The folder of `relative` as a list entry writes it: `.` for the root.
fn folder_of(relative: &Path) -> String {
    match relative.parent() {
        Some(parent) if !parent.as_os_str().is_empty() => parent.to_string_lossy().into_owned(),
        _ => ".".to_owned(),
    }
}

/// What stage one picks under `root` by the strategy, and each line of its
/// check. `baseline` picks every `.py` file, as if each were given alone.
#[must_use]
pub fn pick(root: &Path) -> Picked {
    let strategy = purba_parsing::proto_strategy();
    let mut all = Vec::new();
    walk(root, root, &mut all);
    let shown = |relative: &Path| {
        if root == Path::new(".") {
            relative.to_path_buf()
        } else {
            root.join(relative)
        }
    };
    let mut files = Vec::new();
    let mut lines = Vec::new();
    let list = if strategy == "list-and-check" {
        let list = listed(root);
        match &list {
            None => lines.push((
                root.to_path_buf(),
                "list-missing",
                "[tool.purba] lists no tests, so no file is a test file".to_owned(),
            )),
            Some(entries) => {
                for entry in entries {
                    if !root.join(entry).is_dir() {
                        lines.push((
                            root.join(entry),
                            "list-missing",
                            format!("[tool.purba] tests lists {entry}, and no such folder exists"),
                        ));
                    }
                }
            }
        }
        list.unwrap_or_default()
    } else {
        Vec::new()
    };
    for relative in all {
        let take = match strategy {
            "name" | "name-and-check" | "visibility" => named_as_test(&relative),
            "folder" => in_a_test_folder(root, &relative),
            "list-and-check" => list.contains(&folder_of(&relative)),
            _ => true,
        };
        if strategy == "name-and-check" && near_the_name(&relative) {
            lines.push((
                shown(&relative),
                "file-near",
                "the file name is near test_*.py, and it is not a test file".to_owned(),
            ));
        }
        if strategy == "list-and-check" && !take && named_as_test(&relative) {
            lines.push((
                shown(&relative),
                "list-outside",
                "the file name matches test_*.py, and no folder of [tool.purba] tests holds it"
                    .to_owned(),
            ));
        }
        if take {
            files.push(shown(&relative));
        }
    }
    Picked { files, lines }
}
