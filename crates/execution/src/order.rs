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
/// one path. It holds why no reader or replay was forked when a fork failed.
/// It holds the verdict of each expectation read in order and of each read
/// alone, what each that did not hold shows, and each later expectation that a
/// replay must read alone. It holds the text of each expectation that it names,
/// the words of each missed `try`, and whether the test states an expectation.
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

    /// A row of a test of the order: what it checks, the facts of a case, and
    /// the line that `order` gives.
    type Line<'a> = (&'a str, Facts, &'a str);

    #[test]
    fn each_step_of_the_order_beats_each_step_below_it() {
        for (row, facts, gives) in rows_of_the_call()
            .into_iter()
            .chain(rows_of_the_readings())
            .chain(rows_of_a_fork())
        {
            assert_eq!(line(&facts), gives, "{row}");
        }
    }

    /// The rows of steps 1 and 2, which read the form and the call.
    fn rows_of_the_call() -> Vec<Line<'static>> {
        vec![
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
            // fails when: the check of `alone` moves above the check of `form`
            (
                "a fault of the form beats a reading alone that reached an assert",
                Facts {
                    form: vec![(
                        "assert-before-expectation".to_owned(),
                        "assert ready runs before the first expectation".to_owned(),
                    )],
                    in_order: vec![(1, true)],
                    alone: vec![(2, Verdict::Refused)],
                    texts: texts(),
                    stated: true,
                    ..Facts::default()
                },
                "project error: [assert-before-expectation] assert ready runs \
                 before the first expectation",
            ),
            // fails when: the check of `with` is deleted
            (
                "a raise from a with's entry or cleanup is failed, not raised",
                Facts {
                    called: raised("OSError: no such file"),
                    with: Some("with opened(path) as handle:".to_owned()),
                    in_order: vec![(1, true)],
                    texts: texts(),
                    stated: true,
                    ..Facts::default()
                },
                "failed: with opened(path) as handle:",
            ),
            // fails when: the check of `mark` is deleted
            (
                "a marked assert that did not hold is failed, not raised",
                Facts {
                    called: Called::Raised {
                        exception: "AssertionError".to_owned(),
                        whence: None,
                        assertion: true,
                    },
                    mark: Some(("assert ready".to_owned(), false)),
                    stated: true,
                    ..Facts::default()
                },
                "failed: assert ready",
            ),
            // fails when: the check of `path` moves above the check of `called`
            (
                "a raise beats a fault of the paths",
                Facts {
                    called: raised("ValueError: v"),
                    path: vec![(
                        "second-path".to_owned(),
                        "line 4 ran one way of its branch".to_owned(),
                    )],
                    stated: true,
                    ..Facts::default()
                },
                "raised: ValueError: v",
            ),
        ]
    }

    /// The rows of steps 3 to 7, which read the paths and the readings.
    fn rows_of_the_readings() -> Vec<Line<'static>> {
        vec![
            // fails when: the check of the readings that differ moves above the
            // check of `path`
            (
                "two faults of the paths, joined, beat two readings that differ",
                Facts {
                    path: vec![
                        (
                            "second-path".to_owned(),
                            "line 4 ran one way of its branch".to_owned(),
                        ),
                        (
                            "swallowed-raise".to_owned(),
                            "the with at line 6 swallowed a raise, and line 7 did not run"
                                .to_owned(),
                        ),
                    ],
                    in_order: vec![(1, true)],
                    alone: vec![(1, Verdict::Fails)],
                    texts: texts(),
                    stated: true,
                    ..Facts::default()
                },
                "project error: [second-path] line 4 ran one way of its branch; \
                 [swallowed-raise] the with at line 6 swallowed a raise, and line 7 \
                 did not run",
            ),
            // fails when: the check of the readings that differ is deleted
            (
                "two readings that differ beat an expectation that did not hold",
                Facts {
                    in_order: vec![(1, true), (2, false)],
                    alone: vec![(1, Verdict::Fails)],
                    texts: texts(),
                    stated: true,
                    ..Facts::default()
                },
                "project error: [readings-differ] assert x == 1 holds after the \
                 expectations before it, and not when read alone",
            ),
            // fails when: `of_the_readings` quotes the expectations before the
            // missed `try`s
            (
                "a missed try and an expectation that did not hold are quoted, the try first",
                Facts {
                    in_order: vec![(1, false)],
                    texts: texts(),
                    missed: vec![
                        "the try at line 3 expects KeyError, and the statement did not raise it"
                            .to_owned(),
                    ],
                    stated: true,
                    ..Facts::default()
                },
                "failed: the try at line 3 expects KeyError, and the statement did not \
                 raise it, assert x == 1",
            ),
            // fails when: `of_the_readings` gives "no expectation ran" whatever
            // `stated` holds
            (
                "a test that states no expectation and runs none is a project error",
                Facts::default(),
                "project error: [no-expectation] no expectation stated",
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
            // fails when: `of_the_readings` quotes each expectation, held or not
            (
                "a case whose expectations each held passed",
                Facts {
                    in_order: vec![(1, true), (2, true)],
                    texts: texts(),
                    stated: true,
                    ..Facts::default()
                },
                "passed",
            ),
        ]
    }

    /// The rows of a fork that failed, which step 4 reads between the paths and
    /// the readings.
    fn rows_of_a_fork() -> Vec<Line<'static>> {
        vec![
            // fails when: the check of `refused` is deleted
            (
                "a fork that failed beats an expectation that did not hold",
                Facts {
                    refused: Some(
                        "purba cannot fork to read an expectation alone: EAGAIN".to_owned(),
                    ),
                    in_order: vec![(1, false)],
                    later: vec![2],
                    texts: texts(),
                    stated: true,
                    ..Facts::default()
                },
                "project error: purba cannot fork to read an expectation alone: EAGAIN",
            ),
            // fails when: the check of `refused` moves above the check of `path`
            (
                "a fault of the paths beats a fork that failed",
                Facts {
                    path: vec![(
                        "second-path".to_owned(),
                        "line 4 ran one way of its branch".to_owned(),
                    )],
                    refused: Some(
                        "purba cannot fork to read an expectation alone: EAGAIN".to_owned(),
                    ),
                    in_order: vec![(1, true)],
                    texts: texts(),
                    stated: true,
                    ..Facts::default()
                },
                "project error: [second-path] line 4 ran one way of its branch",
            ),
        ]
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
            // fails when: `judged` is set before the check of `called`
            (
                "a raise stops the order before it judges a suppression",
                Facts {
                    called: raised("ValueError: v"),
                    path: vec![("swallowed-raise".to_owned(), swallowed.to_owned())],
                    stated: true,
                    ..Facts::default()
                },
                &[("swallowed-raise", "the act once swallowed a raise")],
                ("raised: ValueError: v".to_owned(), vec![], false),
            ),
        ];
        for (row, facts, suppressions, gives) in rows {
            assert_eq!(warned(&facts, suppressions), gives, "{row}");
        }
    }
}
