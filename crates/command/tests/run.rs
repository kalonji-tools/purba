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
    let error =
        purba_command::run(&argv(&path), Ok(Vec::new())).expect_err("a missing module is an error");
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
        "\n",
        "def test_four():\n",
        "    raise ValueError(\"boom\")\n",
    );
    fs::write(&path, text).expect("the test writes its module");
    let mut out = Vec::new();
    let code = purba_command::run(&argv(&path), Ok(&mut out)).expect("the run finishes");
    assert_eq!(
        String::from_utf8(out).expect("reporting writes UTF-8"),
        format!(
            "{0}::test_one passed\n\
             {0}::test_two failed: assert 1 == 2\n\
             {0}::test_three unknown: no assert ran\n\
             {0}::test_four raised: ValueError: boom\n\
             4 tests: 1 passed, 1 failed, 1 raised, 1 unknown, 0 errors\n",
            path.display()
        ),
        "one line for each test, in order, then the count, to the writer"
    );
    assert_eq!(
        code, 1,
        "a passed, a failed, an unknown and a raised test exit 1"
    );
}

#[test]
fn a_writer_that_refuses_is_the_write_error() {
    let path = module_at("test_command_refused.py");
    fs::write(&path, "x = 1\n").expect("the test writes its module");
    let error =
        purba_command::run(&argv(&path), Ok(Refuses)).expect_err("a refused write is an error");
    assert!(
        matches!(error, Error::Write(_)),
        "a module that can be read gives the write error: {error}"
    );
}

#[test]
fn a_stdout_that_takes_no_write_stops_the_run_before_any_test() {
    let path = module_at("test_command_closed.py");
    let ran = module_at("test_command_closed.ran");
    let _ = fs::remove_file(&ran);
    let text = format!(
        "def test_a():\n    open({:?}, \"w\").close()\n    assert 1 == 1\n",
        ran.to_str().expect("the path is UTF-8")
    );
    fs::write(&path, text).expect("the test writes its module");
    let error = purba_command::run(&argv(&path), Err::<Vec<u8>, _>(io::Error::other("closed")))
        .expect_err("a stdout that takes no write is an error");
    assert_eq!(
        error.to_string(),
        "cannot write to stdout: closed",
        "the error names stdout"
    );
    assert!(!ran.exists(), "no test runs");
}

#[test]
fn a_usage_error_comes_before_a_stdout_that_takes_no_write() {
    let error = purba_command::run(
        &["purba".to_owned()],
        Err::<Vec<u8>, _>(io::Error::other("closed")),
    )
    .expect_err("no path is an error");
    assert!(
        matches!(error, Error::Usage),
        "the usage error comes first: {error}"
    );
}

#[test]
fn a_module_with_no_test_exits_2() {
    let path = module_at("test_command_none.py");
    fs::write(&path, "x = 1\n").expect("the test writes its module");
    assert_eq!(
        purba_command::run(&argv(&path), Ok(Vec::new())).expect("the run finishes"),
        2,
        "a run with no test exits 2"
    );
}

#[test]
fn a_module_that_does_not_parse_shows_its_syntax_error() {
    for (name, text, line) in [
        ("test_command_bare.py", "def test_a():\n    assert\n", 2),
        (
            "test_command_star.py",
            "def test_a():\n    x = [1]\n    assert *x\n",
            3,
        ),
        (
            "test_command_dstar.py",
            "def test_a():\n    x = {\"value\": 1}\n    assert **x\n",
            3,
        ),
    ] {
        let path = module_at(name);
        fs::write(&path, text).expect("the test writes its module");
        let mut out = Vec::new();
        let code = purba_command::run(&argv(&path), Ok(&mut out)).expect("the run finishes");
        assert_eq!(
            String::from_utf8(out).expect("reporting writes UTF-8"),
            format!(
                "{}::test_a error: SyntaxError: invalid syntax ({name}, line {line})\n\
                 1 test: 0 passed, 0 failed, 0 raised, 0 unknown, 1 error\n",
                path.display()
            ),
            "{name}: CPython's syntax error, as for any module that does not parse"
        );
        assert_eq!(code, 4, "{name}: a test with an error exits 4");
    }
}

#[test]
fn a_module_that_nests_too_deep_shows_cpythons_error() {
    let path = module_at("test_command_deep.py");
    let text = format!(
        "def test_a():\n    x = {}1\n    assert x\n",
        "-".repeat(200_000)
    );
    fs::write(&path, text).expect("the test writes its module");
    let mut out = Vec::new();
    let code = purba_command::run(&argv(&path), Ok(&mut out)).expect("the run finishes");
    assert_eq!(
        String::from_utf8(out).expect("reporting writes UTF-8"),
        format!(
            "{}::test_a error: MemoryError: Parser stack overflowed - \
             Python source too complex to parse\n\
             1 test: 0 passed, 0 failed, 0 raised, 0 unknown, 1 error\n",
            path.display()
        ),
        "CPython's error, as for any module that does not compile"
    );
    assert_eq!(code, 4, "a test with an error exits 4");
}

#[test]
fn a_module_that_defines_the_hooks_name_fails_to_load() {
    for (name, hook) in [
        ("test_command_shadow.py", "_purba_ran"),
        ("test_command_shadow_full_width.py", "_\u{ff50}urba_ran"),
    ] {
        let path = module_at(name);
        let text = format!(
            "def {hook}(i, x):\n    return True\n\ndef test_shadow():\n    assert 1 == 2\n"
        );
        fs::write(&path, text).expect("the test writes its module");
        let mut out = Vec::new();
        let code = purba_command::run(&argv(&path), Ok(&mut out)).expect("the run finishes");
        assert_eq!(
            String::from_utf8(out).expect("reporting writes UTF-8"),
            format!(
                "{0}::test_shadow error: NameError: {0} binds _purba_ran, \
                 the name of purba's assert hook\n\
                 1 test: 0 passed, 0 failed, 0 raised, 0 unknown, 1 error\n",
                path.display()
            ),
            "{name}: the load names the clash"
        );
        assert_eq!(code, 4, "{name}: a test with an error exits 4");
    }
}
