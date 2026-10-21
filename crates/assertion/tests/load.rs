use pyo3::prelude::*;
use std::fs;
use std::path::{Path, PathBuf};

fn rewritten(name: &str, text: &str) -> (PathBuf, String) {
    let path = PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join(name);
    fs::write(&path, text).expect("the test writes its module");
    let module = purba_parsing::parse(&path).expect("the module parses");
    (path, purba_assertion::rewrite(&module).text().to_owned())
}

#[test]
fn loading_runs_the_body_of_the_module() {
    Python::initialize();
    Python::attach(|py| {
        let loaded = purba_assertion::load(py, Path::new("test_body.py"), "value = 1\n")
            .expect("the module loads");
        let value: i64 = loaded.module().getattr("value").unwrap().extract().unwrap();
        assert_eq!(value, 1, "the body ran");
        assert_eq!(loaded.take(), None, "no assert ran");
    });
}

#[test]
fn a_marked_assert_records_its_index_once() {
    let (path, text) = rewritten("test_marks.py", "assert 1 == 1\nassert 2 == 2\n");
    Python::initialize();
    Python::attach(|py| {
        let loaded = purba_assertion::load(py, &path, &text).expect("the module loads");
        assert_eq!(loaded.take(), Some(1), "the last assert that ran");
        assert_eq!(loaded.take(), None, "take clears it");
    });
}

#[test]
fn the_mark_gives_its_value_back() {
    Python::initialize();
    Python::attach(|py| {
        let loaded =
            purba_assertion::load(py, Path::new("test_value.py"), "kept = _purba_ran(7, [])\n")
                .expect("the module loads");
        let kept = loaded.module().getattr("kept").unwrap();
        assert_eq!(kept.to_string(), "[]", "the value, unchanged");
        assert_eq!(loaded.take(), Some(7), "its index");
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
