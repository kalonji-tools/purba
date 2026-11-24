// Probe of Jujutsu's revset parser, from the crate jj-dsl, which jj-lib uses.
// Usage: p-jj <query>, or "-" to read stdin. It parses with revset_parser::parse_program,
// expands the aliases below, prints the tree with explicit brackets, and selects from the
// probe suite. The functions tag() and module() are this probe's, not jj's: jj names commits.
use jj_dsl::dsl_util::expand_aliases;
use jj_dsl::revset_parser::{
    parse_program, BinaryOp, ExpressionKind, ExpressionNode, RevsetAliasesMap, UnaryOp,
};
use std::process::exit;

struct Test {
    id: &'static str,
    module: &'static str,
    tags: &'static [&'static str],
}

const SUITE: &[Test] = &[
    Test { id: "math::add", module: "math", tags: &[] },
    Test { id: "math::add_slow", module: "math", tags: &["slow"] },
    Test { id: "store::query", module: "store", tags: &["db"] },
    Test { id: "store::migrate", module: "store", tags: &["slow", "db"] },
    Test { id: "store::load", module: "store", tags: &["slow", "flaky"] },
    Test { id: "store::save", module: "store", tags: &["slow"] },
];

fn arg_text(node: &ExpressionNode) -> String {
    match &node.kind {
        ExpressionKind::Identifier(s) => s.to_string(),
        ExpressionKind::String(s) => s.clone(),
        other => panic!("this probe takes a name as an argument, not {other:?}"),
    }
}

fn tree(node: &ExpressionNode) -> String {
    match &node.kind {
        ExpressionKind::Identifier(s) => format!("id {s}"),
        ExpressionKind::String(s) => format!("string {s:?}"),
        ExpressionKind::Unary(UnaryOp::Negate, a) => format!("not[{}]", tree(a)),
        ExpressionKind::Unary(op, a) => format!("{op:?}[{}]", tree(a)),
        ExpressionKind::Binary(BinaryOp::Intersection, a, b) => format!("and[{}, {}]", tree(a), tree(b)),
        ExpressionKind::Binary(BinaryOp::Difference, a, b) => format!("minus[{}, {}]", tree(a), tree(b)),
        ExpressionKind::Binary(op, a, b) => format!("{op:?}[{}, {}]", tree(a), tree(b)),
        ExpressionKind::UnionAll(all) => format!("or[{}]", all.iter().map(tree).collect::<Vec<_>>().join(", ")),
        ExpressionKind::FunctionCall(f) => {
            format!("{}({})", f.name, f.args.iter().map(arg_text).collect::<Vec<_>>().join(", "))
        }
        ExpressionKind::AliasExpanded(_, inner) => tree(inner),
        other => format!("{other:?}"),
    }
}

fn eval(node: &ExpressionNode, t: &Test) -> bool {
    match &node.kind {
        ExpressionKind::Unary(UnaryOp::Negate, a) => !eval(a, t),
        ExpressionKind::Binary(BinaryOp::Intersection, a, b) => eval(a, t) && eval(b, t),
        ExpressionKind::Binary(BinaryOp::Difference, a, b) => eval(a, t) && !eval(b, t),
        ExpressionKind::UnionAll(all) => all.iter().any(|n| eval(n, t)),
        ExpressionKind::FunctionCall(f) if f.name == "tag" => t.tags.contains(&arg_text(&f.args[0]).as_str()),
        ExpressionKind::FunctionCall(f) if f.name == "module" => t.module == arg_text(&f.args[0]),
        ExpressionKind::AliasExpanded(_, inner) => eval(inner, t),
        other => panic!("this probe cannot select by {other:?}"),
    }
}

fn main() {
    let arg = std::env::args().nth(1).expect("a query");
    let q = if arg == "-" { std::io::read_to_string(std::io::stdin()).unwrap() } else { arg };
    // As a user's config would hold them under [revset-aliases].
    let mut aliases = RevsetAliasesMap::new();
    aliases.insert("slow_store", "module(store) & tag(slow)", None).unwrap();
    aliases.insert("stable(x)", "x ~ tag(flaky)", None).unwrap();
    let node = match parse_program(&q).and_then(|n| expand_aliases(n, &aliases)) {
        Ok(n) => n,
        Err(e) => {
            println!("parse error:\n{e}");
            exit(2)
        }
    };
    eprintln!("stage: parsed");
    if q.len() < 200 {
        println!("tree: {}", tree(&node));
        let picked: Vec<&str> = SUITE.iter().filter(|t| eval(&node, t)).map(|t| t.id).collect();
        println!("selected {}: {}", picked.len(), picked.join(", "));
    }
    eprintln!("stage: evaluated");
}
