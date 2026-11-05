use std::ops::ControlFlow;
use std::path::PathBuf;

use purba_dispatch::{Handed, Module, Test};
use purba_execution::{Kind, Rewrite};

#[derive(Clone)]
struct Named(&'static str);

impl Test for Named {
    fn function(&self) -> &str {
        self.0
    }
}

/// What dispatch hands on, as data a test can compare.
#[derive(Debug, PartialEq)]
enum Seen {
    Test(&'static str, Result<Kind, String>),
    Unreached(&'static str),
    Other,
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
fn dispatch_hands_on_each_test_after_an_interrupt_as_unreached() {
    purba_execution::interrupt::install().expect("purba's signal handler takes SIGINT");
    let first = module(
        "test_first.py",
        "import signal\n\n\
         def test_one():\n    assert True\n\n\
         def test_two():\n    signal.raise_signal(signal.SIGINT)\n\n\
         def test_three():\n    assert True\n",
        vec![Named("test_one"), Named("test_two"), Named("test_three")],
    );
    let second = module(
        "test_second.py",
        "raise ValueError(\"loaded\")\n\ndef test_four():\n    assert True\n",
        vec![Named("test_four")],
    );
    let mut seen = Vec::new();
    let flow: ControlFlow<()> = purba_dispatch::hand(vec![first, second], |handed| {
        seen.push(match handed {
            Handed::Test(test, result) => Seen::Test(
                test.0,
                result
                    .map(|outcome| outcome.kind().clone())
                    .map_err(|err| err.to_string()),
            ),
            Handed::Unreached(test) => Seen::Unreached(test.0),
            Handed::NotLoaded(..) | Handed::Blocked(..) | Handed::Thread(..) => Seen::Other,
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
