mod common;

use std::ops::ControlFlow;

use common::module;
use purba_dispatch::Handed;
use purba_execution::Kind;

/// What dispatch hands on, as data a test can compare.
#[derive(Debug, PartialEq)]
enum Seen {
    Test(&'static str, Result<Kind, String>),
    Unreached(&'static str),
    Other,
}

// purba's signal handler keeps the first signal for the life of the process.
// So the one signal of this binary lives in this one test.
#[test]
fn dispatch_hands_on_each_test_after_an_interrupt_as_unreached() {
    purba_execution::set_hooks().expect("purba's signal handler takes SIGINT");
    let first = module(
        "test_first.py",
        "import signal\n\n\
         def test_one():\n    assert True\n\n\
         def test_two():\n    signal.raise_signal(signal.SIGINT)\n\n\
         def test_three():\n    assert True\n",
        &["test_one", "test_two", "test_three"],
    );
    let second = module(
        "test_second.py",
        "raise ValueError(\"loaded\")\n\ndef test_four():\n    assert True\n",
        &["test_four"],
    );
    let mut seen = Vec::new();
    let flow: ControlFlow<()> = purba_dispatch::hand(vec![first, second], |handed| {
        seen.push(match handed {
            Handed::Test(test, result, _) => Seen::Test(
                test.0,
                result
                    .map(|outcome| outcome.kind().clone())
                    .map_err(|err| err.to_string()),
            ),
            Handed::Unreached(test) => Seen::Unreached(test.0),
            Handed::Stopped { .. } | Handed::Thread(..) => Seen::Other,
        });
        ControlFlow::Continue(())
    });
    assert_eq!(flow, ControlFlow::Continue(()), "nothing breaks");
    assert_eq!(
        seen,
        [
            Seen::Test("test_one", Ok(Kind::Passed)),
            Seen::Test("test_two", Ok(Kind::Interrupted)),
            Seen::Unreached("test_three"),
            Seen::Unreached("test_four"),
        ],
        "the signal's test is interrupted. Each later test is unreached. The next \
         module never loads."
    );
}
