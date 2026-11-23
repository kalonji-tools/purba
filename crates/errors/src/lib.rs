//! The crate of the errors concern: the names that keep a test's outcome, a
//! project error and an internal error apart, and their tally. It links no
//! Python, so none of them can hold a live Python object.

pub mod panic;

use std::any::Any;
use std::fmt;
use std::panic::AssertUnwindSafe;

use serde::{Deserialize, Serialize};

/// What a run says about one test. Its field is private, so no part changes
/// it once execution makes it.
///
/// ```compile_fail,E0451
/// let outcome = purba_errors::Outcome { kind: purba_errors::Kind::Passed };
/// ```
///
/// ```compile_fail,E0616
/// fn forge(outcome: &mut purba_errors::Outcome) {
///     outcome.kind = purba_errors::Kind::Passed;
/// }
/// ```
#[derive(Debug, Serialize, Deserialize)]
pub struct Outcome {
    kind: Kind,
}

impl Outcome {
    /// The outcome of `kind`. Only execution makes an outcome. Clippy refuses
    /// a call of this function outside execution.
    #[must_use]
    pub const fn new(kind: Kind) -> Self {
        Self { kind }
    }

    /// Whether the case passed, failed, raised, or is `interrupted`.
    #[must_use]
    pub const fn kind(&self) -> &Kind {
        &self.kind
    }
}

/// Passed, failed, raised or interrupted.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Kind {
    /// Each expectation that the case read held, and the test returned.
    Passed,
    /// An expectation did not hold, and the test returned, or a marked assert
    /// did not hold, and the test raised `AssertionError`. `asserts` quotes each
    /// expectation that did not hold, in order, or the last marked assert that
    /// did not hold.
    Failed { asserts: Vec<Quote> },
    /// The test's call raised, and the outcome is not `Failed` or
    /// `Interrupted`. A test whose process ends before its outcome gives
    /// `Raised` too. `exception` is the type and the message of the exception,
    /// or how the process ended, as text. `whence` names the frame that the
    /// exception came from, when it is not the test's own.
    Raised {
        exception: String,
        whence: Option<String>,
    },
    /// The first interrupt landed during the test's step: its lookup or its
    /// call. The test has this outcome whether it then passed, failed or raised.
    Interrupted,
}

/// The text of an assert that did not hold, or of the `with` that raised, and
/// what an expectation that compares two values shows.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Quote {
    pub assert: String,
    pub shown: Option<Shown>,
}

impl Quote {
    /// A quote of `text` that shows no reading.
    #[must_use]
    pub const fn alone(text: String) -> Self {
        Self {
            assert: text,
            shown: None,
        }
    }
}

/// What an expectation that did not hold shows.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Shown {
    /// A check of one value: its expected value and its reading. Each is the
    /// `repr` of its value, beside the name of its class when the two `repr`s
    /// are the same. The expected value reads by the rule of its operator.
    Values { expected: String, reading: String },
    /// An `all()` or an `any()`: a heading, and a line for each item that a
    /// failure names.
    Items { heading: String, items: Vec<String> },
}

/// Why a run made no outcome: a project error or an internal error.
///
/// A project error is text, such as Python's error or a test that selection
/// refuses. Any part builds one with [`Error::project`].
/// An internal error is a panic, with the part that found it and its message.
/// Only execution builds one with [`Error::internal`], and clippy refuses a
/// call of it outside execution. [`catch`] builds one for a panic that crosses
/// it.
#[derive(Debug)]
pub struct Error(Reason);

#[derive(Debug)]
enum Reason {
    Project(String),
    Internal(panic::Caught),
}

/// Whether an error is a project error or an internal error, and the part
/// that found an internal error.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ErrorKind<'a> {
    /// An error in the tester's code.
    Project,
    /// An error in purba itself, found by `part`.
    Internal { part: &'a str },
}

impl Error {
    /// A project error of `text`.
    #[must_use]
    pub const fn project(text: String) -> Self {
        Self(Reason::Project(text))
    }

    /// The internal error of `caught`. Only execution and [`catch`] build one.
    /// Clippy refuses a call of this function outside execution.
    #[must_use]
    pub const fn internal(caught: panic::Caught) -> Self {
        Self(Reason::Internal(caught))
    }

