mod common;

use purba_execution::{Kind, Purba};

/// The tests of a module whose calls take or raise signals. `test_handler`
/// marks, as its process ends, the type of the signal handler of SIGINT and
/// of SIGTERM, after purba put its own back.
fn test_signals(marker: &str) -> String {
    format!(
        "import atexit\nimport signal\n\n\
         def test_bare():\n    raise KeyboardInterrupt\n\n\
         def mark():\n    with open({marker:?}, 'w') as marker:\n        \
         marker.write(' '.join(type(signal.getsignal(s)).__name__ \
         for s in (signal.SIGINT, signal.SIGTERM)))\n\n\
         def test_handler():\n    signal.signal(signal.SIGINT, lambda signum, frame: None)\n    \
         signal.signal(signal.SIGTERM, lambda signum, frame: None)\n    \
         atexit.register(mark)\n    assert True\n\n\
         def test_signal():\n    try:\n        signal.raise_signal(signal.SIGINT)\n    \
         except KeyboardInterrupt:\n        pass\n    assert True\n"
    )
}

/// What one step of the test saw: the outcome of a test, then whether purba
/// kept a signal.
type Step = (Result<Kind, String>, bool);

// purba's signal handler keeps the first signal for the life of the process.
// Cargo runs the tests of one binary on several threads. So each signal in
// this process lives in this one test, alone in its binary. Its steps run in
// order.
#[test]
fn purbas_signal_handler_keeps_a_signal_and_raises_it_inside_a_call() {
    let marker = std::env::temp_dir().join(format!("purba-handlers-{}", std::process::id()));
    let module = common::module(
        "test_signals.py",
        &test_signals(marker.to_str().expect("the marker's path is text")),
    );
    let steps: Vec<Step> = purba_execution::attach(|purba: &Purba<'_>| {
        purba_execution::set_hooks().expect("purba's signal handler takes SIGINT");
        let (mut zygote, _) = purba.zygote(&module).expect("the zygote loads the module");
        let steps = ["test_bare", "test_handler", "test_signal"]
            .into_iter()
            .map(|function| {
                let (result, _, _) = zygote.run(function, &[]);
                (
                    result
                        .map(|outcome| outcome.kind().clone())
                        .map_err(|error| error.to_string()),
                    purba_execution::interrupted(),
                )
            })
            .collect();
        zygote.end();
        steps
    });
    let handlers = std::fs::read_to_string(&marker).expect("test_handler marked its handlers");
    std::fs::remove_file(&marker).expect("the marker goes");
    assert_eq!(
        handlers, "SignalHandler SignalHandler",
        "purba puts its own signal handler back for SIGINT and for SIGTERM when the call \
         of a test that took them returns"
    );
    assert_eq!(
        steps,
        [
            (
                Ok(Kind::Raised {
                    exception: "KeyboardInterrupt: ".to_owned(),
                    whence: None,
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
