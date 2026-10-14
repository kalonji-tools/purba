//! PROTOTYPE parsing: a line-based stand-in for ruff's AST.

/// One line of a module: its indent and its text.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Line {
    pub indent: String,
    pub text: String,
}

/// Splits Python source into lines, the stand-in for an AST.
#[must_use]
pub fn parse(source: &str) -> Vec<Line> {
    source
        .lines()
        .map(|line| {
            let text = line.trim_start();
            Line {
                indent: line[..line.len() - text.len()].to_owned(),
                text: text.to_owned(),
            }
        })
        .collect()
}
