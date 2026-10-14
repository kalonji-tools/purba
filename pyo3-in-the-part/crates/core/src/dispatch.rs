use crate::execution::{self, Outcome};

/// One worker, in-process.
///
/// # Errors
/// When execution cannot run the module.
pub fn run(path: &str, source: &str, names: &[String]) -> Result<Vec<Outcome>, String> {
    execution::run(path, source, names)
}
