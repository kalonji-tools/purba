mod common;

use std::ops::ControlFlow;
use std::path::PathBuf;

use purba_dispatch::Handed;
use purba_execution::Kind;

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
    let module = common::module("test_late.py", source, &["test_leaves", "test_after"]);
    // The load leaves a thread alive, and the test makes it raise.
    let leaves = "import threading\n\ngo = threading.Event()\n\n\
                  def late():\n    go.wait()\n    raise ValueError(\"late\")\n\n\
                  thread = threading.Thread(target=late, name=\"M\", daemon=True)\n\
                  thread.start()\n\n\
                  def test_releases():\n    go.set()\n    thread.join()\n    assert True\n";
    let leaves = common::module("test_leaves.py", leaves, &["test_releases"]);
    let mut seen = Vec::new();
    let flow: ControlFlow<()> = purba_dispatch::hand(vec![module, leaves], |handed| {
        seen.push(match handed {
            Handed::Test(test, result, _) => Seen::Test(
                test.0,
                result
                    .map(|outcome| outcome.kind().clone())
                    .map_err(|err| err.to_string()),
            ),
            Handed::TestThread(test, err) => Seen::Thread(Ok(test.0), err.to_string()),
            Handed::LoadThread(path, err) => Seen::Thread(Err(path), err.to_string()),
            Handed::Stopped { .. } | Handed::Ended { .. } | Handed::Unreached(..) => Seen::Other,
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
            Seen::Test("test_releases", Ok(Kind::Passed)),
            Seen::Thread(
                Err(PathBuf::from("test_leaves.py")),
                "thread M raised ValueError: late".to_owned()
            ),
        ],
        "the load's thread error before the first test, and the error of a thread that \
         raises after its test returns under that test, before the next test. A thread \
         that the load left alive, and that raises during a test, charges the module"
    );
}
