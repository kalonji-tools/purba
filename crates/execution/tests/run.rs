use std::path::PathBuf;

use purba_execution::{Error, ErrorKind, Kind, Module, Outcome, Rewrite};

fn module(name: &str, source: &str) -> Module {
    Module::new(
        PathBuf::from(name),
        Rewrite::of_source(source).expect("the source parses"),
    )
}

fn kinds(module: &Module, functions: &[&str]) -> Vec<Kind> {
    purba_execution::attach(|worker| {
        functions
            .iter()
            .map(|function| {
                worker
                    .run(module, function)
                    .expect("the test runs")
                    .kind()
                    .clone()
            })
            .collect()
    })
}

#[test]
fn the_three_tests_of_test_a_pass_fail_and_are_unknown() {
    let test_a = module(
        "test_a.py",
        "def test_one():\n    assert 1 == 1\n\n\
         def test_two():\n    assert 1 == 2\n\n\
         def test_three():\n    pass\n",
    );
    assert_eq!(
        kinds(&test_a, &["test_one", "test_two", "test_three"]),
        [
            Kind::Passed,
            Kind::Failed {
                assert: "assert 1 == 2".to_owned()
            },
            Kind::Unknown,
        ],
        "one outcome for each test, in order",
    );
}

#[test]
fn a_module_level_assert_leaves_a_test_unknown() {
    let test_level = module(
        "test_level.py",
        "assert True\n\ndef test_none():\n    pass\n",
    );
    assert_eq!(
        kinds(&test_level, &["test_none"]),
        [Kind::Unknown],
        "the module-level assert ran before the test's call",
    );
}

#[test]
fn a_worker_loads_each_module_once() {
    let test_once = module(
        "test_once.py",
        "calls = []\n\n\
         def test_first():\n    calls.append(1)\n    assert calls == [1]\n\n\
         def test_second():\n    calls.append(2)\n    assert calls == [1, 2]\n",
    );
    assert_eq!(
        kinds(&test_once, &["test_first", "test_second"]),
        [Kind::Passed, Kind::Passed],
        "the second test sees the list that the first test changed",
    );
}

#[test]
fn another_exception_gives_raised() {
    let test_raises = module(
        "test_raises.py",
        "def test_raises():\n    raise ValueError('no')\n",
    );
    assert_eq!(
        kinds(&test_raises, &["test_raises"]),
        [Kind::Raised {
            exception: "ValueError: no".to_owned()
        }],
        "Python's exception as text"
    );
}

#[test]
fn a_name_that_is_not_callable_is_an_error() {
    let test_bound = module(
        "test_bound.py",
        "import collections\n\n\
         def test_bound():\n    pass\n\ntest_bound = 5\n\n\
         def test_deque():\n    pass\n\ntest_deque = collections.deque()\n",
    );
    let errors: Vec<String> = purba_execution::attach(|worker| {
        ["test_bound", "test_deque"]
            .iter()
            .map(|function| {
                worker
                    .run(&test_bound, function)
                    .expect_err("an object that is not callable is not a test")
                    .to_string()
            })
            .collect()
    });
    assert_eq!(
        errors,
        [
            "TypeError: 'int' object is not callable",
            "TypeError: 'collections.deque' object is not callable",
        ],
        "CPython's own text, which names a type by its full name, and no test's call ran"
    );
}

#[test]
fn an_outcome_is_data() {
    fn data<T: serde::Serialize + serde::de::DeserializeOwned>() {}
    data::<Outcome>();
}

#[test]
fn a_module_that_fails_to_load_runs_its_body_once() {
    let loads = module(
        "test_fails.py",
        "import builtins\nbuiltins.purba_loads = getattr(builtins, 'purba_loads', 0) + 1\n\
         raise ValueError(f'load {builtins.purba_loads}')\n\n\
         def test_one():\n    pass\n\ndef test_two():\n    pass\n",
    );
    let errors: Vec<String> = purba_execution::attach(|worker| {
        ["test_one", "test_two"]
            .iter()
            .map(|function| {
                worker
                    .run(&loads, function)
                    .expect_err("the module does not load")
                    .to_string()
            })
            .collect()
    });
    assert_eq!(
        errors,
        ["ValueError: load 1", "test_fails.py did not load"],
        "the second test names the module, and the body runs once",
    );
}

