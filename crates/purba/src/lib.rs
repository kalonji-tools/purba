//! purba — a Python test framework written in Rust.
//!
//! This crate is the extension module, and holds `_main`, the entry of the
//! console script.
//!
//! Rust examples in these docs are verified by `cargo test --doc`, which is
//! only possible because `[lib] crate-type` carries `rlib` alongside `cdylib`.
//! Note the bound: a verified example can cover the Rust core, never the
//! bridge — `extension-module` omits libpython, so anything touching `Py*`
//! needs that feature off.

use std::fmt::Display;
use std::io::{self, Write};
use std::panic;

use pyo3::types::{PyAnyMethods, PyModule, PyModuleMethods};
use pyo3::{Bound, PyResult, Python, pyfunction, pymodule, wrap_pyfunction};

/// The console script `purba`. It copies `sys.argv` into a list of strings,
/// runs command on it, and returns the exit code. For each error that stops
/// the run, it writes one line to stderr, and returns 3.
#[pyfunction]
#[pyo3(name = "_main")]
fn main(py: Python<'_>) -> u8 {
    let argv = match py
        .import("sys")
        .and_then(|sys| sys.getattr("argv"))
        .and_then(|argv| argv.extract::<Vec<String>>())
    {
        Ok(argv) => argv,
        Err(error) => return stop(&error),
    };
    match panic::catch_unwind(|| purba_command::run(&argv)) {
        Ok(Ok(code)) => code,
        Ok(Err(error)) => stop(&error),
        // The panic hook already wrote the panic's message to stderr.
        Err(_) => 3,
    }
}

fn stop(error: &impl Display) -> u8 {
    // stderr may refuse the write too. The run exits 3 either way.
    let _ = writeln!(io::stderr(), "purba: {error}");
    3
}

/// The extension module Python imports as `purba`.
#[pymodule]
fn purba(module: &Bound<'_, PyModule>) -> PyResult<()> {
    module.add_function(wrap_pyfunction!(main, module)?)
}
