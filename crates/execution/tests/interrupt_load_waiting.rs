//! A signal that waits as a load starts stops that load.

use std::path::PathBuf;

use purba_execution::{Module, Rewrite, Worker};
use pyo3::Python;
use pyo3::types::PyAnyMethods;

/// A module whose first line marks that it ran.
const TEST_WAITING: &str = "import builtins\nbuiltins.purba_waiting_ran = True\n";

// purba's signal handler keeps the first signal for the life of the process.
// So the one signal of this binary lives in this one test.
#[test]
fn a_sigint_that_waits_as_a_load_starts_stops_the_load_and_no_line_of_the_module_runs() {
    let module = Module::new(
        PathBuf::from("test_waiting.py"),
        Rewrite::of_source(TEST_WAITING).expect("the source parses"),
    );
    let (loaded, ran, kept) = purba_execution::attach(|worker: &mut Worker<'_, ()>| {
        purba_execution::interrupt::install().expect("purba's signal handler takes SIGINT");
        // SAFETY: `raise` takes a signal number. Python's own C handler only
        // marks the signal as waiting, so purba's signal handler runs at the
        // next line of Python, which is inside the load's window.
        unsafe { libc::raise(libc::SIGINT) };
        let loaded = worker.load(&module).map_err(|error| error.to_string());
        let ran = Python::attach(|py| {
            py.import("builtins")
                .and_then(|builtins| builtins.hasattr("purba_waiting_ran"))
                .expect("builtins answers hasattr")
        });
        (loaded, ran, purba_execution::interrupted())
    });
    assert_eq!(
        (loaded, ran, kept),
        (Ok(()), false, true),
        "a SIGINT that waits as a load starts stops the load before the module's first line, \
         gives no error, and purba keeps it"
    );
}
