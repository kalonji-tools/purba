//! The order that decides the outcome of a case from all that the case met:
//! the facts that its run gathered, and what the test's process, its replays
//! and purba's process saw besides them. It reads no Python, so the process of the test
//! gathers the facts, and purba decides the outcome once it has read each
//! expectation alone.

use std::ops::RangeInclusive;

use purba_assertion::{Swallowed, Try, Verdict};
use purba_errors::Shown;

use crate::zygote::{gone, lost};
use crate::{Error, ErrorKind, Kind, Quote, SUPPRESSIBLE, Suppressed, Warning};

/// How a line of a path failed.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum How {
    /// The line did not run.
    DidNotRun,
    /// The branch on the line went one way only.
    OneWay,
}

/// A fault of a path of a case. `line` is the line of the test. `reached` is
/// the line and the name of the code that the test reached from it, when that
/// code is not the test's own. `how` says how the line of the fault failed: the
/// reached line when there is one, or else the line of the test.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PathFault {
    pub line: usize,
    pub reached: Option<(usize, String)>,
    pub how: How,
}

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
/// beside whether it held. It holds each fault of the paths, each `with` that
/// swallowed a raise, and each `try` of the test. It holds why no reader or
/// replay read an expectation alone, when a fork failed or the module's process
/// ended.
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
    pub path: Vec<PathFault>,
    pub swallowed: Vec<Swallowed>,
    pub tries: Vec<Try>,
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

/// What a case met: its facts, and what the test's process and purba's
/// process saw besides them.
#[derive(Debug)]
pub struct Case {
    /// The facts, when the test's call ran.
    pub facts: Option<Facts>,
    /// Each error that the case met, in the order that purba read it: the
    /// step's own error, a panic that the hook noted, a frame that holds no
    /// message, the error of a replay.
    pub errors: Vec<Error>,
    /// The first interrupt landed in the test's step, or arrived during its
    /// replays.
    pub landed: bool,
    /// An interrupt arrived as the case ran.
    pub arrived: bool,
    /// The wait status of the test's process, or `Err` with the wait status of
    /// the module's process when it ended first.
    pub end: Result<libc::c_int, libc::c_int>,
}

impl Case {
    /// Each later expectation that a replay must read alone. None when an
    /// error or an interrupt decides the case above its facts, or when the
    /// module's process ended.
    #[must_use]
    pub fn later(&self) -> &[usize] {
        match &self.facts {
            Some(facts) if self.errors.is_empty() && !self.landed && self.end.is_ok() => {
                &facts.later
            }
            _ => &[],
        }
    }

