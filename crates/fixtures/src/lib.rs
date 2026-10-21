//! Fixtures: builds the inputs that a test names. The walking skeleton builds none.

/// The inputs of one test. It holds none yet.
pub struct Inputs {}

/// The inputs that the test `function` names.
#[must_use]
pub const fn inputs(_function: &str) -> Inputs {
    Inputs {}
}
