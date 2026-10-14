//! PROTOTYPE lib: the extension module, and later what a test imports.
use pyo3::prelude::*;

#[pyfunction]
fn _main(py: Python<'_>) -> i32 {
    let argv: PyResult<Vec<String>> = py.import("sys").and_then(|s| s.getattr("argv")?.extract());
    argv.map_or(3, |argv| py.detach(|| purba_cli::run(&argv[1..])))
}

#[pymodule]
fn purba(module: &Bound<'_, PyModule>) -> PyResult<()> {
    module.add_function(wrap_pyfunction!(_main, module)?)
}
