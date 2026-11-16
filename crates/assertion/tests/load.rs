use purba_assertion::{Readers, Readings, Rewrite, Shown, Verdict, Watched};
use pyo3::Python;
use pyo3::exceptions::{PyIndexError, PyNameError, PySystemExit};
use pyo3::types::{PyAnyMethods, PyModule, PyModuleMethods};
use std::fs;
use std::path::{Path, PathBuf};

fn source(text: &str) -> Rewrite {
    Rewrite::of_source(text, &[]).expect("the source parses")
}

fn rewritten(name: &str, text: &str) -> (PathBuf, Rewrite) {
    let path = PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join(name);
    fs::write(&path, text).expect("the test writes its module");
    let module = purba_parsing::parse(&path).expect("the module parses");
    (path, purba_assertion::rewrite(&module, &[]))
}

#[test]
fn loading_runs_the_body_of_the_module() {
    Python::initialize();
    Python::attach(|py| {
        let loaded = purba_assertion::load(py, Path::new("test_body.py"), &source("value = 1\n"))
            .expect("the module loads");
        let value: i64 = loaded.module().getattr("value").unwrap().extract().unwrap();
        assert_eq!(value, 1, "the body ran");
    });
}

#[test]
fn watch_gives_the_text_of_the_last_assert_that_ran() {
    let (path, rewrite) = rewritten(
        "test_marks.py",
        "def test_two():\n    assert 1 == 1\n    assert 2 == 2\n",
    );
    Python::initialize();
    Python::attach(|py| {
        let loaded = purba_assertion::load(py, &path, &rewrite).expect("the module loads");
        let test_two = loaded.module().getattr("test_two").unwrap();
        let Watched {
            result, mark: last, ..
        } = loaded.watch(&NO_READER, || test_two.call0());
        assert!(result.is_ok(), "the test returned");
        assert_eq!(
            last,
            Some(("assert 2 == 2", true)),
            "the last assert that ran, which held"
        );
    });
}

#[test]
fn watch_starts_clear() {
    let (path, rewrite) = rewritten(
        "test_clear.py",
        "assert 0 == 0\n\ndef test_marks():\n    assert 1 == 1\n\ndef test_none():\n    pass\n",
    );
    Python::initialize();
    Python::attach(|py| {
        let loaded = purba_assertion::load(py, &path, &rewrite).expect("the module loads");
        let marks = loaded.module().getattr("test_marks").unwrap();
        let none = loaded.module().getattr("test_none").unwrap();
        assert_eq!(
            loaded.watch(&NO_READER, || none.call0()).mark,
            None,
            "the module-level assert ran before watch"
        );
        assert_eq!(
            loaded.watch(&NO_READER, || marks.call0()).mark,
            Some(("assert 1 == 1", true)),
            "the test's own assert"
        );
        assert_eq!(
            loaded.watch(&NO_READER, || none.call0()).mark,
            None,
            "an assert ran in the watch before"
        );
    });
}

#[test]
fn the_mark_gives_back_whether_its_value_held() {
    let rewrite = source("if False:\n    assert x\n\ndef kept():\n    return _purba_ran(0, [])\n");
    Python::initialize();
    Python::attach(|py| {
        let loaded = purba_assertion::load(py, Path::new("test_value.py"), &rewrite)
            .expect("the module loads");
        let kept = loaded.module().getattr("kept").unwrap();
        let Watched {
            result, mark: last, ..
        } = loaded.watch(&NO_READER, || kept.call0());
        assert_eq!(
            result.unwrap().to_string(),
            "False",
            "an empty list does not hold"
        );
        assert_eq!(
            last,
            Some(("assert x", false)),
            "the text at its index, which did not hold"
        );
    });
}

