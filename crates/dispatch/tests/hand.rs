use std::fs;
use std::ops::ControlFlow;
use std::path::PathBuf;

use purba_dispatch::{Module, Test};
use purba_execution::Kind;

struct Named(&'static str);

impl Test for Named {
    fn function(&self) -> &str {
        self.0
    }
}

fn module(name: &str, text: &str, asserts: &[&str], tests: &[&'static str]) -> Module<Named> {
    Module::new(
        PathBuf::from(name),
        text.to_owned(),
        asserts.iter().map(|&assert| assert.to_owned()).collect(),
        tests.iter().map(|&test| Named(test)).collect(),
    )
}

fn seen(module: Module<Named>) -> Vec<(&'static str, Result<Kind, String>)> {
    let mut seen = Vec::new();
    let flow: ControlFlow<()> = purba_dispatch::hand(vec![module], |test, result| {
        seen.push((
            test.0,
            result
                .map(|outcome| outcome.kind().clone())
                .map_err(|err| err.to_string()),
        ));
        ControlFlow::Continue(())
    });
    assert_eq!(
        flow,
        ControlFlow::Continue(()),
        "a function that never breaks lets every test run"
    );
    seen
}

fn ran_at(name: &str) -> PathBuf {
    let ran = PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join(name);
    let _ = fs::remove_file(&ran);
    ran
}

/// A module whose first test passes, and whose second test creates `ran`.
fn two_tests(name: &str, ran: &std::path::Path) -> Module<Named> {
    module(
        name,
        &format!(
            "def test_one():\n    assert _purba_ran(0, True)\n\n\
             def test_two():\n    open({:?}, \"w\").close()\n",
            ran.to_str().expect("the path is UTF-8")
        ),
        &["assert True"],
        &["test_one", "test_two"],
    )
}

#[test]
fn each_outcome_stands_beside_its_test() {
    let test_a = module(
        "test_a.py",
        "def test_one():\n    assert _purba_ran(0, 1 == 1)\n\n\
         def test_two():\n    assert _purba_ran(1, 1 == 2)\n\n\
         def test_three():\n    pass\n",
        &["assert 1 == 1", "assert 1 == 2"],
        &["test_one", "test_two", "test_three"],
    );
    assert_eq!(
        seen(test_a),
        [
            ("test_one", Ok(Kind::Passed)),
            (
                "test_two",
                Ok(Kind::Failed {
                    assert: "assert 1 == 2".to_owned()
                })
            ),
            ("test_three", Ok(Kind::Unknown)),
        ],
        "each outcome beside its test, in order",
    );
}

#[test]
fn dispatch_calls_only_the_tests_it_is_handed() {
    let test_only = module(
        "test_only.py",
        "ran = []\n\ndef test_left_out():\n    ran.append(1)\n\n\
         def test_check():\n    assert _purba_ran(0, ran == [])\n",
        &["assert ran == []"],
        &["test_check"],
    );
    assert_eq!(
        seen(test_only),
        [("test_check", Ok(Kind::Passed))],
        "the left-out test never ran",
    );
}

#[test]
fn the_tests_of_a_module_share_one_worker() {
    let test_shared = module(
        "test_shared.py",
        "seen = []\n\n\
         def test_first():\n    seen.append(1)\n    assert _purba_ran(0, seen == [1])\n\n\
         def test_second():\n    seen.append(2)\n    assert _purba_ran(1, seen == [1, 2])\n",
        &["assert seen == [1]", "assert seen == [1, 2]"],
        &["test_first", "test_second"],
    );
    assert_eq!(
        seen(test_shared),
        [
            ("test_first", Ok(Kind::Passed)),
            ("test_second", Ok(Kind::Passed))
        ],
        "the module loaded once for both tests",
    );
}

#[test]
fn an_error_stands_beside_its_test_and_the_run_goes_on() {
    let test_err = module(
        "test_err.py",
        "def test_one():\n    pass\n\ntest_one = 5\n\n\
         def test_two():\n    assert _purba_ran(0, True)\n",
        &["assert True"],
        &["test_one", "test_two"],
    );
    assert_eq!(
        seen(test_err),
        [
            (
                "test_one",
                Err("TypeError: 'int' object is not callable".to_owned())
            ),
            ("test_two", Ok(Kind::Passed)),
        ],
        "the error beside its test, and the next test runs",
    );
}

#[test]
fn dispatch_hands_each_test_before_it_runs_the_next() {
    let ran = ran_at("test_dispatch_before.ran");
    let mut ran_before = Vec::new();
    let flow: ControlFlow<()> =
        purba_dispatch::hand(vec![two_tests("test_before.py", &ran)], |test, _| {
            ran_before.push((test.0, ran.exists()));
            ControlFlow::Continue(())
        });
    assert_eq!(flow, ControlFlow::Continue(()), "no test breaks");
    assert_eq!(
        ran_before,
        [("test_one", false), ("test_two", true)],
        "dispatch hands test_one before test_two runs"
    );
}

#[test]
fn dispatch_calls_no_test_after_a_break() {
    let ran = ran_at("test_dispatch_break.ran");
    let mut handed = Vec::new();
    let flow = purba_dispatch::hand(vec![two_tests("test_break.py", &ran)], |test, _| {
        handed.push(test.0);
        ControlFlow::Break("stop")
    });
    assert_eq!(flow, ControlFlow::Break("stop"), "hand returns the break");
    assert_eq!(
        handed,
        ["test_one"],
        "dispatch hands no test after the break"
    );
    assert!(!ran.exists(), "test_two never ran");
}
