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
    #[cfg(feature = "fault")]
    assert!(std::env::var("PURBA_FAULT").as_deref() != Ok("report-panics"), "PROTOTYPE fault: reporting panicked");
    for (test, result) in results {
        tally.add(&result);
        match result {
            Ok(Kind::Passed) => writeln!(out, "{test} passed")?,
            Ok(Kind::Failed { assert }) => {
                writeln!(out, "{test} failed: {}", one_line(assert))?;
            }
            Ok(Kind::Unknown) => writeln!(out, "{test} unknown: no assert ran")?,
            Err(error) => {
                writeln!(out, "{test} error: {}", one_line(&error.to_string()))?;
            }
        }
    }
    writeln!(out, "{}", count_line(&tally))?;
    out.flush()?;
    Ok(exit_code(&tally))
}

fn count_line(tally: &Tally) -> String {
    let tests = tally.passed + tally.failed + tally.unknown + tally.errors;
    format!(
        "{tests} test{}: {} passed, {} failed, {} unknown, {} error{}",
        plural(tests),
        tally.passed,
        tally.failed,
        tally.unknown,
        tally.errors,
        plural(tally.errors),
    )
}

const fn exit_code(tally: &Tally) -> u8 {
    if tally.failed > 0 {
        1
    } else if tally.errors > 0 {
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
        let (text, code) = report_of(&[
            ("test_a.py::test_one", Ok(&Kind::Passed)),
            ("test_a.py::test_two", Ok(&failed)),
            ("test_a.py::test_three", Ok(&Kind::Unknown)),
            ("test_c.py::test_x", Err("ValueError: boom")),
        ]);
        assert_eq!(
            text,
            "test_a.py::test_one passed\n\
             test_a.py::test_two failed: assert 1 == 2\n\
             test_a.py::test_three unknown: no assert ran\n\
             test_c.py::test_x error: ValueError: boom\n\
             4 tests: 1 passed, 1 failed, 1 unknown, 1 error\n",
            "one line for each test, in order, then the count"
        );
        assert_eq!(code, 1, "a failed test makes the run exit 1");
    }

    #[test]
    fn the_exit_code_names_the_worst_of_the_run() {
        let tally = |passed, failed, unknown, errors| Tally {
            passed,
            failed,
            unknown,
            errors,
        };
        assert_eq!(
            exit_code(&tally(1, 1, 1, 1)),
            1,
            "a failed test comes first"
        );
        assert_eq!(exit_code(&tally(1, 0, 1, 1)), 4, "then an error");
        assert_eq!(exit_code(&tally(1, 0, 1, 0)), 2, "then an unknown test");
        assert_eq!(exit_code(&tally(0, 0, 0, 0)), 2, "then a run with no test");
        assert_eq!(exit_code(&tally(2, 0, 0, 0)), 0, "every test passed");
    }

    #[test]
    fn a_reason_over_several_lines_prints_on_one() {
        let failed = Kind::Failed {
            assert: "assert (\n    total  ==  3\n)".to_owned(),
        };
        let (text, _) = report_of(&[
            ("test_a.py::test_one", Ok(&failed)),
            ("test_a.py::test_two", Err("ValueError: a \r\n\n  b")),
            ("test_a.py::test_three", Err("ValueError: a\rb")),
            ("test_a.py::test_four", Err("ValueError: boom\n")),
        ]);
        assert_eq!(
            text.lines().take(4).collect::<Vec<_>>(),
            [
                "test_a.py::test_one failed: assert ( total  ==  3 )",
                "test_a.py::test_two error: ValueError: a b",
                "test_a.py::test_three error: ValueError: a b",
                "test_a.py::test_four error: ValueError: boom",
            ],
            "a lone \\r breaks a line too, and the ends of a reason lose their whitespace"
        );
    }

    #[test]
    fn the_count_names_all_four_and_one_in_the_singular() {
        assert_eq!(
            report_of(&[("test_a.py::test_one", Ok(&Kind::Passed))]).0,
            "test_a.py::test_one passed\n1 test: 1 passed, 0 failed, 0 unknown, 0 errors\n",
            "one test, and no error"
        );
        assert_eq!(
            report_of(&[]),
            (
                "0 tests: 0 passed, 0 failed, 0 unknown, 0 errors\n".to_owned(),
                2
            ),
            "a run with no test exits 2"
        );
    }
}