#[test]
fn an_assert_tests_its_value_once() {
    let (path, rewrite) = rewritten(
        "test_once.py",
        concat!(
            "class Counted:\n",
            "    tests = 0\n",
            "    def __bool__(self):\n",
            "        Counted.tests += 1\n",
            "        return True\n",
            "def test_counted():\n",
            "    assert Counted()\n",
        ),
    );
    Python::initialize();
    Python::attach(|py| {
        let loaded = purba_assertion::load(py, &path, &rewrite).expect("the module loads");
        let module = loaded.module();
        let counted = module.getattr("test_counted").unwrap();
        let Watched { result, .. } = loaded.watch(&NO_READER, || counted.call0());
        assert!(result.is_ok(), "the test returned");
        let tests: i64 = module
            .getattr("Counted")
            .unwrap()
            .getattr("tests")
            .unwrap()
            .extract()
            .unwrap();
        assert_eq!(tests, 1, "__bool__ ran once");
    });
}

#[test]
fn a_mark_with_no_assert_raises() {
    let rewrite = source("def test_beyond():\n    assert True\n    _purba_ran(1, True)\n");
    Python::initialize();
    Python::attach(|py| {
        let loaded = purba_assertion::load(py, Path::new("test_beyond.py"), &rewrite)
            .expect("the module loads");
        let test = loaded.module().getattr("test_beyond").unwrap();
        let Watched {
            result, mark: last, ..
        } = loaded.watch(&NO_READER, || test.call0());
        let err = result.expect_err("the mark raises");
        assert!(
            err.is_instance_of::<PyIndexError>(py),
            "IndexError, not {err}"
        );
        assert_eq!(
            last,
            Some(("assert True", true)),
            "the bad mark recorded nothing"
        );
    });
}

#[test]
fn the_module_is_named_by_its_file() {
    Python::initialize();
    Python::attach(|py| {
        let loaded = purba_assertion::load(
            py,
            Path::new("dir/test_named.py"),
            &source("import sys\nhere = __name__ in sys.modules\ndef f():\n    raise ValueError\n"),
        )
        .expect("the module loads");
        let module = loaded.module();
        assert_eq!(
            module.name().unwrap().to_string(),
            "test_named",
            "the stem of its file"
        );
        assert_eq!(
            module.getattr("__file__").unwrap().to_string(),
            "dir/test_named.py",
            "its path"
        );
        assert!(
            module.getattr("here").unwrap().is_truthy().unwrap(),
            "in sys.modules as it runs"
        );
        let code = module.getattr("f").unwrap().getattr("__code__").unwrap();
        assert_eq!(
            code.getattr("co_filename").unwrap().to_string(),
            "dir/test_named.py",
            "a traceback names its path"
        );
    });
}

#[test]
fn a_dataclass_under_postponed_annotations_loads() {
    Python::initialize();
    Python::attach(|py| {
        let loaded = purba_assertion::load(
            py,
            Path::new("test_dataclass.py"),
            &source(concat!(
                "from __future__ import annotations\n",
                "import dataclasses, pickle\n",
                "@dataclasses.dataclass\n",
                "class Point:\n",
                "    x: int\n",
                "kept = pickle.loads(pickle.dumps(Point(1))) == Point(1)\n",
            )),
        );
        let loaded = loaded.expect("the module loads");
        assert!(
            loaded
                .module()
                .getattr("kept")
                .unwrap()
                .is_truthy()
                .unwrap(),
            "the dataclass loaded, and its instance pickled"
        );
    });
}

fn in_sys_modules(py: Python<'_>, name: &str) -> bool {
    py.import("sys")
        .unwrap()
        .getattr("modules")
        .unwrap()
        .contains(name)
        .unwrap()
}

#[test]
fn a_new_name_leaves_sys_modules_when_its_body_raises() {
    Python::initialize();
    Python::attach(|py| {
        let loaded = purba_assertion::load(
            py,
            Path::new("test_raises_on_load.py"),
            &source("raise ValueError('x')\n"),
        );
        assert!(loaded.is_err(), "the body raised");
        assert!(
            !in_sys_modules(py, "test_raises_on_load"),
            "the half-made module left sys.modules"
        );
    });
}

