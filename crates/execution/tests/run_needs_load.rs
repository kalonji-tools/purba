use std::path::PathBuf;

use purba_execution::{ErrorKind, Module, Rewrite};

/// A module whose import sends SIGINT, so a line of it that runs leaves a
/// signal kept.
const TEST_PROBE_SIGNAL: &str = "import signal\n\nsignal.raise_signal(signal.SIGINT)\n\n\n\
    def test_one():\n    pass\n";

// purba's signal handler keeps the first signal for the life of the process.
// So this test lives alone in its binary.
#[test]
fn a_run_on_a_module_never_given_to_load_is_an_internal_error_and_runs_none_of_it() {
    let module = Module::new(
        PathBuf::from("test_probe_signal.py"),
        Rewrite::of_source(TEST_PROBE_SIGNAL).expect("the source parses"),
    );
    let (shown, kept) = purba_execution::attach(|worker| {
        purba_execution::interrupt::install().expect("purba's signal handler takes SIGINT");
        let error = worker
            .run(&module, "test_one", "test_one")
            .expect_err("a run needs a load of its module first");
        let shown = match error.kind() {
            ErrorKind::Project => format!("project error: {error}"),
            ErrorKind::Internal { part } => format!("internal error: {part}: {error}"),
        };
        (shown, purba_execution::interrupted())
    });
    assert_eq!(
        (shown.as_str(), kept),
        (
            "internal error: execution: test_probe_signal.py was never given to load",
            false
        ),
        "a run on a module that was never given to load is an internal error of execution, \
         and no line of the module runs, so no signal is kept"
    );
}
