//! PROTOTYPE for #536 Q2a — throwaway, never merges.
//!
//! Two shapes of the command run the same skeleton run:
//!
//! - the CRATE shape: `purba = "purba:_main"` is a console script, so the
//!   tester's python loads this module and calls `_main()`.
//! - the EXECUTABLE shape: `cli/` builds a Rust binary that finds a python and
//!   spawns it with `import purba; purba._run(path)`.
//!
//! Both print a `[proto]` line on stderr naming the interpreter that ran the
//! test, so the two shapes can be compared.

use pyo3::exceptions::PyAssertionError;
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

/// assertion's stand-in: a line-based rewrite, where ruff will rewrite the AST.
fn rewrite(source: &str) -> (String, Vec<String>) {
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

/// selection's stand-in: top-level `def test_` lines, in source order.
fn select(source: &str) -> Vec<String> {
    source
        .lines()
        .filter_map(|l| l.strip_prefix("def "))
        .filter_map(|l| l.split('(').next())
        .filter(|n| n.starts_with("test_"))
        .map(str::to_owned)
        .collect()
}

fn run(py: Python<'_>, path: &str, via: &str) -> PyResult<i32> {
    let sys = py.import("sys")?;
    let exe: String = sys.getattr("executable")?.extract()?;
    let version: String = sys.getattr("version")?.extract()?;
    let argv: Vec<String> = sys.getattr("argv")?.extract()?;
    let os_args: Vec<String> = std::env::args().collect();
    eprintln!("[proto] shape: {via}");
    eprintln!(
        "[proto] interpreter: {exe} ({})",
        version.split(' ').next().unwrap_or("")
    );
    eprintln!("[proto] sys.argv: {argv:?}");
    eprintln!("[proto] std::env::args: {os_args:?}");

    let source = std::fs::read_to_string(path)?;
    let (rewritten, asserts) = rewrite(&source);
    let module = py
        .import("types")?
        .getattr("ModuleType")?
        .call1(("test_a",))?;
    let globals = module.getattr("__dict__")?;
    globals.set_item("__file__", path)?;
    globals.set_item("_purba_ran", wrap_pyfunction!(_purba_ran, py)?)?;
    let builtins = py.import("builtins")?;
    let code = builtins
        .getattr("compile")?
        .call1((rewritten, path, "exec"))?;
    builtins.getattr("exec")?.call1((code, &globals))?;

    let (mut passed, mut failed, mut unknown) = (0, 0, 0);
    for name in select(&source) {
        let before = RAN.load(Ordering::SeqCst);
        match module.getattr(name.as_str())?.call0() {
            Err(e) if e.is_instance_of::<PyAssertionError>(py) => {
                failed += 1;
                let text = &asserts[LAST.load(Ordering::SeqCst)];
                println!("{path}::{name} failed: {text}");
            }
            Err(e) => return Err(e),
            Ok(_) if RAN.load(Ordering::SeqCst) > before => {
                passed += 1;
                println!("{path}::{name} passed");
            }
            Ok(_) => {
                unknown += 1;
                println!("{path}::{name} unknown: no assert ran");
            }
        }
    }
    let total = passed + failed + unknown;
    println!("{total} tests: {passed} passed, {failed} failed, {unknown} unknown");
    Ok(if failed > 0 {
        1
    } else if unknown > 0 || total == 0 {
        2
    } else {
        0
    })
}

/// The CRATE shape: a console script calls this with no arguments.
#[pyfunction]
fn _main(py: Python<'_>) -> i32 {
    let argv: Vec<String> = match py
        .import("sys")
        .and_then(|s| s.getattr("argv"))
        .and_then(|a| a.extract())
    {
        Ok(argv) => argv,
        Err(_) => return 3,
    };
    match argv.as_slice() {
        [_, path] => run(py, path, "crate (console script)").unwrap_or(3),
        _ => 3,
    }
}

/// The EXECUTABLE shape: the binary in `cli/` spawns python, which calls this.
#[pyfunction]
fn _run(py: Python<'_>, path: &str) -> i32 {
    run(py, path, "executable (spawned python)").unwrap_or(3)
}

/// The extension module Python imports as `purba`.
#[pymodule]
fn purba(module: &Bound<'_, PyModule>) -> PyResult<()> {
    module.add_function(wrap_pyfunction!(_main, module)?)?;
    module.add_function(wrap_pyfunction!(_run, module)?)?;
    Ok(())
}

// PROTOTYPE: execution and assertion run under `cargo test`, with no wheel.
// On Linux the test binary needs LD_LIBRARY_PATH set to the interpreter's LIBDIR.
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_skeleton_module_exits_1() {
        Python::initialize();
        let path = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/skeleton/test_a.py");
        let code = Python::attach(|py| run(py, path, "cargo test")).unwrap();
        assert_eq!(code, 1, "test_two fails, so the run exits 1");
    }
}
