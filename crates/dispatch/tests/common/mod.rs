//! The setup that each test of dispatch shares.

use std::path::PathBuf;

use purba_dispatch::{Module, Test};
use purba_execution::Rewrite;

/// A test by the name of its function.
#[derive(Clone)]
pub struct Named(pub &'static str);

impl Test for Named {
    fn function(&self) -> &str {
        self.0
    }
}

/// The module at `name`, which hands on each test of `tests` by its name.
pub fn module(name: &str, source: &str, tests: &[&'static str]) -> Module<Named> {
    module_of(
        name,
        source,
        tests.iter().map(|&test| Named(test)).collect(),
    )
}

/// The module at `name`, whose tests are the tests of `source`, and which
/// hands on `tests`.
pub fn module_of<T: Test>(name: &str, source: &str, tests: Vec<T>) -> Module<T> {
    Module::new(
        PathBuf::from(name),
        Rewrite::of_source(source, &tests_in(source)).expect("the source parses"),
        tests,
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
