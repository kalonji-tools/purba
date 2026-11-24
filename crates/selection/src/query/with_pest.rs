//! The query's grammar on pest: `query.pest`, and the walk from pest's pairs to a [`Query`].

use pest::Parser;
use pest::iterators::Pair;

use super::{Query, one_or};

#[derive(pest_derive::Parser)]
#[grammar = "query/query.pest"]
struct QueryParser;

fn build(pair: Pair<'_, Rule>) -> Query {
    let text = |pair: Pair<'_, Rule>| pair.into_inner().next().expect("a literal").as_str().to_owned();
    match pair.as_rule() {
        Rule::or => one_or(pair.into_inner().map(build).collect(), Query::Or),
        Rule::and => one_or(pair.into_inner().map(build).collect(), Query::And),
        Rule::not => Query::Not(Box::new(build(pair.into_inner().next().expect("an operand")))),
        Rule::tag => Query::Tag(text(pair)),
        Rule::name => Query::Name(text(pair)),
        Rule::path => Query::Path(text(pair)),
        Rule::call => {
            let mut inner = pair.into_inner();
            let word = inner.next().expect("a word").as_str().to_owned();
            let query = inner.next().expect("an argument").into_inner().next().expect("a query");
            Query::Call(word, Box::new(build(query)))
        }
        Rule::id => Query::Id(pair.as_str().to_owned()),
        rule => unreachable!("{rule:?}"),
    }
}

pub fn parse(text: &str) -> Result<Query, String> {
    let mut pairs = QueryParser::parse(Rule::query, text).map_err(|error| error.to_string())?;
    let or = pairs.next().expect("a query").into_inner().next().expect("an or");
    Ok(build(or))
}