#[test]
fn a_shadowed_name_gets_its_module_back_when_the_body_raises() {
    Python::initialize();
    Python::attach(|py| {
        let modules = py.import("sys").unwrap().getattr("modules").unwrap();
        let before = PyModule::new(py, "test_shadowed").unwrap();
        modules.set_item("test_shadowed", &before).unwrap();
        let loaded = purba_assertion::load(
            py,
            Path::new("test_shadowed.py"),
            &source("raise ValueError('x')\n"),
        );
        assert!(loaded.is_err(), "the body raised");
        assert!(
            modules.get_item("test_shadowed").unwrap().is(&before),
            "the name holds the module it held before the load"
        );
    });
}

#[test]
fn a_body_that_calls_sys_exit_leaves_no_entry() {
    Python::initialize();
    Python::attach(|py| {
        let loaded = purba_assertion::load(
            py,
            Path::new("test_exits_on_load.py"),
            &source("import sys\nsys.exit(3)\n"),
        );
        assert!(
            loaded.is_err_and(|err| err.is_instance_of::<PySystemExit>(py)),
            "the body raised SystemExit"
        );
        assert!(
            !in_sys_modules(py, "test_exits_on_load"),
            "a SystemExit took the module out too"
        );
    });
}

#[test]
fn a_module_that_binds_the_hook_fails_to_load() {
    Python::initialize();
    Python::attach(|py| {
        let helpers = PyModule::new(py, "helpers_that_bind_the_hook").unwrap();
        helpers.setattr("__all__", vec!["_purba_ran"]).unwrap();
        helpers.setattr("_purba_ran", 0).unwrap();
        py.import("sys")
            .unwrap()
            .getattr("modules")
            .unwrap()
            .set_item("helpers_that_bind_the_hook", helpers)
            .unwrap();
        for (file, body) in [
            (
                "test_bind_def.py",
                "def _purba_ran(i, x):\n    return True\n",
            ),
            (
                "test_bind_full_width.py",
                "def _\u{ff50}urba_ran(i, x):\n    return True\n",
            ),
            ("test_bind_del.py", "del _purba_ran\n"),
            (
                "test_bind_star.py",
                "from helpers_that_bind_the_hook import *\n",
            ),
            (
                "test_bind_update.py",
                "globals().update({'_purba_ran': None})\n",
            ),
        ] {
            let loaded = purba_assertion::load(py, Path::new(file), &source(body));
            let err = loaded
                .err()
                .unwrap_or_else(|| panic!("{file}: the load fails"));
            assert!(
                err.is_instance_of::<PyNameError>(py),
                "{file}: NameError, not {err}"
            );
            assert_eq!(
                err.value(py).to_string(),
                format!("{file} binds _purba_ran, the name of purba's assert hook"),
                "{file}: the error names the file and the hook"
            );
            assert!(
                !in_sys_modules(py, file.trim_end_matches(".py")),
                "{file}: the module left sys.modules, as a failed body does"
            );
        }
        py.import("sys")
            .unwrap()
            .getattr("modules")
            .unwrap()
            .del_item("helpers_that_bind_the_hook")
            .unwrap();
    });
}

#[test]
fn a_call_that_rebinds_the_hook_gives_a_name_error() {
    Python::initialize();
    Python::attach(|py| {
        let loaded = purba_assertion::load(
            py,
            Path::new("test_rebinds.py"),
            &source(concat!(
                "def test_swap():\n",
                "    global _purba_ran\n",
                "    _purba_ran = lambda i, x: True\n",
                "\n",
                "def test_after():\n",
                "    assert 1 == 1\n",
            )),
        )
        .expect("the module loads");
        let swap = loaded.module().getattr("test_swap").unwrap();
        let after = loaded.module().getattr("test_after").unwrap();

        let err = loaded
            .watch(&NO_READER, || swap.call0())
            .result
            .expect_err("the call that rebinds the hook gives an error");
        assert!(
            err.is_instance_of::<PyNameError>(py),
            "NameError, not {err}"
        );
        assert_eq!(
            err.value(py).to_string(),
            "test_rebinds.py binds _purba_ran, the name of purba's assert hook",
            "the error names the file and the hook"
        );
        assert_eq!(
            loaded.watch(&NO_READER, || after.call0()).mark,
            Some(("assert 1 == 1", true)),
            "the next call finds the hook back"
        );
    });
}

