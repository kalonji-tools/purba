use purba_query::{Aliases, Query, parse};

fn id(text: &str) -> Query {
    Query::Id(text.to_owned())
}

fn path(glob: &str) -> Query {
    Query::Path(glob.to_owned())
}

fn not(term: Query) -> Query {
    Query::Not(Box::new(term))
}

/// The first of `names` that starts with the letter that `word` starts with:
/// a stand-in for the nearest by edits that reads the word, and that the order
/// of `names` decides only between names of one initial.
fn initial<'a>(word: &str, names: &[&'a str]) -> Option<&'a str> {
    let letter = word.chars().next()?;
    names.iter().copied().find(|name| name.starts_with(letter))
}

/// The aliases that `declared` declares, each as a key and its query.
fn aliases(declared: &[(&str, &str)]) -> Aliases {
    Aliases::new(declared.iter().copied(), initial)
        .unwrap_or_else(|words| panic!("the aliases {declared:?} are refused: {words:?}"))
}

/// The words of each refusal of the aliases that `declared` declares.
fn refused(declared: &[(&str, &str)]) -> Vec<String> {
    match Aliases::new(declared.iter().copied(), initial) {
        Ok(_) => panic!("the aliases {declared:?} are taken"),
        Err(words) => words,
    }
}

/// The tree of `text` once `aliases` expand it, or the words of the refusal.
fn expanded(aliases: &Aliases, text: &str) -> Result<Query, String> {
    let tree = parse(text).unwrap_or_else(|refusal| panic!("{text:?} does not parse: {refusal}"));
    aliases.expand(&tree).map_err(|refusal| refusal.to_string())
}

/// A chain of `length` aliases, `c000` to the last, each naming the next by a
/// bare word, and the last holding `name(x)`.
fn chain(length: usize) -> Vec<(String, String)> {
    (0..length)
        .map(|at| {
            let body = if at + 1 == length {
                "name(x)".to_owned()
            } else {
                format!("c{:03}", at + 1)
            };
            (format!("c{at:03}"), body)
        })
        .collect()
}

fn borrowed(owned: &[(String, String)]) -> Vec<(&str, &str)> {
    owned
        .iter()
        .map(|(key, body)| (key.as_str(), body.as_str()))
        .collect()
}

#[test]
fn a_word_before_a_parenthesis_is_a_call_with_its_column() {
    assert_eq!(
        parse("stable(tests)"),
        Ok(Query::Call {
            name: "stable".to_owned(),
            argument: Box::new(id("tests")),
            column: 1,
        }),
        "a word before ( calls the alias of that name, with one argument"
    );
    assert_eq!(
        parse("name(add) | stable(a | b)"),
        Ok(Query::Or(vec![
            Query::Name("add".to_owned()),
            Query::Call {
                name: "stable".to_owned(),
                argument: Box::new(Query::Or(vec![id("a"), id("b")])),
                column: 13,
            },
        ])),
        "the argument is a query, and the column is that of the word"
    );
}

#[test]
fn a_call_holds_an_argument() {
    assert_eq!(
        parse("stable()").map_err(|refusal| refusal.to_string()),
        Err("column 8: expected an id, name(, path(, ! or (".to_owned()),
        "a call with no argument names no query"
    );
    assert_eq!(
        parse("stable(a, b)").map_err(|refusal| refusal.to_string()),
        Err("column 9: expected &, | or )".to_owned()),
        "a call holds one query, so a , is no part of it"
    );
}

#[test]
fn a_bare_word_that_names_an_alias_takes_its_query() {
    let aliases = aliases(&[("fast", "!path(tests/slow/**)")]);
    assert_eq!(
        expanded(&aliases, "fast"),
        Ok(not(path("tests/slow/**"))),
        "the alias stands in place of its name"
    );
    assert_eq!(
        expanded(&aliases, "tests & fast"),
        Ok(Query::And(vec![id("tests"), not(path("tests/slow/**"))])),
        "a bare word that names no alias stays an id"
    );
}

#[test]
fn with_no_alias_a_tree_with_no_call_stays_as_it_is() {
    let tree = parse("tests | !name(add)").expect("the query parses");
    assert_eq!(
        Aliases::default().expand(&tree),
        Ok(tree.clone()),
        "no alias changes no term"
    );
}

#[test]
fn a_call_puts_its_argument_in_place_of_the_parameter() {
    let aliases = aliases(&[("stable(x)", "x & !path(tests/slow/**)")]);
    assert_eq!(
        expanded(&aliases, "stable(tests/test_a.py)"),
        Ok(Query::And(vec![
            id("tests/test_a.py"),
            not(path("tests/slow/**")),
        ])),
        "x takes the argument"
    );
}

