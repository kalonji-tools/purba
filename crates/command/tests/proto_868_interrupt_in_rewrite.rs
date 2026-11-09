//! PROTOTYPE #868, seam "a command test hands in the rewrite". Never merge.
//!
//! purba's signal handler keeps the first signal for the life of the process,
//! so this test has a binary of its own.

use std::fs;
use std::path::PathBuf;

use purba_command::Error;
use pyo3::Python;

#[test]
fn an_interrupt_in_the_rewrite_leaves_each_test_unreached_and_prints_the_count() {
    purba_execution::interrupt::install().expect("purba's signal handler takes SIGINT");
    let path = PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join("test_proto_868.py");
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
        !matches!(result, Err(Error::Interrupted)),
        "an interrupt after selection does not stop the run before the count: {result:?}"
    );
    assert!(
        !stdout.contains("::test_"),
        "no test runs after the interrupt, so no test has a line: {stdout:?}"
    );
    assert!(
        !stdout.is_empty(),
        "the count prints, since selection knows the tests: {stdout:?}"
    );
}
