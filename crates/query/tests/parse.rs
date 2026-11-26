use purba_query::{Query, parse};

fn id(text: &str) -> Query {
    Query::Id(text.to_owned())
}

/// The words of the refusal of `text`.
fn refused(text: &str) -> String {
    match parse(text) {
        Ok(tree) => panic!("the query {text:?} parses, to {tree:?}"),
        Err(refusal) => refusal.to_string(),
    }
}

#[test]
fn an_id_of_a_test_is_one_term() {
    assert_eq!(
        parse("tests/test_a.py::test_add"),
        Ok(id("tests/test_a.py::test_add")),
        "the id of a test is one term, with its path and its name"
    );
    assert_eq!(parse("tests"), Ok(id("tests")), "a bare word is an id");
}

#[test]
fn name_and_path_hold_their_literal() {
    assert_eq!(
        parse("name(add)"),
        Ok(Query::Name("add".to_owned())),
        "name() holds the part of a name"
    );
    assert_eq!(
        parse("path(tests/**/db_*.py)"),
        Ok(Query::Path("tests/**/db_*.py".to_owned())),
        "path() holds a glob"
    );
}

#[test]
fn not_binds_tightest_then_and_then_or() {
    assert_eq!(
        parse("a | b & !c"),
        Ok(Query::Or(vec![
            id("a"),
            Query::And(vec![id("b"), Query::Not(Box::new(id("c")))]),
        ])),
        "& binds tighter than |, and ! tighter than &"
    );
}

#[test]
fn not_binds_tighter_than_and() {
    assert_eq!(
        parse("!a & b"),
        Ok(Query::And(vec![Query::Not(Box::new(id("a"))), id("b")])),
        "! takes only the term after it"
    );
}

#[test]
fn a_chain_of_and_is_one_node() {
    assert_eq!(
        parse("a & b & c"),
        Ok(Query::And(vec![id("a"), id("b"), id("c")])),
        "three terms of & are one node"
    );
}

#[test]
fn parentheses_group_terms() {
    assert_eq!(
        parse("(a | b) & c"),
        Ok(Query::And(vec![Query::Or(vec![id("a"), id("b")]), id("c")])),
        "the group holds the |"
    );
}

#[test]
fn a_chain_of_one_operator_is_one_node() {
    assert_eq!(
        parse("a | b | c"),
        Ok(Query::Or(vec![id("a"), id("b"), id("c")])),
        "three terms of | are one node, so a long list is width and not depth"
    );
}

#[test]
fn spaces_may_stand_around_an_operator() {
    assert_eq!(
        parse("  a  &  ! b  "),
        Ok(Query::And(vec![id("a"), Query::Not(Box::new(id("b")))])),
        "spaces around a term and an operator change nothing"
    );
}

#[test]
fn a_query_nests_at_most_255_levels() {
    let deep = |levels: usize| format!("{}name(add){}", "(".repeat(levels), ")".repeat(levels));
    assert_eq!(
        parse(&deep(255)),
        Ok(Query::Name("add".to_owned())),
        "255 levels of ( parse"
    );
    let words = "the query nests deeper than 255 levels at column 256";
    assert_eq!(refused(&deep(256)), words, "256 levels of ( are refused");
    assert_eq!(
        refused(&deep(10_000)),
        words,
        "10000 levels of ( are refused at the same column, before the parser reads them"
    );
    assert_eq!(
        refused(&format!("{}name(add)", "!".repeat(256))),
        words,
        "each ! opens a level too"
    );
    assert!(
        parse(&format!("{}name(add)", "!".repeat(255))).is_ok(),
        "255 ! parse"
    );
}

#[test]
fn the_depth_error_names_the_column_of_the_opener_that_goes_too_deep() {
    assert_eq!(
        refused(&format!("name(a) & {}", "(".repeat(256))),
        "the query nests deeper than 255 levels at column 266",
        "ten characters stand before the 256th (, at column 266"
    );
}

#[test]
fn the_scan_of_depth_comes_before_the_parser() {
    assert_eq!(
        refused(&format!("tag(x) {}", "(".repeat(256))),
        "the query nests deeper than 255 levels at column 263",
        "tag( is a parse error, but the scan refuses the depth first"
    );
}

#[test]
fn a_level_closes_at_its_parenthesis_and_at_the_end_of_the_term_of_a_not() {
    let groups = vec!["(name(a))"; 256].join(" | ");
    assert!(
        parse(&groups).is_ok(),
        "256 groups side by side nest one level"
    );
    let nots = vec!["!name(a)"; 256].join(" | ");
    assert!(
        parse(&nots).is_ok(),
        "256 terms under ! side by side nest one level"
    );
}

#[test]
fn an_empty_term_and_a_letter_outside_the_set_are_parse_errors() {
    assert_eq!(
        refused("name()"),
        "column 6: expected a name",
        "name() holds a part of a name"
    );
    assert_eq!(
        refused("tests/*"),
        "column 7: expected &, | or the end of the query",
        "* stands only in path()"
    );
}

#[test]
fn a_call_that_is_no_term_names_the_terms_that_take_a_parenthesis() {
    assert_eq!(
        refused("tag(slow)"),
        "column 1: tag( is no term; a query takes name( and path(",
        "tag() lands with purba.tag, so the grammar has no tag( yet"
    );
    assert_eq!(
        refused("name(add) | tag(slow)"),
        "column 13: tag( is no term; a query takes name( and path(",
        "the column is that of the word before the ("
    );
}

#[test]
fn a_missing_term_names_what_may_stand_there() {
    let expected = "expected an id, name(, path(, ! or (";
    assert_eq!(
        refused("name(add) &"),
        format!("column 12: {expected}"),
        "a term must follow &"
    );
    assert_eq!(
        refused(""),
        format!("column 1: {expected}"),
        "an empty query"
    );
    assert_eq!(
        refused("(name(add)"),
        "column 11: expected &, | or )",
        "an unclosed ( names what may close it"
    );
    assert_eq!(
        refused("name(add) name(sub)"),
        "column 11: expected &, | or the end of the query",
        "two terms need an operator between them"
    );
}
