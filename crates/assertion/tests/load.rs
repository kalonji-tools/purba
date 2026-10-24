use pyo3::Python;
use pyo3::exceptions::PyIndexError;
use pyo3::types::{PyAnyMethods, PyModuleMethods};
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
        assert_eq!(last, Some("assert 2 == 2"), "the last assert that ran");
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
            Some("assert 1 == 1"),
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
fn the_mark_gives_its_value_back() {
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
        assert_eq!(result.unwrap().to_string(), "[]", "the value, unchanged");
        assert_eq!(last, Some("assert x"), "the text at its index");
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
        assert_eq!(last, Some("assert True"), "the bad mark recorded nothing");
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
