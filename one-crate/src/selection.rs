//! PROTOTYPE selection: names the top-level tests of one module. No `PyO3`.
use crate::parsing;

/// A selected test, by the identity selection gives it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Test {
    pub path: String,
    pub name: String,
}

/// The top-level test functions of a module, in source order.
#[must_use]
pub fn select(path: &str, source: &str) -> Vec<Test> {
    parsing::parse(source)
        .into_iter()
        .filter(|line| line.indent.is_empty())
        .filter_map(|line| {
            let rest = line.text.strip_prefix("def ")?;
            rest.split('(').next().map(str::to_owned)
        })
        .filter(|name| name.starts_with("test_"))
        .map(|name| Test {
            path: path.to_owned(),
            name,
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn names_top_level_tests_only() {
        let source =
            "def test_one():\n    pass\ndef helper():\n    def test_inner():\n        pass\n";
        let names: Vec<String> = select("test_a.py", source)
            .into_iter()
            .map(|test| test.name)
            .collect();
        assert_eq!(
            names,
            ["test_one"],
            "a helper and a nested def are not tests"
        );
    }
}