#[test]
fn a_function_that_binds_the_hook_as_a_local_name_fails_to_load() {
    Python::initialize();
    Python::attach(|py| {
        for (file, body) in [
            (
                "test_local_assign.py",
                "def test_x():\n    _purba_ran = None\n",
            ),
            (
                "test_local_parameter.py",
                "def helper(_purba_ran):\n    pass\n",
            ),
            (
                "test_local_loop.py",
                "def test_x():\n    for _purba_ran in []:\n        pass\n",
            ),
            (
                "test_local_full_width.py",
                "def test_x():\n    _\u{ff50}urba_ran = None\n",
            ),
            (
                "test_local_closure.py",
                concat!(
                    "def test_x():\n",
                    "    _purba_ran = None\n",
                    "    def inner():\n",
                    "        return _purba_ran\n",
                ),
            ),
        ] {
            let loaded = purba_assertion::load(py, Path::new(file), &source(body));
            let err = loaded
                .err()
                .unwrap_or_else(|| panic!("{file}: the load fails"));
            assert_eq!(
                err.value(py).to_string(),
                format!("{file} binds _purba_ran, the name of purba's assert hook"),
                "{file}: the error names the file and the hook"
            );
        }
    });
}

#[test]
fn a_call_that_rebinds_the_hook_and_fails_its_assert_gives_a_name_error() {
    Python::initialize();
    Python::attach(|py| {
        let loaded = purba_assertion::load(
            py,
            Path::new("test_rebinds_and_fails.py"),
            &source(concat!(
                "def test_swap():\n",
                "    global _purba_ran\n",
                "    _purba_ran = lambda i, x: False\n",
                "    assert 1 == 1\n",
            )),
        )
        .expect("the module loads");
        let swap = loaded.module().getattr("test_swap").unwrap();
        let err = loaded
            .watch(&NO_READER, || swap.call0())
            .result
            .expect_err("the call gives an error");
        assert!(
            err.is_instance_of::<PyNameError>(py),
            "the NameError, and not the AssertionError of the call: {err}"
        );
    });
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
fn a_body_that_raises_purbas_panic_exception_leaves_no_entry() {
    Python::initialize();
    Python::attach(|py| {
        // PyO3 resumes its own `PanicException` as a Rust panic.
        let crossed = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            purba_assertion::load(
                py,
                Path::new("test_panics_on_load.py"),
                &source(&format!(
                    "{PANIC_EXCEPTION}raise panic_exception()('raised while it loads')\n"
                )),
            )
            .is_ok()
        }));
        assert!(crossed.is_err(), "the panic crossed the load");
        assert!(
            !in_sys_modules(py, "test_panics_on_load"),
            "a panic took the module out too"
        );
    });
}

#[test]
fn a_syntax_error_that_ruff_passes_is_cpythons_error_at_the_load() {
    Python::initialize();
    Python::attach(|py| {
        let rewrite = source("def test_a():\n    assert True\n\nreturn 5\n");
        let err = purba_assertion::load(py, Path::new("test_return.py"), &rewrite)
            .err()
            .expect("the load fails");
        assert_eq!(
            err.to_string(),
            "SyntaxError: 'return' outside function (test_return.py, line 4)",
            "CPython names the error when the module compiles"
        );
    });
}

#[test]
fn a_module_that_nests_too_deep_is_cpythons_error_at_the_load() {
    Python::initialize();
    Python::attach(|py| {
        let rewrite = source(&format!(
            "def test_a():\n    x = {}1\n    assert x\n",
            "-".repeat(200_000)
        ));
        let err = purba_assertion::load(py, Path::new("test_deep.py"), &rewrite)
            .err()
            .expect("the load fails");
        assert_eq!(
            err.to_string(),
            "MemoryError: Parser stack overflowed - Python source too complex to parse",
            "CPython's error, as for any module that does not compile"
        );
    });
}

