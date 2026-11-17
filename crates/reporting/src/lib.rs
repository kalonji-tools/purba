//! Reporting: prints one line for each test, for each module that did not
//! load, and for the error of each thread, as it takes them, then a count, and
//! renders the exit code. Reporting prints `interrupted` for a test that an
//! interrupt landed in. It prints no line for an unreached test. The count
//! names it.

use std::fmt::{self, Display, Write as _};
use std::io::{self, Write};
use std::iter;

use purba_execution::{Error, ErrorKind, Kind, Outcome, Shown, Tally};

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
    /// writes its line and flushes it. `subject` is a module that an error
    /// stopped before any test of it ran, or the test or the module that the
    /// error of a thread charges.
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

    /// Writes `words`, a notice of the case, under its line as `⚠ <words>`. It
    /// changes no count and no exit code.
    ///
    /// # Errors
    ///
    /// When `out` refuses the write or the flush.
    pub fn noticed(&mut self, words: &str) -> Result<(), Refused> {
        let written = writeln!(self.out, "    \u{26a0} {}", one_line(words));
        self.flushed(written)
    }

    /// Adds one suppressed verdict of `check` to the tally. Writes it under the
    /// line of its case as `⚠ suppressed [<check>] <words>`, then `reason:
    /// <reason>` on the next line. It changes no exit code.
    ///
    /// # Errors
    ///
    /// When `out` refuses the write or the flush.
    pub fn warned(&mut self, check: &str, words: &str, reason: &str) -> Result<(), Refused> {
        self.tally.add_suppressed();
        let written = write!(
            self.out,
            "    \u{26a0} suppressed [{check}] {}\n      reason: {}\n",
            one_line(words),
            one_line(reason)
        );
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
            Ok(Kind::Failed { asserts }) => {
                let mut lines = format!("{test} failed\n");
                for quote in asserts {
                    let _ = writeln!(lines, "    {}", one_line(&quote.assert));
                    match &quote.shown {
                        Some(Shown::Values { expected, reading }) => {
                            let _ = writeln!(lines, "      expected  {}", one_line(expected));
                            let _ = writeln!(lines, "      reading   {}", one_line(reading));
                        }
                        Some(Shown::Items { heading, items }) => {
                            for line in iter::once(heading).chain(items) {
                                let _ = writeln!(lines, "      {}", one_line(line));
                            }
                        }
                        None => {}
                    }
                }
                out.write_all(lines.as_bytes())
            }
            Ok(Kind::Raised { exception, whence }) => {
                let mut lines = format!("{test} raised: {}\n", one_line(exception));
                if let Some(whence) = whence {
                    let _ = writeln!(lines, "    {}", one_line(whence));
                }
                out.write_all(lines.as_bytes())
            }
            Ok(Kind::Interrupted) => writeln!(out, "{test} interrupted"),
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
    let cases = tally.passed
        + tally.failed
        + tally.raised
        + tally.interrupted
        + tally.blocked
        + tally.unreached;
    let outcomes: Vec<String> = [
        (tally.passed, "passed"),
        (tally.failed, "failed"),
        (tally.raised, "raised"),
        (tally.interrupted, "interrupted"),
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
    let mut line = format!("{cases} case{}", plural(cases));
    let suppressed: Vec<String> = (tally.suppressed > 0)
        .then(|| format!("{} suppressed", tally.suppressed))
        .into_iter()
        .collect();
    for (mark, columns) in [(':', outcomes), (';', errors), (';', suppressed)] {
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
    } else if tally.passed == 0 {
        2
    } else {
        0
    }
}

/// Each character that Python's `str.splitlines` reads as a line break.
/// <https://docs.python.org/3.12/library/stdtypes.html#str.splitlines>
const BREAKS: [char; 10] = [
    '\n', '\r', '\u{b}', '\u{c}', '\u{1c}', '\u{1d}', '\u{1e}', '\u{85}', '\u{2028}', '\u{2029}',
];

fn one_line(text: &str) -> String {
    let joined = text
        .split(BREAKS)
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .collect::<Vec<_>>()
        .join(" ");
    let mut line = String::with_capacity(joined.len());
    for character in joined.chars() {
        match character {
            '\t' => line.push_str("\\t"),
            control if control.is_control() => {
                let _ = write!(line, "\\x{:02x}", u32::from(control));
            }
            other => line.push(other),
        }
    }
    line
}

const fn plural(count: usize) -> &'static str {
    if count == 1 { "" } else { "s" }
}

