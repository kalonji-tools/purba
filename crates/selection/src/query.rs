//! The tests that a query names: stage one keeps the test files that a query
//! can reach by their paths, and selection picks the tests of each file that
//! stage two parses.

use std::fmt;
use std::path::{Component, Path, PathBuf};

use purba_config::Dial;
use purba_parsing::Facts;
pub use purba_query::Aliases;

use crate::pattern::{Patterns, kind};
use crate::stage_one::{NoTestFile, TestFiles, test_files};
use crate::{Selected, TestId};

/// A query that parses, and the name and the form of each alias that expanded
/// it, so that a bare word that names nothing can name the nearest alias.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Query {
    tree: purba_query::Query,
    aliases: Vec<(String, String)>,
}

impl Query {
    /// The query with each alias of `aliases` expanded.
    ///
    /// # Errors
    ///
    /// A call names no alias, hands an alias the wrong count of arguments, or
    /// the expansion nests deeper than 255 levels.
    pub fn expand(&self, aliases: &Aliases) -> Result<Self, Unparsed> {
        let tree = aliases.expand(&self.tree).map_err(Unparsed)?;
        let aliases = aliases
            .names()
            .into_iter()
            .zip(aliases.forms())
            .map(|(name, form)| (name.to_owned(), form.to_owned()))
            .collect();
        Ok(Self { tree, aliases })
    }
}

/// Why the text of a query is no query, in the words of query.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Unparsed(purba_query::Refused);

impl fmt::Display for Unparsed {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(formatter)
    }
}

/// The query that `text` holds. Each call of an alias stays until
/// [`Query::expand`] expands it.
///
/// # Errors
///
/// Why the text is no query.
pub fn query(text: &str) -> Result<Query, Unparsed> {
    purba_query::parse(text)
        .map(|tree| Query {
            tree,
            aliases: Vec::new(),
        })
        .map_err(Unparsed)
}

/// `[tool.purba.queries]`: the aliases of the project's file.
pub static QUERIES: Dial<Aliases> = Dial::new("queries", read_aliases, Aliases::default);

/// The aliases that the value of `queries` declares, or the words of each
/// error of the value.
fn read_aliases(value: &toml::Value) -> Result<Aliases, Vec<String>> {
    let toml::Value::Table(table) = value else {
        return Err(vec![format!(
            "queries under [tool.purba] is {}; write a table, such as [tool.purba.queries]",
            kind(value)
        )]);
    };
    let mut errors = Vec::new();
    let mut declared = Vec::new();
    for (key, value) in table {
        match value {
            toml::Value::String(text) => declared.push((key.as_str(), text.as_str())),
            _ => errors.push(format!(
                "{key} under [tool.purba.queries] is {}; write a query in a string, \
                 such as fast = \"!path(tests/slow/**)\"",
                kind(value)
            )),
        }
    }
    if !errors.is_empty() {
        return Err(errors);
    }
    Aliases::new(declared, purba_config::nearest)
}

/// A term of a query that names nothing that purba read.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Unnamed {
    /// The path of an id, as the tester typed it, names no test file, for the
    /// reason of stage one.
    File(String, NoTestFile),
    /// The term, as the tester typed it, names nothing.
    Nothing {
        /// The term as the tester typed it.
        term: String,
        /// What kind of thing the term names.
        kind: Kind,
        /// The nearest thing of that kind that purba read, if any.
        nearest: Option<String>,
        /// How many test files the query reaches, of how many stage one found,
        /// where a file left out could hold what the term names.
        reached: Option<(usize, usize)>,
    },
}

/// The kind of thing that a term names.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Kind {
    /// A folder, a file or a test.
    Id,
    /// A folder, a file or a test, or an alias, where an id is nearer than an
    /// alias.
    IdOrAlias,
    /// A folder, a file or a test, or an alias, where an alias is nearer.
    Alias,
    /// The name of a test.
    Name,
    /// A test file, by a glob.
    Path,
}

