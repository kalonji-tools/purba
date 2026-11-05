use std::path::PathBuf;

use purba_execution::{Kind, Module, Rewrite, Worker};
use pyo3::Python;
use pyo3::types::{PyAnyMethods, PyTypeMethods};

const TEST_SIGNALS: &str = "import signal\n\n\
    def test_bare():\n    raise KeyboardInterrupt\n\n\
    def test_handler():\n    signal.signal(signal.SIGINT, lambda signum, frame: None)\n    \
    assert True\n\n\
    def test_signal():\n    try:\n        signal.raise_signal(signal.SIGINT)\n    \
    except KeyboardInterrupt:\n        pass\n    assert True\n";

/// What one step of the test saw: the outcome of a test, or a signal that the
/// test raised outside a call, then whether purba kept a signal.
type Step = (Result<Option<Kind>, String>, bool);

/// The outcome of `function`, or the text of its error.
fn run(worker: &mut Worker<'_>, module: &Module, function: &str) -> Result<Option<Kind>, String> {
    worker
        .run(module, function)
        .map(|outcome| Some(outcome.kind().clone()))
        .map_err(|error| error.to_string())
}

/// The name of the type of the signal handler that SIGINT has now.
fn sigint_handler() -> String {
    Python::attach(|py| {
        py.import("signal")
            .and_then(|signal| {
                let handler = signal
                    .getattr("getsignal")?
                    .call1((signal.getattr("SIGINT")?,))?;
                handler.get_type().name()?.extract::<String>()
            })
            .expect("signal.getsignal names SIGINT's signal handler")
    })
}

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
    let (steps, handler): (Vec<Step>, String) = purba_execution::attach(|worker| {
        purba_execution::interrupt::install().expect("purba's signal handler takes SIGINT");
        let mut steps = Vec::new();
        let kind = run(worker, &module, "test_bare");
        steps.push((kind, worker.interrupted()));
        let kind = run(worker, &module, "test_handler");
        let handler = sigint_handler();
        steps.push((kind, worker.interrupted()));
        let kind = run(worker, &module, "test_signal");
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
        (steps, handler)
    });
    assert_eq!(
        handler, "SignalHandler",
        "purba puts its own signal handler back when the call of a test that took SIGINT returns"
    );
    assert_eq!(
        steps,
        [
            (
                Ok(Some(Kind::Raised {
                    exception: "KeyboardInterrupt: ".to_owned(),
                })),
                false
            ),
            (Ok(Some(Kind::Passed)), false),
            (Ok(Some(Kind::Interrupted)), true),
            (Ok(None), true),
        ],
        "purba keeps no signal after a bare raise. A test that takes SIGINT passes. A \
         signal in a call makes the test interrupted, though it caught the \
         KeyboardInterrupt and passed. A signal outside a call raises nothing."
    );
}