#[test]
fn an_alias_names_another_alias() {
    let aliases = aliases(&[("fast", "!slow"), ("slow", "path(tests/slow/**)")]);
    assert_eq!(
        expanded(&aliases, "fast"),
        Ok(not(path("tests/slow/**"))),
        "slow expands inside fast"
    );
}

#[test]
fn the_parameter_hides_an_alias_of_its_name() {
    let aliases = aliases(&[("only(fast)", "fast"), ("fast", "name(f)")]);
    assert_eq!(
        expanded(&aliases, "only(name(g))"),
        Ok(Query::Name("g".to_owned())),
        "inside only, fast is the argument, not the alias"
    );
}

#[test]
fn an_argument_reads_the_parameter_of_the_alias_that_calls() {
    let aliases = aliases(&[("outer(x)", "inner(x & a)"), ("inner(y)", "y | b")]);
    assert_eq!(
        expanded(&aliases, "outer(c)"),
        Ok(Query::Or(
            vec![Query::And(vec![id("c"), id("a")]), id("b"),]
        )),
        "the x of outer is c inside the argument that outer hands inner"
    );
}

#[test]
fn a_body_reads_only_the_parameter_of_its_own_alias() {
    let aliases = aliases(&[("outer(x)", "inner(a)"), ("inner(y)", "y | x")]);
    assert_eq!(
        expanded(&aliases, "outer(c)"),
        Ok(Query::Or(vec![id("a"), id("x")])),
        "the x in the body of inner is an id, not the parameter of outer that calls it"
    );
}

#[test]
fn an_argument_that_names_an_alias_expands() {
    let aliases = aliases(&[("fast", "name(f)"), ("stable(x)", "x")]);
    assert_eq!(
        expanded(&aliases, "stable(fast)"),
        Ok(Query::Name("f".to_owned())),
        "fast, typed as the argument, takes its query"
    );
}

#[test]
fn a_call_that_names_no_alias_names_the_nearest_alias() {
    assert_eq!(
        expanded(&Aliases::default(), "name(add) | foo(x)"),
        Err("column 13: foo( is no term; a query takes name( and path(".to_owned()),
        "with no alias, the words stay those of a call that is no term"
    );
    let aliases = aliases(&[("stable(x)", "x")]);
    assert_eq!(
        expanded(&aliases, "stale(tests)"),
        Err("column 1: stale( is no term; the nearest alias is stable(x)".to_owned()),
        "the words name the nearest alias as the project's file declares it"
    );
}

#[test]
fn a_call_gives_an_alias_as_many_arguments_as_it_takes() {
    let aliases = aliases(&[("fast", "name(f)"), ("stable(x)", "x")]);
    assert_eq!(
        expanded(&aliases, "fast(tests)"),
        Err("column 1: fast takes no argument; write fast".to_owned()),
        "fast declares no parameter"
    );
    assert_eq!(
        expanded(&aliases, "stable"),
        Err("stable takes one argument; write stable(x)".to_owned()),
        "a bare word reads as the alias first, and stable declares a parameter"
    );
}

#[test]
fn an_alias_that_names_itself_is_refused() {
    assert_eq!(
        refused(&[("loop", "loop")]),
        vec!["loop under [tool.purba.queries] names itself".to_owned()],
        "directly"
    );
    assert_eq!(
        refused(&[("a", "b"), ("b", "!a")]),
        vec![
            "a under [tool.purba.queries] names itself through b".to_owned(),
            "b under [tool.purba.queries] names itself through a".to_owned(),
        ],
        "through another alias, one line for each alias of the loop, sorted by name"
    );
    assert_eq!(
        refused(&[("a", "b"), ("b", "c"), ("c", "a"), ("d", "a")]),
        vec![
            "a under [tool.purba.queries] names itself through b and c".to_owned(),
            "b under [tool.purba.queries] names itself through c and a".to_owned(),
            "c under [tool.purba.queries] names itself through a and b".to_owned(),
        ],
        "d only calls the loop, so the lines name the aliases of the loop alone"
    );
    assert_eq!(
        refused(&[("s(x)", "s(x)")]),
        vec!["s(x) under [tool.purba.queries] names itself".to_owned()],
        "a call of itself"
    );
}

