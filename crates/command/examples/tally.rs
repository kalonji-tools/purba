//! PROTOTYPE for #1229, never merge. For each suite root named on the command
//! line as `<name>=<dir>`, run stage one of the strategy that
//! `PURBA_PROTO_MARKERS` names over the root, then give each file it picks to
//! the real selection. `baseline` picks every `.py` file, each given alone, as
//! main takes one path. The walk skips each folder whose name starts with `.`.
//! It imports nothing and runs nothing.
//!
//! One line per row, tab-separated:
//!
//! - `F <suite> <path> <ok|parse-error> <module fault or ->` for each file picked
//! - `T <suite> <path> <id> <selected|refused> <reasons or ->` for each test
//! - `R <suite> <path> <kind> <words>` for each line of a check

use std::path::Path;

fn one_line(text: &str) -> String {
    text.replace(['\t', '\n'], " ")
}

fn main() {
    let near = purba_parsing::proto_strategy() == "name-and-check";
    for arg in std::env::args().skip(1) {
        let (suite, dir) = arg.split_once('=').expect("<name>=<dir>");
        let root = Path::new(dir);
        let relative = |path: &Path| {
            path.strip_prefix(root)
                .unwrap_or(path)
                .display()
                .to_string()
        };
        let picked = purba_command::proto::pick(root);
        for (path, kind, words) in &picked.lines {
            println!(
                "R\t{suite}\t{}\t{kind}\t{}",
                relative(path),
                one_line(words)
            );
        }
        for full in &picked.files {
            let shown = relative(full);
            let rel = Path::new(&shown);
            let Ok(module) = purba_parsing::parse(full) else {
                println!("F\t{suite}\t{shown}\tparse-error\t-");
                continue;
            };
            let facts = purba_parsing::Facts::of(&module);
            let selected = purba_selection::select(rel, &facts);
            let fault = selected.fault().map_or_else(|| "-".to_owned(), one_line);
            println!("F\t{suite}\t{shown}\tok\t{fault}");
            if near {
                for (line, kind, words) in purba_selection::proto::near_misses(&facts) {
                    println!(
                        "R\t{suite}\t{shown}\t{kind}\tline {line}: {}",
                        one_line(&words)
                    );
                }
            }
            for test in selected.tests() {
                println!("T\t{suite}\t{shown}\t{test}\tselected\t-");
            }
            for refused in selected.refused() {
                println!(
                    "T\t{suite}\t{shown}\t{}\trefused\t{}",
                    refused.test(),
                    one_line(&refused.to_string())
                );
            }
        }
    }
}
