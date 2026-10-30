//! Reporting: prints one line for each test, and for each module that did not
//! load, as it takes them, then a count, and renders the exit code.

use std::fmt::Display;
use std::io::{self, Write};

use purba_execution::{Error, Kind, Outcome, Tally};

/// Prints each test beside its outcome or error to `out`, and each module that
/// did not load, as it takes them, then the count. Keeps only the tally.
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

    /// Adds `test` to the tally, and writes its line and flushes it.
    ///
    /// # Errors
    ///
    /// When `out` refuses the write or the flush.
    pub fn line<T: Display>(
        &mut self,
        test: &T,
        result: &Result<Outcome, Error>,
    ) -> io::Result<()> {
        // PROTOTYPE (#715): an internal error gets a line and a column of its own.
        if let Err(error @ Error::Internal { .. }) = result {
            self.tally.add_blocked();
            self.tally.add_internal();
            writeln!(
                self.out,
                "{test} internal error: {}",
                one_line(&error.to_string())
            )?;
            return self.out.flush();
        }
        self.add(test, result.as_ref().map(Outcome::kind))
    }

    /// PROTOTYPE (#715): a module that did not load, by its execution error.
    ///
    /// # Errors
    ///
    /// When `out` refuses the write or the flush.
    pub fn not_loaded_by<M: Display>(&mut self, module: &M, error: &Error) -> io::Result<()> {
        if matches!(error, Error::Internal { .. }) {
            self.tally.add_internal();
            writeln!(
                self.out,
                "{module} internal error: {}",
                one_line(&error.to_string())
            )?;
            return self.out.flush();
        }
        self.not_loaded(module, error)
    }

    /// Adds `test` to the tally as blocked by its own error, which a part
    /// before execution found, and writes its line and flushes it.
    ///
    /// # Errors
    ///
    /// When `out` refuses the write or the flush.
    pub fn error<T: Display, E: Display>(&mut self, test: &T, error: &E) -> io::Result<()> {
        self.add(test, Err::<&Kind, _>(error))
    }

    /// Adds the error of a module that did not load to the tally, and writes
    /// its line and flushes it.
    ///
    /// # Errors
    ///
    /// When `out` refuses the write or the flush.
    pub fn not_loaded<M: Display, E: Display>(&mut self, module: &M, error: &E) -> io::Result<()> {
        self.tally.add_error();
        writeln!(
            self.out,
            "{module} project error: {}",
            one_line(&error.to_string())
        )?;
        self.out.flush()
    }

    /// Adds `test` to the tally as blocked, because `module` did not load, and
    /// writes its line and flushes it.
    ///
    /// # Errors
    ///
    /// When `out` refuses the write or the flush.
    pub fn blocked<T: Display, M: Display>(&mut self, test: &T, module: &M) -> io::Result<()> {
        self.tally.add_blocked();
        writeln!(self.out, "{test} blocked: {module} did not load")?;
        self.out.flush()
    }

    /// A test passes a text for an error, because no crate but execution
    /// builds an `Error`.
    fn add<T: Display, E: Display>(
        &mut self,
        test: &T,
        result: Result<&Kind, E>,
    ) -> io::Result<()> {
        self.tally.add(&result);
        let out = &mut self.out;
        match result {
            Ok(Kind::Passed) => writeln!(out, "{test} passed")?,
            Ok(Kind::Failed { assert }) => {
                writeln!(out, "{test} failed: {}", one_line(assert))?;
            }
            Ok(Kind::Raised { exception }) => {
                writeln!(out, "{test} raised: {}", one_line(exception))?;
            }
            Ok(Kind::Unknown) => writeln!(out, "{test} unknown: no assert ran")?,
            Err(error) => {
                writeln!(
                    out,
                    "{test} project error: {}",
                    one_line(&error.to_string())
                )?;
            }
        }
        out.flush()
    }

    /// Writes the count and flushes it. Returns the exit code of the run.
    ///
    /// # Errors
    ///
    /// When `out` refuses the write or the flush.
    pub fn finish(mut self) -> io::Result<u8> {
        writeln!(self.out, "{}", count_line(&self.tally))?;
        self.out.flush()?;
        Ok(exit_code(&self.tally))
    }
}

fn count_line(tally: &Tally) -> String {
    let tests = tally.passed + tally.failed + tally.raised + tally.unknown + tally.blocked;
    format!(
        "{tests} test{}: {} passed, {} failed, {} raised, {} unknown, {} blocked; {} project error{}, {} internal error{}",
        plural(tests),
        tally.passed,
        tally.failed,
        tally.raised,
        tally.unknown,
        tally.blocked,
        tally.errors,
        plural(tally.errors),
        tally.internal,
        plural(tally.internal),
    )
}

