/// The top-level test functions of a module, in source order.
///
/// ```
/// let names = purba_core::selection::select("def test_one():\n    pass\n");
/// assert_eq!(names, ["test_one"]);
/// ```
#[must_use]
pub fn select(source: &str) -> Vec<String> {
    source
        .lines()
        .filter_map(|l| l.strip_prefix("def "))
        .filter_map(|l| l.split('(').next())
        .filter(|n| n.starts_with("test_"))
        .map(str::to_owned)
        .collect()
}
