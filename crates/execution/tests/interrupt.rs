use std::path::PathBuf;

use purba_execution::{Kind, Module, Rewrite};
use pyo3::Python;

const TEST_SIGNALS: &str = "import signal\n\n\
    def test_bare():\n    raise KeyboardInterrupt\n\n\
    def test_signal():\n    signal.raise_signal(signal.SIGINT)\n";

const TEST_LOAD_SIGNAL: &str = "import signal\n\nsignal.raise_signal(signal.SIGINT)\n";

/// What one step of the test saw: the outcome of a test, the result of a load,
/// or a signal that the test raised outside a call, then whether purba kept a
/// signal.
type Step = (Result<Option<Kind>, String>, bool);

// purba's signal handler keeps the first signal for the life of the process.
// Cargo runs the tests of one binary on several threads. So each signal in
// this process lives in this one test, alone in its binary. Its steps run in
// order.
#[test]
fn purbas_signal_handler_keeps_a_signal_and_raises_it_only_inside_a_call() {
    let module = Module::new(
        PathBuf::from("test_signals.py"),
        Rewrite::of_source(TEST_SIGNALS).expect("the source parses"),
    );
    let load_signal = Module::new(
        PathBuf::from("test_load_signal.py"),
        Rewrite::of_source(TEST_LOAD_SIGNAL).expect("the source parses"),
    );
    let steps: Vec<Step> = purba_execution::attach(|worker| {
        purba_execution::interrupt::install().expect("purba's signal handler takes SIGINT");
        let mut steps = Vec::new();
        let kind = worker
            .run(&module, "test_bare")
            .map(|outcome| Some(outcome.kind().clone()))
            .map_err(|error| error.to_string());
        steps.push((kind, worker.interrupted()));
        let outside = Python::attach(|py| {
            py.run(
                c"import signal\nsignal.raise_signal(signal.SIGINT)",
                None,
                None,
            )
        })
        .map(|()| None)
        .map_err(|error| error.to_string());
        steps.push((outside, worker.interrupted()));
        let kind = worker
            .run(&module, "test_signal")
            .map(|outcome| Some(outcome.kind().clone()))
            .map_err(|error| error.to_string());
        steps.push((kind, worker.interrupted()));
        let loaded = worker
            .load(&load_signal)
            .map(|()| None)
            .map_err(|error| error.to_string());
        steps.push((loaded, worker.interrupted()));
        steps
    });
    let raised = || {
        Ok(Some(Kind::Raised {
            exception: "KeyboardInterrupt: ".to_owned(),
        }))
    };
    assert_eq!(
        steps,
        [
            (raised(), false),
            (Ok(None), true),
            (raised(), true),
            (Err("KeyboardInterrupt: ".to_owned()), true),
        ],
        "purba keeps no signal after a bare raise; it keeps a signal outside a call, \
         and raises nothing; a signal raises KeyboardInterrupt in a call and in a load"
    );
}
