use std::path::PathBuf;

use purba_execution::{Error, ErrorKind, Kind, Module, Outcome, Quote, Rewrite, Shown};

/// The quote of an expectation that did not hold, with what it shows.
fn shown(assert: &str, expected: &str, reading: &str) -> Quote {
    Quote {
        assert: assert.to_owned(),
        shown: Some(Shown {
            expected: expected.to_owned(),
            reading: reading.to_owned(),
        }),
    }
}

fn module(name: &str, source: &str) -> Module {
    Module::new(
        PathBuf::from(name),
        Rewrite::of_source(source, &tests_in(source)).expect("the source parses"),
    )
}

fn results(module: &Module, functions: &[&str]) -> Vec<Result<Kind, String>> {
    purba_execution::attach(|worker| {
        worker.load(module).expect("the module loads");
        functions
            .iter()
            .map(|function| {
                worker
                    .run(module, function, &[], *function)
                    .map(|outcome| outcome.kind().clone())
                    .map_err(|error| error.to_string())
            })
            .collect()
    })
}

fn kinds(module: &Module, functions: &[&str]) -> Vec<Kind> {
    purba_execution::attach(|worker| {
        worker.load(module).expect("the module loads");
        functions
            .iter()
            .map(|function| {
                worker
                    .run(module, function, &[], *function)
                    .expect("the test runs")
                    .kind()
                    .clone()
            })
            .collect()
    })
}

#[test]
fn the_three_tests_of_test_a_pass_fail_and_state_no_expectation() {
    let test_a = module(
        "test_a.py",
        "def test_one():\n    assert 1 == 1\n\n\
         def test_two():\n    assert 1 == 2\n\n\
         def test_three():\n    pass\n",
    );
    assert_eq!(
        results(&test_a, &["test_one", "test_two", "test_three"]),
        [
            Ok(Kind::Passed),
            Ok(Kind::Failed {
                asserts: vec![shown("assert 1 == 2", "2", "1")]
            }),
            Err("[no-expectation] no expectation stated".to_owned()),
        ],
        "one outcome or error for each test, in order",
    );
}

