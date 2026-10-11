// PROTOTYPE #482, thrown away. The tester's CPython starts first, then loads this.
use pyo3::prelude::*;

#[path = "../../common.rs"]
mod common;

#[pyfunction]
fn run(py: Python<'_>, path: &str) -> PyResult<()> {
    let tests = common::select(path);
    println!("  selected after the interpreter started, before any test import: {tests:?}");
    common::report(py)?;
    common::run_tests(py, path, &tests)
}

#[pymodule]
fn proto482_ext(module: &Bound<'_, PyModule>) -> PyResult<()> {
    module.add_function(wrap_pyfunction!(run, module)?)
}