const fn exit_code(tally: &Tally) -> u8 {
    if tally.internal > 0 {
        125
    } else if tally.failed > 0 {
        1
    } else if tally.raised > 0 || tally.errors > 0 {
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

    fn report_of<'a>(results: &[(&'a str, Result<&'a Kind, &'a str>)]) -> (String, u8) {
        let mut out = Vec::new();
        let mut report = Report::new(&mut out);
        for (test, result) in results {
            report.add(test, *result).expect("a Vec takes every write");
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
            .add(&"test_a.py::test_one", Ok::<_, &str>(&Kind::Passed))
            .expect("the writer takes every write");
        assert_eq!(
            *seen.borrow(),
            "test_a.py::test_one passed\n",
            "the first line is flushed before the second test"
        );
        report
            .add(&"test_a.py::test_two", Ok::<_, &str>(&Kind::Unknown))
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
             2 tests: 1 passed, 0 failed, 0 raised, 1 unknown, 0 blocked; 0 errors\n",
            "the count is flushed when the run finishes"
        );
        assert_eq!(code, 2, "an unknown test exits 2");
    }

    #[test]
    fn a_refused_write_is_the_error_of_its_line() {
        let mut report = Report::new(RefusesSecondLine::default());
        report
            .add(&"test_a.py::test_one", Ok::<_, &str>(&Kind::Passed))
            .expect("the writer takes the first line");
        let error = report
            .add(&"test_a.py::test_two", Ok::<_, &str>(&Kind::Passed))
            .expect_err("the writer refuses the second line");
        assert_eq!(error.to_string(), "refused", "the writer's own error");
    }

    #[test]
    fn an_error_that_another_part_finds_prints_as_an_error() {
        let mut out = Vec::new();
        let mut report = Report::new(&mut out);
        report
            .error(
                &"test_a.py::test_one",
                &"the parameter x has no default, and purba cannot see which fixture builds it",
            )
            .expect("a Vec takes every write");
        let code = report.finish().expect("a Vec takes every write");
        assert_eq!(
            String::from_utf8(out).expect("reporting writes UTF-8"),
            "test_a.py::test_one error: the parameter x has no default, \
             and purba cannot see which fixture builds it\n\
             1 test: 0 passed, 0 failed, 0 raised, 0 unknown, 1 blocked; 1 error\n",
            "the line of a test whose own error blocks it, and the count of both"
        );
        assert_eq!(code, 4, "an error makes the run exit 4");
    }

    #[test]
    fn a_module_that_did_not_load_prints_once_and_each_test_as_blocked() {
        let mut out = Vec::new();
        let mut report = Report::new(&mut out);
        report
            .not_loaded(&"test_b.py", &"ValueError: x")
            .expect("a Vec takes every write");
        for test in ["test_b.py::test_one", "test_b.py::test_two"] {
            report
                .blocked(&test, &"test_b.py")
                .expect("a Vec takes every write");
        }
        let code = report.finish().expect("a Vec takes every write");
        assert_eq!(
            String::from_utf8(out).expect("reporting writes UTF-8"),
            "test_b.py error: ValueError: x\n\
             test_b.py::test_one blocked: test_b.py did not load\n\
             test_b.py::test_two blocked: test_b.py did not load\n\
             2 tests: 0 passed, 0 failed, 0 raised, 0 unknown, 2 blocked; 1 error\n",
            "the error once, then each test that it blocked"
        );
        assert_eq!(
            code, 4,
            "the error sets the code, and a blocked test changes nothing"
        );
    }

    #[test]
    fn a_test_whose_own_error_blocks_it_prints_error_and_counts_as_blocked() {
        let mut out = Vec::new();
        let mut report = Report::new(&mut out);
        report
            .error(
                &"test_a.py::test_async",
                &"purba cannot run an async test yet",
            )
            .expect("a Vec takes every write");
        report
            .add(
                &"test_a.py::test_five",
                Err::<&Kind, _>("TypeError: 'int' object is not callable"),
            )
            .expect("a Vec takes every write");
        let code = report.finish().expect("a Vec takes every write");
        assert_eq!(
            String::from_utf8(out).expect("reporting writes UTF-8"),
            "test_a.py::test_async error: purba cannot run an async test yet\n\
             test_a.py::test_five error: TypeError: 'int' object is not callable\n\
             2 tests: 0 passed, 0 failed, 0 raised, 0 unknown, 2 blocked; 2 errors\n",
            "each line keeps error, and each test counts as blocked and as an error"
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
             test_c.py::test_x error: ValueError: boom\n\
             5 tests: 1 passed, 1 failed, 1 raised, 1 unknown, 1 blocked; 1 error\n",
            "one line for each test, in order, then the count"
        );
        assert_eq!(code, 1, "a failed test makes the run exit 1");
    }

    #[test]
    fn the_exit_code_names_the_worst_of_the_run() {
        let tally = |passed, failed, raised, unknown, errors| Tally {
            passed,
            failed,
            raised,
            unknown,
            blocked: 0,
            errors,
        };
        assert_eq!(
            exit_code(&tally(1, 1, 1, 1, 1)),
            1,
            "a failed test comes first"
        );
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
                "test_a.py::test_two error: ValueError: a b",
                "test_a.py::test_three error: ValueError: a b",
                "test_a.py::test_four error: ValueError: boom",
                "test_a.py::test_five raised: ValueError: a b",
            ],
            "a lone \\r breaks a line too, and the ends of a reason lose their whitespace"
        );
    }

    #[test]
    fn the_count_names_each_column_and_one_in_the_singular() {
        assert_eq!(
            report_of(&[("test_a.py::test_one", Ok(&Kind::Passed))]).0,
            "test_a.py::test_one passed\n\
             1 test: 1 passed, 0 failed, 0 raised, 0 unknown, 0 blocked; 0 errors\n",
            "one test, and no error"
        );
        assert_eq!(
            report_of(&[]),
            (
                "0 tests: 0 passed, 0 failed, 0 raised, 0 unknown, 0 blocked; 0 errors\n"
                    .to_owned(),
                2
            ),
            "a run with no test exits 2"
        );
    }
}
