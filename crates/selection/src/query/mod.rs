//! Prototype of one query language, built twice: on pest and on chumsky. This
//! branch is never merged.
//!
//! Both crates build one [`Query`]. Everything after the tree is shared: the
//! scan of depth, the aliases, stage one's verdict on a path, a test's pick,
//! and the nearest name of an empty pick.
#![allow(
    clippy::all,
    clippy::pedantic,
    clippy::nursery,
    clippy::cargo,
    missing_docs,
    rustdoc::all
)]

mod with_chumsky;
mod with_pest;

use std::collections::{BTreeSet, HashMap};

/// The most levels a query nests: each `(` of a group, each `!`, and each
/// alias call opens one.
pub const LIMIT: usize = 255;

/// The tree of a query, from either crate.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Query {
    Or(Vec<Query>),
    And(Vec<Query>),
    Not(Box<Query>),
    /// An id, or a prefix of one. A bare word that names an alias is one too,
    /// until the aliases expand.
    Id(String),
    Tag(String),
    Name(String),
    Path(String),
    /// A call `word(query)`, which only an alias with an argument answers.
    Call(String, Box<Query>),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Crate {
    Pest,
    Chumsky,
}

impl Crate {
    pub const fn name(self) -> &'static str {
        match self {
            Self::Pest => "pest",
            Self::Chumsky => "chumsky",
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Options {
    pub parser: Crate,
    /// Whether the depth limit applies.
    pub limit: bool,
}

/// One alias of `[tool.purba.queries]`: its name, its parameter, its body.
#[derive(Debug, Clone)]
pub struct Alias {
    pub name: String,
    pub parameter: Option<String>,
    pub body: String,
}

impl Alias {
    /// The alias that the key `key` names, such as `stable(x)`.
    pub fn new(key: &str, body: &str) -> Self {
        match key.split_once('(') {
            Some((name, rest)) => Self {
                name: name.trim().to_owned(),
                parameter: Some(rest.trim_end_matches(')').trim().to_owned()),
                body: body.to_owned(),
            },
            None => Self {
                name: key.trim().to_owned(),
                parameter: None,
                body: body.to_owned(),
            },
        }
    }

    fn key(&self) -> String {
        match &self.parameter {
            Some(parameter) => format!("{}({parameter})", self.name),
            None => self.name.clone(),
        }
    }
}

/// The tree of `text`, with every alias expanded.
pub fn parse(text: &str, aliases: &[Alias], options: Options) -> Result<Query, String> {
    let tree = parse_alone(text, options)?;
    let tree = expand(tree, aliases, options, &HashMap::new(), &mut Vec::new())?;
    if options.limit {
        let depth = depth(&tree);
        if depth > LIMIT + 1 {
            return Err(format!(
                "after its aliases expand, the query nests {} levels, and a query nests at most {LIMIT}",
                depth - 1
            ));
        }
    }
    Ok(tree)
}

fn parse_alone(text: &str, options: Options) -> Result<Query, String> {
    if options.limit {
        scan(text)?;
    }
    match options.parser {
        Crate::Pest => with_pest::parse(text),
        Crate::Chumsky => with_chumsky::parse(text),
    }
    .map_err(|error| format!("the query does not parse ({}):\n{error}", options.parser.name()))
}

const fn is_word(c: char) -> bool {
    c.is_ascii_alphanumeric() || matches!(c, '_' | '-' | '.' | ':' | '/' | '[' | ']' | '*' | '?')
}

/// Refuses a text that nests deeper than [`LIMIT`], before any crate parses it.
fn scan(text: &str) -> Result<(), String> {
    let chars: Vec<char> = text.chars().collect();
    // The `!` that wait for their operand, at each open group.
    let mut waiting: Vec<usize> = vec![0];
    let mut level = 0usize;
    let mut word = String::new();
    let mut i = 0;
    let atom_ends = |waiting: &mut Vec<usize>, level: &mut usize| {
        let top = waiting.last_mut().expect("the outer level");
        *level -= *top;
        *top = 0;
    };
    while i < chars.len() {
        let c = chars[i];
        if is_word(c) {
            word.push(c);
            i += 1;
            continue;
        }
        let before = std::mem::take(&mut word);
        if !before.is_empty() && c != '(' {
            atom_ends(&mut waiting, &mut level);
        }
        match c {
            '!' => {
                *waiting.last_mut().expect("the outer level") += 1;
                level += 1;
            }
            '(' if matches!(before.as_str(), "tag" | "name" | "path") => {
                while i < chars.len() && chars[i] != ')' {
                    i += 1;
                }
                atom_ends(&mut waiting, &mut level);
            }
            '(' => {
                waiting.push(0);
                level += 1;
            }
            ')' => {
                if waiting.len() > 1 {
                    let bangs = waiting.pop().expect("an open group");
                    level -= bangs + 1;
                }
                atom_ends(&mut waiting, &mut level);
            }
            _ => {}
        }
        if level > LIMIT {
            return Err(format!(
                "the query nests deeper than {LIMIT} levels at column {}; \
                 each `(` of a group, each `!` and each alias call opens a level",
                i + 1
            ));
        }
        i += 1;
    }
    Ok(())
}

/// The depth of `tree`, by an explicit stack, so a deep tree cannot overflow it.
fn depth(tree: &Query) -> usize {
    let mut deepest = 0;
    let mut stack = vec![(tree, 1usize)];
    while let Some((node, at)) = stack.pop() {
        deepest = deepest.max(at);
        match node {
            Query::Or(terms) | Query::And(terms) => {
                stack.extend(terms.iter().map(|term| (term, at + 1)));
            }
            Query::Not(term) | Query::Call(_, term) => stack.push((term, at + 1)),
            _ => {}
        }
    }
    deepest
}

fn expand(
    tree: Query,
    aliases: &[Alias],
    options: Options,
    bound: &HashMap<String, Query>,
    within: &mut Vec<String>,
) -> Result<Query, String> {
    Ok(match tree {
        Query::Or(terms) => Query::Or(
            terms
                .into_iter()
                .map(|term| expand(term, aliases, options, bound, within))
                .collect::<Result<_, _>>()?,
        ),
        Query::And(terms) => Query::And(
            terms
                .into_iter()
                .map(|term| expand(term, aliases, options, bound, within))
                .collect::<Result<_, _>>()?,
        ),
        Query::Not(term) => Query::Not(Box::new(expand(*term, aliases, options, bound, within)?)),
        Query::Id(word) => {
            if let Some(value) = bound.get(&word) {
                value.clone()
            } else if let Some(alias) = aliases
                .iter()
                .find(|alias| alias.name == word && alias.parameter.is_none())
            {
                body(alias, None, aliases, options, within)?
            } else {
                Query::Id(word)
            }
        }
        Query::Call(word, argument) => {
            let argument = expand(*argument, aliases, options, bound, within)?;
            match aliases
                .iter()
                .find(|alias| alias.name == word && alias.parameter.is_some())
            {
                Some(alias) => body(alias, Some(argument), aliases, options, within)?,
                None => {
                    let keys: Vec<String> = aliases
                        .iter()
                        .filter(|alias| alias.parameter.is_some())
                        .map(Alias::key)
                        .collect();
                    let nearest = nearest(&word, keys.iter().map(String::as_str))
                        .map_or_else(String::new, |key| format!("; the nearest is `{key}`"));
                    return Err(format!(
                        "`{word}(…)` names no alias with an argument in [tool.purba.queries]{nearest}"
                    ));
                }
            }
        }
        leaf => leaf,
    })
}

fn body(
    alias: &Alias,
    argument: Option<Query>,
    aliases: &[Alias],
    options: Options,
    within: &mut Vec<String>,
) -> Result<Query, String> {
    if let Some(first) = within.iter().position(|name| *name == alias.name) {
        let through: Vec<&str> = within[first + 1..].iter().map(String::as_str).collect();
        return Err(if through.is_empty() {
            format!("the alias `{}` names itself", alias.name)
        } else {
            format!(
                "the alias `{}` names itself through `{}`",
                alias.name,
                through.join("`, `")
            )
        });
    }
    let tree = parse_alone(&alias.body, options)
        .map_err(|error| format!("the alias `{}`: {error}", alias.key()))?;
    let mut bound = HashMap::new();
    if let (Some(parameter), Some(argument)) = (&alias.parameter, argument) {
        bound.insert(parameter.clone(), argument);
    }
    within.push(alias.name.clone());
    let tree = expand(tree, aliases, options, &bound, within);
    within.pop();
    tree
}

/// What a query reads of one test.
pub struct Test<'t> {
    pub path: &'t str,
    pub id: &'t str,
    pub function: &'t str,
    pub tags: &'t [String],
}

fn id_matches_path(id: &str, path: &str) -> bool {
    let id = id.trim_end_matches('/');
    id == "." || path == id || path.strip_prefix(id).is_some_and(|rest| rest.starts_with('/'))
}

impl Query {
    /// Stage one's verdict on the file at `path`: yes, no, or unknown until
    /// the file is parsed.
    pub fn file(&self, path: &str) -> Option<bool> {
        match self {
            Self::Or(_) => self.or_file(path),
            Self::And(_) => self.and_file(path),
            Self::Not(term) => term.file(path).map(|value| !value),
            Self::Id(id) => match id.split_once("::") {
                Some((file, _)) if file == path => None,
                Some(_) => Some(false),
                None => Some(id_matches_path(id, path)),
            },
            Self::Path(glob) => Some(matches_glob(glob.as_bytes(), path.as_bytes())),
            Self::Tag(_) | Self::Name(_) => None,
            Self::Call(..) => None,
        }
    }

