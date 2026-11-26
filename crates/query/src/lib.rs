//! Turns the text of a query into a tree of terms, and expands each alias.
//!
//! `!` binds tightest, then `&`, then `|`, and parentheses group terms. A term
//! is an id, `name(…)`, `path(…)` or the call of an alias. A scan refuses a
//! text that nests deeper than 255 levels before the grammar reads it, so the
//! grammar's recursion stays shallow.

mod alias;

use std::fmt;

use winnow::ascii::multispace0;
use winnow::combinator::{alt, cut_err, delimited, preceded, repeat};
use winnow::error::{ContextError, ErrMode, ModalResult, StrContext, StrContextValue};
use winnow::prelude::*;
use winnow::token::{literal, take_while};

pub use alias::{Aliases, Nearest};

/// The most levels a query nests. Each `(` that opens a group or a call, each
/// `!`, and each alias that expands opens a level.
const DEPTH: usize = 255;

/// A query, as a tree of terms.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Query {
    /// A folder, a file or the id of a test, as the tester typed it.
    Id(String),
    /// Each test whose function's name holds the text.
    Name(String),
    /// Each test of a file whose path from the root the glob matches.
    Path(String),
    /// Each test that the term does not pick.
    Not(Box<Self>),
    /// Each test that every term picks.
    And(Vec<Self>),
    /// Each test that any term picks.
    Or(Vec<Self>),
    /// The call of the alias `name`, with its argument. `column` is the column
    /// of the name in the text.
    Call {
        /// The name of the alias.
        name: String,
        /// The query that the alias takes in place of its parameter.
        argument: Box<Self>,
        /// The column of the name in the text.
        column: usize,
    },
}

/// Why the text of a query is no query, or why its aliases do not expand.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Refused {
    column: usize,
    why: Why,
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum Why {
    /// The text nests deeper than [`DEPTH`] levels.
    Deep,
    /// The expansion of the alias, which the tester typed, nests deeper than
    /// [`DEPTH`] levels.
    Through(String),
    /// A word before `(` names no term, and the nearest alias, as the
    /// project's file declares it, if any.
    Call(String, Option<String>),
    /// A call hands an argument to the alias, which takes none.
    NoArgument(String),
    /// The bare word names the alias, which takes an argument, as the
    /// project's file declares it.
    OneArgument(String, String),
    /// Something else stands where the words name what may stand.
    Expected(&'static str),
}

impl fmt::Display for Refused {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let column = self.column;
        match &self.why {
            Why::Deep => write!(
                formatter,
                "the query nests deeper than {DEPTH} levels at column {column}"
            ),
            Why::Through(name) => write!(
                formatter,
                "the query nests deeper than {DEPTH} levels through the alias {name}"
            ),
            Why::Call(word, None) => write!(
                formatter,
                "column {column}: {word}( is no term; a query takes name( and path("
            ),
            Why::Call(word, Some(nearest)) => write!(
                formatter,
                "column {column}: {word}( is no term; the nearest alias is {nearest}"
            ),
            Why::NoArgument(name) => write!(
                formatter,
                "column {column}: {name} takes no argument; write {name}"
            ),
            Why::OneArgument(name, form) => {
                write!(formatter, "{name} takes one argument; write {form}")
            }
            Why::Expected(what) => write!(formatter, "column {column}: expected {what}"),
        }
    }
}

/// The tree of the query that `text` holds.
///
/// # Errors
///
/// The text nests deeper than 255 levels, or does not parse.
pub fn parse(text: &str) -> Result<Query, Refused> {
    scan(text)?;
    let mut input = text;
    let outcome = delimited(multispace0, or, multispace0).parse_next(&mut input);
    let at = text.len() - input.len();
    let refused = |why| Refused {
        column: text[..at].chars().count() + 1,
        why,
    };
    match outcome {
        Ok(mut tree) if input.is_empty() => {
            count_columns(&mut tree, text);
            Ok(tree)
        }
        Ok(_) => Err(refused(Why::Expected(AFTER_A_TERM))),
        Err(ErrMode::Backtrack(error) | ErrMode::Cut(error)) => Err(refused(why(&error, input))),
        Err(ErrMode::Incomplete(_)) => Err(refused(Why::Expected(A_TERM))),
    }
}

/// What may stand where a term must.
const A_TERM: &str = "an id, name(, path(, ! or (";

/// What may follow a term at the top of the query.
const AFTER_A_TERM: &str = "&, | or the end of the query";

/// What may follow a term in a group.
const IN_A_GROUP: &str = "&, | or )";

/// The label of a word before `(` that names no term.
const CALL: &str = "call";

/// Why the grammar stopped at `rest`, from the context of `error`.
fn why(error: &ContextError, rest: &str) -> Why {
    for context in error.context() {
        match context {
            StrContext::Label(CALL) => {
                return Why::Call(take_word(rest).to_owned(), None);
            }
            StrContext::Expected(StrContextValue::Description(what)) => {
                return Why::Expected(what);
            }
            _ => {}
        }
    }
    Why::Expected(A_TERM)
}

/// Whether `c` may stand in an id or in the text of `name(…)`.
fn is_word(c: char) -> bool {
    c.is_alphanumeric() || matches!(c, '_' | '-' | '.' | '/' | ':' | '[' | ']')
}

/// Whether `c` may stand in the glob of `path(…)`.
fn is_glob(c: char) -> bool {
    is_word(c) || matches!(c, '*' | '?')
}

/// The word at the start of `text`.
fn take_word(text: &str) -> &str {
    let end = text.find(|c| !is_word(c)).unwrap_or(text.len());
    &text[..end]
}

