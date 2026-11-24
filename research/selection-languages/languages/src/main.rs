// One probe per language crate. Usage: small <lang> <query>
// It parses <query> with the crate, then selects from the probe suite, and prints the ids.
// On a parse error it prints the crate's own error text and exits 2.
use std::collections::BTreeMap;
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

fn fail(text: String) -> ! {
    println!("parse error:\n{text}");
    exit(2)
}

fn show(picked: Vec<&str>) {
    eprintln!("stage: evaluated");
    println!("selected {}: {}", picked.len(), picked.join(", "));
}

fn cfg_expr(q: &str) {
    let expr = cfg_expr::Expression::parse(q).unwrap_or_else(|e| fail(e.to_string()));
    eprintln!("stage: parsed");
    let mut picked = vec![];
    for t in SUITE {
        let hit = expr.eval(|p| match p {
            cfg_expr::expr::Predicate::Flag(f) => t.tags.contains(f),
            cfg_expr::expr::Predicate::KeyValue { key, val } => *key == "module" && *val == t.module,
            _ => false,
        });
        if hit {
            picked.push(t.id);
        }
    }
    show(picked);
}

fn cel(q: &str) {
    let program = cel::Program::compile(q).unwrap_or_else(|e| fail(e.to_string()));
    eprintln!("stage: parsed");
    let mut picked = vec![];
    for t in SUITE {
        let mut cx = cel::Context::default();
        let tags: Vec<String> = t.tags.iter().map(|s| s.to_string()).collect();
        cx.add_variable("tags", tags).unwrap();
        cx.add_variable("module", t.module.to_string()).unwrap();
        cx.add_variable("id", t.id.to_string()).unwrap();
        match program.execute(&cx) {
            Ok(cel::Value::Bool(true)) => picked.push(t.id),
            Ok(cel::Value::Bool(false)) => {}
            other => {
                println!("execute error: {other:?}");
                exit(3)
            }
        }
    }
    show(picked);
}

fn gherkin(q: &str) {
    use gherkin::tagexpr::TagOperation;
    let op: TagOperation = q.parse().unwrap_or_else(|e: peg::error::ParseError<_>| fail(e.to_string()));
    eprintln!("stage: parsed");
    if q.len() < 200 {
        println!("tree: {op:?}");
    }
    fn eval(op: &TagOperation, tags: &[String]) -> bool {
        match op {
            TagOperation::And(a, b) => eval(a, tags) && eval(b, tags),
            TagOperation::Or(a, b) => eval(a, tags) || eval(b, tags),
            TagOperation::Not(a) => !eval(a, tags),
            TagOperation::Tag(t) => tags.iter().any(|x| x == t),
        }
    }
    let mut picked = vec![];
    for t in SUITE {
        // Cucumber has tags only, so the module is one more tag here.
        let mut tags: Vec<String> = t.tags.iter().map(|s| s.to_string()).collect();
        tags.push(t.module.to_string());
        if eval(&op, &tags) {
            picked.push(t.id);
        }
    }
    show(picked);
}

fn k8s(q: &str) {
    use kube_core::SelectorExt;
    let parsed = unselector::Expressions::try_from(q).unwrap_or_else(|e| fail(e.to_string()));
    eprintln!("stage: parsed");
    println!("parsed: {parsed:?}");
    let selector: kube_core::Selector = parsed.into();
    let mut picked = vec![];
    for t in SUITE {
        let mut labels = BTreeMap::new();
        labels.insert("module".to_string(), t.module.to_string());
        for tag in t.tags {
            labels.insert(tag.to_string(), String::new());
        }
        if selector.matches(&labels) {
            picked.push(t.id);
        }
    }
    show(picked);
}

fn evalexpr(q: &str) {
    use evalexpr::{ContextWithMutableVariables, DefaultNumericTypes, HashMapContext, Value};
    let tree = evalexpr::build_operator_tree::<DefaultNumericTypes>(q).unwrap_or_else(|e| fail(e.to_string()));
    eprintln!("stage: parsed");
    let mut picked = vec![];
    for t in SUITE {
        let mut cx = HashMapContext::<DefaultNumericTypes>::new();
        for tag in ["slow", "db", "flaky"] {
            cx.set_value(tag.into(), Value::Boolean(t.tags.contains(&tag))).unwrap();
        }
        cx.set_value("module".into(), Value::String(t.module.into())).unwrap();
        match tree.eval_boolean_with_context(&cx) {
            Ok(true) => picked.push(t.id),
            Ok(false) => {}
            Err(e) => {
                println!("execute error: {e}");
                exit(3)
            }
        }
    }
    show(picked);
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let lang = args[1].as_str();
    let owned = if args[2] == "-" { std::io::read_to_string(std::io::stdin()).unwrap() } else { args[2].clone() };
    let q = owned.as_str();
    match lang {
        "cfg-expr" => cfg_expr(q),
        "cel" => cel(q),
        "gherkin" => gherkin(q),
        "k8s" => k8s(q),
        "evalexpr" => evalexpr(q),
        _ => panic!("unknown lang {lang}"),
    }
}