impl fmt::Display for Unnamed {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::File(term, why) => write!(formatter, "{term} names no test file: {why}"),
            Self::Nothing {
                term,
                kind,
                nearest,
                reached,
            } => {
                let (what, near) = match kind {
                    Kind::Id => ("test", "the nearest is"),
                    Kind::IdOrAlias => ("test or alias", "the nearest is"),
                    Kind::Alias => ("test or alias", "the nearest alias is"),
                    Kind::Name => ("test", "the nearest name is"),
                    Kind::Path => ("test file", "the nearest is"),
                };
                write!(formatter, "{term} names no {what}")?;
                if let Some(nearest) = nearest {
                    write!(formatter, "; {near} {nearest}")?;
                }
                if let Some((reached, found)) = reached {
                    write!(
                        formatter,
                        "; the query reaches {reached} of {found} test files"
                    )?;
                }
                Ok(())
            }
        }
    }
}

/// A term whose id stage one resolved from the root.
#[derive(Debug, Clone)]
enum Term {
    /// A folder or a file, by its path from the root, or a test of a file,
    /// by the classes and the function after the file, joined by `::`.
    Id {
        text: String,
        path: PathBuf,
        test: Option<String>,
    },
    Name(String),
    Path(String),
    Not(Box<Self>),
    And(Vec<Self>),
    Or(Vec<Self>),
}

/// Whether a term picks a test of a file, judged from the file's path alone.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Reach {
    Every,
    Some,
    None,
}

impl Term {
    /// The term of `tree`, with each id resolved from `root`. `aliases` holds
    /// the name and the form of each alias that expanded the tree.
    fn of(
        tree: &purba_query::Query,
        root: &Path,
        patterns: &Patterns,
        found: &[PathBuf],
        aliases: &[(String, String)],
    ) -> Result<Self, Unnamed> {
        let each = |trees: &[purba_query::Query]| -> Result<Vec<Self>, Unnamed> {
            trees
                .iter()
                .map(|tree| Self::of(tree, root, patterns, found, aliases))
                .collect()
        };
        Ok(match tree {
            purba_query::Query::Id(text) => resolve(text, root, patterns, found, aliases)?,
            purba_query::Query::Name(text) => Self::Name(text.clone()),
            purba_query::Query::Path(glob) => Self::Path(glob.clone()),
            purba_query::Query::Not(term) => {
                Self::Not(Box::new(Self::of(term, root, patterns, found, aliases)?))
            }
            purba_query::Query::And(terms) => Self::And(each(terms)?),
            purba_query::Query::Or(terms) => Self::Or(each(terms)?),
            // A query that no alias expanded names no alias.
            purba_query::Query::Call { name, .. } => {
                return Err(Unnamed::Nothing {
                    term: format!("{name}("),
                    kind: Kind::Id,
                    nearest: None,
                    reached: None,
                });
            }
        })
    }

    /// Whether the term picks a test of the file at `file`, from the root.
    fn reach(&self, file: &Path) -> Reach {
        match self {
            Self::Id { path, test, .. } => match test {
                _ if !file.starts_with(path) => Reach::None,
                None => Reach::Every,
                Some(_) if file == path => Reach::Some,
                Some(_) => Reach::None,
            },
            Self::Name(_) => Reach::Some,
            Self::Path(glob) => {
                if matches(glob, file) {
                    Reach::Every
                } else {
                    Reach::None
                }
            }
            Self::Not(term) => match term.reach(file) {
                Reach::Every => Reach::None,
                Reach::Some => Reach::Some,
                Reach::None => Reach::Every,
            },
            Self::And(terms) => {
                let each: Vec<Reach> = terms.iter().map(|term| term.reach(file)).collect();
                if each.contains(&Reach::None) {
                    Reach::None
                } else if each.iter().all(|reach| *reach == Reach::Every) {
                    Reach::Every
                } else {
                    Reach::Some
                }
            }
            Self::Or(terms) => {
                let each: Vec<Reach> = terms.iter().map(|term| term.reach(file)).collect();
                if each.contains(&Reach::Every) {
                    Reach::Every
                } else if each.iter().all(|reach| *reach == Reach::None) {
                    Reach::None
                } else {
                    Reach::Some
                }
            }
        }
    }

