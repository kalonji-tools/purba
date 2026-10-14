use crate::execution::{self, Interpreter, Outcome};

/// One worker, in-process.
///
/// # Errors
/// When execution cannot run the module.
pub fn run(
    interpreter: &mut dyn Interpreter,
    path: &str,
    source: &str,
    names: &[String],
) -> Result<Vec<Outcome>, String> {
    execution::run(interpreter, path, source, names)
}
