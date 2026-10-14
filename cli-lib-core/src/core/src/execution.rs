//! PROTOTYPE execution: calls each test and makes its outcome. `PyO3` lives here.
use crate::assertion;
use crate::fixtures;
use pyo3::exceptions::PyAssertionError;
use pyo3::prelude::*;
use pyo3::types::PyTuple;

/// What a test did.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Outcome {
    Passed,
    Failed(String),
    Unknown,
}

/// Runs each named test of one module, and makes its outcome.
///
/// # Errors
/// When the module does not load, or a test raises something other than `AssertionError`.
pub fn run(path: &str, source: &str, names: &[String]) -> Result<Vec<Outcome>, String> {
    Python::attach(|py| {
        let (module, asserts) = assertion::load(py, path, source)?;
        names
            .iter()
            .map(|name| {
                let (before, _) = assertion::ran();
                let inputs = PyTuple::new(py, fixtures::build(name))?;
                match module.getattr(name.as_str())?.call1(inputs) {
                    Err(error) if error.is_instance_of::<PyAssertionError>(py) => {
                        Ok(Outcome::Failed(asserts[assertion::ran().1].clone()))
                    }
                    Err(error) => Err(error),
                    Ok(_) if assertion::ran().0 > before => Ok(Outcome::Passed),
                    Ok(_) => Ok(Outcome::Unknown),
                }
            })
            .collect::<PyResult<Vec<_>>>()
    })
    .map_err(|error| error.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn three_outcomes() {
        Python::initialize();
        let source = "def test_one():\n    assert 1 == 1\n\
            def test_two():\n    assert 1 == 2\n\
            def test_three():\n    pass\n";
        let names = ["test_one", "test_two", "test_three"].map(String::from);
        let got = run("test_a.py", source, &names).expect("the module loads");
        let want = [
            Outcome::Passed,
            Outcome::Failed("assert 1 == 2".into()),
            Outcome::Unknown,
        ];
        assert_eq!(got, want, "one passes, one fails, one checks nothing");
    }
}
