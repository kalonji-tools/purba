use crate::assertion;

#[derive(Debug, PartialEq, Eq)]
pub enum Outcome {
    Passed,
    Failed(String),
    Unknown,
}

/// What one call of a test function did, as data.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Call {
    Returned { asserts_ran: usize },
    AssertionFailed { assert_index: usize },
    Raised(String),
}

/// What execution needs from Python. lib implements it with PyO3.
pub trait Interpreter {
    /// Compiles and executes the rewritten source as a module.
    ///
    /// # Errors
    /// When the source does not compile or its body raises.
    fn load(&mut self, path: &str, rewritten: &str) -> Result<(), String>;
    /// Calls one test function of the loaded module.
    fn call(&mut self, name: &str) -> Call;
}

/// Runs each named test of one module, and makes its outcome.
///
/// # Errors
/// When the module does not load, or a test raises something other than `AssertionError`.
pub fn run(
    interpreter: &mut dyn Interpreter,
    path: &str,
    source: &str,
    names: &[String],
) -> Result<Vec<Outcome>, String> {
    let (rewritten, asserts) = assertion::rewrite(source);
    interpreter.load(path, &rewritten)?;
    names
        .iter()
        .map(|name| match interpreter.call(name) {
            Call::Returned { asserts_ran: 0 } => Ok(Outcome::Unknown),
            Call::Returned { .. } => Ok(Outcome::Passed),
            Call::AssertionFailed { assert_index } => {
                Ok(Outcome::Failed(asserts[assert_index].clone()))
            }
            Call::Raised(e) => Err(e),
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    struct Scripted(Vec<Call>);

    impl Interpreter for Scripted {
        fn load(&mut self, _: &str, _: &str) -> Result<(), String> {
            Ok(())
        }
        fn call(&mut self, _: &str) -> Call {
            self.0.remove(0)
        }
    }

    #[test]
    fn three_outcomes() {
        let src = "def test_one():\n    assert 1 == 1\n\
            def test_two():\n    assert 1 == 2\n\
            def test_three():\n    pass\n";
        let names = crate::selection::select(src);
        let mut python = Scripted(vec![
            Call::Returned { asserts_ran: 1 },
            Call::AssertionFailed { assert_index: 1 },
            Call::Returned { asserts_ran: 0 },
        ]);
        let got = run(&mut python, "test_a.py", src, &names).unwrap();
        let want = [
            Outcome::Passed,
            Outcome::Failed("assert 1 == 2".into()),
            Outcome::Unknown,
        ];
        assert_eq!(got, want, "one passes, one fails, one checks nothing");
    }
}
