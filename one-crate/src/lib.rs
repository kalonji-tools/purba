//! PROTOTYPE for purba #537, layout ONE CRATE: every part is a module of the one crate.
use pyo3::prelude::*;

pub mod assertion;
pub mod command;
pub mod dispatch;
pub mod execution;
pub mod fixtures;
pub mod parsing;
pub mod reporting;
pub mod selection;

#[pyfunction]
#[pyo3(name = "_main")]
fn main(py: Python<'_>) -> i32 {
    let argv: PyResult<Vec<String>> = py
        .import("sys")
        .and_then(|sys| sys.getattr("argv")?.extract());
    argv.map_or(3, |argv| py.detach(|| command::run(&argv[1..])))
}

/// The extension module Python imports as `purba`.
#[pymodule]
fn purba(module: &Bound<'_, PyModule>) -> PyResult<()> {
    module.add_function(wrap_pyfunction!(main, module)?)
}
