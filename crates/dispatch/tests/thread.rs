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

// The hook on `threading.excepthook` keeps one record for the process, and
// cargo runs the tests of one binary on several threads. So each thread that
// raises in this process lives in this one test, alone in its binary.
#[test]
fn dispatch_hands_on_a_threads_error_after_the_load_or_the_test_that_took_it() {
    let source = "import threading\n\ngo = threading.Event()\nleft = []\n\n\
                  loader = threading.Thread(target=lambda: 1 / 0, name=\"L\")\n\
                  loader.start()\nloader.join()\n\n\
                  def late():\n    go.wait()\n    raise ValueError(\"late\")\n\n\
                  def test_leaves():\n\
                  \x20   thread = threading.Thread(target=late, name=\"T\", daemon=True)\n\
                  \x20   thread.start()\n    left.append(thread)\n    assert True\n\n\
                  def test_releases():\n    go.set()\n    left[0].join()\n    assert True\n";
    let module = Module::new(
        PathBuf::from("test_late.py"),
        Rewrite::of_source(source).expect("the source parses"),
        vec![Named("test_leaves"), Named("test_releases")],
    );
    let mut seen = Vec::new();
    let flow: ControlFlow<()> = purba_dispatch::hand(vec![module], |handed| {
        seen.push(match handed {
            Handed::Test(test, result) => Seen::Test(
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
            Seen::Test("test_releases", Ok(Kind::Passed)),
            Seen::Thread(
                Ok("test_leaves"),
                "thread T raised ValueError: late".to_owned()
            ),
        ],
        "the load's thread error before the first test, and a test's thread error after \
         the test that ran when it raised, under the test that left the thread"
    );
}
