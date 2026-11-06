use std::path::PathBuf;

use purba_execution::{Kind, Module, Rewrite, Worker};
use pyo3::Python;
use pyo3::types::{PyAnyMethods, PyTypeMethods};

const TEST_SIGNALS: &str = "import signal\n\n\
    def test_bare():\n    raise KeyboardInterrupt\n\n\
    def test_handler():\n    signal.signal(signal.SIGINT, lambda signum, frame: None)\n    \
    signal.signal(signal.SIGTERM, lambda signum, frame: None)\n    \
    assert True\n\n\
    def test_signal():\n    try:\n        signal.raise_signal(signal.SIGINT)\n    \
    except KeyboardInterrupt:\n        pass\n    assert True\n";

/// What one step of the test saw: the outcome of a test, then whether purba
/// kept a signal.
type Step = (Result<Kind, String>, bool);

/// The outcome of `function`, or the text of its error.
fn run(
    worker: &mut Worker<'_, &'static str>,
    module: &Module,
    function: &'static str,
) -> Result<Kind, String> {
    worker
        .run(module, function, function)
        .map(|outcome| outcome.kind().clone())
        .map_err(|error| error.to_string())
}

/// The name of the type of the signal handler that `name` has now.
fn handler(name: &str) -> String {
    Python::attach(|py| {
        py.import("signal")
            .and_then(|signal| {
                let handler = signal
                    .getattr("getsignal")?
                    .call1((signal.getattr(name)?,))?;
                handler.get_type().name()?.extract::<String>()
            })
            .expect("signal.getsignal names the signal handler")
    })
}

// purba's signal handler keeps the first signal for the life of the process.
// Cargo runs the tests of one binary on several threads. So each signal in
// this process lives in this one test, alone in its binary. Its steps run in
// order.
#[test]
fn purbas_signal_handler_keeps_a_signal_and_raises_it_inside_a_call() {
    let module = Module::new(
        PathBuf::from("test_signals.py"),
        Rewrite::of_source(TEST_SIGNALS).expect("the source parses"),
    );
    let (steps, handlers): (Vec<Step>, [String; 2]) = purba_execution::attach(|worker| {
        purba_execution::interrupt::install().expect("purba's signal handler takes SIGINT");
        worker.load(&module).expect("the module loads");
        let mut steps = Vec::new();
        let kind = run(worker, &module, "test_bare");
        steps.push((kind, purba_execution::interrupted()));
        let kind = run(worker, &module, "test_handler");
        let handlers = [handler("SIGINT"), handler("SIGTERM")];
        steps.push((kind, purba_execution::interrupted()));
        let kind = run(worker, &module, "test_signal");
        steps.push((kind, purba_execution::interrupted()));
        (steps, handlers)
    });
    assert_eq!(
        handlers,
        ["SignalHandler", "SignalHandler"],
        "purba puts its own signal handler back for SIGINT and for SIGTERM when the call \
         of a test that took them returns"
    );
    assert_eq!(
        steps,
        [
            (
                Ok(Kind::Raised {
                    exception: "KeyboardInterrupt: ".to_owned(),
                }),
                false
            ),
            (Ok(Kind::Passed), false),
            (Ok(Kind::Interrupted), true),
        ],
        "purba keeps no signal after a bare raise. A test that takes SIGINT passes. A \
         signal in a call makes the test interrupted, though it caught the \
         KeyboardInterrupt and passed."
    );
}
