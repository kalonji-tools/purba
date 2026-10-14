//! PROTOTYPE for purba #537, layout CRATE PER PART: the extension module; each part is a crate under src/.
use pyo3::prelude::*;

#[pyfunction]
#[pyo3(name = "_main")]
fn main(py: Python<'_>) -> i32 {
    let argv: PyResult<Vec<String>> = py
        .import("sys")
        .and_then(|sys| sys.getattr("argv")?.extract());
    argv.map_or(3, |argv| py.detach(|| purba_command::run(&argv[1..])))
}

/// The extension module Python imports as `purba`.
#[pymodule]
fn purba(module: &Bound<'_, PyModule>) -> PyResult<()> {
    module.add_function(wrap_pyfunction!(main, module)?)
}
