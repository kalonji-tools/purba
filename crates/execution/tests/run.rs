mod common;

use std::sync::{Mutex, PoisonError};

use common::module;
use purba_errors::{Error, Kind, Quote, Shown, Suppressed};
use purba_execution::{Module, Purba};

/// The turn of a test to run Python:
/// <https://doc.rust-lang.org/book/ch11-02-running-tests.html>
static TURN: Mutex<()> = Mutex::new(());

/// Gives `work` purba's process while no other test of this file runs Python.
/// Each test of this file calls it, never `purba_execution::attach`.
fn attach<R>(work: impl for<'py> FnOnce(&Purba<'py>) -> R) -> R {
    let _turn = TURN.lock().unwrap_or_else(PoisonError::into_inner);
    purba_execution::attach(work)
}

/// The quote of an expectation that did not hold, with what it shows.
fn shown(assert: &str, expected: &str, reading: &str) -> Quote {
    Quote {
        assert: assert.to_owned(),
        shown: Some(Shown::Values {
            expected: expected.to_owned(),
            reading: reading.to_owned(),
        }),
    }
}

/// The outcome or the error of one case that a zygote ran, and what its
/// suppressions gave.
type Ran = (Result<Kind, Error>, Suppressed);

/// The result of the load, then each of `functions`, run under `suppressions`
/// in a process that a zygote forks, as dispatch runs them.
fn through(
    module: &Module,
    functions: &[&str],
    suppressions: &[(&str, &str)],
) -> (Result<(), String>, Vec<Ran>) {
    attach(|purba: &Purba<'_>| {
        let (mut zygote, _) = match purba.zygote(module) {
            Ok(started) => started,
            Err(stop) => return (Err(stop.error.line()), Vec::new()),
        };
        let out = functions
            .iter()
            .map(|function| {
                let (result, suppressed, _) = zygote.run(function, suppressions);
                (result.map(|outcome| outcome.kind().clone()), suppressed)
            })
            .collect();
        zygote.end();
        (Ok(()), out)
    })
}

fn results(module: &Module, functions: &[&str]) -> Vec<Result<Kind, String>> {
    let (load, out) = through(module, functions, &[]);
    load.expect("the module loads");
    out.into_iter()
        .map(|(result, _)| result.map_err(|error| error.to_string()))
        .collect()
}

fn kinds(module: &Module, functions: &[&str]) -> Vec<Kind> {
    results(module, functions)
        .into_iter()
        .map(|result| result.expect("the test runs"))
        .collect()
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
    // Each process that runs the body writes a line to `loads`. If each test's
    // process loads the module, `loads` holds three lines.
    let loads = std::env::temp_dir().join(format!("purba-once-{}", std::process::id()));
    let _ = std::fs::remove_file(&loads);
    let test_once = module(
        "test_once.py",
        &format!(
            "with open({loads:?}, 'a') as loads:\n    loads.write('load\\n')\n\n\
             calls = []\n\n\
             def test_first():\n    calls.append(1)\n    assert calls == [1]\n\n\
             def test_second():\n    calls.append(2)\n    assert calls == [2]\n"
        ),
    );
    let kinds = kinds(&test_once, &["test_first", "test_second"]);
    let body = std::fs::read_to_string(&loads).expect("the body wrote its line");
    std::fs::remove_file(&loads).expect("the body wrote its line");
    assert_eq!(
        (kinds, body.as_str()),
        ([Kind::Passed, Kind::Passed].to_vec(), "load\n"),
        "the module's body runs once, in the zygote, for two of its tests, and the second \
         test does not see the list that the first test changed",
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
            exception: "ValueError: no".to_owned(),
            whence: None,
        }],
        "Python's exception as text"
    );
}

/// The quote of `assert`, a failed `all()` or `any()`, with its `heading` and
/// its `items`.
fn items(assert: &str, heading: &str, items: &[&str]) -> Kind {
    Kind::Failed {
        asserts: vec![Quote {
            assert: assert.to_owned(),
            shown: Some(Shown::Items {
                heading: heading.to_owned(),
                items: items.iter().map(|&item| item.to_owned()).collect(),
            }),
        }],
    }
}

