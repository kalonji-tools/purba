use std::path::PathBuf;

use purba_execution::{Load, Module, Rewrite};
use pyo3::Python;
use pyo3::types::{PyAnyMethods, PyTypeMethods};

const TEST_LOAD_HANDLER: &str =
    "import signal\n\nsignal.signal(signal.SIGINT, lambda signum, frame: None)\n";

const TEST_LOAD_SIGNAL: &str = "import signal\n\ntry:\n    signal.raise_signal(signal.SIGINT)\n\
    except KeyboardInterrupt:\n    pass\n";

/// What one load gave, then whether purba kept a signal.
type Step = (Result<Load, String>, bool);

fn module(name: &str, source: &str) -> Module {
    Module::new(
        PathBuf::from(name),
        Rewrite::of_source(source).expect("the source parses"),
    )
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
// So the one signal of this binary lives in this one test.
#[test]
fn a_signal_in_a_load_interrupts_it_and_a_modules_signal_handler_lives_only_for_its_load() {
    let handler_module = module("test_load_handler.py", TEST_LOAD_HANDLER);
    let signal_module = module("test_load_signal.py", TEST_LOAD_SIGNAL);
    let (steps, handler): (Vec<Step>, String) = purba_execution::attach(|worker| {
        purba_execution::interrupt::install().expect("purba's signal handler takes SIGINT");
        let mut steps = Vec::new();
        let loaded = worker
            .load(&handler_module)
            .map_err(|error| error.to_string());
        let handler = sigint_handler();
        steps.push((loaded, worker.interrupted()));
        let loaded = worker
            .load(&signal_module)
            .map_err(|error| error.to_string());
        steps.push((loaded, worker.interrupted()));
        (steps, handler)
    });
    assert_eq!(
        handler, "SignalHandler",
        "purba puts its own signal handler back when the load of a module that took SIGINT returns"
    );
    assert_eq!(
        steps,
        [(Ok(Load::Done), false), (Ok(Load::Interrupted), true)],
        "a module that takes SIGINT loads. A signal in a load interrupts it, though the \
         module caught the KeyboardInterrupt and loaded."
    );
}
