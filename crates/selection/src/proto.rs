//! PROTOTYPE, never merge: the name-near notice of `name-and-check` at
//! a5a7b530, without its `asserts` and `name-prefix` lines.

use purba_parsing::Facts;
use ruff_python_ast::Stmt;
use ruff_text_size::Ranged;

/// Whether `word` is within one edit of `test`, case ignored: one letter
/// changed, added, dropped, or two neighbours swapped. `test` itself counts.
#[must_use]
pub fn near_test(word: &str) -> bool {
    let word: Vec<char> = word.to_lowercase().chars().collect();
    let target: Vec<char> = "test".chars().collect();
    if word == target {
        return true;
    }
    match word.len() {
        4 => {
            let differ: Vec<usize> = (0..4).filter(|&i| word[i] != target[i]).collect();
            differ.len() == 1
                || (differ.len() == 2
                    && differ[1] == differ[0] + 1
                    && word[differ[0]] == target[differ[1]]
                    && word[differ[1]] == target[differ[0]])
        }
        3 => (0..4).any(|skip| {
            let shorter: Vec<char> = target
                .iter()
                .enumerate()
                .filter(|&(i, _)| i != skip)
                .map(|(_, &c)| c)
                .collect();
            shorter == word
        }),
        5 => (0..5).any(|skip| {
            let shorter: Vec<char> = word
                .iter()
                .enumerate()
                .filter(|&(i, _)| i != skip)
                .map(|(_, &c)| c)
                .collect();
            shorter == target
        }),
        _ => false,
    }
}

/// Each name-near line of the module-level `def`s of `facts`: `(line, words)`,
/// in source order.
#[must_use]
pub fn near_misses(facts: &Facts<'_>) -> Vec<(usize, String)> {
    let module = facts.module();
    let mut found = Vec::new();
    for statement in &module.syntax().body {
        let Stmt::FunctionDef(function) = statement else {
            continue;
        };
        let name = function.name.as_str();
        if name.starts_with("test") || name.starts_with('_') {
            continue;
        }
        let first = name.split('_').next().unwrap_or(name);
        if near_test(first) {
            let line = module.position(function.start()).line();
            found.push((
                line,
                format!("{name} is not a test, and its name is near the prefix test"),
            ));
        }
    }
    found
}