#[test]
fn a_load_that_fails_gives_its_error_once() {
    let fails = module(
        "test_load_fails.py",
        "import builtins\n\
         builtins.purba_load_fails = getattr(builtins, 'purba_load_fails', 0) + 1\n\
         raise ValueError(f'load {builtins.purba_load_fails}')\n\n\
         def test_one():\n    pass\n",
    );
    let errors: Vec<String> = purba_execution::attach(|worker| {
        vec![
            worker
                .load(&fails)
                .expect_err("the module raises")
                .to_string(),
            worker
                .load(&fails)
                .expect_err("the module did not load")
                .to_string(),
            worker
                .run(&fails, "test_one")
                .expect_err("the module did not load")
                .to_string(),
        ]
    });
    assert_eq!(
        errors,
        [
            "ValueError: load 1",
            "test_load_fails.py did not load",
            "test_load_fails.py did not load",
        ],
        "the load's error once, then the module's name, and the body runs once",
    );
}

#[test]
fn a_test_runs_on_the_module_that_load_loaded() {
    let loads = module(
        "test_load_once.py",
        "import builtins\n\
         builtins.purba_load_once = getattr(builtins, 'purba_load_once', 0) + 1\n\n\
         def test_one():\n    assert builtins.purba_load_once == 1\n",
    );
    let kind = purba_execution::attach(|worker| {
        worker.load(&loads).expect("the module loads");
        worker
            .load(&loads)
            .expect("a second load keeps the first module");
        worker
            .run(&loads, "test_one")
            .expect("the test runs")
            .kind()
            .clone()
    });
    assert_eq!(
        kind,
        Kind::Passed,
        "the body ran once, for both loads and the test"
    );
}

#[cfg(unix)]
#[test]
fn a_forked_child_exits_with_its_own_code() {
    let test_fork = module(
        "test_fork.py",
        "import os\n\n\
         def test_forks():\n    pid = os.fork()\n    if pid == 0:\n        os._exit(7)\n\
         \x20   _, status = os.waitpid(pid, 0)\n\
         \x20   assert os.waitstatus_to_exitcode(status) == 7\n\n\
         def test_forks_with_a_keyword():\n    pid = os.fork()\n    if pid == 0:\n\
         \x20       os._exit(status=7)\n\
         \x20   _, status = os.waitpid(pid, 0)\n\
         \x20   assert os.waitstatus_to_exitcode(status) == 7\n",
    );
    assert_eq!(
        kinds(&test_fork, &["test_forks", "test_forks_with_a_keyword"]),
        [Kind::Passed, Kind::Passed],
        "the guard calls the real os._exit in a process that a test forks",
    );
}

#[test]
fn an_assertion_error_after_an_assert_that_held_gives_raised() {
    let test_helper = module(
        "test_helper.py",
        "def test_helper():\n    assert 1 == 1\n\
         \x20   raise AssertionError('from a helper')\n",
    );
    assert_eq!(
        kinds(&test_helper, &["test_helper"]),
        [Kind::Raised {
            exception: "AssertionError: from a helper".to_owned()
        }],
        "no assert failed, so the test raised"
    );
}

#[test]
fn an_assertion_error_before_any_assert_gives_raised() {
    let test_bare = module(
        "test_bare.py",
        "def test_bare():\n    raise AssertionError('no assert ran')\n",
    );
    assert_eq!(
        kinds(&test_bare, &["test_bare"]),
        [Kind::Raised {
            exception: "AssertionError: no assert ran".to_owned()
        }],
        "no assert ran, so the test raised"
    );
}

/// The text of a project error, or `internal error: <part>: <message>`.
fn reason(error: &Error) -> String {
    match error.kind() {
        ErrorKind::Project => error.to_string(),
        ErrorKind::Internal { part } => format!("internal error: {part}: {error}"),
    }
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
fn purbas_own_panic_exception_gives_an_internal_error_and_the_next_test_runs() {
    let test_panic = module(
        "test_panic_exception.py",
        &format!(
            "{PANIC_EXCEPTION}\
             def test_one():\n    raise panic_exception()('raised by the test')\n\n\
             def test_two():\n    assert True\n\n\
             test_three = 5\n"
        ),
    );
    let results: Vec<Result<Kind, String>> = purba_execution::attach(|worker| {
        ["test_one", "test_two", "test_three"]
            .iter()
            .map(|function| {
                worker
                    .run(&test_panic, function)
                    .map(|outcome| outcome.kind().clone())
                    .map_err(|error| reason(&error))
            })
            .collect()
    });
    assert_eq!(
        results,
        [
            Err("internal error: execution: raised by the test".to_owned()),
            Ok(Kind::Passed),
            Err("TypeError: 'int' object is not callable".to_owned()),
        ],
        "the panic that crosses the call is an internal error of execution, the next test \
         passes, and a project error names no part"
    );
}
