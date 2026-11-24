// Probe of tantivy's query parser, a Lucene-like syntax. Usage: p-tantivy <query>, or "-".
// It indexes the probe suite in RAM, with the fields id, module and tags, and the default
// field tags. It prints tantivy's own grammar tree (tantivy-query-grammar), the parsed query,
// and the ids that the search selects.
use tantivy::collector::DocSetCollector;
use tantivy::query::QueryParser;
use tantivy::schema::{Schema, Value, STORED, STRING};
use tantivy::{doc, Index, TantivyDocument};
use std::process::exit;

fn main() {
    let arg = std::env::args().nth(1).expect("a query");
    let q = if arg == "-" { std::io::read_to_string(std::io::stdin()).unwrap() } else { arg };

    match tantivy_query_grammar::parse_query(&q) {
        Ok(ast) => {
            eprintln!("stage: grammar parsed");
            if q.len() < 200 {
                println!("grammar tree: {ast:?}");
            }
        }
        Err(e) => println!("grammar error: {e:?}"),
    }

    let mut b = Schema::builder();
    let id = b.add_text_field("id", STRING | STORED);
    let module = b.add_text_field("module", STRING);
    let tags = b.add_text_field("tags", STRING);
    let schema = b.build();
    let index = Index::create_in_ram(schema);
    let mut w = index.writer(15_000_000).unwrap();
    let suite: &[(&str, &str, &[&str])] = &[
        ("math::add", "math", &[]),
        ("math::add_slow", "math", &["slow"]),
        ("store::query", "store", &["db"]),
        ("store::migrate", "store", &["slow", "db"]),
        ("store::load", "store", &["slow", "flaky"]),
        ("store::save", "store", &["slow"]),
    ];
    for (i, m, ts) in suite {
        let mut d = doc!(id => *i, module => *m);
        for t in *ts {
            d.add_text(tags, t);
        }
        w.add_document(d).unwrap();
    }
    w.commit().unwrap();
    let reader = index.reader().unwrap();
    let searcher = reader.searcher();
    let parser = QueryParser::for_index(&index, vec![tags]);
    let query = match parser.parse_query(&q) {
        Ok(query) => query,
        Err(e) => {
            println!("parse error: {e}\n  debug: {e:?}");
            exit(2)
        }
    };
    eprintln!("stage: parsed");
    if q.len() < 200 {
        println!("query: {query:?}");
    }
    let hits = searcher.search(&query, &DocSetCollector).unwrap();
    let mut picked: Vec<String> = hits
        .into_iter()
        .map(|addr| {
            let d: TantivyDocument = searcher.doc(addr).unwrap();
            d.get_first(id).and_then(|v| v.as_str()).unwrap().to_string()
        })
        .collect();
    picked.sort();
    eprintln!("stage: evaluated");
    println!("selected {}: {}", picked.len(), picked.join(", "));
}
