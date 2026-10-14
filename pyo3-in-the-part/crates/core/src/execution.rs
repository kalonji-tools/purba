use crate::assertion;
use pyo3::exceptions::PyAssertionError;
use pyo3::prelude::*;

#[derive(Debug, PartialEq, Eq)]
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
                match module.getattr(name.as_str())?.call0() {
                    Err(e) if e.is_instance_of::<PyAssertionError>(py) => {
                        Ok(Outcome::Failed(asserts[assertion::ran().1].clone()))
                    }
                    Err(e) => Err(e),
                    Ok(_) if assertion::ran().0 > before => Ok(Outcome::Passed),
                    Ok(_) => Ok(Outcome::Unknown),
                }
            })
            .collect::<PyResult<Vec<_>>>()
    })
    .map_err(|e| e.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn three_outcomes() {
        Python::initialize();
        let src = "def test_one():\n    assert 1 == 1\n\
            def test_two():\n    assert 1 == 2\n\
            def test_three():\n    pass\n";
        let names = crate::selection::select(src);
        let got = run("test_a.py", src, &names).unwrap();
        let want = [
            Outcome::Passed,
            Outcome::Failed("assert 1 == 2".into()),
            Outcome::Unknown,
        ];
        assert_eq!(got, want, "one passes, one fails, one checks nothing");
    }
}
