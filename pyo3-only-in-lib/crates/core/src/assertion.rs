/// A line-based stand-in for ruff: wraps each assert's test in a recording call.
#[must_use]
pub fn rewrite(source: &str) -> (String, Vec<String>) {
    let mut asserts = vec![];
    let lines: Vec<String> = source
        .lines()
        .map(|line| {
            let body = line.trim_start();
            body.strip_prefix("assert ").map_or_else(
                || line.to_owned(),
                |expr| {
                    let indent = &line[..line.len() - body.len()];
                    asserts.push(body.to_owned());
                    let (test, message) = expr
                        .split_once(", ")
                        .map_or((expr, String::new()), |(t, m)| (t, format!(", {m}")));
                    format!(
                        "{indent}assert _purba_ran({}, {test}){message}",
                        asserts.len() - 1
                    )
                },
            )
        })
        .collect();
    (lines.join("\n") + "\n", asserts)
}
