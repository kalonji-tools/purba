// One grammar, written once in each parser crate. Usage: p-parsers <crate> <query>, or "-".
//
//   query := or
//   or    := and ("|" and)*        lowest rank, left to right
//   and   := unary ("&" unary)*    binds tighter than "|"
//   unary := "!" unary | atom
//   atom  := "tag(" name ")" | id | "(" or ")"
//   id    := [A-Za-z0-9_:/.\[\]-]+, an id or a prefix of one, such as store::migrate
//
// It prints the tree with explicit brackets, or the crate's own error. After a parse it
// forgets the tree, so a deep tree's recursive drop does not count against the parser.
use std::process::exit;

#[derive(Debug)]
pub enum Expr {
    Or(Box<Expr>, Box<Expr>),
    And(Box<Expr>, Box<Expr>),
    Not(Box<Expr>),
    Tag(String),
    Id(String),
}

fn tree(e: &Expr) -> String {
    match e {
        Expr::Or(a, b) => format!("or[{}, {}]", tree(a), tree(b)),
        Expr::And(a, b) => format!("and[{}, {}]", tree(a), tree(b)),
        Expr::Not(a) => format!("not[{}]", tree(a)),
        Expr::Tag(t) => format!("tag {t}"),
        Expr::Id(i) => format!("id {i}"),
    }
}

fn is_id(c: char) -> bool {
    c.is_ascii_alphanumeric() || "_:/.[]-".contains(c)
}

fn bx(e: Expr) -> Box<Expr> {
    Box::new(e)
}

mod with_chumsky {
    use super::{bx, is_id, Expr};
    use chumsky::prelude::*;

    fn parser<'a>() -> impl Parser<'a, &'a str, Expr, extra::Err<Rich<'a, char>>> {
        recursive(|or| {
            let ident = any().filter(|c: &char| is_id(*c)).repeated().at_least(1).to_slice();
            let tag = just("tag(").ignore_then(ident).then_ignore(just(')')).map(|s: &str| Expr::Tag(s.into()));
            let atom = tag
                .or(ident.map(|s: &str| Expr::Id(s.into())))
                .or(or.delimited_by(just('('), just(')')))
                .padded();
            let unary = just('!').padded().repeated().foldr(atom, |_, e| Expr::Not(bx(e)));
            let and = unary.clone().foldl(just('&').padded().ignore_then(unary).repeated(), |a, b| Expr::And(bx(a), bx(b)));
            and.clone().foldl(just('|').padded().ignore_then(and).repeated(), |a, b| Expr::Or(bx(a), bx(b)))
        })
    }

    pub fn parse(q: &str) -> Result<Expr, String> {
        parser().then_ignore(end()).parse(q).into_result().map_err(|errs| {
            errs.iter().map(|e| format!("at {:?}: {e}", e.span())).collect::<Vec<_>>().join("\n")
        })
    }
}

mod with_winnow {
    use super::{bx, is_id, Expr};
    use winnow::ascii::multispace0;
    use winnow::combinator::{alt, delimited, preceded, separated_foldl1};
    use winnow::error::{ContextError, ErrMode, StrContext, StrContextValue};
    use winnow::prelude::*;
    use winnow::token::take_while;

    fn ws<'a, O>(
        p: impl Parser<&'a str, O, ErrMode<ContextError>>,
    ) -> impl Parser<&'a str, O, ErrMode<ContextError>> {
        delimited(multispace0, p, multispace0)
    }

    fn atom(i: &mut &str) -> ModalResult<Expr> {
        ws(alt((
            delimited("tag(", take_while(1.., is_id), ")").map(|s: &str| Expr::Tag(s.into())),
            take_while(1.., is_id).map(|s: &str| Expr::Id(s.into())),
            delimited("(", or, ")"),
        )))
        .context(StrContext::Expected(StrContextValue::Description("a tag, an id or `(`")))
        .parse_next(i)
    }

    fn unary(i: &mut &str) -> ModalResult<Expr> {
        alt((preceded(ws("!"), unary).map(|e| Expr::Not(bx(e))), atom)).parse_next(i)
    }

    fn and(i: &mut &str) -> ModalResult<Expr> {
        separated_foldl1(unary, ws("&"), |a, _, b| Expr::And(bx(a), bx(b))).parse_next(i)
    }

    fn or(i: &mut &str) -> ModalResult<Expr> {
        separated_foldl1(and, ws("|"), |a, _, b| Expr::Or(bx(a), bx(b))).parse_next(i)
    }

    pub fn parse(q: &str) -> Result<Expr, String> {
        or.parse(q).map_err(|e| e.to_string())
    }
}

