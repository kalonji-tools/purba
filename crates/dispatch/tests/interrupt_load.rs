mod common;

use std::ops::ControlFlow;

use common::module;
use purba_dispatch::Handed;

/// What dispatch hands on, by the name of its variant and of its test.
#[derive(Debug, PartialEq)]
enum Seen {
    Test(&'static str),
    Stopped,
    Thread,
    Unreached(&'static str),
}

// purba's signal handler keeps the first signal for the life of the process.
// So the one signal of this binary lives in this one test.
#[test]
fn dispatch_hands_on_each_test_after_an_interrupt_in_a_load_as_unreached() {
    purba_execution::set_hooks().expect("purba's signal handler takes SIGINT");
    let first = module(
        "test_first.py",
        "import signal\n\nsignal.raise_signal(signal.SIGINT)\n\n\
         def test_one():\n    assert True\n\n\
         def test_two():\n    assert True\n",
        &["test_one", "test_two"],
    );
    let second = module(
        "test_second.py",
        "def test_three():\n    assert True\n",
        &["test_three"],
    );
    let mut seen = Vec::new();
    let flow: ControlFlow<()> = purba_dispatch::hand(vec![first, second], |handed| {
        seen.push(match handed {
            Handed::Test(test, _, _) => Seen::Test(test.0),
            Handed::Stopped { .. } | Handed::Ended { .. } => Seen::Stopped,
            Handed::Thread(..) => Seen::Thread,
            Handed::Unreached(test) => Seen::Unreached(test.0),
        });
        ControlFlow::Continue(())
    });
    assert_eq!(flow, ControlFlow::Continue(()), "nothing breaks");
    assert_eq!(
        seen,
        [
            Seen::Unreached("test_one"),
            Seen::Unreached("test_two"),
            Seen::Unreached("test_three"),
        ],
        "the load that the signal lands in gives no error, and blocks no test. Each \
         test of its module and of the next module is unreached."
    );
}
