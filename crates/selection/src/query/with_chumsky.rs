//! The query's grammar on chumsky, which builds the [`Query`] as it parses.

use chumsky::prelude::*;

use super::{Query, one_or};

fn is_literal(c: char) -> bool {
    c.is_ascii_alphanumeric() || "_-.:/[]".contains(c)
}

fn is_glob(c: char) -> bool {
    c.is_ascii_alphanumeric() || "_-./*?[]".contains(c)
}

fn parser<'a>() -> impl Parser<'a, &'a str, Query, extra::Err<Rich<'a, char>>> {
    recursive(|or| {
        let literal = any().filter(|c: &char| is_literal(*c)).repeated().at_least(1).to_slice();
        let glob = any().filter(|c: &char| is_glob(*c)).repeated().at_least(1).to_slice();
        let word = any()
            .filter(|c: &char| c.is_ascii_alphabetic() || *c == '_')
            .then(any().filter(|c: &char| c.is_ascii_alphanumeric() || *c == '_').repeated())
            .to_slice();
        let predicate = |name: &'static str| just(name).then(just('(')).ignore_then(literal).then_ignore(just(')'));
        let tag = predicate("tag").map(|s: &str| Query::Tag(s.to_owned()));
        let name = predicate("name").map(|s: &str| Query::Name(s.to_owned()));
        let path = just("path(")
            .ignore_then(glob)
            .then_ignore(just(')'))
            .map(|s: &str| Query::Path(s.to_owned()));
        let call = word
            .then_ignore(just('('))
            .then(or.clone())
            .then_ignore(just(')'))
            .map(|(w, q): (&str, Query)| Query::Call(w.to_owned(), Box::new(q)));
        let id = literal.map(|s: &str| Query::Id(s.to_owned()));
        let group = or.delimited_by(just('('), just(')'));
        let atom = choice((tag, name, path, call, id, group)).padded();
        let unary = just('!').padded().repeated().foldr(atom, |_, e| Query::Not(Box::new(e)));
        let and = unary
            .separated_by(just('&').padded())
            .at_least(1)
            .collect::<Vec<_>>()
            .map(|terms| one_or(terms, Query::And));
        and.separated_by(just('|').padded())
            .at_least(1)
            .collect::<Vec<_>>()
            .map(|terms| one_or(terms, Query::Or))
    })
}

pub fn parse(text: &str) -> Result<Query, String> {
    parser().then_ignore(end()).parse(text).into_result().map_err(|errors| {
        errors
            .iter()
            .map(|error| {
                let column = text[..error.span().start].chars().count() + 1;
                format!("at column {column}:\n  {text}\n  {:>column$}\n  {error}", "^")
            })
            .collect::<Vec<_>>()
            .join("\n")
    })
}
