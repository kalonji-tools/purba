//! Reporting: prints one line for each test, for each module that did not
//! load, and for the error of each thread, as it takes them, then a count, and
//! renders the exit code. Reporting prints no line for an unreached test. The
//! count names it.

use std::fmt::{self, Display};
use std::io::{self, Write};

use purba_execution::{Error, ErrorKind, Kind, Outcome, Tally};

/// A write that the writer refused, and the exit code of what the run met by
/// then: 125 after an internal error, and 3 otherwise.
#[derive(Debug)]
pub struct Refused {
    pub error: io::Error,
    pub code: u8,
}

/// The line of an error: its kind, and its text on one line.
struct Reason<'a>(&'a Error);

impl Display for Reason<'_> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let text = one_line(&self.0.to_string());
        match self.0.kind() {
            ErrorKind::Project => write!(formatter, "project error: {text}"),
            ErrorKind::Internal { part } => write!(formatter, "internal error: {part}: {text}"),
        }
    }
}

/// Prints each test beside its outcome or error to `out`, each module that did
/// not load, and the error of each thread, as it takes them, then the count.
/// Keeps only the tally.
pub struct Report<W: Write> {
    out: W,
    tally: Tally,
}

impl<W: Write> Report<W> {
    /// A report that prints to `out`.
    pub fn new(out: W) -> Self {
        Self {
            out,
            tally: Tally::default(),
        }
    }

    /// Adds `test` to the tally, beside its outcome or its own error, and
    /// writes its line and flushes it.
    ///
    /// # Errors
    ///
    /// When `out` refuses the write or the flush.
    pub fn line<T: Display>(
        &mut self,
        test: &T,
        result: &Result<Outcome, Error>,
    ) -> Result<(), Refused> {
        self.add(test, result.as_ref().map(Outcome::kind))
    }

    /// Adds an error to the tally, beside `subject`, counts no blocked test, and
    /// writes its line and flushes it. `subject` is a module that did not load
    /// or parse, or the test or the module that the error of a thread charges.
    ///
    /// # Errors
    ///
    /// When `out` refuses the write or the flush.
    pub fn error<S: Display>(&mut self, subject: &S, error: &Error) -> Result<(), Refused> {
        self.tally.add_error(error);
        let written = writeln!(self.out, "{subject} {}", Reason(error));
        self.flushed(written)
    }

    /// Adds `test` to the tally as blocked, because `module` did not load, and
    /// writes its line and flushes it.
    ///
    /// # Errors
    ///
    /// When `out` refuses the write or the flush.
    pub fn blocked<T: Display, M: Display>(&mut self, test: &T, module: &M) -> Result<(), Refused> {
        self.tally.add_blocked();
        let written = writeln!(self.out, "{test} blocked: {module} did not load");
        self.flushed(written)
    }

    /// Adds a test that the run never reached to the tally. It writes no line.
    pub const fn unreached(&mut self) {
        self.tally.add_unreached();
    }

    fn add<T: Display>(&mut self, test: &T, result: Result<&Kind, &Error>) -> Result<(), Refused> {
        self.tally.add(result);
        let out = &mut self.out;
        let written = match result {
            Ok(Kind::Passed) => writeln!(out, "{test} passed"),
            Ok(Kind::Failed { assert }) => writeln!(out, "{test} failed: {}", one_line(assert)),
            Ok(Kind::Raised { exception }) => {
                writeln!(out, "{test} raised: {}", one_line(exception))
            }
            Ok(Kind::Unknown) => writeln!(out, "{test} unknown: no assert ran"),
            Err(error) => writeln!(out, "{test} {}", Reason(error)),
        };
        self.flushed(written)
    }

    /// Flushes `out` after a write. A refused write or flush counts as an
    /// invocation error, and carries the exit code of the run so far.
    fn flushed(&mut self, written: io::Result<()>) -> Result<(), Refused> {
        written.and_then(|()| self.out.flush()).map_err(|error| {
            self.tally.add_invocation_error();
            Refused {
                error,
                code: exit_code(&self.tally),
            }
        })
    }