#[test]
fn a_module_level_assert_is_no_expectation_of_a_test() {
    let test_level = module(
        "test_level.py",
        "assert True\n\ndef test_none():\n    pass\n",
    );
    assert_eq!(
        results(&test_level, &["test_none"]),
        [Err("[no-expectation] no expectation stated".to_owned())],
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
        worker.load(&test_bound).expect("the module loads");
        ["test_bound", "test_deque"]
            .iter()
            .map(|function| {
                worker
                    .run(&test_bound, function, &[], *function)
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
        let loaded = worker
            .load(&loads)
            .expect_err("the module raises")
            .to_string();
        std::iter::once(loaded)
            .chain(["test_one", "test_two"].iter().map(|function| {
                worker
                    .run(&loads, function, &[], *function)
                    .expect_err("the module does not load")
                    .to_string()
            }))
            .collect()
    });
    assert_eq!(
        errors,
        [
            "ValueError: load 1",
            "test_fails.py did not load",
            "test_fails.py did not load"
        ],
        "each test names the module, and the body runs once",
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
                .run(&fails, "test_one", &[], "test_one")
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
            .run(&loads, "test_one", &[], "test_one")
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

#[test]
fn a_forked_child_exits_with_its_own_code() {
    let test_fork = module(
        "test_fork.py",
        "import os\n\n\
         def test_forks():\n    pid = os.fork()\n    {0: os._exit}.get(pid, int)(7)\n\
         \x20   _, status = os.waitpid(pid, 0)\n\
         \x20   assert os.waitstatus_to_exitcode(status) == 7\n\n\
         def test_forks_with_a_keyword():\n    pid = os.fork()\n\
         \x20   {0: os._exit}.get(pid, dict)(status=7)\n\
         \x20   _, status = os.waitpid(pid, 0)\n\
         \x20   assert os.waitstatus_to_exitcode(status) == 7\n",
    );
    assert_eq!(
        kinds(&test_fork, &["test_forks", "test_forks_with_a_keyword"]),
        [Kind::Passed, Kind::Passed],
        "a process that a test forks exits with the code that it gives os._exit",
    );
}

/// The words of an assert before the first expectation, at `line`.
fn before(line: usize) -> String {
    format!(
        "line {line}: an assert ran in the act; one test checks one state; \
         check this state in a test of its own"
    )
}

/// The line of a refused assert before the first expectation, at `line`.
fn asserts_before(line: usize) -> String {
    format!("[assert-before-expectation] {}", before(line))
}

#[test]
fn an_assert_before_the_first_expectation_is_refused_whatever_the_case_does_after_it() {
    // Each row: the test, its body, and the line of its first assert.
    let rows = [
        (
            "test_held_then_raise",
            "    assert 1 == 1\n    raise AssertionError('from a helper')\n",
            2,
        ),
        (
            "test_message_raises",
            "    assert 1 == 2, undefined_name\n    return\n",
            2,
        ),
        (
            "test_caught_then_raise",
            "    try:\n        assert 1 == 2\n    except AssertionError:\n        pass\n\
             \x20   raise ValueError('boom')\n",
            3,
        ),
        (
            "test_caught_then_return",
            "    try:\n        assert 1 == 2\n    except AssertionError:\n        pass\n",
            3,
        ),
        (
            "test_cleanup_fails",
            "    try:\n        assert 1 + 1 == 3\n    finally:\n        assert 2 + 2 == 5\n",
            3,
        ),
        (
            "test_caught_held_raise",
            "    try:\n        assert 1 == 2\n    except AssertionError:\n        pass\n\
             \x20   assert 2 == 2\n    raise AssertionError('from a helper')\n",
            3,
        ),
    ];
    for (name, body, line) in rows {
        let test = module(&format!("{name}.py"), &format!("def {name}():\n{body}"));
        assert_eq!(
            results(&test, &[name]),
            [Err(asserts_before(line))],
            "{name}: the first assert that ran is refused, ahead of a raise or a later assert"
        );
    }
}

#[test]
fn an_assert_of_a_helper_names_the_line_of_the_test_that_runs_it() {
    let test_reach = module(
        "test_reach.py",
        "def check(user):\n    if user:\n        assert user == 'ann'\n\n\
         def test_reach():\n    check('')\n    check('ann')\n    check('')\n    assert 1 == 1\n",
    );
    assert_eq!(
        results(&test_reach, &["test_reach"]),
        [Err(
            "[assert-before-expectation] line 7 reaches line 3 of check, \
             which ran an assert in the act; one test checks one state; \
             check this state in a test of its own"
                .to_owned()
        )],
        "line 6 reaches check first, and line 7 runs its assert"
    );
}

#[test]
fn an_assert_inside_a_reading_is_refused_whether_it_holds_or_not() {
    for (name, user) in [("test_holds", "ann"), ("test_fails", "bob")] {
        let test = module(
            &format!("{name}.py"),
            &format!(
                "def check(user):\n    assert user == 'ann'\n\n\
                 def {name}():\n    assert check('{user}') is None\n"
            ),
        );
        assert_eq!(
            results(&test, &[name]),
            [Err(
                "[assert-in-reading] line 5 reaches line 2 of check, which asserts inside a \
                  reading, and only an expectation asserts"
                    .to_owned()
            )],
            "{name}: execution refuses the assert of check, and does not read its result"
        );
    }
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

#[test]
fn the_message_of_an_expectation_is_never_evaluated() {
    let test_message = module(
        "test_expected_message.py",
        "def test_expected_message():\n    assert 1 == 2, undefined_name\n",
    );
    assert_eq!(
        kinds(&test_message, &["test_expected_message"]),
        [Kind::Failed {
            asserts: vec![shown("assert 1 == 2, undefined_name", "2", "1")]
        }],
        "the expectation did not hold, and its message never ran"
    );
}

#[test]
fn an_expectation_inside_a_checking_context_fails_and_an_assert_in_a_try_is_refused() {
    let test_cleanup = module(
        "test_cleanup_assert.py",
        "import contextlib\n\nopened = []\n\n\
         @contextlib.contextmanager\ndef no_leaks():\n    try:\n        yield\n\
         \x20   finally:\n        assert not opened, 'a handle leaked'\n\n\
         def test_inside_a_checking_context():\n    with no_leaks():\n\
         \x20       assert 1 + 1 == 3\n\n\
         def test_with_a_finally_check():\n    try:\n        assert 1 + 1 == 3\n\
         \x20   finally:\n        assert not opened\n",
    );
    assert_eq!(
        results(
            &test_cleanup,
            &[
                "test_inside_a_checking_context",
                "test_with_a_finally_check"
            ]
        ),
        [
            Ok(Kind::Failed {
                asserts: vec![shown("assert 1 + 1 == 3", "3", "2")],
            }),
            Err(asserts_before(18)),
        ],
        "the assert at the end of the with's body is an expectation, and its cleanup's assert \
         keeps its mark; the assert in the try runs before any expectation"
    );
}

#[test]
fn a_with_whose_cleanup_raises_fails_and_names_the_with() {
    let test_with = module(
        "test_with_cleanup.py",
        "import contextlib\n\nopened = ['a handle']\n\n\
         @contextlib.contextmanager\ndef no_leaks():\n    yield\n\
         \x20   assert opened == [], 'a handle leaked'\n\n\
         def test_cleanup():\n    with no_leaks():\n        total = 1\n\
         \x20   assert total == 1\n",
    );
    assert_eq!(
        kinds(&test_with, &["test_cleanup"]),
        [Kind::Failed {
            asserts: vec![Quote {
                assert: "the cleanup of the with at line 11 raised AssertionError: a handle leaked"
                    .to_owned(),
                shown: None,
            }],
        }],
        "the cleanup ran before the first expectation, and its assert keeps its mark"
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
        worker.load(&test_panic).expect("the module loads");
        ["test_one", "test_two", "test_three"]
            .iter()
            .map(|function| {
                worker
                    .run(&test_panic, function, &[], *function)
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

/// The name of each `def test_…` at the top of `source`, as selection names
/// the tests of a module.
fn tests_in(source: &str) -> Vec<&str> {
    source
        .lines()
        .filter_map(|line| line.strip_prefix("def test_").map(|_| &line[4..]))
        .filter_map(|line| line.split_once('(').map(|(name, _)| name))
        .collect()
}

/// The words, the check and the reason of each warning that a run gave.
type Warned = Vec<(String, String, String)>;

/// The outcome or the error of each of `functions`, each run under
/// `suppressions`, beside the warnings of its run.
fn suppressed_runs(
    module: &Module,
    functions: &[&str],
    suppressions: &[(&str, &str)],
) -> Vec<(Result<Kind, String>, Warned)> {
    purba_execution::attach(|worker| {
        worker.load(module).expect("the module loads");
        functions
            .iter()
            .map(|function| {
                let result = worker
                    .run(module, function, suppressions, *function)
                    .map(|outcome| outcome.kind().clone())
                    .map_err(|error| error.to_string());
                let warned = worker
                    .warnings()
                    .iter()
                    .map(|warning| {
                        (
                            warning.check.clone(),
                            warning.words.clone(),
                            warning.reason.clone(),
                        )
                    })
                    .collect();
                (result, warned)
            })
            .collect()
    })
}

/// One warning of `check`, with its `words` and its `reason`.
fn warned(check: &str, words: &str, reason: &str) -> Warned {
    vec![(check.to_owned(), words.to_owned(), reason.to_owned())]
}

#[test]
fn a_suppressed_assert_before_the_first_expectation_gives_the_outcome_without_the_check() {
    let test_act = module(
        "test_act_suppressed.py",
        "def test_holds():\n    data = 1\n    assert data == 1\n    got = data + 1\n    \
         assert got == 2\n\n\
         def test_fails():\n    data = 1\n    assert data == 3\n    got = data + 1\n    \
         assert got == 2\n\n\
         def test_plain():\n    got = 1\n    assert got == 1\n",
    );

    let runs = suppressed_runs(
        &test_act,
        &["test_holds", "test_fails", "test_plain"],
        &[("assert-before-expectation", "the act checks its input")],
    );

    assert_eq!(
        runs,
        [
            (
                Ok(Kind::Passed),
                warned(
                    "assert-before-expectation",
                    &before(3),
                    "the act checks its input"
                )
            ),
            (
                Ok(Kind::Failed {
                    asserts: vec![Quote {
                        assert: "assert data == 3".to_owned(),
                        shown: None,
                    }],
                }),
                warned(
                    "assert-before-expectation",
                    &before(9),
                    "the act checks its input"
                )
            ),
            (Ok(Kind::Passed), Vec::new()),
        ],
        "the case passes, or fails on the act's assert, with a warning; \
         a case that breaks no check gives none"
    );
}

#[test]
fn a_suppressed_path_or_swallowed_raise_gives_the_outcome_without_the_check() {
    let test_paths = module(
        "test_paths_suppressed.py",
        "import contextlib\n\n\n\
         def test_branch():\n    x = 1\n    if x == 2:\n        x = 3\n    assert x == 1\n\n\n\
         def test_swallow():\n    x = 1\n    with contextlib.suppress(ValueError):\n        \
         raise ValueError(\"x\")\n        x = 2\n    assert x == 1\n",
    );

    let runs = suppressed_runs(
        &test_paths,
        &["test_branch", "test_swallow"],
        &[
            ("second-path", "the branch depends on the platform"),
            ("swallowed-raise", "the raise is the act"),
        ],
    );

    assert_eq!(
        runs,
        [
            (
                Ok(Kind::Passed),
                warned(
                    "second-path",
                    "line 6 went one way only, and the lines before the first expectation \
                     take one path; line 7 did not run, and the lines before the first \
                     expectation take one path",
                    "the branch depends on the platform"
                )
            ),
            (
                Ok(Kind::Passed),
                warned(
                    "swallowed-raise",
                    "the with at line 13 swallowed a raise, and line 15 did not run",
                    "the raise is the act"
                )
            ),
        ],
        "each case passes on its expectation, with one warning for its check"
    );
}

#[test]
fn a_check_of_the_run_names_itself_in_brackets_and_a_suppression_does_not_reach_another() {
    let test_hook = module(
        "test_hook_after_act.py",
        "import threading\n\n\n\
         def test_hook():\n    data = 1\n    assert data == 1\n    \
         threading.excepthook = print\n    assert data == 1\n\n\n\
         def test_branch():\n    x = 1\n    if x == 2:\n        x = 3\n    assert x == 1\n\n\n\
         def test_none():\n    print('only logs')\n",
    );

    let runs = suppressed_runs(
        &test_hook,
        &["test_hook", "test_branch", "test_none"],
        &[
            ("assert-before-expectation", "the act checks its input"),
            ("hook-replaced", "no suppression reaches it"),
            ("no-expectation", "no suppression reaches it"),
        ],
    );

    assert_eq!(
        runs,
        [
            (
                Err("[hook-replaced] line 7 replaced threading.excepthook, \
                     and a test keeps purba's hooks"
                    .to_owned()),
                warned(
                    "assert-before-expectation",
                    &before(6),
                    "the act checks its input"
                )
            ),
            (
                Err(
                    "[second-path] line 13 went one way only, and the lines before the first \
                     expectation take one path; [second-path] line 14 did not run, and the \
                     lines before the first expectation take one path"
                        .to_owned()
                ),
                Vec::new()
            ),
            (
                Err("[no-expectation] no expectation stated".to_owned()),
                Vec::new()
            ),
        ],
        "a fault of the form after a suppressed one still refuses; each clause names its check; \
         a check that no suppression reaches refuses whatever the run is handed"
    );
}

#[test]
fn a_case_judges_its_suppressions_only_when_it_reads_the_paths() {
    let test_judged = module(
        "test_judged.py",
        "def test_holds():\n    value = 1\n    assert value == 1\n\n\n\
         def test_fails():\n    value = 1\n    assert value == 2\n\n\n\
         def test_raises():\n    value = int(\"x\")\n    assert value == 1\n\n\n\
         def test_form():\n    value = 1\n    assert value == 1\n    value = 2\n    \
         assert value == 2\n",
    );

    let judged = purba_execution::attach(|worker| {
        worker.load(&test_judged).expect("the module loads");
        ["test_holds", "test_fails", "test_raises", "test_form"]
            .into_iter()
            .map(|function| {
                let result = worker
                    .run(
                        &test_judged,
                        function,
                        &[("second-path", "the act branches")],
                        function,
                    )
                    .map(|outcome| outcome.kind().clone())
                    .map_err(|error| error.to_string());
                (
                    function,
                    result.is_ok(),
                    worker.judged(),
                    worker.warnings().len(),
                )
            })
            .collect::<Vec<_>>()
    });

    assert_eq!(
        judged,
        [
            ("test_holds", true, true, 0),
            ("test_fails", true, true, 0),
            ("test_raises", true, false, 0),
            ("test_form", false, false, 0),
        ],
        "a case that passed or failed on its expectations read the paths; \
         a raise or a fault of the form stops before"
    );
}

#[test]
fn a_try_of_one_path_whose_statement_does_not_raise_fails_and_shows_each_failed_expectation() {
    let test_try = module(
        "test_try.py",
        "def test_missed():\n    seen = []\n    try:\n        int('1')\n\
         \x20   except ValueError:\n        seen.append(1)\n    assert seen == [1]\n\n\
         def test_missed_held():\n    seen = []\n    try:\n        int('1')\n\
         \x20   except ValueError:\n        seen.append(1)\n    assert seen == []\n\n\
         def test_raised():\n    seen = []\n    try:\n        int('x')\n\
         \x20   except ValueError:\n        seen.append(1)\n    assert seen == [1]\n",
    );
    let missed = |line: usize| Quote {
        assert: format!(
            "the try at line {line} expects ValueError, and the statement did not raise it"
        ),
        shown: None,
    };
    assert_eq!(
        results(
            &test_try,
            &["test_missed", "test_missed_held", "test_raised"]
        ),
        [
            Ok(Kind::Failed {
                asserts: vec![missed(3), shown("assert seen == [1]", "[1]", "[]")],
            }),
            Ok(Kind::Failed {
                asserts: vec![missed(11)],
            }),
            Ok(Kind::Passed),
        ],
        "the missed try comes first, then each expectation that did not hold; the lines of \
         its except that did not run are no fault; a statement that raises passes"
    );
}

#[test]
fn an_act_that_changes_the_warning_filters_is_refused_and_one_that_keeps_them_runs() {
    let test_filters = module(
        "test_filters.py",
        "import warnings\n\n\
         def test_filter():\n    warnings.simplefilter('ignore')\n    assert 1 == 1\n\n\
         def test_caught():\n    with warnings.catch_warnings():\n\
         \x20       warnings.simplefilter('error')\n    assert 1 == 1\n\n\
         def test_kept():\n    with warnings.catch_warnings(record=True) as seen:\n\
         \x20       warnings.warn('old', UserWarning)\n    assert len(seen) == 1\n",
    );
    let replaced = |line: usize| {
        Err(format!(
            "[hook-replaced] line {line} replaced the warning filters, \
             and a test keeps the filters that the run began with"
        ))
    };
    assert_eq!(
        results(&test_filters, &["test_kept", "test_filter", "test_caught"]),
        [Ok(Kind::Passed), replaced(4), replaced(9)],
        "catch_warnings with no filter keeps them; a filter that the act adds is refused, also \
         inside catch_warnings, which puts the filters back before the first expectation. \
         purba does not put a refused case's filters back, since each case runs in a \
         process of its own, so the case that keeps them runs first here"
    );
}

#[test]
fn a_function_of_the_test_module_that_c_code_calls_back_takes_any_path() {
    let test_callback = module(
        "test_callback.py",
        "def key(x):\n    if x < 0:\n        return -x\n    return x\n\n\
         def test_sorted():\n    got = sorted([3, 1, 2], key=key)\n    assert got == [1, 2, 3]\n\n\
         def test_by_name():\n    got = key(1)\n    assert got == 1\n",
    );
    let path = "the lines before the first expectation take one path";
    assert_eq!(
        results(&test_callback, &["test_sorted", "test_by_name"]),
        [
            Ok(Kind::Passed),
            Err(format!(
                "[second-path] line 11 reaches line 2 of key, which went one way only, \
                 and {path}; [second-path] line 11 reaches line 3 of key, which did not run, \
                 and {path}"
            )),
        ],
        "sorted calls key, so its paths are free; the test calls key by name, so its \
         paths keep the rule"
    );
}

#[test]
fn a_method_that_the_interpreter_calls_for_an_operator_takes_any_path() {
    let test_eq = module(
        "test_eq.py",
        "class Point:\n    def __init__(self, x):\n        self.x = x\n\n\
         \x20   def __eq__(self, other):\n        if isinstance(other, Point):\n\
         \x20           return self.x == other.x\n        return NotImplemented\n\n\
         def test_eq():\n    got = Point(1) == Point(1)\n    assert got is True\n",
    );
    assert_eq!(
        results(&test_eq, &["test_eq"]),
        [Ok(Kind::Passed)],
        "the test calls Point by name, which runs __init__; == calls __eq__, and no call names it"
    );
}

#[test]
fn the_refused_line_of_another_try_ends_with_the_shape_of_a_try_of_one_path() {
    let test_else = module(
        "test_else.py",
        "def test_else():\n    seen = []\n    try:\n        seen.append(1)\n\
         \x20   except ValueError:\n        seen.append(2)\n    else:\n        seen.append(3)\n\
         \x20   for name in []:\n        print(name)\n    assert seen == [1, 3]\n",
    );
    let path = "the lines before the first expectation take one path";
    let shape = "a try of one path holds one statement and one except SomeError:, \
                 with no else and no finally";
    assert_eq!(
        results(&test_else, &["test_else"]),
        [Err(format!(
            "[second-path] line 5 did not run, and {path}; [second-path] line 6 did not run, \
             and {path}; [second-path] line 10 did not run, and {path}; {shape}"
        ))],
        "the shape comes once, after every fault, though the last fault lies outside the try"
    );
}
