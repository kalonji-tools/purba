//! PROTOTYPE assertion: rewrites each assert to record that it ran. `PyO3` lives here.
use crate::parsing;
use pyo3::prelude::*;
use std::sync::atomic::{AtomicUsize, Ordering};

static RAN: AtomicUsize = AtomicUsize::new(0);
static LAST: AtomicUsize = AtomicUsize::new(0);

#[pyfunction]
#[pyo3(name = "_purba_ran")]
fn purba_ran(index: usize, value: Bound<'_, PyAny>) -> Bound<'_, PyAny> {
    RAN.fetch_add(1, Ordering::SeqCst);
    LAST.store(index, Ordering::SeqCst);
    value
}

/// How many asserts ran so far, and the index of the last one.
#[must_use]
pub fn ran() -> (usize, usize) {
    (RAN.load(Ordering::SeqCst), LAST.load(Ordering::SeqCst))
}

/// Wraps each assert's test in a recording call, and keeps each assert's text.
#[must_use]
pub fn rewrite(source: &str) -> (String, Vec<String>) {
    let mut asserts = vec![];
    let lines: Vec<String> = parsing::parse(source)
        .into_iter()
        .map(|line| {
            line.text.strip_prefix("assert ").map_or_else(
                || format!("{}{}", line.indent, line.text),
                |test| {
                    asserts.push(line.text.clone());
                    format!(
                        "{}assert _purba_ran({}, {test})",
                        line.indent,
                        asserts.len() - 1
                    )
                },
            )
        })
        .collect();
    (lines.join("\n") + "\n", asserts)
}

/// The rewritten module, executed, and the source text of each assert.
///
/// # Errors
/// When the rewritten source does not compile or its body raises.
pub fn load<'py>(
    py: Python<'py>,
    path: &str,
    source: &str,
) -> PyResult<(Bound<'py, PyAny>, Vec<String>)> {
    let (rewritten, asserts) = rewrite(source);
    let module = py
        .import("types")?
        .getattr("ModuleType")?
        .call1(("test_module",))?;
    let globals = module.getattr("__dict__")?;
    globals.set_item("__file__", path)?;
    py.import("sys")?
        .getattr("modules")?
        .set_item("test_module", &module)?;
    globals.set_item("_purba_ran", wrap_pyfunction!(purba_ran, py)?)?;
    let builtins = py.import("builtins")?;
    let code = builtins
        .getattr("compile")?
        .call1((rewritten, path, "exec"))?;
    builtins.getattr("exec")?.call1((code, &globals))?;
    Ok((module, asserts))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rewrite_numbers_each_assert() {
        let (rewritten, asserts) = rewrite("def test_two():\n    assert 1 == 2\n");
        assert_eq!(
            rewritten, "def test_two():\n    assert _purba_ran(0, 1 == 2)\n",
            "the first assert gets index 0"
        );
        assert_eq!(
            asserts,
            ["assert 1 == 2"],
            "the text is kept for the failed line"
        );
    }
}
