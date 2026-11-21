//! The order that decides the outcome of a case from the facts that its run
//! gathered. It reads no Python, so the process of the test gathers the facts,
//! and purba decides the outcome once it has read each expectation alone.

use purba_assertion::{Shown, Verdict};

use crate::{Error, Kind, Quote, SUPPRESSIBLE, Suppressed, Warning};

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

impl Facts {
    /// The text of the expectation at `index`, when the facts hold it.
    fn text(&self, index: usize) -> Option<&str> {
        self.texts
            .iter()
            .find(|(held, _)| *held == index)
            .map(|(_, text)| text.as_str())
    }
}

/// The suppressions of a case, and the warnings that they gave.
struct Suppressing<'a> {
    suppressions: &'a [(&'a str, &'a str)],
    warnings: Vec<Warning>,
    judged: bool,
}

impl Suppressing<'_> {
    /// Gives `words`, a fault of `check`, back as a clause of a project error,
    /// or turns it into a warning when a suppression reaches `check`.
    fn clause(&mut self, check: &str, words: &str) -> Option<String> {
        let Some((_, reason)) = self
            .suppressions
            .iter()
            .find(|(named, _)| *named == check && SUPPRESSIBLE.contains(named))
        else {
            return Some(format!("[{check}] {words}"));
        };
        match self
            .warnings
            .iter_mut()
            .find(|warning| warning.check == check)
        {
            Some(warning) => {
                warning.words.push_str("; ");
                warning.words.push_str(words);
            }
            None => self.warnings.push(Warning {
                check: check.to_owned(),
                words: words.to_owned(),
                reason: (*reason).to_owned(),
            }),
        }
        None
    }
}

/// The shape that a fault of a line of a `try` of another shape names.
const ONE_PATH: &str =
    "a try of one path holds one statement and one except SomeError:, with no else and no finally";

/// The words of a reading alone that an assert ran inside.
const REFUSED_ALONE: &str = "reaches an assert when read alone, and only an expectation asserts";

/// The words of two readings of one expectation that differ, or `None` when
/// they agree.
const fn differs(alone: Verdict, in_order: bool) -> Option<&'static str> {
    Some(match (alone, in_order) {
        (Verdict::Holds, false) => {
            "holds when read alone, and not after the expectations before it"
        }
        (Verdict::Fails, true) => "holds after the expectations before it, and not when read alone",
        (Verdict::Raises, true) => {
            "holds after the expectations before it, and raises when read alone"
        }
        (Verdict::Raises, false) => {
            "does not hold after the expectations before it, and raises when read alone"
        }
        (Verdict::Holds, true) | (Verdict::Fails, false) | (Verdict::Refused, _) => return None,
    })
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
    let mut suppressing = Suppressing {
        suppressions,
        warnings: Vec::new(),
        judged: false,
    };
    let own = steps(facts, &mut suppressing);
    (
        own,
        Suppressed {
            warnings: suppressing.warnings,
            judged: suppressing.judged,
        },
    )
}

/// The steps of [`order`], in turn.
fn steps(facts: &Facts, suppressing: &mut Suppressing<'_>) -> Result<Kind, Error> {
    for (check, words) in &facts.form {
        if let Some(clause) = suppressing.clause(check, words) {
            return Err(Error::project(clause));
        }
    }
    if let Some((index, _)) = facts
        .alone
        .iter()
        .find(|(_, verdict)| *verdict == Verdict::Refused)
    {
        let text = facts.text(*index).unwrap_or_default();
        return Err(Error::project(format!(
            "[assert-in-reading] {text} {REFUSED_ALONE}"
        )));
    }
    if let Called::Raised {
        exception,
        whence,
        assertion,
    } = &facts.called
    {
        if let Some(with) = &facts.with {
            return Ok(Kind::Failed {
                asserts: vec![Quote::alone(with.clone())],
            });
        }
        if let Some((assert, false)) = &facts.mark
            && *assertion
        {
            return Ok(Kind::Failed {
                asserts: vec![Quote::alone(assert.clone())],
            });
        }
        return Ok(Kind::Raised {
            exception: exception.clone(),
            whence: whence.clone(),
        });
    }
    suppressing.judged = true;
    let path: Vec<String> = facts
        .path
        .iter()
        .filter_map(|(check, words)| suppressing.clause(check, words))
        .collect();
    if !path.is_empty() {
        let mut line = path.join("; ");
        if facts.in_another_try {
            line.push_str("; ");
            line.push_str(ONE_PATH);
        }
        return Err(Error::project(line));
    }
    if let Some(refused) = &facts.refused {
        return Err(Error::project(refused.clone()));
    }
    for &(index, alone) in &facts.alone {
        let Some(&(_, in_order)) = facts.in_order.iter().find(|(read, _)| *read == index) else {
            continue;
        };
        if let Some(differ) = differs(alone, in_order) {
            let text = facts.text(index).unwrap_or_default();
            return Err(Error::project(format!("[readings-differ] {text} {differ}")));
        }
    }
    of_the_readings(facts)
}

/// The outcome of a case from the readings of its expectations in order and
/// each `try` of one path that missed: `failed`, with each missed `try` and
/// then each expectation that did not hold, a project error when no
/// expectation ran, or else `passed`.
fn of_the_readings(facts: &Facts) -> Result<Kind, Error> {
    let failed: Vec<Quote> = facts
        .missed
        .iter()
        .map(|words| Quote::alone(words.clone()))
        .chain(
            facts
                .in_order
                .iter()
                .filter(|(_, held)| !held)
                .filter_map(|(index, _)| {
                    Some(Quote {
                        assert: facts.text(*index)?.to_owned(),
                        shown: facts
                            .shown
                            .iter()
                            .find(|(shown, _)| shown == index)
                            .map(|(_, shown)| shown.clone()),
                    })
                }),
        )
        .collect();
    if !failed.is_empty() {
        Ok(Kind::Failed { asserts: failed })
    } else if !facts.in_order.is_empty() {
        Ok(Kind::Passed)
    } else if !facts.stated {
        Err(Error::project(
            "[no-expectation] no expectation stated".to_owned(),
        ))
    } else {
        Err(Error::project(
            "[no-expectation] no expectation ran".to_owned(),
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The line that `order` gives for `facts` with no suppression, as text a
    /// row can compare.
    fn line(facts: &Facts) -> String {
        match order(facts, &[]).0 {
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
}
