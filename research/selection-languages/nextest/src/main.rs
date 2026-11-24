// Probe of nextest-filtering's parser. Usage: p-nextest <query>, or "-" to read stdin.
// It parses with ParsedExpr::parse, which needs no cargo package graph, prints the tree with
// explicit brackets, and selects from the probe suite by test name. nextest has no tags, so
// each name carries its tags, as in the predecessor research.
use nextest_filtering::ParsedExpr;
use std::process::exit;

const SUITE: &[&str] = &[
    "math::add",
    "math::add_slow",
    "store::query_db",
    "store::migrate_slow_db",
    "store::load_slow_flaky",
    "store::save_slow",
];

fn tree(e: &ParsedExpr) -> String {
    match e {
        ParsedExpr::Not(_, a) => format!("not[{}]", tree(a)),
        ParsedExpr::Union(_, a, b) => format!("or[{}, {}]", tree(a), tree(b)),
        ParsedExpr::Intersection(_, a, b) => format!("and[{}, {}]", tree(a), tree(b)),
        ParsedExpr::Difference(_, a, b) => format!("minus[{}, {}]", tree(a), tree(b)),
        ParsedExpr::Parens(a) => tree(a),
        ParsedExpr::Set(leaf) => leaf.to_string(),
    }
}

fn eval(e: &ParsedExpr, name: &str) -> bool {
    match e {
        ParsedExpr::Not(_, a) => !eval(a, name),
        ParsedExpr::Union(_, a, b) => eval(a, name) || eval(b, name),
        ParsedExpr::Intersection(_, a, b) => eval(a, name) && eval(b, name),
        ParsedExpr::Difference(_, a, b) => eval(a, name) && !eval(b, name),
        ParsedExpr::Parens(a) => eval(a, name),
        ParsedExpr::Set(leaf) => {
            let text = leaf.to_string();
            let inner = text
                .strip_prefix("test(")
                .and_then(|s| s.strip_suffix(')'))
                .unwrap_or_else(|| panic!("this probe reads test() only, not {text}"));
            match inner.strip_prefix('=') {
                Some(exact) => name == exact,
                None => name.contains(inner.trim_start_matches('~')),
            }
        }
    }
}

// The parse that cargo-nextest itself runs: Filterset::parse, against a package graph.
// The graph is this probe's own cargo project, read by guppy.
fn strict(q: String) {
    let graph = guppy::MetadataCommand::new()
        .manifest_path(concat!(env!("CARGO_MANIFEST_DIR"), "/Cargo.toml"))
        .build_graph()
        .expect("cargo metadata of the probe");
    let cx = nextest_filtering::ParseContext::new(&graph);
    let groups = nextest_filtering::KnownGroups::Known { custom_groups: Default::default() };
    match nextest_filtering::Filterset::parse(q, &cx, nextest_filtering::FiltersetKind::Test, &groups) {
        Ok(f) => println!("strict: parsed; tree: {}", tree(&f.parsed)),
        Err(errs) => {
            let handler = miette::GraphicalReportHandler::new_themed(miette::GraphicalTheme::unicode_nocolor());
            println!("strict: parse error:");
            for e in errs.errors {
                let report = miette::Report::new(e).with_source_code(errs.input.clone());
                let mut out = String::new();
                handler.render_report(&mut out, report.as_ref()).unwrap();
                print!("{out}");
            }
            exit(2)
        }
    }
}

fn main() {
    let mut args: Vec<String> = std::env::args().skip(1).collect();
    let is_strict = args.first().is_some_and(|a| a == "--strict");
    if is_strict {
        args.remove(0);
    }
    let arg = args.into_iter().next().expect("a query");
    let q = if arg == "-" { std::io::read_to_string(std::io::stdin()).unwrap() } else { arg };
    if is_strict {
        return strict(q);
    }
    match ParsedExpr::parse(&q) {
        Ok(expr) => {
            eprintln!("stage: parsed");
            if q.len() < 200 {
                println!("tree: {}", tree(&expr));
                let picked: Vec<&str> = SUITE.iter().copied().filter(|n| eval(&expr, n)).collect();
                println!("selected {}: {}", picked.len(), picked.join(", "));
            }
            eprintln!("stage: evaluated");
        }
        Err(errors) => {
            let handler = miette::GraphicalReportHandler::new_themed(miette::GraphicalTheme::unicode_nocolor());
            println!("parse error:");
            for e in errors {
                let report = miette::Report::new(e).with_source_code(q.clone());
                let mut out = String::new();
                handler.render_report(&mut out, report.as_ref()).unwrap();
                print!("{out}");
            }
            exit(2)
        }
    }
}