#[test]
fn an_alias_that_nests_deeper_than_255_levels_alone_is_refused() {
    assert!(
        Aliases::new(borrowed(&chain(255)), initial).is_ok(),
        "the body of the last of 255 calls sits at level 255"
    );
    assert_eq!(
        Aliases::new(borrowed(&chain(256)), initial).map(|_| ()),
        Err(vec![
            "c000 under [tool.purba.queries] nests deeper than 255 levels".to_owned()
        ]),
        "each call opens a level, so the last body of 256 calls sits at level 256"
    );
}

#[test]
fn a_query_that_nests_deeper_than_255_levels_through_an_alias_is_refused() {
    let aliases = Aliases::new(borrowed(&chain(255)), initial).expect("255 levels are taken");
    assert!(
        expanded(&aliases, "c000").is_ok(),
        "the alias alone nests 255 levels"
    );
    assert_eq!(
        expanded(&aliases, "!c000"),
        Err("the query nests deeper than 255 levels through the alias c000".to_owned()),
        "the ! opens one more level"
    );
}

/// `levels` calls of `w`, each the argument of the one around it, around
/// `name(a)`.
fn calls(levels: usize) -> String {
    format!("{}name(a){}", "w(".repeat(levels), ")".repeat(levels))
}

#[test]
fn a_call_that_the_tester_types_opens_one_level() {
    let plain = aliases(&[("w(x)", "x")]);
    assert_eq!(
        expanded(&plain, &calls(255)),
        Ok(Query::Name("a".to_owned())),
        "255 typed calls, each one level, nest 255 levels"
    );
    let not = aliases(&[("w(x)", "!x")]);
    assert_eq!(
        expanded(&not, &calls(127)).map(|_| ()),
        Ok(()),
        "127 calls, each with its !, nest 254 levels"
    );
    assert_eq!(
        expanded(&not, &calls(128)),
        Err("the query nests deeper than 255 levels through the alias w".to_owned()),
        "128 calls, each with its !, nest 256 levels"
    );
}

#[test]
fn a_key_that_declares_no_alias_is_refused() {
    let words = |key: &str| {
        format!(
            "{key} under [tool.purba.queries] is no name of an alias; \
             a name holds letters, digits, _ and -, as fast or stable(x)"
        )
    };
    assert_eq!(
        refused(&[("fast x", "name(f)"), ("a/b", "name(f)"), ("s(x, y)", "x")]),
        vec![words("a/b"), words("fast x"), words("s(x, y)")],
        "a space, a / and two parameters each break the form, sorted by key"
    );
    assert_eq!(
        refused(&[("a::b", "name(f)"), ("a.b", "name(f)"), ("s()", "name(f)")]),
        vec![words("a.b"), words("a::b"), words("s()")],
        "a :: and a . belong to an id, and a parameter has a name"
    );
    let term = |key: &str| {
        format!(
            "{key} under [tool.purba.queries] is a term of a query; give the alias another name"
        )
    };
    assert_eq!(
        refused(&[("name", "name(f)"), ("path(x)", "x"), ("tag", "name(f)")]),
        vec![term("name"), term("path(x)"), term("tag")],
        "name, path and tag are terms"
    );
    assert_eq!(
        refused(&[("fast", "name(f)"), ("fast(x)", "x")]),
        vec!["fast and fast(x) under [tool.purba.queries] share a name; keep one".to_owned()],
        "one name has one declaration"
    );
}

#[test]
fn a_body_that_is_no_query_is_refused_in_the_words_of_the_grammar() {
    assert_eq!(
        refused(&[("fast", "name(")]),
        vec!["fast under [tool.purba.queries] is no query: column 6: expected a name".to_owned()],
        "the body does not parse"
    );
    assert_eq!(
        refused(&[("flaky", "tag(flaky)")]),
        vec![
            "flaky under [tool.purba.queries] is no query: \
             column 1: tag( is no term; a query takes name( and path("
                .to_owned()
        ],
        "tag( waits for purba.tag"
    );
    assert_eq!(
        refused(&[("fast", "slwo(x)"), ("slow(x)", "x")]),
        vec![
            "fast under [tool.purba.queries]: column 1: slwo( is no term; \
             the nearest alias is slow(x)"
                .to_owned()
        ],
        "a call in a body that names no alias is refused when purba reads it, and the \
         nearest reads the word of the call, not the key of the alias that holds it"
    );
    assert_eq!(
        refused(&[("fast", "slow"), ("slow(x)", "x")]),
        vec!["fast under [tool.purba.queries]: slow takes one argument; write slow(x)".to_owned()],
        "a call in a body with the wrong count is refused when purba reads it"
    );
}
