//! PROTOTYPE for #1229, never merge: the near-miss check of stage two under
//! `name-and-check`.

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

/// Whether `body` holds an `assert`, in its blocks but not in a nested `def`
/// or `class`.
fn asserts(body: &[Stmt]) -> bool {
    body.iter().any(|statement| {
        matches!(statement, Stmt::Assert(_))
            || purba_parsing::blocks(statement).into_iter().any(asserts)
    })
}

/// Each line of the near-miss check of the module-level `def`s of `facts`:
/// `(line, kind, words)`, in source order.
#[must_use]
pub fn near_misses(facts: &Facts<'_>) -> Vec<(usize, &'static str, String)> {
    let module = facts.module();
    let mut found = Vec::new();
    for statement in &module.syntax().body {
        let Stmt::FunctionDef(function) = statement else {
            continue;
        };
        let name = function.name.as_str();
        let line = module.position(function.start()).line();
        if purba_parsing::is_test_name(name) {
            if name != "test" && !name.starts_with("test_") {
                found.push((
                    line,
                    "name-prefix",
                    format!("{name} is a test, but its name does not start with test_"),
                ));
            }
            continue;
        }
        if name.starts_with('_') {
            continue;
        }
        let first = name.split('_').next().unwrap_or(name);
        if near_test(first) {
            found.push((
                line,
                "name-near",
                format!("{name} is not a test, and its name is near the prefix test"),
            ));
        } else if asserts(&function.body) {
            found.push((
                line,
                "asserts",
                format!("{name} is not a test, and it asserts"),
            ));
        }
    }
    found
}

#[cfg(test)]
mod tests {
    use super::near_test;

    #[test]
    fn near_test_reads_one_edit() {
        for word in [
            "test", "tset", "Test", "TEST", "tst", "tests", "est", "test", "tesst",
        ] {
            assert!(near_test(word), "{word} is within one edit of test");
        }
        for word in ["testing", "conftest", "helpers", "tt", "best_", "toast"] {
            assert!(!near_test(word), "{word} is not within one edit of test");
        }
    }
}
