//! purba — a Python test framework written in Rust.
//!
//! PROTOTYPE for #535, branch `proto-535-wheel-test`. Never merges. `_main` is a
//! stub that prints the four lines the end-to-end test expects, so the harness
//! around it can be judged. Two environment variables break it on purpose:
//!
//! - `PURBA_PROTO_IMPORT_B=1` imports `test_b.py` and swallows its error.
//! - `PURBA_PROTO_WRONG_LINE=1` prints `test_three` as failed.

use pyo3::prelude::*;

const EXPECTED: &str = "\
test_a.py::test_one passed
test_a.py::test_two failed: assert 1 == 2
test_a.py::test_three unknown: no assert ran
3 tests: 1 passed, 1 failed, 1 unknown";

#[pyfunction]
fn _main(py: Python<'_>) -> i32 {
    if std::env::var_os("PURBA_PROTO_IMPORT_B").is_some() {
        // The error is dropped, so stdout and the exit code stay the same.
        let _ = py.run(
            c"import importlib.util as u
s = u.spec_from_file_location('test_b', 'test_b.py')
s.loader.exec_module(u.module_from_spec(s))",
            None,
            None,
        );
    }
    if std::env::var_os("PURBA_PROTO_WRONG_LINE").is_some() {
        println!(
            "{}",
            EXPECTED.replace("test_three unknown: no assert ran", "test_three failed")
        );
    } else {
        println!("{EXPECTED}");
    }
    1
}

/// The extension module Python imports as `purba`.
#[pymodule]
fn purba(module: &Bound<'_, PyModule>) -> PyResult<()> {
    module.add_function(wrap_pyfunction!(_main, module)?)
}
