use std::fs;
use std::path::PathBuf;

use pyo3::Python;

// purba's signal handler keeps the first signal for the life of the process.
// So the signal of this binary lives in this one test.
#[test]
fn an_interrupt_in_the_rewrite_leaves_each_test_unreached() {
    purba_execution::interrupt::install().expect("purba's signal handler takes SIGINT");
    let path = PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join("test_interrupt_in_rewrite.py");
    fs::write(
        &path,
        "def test_one():\n    assert True\ndef test_two():\n    assert True\n",
    )
    .expect("the test writes its module");
    let argv = [
        "purba".to_owned(),
        path.to_str().expect("the path is UTF-8").to_owned(),
    ];
    let mut out = Vec::new();
    let result = purba_command::run_with(&argv, Ok(&mut out), |module| {
        Python::attach(|py| {
            py.run(
                c"import signal\nsignal.raise_signal(signal.SIGINT)",
                None,
                None,
            )
        })
        .expect("purba's signal handler raises nothing outside a call");
        purba_assertion::rewrite(module)
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
