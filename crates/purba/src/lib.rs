//! purba — a Python test framework written in Rust.
//!
//! This crate holds no product code yet. It exists so that the seam, the
//! toolchain and the build are facts.
//!
//! Rust examples in these docs are verified by `cargo test --doc`, which is
//! only possible because `[lib] crate-type` carries `rlib` alongside `cdylib`.
//! Note the bound: a verified example can cover the Rust core, never the
//! bridge — `extension-module` omits libpython, so anything touching `Py*`
//! needs that feature off.

use pyo3::types::PyModule;
use pyo3::{Bound, PyResult, pymodule};

/// The extension module Python imports as `purba`.
#[pymodule]
const fn purba(_module: &Bound<'_, PyModule>) -> PyResult<()> {
    Ok(())
}