    /// Whether this is a project error or an internal error.
    #[must_use]
    pub fn kind(&self) -> ErrorKind<'_> {
        match &self.0 {
            Reason::Project(_) => ErrorKind::Project,
            Reason::Internal(caught) => ErrorKind::Internal { part: &caught.part },
        }
    }

    /// The line of this error: `project error: <text>`, or
    /// `internal error: <part>: <message>`. It keeps each break of the text.
    #[must_use]
    pub fn line(&self) -> String {
        self.line_with(str::to_owned)
    }

    /// The line of this error, with its text or its message as `text` gives
    /// it.
    #[must_use]
    pub fn line_with(&self, text: impl FnOnce(&str) -> String) -> String {
        match &self.0 {
            Reason::Project(own) => format!("project error: {}", text(own)),
            Reason::Internal(caught) => {
                format!("internal error: {}: {}", caught.part, text(&caught.message))
            }
        }
    }

    fn caught(payload: &(dyn Any + Send), catcher: &str) -> Self {
        Self(Reason::Internal(panic::caught(payload, catcher)))
    }
}

/// The text of a project error, or the message of an internal error.
impl fmt::Display for Error {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.0 {
            Reason::Project(text) => formatter.write_str(text),
            Reason::Internal(caught) => formatter.write_str(&caught.message),
        }
    }
}

impl std::error::Error for Error {}

/// Runs `work`, and gives an internal error for a panic that crosses it. The
/// part comes from the note of the panic hook when the note holds the
/// panic's message, and is `catcher` otherwise.
///
/// # Errors
///
/// The error of `work`, or the internal error of its panic.
pub fn catch<R>(catcher: &str, work: impl FnOnce() -> Result<R, Error>) -> Result<R, Error> {
    std::panic::catch_unwind(AssertUnwindSafe(work))
        .unwrap_or_else(|payload| Err(Error::caught(payload.as_ref(), catcher)))
}

/// A violation of a strict check that a suppression turned into a warning: the
/// name of the check, its words, and the reason of the suppression.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Warning {
    pub check: String,
    pub words: String,
    pub reason: String,
}

/// What the suppressions of a case gave: each warning, and whether execution
/// read the paths of the case, and so judged each suppression of its checks.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Suppressed {
    pub warnings: Vec<Warning>,
    pub judged: bool,
}

/// How many cases of a run passed, failed, raised, are `interrupted`, are
/// blocked or unreached, and how many errors of each kind it met.
///
/// Each case counts once, and each error once, by its kind. It also counts each
/// verdict that a suppression turned into a warning.
#[derive(Debug, Default, PartialEq, Eq)]
pub struct Tally {
    pub passed: usize,
    pub failed: usize,
    pub raised: usize,
    pub interrupted: usize,
    pub blocked: usize,
    /// Cases that the run never reached, because an interrupt stopped the run.
    pub unreached: usize,
    pub project_errors: usize,
    pub internal_errors: usize,
    /// Writes that stdout refused. The run stops at the first.
    pub invocation_errors: usize,
    /// Verdicts that a suppression turned into a warning: one for each check
    /// of each test.
    pub suppressed: usize,
}

impl Tally {
    /// Counts one test by its kind. An error in place of an outcome blocked
    /// that test, so it counts one blocked test and one error of its kind.
    pub fn add(&mut self, result: Result<&Kind, &Error>) {
        match result {
            Ok(Kind::Passed) => self.passed += 1,
            Ok(Kind::Failed { .. }) => self.failed += 1,
            Ok(Kind::Raised { .. }) => self.raised += 1,
            Ok(Kind::Interrupted) => self.interrupted += 1,
            Err(error) => {
                self.blocked += 1;
                self.add_error(error);
            }
        }
    }

    /// How many cases the run has: each case that passed, failed, raised, is
    /// `interrupted`, is blocked or is unreached. An error and a suppression
    /// are no case.
    #[must_use]
    pub const fn cases(&self) -> usize {
        self.passed + self.failed + self.raised + self.interrupted + self.blocked + self.unreached
    }

    /// Counts one test that the error of its module blocked.
    pub const fn add_blocked(&mut self) {
        self.blocked += 1;
    }

    /// Counts one verdict that a suppression turned into a warning.
    pub const fn add_suppressed(&mut self) {
        self.suppressed += 1;
    }

    /// Counts one test that the run never reached.
    pub const fn add_unreached(&mut self) {
        self.unreached += 1;
    }

    /// Counts one write that stdout refused.
    pub const fn add_invocation_error(&mut self) {
        self.invocation_errors += 1;
    }

    /// Counts one error by its kind, and no test. The error of a module counts
    /// through it, and each test that the error blocked through
    /// [`Tally::add_blocked`]. The error of a thread counts through it alone.
    pub fn add_error(&mut self, error: &Error) {
        match error.kind() {
            ErrorKind::Project => self.project_errors += 1,
            ErrorKind::Internal { .. } => self.internal_errors += 1,
        }
    }
}
