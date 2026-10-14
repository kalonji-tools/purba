//! PROTOTYPE reporting: one line for each test, and a count. No `PyO3`.
use crate::execution;

/// Prints one line for each test and a count, and returns (passed, failed, unknown).
#[must_use]
pub fn report(
    path: &str,
    names: &[String],
    outcomes: &[execution::Outcome],
) -> (usize, usize, usize) {
    let (mut passed, mut failed, mut unknown) = (0, 0, 0);
    for (name, outcome) in names.iter().zip(outcomes) {
        match outcome {
            execution::Outcome::Passed => {
                passed += 1;
                println!("{path}::{name} passed");
            }
            execution::Outcome::Failed(why) => {
                failed += 1;
                println!("{path}::{name} failed: {why}");
            }
            execution::Outcome::Unknown => {
                unknown += 1;
                println!("{path}::{name} unknown: no assert ran");
            }
        }
    }
    println!(
        "{} tests: {passed} passed, {failed} failed, {unknown} unknown",
        passed + failed + unknown
    );
    (passed, failed, unknown)
}
