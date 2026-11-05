use std::fs;
use std::ops::ControlFlow;
use std::path::PathBuf;

use purba_dispatch::{Handed, Module, Test};
use purba_execution::{Error, ErrorKind, Kind, Rewrite};

#[derive(Clone)]
struct Named(&'static str);

impl Test for Named {
    fn function(&self) -> &str {
        self.0
    }
}

fn module(name: &str, source: &str, tests: &[&'static str]) -> Module<Named> {
    Module::new(
        PathBuf::from(name),
        Rewrite::of_source(source).expect("the source parses"),
        tests.iter().map(|&test| Named(test)).collect(),
    )
}

/// What dispatch hands on, as data a test can compare.
#[derive(Debug, PartialEq)]
enum Seen {
    Test(&'static str, Result<Kind, String>),
    NotLoaded(PathBuf, String),
    Blocked(&'static str, PathBuf),
    Thread(String),
    Unreached(&'static str),
}

/// The text of a project error, or `internal error: <part>: <message>`.
fn reason(error: &Error) -> String {
    match error.kind() {
        ErrorKind::Project => error.to_string(),
        ErrorKind::Internal { part } => format!("internal error: {part}: {error}"),
    }
}

fn test(name: &'static str, result: Result<Kind, &str>) -> Seen {
    Seen::Test(name, result.map_err(str::to_owned))
}

fn seen(module: Module<Named>) -> Vec<Seen> {
    let mut seen = Vec::new();
    let flow: ControlFlow<()> = purba_dispatch::hand(vec![module], |handed| {
        seen.push(match handed {
            Handed::Test(test, result) => Seen::Test(
                test.0,
                result
                    .map(|outcome| outcome.kind().clone())
                    .map_err(|err| reason(&err)),
            ),
            Handed::NotLoaded(path, err) => Seen::NotLoaded(path, reason(&err)),
            Handed::Blocked(test, path) => Seen::Blocked(test.0, path),
            Handed::Thread(_, err) => Seen::Thread(reason(&err)),
            Handed::Unreached(test) => Seen::Unreached(test.0),
        });
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
            "def test_one():\n    assert True\n\n\
             def test_two():\n    open({:?}, \"w\").close()\n",
            ran.to_str().expect("the path is UTF-8")
        ),
        &["test_one", "test_two"],
    )
}

#[test]
fn each_outcome_stands_beside_its_test() {
    let test_a = module(
        "test_a.py",
        "def test_one():\n    assert 1 == 1\n\n\
         def test_two():\n    assert 1 == 2\n\n\
         def test_three():\n    pass\n",
        &["test_one", "test_two", "test_three"],
    );
    assert_eq!(
        seen(test_a),
        [
            test("test_one", Ok(Kind::Passed)),
            test(
                "test_two",
                Ok(Kind::Failed {
                    assert: "assert 1 == 2".to_owned()
                })
            ),
            test("test_three", Ok(Kind::Unknown)),
        ],
        "each outcome beside its test, in order",
    );
}

#[test]
fn dispatch_calls_only_the_tests_it_is_handed() {
    let test_only = module(
        "test_only.py",
        "ran = []\n\ndef test_left_out():\n    ran.append(1)\n\n\
         def test_check():\n    assert ran == []\n",
        &["test_check"],
    );
    assert_eq!(
        seen(test_only),
        [test("test_check", Ok(Kind::Passed))],
        "the left-out test never ran",
    );
}

#[test]
fn the_tests_of_a_module_share_one_worker() {
    let test_shared = module(
        "test_shared.py",
        "seen = []\n\n\
         def test_first():\n    seen.append(1)\n    assert seen == [1]\n\n\
         def test_second():\n    seen.append(2)\n    assert seen == [1, 2]\n",
        &["test_first", "test_second"],
    );
    assert_eq!(
        seen(test_shared),
        [
            test("test_first", Ok(Kind::Passed)),
            test("test_second", Ok(Kind::Passed))
        ],
        "the module loaded once for both tests",
    );
}

#[test]
fn an_error_stands_beside_its_test_and_the_run_goes_on() {
    let test_err = module(
        "test_err.py",
        "def test_one():\n    pass\n\ntest_one = 5\n\n\
         def test_two():\n    assert True\n",
        &["test_one", "test_two"],
    );
    assert_eq!(
        seen(test_err),
        [
            test("test_one", Err("TypeError: 'int' object is not callable")),
            test("test_two", Ok(Kind::Passed)),
        ],
        "the error beside its test, and the next test runs",
    );
}

#[test]
fn dispatch_hands_each_test_before_it_runs_the_next() {
    let ran = ran_at("test_dispatch_before.ran");
    let mut ran_before = Vec::new();
    let flow: ControlFlow<()> =
        purba_dispatch::hand(vec![two_tests("test_before.py", &ran)], |handed| {
            if let Handed::Test(test, _) = handed {
                ran_before.push((test.0, ran.exists()));
            }
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
    let flow = purba_dispatch::hand(vec![two_tests("test_break.py", &ran)], |given| {
        if let Handed::Test(test, _) = given {
            handed.push(test.0);
        }
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

#[test]
fn a_module_that_does_not_load_hands_its_error_once_then_each_test_as_blocked() {
    let test_fails = module(
        "test_not_loaded.py",
        "raise ValueError('x')\n\n\
         def test_one():\n    pass\n\n\
         def test_two():\n    pass\n",
        &["test_one", "test_two"],
    );
    let path = PathBuf::from("test_not_loaded.py");
    assert_eq!(
        seen(test_fails),
        [
            Seen::NotLoaded(path.clone(), "ValueError: x".to_owned()),
            Seen::Blocked("test_one", path.clone()),
            Seen::Blocked("test_two", path),
        ],
        "the load's error once, and each test as blocked",
    );
}

#[test]
fn a_break_at_a_module_that_does_not_load_hands_no_blocked_test() {
    let test_fails = module(
        "test_not_loaded_break.py",
        "raise ValueError('x')\n\ndef test_one():\n    pass\n",
        &["test_one"],
    );
    let mut handed = 0;
    let flow = purba_dispatch::hand(vec![test_fails], |_| {
        handed += 1;
        ControlFlow::Break("stop")
    });
    assert_eq!(flow, ControlFlow::Break("stop"), "hand returns the break");
    assert_eq!(handed, 1, "dispatch hands nothing after the break");
}

#[test]
fn dispatch_loads_no_module_that_has_no_test() {
    let ran = ran_at("test_dispatch_no_test.ran");
    let test_none = module(
        "test_no_test.py",
        &format!(
            "open({:?}, \"w\").close()\n",
            ran.to_str().expect("the path is UTF-8")
        ),
        &[],
    );
    assert_eq!(seen(test_none), [], "dispatch hands on nothing");
    assert!(!ran.exists(), "the module's body never ran");
}

/// Python that finds `PyO3`'s `PanicException`, which no module exports.
const PANIC_EXCEPTION: &str = "def panic_exception():\n\
    \x20   pending = [BaseException]\n\
    \x20   while pending:\n\
    \x20       kind = pending.pop()\n\
    \x20       if kind.__module__ == 'pyo3_runtime' and kind.__name__ == 'PanicException':\n\
    \x20           return kind\n\
    \x20       pending.extend(kind.__subclasses__())\n\
    \x20   raise LookupError('no PanicException')\n\n";

#[test]
fn a_module_whose_load_panics_hands_an_internal_error_once_then_each_test_as_blocked() {
    let test_panics = module(
        "test_load_panics.py",
        &format!(
            "{PANIC_EXCEPTION}\
             raise panic_exception()('raised while it loads')\n\n\
             def test_one():\n    pass\n\n\
             def test_two():\n    pass\n"
        ),
        &["test_one", "test_two"],
    );
    let path = PathBuf::from("test_load_panics.py");
    assert_eq!(
        seen(test_panics),
        [
            Seen::NotLoaded(
                path.clone(),
                "internal error: dispatch: raised while it loads".to_owned()
            ),
            Seen::Blocked("test_one", path.clone()),
            Seen::Blocked("test_two", path),
        ],
        "dispatch catches the panic of the load, and names itself",
    );
}

#[test]
fn a_panic_outside_a_tests_call_gives_that_test_an_internal_error_and_the_next_test_runs() {
    let test_getattr = module(
        "test_getattr_panics.py",
        &format!(
            "{PANIC_EXCEPTION}\
             def __getattr__(name):\n    raise panic_exception()('raised by __getattr__')\n\n\
             def test_two():\n    assert True\n"
        ),
        &["test_gone", "test_two"],
    );
    assert_eq!(
        seen(test_getattr),
        [
            test(
                "test_gone",
                Err("internal error: dispatch: raised by __getattr__")
            ),
            test("test_two", Ok(Kind::Passed)),
        ],
        "the lookup of test_gone panics before its call, dispatch names itself, and \
         test_two runs",
    );
}