#[test]
fn a_failed_all_or_any_shows_a_heading_and_each_item_after_its_reading_closes() {
    let test_each = module(
        "test_each_items.py",
        concat!(
            "class Item:\n    def __repr__(self):\n",
            "        assert True\n        return 'Item()'\n\n",
            "def test_repr_asserts():\n    items = [Item()]\n",
            "    assert all(isinstance(i, int) for i in items)\n\n",
            "def test_one():\n    xs = [5]\n    assert all(x < 3 for x in xs)\n\n",
            "def test_reading():\n    ws = ['ab', 'c']\n",
            "    assert all(len(w) == 2 for w in ws)\n\n",
            "def test_any():\n    xs = ['a', 'b']\n    assert any(x == 'z' for x in xs)\n\n",
            "def test_any_empty():\n    xs = []\n    assert any(x == 'z' for x in xs)\n",
        ),
    );

    assert_eq!(
        results(
            &test_each,
            &[
                "test_repr_asserts",
                "test_one",
                "test_reading",
                "test_any",
                "test_any_empty"
            ]
        ),
        [
            Ok(items(
                "assert all(isinstance(i, int) for i in items)",
                "the first item that breaks  isinstance(i, int)",
                &["[0]  Item()"]
            )),
            Ok(items(
                "assert all(x < 3 for x in xs)",
                "the first item that breaks  x < 3",
                &["[0]  5"]
            )),
            Ok(items(
                "assert all(len(w) == 2 for w in ws)",
                "the first item that breaks  len(w) == 2",
                &["[1]  len(w)  1     w  'c'"]
            )),
            Ok(items(
                "assert any(x == 'z' for x in xs)",
                "none of 2 items holds  x == 'z'",
                &["[0]  'a'", "[1]  'b'"]
            )),
            Ok(items(
                "assert any(x == 'z' for x in xs)",
                "none of 0 items holds  x == 'z'",
                &[]
            )),
        ],
        "a repr that asserts runs after the reading; one item, a reading that is not the item, \
         each item of an any(), and an any() of no item"
    );
}

#[test]
fn a_failed_all_stops_at_its_first_item_that_breaks() {
    let test_stops = module(
        "test_all_stops.py",
        concat!(
            "class Item:\n    made = 0\n\n",
            "    def __init__(self, ok):\n        Item.made += 1\n        self.ok = ok\n\n",
            "    def __repr__(self):\n        return f'Item(made={Item.made})'\n\n",
            "def made(oks):\n    for ok in oks:\n        yield Item(ok)\n\n",
            "def test_stops():\n",
            "    assert all(item.ok is True for item in made([False, True, True]))\n",
        ),
    );

    assert_eq!(
        results(&test_stops, &["test_stops"]),
        [Ok(items(
            "assert all(item.ok is True for item in made([False, True, True]))",
            "the first item that breaks  item.ok is True",
            &["[0]  item.ok  False     item  Item(made=1)"]
        ))],
        "the reading made one item, as Python's own all() reads no item after the first that \
         breaks"
    );
}

#[test]
fn an_iterator_that_a_parameter_holds_is_not_one_that_the_act_made() {
    let test_input = module(
        "test_input_iterator.py",
        "def test_input(it=iter([1])):\n    assert list(it) == [1]\n",
    );

    assert_eq!(
        results(&test_input, &["test_input"]),
        [Ok(Kind::Passed)],
        "the default of a parameter is no iterator that the act made"
    );
}

#[test]
fn an_any_that_holds_stops_at_its_first_true_item() {
    let test_any = module(
        "test_any_stops.py",
        "def gen():\n    yield 1\n    yield 5\n    raise ValueError('read past the item')\n\n\
         def test_stops():\n    assert any(x == 5 for x in gen())\n",
    );

    assert_eq!(
        results(&test_any, &["test_stops"]),
        [Ok(Kind::Passed)],
        "an any() reads no item after the first that holds, as Python's own does"
    );
}

