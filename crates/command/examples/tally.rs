//! PROTOTYPE for #1229, never merge. For each suite root named on the command
//! line as `<name>=<dir>`, give each `.py` file under it to main's real
//! selection, one at a time, as main takes one path today. It skips each folder
//! whose name starts with `.`. It imports nothing and runs nothing.
//!
//! One line per row, tab-separated:
//!
//! - `F <suite> <path> <ok|parse-error> <module fault or ->` for each file given
//! - `T <suite> <path> <id> <selected|refused> <reasons or ->` for each test
//! - `R <suite> <path> <kind> <words>` for each line of a check (none on main)

use std::path::{Path, PathBuf};

fn walk(root: &Path, here: &Path, found: &mut Vec<PathBuf>) {
    let Ok(entries) = std::fs::read_dir(here) else {
        return;
    };
    let mut entries: Vec<PathBuf> = entries.filter_map(|e| e.ok().map(|e| e.path())).collect();
    entries.sort();
    for path in entries {
        let name = path.file_name().unwrap_or_default().to_string_lossy();
        if name.starts_with('.') {
            continue;
        }
        if path.is_dir() && !path.is_symlink() {
            walk(root, &path, found);
        } else if path.extension().is_some_and(|e| e == "py") && path.is_file() {
            found.push(path.strip_prefix(root).unwrap_or(&path).to_path_buf());
        }
    }
}

fn one_line(text: &str) -> String {
    text.replace(['\t', '\n'], " ")
}

fn main() {
    for arg in std::env::args().skip(1) {
        let (suite, dir) = arg.split_once('=').expect("<name>=<dir>");
        let root = Path::new(dir);
        let mut files = Vec::new();
        walk(root, root, &mut files);
        for rel in files {
            let shown = rel.display().to_string();
            let full = root.join(&rel);
            let Ok(module) = purba_parsing::parse(&full) else {
                println!("F\t{suite}\t{shown}\tparse-error\t-");
                continue;
            };
            let facts = purba_parsing::Facts::of(&module);
            let selected = purba_selection::select(&rel, &facts);
            let fault = selected.fault().map_or_else(|| "-".to_owned(), one_line);
            println!("F\t{suite}\t{shown}\tok\t{fault}");
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
