// PROTOTYPE #482, thrown away. This binary starts first, then loads the tester's libpython.
use pyo3::prelude::*;

#[path = "../../common.rs"]
mod common;

fn main() {
    let path = std::env::args().nth(1).expect("usage: proto482_host <test file>");
    let tests = common::select(&path);
    println!("  selected before any interpreter existed: {tests:?}");
    let result = Python::attach(|py| {
        common::report(py)?;
        common::run_tests(py, &path, &tests)
    });
    if let Err(error) = result {
        println!("  ERROR: {error}");
    }
}