    fn or_file(&self, path: &str) -> Option<bool> {
        let Self::Or(terms) = self else { unreachable!() };
        let mut unknown = false;
        for term in terms {
            match term.file(path) {
                Some(true) => return Some(true),
                Some(false) => {}
                None => unknown = true,
            }
        }
        if unknown { None } else { Some(false) }
    }

    fn and_file(&self, path: &str) -> Option<bool> {
        let Self::And(terms) = self else { unreachable!() };
        let mut unknown = false;
        for term in terms {
            match term.file(path) {
                Some(false) => return Some(false),
                Some(true) => {}
                None => unknown = true,
            }
        }
        if unknown { None } else { Some(true) }
    }

    /// Whether the query picks `test`.
    pub fn picks(&self, test: &Test<'_>) -> bool {
        match self {
            Self::Or(terms) => terms.iter().any(|term| term.picks(test)),
            Self::And(terms) => terms.iter().all(|term| term.picks(test)),
            Self::Not(term) => !term.picks(test),
            Self::Id(id) if id.contains("::") => {
                test.id == id
                    || test
                        .id
                        .strip_prefix(id.as_str())
                        .is_some_and(|rest| rest.starts_with("::"))
            }
            Self::Id(id) => id_matches_path(id, test.path),
            Self::Tag(tag) => test.tags.iter().any(|carried| carried == tag),
            Self::Name(part) => test.function.contains(part.as_str()),
            Self::Path(glob) => matches_glob(glob.as_bytes(), test.path.as_bytes()),
            Self::Call(..) => false,
        }
    }