mod with_nom {
    use super::{bx, is_id, Expr};
    use nom::branch::alt;
    use nom::bytes::complete::{tag, take_while1};
    use nom::character::complete::{char, multispace0};
    use nom::combinator::{all_consuming, map};
    use nom::multi::many0;
    use nom::sequence::{delimited, preceded};
    use nom::{IResult, Parser};

    fn atom(i: &str) -> IResult<&str, Expr> {
        delimited(
            multispace0,
            alt((
                map(delimited(tag("tag("), take_while1(is_id), char(')')), |s: &str| Expr::Tag(s.into())),
                map(take_while1(is_id), |s: &str| Expr::Id(s.into())),
                delimited(char('('), or, char(')')),
            )),
            multispace0,
        )
        .parse(i)
    }

    fn unary(i: &str) -> IResult<&str, Expr> {
        alt((map(preceded(delimited(multispace0, char('!'), multispace0), unary), |e| Expr::Not(bx(e))), atom)).parse(i)
    }

    fn and(i: &str) -> IResult<&str, Expr> {
        let (i, first) = unary(i)?;
        let (i, rest) = many0(preceded(char('&'), unary)).parse(i)?;
        Ok((i, rest.into_iter().fold(first, |a, b| Expr::And(bx(a), bx(b)))))
    }

    fn or(i: &str) -> IResult<&str, Expr> {
        let (i, first) = and(i)?;
        let (i, rest) = many0(preceded(char('|'), and)).parse(i)?;
        Ok((i, rest.into_iter().fold(first, |a, b| Expr::Or(bx(a), bx(b)))))
    }

    pub fn parse(q: &str) -> Result<Expr, String> {
        all_consuming(or).parse(q).map(|(_, e)| e).map_err(|e| format!("{e:?}"))
    }
}

mod with_pest {
    use super::{bx, Expr};
    use pest::iterators::Pair;
    use pest::Parser;

    #[derive(pest_derive::Parser)]
    #[grammar = "grammar/query.pest"]
    struct QueryParser;

    fn build(p: Pair<Rule>) -> Expr {
        match p.as_rule() {
            Rule::or => {
                let mut inner = p.into_inner().map(build);
                let first = inner.next().unwrap();
                inner.fold(first, |a, b| Expr::Or(bx(a), bx(b)))
            }
            Rule::and => {
                let mut inner = p.into_inner().map(build);
                let first = inner.next().unwrap();
                inner.fold(first, |a, b| Expr::And(bx(a), bx(b)))
            }
            Rule::not => Expr::Not(bx(build(p.into_inner().next().unwrap()))),
            Rule::tag => Expr::Tag(p.into_inner().next().unwrap().as_str().into()),
            Rule::id => Expr::Id(p.as_str().into()),
            r => unreachable!("{r:?}"),
        }
    }

    pub fn parse(q: &str) -> Result<Expr, String> {
        let mut pairs = QueryParser::parse(Rule::query, q).map_err(|e| e.to_string())?;
        // pest returns a flat list of pairs; the tree below is this probe's, built by recursion.
        eprintln!("stage: pest parsed");
        Ok(build(pairs.next().unwrap().into_inner().next().unwrap()))
    }
}

mod with_lalrpop {
    use super::Expr;
    lalrpop_util::lalrpop_mod!(query, "/grammar/query.rs");

    pub fn parse(q: &str) -> Result<Expr, String> {
        query::QueryParser::new().parse(q).map_err(|e| format!("{e}\n  debug: {e:?}"))
    }
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let krate = args[1].as_str();
    let q = if args[2] == "-" { std::io::read_to_string(std::io::stdin()).unwrap() } else { args[2].clone() };
    let result = match krate {
        "chumsky" => with_chumsky::parse(&q),
        "winnow" => with_winnow::parse(&q),
        "nom" => with_nom::parse(&q),
        "pest" => with_pest::parse(&q),
        "lalrpop" => with_lalrpop::parse(&q),
        _ => panic!("unknown crate {krate}"),
    };
    match result {
        Ok(e) => {
            eprintln!("stage: parsed");
            if q.len() < 200 {
                println!("tree: {}", tree(&e));
            }
            std::mem::forget(e);
        }
        Err(text) => {
            println!("parse error:\n{text}");
            exit(2)
        }
    }
}
