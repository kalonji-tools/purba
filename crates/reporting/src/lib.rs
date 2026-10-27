//! Reporting: prints one line for each test and a count, and renders the exit
//! code.

use std::fmt::Display;
use std::io::{self, BufWriter, Write};

use purba_execution::{Error, Kind, Outcome, Tally};

/// Prints each test beside its outcome or error, in the order given, then the
/// count, to `out`. Returns the exit code of the run.
///
/// # Errors
///
/// When `out` refuses a write.
pub fn print<T: Display>(
    out: &mut impl Write,
    results: &[(T, Result<Outcome, Error>)],
) -> io::Result<u8> {
    let results = results
        .iter()
        .map(|(test, result)| (test, result.as_ref().map(Outcome::kind)));
    report(&mut BufWriter::new(out), results)
}

/// A test passes a text for an error, because no crate but execution builds an
/// `Error`.
fn report<'a, T: Display + 'a, E: Display>(
    out: &mut impl Write,
    results: impl IntoIterator<Item = (&'a T, Result<&'a Kind, E>)>,
) -> io::Result<u8> {
    let mut tally = Tally::default();
    for (test, result) in results {
        tally.add(&result);
        write_line(out, test, result)?;
    }
    writeln!(out, "{}", count_line(&tally))?;
    out.flush()?;
    Ok(exit_code(&tally))
}

fn write_line<T: Display, E: Display>(
    out: &mut impl Write,
    test: &T,
    result: Result<&Kind, E>,
) -> io::Result<()> {
    match result {
        Ok(Kind::Passed) => writeln!(out, "{test} passed"),
        Ok(Kind::Failed { assert }) => writeln!(out, "{test} failed: {}", one_line(assert)),
        Ok(Kind::Raised { exception }) => {
            writeln!(out, "{test} raised: {}", one_line(exception))
        }
        Ok(Kind::Unknown) => writeln!(out, "{test} unknown: no assert ran"),
        Err(error) => writeln!(out, "{test} error: {}", one_line(&error.to_string())),
    }
}

/// PROTOTYPE #664: prints each line as dispatch hands it, and keeps only the
/// tally until the count. After a refused write it counts and writes nothing.
pub struct Report<W: Write> {
    out: W,
    tally: Tally,
    flush_each: bool,
    muted: bool,
}

impl<W: Write> Report<W> {
    /// `flush_each` flushes after each line. Off, the lines wait in `out`.
    pub fn new(out: W, flush_each: bool) -> Self {
        Self {
            out,
            tally: Tally::default(),
            flush_each,
            muted: false,
        }
    }

    /// Counts the test, and writes its line unless an earlier write failed.
    ///
    /// # Errors
    ///
    /// When `out` refuses the write. Every later line is then only counted.
    pub fn line<T: Display>(
        &mut self,
        test: &T,
        result: &Result<Outcome, Error>,
    ) -> io::Result<()> {
        let result = result.as_ref().map(Outcome::kind);
        self.tally.add(&result);
        if self.muted {
            return Ok(());
        }
        let written = write_line(&mut self.out, test, result).and_then(|()| {
            if self.flush_each {
                self.out.flush()
            } else {
                Ok(())
            }
        });
        self.muted = written.is_err();
        written
    }

    /// Writes the count, unless a write failed, and returns the exit code.
    ///
    /// # Errors
    ///
    /// When `out` refuses the count or its flush.
    pub fn finish(mut self) -> io::Result<u8> {
        if !self.muted {
            writeln!(self.out, "{}", count_line(&self.tally))?;
            self.out.flush()?;
        }
        Ok(exit_code(&self.tally))
    }
}

fn count_line(tally: &Tally) -> String {
    let tests = tally.passed + tally.failed + tally.raised + tally.unknown + tally.errors;
    format!(
        "{tests} test{}: {} passed, {} failed, {} raised, {} unknown, {} error{}",
        plural(tests),
        tally.passed,
        tally.failed,
        tally.raised,
        tally.unknown,
        tally.errors,
        plural(tally.errors),
    )
}

const fn exit_code(tally: &Tally) -> u8 {
    if tally.failed > 0 {
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
    use super::*;

    fn report_of<'a>(results: &[(&'a str, Result<&'a Kind, &'a str>)]) -> (String, u8) {
        let mut out = Vec::new();
        let code = report(
            &mut out,
            results.iter().map(|(test, result)| (test, *result)),
        )
        .expect("a Vec takes every write");
        (
            String::from_utf8(out).expect("reporting writes UTF-8"),
            code,
        )
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
             5 tests: 1 passed, 1 failed, 1 raised, 1 unknown, 1 error\n",
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
    fn the_count_names_all_five_and_one_in_the_singular() {
        assert_eq!(
            report_of(&[("test_a.py::test_one", Ok(&Kind::Passed))]).0,
            "test_a.py::test_one passed\n\
             1 test: 1 passed, 0 failed, 0 raised, 0 unknown, 0 errors\n",
            "one test, and no error"
        );
        assert_eq!(
            report_of(&[]),
            (
                "0 tests: 0 passed, 0 failed, 0 raised, 0 unknown, 0 errors\n".to_owned(),
                2
            ),
            "a run with no test exits 2"
        );
    }
}
