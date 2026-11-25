use std::fs;
use std::path::PathBuf;

use pyo3::Python;

// purba's signal handler keeps the first signal for the life of the process.
// So the signal of this binary lives in this one test.
#[test]
fn an_interrupt_in_the_rewrite_leaves_each_test_unreached() {
    purba_execution::set_hooks().expect("purba's signal handler takes SIGINT");
    let root = PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join("interrupt_in_rewrite");
    fs::create_dir_all(&root).expect("the test makes its project");
    fs::write(root.join("pyproject.toml"), "[tool.purba]\n")
        .expect("the test writes the project's file");
    fs::write(
        root.join("test_interrupt_in_rewrite.py"),
        "def test_one():\n    assert 1 == 1\ndef test_two():\n    assert 1 == 1\n",
    )
    .expect("the test writes its module");
    let argv = ["purba".to_owned()];
    let mut out = Vec::new();
    let result = purba_command::run_with(&argv, &root, Ok(&mut out), |facts, tests| {
        Python::attach(|py| {
            py.run(
                c"import signal\nsignal.raise_signal(signal.SIGINT)",
                None,
                None,
            )
        })
        .expect("purba's signal handler raises nothing outside a call");
        purba_assertion::rewrite(facts, tests)
    });
    let stdout = String::from_utf8(out).expect("reporting writes UTF-8");
    assert!(
        result.is_ok(),
        "selection knows the tests, so the run goes on to the count: {result:?}"
    );
    assert_eq!(
        stdout.lines().filter(|line| line.contains("::")).count(),
        0,
        "no test runs after the interrupt, so no test has a line: {stdout:?}"
    );
    assert_eq!(stdout.lines().count(), 1, "the count prints: {stdout:?}");
}
