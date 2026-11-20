//! The setup that each test of execution shares.

use std::path::PathBuf;

use purba_execution::{Module, Rewrite};

/// The module at `name`, whose tests are the tests of `source`.
pub fn module(name: &str, source: &str) -> Module {
    Module::new(
        PathBuf::from(name),
        Rewrite::of_source(source, &tests_in(source)).expect("the source parses"),
    )
}

/// The name of each `def test_…` at the top of `source`, as selection names
/// the tests of a module.
fn tests_in(source: &str) -> Vec<&str> {
    source
        .lines()
        .filter_map(|line| line.strip_prefix("def test_").map(|_| &line[4..]))
        .filter_map(|line| line.split_once('(').map(|(name, _)| name))
        .collect()
}
