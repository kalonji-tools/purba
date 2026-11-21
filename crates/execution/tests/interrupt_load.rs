mod common;

use common::module;
use purba_execution::{Kind, Worker};

/// A module whose load takes SIGINT, and whose test holds when purba put its
/// own signal handler back after the load.
const TEST_LOAD_HANDLER: &str = "import signal\n\n\
    signal.signal(signal.SIGINT, lambda signum, frame: None)\n\n\
    def test_handler():\n    \
    assert type(signal.getsignal(signal.SIGINT)).__name__ == 'SignalHandler'\n";

const TEST_LOAD_SIGNAL: &str = "import signal\n\ntry:\n    signal.raise_signal(signal.SIGINT)\n\
    except KeyboardInterrupt:\n    pass\n\n\
    def test_one():\n    assert True\n";

/// The error of one load, if any, then whether purba kept a signal.
type Step = (Result<(), String>, bool);

// purba's signal handler keeps the first signal for the life of the process.
// So the one signal of this binary lives in this one test.
#[test]
fn a_signal_in_a_load_interrupts_it_and_a_modules_signal_handler_lives_only_for_its_load() {
    let handler_module = module("test_load_handler.py", TEST_LOAD_HANDLER);
    let signal_module = module("test_load_signal.py", TEST_LOAD_SIGNAL);
    let (steps, handler, after): (Vec<Step>, Result<Kind, String>, Result<Kind, String>) =
        purba_execution::attach(|worker: &mut Worker<'_, &str>| {
            purba_execution::set_hooks().expect("purba's signal handler takes SIGINT");
            let mut steps = Vec::new();
            let mut run = |module, function| match worker.zygote(module) {
                Ok((mut zygote, _)) => {
                    steps.push((Ok(()), purba_execution::interrupted()));
                    let (result, _, _) = zygote.run(function, &[], function);
                    zygote.end();
                    result
                        .map(|outcome| outcome.kind().clone())
                        .map_err(|error| error.to_string())
                }
                Err(stop) => {
                    steps.push((Err(stop.error.to_string()), purba_execution::interrupted()));
                    Err("the module stopped".to_owned())
                }
            };
            let handler = run(&handler_module, "test_handler");
            let after = run(&signal_module, "test_one");
            (steps, handler, after)
        });
    assert_eq!(
        handler,
        Ok(Kind::Passed),
        "purba puts its own signal handler back when the load of a module that took SIGINT returns"
    );
    assert_eq!(
        steps,
        [(Ok(()), false), (Ok(()), true)],
        "a module that takes SIGINT loads. A signal in a load gives no error, and purba \
         keeps it."
    );
    assert_eq!(
        after,
        Err("test_load_signal.py did not load".to_owned()),
        "a load that the first interrupt lands in keeps no module, so a later run names it"
    );
}
