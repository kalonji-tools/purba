//! PROTOTYPE dispatch: one worker, in-process. No `PyO3`.
use crate::execution;
use crate::selection;

/// Hands each selected test to the one worker, and returns its outcome.
///
/// # Errors
/// When execution cannot run the module.
pub fn run(
    path: &str,
    source: &str,
    tests: &[selection::Test],
) -> Result<Vec<execution::Outcome>, String> {
    let names: Vec<String> = tests.iter().map(|test| test.name.clone()).collect();
    execution::run(path, source, &names)
}
