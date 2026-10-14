//! PROTOTYPE command: reads the arguments and sets the exit code. No `PyO3`.
use crate::dispatch;
use crate::reporting;
use crate::selection;

/// Runs purba on the arguments after the script's name, and returns the exit code.
#[must_use]
pub fn run(args: &[String]) -> i32 {
    let [path] = args else { return 3 };
    let Ok(source) = std::fs::read_to_string(path) else {
        return 3;
    };
    let tests = selection::select(path, &source);
    let Ok(outcomes) = dispatch::run(path, &source, &tests) else {
        return 3;
    };
    let names: Vec<String> = tests.into_iter().map(|test| test.name).collect();
    match reporting::report(path, &names, &outcomes) {
        (_, failed, _) if failed > 0 => 1,
        (passed, 0, 0) if passed > 0 => 0,
        _ => 2,
    }
}