    /// The case of a test's process that sent `facts` and met nothing else.
    #[cfg(test)]
    pub(crate) const fn of(facts: Facts) -> Self {
        Self {
            facts: Some(facts),
            errors: Vec::new(),
            landed: false,
            arrived: false,
            end: Ok(0),
        }
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

/// The rule that a fault of a path breaks.
const RULE: &str = "the lines before the first expectation take one path";

/// The faults of the paths of a case, each beside the name of its check and
/// whether it lies in a `try` of another shape than one path. A `with` that
/// swallowed a raise lies there when a line that it skipped does. A fault that
/// reached other code lies in no such `try`. The faults
/// sort by their lines, and a `with` sorts by its own line. A fault of a line
/// that a `with` skipped, or of a line in the `except` of a `try` of one path,
/// is no fault, unless the test reached other code from that line.
fn path_faults(facts: &Facts) -> Vec<(&'static str, String, bool)> {
    let in_another_try = |lines: RangeInclusive<usize>| {
        facts.tries.iter().any(|attempt| {
            attempt.except.is_none()
                && attempt.lines.start() <= lines.end()
                && lines.start() <= attempt.lines.end()
        })
    };
    let explained = |line: usize| {
        facts
            .swallowed
            .iter()
            .any(|swallow| (swallow.skipped..=swallow.last).contains(&line))
            || facts.tries.iter().any(|attempt| {
                attempt
                    .except
                    .as_ref()
                    .is_some_and(|except| except.contains(&line))
            })
    };
    let mut faults: Vec<_> = facts
        .path
        .iter()
        .filter(|fault| fault.reached.is_some() || !explained(fault.line))
        .map(|fault| {
            let what = match fault.how {
                How::DidNotRun => "did not run",
                How::OneWay => "went one way only",
            };
            let words = match &fault.reached {
                Some((at, name)) => format!(
                    "line {} reaches line {at} of {name}, which {what}, and {RULE}",
                    fault.line
                ),
                None => format!("line {} {what}, and {RULE}", fault.line),
            };
            (
                fault.line,
                fault.reached.as_ref().map(|(at, _)| *at),
                words,
                "second-path",
                fault.reached.is_none() && in_another_try(fault.line..=fault.line),
            )
        })
        .chain(facts.swallowed.iter().map(|swallow| {
            (
                swallow.with,
                None,
                format!(
                    "the with at line {} swallowed a raise, and line {} did not run",
                    swallow.with, swallow.skipped
                ),
                "swallowed-raise",
                in_another_try(swallow.skipped..=swallow.last),
            )
        }))
        .collect();
    faults.sort();
    faults.dedup();
    faults
        .into_iter()
        .map(|(_, _, words, check, hint)| (check, words, hint))
        .collect()
}

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

/// The outcome of `case`, and each warning that `suppressions` gave. The first
/// of these wins, numbered as the record's table numbers them:
///
/// 0. the error that purba read last of those that are not a project error;
/// 1. `interrupted` when the first interrupt landed in the test's step or
///    arrived during its replays;
/// 2. the project error that the test's step or a replay gave;
/// 3. a project error for the first fault of each check of the form in turn,
///    then for a reading alone that an assert ran inside;
/// 4. `raised` when the call raised, unless a `with`'s entry or cleanup raised
///    it, or a marked assert that did not hold raised `AssertionError`, which
///    are `failed`;
/// 5. a project error for the faults of the paths, joined by `; `. The error
///    ends with the shape of a `try` of one path when a fault that it names
///    lies in a `try` of another shape;
/// 6. a project error when the two readings of one expectation differ;
/// 7. a project error when a fork to read a later expectation alone failed, or
///    when the module's process ended before a replay sent its reading;
/// 8. `failed` when a `try` of one path missed, or an expectation did not
///    hold, then a project error when no expectation ran, then `passed`;
/// 9. with no facts: a project error when the module's process ended, then
///    `interrupted` when an interrupt arrived, then `raised` that names the
///    signal or the exit code of the test's process.
///
/// A fault of a check that a suppression reaches gives a warning in place of
/// its project error, and the case goes on to the next of these. Each project
/// error of a strict check names its check in brackets before the words of
/// each fault. The order judges each suppression of a check that execution
/// sees only when it reaches step 5. Under step 0, 1 or 2, it still reads the
/// facts for their warnings.
pub fn order(mut case: Case, suppressions: &[(&str, &str)]) -> (Result<Kind, Error>, Suppressed) {
    if let (Some(facts), Err(status)) = (case.facts.as_mut(), case.end)
        && !facts.later.is_empty()
    {
        facts.refused.get_or_insert_with(|| gone(status));
    }
    let mut suppressing = Suppressing {
        suppressions,
        warnings: Vec::new(),
        judged: false,
    };
    let of_the_facts = case
        .facts
        .as_ref()
        .map(|facts| steps(facts, &mut suppressing));
    let internal = case
        .errors
        .iter()
        .rposition(|error| error.kind() != ErrorKind::Project);
    let own = if let Some(at) = internal {
        Err(case.errors.swap_remove(at))
    } else if case.landed {
        Ok(Kind::Interrupted)
    } else if let Some(error) = case.errors.pop() {
        Err(error)
    } else if let Some(own) = of_the_facts {
        own
    } else {
        match case.end {
            Ok(_) if case.arrived => Ok(Kind::Interrupted),
            Ok(status) => Ok(Kind::Raised {
                exception: lost(status, "its outcome"),
                whence: None,
            }),
            Err(status) => Err(Error::project(gone(status))),
        }
    };
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
    let mut in_another_try = false;
    let path: Vec<String> = path_faults(facts)
        .into_iter()
        .filter_map(|(check, words, hint)| {
            let clause = suppressing.clause(check, &words)?;
            in_another_try |= hint;
            Some(clause)
        })
        .collect();
    if !path.is_empty() {
        let mut line = path.join("; ");
        if in_another_try {
            line.push_str("; ");
            line.push_str(ONE_PATH);
        }
        return Err(Error::project(line));
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
    if let Some(refused) = &facts.refused {
        return Err(Error::project(refused.clone()));
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
    use std::ops::RangeInclusive;

    use super::*;

    /// The line that `order` gives for `facts` with no suppression, as text a
    /// row can compare.
    fn line(facts: &Facts) -> String {
        shown(order(Case::of(facts.clone()), &[]).0)
    }

    /// What `warned` gives: the line, each warning as `[<check>] <words>`, and
    /// whether the order judged the suppressions.
    type Warned = (String, Vec<String>, bool);

    /// A row of a test of suppressions: what it checks, the facts of a case,
    /// its suppressions, and what `warned` gives.
    type Row<'a> = (&'a str, Facts, &'a [(&'a str, &'a str)], Warned);

    /// What `order` gives for `facts` under `suppressions`.
    fn warned(facts: &Facts, suppressions: &[(&str, &str)]) -> Warned {
        let (own, suppressed) = order(Case::of(facts.clone()), suppressions);
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

    /// A fault of the line `line` of the test, which did not run.
    fn not_run(line: usize) -> PathFault {
        PathFault {
            line,
            reached: None,
            how: How::DidNotRun,
        }
    }

    /// A fault of the branch on the line `line` of the test, which went one
    /// way only.
    fn one_way(line: usize) -> PathFault {
        PathFault {
            line,
            reached: None,
            how: How::OneWay,
        }
    }

    /// A fault of the line `at` of `name`, which the line `line` of the test
    /// reached, and which did not run.
    fn reached(line: usize, at: usize, name: &str) -> PathFault {
        PathFault {
            line,
            reached: Some((at, name.to_owned())),
            how: How::DidNotRun,
        }
    }

    /// The `with` on the line `with`, which swallowed a raise, and skipped the
    /// lines from `skipped` to `last`.
    const fn swallowed(with: usize, skipped: usize, last: usize) -> Swallowed {
        Swallowed {
            with,
            skipped,
            last,
        }
    }

    /// A `try` of another shape than one path, on `lines`.
    const fn another_shape(lines: RangeInclusive<usize>) -> Try {
        Try {
            lines,
            except: None,
        }
    }

    /// A `try` of one path, on `lines`, whose `except` is on `except`.
    const fn one_path(lines: RangeInclusive<usize>, except: RangeInclusive<usize>) -> Try {
        Try {
            lines,
            except: Some(except),
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
        let of_the_facts = rows_of_the_call()
            .into_iter()
            .chain(rows_of_the_readings())
            .chain(rows_of_a_fork())
            .map(|(row, facts, gives)| (row, Case::of(facts), gives));
        for (row, case, gives) in rows_of_the_case()
            .into_iter()
            .chain(rows_of_the_end())
            .chain(of_the_facts)
        {
            assert_eq!(shown(order(case, &[]).0), gives, "{row}");
        }
    }

    /// An internal error that execution found, with `message`.
    fn internal(message: &str) -> Error {
        crate::internal(crate::panic::Caught {
            part: "execution".to_owned(),
            message: message.to_owned(),
        })
    }

    /// The facts of a case whose one expectation held.
    fn held() -> Facts {
        Facts {
            in_order: vec![(1, true)],
            texts: texts(),
            stated: true,
            ..Facts::default()
        }
    }

    /// The case of a test's process that sent no facts, and met nothing else.
    fn no_facts() -> Case {
        Case {
            facts: None,
            ..Case::of(Facts::default())
        }
    }

    /// The rows of steps 0, 1 and 2, which read what the case met besides its
    /// facts.
    fn rows_of_the_case() -> Vec<(&'static str, Case, &'static str)> {
        vec![
            // fails when: the check of `landed` moves above the check of an
            // internal error
            (
                "an internal error beats an interrupt",
                Case {
                    errors: vec![internal("boom")],
                    landed: true,
                    ..Case::of(held())
                },
                "internal error: execution: boom",
            ),
            // fails when: step 0 takes the first error that purba read
            (
                "an internal error that purba read later hides one before it",
                Case {
                    errors: vec![internal("own"), internal("panic")],
                    ..Case::of(held())
                },
                "internal error: execution: panic",
            ),
            // fails when: step 0 takes the last error of any kind
            (
                "an internal error beats a project error that purba read after it",
                Case {
                    errors: vec![
                        internal("frame"),
                        Error::project("x.py did not load".to_owned()),
                    ],
                    ..no_facts()
                },
                "internal error: execution: frame",
            ),
            // fails when: the facts are read above an internal error
            (
                "an internal error beats an expectation that did not hold",
                Case {
                    errors: vec![internal("frame")],
                    ..Case::of(Facts {
                        in_order: vec![(1, false)],
                        later: vec![2],
                        ..held()
                    })
                },
                "internal error: execution: frame",
            ),
            // fails when: the facts are read above the check of `landed`
            (
                "an interrupt beats a raise",
                Case {
                    landed: true,
                    ..Case::of(Facts {
                        called: raised("ValueError: v"),
                        ..held()
                    })
                },
                "interrupted",
            ),
            // fails when: the step's own error moves above the check of `landed`
            (
                "an interrupt beats the step's own project error",
                Case {
                    errors: vec![Error::project("x.py did not load".to_owned())],
                    landed: true,
                    ..no_facts()
                },
                "interrupted",
            ),
            // fails when: the form is read above the error that a replay gave
            (
                "the project error that a replay gave beats a fault of the form",
                Case {
                    errors: vec![Error::project("x.py did not load".to_owned())],
                    ..Case::of(Facts {
                        form: vec![(
                            "assert-before-expectation".to_owned(),
                            "assert ready runs before the first expectation".to_owned(),
                        )],
                        ..held()
                    })
                },
                "project error: x.py did not load",
            ),
            // fails when: the facts are read above the error that a replay gave
            (
                "the project error that a replay gave beats a raise",
                Case {
                    errors: vec![Error::project("x.py did not load".to_owned())],
                    ..Case::of(Facts {
                        called: raised("ValueError: v"),
                        ..held()
                    })
                },
                "project error: x.py did not load",
            ),
        ]
    }

    /// The rows that read `arrived` and `end`: steps 2 and 8 over an
    /// interrupt that arrived, step 8 over the end of the test's process,
    /// step 7 when the module's process ended, and step 9, which reads a case
    /// with no facts.
    fn rows_of_the_end() -> Vec<(&'static str, Case, &'static str)> {
        vec![
            // fails when: the check of `arrived` moves above the facts
            (
                "an interrupt that arrived but did not land leaves the facts to the order",
                Case {
                    arrived: true,
                    ..Case::of(held())
                },
                "passed",
            ),
            // fails when: the end of the test's process is read above the facts
            (
                "a test's process that died by a signal after it sent its facts leaves them \
                 to the order",
                Case {
                    end: Ok(libc::SIGABRT),
                    ..Case::of(held())
                },
                "passed",
            ),
            // fails when: the order no longer puts the end of the module's
            // process in place of a refusal
            (
                "a module's process that ended before a replay beats an expectation \
                 that did not hold",
                Case {
                    end: Err(libc::SIGKILL),
                    ..Case::of(Facts {
                        in_order: vec![(1, false)],
                        later: vec![2],
                        ..held()
                    })
                },
                "project error: the module's process died by SIGKILL",
            ),
            // fails when: the end of the module's process replaces a refusal
            // that a replay gave
            (
                "a refusal that a replay gave beats the end of the module's process",
                Case {
                    end: Err(libc::SIGKILL),
                    ..Case::of(Facts {
                        later: vec![2],
                        refused: Some(
                            "purba cannot fork to read an expectation alone: EAGAIN".to_owned(),
                        ),
                        ..held()
                    })
                },
                "project error: purba cannot fork to read an expectation alone: EAGAIN",
            ),
            // fails when: the check of `arrived` moves above the step's own error
            (
                "the step's own error beats an interrupt that arrived",
                Case {
                    errors: vec![Error::project("x.py did not load".to_owned())],
                    arrived: true,
                    ..no_facts()
                },
                "project error: x.py did not load",
            ),
            // fails when: the check of `arrived` is deleted
            (
                "with no facts, an interrupt that arrived beats the end of the test's process",
                Case {
                    arrived: true,
                    ..no_facts()
                },
                "interrupted",
            ),
            // fails when: an exit code reads as a signal
            (
                "with no facts, a test's process that exited gives raised with its code",
                Case {
                    end: Ok(3 << 8),
                    ..no_facts()
                },
                "raised: its process exited 3 before its outcome",
            ),
            // fails when: a signal reads as an exit code
            (
                "with no facts, a test's process that died by a signal gives raised with \
                 the signal",
                Case {
                    end: Ok(libc::SIGABRT),
                    ..no_facts()
                },
                "raised: its process died by SIGABRT",
            ),
            // fails when: the check of `arrived` moves above the end of the
            // module's process
            (
                "with no facts, the end of the module's process gives a project error",
                Case {
                    arrived: true,
                    end: Err(libc::SIGKILL),
                    ..no_facts()
                },
                "project error: the module's process died by SIGKILL",
            ),
        ]
    }

    #[test]
    fn a_replay_is_asked_only_when_no_step_above_the_facts_decides() {
        let later = || Facts {
            later: vec![2],
            ..held()
        };
        let rows: Vec<(&str, Case, &[usize])> = vec![
            // fails when: `later` gives nothing
            ("a case that met nothing", Case::of(later()), &[2]),
            // fails when: `later` does not read `errors`
            (
                "a case that met an error",
                Case {
                    errors: vec![internal("boom")],
                    ..Case::of(later())
                },
                &[],
            ),
            // fails when: `later` does not read `landed`
            (
                "a case that an interrupt landed in",
                Case {
                    landed: true,
                    ..Case::of(later())
                },
                &[],
            ),
            // fails when: `later` does not read `end`
            (
                "a case whose module's process ended",
                Case {
                    end: Err(libc::SIGKILL),
                    ..Case::of(later())
                },
                &[],
            ),
        ];
        for (row, case, later) in rows {
            assert_eq!(case.later(), later, "{row}");
        }
    }

    /// The rows of steps 3 and 4, which read the form and the call.
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
                    path: vec![one_way(4)],
                    stated: true,
                    ..Facts::default()
                },
                "raised: ValueError: v",
            ),
        ]
    }

    /// The rows of steps 5, 6 and 8, which read the paths and the readings.
    fn rows_of_the_readings() -> Vec<Line<'static>> {
        vec![
            // fails when: the check of the readings that differ moves above the
            // check of `path`
            (
                "two faults of the paths, joined, beat two readings that differ",
                Facts {
                    path: vec![one_way(4)],
                    swallowed: vec![swallowed(6, 7, 7)],
                    in_order: vec![(1, true)],
                    alone: vec![(1, Verdict::Fails)],
                    texts: texts(),
                    stated: true,
                    ..Facts::default()
                },
                "project error: [second-path] line 4 went one way only, and the lines \
                 before the first expectation take one path; [swallowed-raise] the with \
                 at line 6 swallowed a raise, and line 7 did not run",
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

    /// The rows of a fork that failed, which step 7 reads after the paths and
    /// the readings that differ.
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
                    path: vec![one_way(4)],
                    refused: Some(
                        "purba cannot fork to read an expectation alone: EAGAIN".to_owned(),
                    ),
                    in_order: vec![(1, true)],
                    texts: texts(),
                    stated: true,
                    ..Facts::default()
                },
                "project error: [second-path] line 4 went one way only, and the lines \
                 before the first expectation take one path",
            ),
            // fails when: the check of `refused` moves above the check of the
            // readings that differ
            (
                "two readings that differ beat a fork that failed",
                Facts {
                    refused: Some(
                        "purba cannot fork to read an expectation alone: EAGAIN".to_owned(),
                    ),
                    in_order: vec![(1, true)],
                    alone: vec![(1, Verdict::Fails)],
                    later: vec![2],
                    texts: texts(),
                    stated: true,
                    ..Facts::default()
                },
                "project error: [readings-differ] assert x == 1 holds after the \
                 expectations before it, and not when read alone",
            ),
        ]
    }