/// Readers that a test of the hook needs: none, since a Python thread always
/// seems alive.
struct NoReader;

static NO_READER: NoReader = NoReader;

impl Readers for NoReader {
    fn threaded(&self, _py: Python<'_>) -> bool {
        true
    }

    fn fork(&self, _py: Python<'_>) -> std::io::Result<Option<i32>> {
        unreachable!("a process with a thread forks no reader")
    }

    fn wait(&self, _pid: i32) -> Verdict {
        unreachable!("a process with a thread forks no reader")
    }

    fn end(&self, _verdict: Verdict) -> ! {
        unreachable!("no process is a reader")
    }
}

/// Readers that never fork: each looks forked, and gives `Fails`.
struct Failing;

static FAILING: Failing = Failing;

impl Readers for Failing {
    fn threaded(&self, _py: Python<'_>) -> bool {
        false
    }

    fn fork(&self, _py: Python<'_>) -> std::io::Result<Option<i32>> {
        Ok(Some(1))
    }

    fn wait(&self, _pid: i32) -> Verdict {
        Verdict::Fails
    }

    fn end(&self, _verdict: Verdict) -> ! {
        unreachable!("no process is a reader")
    }
}

const EXPECTS: &str = "seen = []\n\n\
    def test_three():\n    seen.append(1)\n\
    \x20   assert seen == [1]\n    assert seen != []\n    assert seen == [2]\n";

#[test]
fn the_first_expectation_reads_each_later_one_alone_before_it_reads_in_order() {
    Python::initialize();
    Python::attach(|py| {
        let rewrite = Rewrite::of_source(EXPECTS, &["test_three"]).expect("the source parses");
        let loaded = purba_assertion::load(py, Path::new("test_expects.py"), &rewrite)
            .expect("the module loads");
        let test = loaded.module().getattr("test_three").unwrap();
        let Watched {
            result,
            mark,
            readings,
            ..
        } = loaded.watch(&FAILING, || test.call0());
        assert!(result.is_ok(), "the test returned: {result:?}");
        assert_eq!(mark, None, "an expectation keeps no mark");
        assert_eq!(
            readings,
            Readings {
                in_order: vec![(0, true), (1, true), (2, false)],
                shown: vec![(
                    2,
                    Shown {
                        expected: "[2]".to_owned(),
                        reading: "[1]".to_owned(),
                    },
                )],
                alone: vec![(1, Verdict::Fails), (2, Verdict::Fails)],
                refused: None,
                replay: Vec::new(),
            },
            "a reader for each expectation after the first, then each in order"
        );
    });
}

#[test]
fn a_python_thread_alive_at_the_first_expectation_leaves_the_case_with_no_reader() {
    Python::initialize();
    Python::attach(|py| {
        let rewrite = Rewrite::of_source(EXPECTS, &["test_three"]).expect("the source parses");
        let loaded = purba_assertion::load(py, Path::new("test_threaded.py"), &rewrite)
            .expect("the module loads");
        let test = loaded.module().getattr("test_three").unwrap();
        let readings = loaded.watch(&NO_READER, || test.call0()).readings;
        assert_eq!(readings.alone, [], "no reader, so no verdict alone");
        assert_eq!(
            readings.in_order,
            [(0, true), (1, true), (2, false)],
            "each expectation in order"
        );
    });
}

#[test]
fn a_module_that_binds_the_expectation_hook_does_not_load() {
    Python::initialize();
    Python::attach(|py| {
        let err = purba_assertion::load(
            py,
            Path::new("test_binds_read.py"),
            &source("_purba_read = None\n"),
        )
        .err()
        .expect("the module binds the hook");
        assert!(
            err.is_instance_of::<PyNameError>(py),
            "NameError, not {err}"
        );
        assert_eq!(
            err.value(py).to_string(),
            "test_binds_read.py binds _purba_read, the name of purba's expectation hook",
            "the error names the hook"
        );
    });
}

