use pyo3::Python;
use pyo3::exceptions::{PyIndexError, PyNameError, PySystemExit};
use pyo3::types::{PyAnyMethods, PyModule, PyModuleMethods};
use std::fs;
use std::path::{Path, PathBuf};

fn rewritten(name: &str, text: &str) -> (PathBuf, String, Vec<String>) {
    let path = PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join(name);
    fs::write(&path, text).expect("the test writes its module");
    let module = purba_parsing::parse(&path).expect("the module parses");
    let (text, asserts) = purba_assertion::rewrite(&module).into_parts();
    (path, text, asserts)
}

#[test]
fn loading_runs_the_body_of_the_module() {
    Python::initialize();
    Python::attach(|py| {
        let loaded =
            purba_assertion::load(py, Path::new("test_body.py"), "value = 1\n", Vec::new())
                .expect("the module loads");
        let value: i64 = loaded.module().getattr("value").unwrap().extract().unwrap();
        assert_eq!(value, 1, "the body ran");
    });
}

#[test]
fn watch_gives_the_text_of_the_last_assert_that_ran() {
    let (path, text, asserts) = rewritten(
        "test_marks.py",
        "def test_two():\n    assert 1 == 1\n    assert 2 == 2\n",
    );
    Python::initialize();
    Python::attach(|py| {
        let loaded = purba_assertion::load(py, &path, &text, asserts).expect("the module loads");
        let test_two = loaded.module().getattr("test_two").unwrap();
        let (result, last) = loaded.watch(|| test_two.call0());
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
    let (path, text, asserts) = rewritten(
        "test_clear.py",
        "assert 0 == 0\n\ndef test_marks():\n    assert 1 == 1\n\ndef test_none():\n    pass\n",
    );
    Python::initialize();
    Python::attach(|py| {
        let loaded = purba_assertion::load(py, &path, &text, asserts).expect("the module loads");
        let marks = loaded.module().getattr("test_marks").unwrap();
        let none = loaded.module().getattr("test_none").unwrap();
        assert_eq!(
            loaded.watch(|| none.call0()).1,
            None,
            "the module-level assert ran before watch"
        );
        assert_eq!(
            loaded.watch(|| marks.call0()).1,
            Some(("assert 1 == 1", true)),
            "the test's own assert"
        );
        assert_eq!(
            loaded.watch(|| none.call0()).1,
            None,
            "an assert ran in the watch before"
        );
    });
}

#[test]
fn the_mark_gives_back_whether_its_value_held() {
    Python::initialize();
    Python::attach(|py| {
        let loaded = purba_assertion::load(
            py,
            Path::new("test_value.py"),
            "def kept():\n    return _purba_ran(0, [])\n",
            vec!["assert x".to_owned()],
        )
        .expect("the module loads");
        let kept = loaded.module().getattr("kept").unwrap();
        let (result, last) = loaded.watch(|| kept.call0());
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
    let (path, text, asserts) = rewritten(
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
        let loaded = purba_assertion::load(py, &path, &text, asserts).expect("the module loads");
        let module = loaded.module();
        let counted = module.getattr("test_counted").unwrap();
        let (result, _) = loaded.watch(|| counted.call0());
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
    Python::initialize();
    Python::attach(|py| {
        let loaded = purba_assertion::load(
            py,
            Path::new("test_beyond.py"),
            "def test_beyond():\n    assert _purba_ran(0, True)\n    assert _purba_ran(1, True)\n",
            vec!["assert True".to_owned()],
        )
        .expect("the module loads");
        let test = loaded.module().getattr("test_beyond").unwrap();
        let (result, last) = loaded.watch(|| test.call0());
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
            "import sys\nhere = __name__ in sys.modules\ndef f():\n    raise ValueError\n",
            Vec::new(),
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
            concat!(
                "from __future__ import annotations\n",
                "import dataclasses, pickle\n",
                "@dataclasses.dataclass\n",
                "class Point:\n",
                "    x: int\n",
                "kept = pickle.loads(pickle.dumps(Point(1))) == Point(1)\n",
            ),
            Vec::new(),
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
            "raise ValueError('x')\n",
            Vec::new(),
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
            "raise ValueError('x')\n",
            Vec::new(),
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
            "import sys\nsys.exit(3)\n",
            Vec::new(),
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
            let loaded = purba_assertion::load(py, Path::new(file), body, Vec::new());
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
            concat!(
                "def test_swap():\n",
                "    global _purba_ran\n",
                "    _purba_ran = lambda i, x: True\n",
                "\n",
                "def test_after():\n",
                "    assert _purba_ran(0, 1 == 1)\n",
            ),
            vec!["assert 1 == 1".to_owned()],
        )
        .expect("the module loads");
        let swap = loaded.module().getattr("test_swap").unwrap();
        let after = loaded.module().getattr("test_after").unwrap();

        let err = loaded
            .watch(|| swap.call0())
            .0
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
            loaded.watch(|| after.call0()).1,
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
            let loaded = purba_assertion::load(py, Path::new(file), body, Vec::new());
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
            concat!(
                "def test_swap():\n",
                "    global _purba_ran\n",
                "    _purba_ran = lambda i, x: False\n",
                "    assert _purba_ran(0, 1 == 1)\n",
            ),
            vec!["assert 1 == 1".to_owned()],
        )
        .expect("the module loads");
        let swap = loaded.module().getattr("test_swap").unwrap();
        let err = loaded
            .watch(|| swap.call0())
            .0
            .expect_err("the call gives an error");
        assert!(
            err.is_instance_of::<PyNameError>(py),
            "the NameError, and not the AssertionError of the call: {err}"
        );
    });
}