    /// Whether the term picks `test`.
    fn picks(&self, candidate: &TestId) -> bool {
        match self {
            Self::Id { path, test, .. } => test.as_ref().map_or_else(
                || candidate.path.starts_with(path),
                |test| candidate.path == *path && *test == after_the_file(candidate),
            ),
            Self::Name(part) => candidate.function.contains(part.as_str()),
            Self::Path(glob) => matches(glob, &candidate.path),
            Self::Not(term) => !term.picks(candidate),
            Self::And(terms) => terms.iter().all(|term| term.picks(candidate)),
            Self::Or(terms) => terms.iter().any(|term| term.picks(candidate)),
        }
    }

    /// Each term of the tree that names a thing, in the order of the text.
    fn leaves<'t>(&'t self, into: &mut Vec<&'t Self>) {
        match self {
            Self::Not(term) => term.leaves(into),
            Self::And(terms) | Self::Or(terms) => {
                for term in terms {
                    term.leaves(into);
                }
            }
            _ => into.push(self),
        }
    }
}

/// The classes and the function of `test`, joined by `::`: its id after the
/// path of its file.
pub fn after_the_file(test: &TestId) -> String {
    let mut parts = test.classes.clone();
    parts.push(test.function.clone());
    parts.join("::")
}

/// The term of the id `text`, its path resolved from `root`.
fn resolve(
    text: &str,
    root: &Path,
    patterns: &Patterns,
    found: &[PathBuf],
    aliases: &[(String, String)],
) -> Result<Term, Unnamed> {
    let (typed, in_file) = text
        .split_once("::")
        .map_or((text, None), |(before, after)| {
            (before, Some(after.to_owned()))
        });
    let missing = || {
        let id = nearest_path(typed, root, found);
        let names: Vec<&str> = aliases.iter().map(|(name, _)| name.as_str()).collect();
        let could_be_an_alias = !aliases.is_empty()
            && text
                .chars()
                .all(|c| c.is_alphanumeric() || matches!(c, '_' | '-'));
        let alias = could_be_an_alias
            .then(|| purba_config::nearest(text, &names))
            .flatten();
        let (kind, nearest) = match (alias, id) {
            (_, id) if !could_be_an_alias => (Kind::Id, id),
            (Some(alias), Some(id))
                if purba_config::distance(text, &id) < purba_config::distance(text, alias) =>
            {
                (Kind::IdOrAlias, Some(id))
            }
            (Some(alias), _) => (
                Kind::Alias,
                aliases
                    .iter()
                    .find(|(name, _)| name == alias)
                    .map(|(_, form)| form.clone()),
            ),
            (None, id) => (Kind::IdOrAlias, id),
        };
        Unnamed::Nothing {
            term: text.to_owned(),
            kind,
            nearest,
            reached: None,
        }
    };
    match test_files(root, Some(Path::new(typed)), patterns) {
        Ok(_) => {}
        Err(NoTestFile::Missing) => return Err(missing()),
        Err(why) => return Err(Unnamed::File(text.to_owned(), why)),
    }
    let path = root
        .join(typed)
        .canonicalize()
        .ok()
        .and_then(|real| real.strip_prefix(root).ok().map(Path::to_path_buf))
        .ok_or_else(missing)?;
    if in_file.is_some() && !found.contains(&path) {
        return Err(missing());
    }
    Ok(Term::Id {
        text: text.to_owned(),
        path,
        test: in_file,
    })
}

/// The id nearest to the path `typed`, among the test files `found`, part by
/// part: the first part that no test file has there takes the nearest part
/// that one has.
fn nearest_path(typed: &str, root: &Path, found: &[PathBuf]) -> Option<String> {
    let typed = Path::new(typed);
    let typed = typed.strip_prefix(root).unwrap_or(typed);
    let parts: Vec<String> = typed
        .components()
        .filter_map(|part| match part {
            Component::Normal(part) => Some(part.to_string_lossy().into_owned()),
            _ => None,
        })
        .collect();
    let candidates: Vec<Vec<String>> = found
        .iter()
        .map(|file| {
            file.components()
                .map(|part| part.as_os_str().to_string_lossy().into_owned())
                .collect()
        })
        .collect();
    let near = nearest_parts(&parts, &candidates);
    (!near.is_empty()).then(|| near.join("/"))
}