    fn leaves<'q>(&'q self, into: &mut Vec<&'q Self>) {
        match self {
            Self::Or(terms) | Self::And(terms) => {
                for term in terms {
                    term.leaves(into);
                }
            }
            Self::Not(term) | Self::Call(_, term) => term.leaves(into),
            leaf => {
                if !into.contains(&leaf) {
                    into.push(leaf);
                }
            }
        }
    }

    fn text(&self) -> String {
        match self {
            Self::Id(id) => id.clone(),
            Self::Tag(tag) => format!("tag({tag})"),
            Self::Name(name) => format!("name({name})"),
            Self::Path(glob) => format!("path({glob})"),
            _ => String::new(),
        }
    }

    /// Why the query picks no test of `tests`, which stage one's `files` hold,
    /// of `parsed` files parsed.
    pub fn empty(&self, tests: &[Test<'_>], files: &[String], parsed: usize) -> String {
        let mut leaves = Vec::new();
        self.leaves(&mut leaves);
        let mut lines = vec![format!(
            "the query selects no test of the {} tests in the {parsed} files that purba parsed",
            tests.len()
        )];
        for leaf in leaves {
            if tests.iter().any(|test| leaf.picks(test)) {
                continue;
            }
            let (kind, candidates): (&str, BTreeSet<String>) = match leaf {
                Self::Id(_) => {
                    let mut all: BTreeSet<String> = files.iter().cloned().collect();
                    for file in files {
                        let mut at = file.as_str();
                        while let Some((parent, _)) = at.rsplit_once('/') {
                            all.insert(parent.to_owned());
                            at = parent;
                        }
                    }
                    all.extend(tests.iter().map(|test| test.id.to_owned()));
                    ("id", all)
                }
                Self::Tag(_) => (
                    "tag",
                    tests.iter().flat_map(|test| test.tags.iter().cloned()).collect(),
                ),
                Self::Name(_) => (
                    "name",
                    tests.iter().map(|test| test.function.to_owned()).collect(),
                ),
                _ => ("path", files.iter().cloned().collect()),
            };
            let bare = match leaf {
                Self::Tag(word) | Self::Name(word) | Self::Path(word) | Self::Id(word) => word,
                _ => continue,
            };
            let near = nearest(bare, candidates.iter().map(String::as_str))
                .map_or_else(|| format!("; purba read no {kind}"), |near| {
                    format!("; the nearest {kind} is `{near}`")
                });
            lines.push(format!("`{}` picks no test{near}", leaf.text()));
        }
        if lines.len() == 1 {
            lines.push("each part picks a test, and the operators leave none".to_owned());
        }
        lines.join("\n  ")
    }
}

/// `*` and `?` stay inside one directory; `**` crosses directories.
fn matches_glob(pattern: &[u8], path: &[u8]) -> bool {
    match pattern {
        [] => path.is_empty(),
        [b'*', b'*', b'/', rest @ ..] => {
            matches_glob(rest, path)
                || path
                    .iter()
                    .enumerate()
                    .any(|(i, &c)| c == b'/' && matches_glob(rest, &path[i + 1..]))
        }
        [b'*', b'*', rest @ ..] => (0..=path.len()).any(|i| matches_glob(rest, &path[i..])),
        [b'*', rest @ ..] => {
            let end = path.iter().position(|&c| c == b'/').unwrap_or(path.len());
            (0..=end).any(|i| matches_glob(rest, &path[i..]))
        }
        [b'?', rest @ ..] => path.first().is_some_and(|&c| c != b'/') && matches_glob(rest, &path[1..]),
        [c, rest @ ..] => path.first() == Some(c) && matches_glob(rest, &path[1..]),
    }
}

/// The candidate nearest to `word` by edit distance; the first on a tie.
fn nearest<'c>(word: &str, candidates: impl Iterator<Item = &'c str>) -> Option<&'c str> {
    candidates.min_by_key(|candidate| distance(word, candidate))
}