    /// Writes the count and flushes it. Returns the exit code of the run.
    ///
    /// # Errors
    ///
    /// When `out` refuses the write or the flush.
    pub fn finish(mut self) -> Result<u8, Refused> {
        let written = writeln!(self.out, "{}", count_line(&self.tally));
        self.flushed(written)?;
        Ok(exit_code(&self.tally))
    }
}

fn count_line(tally: &Tally) -> String {
    let tests = tally.passed
        + tally.failed
        + tally.raised
        + tally.unknown
        + tally.blocked
        + tally.unreached;
    let outcomes: Vec<String> = [
        (tally.passed, "passed"),
        (tally.failed, "failed"),
        (tally.raised, "raised"),
        (tally.unknown, "unknown"),
        (tally.blocked, "blocked"),
        (tally.unreached, "unreached"),
    ]
    .into_iter()
    .filter(|&(count, _)| count > 0)
    .map(|(count, outcome)| format!("{count} {outcome}"))
    .collect();
    let errors: Vec<String> = [
        (tally.project_errors, "project error"),
        (tally.internal_errors, "internal error"),
    ]
    .into_iter()
    .filter(|&(count, _)| count > 0)
    .map(|(count, error)| format!("{count} {error}{}", plural(count)))
    .collect();
    let mut line = format!("{tests} test{}", plural(tests));
    for (mark, columns) in [(':', outcomes), (';', errors)] {
        if !columns.is_empty() {
            line.push(mark);
            line.push(' ');
            line.push_str(&columns.join(", "));
        }
    }
    line
}

const fn exit_code(tally: &Tally) -> u8 {
    if tally.internal_errors > 0 {
        125
    } else if tally.invocation_errors > 0 {
        3
    } else if tally.failed > 0 {
        1
    } else if tally.raised > 0 || tally.project_errors > 0 {
        4
    } else if tally.unknown > 0 || tally.passed == 0 {
        2
    } else {
        0
    }
}

fn one_line(text: &str) -> String {
    text.split(['\n', '\r'])
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .collect::<Vec<_>>()
        .join(" ")
}

const fn plural(count: usize) -> &'static str {
    if count == 1 { "" } else { "s" }
}

#[cfg(test)]
mod tests {
    use std::cell::RefCell;
    use std::rc::Rc;

    use super::*;

    fn project(text: &str) -> Error {
        Error::project(text.to_owned())
    }

    /// The internal error that `catch` gives for a panic of `message`, which
    /// names `part`.
    fn internal(part: &str, message: &str) -> Error {
        purba_execution::catch(part, || -> Result<(), Error> { panic!("{message}") })
            .expect_err("a panic gives an error")
    }