/// The parts of the candidate nearest to `parts`, part by part, up to the first
/// part that differs: each part that a candidate shares with `parts`, then the
/// part of a candidate nearest to the first part that none shares. A tie goes
/// to the first candidate.
fn nearest_parts(parts: &[String], candidates: &[Vec<String>]) -> Vec<String> {
    let mut near: Vec<String> = Vec::new();
    for part in parts {
        let mut here: Vec<&String> = Vec::new();
        for candidate in candidates {
            if candidate.starts_with(&near)
                && let Some(next) = candidate.get(near.len())
                && !here.contains(&next)
            {
                here.push(next);
            }
        }
        if here.contains(&part) {
            near.push(part.clone());
            continue;
        }
        if let Some(nearest) = nearest(part, here.iter().map(|each| each.as_str())) {
            near.push(nearest.to_owned());
        }
        break;
    }
    near
}

/// The candidate nearest to `word` by edit distance, the first on a tie.
fn nearest<'c>(word: &str, candidates: impl Iterator<Item = &'c str>) -> Option<&'c str> {
    candidates.min_by_key(|candidate| purba_config::distance(word, candidate))
}

/// Whether the glob `glob` matches the path `file`, from the root. `*` and `?`
/// stay inside one part, and `**` crosses parts.
fn matches(glob: &str, file: &Path) -> bool {
    let glob: Vec<char> = glob.chars().collect();
    let file: Vec<char> = file.to_string_lossy().replace('\\', "/").chars().collect();
    glob_matches(&glob, &file)
}

fn glob_matches(glob: &[char], text: &[char]) -> bool {
    match glob {
        [] => text.is_empty(),
        ['*', '*', '/', rest @ ..] => {
            glob_matches(rest, text)
                || text
                    .iter()
                    .enumerate()
                    .any(|(at, &c)| c == '/' && glob_matches(rest, &text[at + 1..]))
        }
        ['*', '*', rest @ ..] => (0..=text.len()).any(|at| glob_matches(rest, &text[at..])),
        ['*', rest @ ..] => {
            let end = text.iter().position(|&c| c == '/').unwrap_or(text.len());
            (0..=end).any(|at| glob_matches(rest, &text[at..]))
        }
        ['?', rest @ ..] => text
            .first()
            .is_some_and(|&c| c != '/' && glob_matches(rest, &text[1..])),
        [c, rest @ ..] => text.first() == Some(c) && glob_matches(rest, &text[1..]),
    }
}

/// What stage one kept for a query: the test files that it can reach, and the
/// query with each id resolved from the root.
#[derive(Debug, Clone)]
pub struct Reached {
    files: TestFiles,
    found: Vec<PathBuf>,
    term: Option<Term>,
}

/// Stage one for `query`.
///
/// It keeps each test file under `root` whose name matches a pattern of
/// `patterns`, and that `query` can reach by its path, or each test file when
/// no query is given. Each id of the query starts at `root`.
///
/// # Errors
///
/// An id names no test file: stage one skips its path, its file's name is no
/// test file's, or no test file has its path.
pub fn reach(root: &Path, query: Option<&Query>, patterns: &Patterns) -> Result<Reached, Unnamed> {
    let all = test_files(root, None, patterns).unwrap_or_default();
    let Some(Query { tree, aliases }) = query else {
        return Ok(Reached {
            found: all.files.clone(),
            files: all,
            term: None,
        });
    };
    let term = Term::of(tree, root, patterns, &all.files, aliases)?;
    let files = TestFiles {
        files: all
            .files
            .iter()
            .filter(|file| term.reach(file) != Reach::None)
            .cloned()
            .collect(),
        unread: all
            .unread
            .iter()
            .filter(|(folder, _)| reaches_into(&term, folder))
            .cloned()
            .collect(),
    };
    Ok(Reached {
        files,
        found: all.files,
        term: Some(term),
    })
}

