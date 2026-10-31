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

use std::any::Any;
use std::env;
use std::fmt::Display;
use std::io::{self, Write};
use std::panic;

use pyo3::types::{PyAnyMethods, PyModule, PyModuleMethods};
use pyo3::{Bound, PyResult, Python, pyfunction, pymodule, wrap_pyfunction};

/// The console script `purba`. It copies `sys.argv` into a list of strings,
/// runs command on it, and returns the exit code. For the error that stops
/// the run, it writes one line to stderr, and returns 3, or 125 for a panic.
/// A run that exits 125 then asks for a report, in two lines on stderr.
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
    purba_execution::panic::wrap_hook();
    let code = match panic::catch_unwind(|| {
        purba_command::run(&argv, writable_stdout().map(|()| io::stdout()))
    }) {
        Ok(Ok(code)) => code,
        Ok(Err(error)) => stop(&error),
        // Rust's panic hook already wrote the panic's text to stderr, unless
        // `resume_unwind` started the panic.
        Err(payload) => internal(payload.as_ref()),
    };
    if code == 125 {
        ask_for_report(py);
    }
    code
}

/// Whether stdout takes a write. `io::stdout()` reports a write that fails
/// with `EBADF` as done, so purba writes zero bytes to a copy of fd 1:
/// <https://github.com/rust-lang/rust/blob/b6eeb93c99/library/std/src/io/stdio.rs#L203>
/// `write_all` makes no call for an empty buffer, and `write` makes one.
#[cfg(unix)]
fn writable_stdout() -> io::Result<()> {
    use std::os::fd::AsFd;
    let copy = std::fs::File::from(io::stdout().as_fd().try_clone_to_owned()?);
    (&copy).write(&[]).map(drop)
}

/// Whether stdout takes a write. A process with no stdout has a null handle.
/// Nothing writes, because a write of zero bytes reaches a pipe's reader as a
/// read of zero bytes:
/// <https://learn.microsoft.com/en-us/windows/win32/api/fileapi/nf-fileapi-readfile#pipes>
#[cfg(windows)]
fn writable_stdout() -> io::Result<()> {
    use std::os::windows::io::AsRawHandle;
    const ERROR_INVALID_HANDLE: i32 = 6;
    if io::stdout().as_raw_handle().is_null() {
        Err(io::Error::from_raw_os_error(ERROR_INVALID_HANDLE))
    } else {
        Ok(())
    }
}

fn stop(error: &impl Display) -> u8 {
    // stderr may refuse the write too. The run exits 3 either way.
    let _ = writeln!(io::stderr(), "purba: invocation error: {error}");
    3
}

/// Writes the line of the panic that stopped the run, and returns 125. A
/// panic outside the source of a part names `command`, the part `_main` runs.
fn internal(payload: &(dyn Any + Send)) -> u8 {
    let message = text(payload);
    // A panic that `resume_unwind` starts never reaches the hook, so the note
    // can be that of a panic that a test caught:
    // https://doc.rust-lang.org/1.99.1/std/panic/fn.resume_unwind.html
    let part = purba_execution::panic::take()
        .filter(|panic| panic.message == message)
        .and_then(|panic| panic.part);
    let part = part.as_deref().unwrap_or("command");
    // stderr may refuse the write. The run exits 125 either way.
    let _ = writeln!(io::stderr(), "purba: internal error: {part}: {message}");
    125
}

/// The text of a panic's payload, or what Rust's hook writes for one that is
/// not text.
fn text(payload: &(dyn Any + Send)) -> String {
    payload
        .downcast_ref::<&str>()
        .map(|text| (*text).to_owned())
        .or_else(|| payload.downcast_ref::<String>().cloned())
        .unwrap_or_else(|| "Box<dyn Any>".to_owned())
}

/// Writes the two lines that ask for a report of an internal error.
fn ask_for_report(py: Python<'_>) {
    let version = py.version_info();
    let flags = py
        .import("sys")
        .and_then(|sys| sys.getattr("abiflags"))
        .and_then(|flags| flags.extract::<String>())
        .unwrap_or_default();
    let mut stderr = io::stderr();
    // Each wheel of purba carries a `cp` tag, and pip gives it to CPython alone:
    // https://github.com/pypa/packaging/blob/26.2/src/packaging/tags.py#L875-L878
    let _ = writeln!(
        stderr,
        "purba: an internal error is a bug in purba {}, on CPython {}.{}.{}{}{flags}, {} {}.",
        env!("CARGO_PKG_VERSION"),
        version.major,
        version.minor,
        version.patch,
        version.suffix.unwrap_or(""),
        env::consts::OS,
        env::consts::ARCH,
    );
    let _ = writeln!(
        stderr,
        "purba: report it at https://github.com/kalonji-tools/purba/issues/new?template=defect.md"
    );
}

/// The extension module Python imports as `purba`.
#[pymodule]
fn purba(module: &Bound<'_, PyModule>) -> PyResult<()> {
    module.add_function(wrap_pyfunction!(main, module)?)
}
