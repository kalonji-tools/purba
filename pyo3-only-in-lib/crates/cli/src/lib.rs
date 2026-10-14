//! PROTOTYPE cli: command. No PyO3.
use purba_core::{dispatch, reporting, selection};

/// Runs purba on the arguments after the script's name, and returns the exit code.
#[must_use]
pub fn run(args: &[String], interpreter: &mut dyn purba_core::execution::Interpreter) -> i32 {
    let [path] = args else { return 3 };
    let Ok(source) = std::fs::read_to_string(path) else {
        return 3;
    };
    let names = selection::select(&source);
    let Ok(outcomes) = dispatch::run(interpreter, path, &source, &names) else {
        return 3;
    };
    match reporting::report(path, &names, &outcomes) {
        (_, f, _) if f > 0 => 1,
        (p, 0, 0) if p > 0 => 0,
        _ => 2,
    }
}
