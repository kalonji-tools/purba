use std::fs;
use std::ops::ControlFlow;
use std::path::PathBuf;

use purba_dispatch::{Handed, Module, Test};
use purba_execution::{Error, ErrorKind, Kind, Quote, Rewrite, Shown};

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
        Rewrite::of_source(source, &tests_in(source)).expect("the source parses"),
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
            Handed::Test(test, result, _) => Seen::Test(
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
                    asserts: vec![Quote {
                        assert: "assert 1 == 2".to_owned(),
                        shown: Some(Shown::Values {
                            expected: "2".to_owned(),
                            reading: "1".to_owned(),
                        }),
                    }]
                })
            ),
            test("test_three", Err("[no-expectation] no expectation stated")),
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
fn each_test_reads_the_module_as_its_load_left_it() {
    let test_shared = module(
        "test_shared.py",
        "seen = []\n\n\
         def test_first():\n    seen.append(1)\n    assert seen == [1]\n\n\
         def test_second():\n    seen.append(2)\n    assert seen == [2]\n",
        &["test_first", "test_second"],
    );
    assert_eq!(
        seen(test_shared),
        [
            test("test_first", Ok(Kind::Passed)),
            test("test_second", Ok(Kind::Passed))
        ],
        "test_second does not see what test_first appended",
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
            if let Handed::Test(test, _, _) = handed {
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
        if let Handed::Test(test, _, _) = given {
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
fn a_load_that_leaves_a_thread_alive_hands_its_threads_error_once() {
    let test_twice = module(
        "test_twice.py",
        "import threading\n\n\
         loader = threading.Thread(target=lambda: 1 / 0, name=\"L\")\n\
         loader.start()\nloader.join()\n\
         threading.Thread(target=threading.Event().wait, daemon=True).start()\n\n\
         def test_one():\n    assert True\n\n\
         def test_two():\n    assert True\n",
        &["test_one", "test_two"],
    );
    assert_eq!(
        seen(test_twice),
        [
            Seen::Thread("thread L raised ZeroDivisionError: division by zero".to_owned()),
            test("test_one", Ok(Kind::Passed)),
            test("test_two", Ok(Kind::Passed)),
        ],
        "the zygote's load hands the error once, and the load in each test's process \
         hands it again to no test",
    );
}

#[test]
fn a_module_whose_load_exits_hands_its_exit_code_once_then_each_test_as_blocked() {
    let test_exits = module(
        "test_load_exits.py",
        "import os\n\n\
         try:\n    os._exit(5)\nexcept BaseException:\n    pass\n\n\
         def test_one():\n    pass\n",
        &["test_one"],
    );
    let path = PathBuf::from("test_load_exits.py");
    assert_eq!(
        seen(test_exits),
        [
            Seen::NotLoaded(
                path.clone(),
                "its process exited 5 before its load ended".to_owned()
            ),
            Seen::Blocked("test_one", path),
        ],
        "a call to os._exit ends the zygote, though the module catches every exception",
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
                "internal error: execution: raised while it loads".to_owned()
            ),
            Seen::Blocked("test_one", path.clone()),
            Seen::Blocked("test_two", path),
        ],
        "execution catches the panic of the load in the zygote, and names itself",
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
                Err("internal error: execution: raised by __getattr__")
            ),
            test("test_two", Ok(Kind::Passed)),
        ],
        "the lookup of test_gone panics before its call, execution names itself, and \
         test_two runs",
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

#[test]
fn what_a_failed_expectation_shows_crosses_the_fork_whole() {
    let test_shown = module(
        "test_shown.py",
        "import contextlib\n\n\
         class Nul:\n    def __repr__(self):\n        return 'a\\x00b'\n\n\
         @contextlib.contextmanager\ndef leaks():\n    yield\n\
         \x20   assert False, 'a handle leaked'\n\n\
         def test_nul():\n    got = Nul()\n    assert got == ''\n    assert got == 'x'\n\n\
         def test_cleanup():\n    with leaks():\n        got = 1\n    assert got == 1\n",
        &["test_nul", "test_cleanup"],
    );
    let shown = |assert: &str, expected: &str| Quote {
        assert: assert.to_owned(),
        shown: Some(Shown::Values {
            expected: expected.to_owned(),
            reading: "a\0b".to_owned(),
        }),
    };
    assert_eq!(
        seen(test_shown),
        [
            test(
                "test_nul",
                Ok(Kind::Failed {
                    asserts: vec![
                        shown("assert got == ''", "''"),
                        shown("assert got == 'x'", "'x'"),
                    ]
                })
            ),
            test(
                "test_cleanup",
                Ok(Kind::Failed {
                    asserts: vec![Quote {
                        assert: "the cleanup of the with at line 18 raised AssertionError: \
                                 a handle leaked"
                            .to_owned(),
                        shown: None,
                    }]
                })
            ),
        ],
        "a reading that holds a NUL, each of two failed expectations, and a quote with no \
         reading reach dispatch as the test's process made them",
    );
}

#[test]
fn the_items_of_a_failed_any_and_the_frame_of_a_raise_cross_the_fork_whole() {
    let test_cross = module(
        "test_cross.py",
        "def gen():\n    yield 1\n    raise ValueError('boom')\n\n\
         class Nul:\n    def __repr__(self):\n        return 'a\\x00b'\n\n\
         def test_items():\n    xs = [Nul(), Nul()]\n    assert any(x == 'z' for x in xs)\n\n\
         def test_whence():\n    list(gen())\n",
        &["test_items", "test_whence"],
    );
    assert_eq!(
        seen(test_cross),
        [
            test(
                "test_items",
                Ok(Kind::Failed {
                    asserts: vec![Quote {
                        assert: "assert any(x == 'z' for x in xs)".to_owned(),
                        shown: Some(Shown::Items {
                            heading: "none of 2 items holds  x == 'z'".to_owned(),
                            items: vec!["[0]  a\0b".to_owned(), "[1]  a\0b".to_owned()],
                        }),
                    }]
                })
            ),
            test(
                "test_whence",
                Ok(Kind::Raised {
                    exception: "ValueError: boom".to_owned(),
                    whence: Some("raised inside the generator gen() at test_cross.py:3".to_owned()),
                })
            ),
        ],
        "the heading and each item, a NUL in an item, and the frame of a raise reach dispatch as \
         the test's process made them",
    );
}

/// A test that carries its suppressions: the name of each check, and its reason.
#[derive(Clone)]
struct Suppressing(&'static str, &'static [(&'static str, &'static str)]);

impl Test for Suppressing {
    fn function(&self) -> &str {
        self.0
    }

    fn suppressions(&self) -> Vec<(&str, &str)> {
        self.1.to_vec()
    }
}

#[test]
fn a_test_takes_its_suppressions_into_its_process_and_its_warnings_cross_back() {
    let source = "def test_act():\n    data = 1\n    assert data == 1\n    got = data + 1\n    \
                  assert got == 2\n\n\
                  def test_plain():\n    data = 1\n    assert data == 1\n    got = data + 1\n    \
                  assert got == 2\n";
    let module = Module::new(
        PathBuf::from("test_act.py"),
        Rewrite::of_source(source, &tests_in(source)).expect("the source parses"),
        vec![
            Suppressing(
                "test_act",
                &[("assert-before-expectation", "two\nlines and a \0 NUL")],
            ),
            Suppressing("test_plain", &[]),
        ],
    );
    let mut seen = Vec::new();

    let flow: ControlFlow<()> = purba_dispatch::hand(vec![module], |handed| {
        if let Handed::Test(test, result, suppressed) = handed {
            seen.push((
                test.0,
                result
                    .map(|outcome| outcome.kind().clone())
                    .map_err(|err| reason(&err)),
                suppressed
                    .warnings
                    .iter()
                    .map(|warning| {
                        (
                            warning.check.clone(),
                            warning.words.clone(),
                            warning.reason.clone(),
                        )
                    })
                    .collect::<Vec<_>>(),
            ));
        }
        ControlFlow::Continue(())
    });

    assert_eq!(flow, ControlFlow::Continue(()), "every test runs");
    assert_eq!(
        seen,
        [
            (
                "test_act",
                Ok(Kind::Passed),
                vec![(
                    "assert-before-expectation".to_owned(),
                    "line 3: an assert ran in the act; one test checks one state; \
                     check this state in a test of its own"
                        .to_owned(),
                    "two\nlines and a \0 NUL".to_owned(),
                )],
            ),
            (
                "test_plain",
                Err(
                    "[assert-before-expectation] line 9: an assert ran in the act; \
                     one test checks one state; check this state in a test of its own"
                        .to_owned()
                ),
                Vec::new(),
            ),
        ],
        "the suppression reaches its own test's process whole, and its warning crosses back"
    );
}