/// Readers whose fork the operating system refuses.
struct Refusing;

static REFUSING: Refusing = Refusing;

impl Readers for Refusing {
    fn threaded(&self, _py: Python<'_>) -> bool {
        false
    }

    fn fork(&self, _py: Python<'_>) -> std::io::Result<Option<i32>> {
        Err(std::io::Error::other("no process left"))
    }

    fn wait(&self, _pid: i32) -> Verdict {
        unreachable!("no fork took place")
    }

    fn end(&self, _verdict: Verdict) -> ! {
        unreachable!("no process is a reader")
    }
}

#[test]
fn a_refused_fork_is_kept_and_each_expectation_is_read_in_order() {
    Python::initialize();
    Python::attach(|py| {
        let rewrite = Rewrite::of_source(EXPECTS, &["test_three"]).expect("the source parses");
        let loaded = purba_assertion::load(py, Path::new("test_refused.py"), &rewrite)
            .expect("the module loads");
        let test = loaded.module().getattr("test_three").unwrap();
        let readings = loaded.watch(&REFUSING, || test.call0()).readings;
        assert_eq!(
            readings.refused.as_deref(),
            Some("purba cannot fork to read an expectation alone: no process left"),
            "the refusal, with the operating system's error"
        );
        assert_eq!(
            readings.in_order,
            [(0, true), (1, true), (2, false)],
            "each expectation in order"
        );
    });
}

#[test]
fn a_test_that_rebinds_the_expectation_hook_gives_the_name_error_and_the_hook_is_back() {
    Python::initialize();
    Python::attach(|py| {
        let rewrite = Rewrite::of_source(
            "def test_rebind():\n    global _purba_read\n    _purba_read = None\n\n\
             def test_after():\n    assert 1 == 1\n",
            &["test_rebind", "test_after"],
        )
        .expect("the source parses");
        let loaded = purba_assertion::load(py, Path::new("test_rebind.py"), &rewrite)
            .expect("the module loads");
        let rebind = loaded.module().getattr("test_rebind").unwrap();
        let after = loaded.module().getattr("test_after").unwrap();
        let err = loaded
            .watch(&NO_READER, || rebind.call0())
            .result
            .expect_err("the call that rebinds the hook gives an error");
        assert_eq!(
            err.value(py).to_string(),
            "test_rebind.py binds _purba_read, the name of purba's expectation hook",
            "the error names the hook"
        );
        assert_eq!(
            loaded.watch(&NO_READER, || after.call0()).readings.in_order,
            [(0, true)],
            "the next test reads through the hook that is back"
        );
    });
}

/// What a test of `SHOWS` gave: how its call ended, each verdict in order, and
/// the expected value and the reading of each expectation that did not hold.
type Seen = (String, Vec<(usize, bool)>, Vec<(String, String)>);

