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

/// Each kind that a line names after its test, in reporting's words.
const KINDS: [&str; 7] = [
    "passed",
    "failed",
    "raised",
    "unknown",
    "blocked",
    "project error",
    "internal error",
];

/// Runs purba on the module `source` at `name`. Gives its path, each line but
/// the count as its test and its kind, and the exit code.
fn run_module(name: &str, source: &str) -> (PathBuf, Vec<(String, &'static str)>, u8) {
    let path = module_at(name);
    fs::write(&path, source).expect("the test writes its module");
    let mut out = Vec::new();
    let code = purba_command::run(&argv(&path), Ok(&mut out)).expect("the run finishes");
    let stdout = String::from_utf8(out).expect("reporting writes UTF-8");
    let mut lines: Vec<&str> = stdout.lines().collect();
    lines.pop();
    let shown = path.display().to_string();
    let kinds = lines
        .into_iter()
        .map(|line| {
            // The path can hold a space, and the name of a test cannot.
            let (name, rest) = line
                .strip_prefix(&shown)
                .and_then(|after| after.split_once(' '))
                .unwrap_or_else(|| panic!("a line names the module or its test: {line}"));
            let test = format!("{shown}{name}");
            let kind = KINDS
                .into_iter()
                .find(|kind| {
                    rest.strip_prefix(kind)
                        .is_some_and(|after| after.is_empty() || after.starts_with(':'))
                })
                .unwrap_or_else(|| panic!("a line names one of reporting's kinds: {line}"));
            (test, kind)
        })
        .collect();
    (path, kinds, code)
}

/// The identity of `test` in the module at `path`.
fn id(path: &Path, test: &str) -> String {
    format!("{}::{test}", path.display())
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
    let (path, kinds, code) = run_module("test_command_a.py", text);
    assert_eq!(
        kinds,
        [
            (id(&path, "test_one"), "passed"),
            (id(&path, "test_two"), "failed"),
            (id(&path, "test_three"), "unknown"),
            (id(&path, "test_four"), "raised"),
        ],
        "one line for each test, in order, to the writer"
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
        matches!(error, Error::Refused { code: 3, .. }),
        "a module that can be read gives the refused write, with the code 3: {error}"
    );
}

/// Takes each write until it holds one whole line, then refuses.
#[derive(Default)]
struct RefusesSecondLine(Vec<u8>);

impl Write for RefusesSecondLine {
    fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
        if self.0.contains(&b'\n') {
            return Err(io::Error::other("refused"));
        }
        self.0.extend_from_slice(bytes);
        Ok(bytes.len())
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

#[test]
fn a_write_refused_during_the_run_stops_the_tests_after_it() {
    let path = module_at("test_command_refused_second.py");
    let ran = module_at("test_command_refused_second.ran");
    let _ = fs::remove_file(&ran);
    let text = format!(
        "def test_one():\n    assert 1 == 1\n\n\
         def test_two():\n    assert 1 == 1\n\n\
         def test_three():\n    open({:?}, \"w\").close()\n    assert 1 == 1\n",
        ran.to_str().expect("the path is UTF-8")
    );
    fs::write(&path, text).expect("the test writes its module");
    let error = purba_command::run(&argv(&path), Ok(RefusesSecondLine::default()))
        .expect_err("a refused write is an error");
    assert_eq!(
        error.to_string(),
        "cannot write to stdout: refused",
        "the error names stdout"
    );
    assert!(!ran.exists(), "no test runs after the refused line");
}

/// Python that finds `PyO3`'s `PanicException`, which no module exports.
const PANIC_EXCEPTION: &str = "def panic_exception():\n\
    \x20   pending = [BaseException]\n\
    \x20   while pending:\n\
    \x20       kind = pending.pop()\n\
    \x20       if kind.__module__ == 'pyo3_runtime' and kind.__name__ == 'PanicException':\n\
    \x20           return kind\n\
    \x20       pending.extend(kind.__subclasses__())\n\
    \x20   raise LookupError('no PanicException')\n\n";

#[test]
fn a_write_refused_after_an_internal_error_returns_125() {
    let path = module_at("test_command_refused_after_internal.py");
    let text = format!(
        "{PANIC_EXCEPTION}\
         def test_one():\n    raise panic_exception()('raised by the test')\n\n\
         def test_two():\n    assert True\n"
    );
    fs::write(&path, text).expect("the test writes its module");
    let error = purba_command::run(&argv(&path), Ok(RefusesSecondLine::default()))
        .expect_err("a refused write is an error");
    assert!(
        matches!(error, Error::Refused { code: 125, .. }),
        "the internal error of the first line comes before the refused write: {error}"
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
fn a_test_that_selection_refuses_prints_above_the_tests_that_run() {
    let text = concat!(
        "def test_d():\n    assert True\n\n",
        "async def test_a():\n    ...\n",
    );
    let (path, kinds, code) = run_module("test_command_selection_refuses.py", text);
    assert_eq!(
        kinds,
        [
            (id(&path, "test_a"), "project error"),
            (id(&path, "test_d"), "passed"),
        ],
        "the refused line first, then the line of the test that runs"
    );
    assert_eq!(code, 4, "a test with an error exits 4");
}

#[test]
fn a_refused_line_is_written_before_any_test_runs() {
    let path = module_at("test_command_refused_first.py");
    let ran = module_at("test_command_refused_first.ran");
    let _ = fs::remove_file(&ran);
    let text = format!(
        "def test_d():\n    open({:?}, \"w\").close()\n    assert True\n\n\
         async def test_a():\n    ...\n",
        ran.to_str().expect("the path is UTF-8")
    );
    fs::write(&path, text).expect("the test writes its module");
    let error =
        purba_command::run(&argv(&path), Ok(Refuses)).expect_err("a refused write is an error");
    assert_eq!(
        error.to_string(),
        "cannot write to stdout: refused",
        "the run stops at a refused write"
    );
    assert!(!ran.exists(), "no test runs before the refused line");
}

#[test]
fn a_module_whose_every_test_is_refused_does_not_load() {
    let path = module_at("test_command_all_refused.py");
    let sentinel = module_at("test_command_all_refused.loaded");
    let _ = fs::remove_file(&sentinel);
    let text = format!(
        "open({:?}, \"w\").close()\n\nasync def test_a():\n    ...\n",
        sentinel.to_str().expect("the path is UTF-8")
    );
    fs::write(&path, text).expect("the test writes its module");
    let code = purba_command::run(&argv(&path), Ok(Vec::new())).expect("the run finishes");
    assert!(!sentinel.exists(), "the module's body never ran");
    assert_eq!(code, 4, "a test with an error exits 4");
}

#[test]
fn a_module_that_does_not_parse_prints_its_line_and_no_test() {
    for (name, text) in [
        ("test_command_bare.py", "def test_a():\n    assert\n"),
        ("test_command_eof.py", "x = (\n"),
    ] {
        let (path, kinds, code) = run_module(name, text);
        assert_eq!(
            kinds,
            [(path.display().to_string(), "project error")],
            "{name}: the module's line, and no test of it"
        );
        assert_eq!(code, 4, "{name}: a module that does not parse exits 4");
    }
}

#[test]
fn a_module_that_did_not_load_prints_its_error_once() {
    let text = concat!(
        "raise ValueError(\"x\")\n\n",
        "def test_one():\n    assert 1 == 1\n\n",
        "def test_two():\n    assert 1 == 2\n",
    );
    let (path, kinds, code) = run_module("test_command_load.py", text);
    assert_eq!(
        kinds,
        [
            (path.display().to_string(), "project error"),
            (id(&path, "test_one"), "blocked"),
            (id(&path, "test_two"), "blocked"),
        ],
        "the module's error once, then each of its tests as blocked"
    );
    assert_eq!(code, 4, "an error exits 4");
}

#[test]
fn a_test_whose_own_error_blocks_it_keeps_its_project_error_line() {
    let text = concat!(
        "def test_ok():\n    assert True\n\n",
        "async def test_async():\n    ...\n\n",
        "def test_five():\n    pass\n\n",
        "globals()[\"test_five\"] = 5\n",
    );
    let (path, kinds, code) = run_module("test_command_mix.py", text);
    assert_eq!(
        kinds,
        [
            (id(&path, "test_async"), "project error"),
            (id(&path, "test_ok"), "passed"),
            (id(&path, "test_five"), "project error"),
        ],
        "each of the two keeps its project error line"
    );
    assert_eq!(code, 4, "an error exits 4");
}
