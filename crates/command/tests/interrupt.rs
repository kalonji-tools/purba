use std::ffi::OsString;
use std::fs;
use std::path::PathBuf;

use purba_command::Error;
use pyo3::Python;

/// Runs purba on the module `source`, in a project of its own, and gives what
/// `run_with` returned and stdout. With `implicit`, the project holds `.git`
/// and no `pyproject.toml`, and `PURBA_IMPLICIT_ROOT` is `1`.
fn run_module(name: &str, source: &str, implicit: bool) -> (Result<u8, Error>, String) {
    let root = PathBuf::from(env!("CARGO_TARGET_TMPDIR"))
        .join("interrupt")
        .join(name);
    fs::create_dir_all(&root).expect("the test makes its project");
    let variables = if implicit {
        fs::create_dir_all(root.join(".git")).expect("the test makes .git");
        vec![(OsString::from("PURBA_IMPLICIT_ROOT"), OsString::from("1"))]
    } else {
        fs::write(root.join("pyproject.toml"), "[tool.purba]\n")
            .expect("the test writes the project's file");
        Vec::new()
    };
    fs::write(root.join(name), source).expect("the test writes its module");
    let argv = ["purba".to_owned(), name.to_owned()];
    let mut out = Vec::new();
    let result = purba_command::run_with(
        &argv,
        &root,
        &variables,
        Ok(&mut out),
        purba_assertion::rewrite,
    );
    (
        result,
        String::from_utf8(out).expect("reporting writes UTF-8"),
    )
}

// purba's signal handler keeps the first signal for the life of the process.
// So the signal of this binary lives in this one test.
#[test]
fn an_interrupt_before_selection_ends_gives_no_count() {
    purba_execution::set_hooks().expect("purba's signal handler takes SIGINT");
    Python::attach(|py| {
        py.run(
            c"import signal\nsignal.raise_signal(signal.SIGINT)",
            None,
            None,
        )
    })
    .expect("purba's signal handler raises nothing outside a call");
    for (name, source, implicit) in [
        (
            "test_interrupt.py",
            "def test_one():\n    assert 1 == 1\n",
            false,
        ),
        ("test_interrupt_syntax.py", "def test_one(:\n", false),
        (
            "test_interrupt_implicit.py",
            "def test_one():\n    assert 1 == 1\n",
            true,
        ),
    ] {
        let (result, stdout) = run_module(name, source, implicit);
        assert!(
            matches!(result, Err(Error::Interrupted)),
            "{name}: the run ends as interrupted: {result:?}"
        );
        assert_eq!(
            stdout, "",
            "{name}: no line and no count print, not even the notice of an implicit root"
        );
    }
}