#[cfg(test)]
mod tests {
    use std::cell::RefCell;
    use std::rc::Rc;

    use super::*;
    use purba_execution::{Quote, Shown};

    /// The quote of `assert`, with the expected value and the reading that
    /// `shown` names.
    fn quote(assert: &str, shown: Option<(&str, &str)>) -> Quote {
        Quote {
            assert: assert.to_owned(),
            shown: shown.map(|(expected, reading)| Shown::Values {
                expected: expected.to_owned(),
                reading: reading.to_owned(),
            }),
        }
    }

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
        let raised = Kind::Raised {
            exception: "KeyError: 'k'".to_owned(),
            whence: None,
        };
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
            .add(&"test_a.py::test_two", Ok(&raised))
            .expect("the writer takes every write");
        assert_eq!(
            *seen.borrow(),
            "test_a.py::test_one passed\n\
             test_a.py::test_two raised: KeyError: 'k'\n",
            "the second line is flushed too, and no count yet"
        );
        let code = report.finish().expect("the writer takes every write");
        assert_eq!(
            *seen.borrow(),
            "test_a.py::test_one passed\n\
             test_a.py::test_two raised: KeyError: 'k'\n\
             2 cases: 1 passed, 1 raised\n",
            "the count is flushed when the run finishes"
        );
        assert_eq!(code, 4, "a raised case exits 4");
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
             1 case: 1 blocked; 1 project error\n",
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
             2 cases: 2 blocked; 1 project error\n",
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
             1 case: 1 passed; 1 project error\n",
            "the test's line, then the error on its own line, counted once and blocking no test"
        );
        assert_eq!(code, 4, "the project error makes the run exit 4");
    }

    #[test]
    fn a_test_that_an_internal_error_stops_prints_it_and_counts_as_blocked() {
        let mut out = Vec::new();
        let mut report = Report::new(&mut out);
        let failed = Kind::Failed {
            asserts: vec![quote("assert 1 == 2", None)],
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
             test_a.py::test_three failed\n\
             \x20   assert 1 == 2\n\
             3 cases: 1 passed, 1 failed, 1 blocked; 1 internal error\n",
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
             2 cases: 2 blocked; 1 internal error\n",
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
             2 cases: 2 blocked; 2 project errors\n",
            "each line keeps project error, and each test counts as blocked and as an error"
        );
        assert_eq!(code, 4, "an error exits 4");
    }

    #[test]
    fn a_raised_case_prints_the_frame_of_its_raise_on_the_line_below() {
        let raised = Kind::Raised {
            exception: "ValueError: boom".to_owned(),
            whence: Some("raised inside the generator gen() at m.py:3".to_owned()),
        };

        let (printed, code) = report_of(&[("m.py::test_gen", Ok(&raised))]);

        assert_eq!(
            printed,
            "m.py::test_gen raised: ValueError: boom\n    \
             raised inside the generator gen() at m.py:3\n1 case: 1 raised\n",
            "the frame, indented, below the line of the case"
        );
        assert_eq!(code, 4, "a raised case exits 4");
    }

    #[test]
    fn a_failed_all_prints_its_heading_and_each_item_under_its_quote() {
        let failed = Kind::Failed {
            asserts: vec![Quote {
                assert: "assert all(len(c) == 2 for c in parts)".to_owned(),
                shown: Some(Shown::Items {
                    heading: "the first item that breaks  len(c) == 2".to_owned(),
                    items: vec!["[2]  len(c)  1     c  'e'".to_owned()],
                }),
            }],
        };

        let (printed, code) = report_of(&[("m.py::test_each", Ok(&failed))]);

        assert_eq!(
            printed,
            "m.py::test_each failed\n    assert all(len(c) == 2 for c in parts)\n      \
             the first item that breaks  len(c) == 2\n      [2]  len(c)  1     c  'e'\n\
             1 case: 1 failed\n",
            "the heading, then each item, under the quote"
        );
        assert_eq!(code, 1, "a failed case exits 1");
    }

    #[test]
    fn each_case_prints_its_outcome_or_error_and_the_count() {
        let failed = Kind::Failed {
            asserts: vec![quote("assert 1 == 2", Some(("2", "1")))],
        };
        let raised = Kind::Raised {
            exception: "KeyError: 'k'".to_owned(),
            whence: None,
        };
        let (text, code) = report_of(&[
            ("test_a.py::test_one", Ok(&Kind::Passed)),
            ("test_a.py::test_two", Ok(&failed)),
            ("test_a.py::test_four", Ok(&raised)),
            ("test_c.py::test_x", Err("ValueError: boom")),
        ]);
        assert_eq!(
            text,
            "test_a.py::test_one passed\n\
             test_a.py::test_two failed\n\
             \x20   assert 1 == 2\n\
             \x20     expected  2\n\
             \x20     reading   1\n\
             test_a.py::test_four raised: KeyError: 'k'\n\
             test_c.py::test_x project error: ValueError: boom\n\
             4 cases: 1 passed, 1 failed, 1 raised, 1 blocked; 1 project error\n",
            "the lines of each case, in order, then the count"
        );
        assert_eq!(code, 1, "a failed test makes the run exit 1");
    }

    #[test]
    fn the_exit_code_names_the_worst_of_the_run() {
        let tally = |passed, failed, raised, project_errors| Tally {
            passed,
            failed,
            raised,
            interrupted: 0,
            blocked: 0,
            project_errors,
            internal_errors: 0,
            invocation_errors: 0,
            unreached: 0,
            suppressed: 0,
        };
        assert_eq!(
            exit_code(&Tally {
                internal_errors: 1,
                invocation_errors: 1,
                ..tally(1, 1, 1, 1)
            }),
            125,
            "an internal error comes first"
        );
        assert_eq!(
            exit_code(&Tally {
                invocation_errors: 1,
                ..tally(1, 1, 1, 1)
            }),
            3,
            "then an invocation error"
        );
        assert_eq!(exit_code(&tally(1, 1, 1, 1)), 1, "then a failed test");
        assert_eq!(exit_code(&tally(1, 0, 1, 0)), 4, "then a test that raised");
        assert_eq!(exit_code(&tally(1, 0, 0, 1)), 4, "or an error");
        assert_eq!(
            exit_code(&tally(0, 0, 0, 0)),
            2,
            "then a run in which no case passed"
        );
        assert_eq!(exit_code(&tally(2, 0, 0, 0)), 0, "every case passed");
    }

    #[test]
    fn a_reason_over_several_lines_prints_on_one() {
        let failed = Kind::Failed {
            asserts: vec![quote("assert (\n    total  ==  3\n)", None)],
        };
        let raised = Kind::Raised {
            exception: "ValueError: a\n  b".to_owned(),
            whence: None,
        };
        let (text, _) = report_of(&[
            ("test_a.py::test_one", Ok(&failed)),
            ("test_a.py::test_two", Err("ValueError: a \r\n\n  b")),
            ("test_a.py::test_three", Err("ValueError: a\rb")),
            ("test_a.py::test_four", Err("ValueError: boom\n")),
            ("test_a.py::test_five", Ok(&raised)),
        ]);
        assert_eq!(
            text.lines().take(6).collect::<Vec<_>>(),
            [
                "test_a.py::test_one failed",
                "    assert ( total  ==  3 )",
                "test_a.py::test_two project error: ValueError: a b",
                "test_a.py::test_three project error: ValueError: a b",
                "test_a.py::test_four project error: ValueError: boom",
                "test_a.py::test_five raised: ValueError: a b",
            ],
            "a lone \\r breaks a line too, and the ends of a reason lose their whitespace"
        );
    }

    #[test]
    fn a_reason_breaks_at_each_line_break_that_python_reads() {
        let raised = Kind::Raised {
            exception: "ValueError: a\x0bb\x0cc\u{2028}d\u{85}e".to_owned(),
            whence: None,
        };
        let (text, _) = report_of(&[
            ("test_a.py::test_one", Ok(&raised)),
            ("test_a.py::test_two", Err("a\x1cb\x1dc\x1ed\u{2029}e")),
        ]);
        assert_eq!(
            text.lines().take(2).collect::<Vec<_>>(),
            [
                "test_a.py::test_one raised: ValueError: a b c d e",
                "test_a.py::test_two project error: a b c d e",
            ],
            "each character that str.splitlines reads as a line break joins as one space"
        );
    }

    #[test]
    fn a_reason_escapes_each_other_control_character_as_python_writes_it() {
        let raised = Kind::Raised {
            exception: "ValueError: \x1b[32mgreen\x1b[0m\x1b[1A\x1b[2K".to_owned(),
            whence: None,
        };
        let failed = Kind::Failed {
            asserts: vec![quote("assert x ==\t\"a\x00b\"", None)],
        };
        let (text, _) = report_of(&[
            ("test_a.py::test_one", Ok(&raised)),
            ("test_a.py::test_two", Ok(&failed)),
            (
                "test_a.py::test_three",
                Err("bs\x08x bel\x07 del\x7f csi\u{9b}1A"),
            ),
        ]);
        assert_eq!(
            text.lines().take(4).collect::<Vec<_>>(),
            [
                r"test_a.py::test_one raised: ValueError: \x1b[32mgreen\x1b[0m\x1b[1A\x1b[2K",
                "test_a.py::test_two failed",
                r#"    assert x ==\t"a\x00b""#,
                r"test_a.py::test_three project error: bs\x08x bel\x07 del\x7f csi\x9b1A",
            ],
            "a tab prints as \\t, and each other control character as \\x and two hex digits"
        );
    }

    #[test]
    fn each_failed_expectation_prints_its_expected_value_and_its_reading_under_it() {
        let failed = Kind::Failed {
            asserts: vec![
                quote("assert got == []", Some(("[]", "[\n  -1,\n]"))),
                quote("assert len(got) == 0", Some(("0", "a\x00b"))),
                quote("the cleanup of the with at line 4 raised OSError: x", None),
            ],
        };
        let (text, code) = report_of(&[("test_a.py::test_one", Ok(&failed))]);
        assert_eq!(
            text,
            "test_a.py::test_one failed\n\
             \x20   assert got == []\n\
             \x20     expected  []\n\
             \x20     reading   [ -1, ]\n\
             \x20   assert len(got) == 0\n\
             \x20     expected  0\n\
             \x20     reading   a\\x00b\n\
             \x20   the cleanup of the with at line 4 raised OSError: x\n\
             1 case: 1 failed\n",
            "two lines under each quote that shows a reading, each on one line, and none \
             under a quote that shows none"
        );
        assert_eq!(code, 1, "a failed test makes the run exit 1");
    }

    #[test]
    fn an_unreached_test_prints_no_line_and_the_count_names_it_after_blocked() {
        let raised = Kind::Raised {
            exception: "KeyboardInterrupt: ".to_owned(),
            whence: None,
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
             4 cases: 1 passed, 1 raised, 1 blocked, 1 unreached; 1 project error\n",
            "the unreached test has no line, and the count names it after blocked"
        );
    }

    #[test]
    fn an_interrupted_case_prints_its_line_and_the_count_names_it() {
        let mut out = Vec::new();
        let mut report = Report::new(&mut out);
        for (test, kind) in [
            ("test_a.py::test_one", Kind::Passed),
            ("test_a.py::test_two", Kind::Passed),
            ("test_a.py::test_three", Kind::Interrupted),
        ] {
            report
                .add(&test, Ok(&kind))
                .expect("a Vec takes every write");
        }
        report.unreached();
        report.unreached();
        report.finish().expect("a Vec takes every write");
        assert_eq!(
            String::from_utf8(out).expect("reporting writes UTF-8"),
            "test_a.py::test_one passed\n\
             test_a.py::test_two passed\n\
             test_a.py::test_three interrupted\n\
             5 cases: 2 passed, 1 interrupted, 2 unreached\n",
            "the interrupted test prints its line, and the count names it before unreached"
        );
        assert_eq!(
            count_line(&Tally {
                interrupted: 1,
                blocked: 1,
                ..Tally::default()
            }),
            "2 cases: 1 interrupted, 1 blocked",
            "the count names interrupted before blocked"
        );
    }

    #[test]
    fn the_count_names_each_column_only_above_0() {
        assert_eq!(
            count_line(&Tally::default()),
            "0 cases",
            "no column is above 0"
        );
        assert_eq!(
            count_line(&Tally {
                passed: 3,
                ..Tally::default()
            }),
            "3 cases: 3 passed",
            "one column"
        );
        assert_eq!(
            count_line(&Tally {
                passed: 1,
                raised: 1,
                ..Tally::default()
            }),
            "2 cases: 1 passed, 1 raised",
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
            "0 cases; 1 project error",
            "an error, and no case"
        );
        assert_eq!(
            count_line(&Tally {
                passed: 1,
                blocked: 2,
                project_errors: 1,
                internal_errors: 1,
                ..Tally::default()
            }),
            "3 cases: 1 passed, 2 blocked; 1 project error, 1 internal error",
            "both kinds of error, after the tests"
        );
        assert_eq!(
            count_line(&Tally {
                blocked: 2,
                project_errors: 2,
                ..Tally::default()
            }),
            "2 cases: 2 blocked; 2 project errors",
            "a kind of error takes an s above 1"
        );
    }

    #[test]
    fn the_count_names_the_suppressed_verdicts_after_the_errors() {
        assert_eq!(
            count_line(&Tally {
                passed: 1,
                suppressed: 1,
                ..Tally::default()
            }),
            "1 case: 1 passed; 1 suppressed",
            "no error"
        );
        assert_eq!(
            count_line(&Tally {
                passed: 3,
                failed: 1,
                blocked: 1,
                project_errors: 1,
                suppressed: 5,
                ..Tally::default()
            }),
            "5 cases: 3 passed, 1 failed, 1 blocked; 1 project error; 5 suppressed",
            "after the errors, and no s above 1"
        );
    }

    #[test]
    fn each_suppressed_verdict_prints_under_its_case_and_changes_no_exit_code() {
        let mut out = Vec::new();
        let mut report = Report::new(&mut out);
        report
            .add(&"t.py::test_round_trip", Ok(&Kind::Passed))
            .expect("a Vec takes every write");
        report
            .warned(
                "computed-expected",
                "line 10 computes obj, and an expected value is a literal",
                "the payload is its own expected value",
            )
            .expect("a Vec takes every write");
        report
            .warned(
                "assert-before-expectation",
                "line 3 asserts before the first expectation,\nand only an expectation asserts",
                "the act checks its input",
            )
            .expect("a Vec takes every write");
        let code = report.finish().expect("a Vec takes every write");

        assert_eq!(
            String::from_utf8(out).expect("reporting writes UTF-8"),
            "t.py::test_round_trip passed\n    \u{26a0} suppressed [computed-expected] \
             line 10 computes obj, and an expected value is a literal\n      \
             reason: the payload is its own expected value\n    \
             \u{26a0} suppressed [assert-before-expectation] line 3 asserts before the first \
             expectation, and only an expectation asserts\n      reason: the act checks its input\n\
             1 case: 1 passed; 2 suppressed\n",
            "each verdict under its case, on one line, with its reason on the next"
        );
        assert_eq!(code, 0, "a suppressed verdict changes no exit code");
    }

    #[test]
    fn each_notice_prints_under_its_case_and_changes_no_count_or_exit_code() {
        let mut out = Vec::new();
        let mut report = Report::new(&mut out);
        report
            .add(&"t.py::test_private", Ok(&Kind::Passed))
            .expect("a Vec takes every write");
        report
            .noticed("line 12 reads the private name\n_default_encoder")
            .expect("a Vec takes every write");
        let code = report.finish().expect("a Vec takes every write");

        assert_eq!(
            String::from_utf8(out).expect("reporting writes UTF-8"),
            "t.py::test_private passed\n    \u{26a0} line 12 reads the private name \
             _default_encoder\n1 case: 1 passed\n",
            "the notice under its case, on one line, and no count of it"
        );
        assert_eq!(code, 0, "a notice changes no exit code");
    }
}
