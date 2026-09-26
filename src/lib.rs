//! purba — a Python test framework written in Rust.
//!
//! This crate is the standing scaffold and holds no product code. It exists so
//! that the seam, the toolchain and the build are facts before any feature is
//! designed.
//!
//! Rust examples in these docs are verified by `cargo test --doc`, which is
//! only possible because `[lib] crate-type` carries `rlib` alongside `cdylib`.
//! Note the bound: a verified example can cover the Rust core, never the
//! bridge — `extension-module` omits libpython, so anything touching `Py*`
//! needs that feature off.

use pyo3::prelude::*;

/// The extension module Python imports as `purba`.
#[pymodule]
const fn purba(_module: &Bound<'_, PyModule>) -> PyResult<()> {
    Ok(())
}