const SHOWS: &str = "class One:\n    def __repr__(self):\n        return '1'\n\n\
    class NoRepr:\n    def __repr__(self):\n        raise ValueError('no repr')\n\n\
    class Always:\n    def __eq__(self, other):\n        return [1]\n\n\
    def test_eq():\n    got = [-3]\n    assert got == []\n\n\
    def test_ne():\n    got = []\n    assert got != []\n\n\
    def test_left():\n    got = 3\n    assert 2 > got\n\n\
    def test_contains():\n    got = 'ab'\n    assert 'z' in got\n\n\
    def test_not_contains():\n    got = 'ab'\n    assert 'a' not in got\n\n\
    def test_member():\n    got = 2\n    assert got in (0, 1)\n\n\
    def test_is():\n    got = [-1]\n    assert got is None\n\n\
    def test_same_text():\n    got = One()\n    assert got == 1\n\n\
    def test_no_repr():\n    got = NoRepr()\n    assert got == 1\n\n\
    def test_both_literal():\n    assert 1 + 1 == 3\n\n\
    def test_holds():\n    got = Always()\n    assert got == 2\n\n\
    def test_raises():\n    got = 'a'\n    assert got < 2\n\n\
    def test_lt():\n    got = 3\n    assert got < 2\n\n\
    def test_le():\n    got = 3\n    assert got <= 2\n\n\
    def test_gt():\n    got = 1\n    assert got > 2\n\n\
    def test_ge():\n    got = 1\n    assert got >= 2\n\n\
    def test_left_lt():\n    got = 1\n    assert 2 < got\n\n\
    def test_left_le():\n    got = 1\n    assert 2 <= got\n\n\
    def test_left_ge():\n    got = 3\n    assert 2 >= got\n\n\
    def test_not_member():\n    got = 0\n    assert got not in (0, 1)\n\n\
    def test_is_not():\n    got = None\n    assert got is not None\n";

#[test]
fn a_failed_expectation_shows_its_expected_value_and_its_reading() {
    let tests = [
        "test_eq",
        "test_ne",
        "test_left",
        "test_contains",
        "test_not_contains",
        "test_member",
        "test_is",
        "test_same_text",
        "test_no_repr",
        "test_both_literal",
        "test_holds",
        "test_raises",
        "test_lt",
        "test_le",
        "test_gt",
        "test_ge",
        "test_left_lt",
        "test_left_le",
        "test_left_ge",
        "test_not_member",
        "test_is_not",
    ];
    Python::initialize();
    Python::attach(|py| {
        let rewrite = Rewrite::of_source(SHOWS, &tests).expect("the source parses");
        let loaded = purba_assertion::load(py, Path::new("test_shows.py"), &rewrite)
            .expect("the module loads");
        let seen: Vec<Seen> = tests
            .iter()
            .map(|name| {
                let test = loaded.module().getattr(*name).unwrap();
                let Watched {
                    result, readings, ..
                } = loaded.watch(&NO_READER, || test.call0());
                let result = result.map_or_else(|err| err.to_string(), |_| "returned".to_owned());
                let shown = readings
                    .shown
                    .into_iter()
                    .map(|(_, shown)| (shown.expected, shown.reading))
                    .collect();
                (result, readings.in_order, shown)
            })
            .collect();
        let failed = |index: usize, expected: &str, reading: &str| {
            (
                "returned".to_owned(),
                vec![(index, false)],
                vec![(expected.to_owned(), reading.to_owned())],
            )
        };
        assert_eq!(
            seen,
            [
                failed(0, "[]", "[-3]"),
                failed(1, "!= [] (list)", "[] (list)"),
                failed(2, "< 2", "3"),
                failed(3, "to contain 'z'", "'ab'"),
                failed(4, "not to contain 'a'", "'ab'"),
                failed(5, "in (0, 1)", "2"),
                failed(6, "is None", "[-1]"),
                failed(7, "1 (int)", "1 (One)"),
                failed(8, "1", "(repr raised ValueError: no repr)"),
                failed(9, "3", "2"),
                ("returned".to_owned(), vec![(10, true)], Vec::new()),
                (
                    "TypeError: '<' not supported between instances of 'str' and 'int'".to_owned(),
                    Vec::new(),
                    Vec::new()
                ),
                failed(12, "< 2", "3"),
                failed(13, "<= 2", "3"),
                failed(14, "> 2", "1"),
                failed(15, ">= 2", "1"),
                failed(16, "> 2", "1"),
                failed(17, ">= 2", "1"),
                failed(18, "<= 2", "3"),
                failed(19, "not in (0, 1)", "0"),
                failed(20, "is not None (NoneType)", "None (NoneType)"),
            ],
            "the expected value is the literal side, or the right side when both are \
             literals; its line names the rule of any operator but ==; a type shows only \
             when the two texts are the same; Python's own operator decides, and its raise \
             stays the test's"
        );
    });
}