const fn expected(what: &'static str) -> StrContext {
    StrContext::Expected(StrContextValue::Description(what))
}

/// The terms joined by `|`, as one node.
fn or(input: &mut &str) -> ModalResult<Query> {
    chain(input, '|', and, Query::Or)
}

/// The terms joined by `&`, as one node.
fn and(input: &mut &str) -> ModalResult<Query> {
    chain(input, '&', unary, Query::And)
}

/// One or more of `term`, joined by `operator`, as one node of `node` when
/// there are two or more.
fn chain(
    input: &mut &str,
    operator: char,
    mut term: impl FnMut(&mut &str) -> ModalResult<Query>,
    node: fn(Vec<Query>) -> Query,
) -> ModalResult<Query> {
    let first = term(input)?;
    let rest: Vec<Query> = repeat(
        0..,
        preceded(
            (multispace0, operator, multispace0),
            cut_err(|input: &mut &str| term(input)),
        ),
    )
    .parse_next(input)?;
    if rest.is_empty() {
        return Ok(first);
    }
    let mut terms = vec![first];
    terms.extend(rest);
    Ok(node(terms))
}

/// A term, or a term under `!`.
fn unary(input: &mut &str) -> ModalResult<Query> {
    alt((
        preceded(('!', multispace0), cut_err(unary)).map(|term| Query::Not(Box::new(term))),
        group,
        literal_term("name(", is_word, "a name", Query::Name),
        literal_term("path(", is_glob, "a glob", Query::Path),
        id,
    ))
    .context(expected(A_TERM))
    .parse_next(input)
}

/// A query in parentheses.
fn group(input: &mut &str) -> ModalResult<Query> {
    preceded(
        ('(', multispace0),
        cut_err((or, multispace0, cut_err(')').context(expected(IN_A_GROUP)))),
    )
    .map(|(tree, _, _)| tree)
    .parse_next(input)
}

/// `opening`, then a text whose each character `allowed` takes, then `)`.
fn literal_term(
    opening: &'static str,
    allowed: fn(char) -> bool,
    what: &'static str,
    node: fn(String) -> Query,
) -> impl FnMut(&mut &str) -> ModalResult<Query> {
    move |input: &mut &str| {
        preceded(
            literal(opening),
            cut_err((
                take_while(1.., allowed).context(expected(what)),
                cut_err(')').context(expected(")")),
            )),
        )
        .map(|(text, _): (&str, char)| node(text.to_owned()))
        .parse_next(input)
    }
}

/// A folder, a file or the id of a test, or the call of an alias: a word
/// followed by `(`, with a query in the parentheses. `tag(` names no term until
/// `purba.tag` exists. The column of a call holds, for now, how many bytes of
/// the text stand from its name to the end; [`count_columns`] counts it.
fn id(input: &mut &str) -> ModalResult<Query> {
    let start = *input;
    let word = take_while(1.., is_word).parse_next(input)?;
    if !input.starts_with('(') {
        return Ok(Query::Id(word.to_owned()));
    }
    if word == "tag" {
        *input = start;
        let mut error = ContextError::new();
        error.push(StrContext::Label(CALL));
        return Err(ErrMode::Cut(error));
    }
    group
        .map(|argument| Query::Call {
            name: word.to_owned(),
            argument: Box::new(argument),
            column: start.len(),
        })
        .parse_next(input)
}

/// Turns the column of each call in `tree`, the bytes from its name to the end
/// of `text`, into the column of its name.
fn count_columns(tree: &mut Query, text: &str) {
    match tree {
        Query::Id(_) | Query::Name(_) | Query::Path(_) => {}
        Query::Not(term) => count_columns(term, text),
        Query::And(terms) | Query::Or(terms) => {
            for term in terms {
                count_columns(term, text);
            }
        }
        Query::Call {
            argument, column, ..
        } => {
            *column = text[..text.len() - *column].chars().count() + 1;
            count_columns(argument, text);
        }
    }
}

/// Refuses a text that nests deeper than [`DEPTH`] levels.
///
/// Each `(` of a group opens a level until its `)`. Each `!` opens a level
/// until the term after it ends. The text of `name(…)` and `path(…)` opens
/// none.
fn scan(text: &str) -> Result<(), Refused> {
    // The `!` that wait for their term, in each open group.
    let mut waiting: Vec<usize> = vec![0];
    let mut level = 0usize;
    let mut word = String::new();
    let mut chars = text.chars().enumerate().peekable();
    let term_ends = |waiting: &mut Vec<usize>, level: &mut usize| {
        if let Some(bangs) = waiting.last_mut() {
            *level -= *bangs;
            *bangs = 0;
        }
    };
    while let Some((at, c)) = chars.next() {
        if is_glob(c) {
            word.push(c);
            continue;
        }
        let before = std::mem::take(&mut word);
        if !before.is_empty() && c != '(' {
            term_ends(&mut waiting, &mut level);
        }
        match c {
            '!' => {
                if let Some(bangs) = waiting.last_mut() {
                    *bangs += 1;
                }
                level += 1;
            }
            '(' if before == "name" || before == "path" => {
                while chars.next_if(|&(_, c)| c != ')').is_some() {}
                chars.next();
                term_ends(&mut waiting, &mut level);
            }
            '(' => {
                waiting.push(0);
                level += 1;
            }
            ')' if waiting.len() > 1 => {
                let bangs = waiting.pop().unwrap_or_default();
                level -= bangs + 1;
                term_ends(&mut waiting, &mut level);
            }
            _ => {}
        }
        if level > DEPTH {
            return Err(Refused {
                column: at + 1,
                why: Why::Deep,
            });
        }
    }
    Ok(())
}