#[test]
fn a_lambda_of_a_check_binds_its_parameter() {
    let test_lambda = module(
        "test_lambda_bound.py",
        "def test_lambda():\n    y = iter([1])\n    xs = [1]\n    \
         assert all((lambda y: y)(x) == 1 for x in xs)\n",
    );

    assert_eq!(
        results(&test_lambda, &["test_lambda"]),
        [Ok(Kind::Passed)],
        "the parameter of a lambda is no name of the test's frame"
    );
}

#[test]
fn a_raise_names_the_innermost_frame_outside_the_standard_library() {
    let test_whence = module(
        "test_whence.py",
        concat!(
            "import json\n\n",
            "def helper():\n    raise KeyError('k')\n\n",
            "def gen():\n    yield 1\n    raise ValueError('boom')\n\n",
            "def test_helper():\n    helper()\n\n",
            "def test_generator():\n    list(gen())\n\n",
            "def test_library():\n    json.loads('{')\n\n",
            "def test_own_generator():\n    list(1 / x for x in [0])\n\n",
            "def test_own_line():\n    raise ValueError('no')\n\n",
            "import dataclasses\n\n@dataclasses.dataclass(frozen=True)\n",
            "class Point:\n    x: int\n\n",
            "def test_generated():\n    p = Point(1)\n    p.x = 2\n",
        ),
    );

    let frames: Vec<Option<String>> = kinds(
        &test_whence,
        &[
            "test_helper",
            "test_generator",
            "test_library",
            "test_own_generator",
            "test_own_line",
            "test_generated",
        ],
    )
    .into_iter()
    .map(|kind| match kind {
        Kind::Raised { whence, .. } => whence,
        other => panic!("each test raises: {other:?}"),
    })
    .collect();

    assert_eq!(
        frames,
        [
            Some("raised in helper() at test_whence.py:4".to_owned()),
            Some("raised inside the generator gen() at test_whence.py:8".to_owned()),
            None,
            None,
            None,
            None,
        ],
        "a helper and a generator are named; the standard library, a generator of the test, \
         the test's own line and code that the standard library generates are not"
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
    let errors: Vec<String> = results(&test_bound, &["test_bound", "test_deque"])
        .into_iter()
        .map(|result| result.expect_err("an object that is not callable is not a test"))
        .collect();
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
fn a_module_that_fails_to_load_stops_with_its_error_and_runs_its_body_once() {
    let marker = std::env::temp_dir().join(format!("purba-fails-{}", std::process::id()));
    let loads = module(
        "test_fails.py",
        &format!(
            "with open({marker:?}, 'a') as marker:\n    marker.write('load\\n')\n\
             raise ValueError('load')\n\n\
             def test_one():\n    pass\n\ndef test_two():\n    pass\n"
        ),
    );
    let (load, ran) = through(&loads, &["test_one", "test_two"], &[]);
    let body = std::fs::read_to_string(&marker).expect("the body wrote the marker");
    std::fs::remove_file(&marker).expect("the marker goes");
    assert_eq!(
        (load, ran.len(), body.as_str()),
        (
            Err("project error: ValueError: load".to_owned()),
            0,
            "load\n"
        ),
        "the module stops with its error, no test runs, and its body ran once",
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
            exception: "AssertionError: no assert ran".to_owned(),
            whence: None,
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
    let (load, out) = through(&test_panic, &["test_one", "test_two", "test_three"], &[]);
    load.expect("the module loads");
    let results: Vec<Result<Kind, String>> = out
        .into_iter()
        .map(|(result, _)| result.map_err(|error| error.line()))
        .collect();
    assert_eq!(
        results,
        [
            Err("internal error: execution: raised by the test".to_owned()),
            Ok(Kind::Passed),
            Err("project error: TypeError: 'int' object is not callable".to_owned()),
        ],
        "the panic that crosses the call is an internal error of execution, the next test \
         passes, and a project error names no part"
    );
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
    let (load, out) = through(module, functions, suppressions);
    load.expect("the module loads");
    out.into_iter()
        .map(|(result, suppressed)| {
            let warned = suppressed
                .warnings
                .iter()
                .map(|warning| {
                    (
                        warning.check.clone(),
                        warning.words.clone(),
                        warning.reason.clone(),
                    )
                })
                .collect();
            (result.map_err(|error| error.to_string()), warned)
        })
        .collect()
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
fn a_raise_in_a_block_inside_a_swallowing_with_names_the_next_line_at_any_depth() {
    let test_nested = module(
        "test_nested_swallow.py",
        "import contextlib\n\n\n\
         def test_nested():\n    seen = []\n    with contextlib.suppress(ValueError):\n        \
         with contextlib.nullcontext():\n            int(\"x\")\n            seen.append(1)\n    \
         assert seen == []\n",
    );

    let runs = suppressed_runs(
        &test_nested,
        &["test_nested"],
        &[("swallowed-raise", "the raise is the act")],
    );

    assert_eq!(
        runs,
        [(
            Ok(Kind::Passed),
            warned(
                "swallowed-raise",
                "the with at line 6 swallowed a raise, and line 9 did not run",
                "the raise is the act"
            )
        )],
        "the line that did not run is the next statement at any depth of the with's body, \
         inside the nested with"
    );
}

#[test]
fn a_test_that_a_later_def_binds_again_watches_the_paths_of_that_def() {
    let twice = module(
        "test_twice_paths.py",
        "def test_twice():\n    assert 1 == 1\n\n\n\
         def test_twice():\n    x = 1\n    if x == 2:\n        x = 3\n    assert x == 1\n",
    );

    let runs = suppressed_runs(&twice, &["test_twice"], &[]);

    assert_eq!(
        runs,
        [(
            Err(
                "[second-path] line 7 went one way only, and the lines before the first \
                 expectation take one path; [second-path] line 8 did not run, and the lines \
                 before the first expectation take one path"
                    .to_owned()
            ),
            Vec::new()
        )],
        "the second def binds the name, so the case watches its lines before its own first \
         expectation, and not before the expectation of the first def"
    );
}

#[test]
fn a_check_that_reads_an_iterator_is_refused_unless_a_suppression_reaches_it() {
    let test_iterator = module(
        "test_iterator.py",
        "def test_iterator():\n    it = iter([1, 2])\n    assert list(it) == [1, 2]\n\n\
         def test_list():\n    rows = [1, 2]\n    assert list(iter(rows)) == [1, 2]\n",
    );
    let words = "line 3 reads it, an iterator that one reading empties; make it a list in the act";

    let refused = suppressed_runs(&test_iterator, &["test_iterator", "test_list"], &[]);
    let suppressed = suppressed_runs(
        &test_iterator,
        &["test_iterator"],
        &[("iterator-read", "the reader yields each row once")],
    );

    assert_eq!(
        refused,
        [
            (Err(format!("[iterator-read] {words}")), Vec::new()),
            (Ok(Kind::Passed), Vec::new()),
        ],
        "a name of the test that holds an iterator is refused; an iterator that the check \
         makes is not"
    );
    assert_eq!(
        suppressed,
        [(
            Ok(Kind::Passed),
            warned("iterator-read", words, "the reader yields each row once")
        )],
        "a suppression turns the fault into a warning, and the case reads the iterator once"
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

    let functions = ["test_holds", "test_fails", "test_raises", "test_form"];
    let (load, out) = through(
        &test_judged,
        &functions,
        &[("second-path", "the act branches")],
    );
    load.expect("the module loads");
    let judged: Vec<_> = functions
        .into_iter()
        .zip(out)
        .map(|(function, (result, suppressed))| {
            (
                function,
                result.is_ok(),
                suppressed.judged,
                suppressed.warnings.len(),
            )
        })
        .collect();

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
         process of its own"
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
