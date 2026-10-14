use pyo3::prelude::*;
use std::sync::atomic::{AtomicUsize, Ordering};

static RAN: AtomicUsize = AtomicUsize::new(0);
static LAST: AtomicUsize = AtomicUsize::new(0);

#[pyfunction]
fn _purba_ran(index: usize, value: Bound<'_, PyAny>) -> Bound<'_, PyAny> {
    RAN.fetch_add(1, Ordering::SeqCst);
    LAST.store(index, Ordering::SeqCst);
    value
}

/// How many asserts ran so far, and the index of the last one.
#[must_use]
pub fn ran() -> (usize, usize) {
    (RAN.load(Ordering::SeqCst), LAST.load(Ordering::SeqCst))
}

/// A line-based stand-in for ruff: wraps each assert's test in a recording call.
#[must_use]
pub fn rewrite(source: &str) -> (String, Vec<String>) {
    let mut asserts = vec![];
    let lines: Vec<String> = source
        .lines()
        .map(|line| {
            let body = line.trim_start();
            body.strip_prefix("assert ").map_or_else(
                || line.to_owned(),
                |expr| {
                    let indent = &line[..line.len() - body.len()];
                    asserts.push(body.to_owned());
                    format!("{indent}assert _purba_ran({}, {expr})", asserts.len() - 1)
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
    globals.set_item("_purba_ran", wrap_pyfunction!(_purba_ran, py)?)?;
    let builtins = py.import("builtins")?;
    let code = builtins
        .getattr("compile")?
        .call1((rewritten, path, "exec"))?;
    builtins.getattr("exec")?.call1((code, &globals))?;
    Ok((module, asserts))
}
