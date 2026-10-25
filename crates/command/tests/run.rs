use std::fs;
use std::io::{self, Write};
use std::path::{Path, PathBuf};

use purba_command::Error;

fn module_at(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join(name)
}

fn argv(path: &Path) -> [String; 2] {
    [
        "purba".to_owned(),
        path.to_str().expect("the path is UTF-8").to_owned(),
    ]
}

struct Refuses;

impl Write for Refuses {
    fn write(&mut self, _: &[u8]) -> io::Result<usize> {
        Err(io::Error::other("refused"))
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

#[test]
fn a_module_that_cannot_be_read_is_the_read_error() {
    let path = module_at("test_command_missing.py");
    let error = purba_command::run(&argv(&path), &mut Vec::new())
        .expect_err("a missing module is an error");
    assert!(
        error
            .to_string()
            .starts_with(&format!("cannot read {}: ", path.display())),
        "the error names the path: {error}"
    );
}

#[test]
fn a_run_writes_its_lines_and_returns_the_exit_code() {
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
    let mut out = Vec::new();
    let code = purba_command::run(&argv(&path), &mut out).expect("the run finishes");
    assert_eq!(
        String::from_utf8(out).expect("reporting writes UTF-8"),
        format!(
            "{0}::test_one passed\n\
             {0}::test_two failed: assert 1 == 2\n\
             {0}::test_three unknown: no assert ran\n\
             3 tests: 1 passed, 1 failed, 1 unknown, 0 errors\n",
            path.display()
        ),
        "one line for each test, in order, then the count, to the writer"
    );
    assert_eq!(code, 1, "a passed, a failed and an unknown test exit 1");
}

#[test]
fn a_writer_that_refuses_is_the_write_error() {
    let path = module_at("test_command_refused.py");
    fs::write(&path, "x = 1\n").expect("the test writes its module");
    let error =
        purba_command::run(&argv(&path), &mut Refuses).expect_err("a refused write is an error");
    assert!(
        matches!(error, Error::Write(_)),
        "a module that can be read gives the write error: {error}"
    );
}

#[test]
fn a_module_with_no_test_exits_2() {
    let path = module_at("test_command_none.py");
    fs::write(&path, "x = 1\n").expect("the test writes its module");
    assert_eq!(
        purba_command::run(&argv(&path), &mut Vec::new()).expect("the run finishes"),
        2,
        "a run with no test exits 2"
    );
}