    /// Reports each test beside its outcome, or beside a project error of that
    /// text. An outcome goes through `add`, which takes its kind, and an error
    /// through `line`.
    fn report_of<'a>(results: &[(&'a str, Result<&'a Kind, &'a str>)]) -> (String, u8) {
        let mut out = Vec::new();
        let mut report = Report::new(&mut out);
        for (test, result) in results {
            match result {
                Ok(kind) => report.add(test, Ok(*kind)),
                Err(text) => report.line(test, &Err(project(text))),
            }
            .expect("a Vec takes every write");
        }
        let code = report.finish().expect("a Vec takes every write");
        (
            String::from_utf8(out).expect("reporting writes UTF-8"),
            code,
        )
    }

    /// Holds what it takes until a flush, and then shows it in `seen`.
    struct Flushed {
        pending: Vec<u8>,
        seen: Rc<RefCell<String>>,
    }

    impl Write for Flushed {
        fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
            self.pending.extend_from_slice(bytes);
            Ok(bytes.len())
        }

        fn flush(&mut self) -> io::Result<()> {
            let text =
                String::from_utf8(self.pending.split_off(0)).expect("reporting writes UTF-8");
            self.seen.borrow_mut().push_str(&text);
            Ok(())
        }
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
    fn each_line_is_flushed_before_the_next_test() {
        let seen = Rc::new(RefCell::new(String::new()));
        let mut report = Report::new(Flushed {
            pending: Vec::new(),
            seen: Rc::clone(&seen),
        });
        report
            .add(&"test_a.py::test_one", Ok(&Kind::Passed))
            .expect("the writer takes every write");
        assert_eq!(
            *seen.borrow(),
            "test_a.py::test_one passed\n",
            "the first line is flushed before the second test"
        );
        report
            .add(&"test_a.py::test_two", Ok(&Kind::Unknown))
            .expect("the writer takes every write");
        assert_eq!(
            *seen.borrow(),
            "test_a.py::test_one passed\n\
             test_a.py::test_two unknown: no assert ran\n",
            "the second line is flushed too, and no count yet"
        );
        let code = report.finish().expect("the writer takes every write");
        assert_eq!(
            *seen.borrow(),
            "test_a.py::test_one passed\n\
             test_a.py::test_two unknown: no assert ran\n\
             2 tests: 1 passed, 1 unknown\n",
            "the count is flushed when the run finishes"
        );
        assert_eq!(code, 2, "an unknown test exits 2");
    }

    #[test]
    fn a_refused_write_is_the_error_of_its_line_with_the_code_3() {
        let mut report = Report::new(RefusesSecondLine::default());
        report
            .add(&"test_a.py::test_one", Ok(&Kind::Passed))
            .expect("the writer takes the first line");
        let refused = report
            .add(&"test_a.py::test_two", Ok(&Kind::Passed))
            .expect_err("the writer refuses the second line");
        assert_eq!(
            (refused.error.to_string(), refused.code),
            ("refused".to_owned(), 3),
            "the writer's own error, and 3 for a run that met no internal error"
        );
    }

    #[test]
    fn a_refused_count_is_the_error_of_finish() {
        let mut report = Report::new(RefusesSecondLine::default());
        report
            .add(&"test_a.py::test_one", Ok(&Kind::Passed))
            .expect("the writer takes the first line");
        let refused = report.finish().expect_err("the writer refuses the count");
        assert_eq!(
            (refused.error.to_string(), refused.code),
            ("refused".to_owned(), 3),
            "the writer's own error, and 3"
        );
    }

    #[test]
    fn a_write_refused_after_an_internal_error_keeps_125() {
        let mut report = Report::new(RefusesSecondLine::default());
        report
            .line(
                &"test_a.py::test_one",
                &Err(internal("assertion", "PURBA_FAULT=assert")),
            )
            .expect("the writer takes the first line");
        let refused = report
            .add(&"test_a.py::test_two", Ok(&Kind::Passed))
            .expect_err("the writer refuses the second line");
        assert_eq!(
            refused.code, 125,
            "an internal error comes before the invocation error of a refused write"
        );
    }

    #[test]
    fn an_error_that_another_part_finds_prints_as_a_project_error() {
        let mut out = Vec::new();
        let mut report = Report::new(&mut out);
        report
            .line(
                &"test_a.py::test_one",
                &Err(project(
                    "the parameter x has no default, and purba cannot see which fixture builds it",
                )),
            )
            .expect("a Vec takes every write");
        let code = report.finish().expect("a Vec takes every write");
        assert_eq!(
            String::from_utf8(out).expect("reporting writes UTF-8"),
            "test_a.py::test_one project error: the parameter x has no default, \
             and purba cannot see which fixture builds it\n\
             1 test: 1 blocked; 1 project error\n",
            "the line of a test whose own error blocks it, and the count of both"
        );
        assert_eq!(code, 4, "an error makes the run exit 4");
    }

    #[test]
    fn a_module_that_did_not_load_prints_once_and_each_test_as_blocked() {
        let mut out = Vec::new();
        let mut report = Report::new(&mut out);
        report
            .error(&"test_b.py", &project("ValueError: x"))
            .expect("a Vec takes every write");
        for test in ["test_b.py::test_one", "test_b.py::test_two"] {
            report
                .blocked(&test, &"test_b.py")
                .expect("a Vec takes every write");
        }
        let code = report.finish().expect("a Vec takes every write");
        assert_eq!(
            String::from_utf8(out).expect("reporting writes UTF-8"),
            "test_b.py project error: ValueError: x\n\
             test_b.py::test_one blocked: test_b.py did not load\n\
             test_b.py::test_two blocked: test_b.py did not load\n\
             2 tests: 2 blocked; 1 project error\n",
            "the error once, then each test that it blocked"
        );
        assert_eq!(
            code, 4,
            "the error sets the code, and a blocked test changes nothing"
        );
    }

    #[test]
    fn an_error_that_blocks_no_test_prints_under_its_test_and_keeps_the_tests_outcome() {
        let mut out = Vec::new();
        let mut report = Report::new(&mut out);
        report
            .add(&"test_t.py::test_one", Ok(&Kind::Passed))
            .expect("a Vec takes every write");
        report
            .error(
                &"test_t.py::test_one",
                &project("thread T raised ValueError: late"),
            )
            .expect("a Vec takes every write");
        let code = report.finish().expect("a Vec takes every write");
        assert_eq!(
            String::from_utf8(out).expect("reporting writes UTF-8"),
            "test_t.py::test_one passed\n\
             test_t.py::test_one project error: thread T raised ValueError: late\n\
             1 test: 1 passed; 1 project error\n",
            "the test's line, then the error on its own line, counted once and blocking no test"
        );
        assert_eq!(code, 4, "the project error makes the run exit 4");
    }

    #[test]
    fn a_test_that_an_internal_error_stops_prints_it_and_counts_as_blocked() {
        let mut out = Vec::new();
        let mut report = Report::new(&mut out);
        let failed = Kind::Failed {
            assert: "assert 1 == 2".to_owned(),
        };
        report
            .add(&"test_a.py::test_one", Ok(&Kind::Passed))
            .expect("a Vec takes every write");
        report
            .line(
                &"test_a.py::test_two",
                &Err(internal("assertion", "PURBA_FAULT=assert")),
            )
            .expect("a Vec takes every write");
        report
            .add(&"test_a.py::test_three", Ok(&failed))
            .expect("a Vec takes every write");
        let code = report.finish().expect("a Vec takes every write");
        assert_eq!(
            String::from_utf8(out).expect("reporting writes UTF-8"),
            "test_a.py::test_one passed\n\
             test_a.py::test_two internal error: assertion: PURBA_FAULT=assert\n\
             test_a.py::test_three failed: assert 1 == 2\n\
             3 tests: 1 passed, 1 failed, 1 blocked; 1 internal error\n",
            "the internal error in the place of its test, and the count of both"
        );
        assert_eq!(code, 125, "an internal error comes before a failed test");
    }

    #[test]
    fn a_module_that_an_internal_error_stops_prints_once_and_each_test_as_blocked() {
        let mut out = Vec::new();
        let mut report = Report::new(&mut out);
        report
            .error(&"test_m.py", &internal("dispatch", "boom\n  at the load"))
            .expect("a Vec takes every write");
        for test in ["test_m.py::test_one", "test_m.py::test_two"] {
            report
                .blocked(&test, &"test_m.py")
                .expect("a Vec takes every write");
        }
        let code = report.finish().expect("a Vec takes every write");
        assert_eq!(
            String::from_utf8(out).expect("reporting writes UTF-8"),
            "test_m.py internal error: dispatch: boom at the load\n\
             test_m.py::test_one blocked: test_m.py did not load\n\
             test_m.py::test_two blocked: test_m.py did not load\n\
             2 tests: 2 blocked; 1 internal error\n",
            "the error once, on one line, then each test that it blocked"
        );
        assert_eq!(code, 125, "an internal error exits 125");
    }

    #[test]
    fn a_test_whose_own_error_blocks_it_prints_a_project_error_and_counts_as_blocked() {
        let mut out = Vec::new();
        let mut report = Report::new(&mut out);
        for (test, text) in [
            (
                "test_a.py::test_async",
                "purba cannot run an async test yet",
            ),
            (
                "test_a.py::test_five",
                "TypeError: 'int' object is not callable",
            ),
        ] {
            report
                .line(&test, &Err(project(text)))
                .expect("a Vec takes every write");
        }
        let code = report.finish().expect("a Vec takes every write");
        assert_eq!(
            String::from_utf8(out).expect("reporting writes UTF-8"),
            "test_a.py::test_async project error: purba cannot run an async test yet\n\
             test_a.py::test_five project error: TypeError: 'int' object is not callable\n\
             2 tests: 2 blocked; 2 project errors\n",
            "each line keeps project error, and each test counts as blocked and as an error"
        );
        assert_eq!(code, 4, "an error exits 4");
    }

    #[test]
    fn each_test_prints_its_outcome_or_error_and_the_count() {
        let failed = Kind::Failed {
            assert: "assert 1 == 2".to_owned(),
        };
        let raised = Kind::Raised {
            exception: "KeyError: 'k'".to_owned(),
        };
        let (text, code) = report_of(&[
            ("test_a.py::test_one", Ok(&Kind::Passed)),
            ("test_a.py::test_two", Ok(&failed)),
            ("test_a.py::test_three", Ok(&Kind::Unknown)),
            ("test_a.py::test_four", Ok(&raised)),
            ("test_c.py::test_x", Err("ValueError: boom")),
        ]);
        assert_eq!(
            text,
            "test_a.py::test_one passed\n\
             test_a.py::test_two failed: assert 1 == 2\n\
             test_a.py::test_three unknown: no assert ran\n\
             test_a.py::test_four raised: KeyError: 'k'\n\
             test_c.py::test_x project error: ValueError: boom\n\
             5 tests: 1 passed, 1 failed, 1 raised, 1 unknown, 1 blocked; 1 project error\n",
            "one line for each test, in order, then the count"
        );
        assert_eq!(code, 1, "a failed test makes the run exit 1");
    }

    #[test]
    fn the_exit_code_names_the_worst_of_the_run() {
        let tally = |passed, failed, raised, unknown, project_errors| Tally {
            passed,
            failed,
            raised,
            unknown,
            blocked: 0,
            project_errors,
            internal_errors: 0,
            invocation_errors: 0,
            unreached: 0,
        };
        assert_eq!(
            exit_code(&Tally {
                internal_errors: 1,
                invocation_errors: 1,
                ..tally(1, 1, 1, 1, 1)
            }),
            125,
            "an internal error comes first"
        );
        assert_eq!(
            exit_code(&Tally {
                invocation_errors: 1,
                ..tally(1, 1, 1, 1, 1)
            }),
            3,
            "then an invocation error"
        );
        assert_eq!(exit_code(&tally(1, 1, 1, 1, 1)), 1, "then a failed test");
        assert_eq!(
            exit_code(&tally(1, 0, 1, 1, 0)),
            4,
            "then a test that raised"
        );
        assert_eq!(exit_code(&tally(1, 0, 0, 1, 1)), 4, "or an error");
        assert_eq!(exit_code(&tally(1, 0, 0, 1, 0)), 2, "then an unknown test");
        assert_eq!(
            exit_code(&tally(0, 0, 0, 0, 0)),
            2,
            "then a run with no test"
        );
        assert_eq!(exit_code(&tally(2, 0, 0, 0, 0)), 0, "every test passed");
    }

    #[test]
    fn a_reason_over_several_lines_prints_on_one() {
        let failed = Kind::Failed {
            assert: "assert (\n    total  ==  3\n)".to_owned(),
        };
        let raised = Kind::Raised {
            exception: "ValueError: a\n  b".to_owned(),
        };
        let (text, _) = report_of(&[
            ("test_a.py::test_one", Ok(&failed)),
            ("test_a.py::test_two", Err("ValueError: a \r\n\n  b")),
            ("test_a.py::test_three", Err("ValueError: a\rb")),
            ("test_a.py::test_four", Err("ValueError: boom\n")),
            ("test_a.py::test_five", Ok(&raised)),
        ]);
        assert_eq!(
            text.lines().take(5).collect::<Vec<_>>(),
            [
                "test_a.py::test_one failed: assert ( total  ==  3 )",
                "test_a.py::test_two project error: ValueError: a b",
                "test_a.py::test_three project error: ValueError: a b",
                "test_a.py::test_four project error: ValueError: boom",
                "test_a.py::test_five raised: ValueError: a b",
            ],
            "a lone \\r breaks a line too, and the ends of a reason lose their whitespace"
        );
    }

    #[test]
    fn an_unreached_test_prints_no_line_and_the_count_names_it_after_blocked() {
        let raised = Kind::Raised {
            exception: "KeyboardInterrupt: ".to_owned(),
        };
        let mut out = Vec::new();
        let mut report = Report::new(&mut out);
        report
            .add(&"test_a.py::test_one", Ok(&Kind::Passed))
            .expect("a Vec takes every write");
        report
            .line(&"test_a.py::test_two", &Err(project("NameError: x")))
            .expect("a Vec takes every write");
        report
            .add(&"test_a.py::test_three", Ok(&raised))
            .expect("a Vec takes every write");
        report.unreached();
        report.finish().expect("a Vec takes every write");
        assert_eq!(
            String::from_utf8(out).expect("reporting writes UTF-8"),
            "test_a.py::test_one passed\n\
             test_a.py::test_two project error: NameError: x\n\
             test_a.py::test_three raised: KeyboardInterrupt:\n\
             4 tests: 1 passed, 1 raised, 1 blocked, 1 unreached; 1 project error\n",
            "the unreached test has no line, and the count names it after blocked"
        );
    }

    #[test]
    fn the_count_names_each_column_only_above_0() {
        assert_eq!(
            count_line(&Tally::default()),
            "0 tests",
            "no column is above 0"
        );
        assert_eq!(
            count_line(&Tally {
                passed: 3,
                ..Tally::default()
            }),
            "3 tests: 3 passed",
            "one column"
        );
        assert_eq!(
            count_line(&Tally {
                passed: 1,
                failed: 1,
                unknown: 1,
                ..Tally::default()
            }),
            "3 tests: 1 passed, 1 failed, 1 unknown",
            "a column at 0 between two others"
        );
    }

    #[test]
    fn the_count_names_project_errors_apart_from_internal_errors() {
        assert_eq!(
            count_line(&Tally {
                project_errors: 1,
                ..Tally::default()
            }),
            "0 tests; 1 project error",
            "an error, and no test"
        );
        assert_eq!(
            count_line(&Tally {
                passed: 1,
                blocked: 2,
                project_errors: 1,
                internal_errors: 1,
                ..Tally::default()
            }),
            "3 tests: 1 passed, 2 blocked; 1 project error, 1 internal error",
            "both kinds of error, after the tests"
        );
        assert_eq!(
            count_line(&Tally {
                blocked: 2,
                project_errors: 2,
                ..Tally::default()
            }),
            "2 tests: 2 blocked; 2 project errors",
            "a kind of error takes an s above 1"
        );
    }
}