/// Whether `term` can pick a test of a file under `folder`.
fn reaches_into(term: &Term, folder: &Path) -> bool {
    match term {
        Term::Id { path, .. } => folder.starts_with(path) || path.starts_with(folder),
        Term::Name(_) | Term::Path(_) | Term::Not(_) => true,
        Term::And(terms) => terms.iter().all(|term| reaches_into(term, folder)),
        Term::Or(terms) => terms.iter().any(|term| reaches_into(term, folder)),
    }
}

impl Reached {
    /// The test files that the query reaches, and each folder there that stage
    /// one cannot read.
    #[must_use]
    pub const fn files(&self) -> &TestFiles {
        &self.files
    }

    /// What selection makes of the module at `path`, from its `facts`, with
    /// only the tests that the query picks. See [`crate::select`].
    #[must_use]
    pub fn select(&self, path: &Path, facts: &Facts<'_>) -> Selected {
        crate::select_by(path, facts, &|test| {
            self.term.as_ref().is_none_or(|term| term.picks(test))
        })
    }

    /// The first term of the query, in the order of its text, that names
    /// nothing in `read`: each file that stage two read, by its path from the
    /// root, beside what selection made of it, or `None` where it did not
    /// parse.
    ///
    /// # Errors
    ///
    /// That term, and its nearest of the same kind.
    pub fn check<'s>(
        &self,
        read: impl IntoIterator<Item = (&'s Path, Option<&'s Selected>)>,
    ) -> Result<(), Unnamed> {
        let Some(term) = &self.term else {
            return Ok(());
        };
        let read: Vec<(&Path, Option<&Selected>)> = read.into_iter().collect();
        let counted = (self.files.files.len() < self.found.len())
            .then_some((self.files.files.len(), self.found.len()));
        let mut leaves = Vec::new();
        term.leaves(&mut leaves);
        for leaf in leaves {
            match leaf {
                Term::Id {
                    text,
                    path,
                    test: Some(test),
                } => {
                    let Some((_, selected)) = read.iter().find(|(file, _)| *file == path) else {
                        return Err(Unnamed::Nothing {
                            term: text.clone(),
                            kind: Kind::Id,
                            nearest: None,
                            reached: counted,
                        });
                    };
                    let Some(selected) = selected else {
                        continue;
                    };
                    if selected.names.contains(test) {
                        continue;
                    }
                    let parts: Vec<String> = test.split("::").map(str::to_owned).collect();
                    let candidates: Vec<Vec<String>> = selected
                        .names
                        .iter()
                        .map(|name| name.split("::").map(str::to_owned).collect())
                        .collect();
                    let mut near = vec![path.display().to_string()];
                    near.extend(nearest_parts(&parts, &candidates));
                    return Err(Unnamed::Nothing {
                        term: text.clone(),
                        kind: Kind::Id,
                        nearest: Some(near.join("::")),
                        reached: None,
                    });
                }
                Term::Name(text) => {
                    let names: Vec<&str> = read
                        .iter()
                        .filter_map(|(_, selected)| *selected)
                        .flat_map(|selected| selected.functions.iter().map(String::as_str))
                        .collect();
                    let unparsed = read.iter().any(|(_, selected)| selected.is_none());
                    if unparsed || names.iter().any(|name| name.contains(text.as_str())) {
                        continue;
                    }
                    return Err(Unnamed::Nothing {
                        term: format!("name({text})"),
                        kind: Kind::Name,
                        nearest: nearest(text, names.into_iter()).map(str::to_owned),
                        reached: counted,
                    });
                }
                Term::Path(glob) => {
                    if self.found.iter().any(|file| matches(glob, file)) {
                        continue;
                    }
                    let fixed = glob.split(['*', '?']).next().unwrap_or_default();
                    let shown: Vec<String> = self
                        .found
                        .iter()
                        .map(|file| file.display().to_string())
                        .collect();
                    return Err(Unnamed::Nothing {
                        term: format!("path({glob})"),
                        kind: Kind::Path,
                        nearest: nearest(fixed, shown.iter().map(String::as_str))
                            .map(str::to_owned),
                        reached: None,
                    });
                }
                _ => {}
            }
        }
        Ok(())
    }
}
