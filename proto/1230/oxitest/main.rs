// A harness around oxitest's own query files, copied unchanged from a commit by
// run.sh. It builds one entry per test the way oxitest's run path does
// (src/pipeline/transitions/mod.rs, item_to_query_entry), from entries.tsv,
// then prints the ids that an expression selects. Prototype; never merged.
mod query {
    pub mod ast;
    pub mod compile;
    pub mod eval;
    pub mod resource;
}

use query::resource::{QueryEntry, ResourceKind};
use std::collections::HashMap;

fn main() {
    let mut args = std::env::args().skip(1);
    let entries = args.next().expect("entries.tsv");
    let expr_str = std::io::read_to_string(std::io::stdin()).expect("an expression on stdin");
    let expr_str = expr_str.strip_suffix('\n').unwrap_or(&expr_str);
    let tests: Vec<QueryEntry> = std::fs::read_to_string(entries)
        .expect("entries")
        .lines()
        .map(|line| {
            let cols: Vec<&str> = line.split('\t').collect();
            let mut fields = HashMap::new();
            fields.insert("name".to_string(), cols[0].to_string());
            fields.insert("source".to_string(), cols[1].to_string());
            fields.insert("mark".to_string(), cols.get(2).copied().unwrap_or("").to_string());
            fields.insert("async".to_string(), "false".to_string());
            QueryEntry { fields }
        })
        .collect();
    let parsed = query::compile::lex(expr_str).and_then(query::compile::parse);
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
