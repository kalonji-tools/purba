//! Stage one: the test files under the root, read from the paths of the files
//! alone. It reads no file's text.

use std::ffi::OsStr;
use std::fmt;
use std::path::{Path, PathBuf};

use ignore::WalkBuilder;

/// Why a path names no test file.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NoTestFile {
    /// The path names a file whose name does not match `test_*.py`.
    Name,
    /// The path names a file whose name matches none of the patterns in
    /// force, which `test-files` set.
    NameOf(Vec<&'static str>),
    /// A `.gitignore` of the project ignores this file or folder, by its path
    /// from the root, on the way to the path.
    Ignored(PathBuf),
    /// This folder, by its path from the root, is hidden, and lies on the way
    /// to the path.
    Hidden(PathBuf),
    /// No file or folder has the path.
    Missing,
    /// The path lies outside the root.
    Outside,
}

impl fmt::Display for NoTestFile {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Name => write!(formatter, "its name does not match test_*.py"),
            Self::NameOf(patterns) => {
                write!(
                    formatter,
                    "its name does not match {}",
                    patterns.join(" or ")
                )
            }
            Self::Ignored(path) => write!(
                formatter,
                "stage one skips {}, which a .gitignore ignores",
                path.display()
            ),
            Self::Hidden(path) => write!(
                formatter,
                "stage one skips {}, a hidden folder",
                path.display()
            ),
            Self::Missing => write!(formatter, "no file or folder has that path"),
            Self::Outside => write!(formatter, "it is outside the root"),
        }
    }
}

/// What stage one found under a path.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct TestFiles {
    /// Each test file, by its path from the root, in the order of the paths.
    pub files: Vec<PathBuf>,
    /// Each folder that the walk cannot read, by its path from the root, with
    /// the text of its error, in the order of the paths.
    pub unread: Vec<(PathBuf, String)>,
}

/// The test files under `path`, or under `root` when no path is given, and
/// each folder there that the walk cannot read.
///
/// Each is named by its path from `root`. `root` is canonical. A relative
/// `path` starts at `root`, and an absolute one must lie under it.
///
/// The walk skips each hidden folder below `root`, and each file or folder that
/// a `.gitignore` of the project ignores, as git reads it, up to the folder
/// that holds `.git`. It reads no global file of git, no `.git/info/exclude`
/// and no `.ignore`. Outside git, it ignores nothing. It follows no symbolic
/// link.
///
/// # Errors
///
/// Why `path` names no test file.
pub fn test_files(root: &Path, path: Option<&Path>) -> Result<TestFiles, NoTestFile> {
    test_files_matching(root, path, DEFAULT)
}

/// The closed list of patterns that `test-files` may name.
pub const PATTERNS: [&str; 4] = ["test_*.py", "*_test.py", "tests.py", "test.py"];

/// The patterns in force when `test-files` is absent.
pub const DEFAULT: &[&str] = &["test_*.py"];

/// [`test_files`], with `patterns`, each from [`PATTERNS`], in place of
/// `test_*.py`.
///
/// # Errors
///
/// Why `path` names no test file.
pub fn test_files_matching(
    root: &Path,
    path: Option<&Path>,
    patterns: &[&'static str],
) -> Result<TestFiles, NoTestFile> {
    let Some(path) = path else {
        return Ok(walk(root, root, patterns));
    };
    let real = root
        .join(path)
        .canonicalize()
        .map_err(|_| NoTestFile::Missing)?;
    let relative = real
        .strip_prefix(root)
        .map_err(|_| NoTestFile::Outside)?
        .to_path_buf();
    if real.is_dir() {
        if let Some(skipped) = skipped(root, &relative) {
            return Err(skipped);
        }
        return Ok(walk(root, &real, patterns));
    }
    if let Some(skipped) = relative.parent().and_then(|folder| skipped(root, folder)) {
        return Err(skipped);
    }
    if !real
        .file_name()
        .is_some_and(|name| is_test_name(name, patterns))
    {
        return Err(if patterns == DEFAULT {
            NoTestFile::Name
        } else {
            NoTestFile::NameOf(patterns.to_vec())
        });
    }
    if !entered(&real) {
        return Err(NoTestFile::Ignored(relative));
    }
    Ok(TestFiles {
        files: vec![relative],
        unread: Vec::new(),
    })
}

/// Whether `name` matches one of `patterns`.
fn is_test_name(name: &OsStr, patterns: &[&str]) -> bool {
    let name = name.as_encoded_bytes();
    patterns.iter().any(|pattern| match *pattern {
        "test_*.py" => {
            name.len() >= b"test_.py".len() && name.starts_with(b"test_") && name.ends_with(b".py")
        }
        "*_test.py" => name.len() > b"_test.py".len() && name.ends_with(b"_test.py"),
        exact => name == exact.as_bytes(),
    })
}

/// The walk of stage one from `from`, a folder at or under the root.
fn walker(from: &Path) -> WalkBuilder {
    let mut builder = WalkBuilder::new(from);
    builder
        .standard_filters(false)
        .hidden(true)
        .parents(true)
        .git_ignore(true)
        .require_git(true)
        .follow_links(false);
    builder
}

/// Each test file under `from`, and each folder there that the walk cannot
/// read, by its path from `root`, sorted.
fn walk(root: &Path, from: &Path, patterns: &[&str]) -> TestFiles {
    let mut found = TestFiles::default();
    for entry in walker(from).build() {
        match entry {
            Ok(entry) => {
                if entry.file_type().is_some_and(|kind| kind.is_file())
                    && is_test_name(entry.file_name(), patterns)
                    && let Ok(path) = entry.path().strip_prefix(root)
                {
                    found.files.push(path.to_path_buf());
                }
            }
            Err(error) => {
                if let Some(unread) = unread(root, &error) {
                    found.unread.push(unread);
                }
            }
        }
    }
    found.files.sort();
    found.unread.sort();
    found
}

/// The folder that `error` names, by its path from `root`, with the text of
/// the innermost error of the system under it, or `None` where `error` names
/// no path under `root`.
fn unread(root: &Path, error: &ignore::Error) -> Option<(PathBuf, String)> {
    let mut error = error;
    let path = loop {
        match error {
            ignore::Error::WithPath { path, .. } => break path,
            ignore::Error::WithDepth { err, .. } | ignore::Error::WithLineNumber { err, .. } => {
                error = err;
            }
            _ => return None,
        }
    };
    let path = path.strip_prefix(root).ok()?.to_path_buf();
    let mut text: &dyn std::error::Error = match error.io_error() {
        Some(io) => io,
        None => error,
    };
    while let Some(inner) = text.source() {
        text = inner;
    }
    Some((path, text.to_string()))
}

/// The first folder on the way from `root` to `folder`, `folder` among them,
/// that the walk does not enter, and why.
fn skipped(root: &Path, folder: &Path) -> Option<NoTestFile> {
    let mut on_the_way = PathBuf::new();
    for part in folder {
        on_the_way.push(part);
        if !entered(&root.join(&on_the_way)) {
            return Some(if part.as_encoded_bytes().starts_with(b".") {
                NoTestFile::Hidden(on_the_way)
            } else {
                NoTestFile::Ignored(on_the_way)
            });
        }
    }
    None
}

/// Whether the walk of the folder that holds `path` reaches `path`.
fn entered(path: &Path) -> bool {
    let Some(parent) = path.parent() else {
        return true;
    };
    walker(parent)
        .max_depth(Some(1))
        .build()
        .filter_map(Result::ok)
        .any(|entry| entry.depth() == 1 && entry.path() == path)
}
