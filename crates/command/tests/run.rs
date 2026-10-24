use std::fs;
use std::path::{Path, PathBuf};

fn module_at(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join(name)
}

fn argv(path: &Path) -> [String; 2] {
    [
        "purba".to_owned(),
        path.to_str().expect("the path is UTF-8").to_owned(),
    ]
}

#[test]
fn a_module_that_cannot_be_read_is_the_read_error() {
    let path = module_at("test_command_missing.py");
    let error = purba_command::run(&argv(&path)).expect_err("a missing module is an error");
    assert!(
        error
            .to_string()
            .starts_with(&format!("cannot read {}: ", path.display())),
        "the error names the path: {error}"
    );
}

#[test]
fn a_run_returns_the_exit_code_that_reporting_renders() {
    let path = module_at("test_command_a.py");
    let text = concat!(
        "def test_one():\n",
        "    assert 1 == 1\n",
        "\n",
        "def test_two():\n",
        "    assert 1 == 2\n",
        "\n",
        "def test_three():\n",
        "    pass\n",
    );
    fs::write(&path, text).expect("the test writes its module");
    assert_eq!(
        purba_command::run(&argv(&path)).expect("the run finishes"),
        1,
        "a passed, a failed and an unknown test exit 1"
    );
}

#[test]
fn a_module_with_no_test_exits_2() {
    let path = module_at("test_command_none.py");
    fs::write(&path, "x = 1\n").expect("the test writes its module");
    assert_eq!(
        purba_command::run(&argv(&path)).expect("the run finishes"),
        2,
        "a run with no test exits 2"
    );
}
