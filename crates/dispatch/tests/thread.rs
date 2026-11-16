use std::ops::ControlFlow;
use std::path::PathBuf;

use purba_dispatch::{Charged, Handed, Module, Test};
use purba_execution::{Kind, Rewrite};

#[derive(Clone)]
struct Named(&'static str);

impl Test for Named {
    fn function(&self) -> &str {
        self.0
    }
}

/// What dispatch hands on, as data a test can compare. A thread's error names
/// its test, or the path of its module.
#[derive(Debug, PartialEq)]
enum Seen {
    Test(&'static str, Result<Kind, String>),
    Thread(Result<&'static str, PathBuf>, String),
    Other,
}

#[test]
fn dispatch_hands_on_a_threads_error_after_the_load_or_the_test_that_took_it() {
    let source = "import threading, time\n\n\
                  loader = threading.Thread(target=lambda: 1 / 0, name=\"L\")\n\
                  loader.start()\nloader.join()\n\n\
                  def late():\n    time.sleep(0.1)\n    raise ValueError(\"late\")\n\n\
                  def test_leaves():\n\
                  \x20   threading.Thread(target=late, name=\"T\").start()\n    assert True\n\n\
                  def test_after():\n    assert True\n";
    let module = Module::new(
        PathBuf::from("test_late.py"),
        Rewrite::of_source(source, &tests_in(source)).expect("the source parses"),
        vec![Named("test_leaves"), Named("test_after")],
    );
    let mut seen = Vec::new();
    let flow: ControlFlow<()> = purba_dispatch::hand(vec![module], |handed| {
        seen.push(match handed {
            Handed::Test(test, result, _) => Seen::Test(
                test.0,
                result
                    .map(|outcome| outcome.kind().clone())
                    .map_err(|err| err.to_string()),
            ),
            Handed::Thread(Charged::Test(test), err) => Seen::Thread(Ok(test.0), err.to_string()),
            Handed::Thread(Charged::Module(path), err) => Seen::Thread(Err(path), err.to_string()),
            Handed::NotLoaded(..) | Handed::Blocked(..) | Handed::Unreached(..) => Seen::Other,
        });
        ControlFlow::Continue(())
    });
    assert_eq!(flow, ControlFlow::Continue(()), "nothing breaks");
    assert_eq!(
        seen,
        [
            Seen::Thread(
                Err(PathBuf::from("test_late.py")),
                "thread L raised ZeroDivisionError: division by zero".to_owned()
            ),
            Seen::Test("test_leaves", Ok(Kind::Passed)),
            Seen::Thread(
                Ok("test_leaves"),
                "thread T raised ValueError: late".to_owned()
            ),
            Seen::Test("test_after", Ok(Kind::Passed)),
        ],
        "the load's thread error before the first test, and the error of a thread that \
         raises after its test returns under that test, before the next test"
    );
}

/// The name of each `def test_…` at the top of `source`, as selection names
/// the tests of a module.
fn tests_in(source: &str) -> Vec<&str> {
    source
        .lines()
        .filter_map(|line| line.strip_prefix("def test_").map(|_| &line[4..]))
        .filter_map(|line| line.split_once('(').map(|(name, _)| name))
        .collect()
}
