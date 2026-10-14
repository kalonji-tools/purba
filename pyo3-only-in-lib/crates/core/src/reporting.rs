use crate::execution::Outcome;

/// Prints one line for each test and a count, and returns (passed, failed, unknown).
#[must_use]
pub fn report(path: &str, names: &[String], outcomes: &[Outcome]) -> (usize, usize, usize) {
    let (mut p, mut f, mut u) = (0, 0, 0);
    for (name, outcome) in names.iter().zip(outcomes) {
        match outcome {
            Outcome::Passed => {
                p += 1;
                println!("{path}::{name} passed");
            }
            Outcome::Failed(why) => {
                f += 1;
                println!("{path}::{name} failed: {why}");
            }
            Outcome::Unknown => {
                u += 1;
                println!("{path}::{name} unknown: no assert ran");
            }
        }
    }
    println!("{} tests: {p} passed, {f} failed, {u} unknown", p + f + u);
    (p, f, u)
}
