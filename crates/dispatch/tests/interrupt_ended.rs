mod common;

use std::ops::ControlFlow;

use common::module;
use purba_dispatch::Handed;

/// What dispatch hands on, as data a test can compare.
#[derive(Debug, PartialEq)]
enum Seen {
    Test(&'static str),
    Unreached(&'static str),
    Ended(Vec<&'static str>),
    Other,
}

// purba's signal handler keeps the first signal for the life of the process.
// So the one signal of this binary lives in this one test.
#[test]
fn an_interrupt_beats_the_end_of_the_modules_process_for_each_test_left() {
    purba_execution::set_hooks().expect("purba's signal handler takes SIGINT");
    let source = format!(
        "import os\nimport signal\n\n\
         def test_one():\n    os.kill(os.getppid(), signal.SIGKILL)\n\
         \x20   os.kill({purba}, signal.SIGINT)\n    assert True\n\n\
         def test_two():\n    assert True\n\n\
         def test_three():\n    assert True\n",
        purba = std::process::id()
    );
    let ended = module(
        "test_ended.py",
        &source,
        &["test_one", "test_two", "test_three"],
    );
    let mut seen = Vec::new();
    let flow: ControlFlow<()> = purba_dispatch::hand(vec![ended], |handed| {
        seen.push(match handed {
            Handed::Test(test, ..) => Seen::Test(test.0),
            Handed::Unreached(test) => Seen::Unreached(test.0),
            Handed::Ended { tests, .. } => Seen::Ended(tests.iter().map(|test| test.0).collect()),
            Handed::Stopped { .. } | Handed::TestThread(..) | Handed::LoadThread(..) => Seen::Other,
        });
        ControlFlow::Continue(())
    });
    assert_eq!(flow, ControlFlow::Continue(()), "nothing breaks");
    assert_eq!(
        seen,
        [
            Seen::Test("test_one"),
            Seen::Unreached("test_two"),
            Seen::Unreached("test_three"),
        ],
        "test_one ends the module's process, then interrupts purba. Each test left is \
         unreached, and none is blocked by the end"
    );
}
