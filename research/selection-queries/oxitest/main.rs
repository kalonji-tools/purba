// A harness around oxitest's own query files, copied verbatim from a commit by
// run.sh. It builds one entry per test the way oxitest's run path does
// (src/pipeline/transitions/mod.rs, item_to_query_entry), then prints the ids
// that an expression selects.
mod query {
    pub mod ast;
    pub mod compile;
    pub mod eval;
    pub mod resource;
}

use std::collections::HashMap;
use query::resource::{QueryEntry, ResourceKind};

fn entry(node_id: &str, module: &str, marks: &[&str]) -> QueryEntry {
    let mut fields = HashMap::new();
    fields.insert("name".to_string(), node_id.to_string());
    fields.insert("source".to_string(), module.to_string());
    fields.insert("mark".to_string(), marks.join(","));
    fields.insert("async".to_string(), "false".to_string());
    QueryEntry { fields }
}

fn main() {
    let expr_str = std::env::args().nth(1).expect("an expression");
    let tests = [
        entry("test_math.py::test_add", "test_math.py", &[]),
        entry("test_math.py::test_add_slow", "test_math.py", &["slow"]),
        entry("test_math.py::test_parse[a]", "test_math.py", &[]),
        entry("test_math.py::test_parse[b]", "test_math.py", &[]),
        entry("test_store.py::test_query_db", "test_store.py", &["db"]),
        entry("test_store.py::test_migrate_db", "test_store.py", &["slow", "db"]),
    ];
    let parsed = query::compile::lex(&expr_str).and_then(query::compile::parse);
    let expr = match parsed {
        Ok(e) => e,
        Err(e) => {
            println!("error: {e}");
            std::process::exit(2);
        }
    };
    if let Err(e) = query::eval::validate_predicates(&expr, &ResourceKind::Tests) {
        println!("error: {e}");
        std::process::exit(2);
    }
    let mut n = 0;
    for t in &tests {
        if query::eval::eval(&expr, t) {
            println!("{}", t.get("name").unwrap_or_default());
            n += 1;
        }
    }
    println!("{n} selected");
}
