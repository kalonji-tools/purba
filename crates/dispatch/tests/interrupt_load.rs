use std::ops::ControlFlow;
use std::path::PathBuf;

use purba_dispatch::{Handed, Module, Test};
use purba_execution::Rewrite;

#[derive(Clone)]
struct Named(&'static str);

impl Test for Named {
    fn function(&self) -> &str {
        self.0
    }
}

/// What dispatch hands on, by the name of its variant and of its test.
#[derive(Debug, PartialEq)]
enum Seen {
    Test(&'static str),
    NotLoaded,
    Blocked(&'static str),
    Thread,
    Unreached(&'static str),
}

fn module(name: &str, source: &str, tests: Vec<Named>) -> Module<Named> {
    Module::new(
        PathBuf::from(name),
        Rewrite::of_source(source).expect("the source parses"),
        tests,
    )
}

// purba's signal handler keeps the first signal for the life of the process.
// So the one signal of this binary lives in this one test.
#[test]
fn dispatch_hands_on_each_test_after_an_interrupt_in_a_load_as_unreached() {
    purba_execution::interrupt::install().expect("purba's signal handler takes SIGINT");
    let first = module(
        "test_first.py",
        "import signal\n\nsignal.raise_signal(signal.SIGINT)\n\n\
         def test_one():\n    assert True\n\n\
         def test_two():\n    assert True\n",
        vec![Named("test_one"), Named("test_two")],
    );
    let second = module(
        "test_second.py",
        "def test_three():\n    assert True\n",
        vec![Named("test_three")],
    );
    let mut seen = Vec::new();
    let flow: ControlFlow<()> = purba_dispatch::hand(vec![first, second], |handed| {
        seen.push(match handed {
            Handed::Test(test, _) => Seen::Test(test.0),
            Handed::NotLoaded(..) => Seen::NotLoaded,
            Handed::Blocked(test, _) => Seen::Blocked(test.0),
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
