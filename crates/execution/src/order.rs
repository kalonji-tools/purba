//! The order that decides the outcome of a case from the facts that its run
//! gathered. It reads no Python, so the process of the test gathers the facts,
//! and purba decides the outcome once it has read each expectation alone.

use purba_assertion::{Shown, Verdict};

use crate::{Error, Kind, Suppressed};

/// What the call of a test gave.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub enum Called {
    /// The call returned.
    #[default]
    Returned,
    /// The call raised. `exception` is the type and the message of the
    /// exception, `whence` names the frame that it came from when that is not
    /// the test's own, and `assertion` says whether it is an `AssertionError`.
    Raised {
        exception: String,
        whence: Option<String>,
        assertion: bool,
    },
}

/// Each fact of a case that the order reads, with no Python in it.
///
/// It holds the first fault of each check of the form, each beside the name of
/// its check, and what the call gave. It holds the text of a `with` whose
/// entry or cleanup raised what the call raised, and the last marked assert
/// beside whether it held. It holds the faults of the paths, each beside the
/// name of its check, and whether one lies in a `try` of another shape than
/// one path. It holds why no reader was forked when a fork failed. It holds
/// the verdict of each expectation read in order and of each read alone, what
/// each that did not hold shows, and each later expectation that a replay must
/// read alone. It holds the text of each expectation that it names, the words
/// of each missed `try`, and whether the test states an expectation.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Facts {
    pub form: Vec<(String, String)>,
    pub called: Called,
    pub with: Option<String>,
    pub mark: Option<(String, bool)>,
    pub path: Vec<(String, String)>,
    pub in_another_try: bool,
    pub refused: Option<String>,
    pub in_order: Vec<(usize, bool)>,
    pub shown: Vec<(usize, Shown)>,
    pub alone: Vec<(usize, Verdict)>,
    pub later: Vec<usize>,
    pub texts: Vec<(usize, String)>,
    pub missed: Vec<String>,
    pub stated: bool,
}

/// The outcome of a case, from its `facts`, and each warning that
/// `suppressions` gave. The first of these wins:
///
/// 1. a project error for the first fault of each check of the form in turn,
///    then for a reading alone that an assert ran inside;
/// 2. `raised` when the call raised, unless a `with`'s entry or cleanup raised
///    it, or a marked assert that did not hold raised `AssertionError`, which
///    are `failed`;
/// 3. a project error for the faults of the paths, joined by `; `;
/// 4. a project error when a fork to read a later expectation alone failed, or
///    when the two readings of one expectation differ;
/// 5. `failed` when a `try` of one path missed, or an expectation did not
///    hold;
/// 6. a project error when no expectation ran;
/// 7. `passed`.
///
/// A fault of a check that a suppression reaches gives a warning in place of
/// its project error, and the case goes on to the next of these. Each project
/// error of a strict check names its check in brackets before the words of
/// each fault. The order judges each suppression of a check that execution
/// sees only when it reaches step 3.
pub fn order(facts: &Facts, suppressions: &[(&str, &str)]) -> (Result<Kind, Error>, Suppressed) {
    todo!("the body of `decide` moves here: {facts:?}, {suppressions:?}")
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The line that `order` gives for `facts` with no suppression, as text a
    /// row can compare.
    fn line(facts: &Facts) -> String {
        shown(order(facts, &[]).0)
    }

    /// What `warned` gives: the line, each warning as `[<check>] <words>`, and
    /// whether the order judged the suppressions.
    type Warned = (String, Vec<String>, bool);

    /// A row of a test of suppressions: what it checks, the facts of a case,
    /// its suppressions, and what `warned` gives.
    type Row<'a> = (&'a str, Facts, &'a [(&'a str, &'a str)], Warned);

    /// What `order` gives for `facts` under `suppressions`.
    fn warned(facts: &Facts, suppressions: &[(&str, &str)]) -> Warned {
        let (own, suppressed) = order(facts, suppressions);
        let warnings = suppressed
            .warnings
            .iter()
            .map(|warning| format!("[{}] {}", warning.check, warning.words))
            .collect();
        (shown(own), warnings, suppressed.judged)
    }

    /// An outcome or an error as the text of its line.
    fn shown(own: Result<Kind, Error>) -> String {
        match own {
            Ok(Kind::Passed) => "passed".to_owned(),
            Ok(Kind::Failed { asserts }) => format!(
                "failed: {}",
                asserts
                    .iter()
                    .map(|quote| quote.assert.as_str())
                    .collect::<Vec<_>>()
                    .join(", ")
            ),
            Ok(Kind::Raised { exception, .. }) => format!("raised: {exception}"),
            Ok(Kind::Interrupted) => "interrupted".to_owned(),
            Err(error) => error.line(),
        }
    }

    /// The facts of a case whose call raised `exception`.
    fn raised(exception: &str) -> Called {
        Called::Raised {
            exception: exception.to_owned(),
            whence: None,
            assertion: false,
        }
    }

    /// The text of the expectations at index 1 and 2.
    fn texts() -> Vec<(usize, String)> {
        vec![
            (1, "assert x == 1".to_owned()),
            (2, "assert check() is True".to_owned()),
        ]
    }

    #[test]
    fn each_step_of_the_order_beats_each_step_below_it() {
        // Each row: what it checks, the facts of a case, and the line that
        // `order` gives.
        let rows: Vec<(&str, Facts, &str)> = vec![
            // fails when: the check of `called` moves above the check of `alone`
            (
                "a reading alone that reached an assert beats a raise",
                Facts {
                    called: raised("ValueError: v"),
                    in_order: vec![(1, true)],
                    alone: vec![(2, Verdict::Refused)],
                    texts: texts(),
                    stated: true,
                    ..Facts::default()
                },
                "project error: [assert-in-reading] assert check() is True \
                 reaches an assert when read alone, and only an expectation asserts",
            ),
            // fails when: `of_the_readings` gives `passed` when no expectation
            // ran, in place of the project error
            (
                "a test that states an expectation and runs none is a project error",
                Facts {
                    stated: true,
                    ..Facts::default()
                },
                "project error: [no-expectation] no expectation ran",
            ),
            // EXERCISE: one row for each sentence of the doc of `order`.
        ];
        for (row, facts, gives) in rows {
            assert_eq!(line(&facts), gives, "{row}");
        }
    }

    #[test]
    fn a_suppressed_fault_gives_a_warning_and_the_case_goes_on() {
        let swallowed = "the with at line 3 swallowed a raise, and line 4 did not run";
        let rows: Vec<Row<'_>> = vec![
            // fails when: `clause` gives back the clause of a check that a
            // suppression reaches, in place of the warning
            (
                "a suppressed fault of the paths gives a warning, and the case passes",
                Facts {
                    path: vec![("swallowed-raise".to_owned(), swallowed.to_owned())],
                    in_order: vec![(1, true)],
                    texts: texts(),
                    stated: true,
                    ..Facts::default()
                },
                &[("swallowed-raise", "the act once swallowed a raise")],
                (
                    "passed".to_owned(),
                    vec![format!("[swallowed-raise] {swallowed}")],
                    true,
                ),
            ),
            // EXERCISE: a row for the last sentence of the doc of `order`.
        ];
        for (row, facts, suppressions, gives) in rows {
            assert_eq!(warned(&facts, suppressions), gives, "{row}");
        }
    }
}