    #[test]
    fn a_fault_that_another_fact_explains_gives_no_clause() {
        for (row, facts, gives) in rows_of_the_filter().into_iter().chain(rows_of_the_sort()) {
            assert_eq!(line(&facts), gives, "{row}");
        }
    }

    /// The rows of the faults that another fact explains.
    fn rows_of_the_filter() -> Vec<Line<'static>> {
        vec![
            // fails when: the check of `swallowed` is deleted from the filter
            (
                "a line that a swallowing with skipped gives only the clause of the with",
                Facts {
                    path: vec![not_run(7), not_run(8)],
                    swallowed: vec![swallowed(6, 7, 8)],
                    in_order: vec![(1, true)],
                    texts: texts(),
                    stated: true,
                    ..Facts::default()
                },
                "project error: [swallowed-raise] the with at line 6 swallowed a raise, \
                 and line 7 did not run",
            ),
            // fails when: the check of `except` is deleted from the filter
            (
                "a line in the except of a try of one path gives no clause",
                Facts {
                    path: vec![not_run(6)],
                    tries: vec![one_path(4..=6, 6..=6)],
                    in_order: vec![(1, true)],
                    texts: texts(),
                    stated: true,
                    ..Facts::default()
                },
                "passed",
            ),
            // fails when: the filter drops a fault that reached other code
            (
                "a line that a swallowing with skipped keeps a fault that it reached in a helper",
                Facts {
                    path: vec![reached(7, 2, "key")],
                    swallowed: vec![swallowed(6, 7, 7)],
                    in_order: vec![(1, true)],
                    texts: texts(),
                    stated: true,
                    ..Facts::default()
                },
                "project error: [swallowed-raise] the with at line 6 swallowed a raise, \
                 and line 7 did not run; [second-path] line 7 reaches line 2 of key, \
                 which did not run, and the lines before the first expectation take one \
                 path",
            ),
            // fails when: the filter of an `except` drops a fault that reached
            // other code
            (
                "an except of a try of one path keeps a fault that it reached in a helper",
                Facts {
                    path: vec![reached(6, 2, "key")],
                    tries: vec![one_path(4..=6, 6..=6)],
                    in_order: vec![(1, true)],
                    texts: texts(),
                    stated: true,
                    ..Facts::default()
                },
                "project error: [second-path] line 6 reaches line 2 of key, which did not \
                 run, and the lines before the first expectation take one path",
            ),
            // fails when: the filter drops a fault of the line after the last
            // line that a swallowing with skipped
            (
                "a fault of the line after the last line that a with skipped stays",
                Facts {
                    path: vec![one_way(9)],
                    swallowed: vec![swallowed(6, 8, 8)],
                    in_order: vec![(1, true)],
                    texts: texts(),
                    stated: true,
                    ..Facts::default()
                },
                "project error: [swallowed-raise] the with at line 6 swallowed a raise, \
                 and line 8 did not run; [second-path] line 9 went one way only, and the \
                 lines before the first expectation take one path",
            ),
        ]
    }

    /// The rows of the order of the faults, and of their repeats.
    fn rows_of_the_sort() -> Vec<Line<'static>> {
        vec![
            // fails when: the faults are not sorted by their lines
            (
                "the faults come in the order of their lines",
                Facts {
                    path: vec![not_run(9), one_way(3)],
                    swallowed: vec![swallowed(6, 7, 7)],
                    in_order: vec![(1, true)],
                    texts: texts(),
                    stated: true,
                    ..Facts::default()
                },
                "project error: [second-path] line 3 went one way only, and the lines \
                 before the first expectation take one path; [swallowed-raise] the with \
                 at line 6 swallowed a raise, and line 7 did not run; [second-path] line \
                 9 did not run, and the lines before the first expectation take one path",
            ),
            // fails when: the repeats of a fault are not dropped
            (
                "a fault that the facts hold twice gives one clause",
                Facts {
                    path: vec![not_run(3), not_run(3)],
                    in_order: vec![(1, true)],
                    texts: texts(),
                    stated: true,
                    ..Facts::default()
                },
                "project error: [second-path] line 3 did not run, and the lines before \
                 the first expectation take one path",
            ),
            // fails when: a swallowing with sorts at the first line that it
            // skipped
            (
                "a swallowing with sorts at its own line, before a fault of the line that raised",
                Facts {
                    path: vec![one_way(7)],
                    swallowed: vec![swallowed(6, 8, 8)],
                    in_order: vec![(1, true)],
                    texts: texts(),
                    stated: true,
                    ..Facts::default()
                },
                "project error: [swallowed-raise] the with at line 6 swallowed a raise, \
                 and line 8 did not run; [second-path] line 7 went one way only, and the \
                 lines before the first expectation take one path",
            ),
            // fails when: the sort leaves out the line that a fault reached
            (
                "two faults of one line sort by the lines that they reached, as numbers",
                Facts {
                    path: vec![reached(5, 12, "key"), reached(5, 9, "key")],
                    in_order: vec![(1, true)],
                    texts: texts(),
                    stated: true,
                    ..Facts::default()
                },
                "project error: [second-path] line 5 reaches line 9 of key, which did not \
                 run, and the lines before the first expectation take one path; \
                 [second-path] line 5 reaches line 12 of key, which did not run, and the \
                 lines before the first expectation take one path",
            ),
        ]
    }

    #[test]
    fn a_suppressed_fault_gives_a_warning_and_the_case_goes_on() {
        let words = "the with at line 3 swallowed a raise, and line 4 did not run";
        let rows: Vec<Row<'_>> = vec![
            // fails when: `clause` gives back the clause of a check that a
            // suppression reaches, in place of the warning
            (
                "a suppressed fault of the paths gives a warning, and the case passes",
                Facts {
                    swallowed: vec![swallowed(3, 4, 4)],
                    in_order: vec![(1, true)],
                    texts: texts(),
                    stated: true,
                    ..Facts::default()
                },
                &[("swallowed-raise", "the act once swallowed a raise")],
                (
                    "passed".to_owned(),
                    vec![format!("[swallowed-raise] {words}")],
                    true,
                ),
            ),
            // fails when: `judged` is set before the check of `called`
            (
                "a raise stops the order before it judges a suppression",
                Facts {
                    called: raised("ValueError: v"),
                    swallowed: vec![swallowed(3, 4, 4)],
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

    /// The line of a project error of `clause`.
    fn error(clause: &str) -> String {
        format!("project error: {clause}")
    }

    /// The line of a project error of `clause` that ends with the shape of a
    /// `try` of one path.
    fn hinted(clause: &str) -> String {
        format!("project error: {clause}; {ONE_PATH}")
    }

    #[test]
    fn the_hint_ends_a_line_only_for_a_fault_it_names_in_a_try_of_another_shape() {
        for (row, facts, suppressions, gives) in hint_rows_of_a_line()
            .into_iter()
            .chain(hint_rows_of_a_branch())
            .chain(hint_rows_of_a_with())
            .chain(hint_rows_of_a_with_across_a_try())
        {
            assert_eq!(warned(&facts, suppressions), gives, "{row}");
        }
    }

    /// The rows of the hint for a branch of the test that went one way only.
    fn hint_rows_of_a_branch() -> Vec<Row<'static>> {
        vec![
            // fails when: the hint reads only a line that did not run
            (
                "a branch of a try of another shape that went one way gives the hint",
                Facts {
                    path: vec![one_way(5)],
                    tries: vec![another_shape(4..=7)],
                    stated: true,
                    ..Facts::default()
                },
                &[],
                (
                    hinted(&format!(
                        "[second-path] line 5 went one way only, and {RULE}"
                    )),
                    vec![],
                    true,
                ),
            ),
            // fails when: the hint reads only the first try that holds the line
            (
                "a line of a try of one path inside a try of another shape gives the hint",
                Facts {
                    path: vec![one_way(4)],
                    tries: vec![one_path(3..=6, 5..=6), another_shape(2..=9)],
                    stated: true,
                    ..Facts::default()
                },
                &[],
                (
                    hinted(&format!(
                        "[second-path] line 4 went one way only, and {RULE}"
                    )),
                    vec![],
                    true,
                ),
            ),
        ]
    }

    /// The rows of the hint for a swallowing `with` whose skipped lines and a
    /// `try` of another shape only partly overlap.
    fn hint_rows_of_a_with_across_a_try() -> Vec<Row<'static>> {
        let with_at = |skipped: usize| {
            hinted(&format!(
                "[swallowed-raise] the with at line 6 swallowed a raise, and line {skipped} \
                 did not run"
            ))
        };
        vec![
            // fails when: a swallowing with gives the hint only when the try
            // holds the last line that it skipped
            (
                "a with whose skipped lines start inside a try of another shape gives the hint",
                Facts {
                    swallowed: vec![swallowed(6, 9, 12)],
                    tries: vec![another_shape(7..=11)],
                    stated: true,
                    ..Facts::default()
                },
                &[],
                (with_at(9), vec![], true),
            ),
            // fails when: a swallowing with gives the hint only when the try
            // holds the first line that it skipped
            (
                "a with whose skipped lines hold a try of another shape gives the hint",
                Facts {
                    swallowed: vec![swallowed(6, 8, 13)],
                    tries: vec![another_shape(10..=12)],
                    stated: true,
                    ..Facts::default()
                },
                &[],
                (with_at(8), vec![], true),
            ),
            // fails when: the lines that a swallowing with skipped must overlap
            // a try by more than one line
            (
                "a with whose first skipped line is the last line of a try gives the hint",
                Facts {
                    swallowed: vec![swallowed(6, 11, 12)],
                    tries: vec![another_shape(7..=11)],
                    stated: true,
                    ..Facts::default()
                },
                &[],
                (with_at(11), vec![], true),
            ),
            // fails when: a swallowing with gives no hint for a try that starts
            // on the last line that it skipped
            (
                "a with whose last skipped line is the first line of a try gives the hint",
                Facts {
                    swallowed: vec![swallowed(6, 8, 10)],
                    tries: vec![another_shape(10..=12)],
                    stated: true,
                    ..Facts::default()
                },
                &[],
                (with_at(8), vec![], true),
            ),
            // fails when: the hint reads the lines of a swallowing with from the
            // line of the with, or from the line after it
            (
                "a try between a with and the first line that it skipped gives no hint",
                Facts {
                    swallowed: vec![swallowed(6, 11, 11)],
                    tries: vec![another_shape(7..=10)],
                    stated: true,
                    ..Facts::default()
                },
                &[],
                (
                    error(
                        "[swallowed-raise] the with at line 6 swallowed a raise, and line 11 \
                         did not run",
                    ),
                    vec![],
                    true,
                ),
            ),
        ]
    }

    /// The rows of the hint for a fault of a line of the test.
    fn hint_rows_of_a_line() -> Vec<Row<'static>> {
        vec![
            // fails when: the order never adds the hint
            (
                "a line of a try of another shape that did not run gives the hint",
                Facts {
                    path: vec![not_run(5)],
                    tries: vec![another_shape(4..=7)],
                    stated: true,
                    ..Facts::default()
                },
                &[],
                (
                    hinted(&format!("[second-path] line 5 did not run, and {RULE}")),
                    vec![],
                    true,
                ),
            ),
            // fails when: the check of `except.is_none()` is deleted from the hint
            (
                "a line of a try of one path that did not run gives no hint",
                Facts {
                    path: vec![not_run(5)],
                    tries: vec![one_path(4..=7, 6..=7)],
                    stated: true,
                    ..Facts::default()
                },
                &[],
                (
                    error(&format!("[second-path] line 5 did not run, and {RULE}")),
                    vec![],
                    true,
                ),
            ),
            // fails when: the check of `reached` is deleted from the hint
            (
                "a fault that a line of a try of another shape reached in a helper gives no hint",
                Facts {
                    path: vec![reached(5, 2, "key")],
                    tries: vec![another_shape(4..=6)],
                    stated: true,
                    ..Facts::default()
                },
                &[],
                (
                    error(&format!(
                        "[second-path] line 5 reaches line 2 of key, which did not run, and {RULE}"
                    )),
                    vec![],
                    true,
                ),
            ),
            // fails when: the hint reads a fault of the paths that a suppression
            // took
            (
                "a suppressed fault in a try of another shape gives no hint to a swallowing with",
                Facts {
                    path: vec![not_run(9), not_run(14)],
                    swallowed: vec![swallowed(12, 14, 14)],
                    tries: vec![another_shape(7..=11)],
                    stated: true,
                    ..Facts::default()
                },
                &[("second-path", "the loop once read names")],
                (
                    error(
                        "[swallowed-raise] the with at line 12 swallowed a raise, and line 14 \
                         did not run",
                    ),
                    vec![format!("[second-path] line 9 did not run, and {RULE}")],
                    true,
                ),
            ),
        ]
    }

    /// The rows of the hint for a `with` that swallowed a raise.
    fn hint_rows_of_a_with() -> Vec<Row<'static>> {
        vec![
            // fails when: the hint reads a swallowing with that a suppression took
            (
                "a suppressed with that skipped a try of another shape gives no hint",
                Facts {
                    path: vec![not_run(2), not_run(9)],
                    swallowed: vec![swallowed(6, 8, 11)],
                    tries: vec![another_shape(8..=11)],
                    stated: true,
                    ..Facts::default()
                },
                &[("swallowed-raise", "the act once swallowed a raise")],
                (
                    error(&format!("[second-path] line 2 did not run, and {RULE}")),
                    vec![
                        "[swallowed-raise] the with at line 6 swallowed a raise, and line 8 \
                         did not run"
                            .to_owned(),
                    ],
                    true,
                ),
            ),
            // fails when: a swallowing with is read at the line of the with, in
            // place of the lines that it skipped
            (
                "a with that skipped a whole try of another shape gives the hint",
                Facts {
                    path: vec![not_run(8), not_run(9), not_run(11)],
                    swallowed: vec![swallowed(6, 8, 11)],
                    tries: vec![another_shape(8..=11)],
                    stated: true,
                    ..Facts::default()
                },
                &[],
                (
                    hinted(
                        "[swallowed-raise] the with at line 6 swallowed a raise, and line 8 \
                         did not run",
                    ),
                    vec![],
                    true,
                ),
            ),
            // fails when: a swallowing with gives the hint only when the try lies
            // inside the lines that it skipped
            (
                "a with inside a try of another shape gives the hint",
                Facts {
                    swallowed: vec![swallowed(6, 8, 8)],
                    tries: vec![another_shape(5..=10)],
                    stated: true,
                    ..Facts::default()
                },
                &[],
                (
                    hinted(
                        "[swallowed-raise] the with at line 6 swallowed a raise, and line 8 \
                         did not run",
                    ),
                    vec![],
                    true,
                ),
            ),
            // fails when: a swallowing with gives the hint whatever its lines
            (
                "a with that skipped no line of a try of another shape gives no hint",
                Facts {
                    swallowed: vec![swallowed(6, 8, 9)],
                    tries: vec![another_shape(2..=4), another_shape(10..=12)],
                    stated: true,
                    ..Facts::default()
                },
                &[],
                (
                    error(
                        "[swallowed-raise] the with at line 6 swallowed a raise, and line 8 \
                         did not run",
                    ),
                    vec![],
                    true,
                ),
            ),
        ]
    }
}