fn distance(a: &str, b: &str) -> usize {
    let b: Vec<char> = b.chars().collect();
    let mut row: Vec<usize> = (0..=b.len()).collect();
    for (i, ca) in a.chars().enumerate() {
        let mut previous = row[0];
        row[0] = i + 1;
        for (j, &cb) in b.iter().enumerate() {
            let here = row[j + 1];
            row[j + 1] = (row[j + 1] + 1).min(row[j] + 1).min(previous + usize::from(ca != cb));
            previous = here;
        }
    }
    row[b.len()]
}

/// The tree with one term in place of an `&` or `|` of one term.
fn one_or(mut terms: Vec<Query>, node: fn(Vec<Query>) -> Query) -> Query {
    if terms.len() == 1 { terms.pop().expect("one term") } else { node(terms) }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn both(text: &str) -> Query {
        let pest = with_pest::parse(text).expect("pest parses");
        let chumsky = with_chumsky::parse(text).expect("chumsky parses");
        assert_eq!(pest, chumsky, "both crates build one tree for {text}");
        pest
    }

    #[test]
    fn both_crates_build_one_tree() {
        assert_eq!(
            both("tag(db) | tag(slow) & !tag(db)"),
            Query::Or(vec![
                Query::Tag("db".into()),
                Query::And(vec![
                    Query::Tag("slow".into()),
                    Query::Not(Box::new(Query::Tag("db".into())))
                ])
            ]),
            "& binds tighter than |"
        );
        assert_eq!(
            both("stable(slow_store) | tests/a.py::test_b"),
            Query::Or(vec![
                Query::Call("stable".into(), Box::new(Query::Id("slow_store".into()))),
                Query::Id("tests/a.py::test_b".into())
            ]),
            "a call and an id"
        );
        assert_eq!(both("path(tests/api/**)"), Query::Path("tests/api/**".into()), "a path");
    }

    #[test]
    fn a_glob_matches_by_directory() {
        assert!(matches_glob(b"tests/api/**", b"tests/api/test_a.py"), "** crosses");
        assert!(matches_glob(b"tests/**/test_a.py", b"tests/test_a.py"), "**/ matches none");
        assert!(!matches_glob(b"tests/*.py", b"tests/api/test_a.py"), "* stays");
    }
}
