use std::fs;
use std::path::PathBuf;

use purba_command::Error;
use pyo3::Python;

/// Runs purba on the module `source`, and gives what `run` returned and stdout.
fn run_module(name: &str, source: &str) -> (Result<u8, Error>, String) {
    let path = PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join(name);
    fs::write(&path, source).expect("the test writes its module");
    let argv = [
        "purba".to_owned(),
        path.to_str().expect("the path is UTF-8").to_owned(),
    ];
    let mut out = Vec::new();
    let result = purba_command::run(&argv, Ok(&mut out));
    (
        result,
        String::from_utf8(out).expect("reporting writes UTF-8"),
    )
}

// purba's signal handler keeps the first signal for the life of the process.
// So the signal of this binary lives in this one test.
#[test]
fn an_interrupt_before_selection_ends_gives_no_count() {
    purba_execution::interrupt::install().expect("purba's signal handler takes SIGINT");
    Python::attach(|py| {
        py.run(
            c"import signal\nsignal.raise_signal(signal.SIGINT)",
            None,
            None,
        )
    })
    .expect("purba's signal handler raises nothing outside a call");
    for (name, source) in [
        ("test_interrupt.py", "def test_one():\n    assert True\n"),
        ("test_interrupt_syntax.py", "def test_one(:\n"),
    ] {
        let (result, stdout) = run_module(name, source);
        assert!(
            matches!(result, Err(Error::Interrupted)),
            "{name}: the run ends as interrupted: {result:?}"
        );
        assert_eq!(stdout, "", "{name}: no line and no count print");
    }
}
