//! The load: the rewritten module runs in Python, and its hooks record what
//! each case did.

use purba_errors::Shown;
use purba_parsing::Reads;
use pyo3::basic::CompareOp;
use pyo3::exceptions::{PyImportError, PyIndexError, PyNameError};
use pyo3::types::{
    PyAnyMethods, PyDict, PyDictMethods, PyModule, PyModuleMethods, PyTuple, PyTupleMethods,
    PyTypeMethods,
};
use pyo3::{Bound, IntoPyObject, Py, PyAny, PyErr, PyResult, Python, pyclass, pymethods};
use ruff_python_ast::CmpOp;
use std::collections::HashMap;
use std::io;
use std::panic::{self, AssertUnwindSafe};
use std::path::{Path, PathBuf};
use std::sync::{Mutex, MutexGuard, PoisonError};

use crate::rewrite::{Body, Check, Expectation, Items, Mark, Rewrite, Tested, Try, reading_side};
use crate::{MISSED, RAN, READ, READS, STEP, WITH};

/// The panic of the fault `assert`, in a build with the feature `fault`.
fn fault() {
    crate::fault("assert");
}

/// Whether `left` and `right` hold by `op`, as Python's own operator decides.
fn holds(op: CmpOp, left: &Bound<'_, PyAny>, right: &Bound<'_, PyAny>) -> PyResult<bool> {
    let by = |op| left.rich_compare(right, op)?.is_truthy();
    match op {
        CmpOp::Eq => by(CompareOp::Eq),
        CmpOp::NotEq => by(CompareOp::Ne),
        CmpOp::Lt => by(CompareOp::Lt),
        CmpOp::LtE => by(CompareOp::Le),
        CmpOp::Gt => by(CompareOp::Gt),
        CmpOp::GtE => by(CompareOp::Ge),
        CmpOp::In => right.contains(left),
        CmpOp::NotIn => right.contains(left).map(|contains| !contains),
        CmpOp::Is => Ok(left.is(right)),
        CmpOp::IsNot => Ok(!left.is(right)),
    }
}

/// Whether `value`, and `other` when the expectation hands one, hold by
/// `check`, as Python decides. With no check, `value` holds when it is true.
/// It is `None` when the values do not match the check. Then no check or `not`
/// gets two values, a check of two values gets one, or the check is of an
/// `all()` or an `any()`.
fn held(
    check: Option<&Check>,
    value: &Bound<'_, PyAny>,
    other: Option<&Bound<'_, PyAny>>,
) -> PyResult<Option<bool>> {
    let read = |method: &str, argument: &Bound<'_, PyAny>| {
        value.call_method1(method, (argument,))?.is_truthy()
    };
    match (check, other) {
        (None, None) => value.is_truthy(),
        (Some(Check::Compare { op, .. }), Some(right)) => holds(*op, value, right),
        (
            Some(Check::Read {
                read: Reads::Not, ..
            }),
            None,
        ) => value.is_truthy().map(|truthy| !truthy),
        (
            Some(Check::Read {
                read: Reads::Instance,
                ..
            }),
            Some(kind),
        ) => value.is_instance(kind),
        (
            Some(Check::Read {
                read: Reads::Start, ..
            }),
            Some(argument),
        ) => read("startswith", argument),
        (
            Some(Check::Read {
                read: Reads::End, ..
            }),
            Some(argument),
        ) => read("endswith", argument),
        _ => return Ok(None),
    }
    .map(Some)
}

/// Whether `value` is an iterator that one reading empties: its type has
/// `__next__`, `iter(value)` is `value`, and it is no `file`, an `io.IOBase`.
fn is_iterator(value: &Bound<'_, PyAny>, file: &Bound<'_, PyAny>) -> bool {
    value.get_type().hasattr("__next__").unwrap_or(false)
        && !value.is_instance(file).unwrap_or(true)
        && value
            .try_iter()
            .is_ok_and(|iterator| iterator.as_any().is(value))
}

/// The `repr` of `value`, or the words of its raise.
fn text_of(value: &Bound<'_, PyAny>) -> Result<String, String> {
    value
        .repr()
        .map(|text| text.to_string())
        .map_err(|err| format!("(repr raised {err})"))
}

/// The name of the class of `value`.
fn class_of(value: &Bound<'_, PyAny>) -> String {
    value
        .get_type()
        .name()
        .map_or_default(|name| name.to_string())
}

/// What an expectation that checks `value`, and `other` when it hands one, by
/// `check` shows.
fn shown(
    check: &Check,
    value: &Bound<'_, PyAny>,
    other: Option<&Bound<'_, PyAny>>,
) -> Option<Shown> {
    let (op, expected_left, right) = match (check, other) {
        (Check::Compare { op, expected_left }, Some(right)) => (*op, *expected_left, right),
        (Check::Read { read, kind }, other) => {
            let argument = || {
                other.map_or_else(String::new, |other| {
                    text_of(other).unwrap_or_else(|text| text)
                })
            };
            let reading = text_of(value).unwrap_or_else(|text| text);
            let (expected, reading) = match read {
                Reads::Not => ("empty or false".to_owned(), reading),
                Reads::Instance => (
                    format!("an instance of {kind}"),
                    format!("{reading} ({})", class_of(value)),
                ),
                Reads::Start => (format!("to start with {}", argument()), reading),
                Reads::End => (format!("to end with {}", argument()), reading),
            };
            return Some(Shown::Values { expected, reading });
        }
        (Check::Compare { .. }, None) | (Check::Each(_), _) => return None,
    };
    let (expected, reading) = if expected_left {
        (value, right)
    } else {
        (right, value)
    };
    let (mut expected_text, mut reading_text) = (text_of(expected), text_of(reading));
    if let (Ok(expected_text), Ok(reading_text)) = (&mut expected_text, &mut reading_text)
        && expected_text == reading_text
    {
        for (text, value) in [(expected_text, expected), (reading_text, reading)] {
            text.push_str(" (");
            text.push_str(&class_of(value));
            text.push(')');
        }
    }
    Some(Shown::Values {
        expected: rule(op, expected_left, expected_text.unwrap_or_else(|text| text)),
        reading: reading_text.unwrap_or_else(|text| text),
    })
}

/// One item of an `all()` or an `any()`: its index, whether it holds the
/// check, the item, and the value of the side that the check reads.
type Row<'py> = (usize, bool, Bound<'py, PyAny>, Bound<'py, PyAny>);

/// Whether the `all()` or the `any()` of `items` holds over `reading`, the row
/// of each item that a failure names, and how many items it read. `make` gives
/// each item of `reading` with the parts of its check. An `any()` stops at its
/// first item that holds, and an `all()` at its first item that breaks, as
/// Python's own do. An `any()` of no item fails, and an `all()` of no item
/// holds. It takes no `repr`, so a reader ends with its verdict before any.
/// It gives `None` when the parts of an item do not match the check of an item.
fn read_each<'py>(
    items: &Items,
    reading: &Bound<'py, PyAny>,
    make: &Bound<'py, PyAny>,
) -> PyResult<Option<(bool, Vec<Row<'py>>, usize)>> {
    let row_of = |index: usize, row: Bound<'py, PyAny>| -> PyResult<Option<Row<'py>>> {
        let (first, second) = match row.len()? {
            2 => (row.get_item(1)?, None),
            3 => (row.get_item(1)?, Some(row.get_item(2)?)),
            _ => return Ok(None),
        };
        let Some(holds) = held(Some(&items.item), &first, second.as_ref())? else {
            return Ok(None);
        };
        let value = reading_side(&items.item, first, second);
        Ok(Some((index, holds, row.get_item(0)?, value)))
    };
    let mut rows: Vec<Row<'py>> = Vec::new();
    let mut read = 0;
    for (index, row) in make.call1((reading,))?.try_iter()?.enumerate() {
        let Some(row) = row_of(index, row?)? else {
            return Ok(None);
        };
        read += 1;
        let holds = row.1;
        if items.any || !holds {
            rows.push(row);
        }
        if items.any == holds {
            break;
        }
    }
    // Each reading stops at the row that decides it, so the last row gives the
    // verdict, and a reading of no row gives Python's verdict of no item.
    let held = rows.last().map_or(!items.any, |row| row.1);
    Ok(Some((held, rows, read)))
}

/// What a failed `all()` or `any()` of `items` shows from its `rows`, after it
/// read `total` items: a heading, then a line for the first item that breaks
/// the check of an `all()`, or for each item of an `any()`.
fn shown_items(items: &Items, rows: &[Row<'_>], total: usize) -> Shown {
    let heading = if items.any {
        let noun = if total == 1 { "item" } else { "items" };
        format!("none of {total} {noun} holds  {}", items.label)
    } else {
        format!("the first item that breaks  {}", items.label)
    };
    let lines = rows
        .iter()
        .map(|(index, _, item, value)| {
            let item = text_of(item).unwrap_or_else(|text| text);
            if items.bare {
                format!("[{index}]  {item}")
            } else {
                format!(
                    "[{index}]  {}  {}     {}  {item}",
                    items.side,
                    text_of(value).unwrap_or_else(|text| text),
                    items.target
                )
            }
        })
        .collect();
    Shown::Items {
        heading,
        items: lines,
    }
}

/// The text of `expected` under the rule of `op`, read from the side of the
/// reading, where `expected_left` says whether the expected value is the left
/// side: `== 2` shows `2`, `< 2` shows `< 2`, `2 < x` shows `> 2`, and
/// `'z' in x` shows `to contain 'z'`.
fn rule(op: CmpOp, expected_left: bool, expected: String) -> String {
    let words = match (op, expected_left) {
        (CmpOp::Eq, _) => return expected,
        (CmpOp::NotEq, _) => "!=",
        (CmpOp::Lt, false) | (CmpOp::Gt, true) => "<",
        (CmpOp::LtE, false) | (CmpOp::GtE, true) => "<=",
        (CmpOp::Gt, false) | (CmpOp::Lt, true) => ">",
        (CmpOp::GtE, false) | (CmpOp::LtE, true) => ">=",
        (CmpOp::In, false) => "in",
        (CmpOp::NotIn, false) => "not in",
        (CmpOp::In, true) => "to contain",
        (CmpOp::NotIn, true) => "not to contain",
        (CmpOp::Is, _) => "is",
        (CmpOp::IsNot, _) => "is not",
    };
    format!("{words} {expected}")
}

/// Where the assert at `line` runs. It is `line N` when the innermost frame
/// is the test's own frame. Else it is `line T reaches line N of F, which`.
/// `T` is the line that the test runs, and `F` names the function of the
/// innermost frame. The test is the outermost frame whose code comes from
/// `file`, the module's file.
fn place(py: Python<'_>, line: usize, file: &Bound<'_, PyAny>) -> PyResult<String> {
    Ok(match reach(py, file)? {
        None => format!("line {line}"),
        Some((reaches, name)) => format!("line {reaches} reaches line {line} of {name}, which"),
    })
}

/// The words of an assert at `line` that ran in the act, where `file` is the
/// module's file, at the place that [`place`] names.
fn in_the_act(py: Python<'_>, line: usize, file: &Bound<'_, PyAny>) -> PyResult<String> {
    const RULE: &str = "one test checks one state; check this state in a test of its own";
    Ok(match reach(py, file)? {
        None => format!("line {line}: an assert ran in the act; {RULE}"),
        Some((reaches, name)) => format!(
            "line {reaches} reaches line {line} of {name}, which ran an assert in the act; {RULE}"
        ),
    })
}

/// The line that the test runs and the function of the innermost frame, or
/// `None` when the innermost frame is the test's own. The test is the
/// outermost frame whose code comes from `file`, the module's file.
fn reach(py: Python<'_>, file: &Bound<'_, PyAny>) -> PyResult<Option<(usize, String)>> {
    let innermost = py.import("sys")?.getattr("_getframe")?.call0()?;
    let mut test = None;
    let mut frame = innermost.clone();
    while !frame.is_none() {
        if frame.getattr("f_code")?.getattr("co_filename")?.eq(file)? {
            test = Some(frame.clone());
        }
        frame = frame.getattr("f_back")?;
    }
    let Some(test) = test.filter(|test| !test.is(&innermost)) else {
        return Ok(None);
    };
    let reaches: usize = test.getattr("f_lineno")?.extract()?;
    let name: String = innermost.getattr("f_code")?.getattr("co_name")?.extract()?;
    Ok(Some((reaches, name)))
}

/// Whether a frame of the call that runs now, from the innermost out, runs an
/// `__exit__`: the cleanup of a `with`.
fn in_cleanup(py: Python<'_>) -> PyResult<bool> {
    let mut frame = py.import("sys")?.getattr("_getframe")?.call0()?;
    while !frame.is_none() {
        if frame
            .getattr("f_code")?
            .getattr("co_name")?
            .eq("__exit__")?
        {
            return Ok(true);
        }
        frame = frame.getattr("f_back")?;
    }
    Ok(false)
}

/// The text of each marked assert of one loaded module, and the index of the
/// last one that did not hold, or else of the last one that ran, beside
/// whether it held. It also holds the line of each, and the hook of the
/// expectations, which says whether a case runs before its first expectation
/// or inside a reading, and the file of the module.
#[pyclass(frozen)]
struct Ran {
    marks: Vec<Mark>,
    read: Py<Read>,
    file: Py<PyAny>,
    kept: Mutex<Option<(usize, bool)>>,
}

#[pymethods]
impl Ran {
    /// Tests `value` once, and records `index` and whether `value` held, unless
    /// `value` held and an assert that did not hold is on record. Gives back
    /// whether `value` held, so that the assert tests a `bool`.
    ///
    /// While a case runs before its first expectation, it also records the
    /// assert as a fault of the form, unless it runs in the cleanup of a
    /// `with`. Inside a reading, it records the assert as a fault of the form,
    /// and a reader or a replay ends with `Verdict::Refused`.
    ///
    /// # Errors
    ///
    /// `IndexError` when no assert has `index`, or the error that testing
    /// `value` raised. Either records no index.
    ///
    /// # Panics
    ///
    /// In a build with the feature `fault`, when `PURBA_FAULT` is `assert`.
    fn __call__(&self, index: usize, value: &Bound<'_, PyAny>) -> PyResult<bool> {
        fault();
        let Some(mark) = self.marks.get(index) else {
            return Err(PyIndexError::new_err(format!(
                "{RAN}({index}): the module has {} marked asserts",
                self.marks.len()
            )));
        };
        let py = value.py();
        let read = self.read.get();
        let line = mark.line;
        if read.inside_a_reading() {
            if let Some(readers) = read.reader() {
                readers.end(Verdict::Refused);
            }
            read.fault(
                "assert-in-reading",
                format!(
                    "{} asserts inside a reading, and only an expectation asserts",
                    place(py, line, self.file.bind(py))?
                ),
            );
        } else if read.before_the_first_expectation() && !in_cleanup(py)? {
            read.fault(
                "assert-before-expectation",
                in_the_act(py, line, self.file.bind(py))?,
            );
        }
        let held = value.is_truthy()?;
        let mut kept = self.kept.lock().unwrap_or_else(PoisonError::into_inner);
        if !held || !matches!(*kept, Some((_, false))) {
            *kept = Some((index, held));
        }
        drop(kept);
        Ok(held)
    }
}

impl Ran {
    fn take(&self) -> Option<(usize, bool)> {
        self.kept
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .take()
    }
}

/// What a reading of one expectation gave: it held, it did not hold, it
/// raised, or an assert ran inside it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Verdict {
    Holds,
    Fails,
    Raises,
    Refused,
}

/// What the expectations and the steps of one case need from the process that
/// runs it.
pub trait Readers: Sync {
    /// Whether a Python thread other than the main one is alive.
    fn threaded(&self, py: Python<'_>) -> bool;

    /// Forks a reader. Gives `None` in the reader, and the reader's pid in the
    /// process that forked it.
    ///
    /// # Errors
    ///
    /// The operating system refused the fork.
    fn fork(&self, py: Python<'_>) -> io::Result<Option<i32>>;

    /// Waits for the reader `pid` to end, and gives its verdict.
    fn wait(&self, pid: i32) -> Verdict;

    /// Ends a reader, with `verdict`.
    fn end(&self, verdict: Verdict) -> !;

    /// Names what the case replaced: a hook of the process, which it puts back,
    /// the warning filters, or the events of the watch of the paths. Gives
    /// `None` when the case replaced none.
    fn replaced(&self, _py: Python<'_>) -> Option<Replaced> {
        None
    }

    /// Hears that the case reached its first expectation.
    fn expecting(&self, _py: Python<'_>) {}
}

/// What a case or a module replaced that a test keeps: the first hook of the
/// process that it replaced, by its name, the warning filters, or the events
/// that the watch of the paths set.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Replaced {
    Hook(&'static str),
    Filters,
    Events,
}

impl Replaced {
    /// The name of what was replaced.
    #[must_use]
    pub const fn what(self) -> &'static str {
        match self {
            Self::Hook(name) => name,
            Self::Filters => "the warning filters",
            Self::Events => "the events of sys.monitoring that purba watches with",
        }
    }

    /// The rule that a replacement breaks.
    #[must_use]
    pub const fn rule(self) -> &'static str {
        match self {
            Self::Hook(_) | Self::Events => "a test keeps purba's hooks",
            Self::Filters => "a test keeps the filters that the run began with",
        }
    }
}

/// What the expectations of one case gave.
///
/// The verdict of each that the case read in order, what each that did not hold
/// shows, the verdict of each that a reader read alone, why no reader was forked
/// when the fork failed, and each later expectation that a replay must read
/// alone, since a thread was alive at the first expectation.
#[derive(Debug, Default, PartialEq, Eq)]
pub struct Readings {
    pub in_order: Vec<(usize, bool)>,
    pub shown: Vec<(usize, Shown)>,
    pub alone: Vec<(usize, Verdict)>,
    pub refused: Option<String>,
    pub replay: Vec<usize>,
}

#[derive(Default)]
struct State {
    readers: Option<&'static dyn Readers>,
    reader: Option<usize>,
    started: bool,
    reading: bool,
    readings: Readings,
    form: Vec<(&'static str, String)>,
    raised: Vec<(Py<PyAny>, String)>,
    swallowed: Vec<Swallowed>,
    missed: Vec<String>,
}

/// A `with` that swallowed a raise, and the lines that the raise skipped.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Swallowed {
    /// The line of the `with`.
    pub with: usize,
    /// The first line of its body after the line that raised.
    pub skipped: usize,
    /// The last line of its body.
    pub last: usize,
}

/// The hook of the expectations of one loaded module. It holds each
/// expectation, and the body of each `with` of a test, under the line of the
/// `with`.
#[pyclass(frozen)]
struct Read {
    expectations: Vec<Expectation>,
    bodies: HashMap<usize, Body>,
    missed: Vec<String>,
    state: Mutex<State>,
}

impl Read {
    fn state(&self) -> MutexGuard<'_, State> {
        self.state.lock().unwrap_or_else(PoisonError::into_inner)
    }

    fn known(&self, index: usize) -> PyResult<&Expectation> {
        self.expectations.get(index).ok_or_else(|| {
            PyIndexError::new_err(format!(
                "{READ}({index}): the module has {} expectations",
                self.expectations.len()
            ))
        })
    }

    /// Whether a case runs, and has not reached its first expectation.
    fn before_the_first_expectation(&self) -> bool {
        let state = self.state();
        state.readers.is_some() && !state.started
    }

    /// Whether a case runs inside a reading.
    fn inside_a_reading(&self) -> bool {
        let state = self.state();
        state.readers.is_some() && state.reading
    }

    /// The readers, when this process is a reader or a replay.
    fn reader(&self) -> Option<&'static dyn Readers> {
        let state = self.state();
        state.reader.and(state.readers)
    }

    /// Closes the reading of the expectation at `index`, whose call handed
    /// values that do not match its check. A reader ends with `Raises`. Any
    /// other process panics, since the rewrite writes no such call.
    fn mismatched(&self, index: usize) -> ! {
        self.state().reading = false;
        if let Some(readers) = self.reader() {
            readers.end(Verdict::Raises);
        }
        panic!("{READ}({index}): the values do not match the check of the expectation");
    }

    /// Records `text` as a fault of the form, by the name of its `check`, unless
    /// a fault of that check is on record.
    fn fault(&self, check: &'static str, text: String) {
        let mut state = self.state();
        if !state.form.iter().any(|(named, _)| *named == check) {
            state.form.push((check, text));
        }
    }

    /// Records a fault of the form when a name that the expectation at `index`
    /// reads holds an iterator in the frame of the test, while a case runs. A
    /// parameter of the test is no such name. The fault names the first one.
    ///
    /// # Errors
    ///
    /// The error of reading the frame of the test.
    fn iterators(&self, py: Python<'_>, index: usize) -> PyResult<()> {
        let (names, line) = match self.expectations.get(index) {
            Some(expectation)
                if !expectation.names.is_empty() && self.state().readers.is_some() =>
            {
                (&expectation.names, expectation.line)
            }
            _ => return Ok(()),
        };
        let locals = py
            .import("sys")?
            .getattr("_getframe")?
            .call0()?
            .getattr("f_locals")?;
        let file = py.import("io")?.getattr("IOBase")?;
        if let Some(name) = names.iter().find(|name| {
            locals
                .get_item(name.as_str())
                .is_ok_and(|value| is_iterator(&value, &file))
        }) {
            self.fault(
                "iterator-read",
                format!(
                    "line {line} reads {name}, an iterator that one reading empties; \
                     make it a list in the act"
                ),
            );
        }
        Ok(())
    }

    /// Forks the readers of a case, and gives whether this process reads the
    /// expectation at `index`. The first call of a case forks a reader for each
    /// later expectation of its test, one at a time, and waits for each. When a
    /// Python thread other than the main one is alive, it forks none, and
    /// records each later expectation for a replay. A reader reads only its own
    /// expectation.
    ///
    /// # Errors
    ///
    /// `IndexError` when no expectation has `index`.
    fn forks(&self, py: Python<'_>, index: usize) -> PyResult<bool> {
        let test = self.known(index)?.test.clone();
        let mut state = self.state();
        if let Some(reader) = state.reader {
            return Ok(index == reader);
        }
        if state.started {
            return Ok(true);
        }
        state.started = true;
        let Some(readers) = state.readers else {
            return Ok(true);
        };
        // A child of the fork inherits each lock, so none is held across it.
        drop(state);
        readers.expecting(py);
        if readers.threaded(py) {
            self.state().readings.replay = (index + 1..test.end).collect();
            return Ok(true);
        }
        for later in index + 1..test.end {
            match readers.fork(py) {
                Ok(None) => {
                    self.state().reader = Some(later);
                    return Ok(false);
                }
                Ok(Some(pid)) => {
                    let verdict = readers.wait(pid);
                    self.state().readings.alone.push((later, verdict));
                }
                Err(error) => {
                    self.state().readings.refused = Some(format!(
                        "purba cannot fork to read an expectation alone: {error}"
                    ));
                    return Ok(true);
                }
            }
        }
        Ok(true)
    }
}

#[pymethods]
impl Read {
    /// Whether this process reads the expectation at `index`. When it does, a
    /// reading opens.
    ///
    /// # Errors
    ///
    /// `IndexError` when no expectation has `index`.
    fn reads(&self, py: Python<'_>, index: usize) -> PyResult<bool> {
        let reads = self.forks(py, index)?;
        if reads {
            self.iterators(py, index)?;
        }
        self.state().reading = reads;
        Ok(reads)
    }

    /// Records whether `value` held as the reading of the expectation at
    /// `index`, and closes the reading. A reader ends here, with its verdict.
    /// With one more value in `rest`, the expectation compares `value` and that
    /// value by its own operator, and when they do not hold, it records what it
    /// shows. A side can be `None`, so the right side comes in `rest`, where it
    /// cannot read as a missing argument.
    ///
    /// # Errors
    ///
    /// `IndexError` when no expectation has `index`, or the error that the check
    /// of `value` raised.
    ///
    /// # Panics
    ///
    /// It panics when the values do not match the check of the expectation. A
    /// reader ends with `Raises` instead. It also panics in a build with the
    /// feature `fault`, when `PURBA_FAULT` is `assert`.
    #[pyo3(signature = (index, value, *rest))]
    fn __call__(
        &self,
        index: usize,
        value: &Bound<'_, PyAny>,
        rest: &Bound<'_, PyTuple>,
    ) -> PyResult<()> {
        let check = self.known(index)?.check.clone();
        let other = match rest.len() {
            0 => None,
            1 => Some(rest.get_item(0)?),
            _ => self.mismatched(index),
        };
        let (held, rows, read) = match (&check, &other) {
            (Some(Check::Each(items)), Some(make)) => match read_each(items, value, make) {
                Ok(Some((held, rows, read))) => {
                    if read == 0 && !items.any {
                        self.fault("no-item", items.empty.clone());
                    }
                    (Ok(Some(held)), rows, read)
                }
                Ok(None) => (Ok(None), Vec::new(), 0),
                Err(err) => (Err(err), Vec::new(), 0),
            },
            _ => (held(check.as_ref(), value, other.as_ref()), Vec::new(), 0),
        };
        let (reader, readers) = {
            let mut state = self.state();
            state.reading = false;
            (state.reader, state.readers)
        };
        if let (Some(_), Some(readers)) = (reader, readers) {
            readers.end(match held {
                Ok(Some(true)) => Verdict::Holds,
                Ok(Some(false)) => Verdict::Fails,
                Ok(None) | Err(_) => Verdict::Raises,
            });
        }
        fault();
        let Some(held) = held? else {
            self.mismatched(index)
        };
        let shown = |check: &Check| match check {
            Check::Each(items) => Some(shown_items(items, &rows, read)),
            check => shown(check, value, other.as_ref()),
        };
        if !held && let Some(shown) = check.as_ref().and_then(shown) {
            self.state().readings.shown.push((index, shown));
        }
        self.state().readings.in_order.push((index, held));
        Ok(())
    }

    /// Records the `try` at `index` as missed: its one statement did not raise.
    ///
    /// # Errors
    ///
    /// `IndexError` when no `try` of one path has `index`.
    fn missed(&self, index: usize) -> PyResult<()> {
        let words = self.missed.get(index).ok_or_else(|| {
            PyIndexError::new_err(format!(
                "{MISSED}({index}): the module has {} tries of one path",
                self.missed.len()
            ))
        })?;
        self.state().missed.push(words.clone());
        Ok(())
    }

    /// Runs after each statement of a test before its first expectation, at
    /// `line`. Records what the case replaced as the fault of the form, and the
    /// process puts back a hook that it replaced.
    fn step(&self, py: Python<'_>, line: usize) {
        let readers = {
            let state = self.state();
            if state.started {
                return;
            }
            state.readers
        };
        if let Some(replaced) = readers.and_then(|readers| readers.replaced(py)) {
            self.fault(
                "hook-replaced",
                format!(
                    "line {line} replaced {}, and {}",
                    replaced.what(),
                    replaced.rule()
                ),
            );
        }
    }

    /// Wraps `manager`, an item of the `with` at `line`, so that the case sees
    /// its entry and its cleanup. A `manager` whose type lacks `__enter__` or
    /// `__exit__` stays as it is, so that Python refuses it in its own words.
    fn wrap(slf: &Bound<'_, Self>, line: usize, manager: Bound<'_, PyAny>) -> PyResult<Py<PyAny>> {
        let py = slf.py();
        let kind = manager.get_type();
        let (Ok(enter), Ok(exit)) = (kind.getattr("__enter__"), kind.getattr("__exit__")) else {
            return Ok(manager.unbind());
        };
        Ok(Bound::new(
            py,
            With {
                manager: manager.unbind(),
                enter: enter.unbind(),
                exit: exit.unbind(),
                line,
                read: slf.clone().unbind(),
            },
        )?
        .into_any()
        .unbind())
    }
}

/// An item of a `with` of a test, which enters and leaves its manager. It keeps
/// a raise of the entry or of the cleanup, with the line of the `with`, and a
/// raise that the cleanup swallowed.
#[pyclass(frozen)]
struct With {
    manager: Py<PyAny>,
    enter: Py<PyAny>,
    exit: Py<PyAny>,
    line: usize,
    read: Py<Read>,
}

impl With {
    /// Keeps `err`, which the `what` of the `with` raised, and gives it back.
    fn keep(&self, py: Python<'_>, err: PyErr, what: &str) -> PyErr {
        let text = format!("the {what} of the with at line {} raised {err}", self.line);
        let value = err.value(py).clone().into_any().unbind();
        self.read.get().state().raised.push((value, text));
        err
    }
}

#[pymethods]
impl With {
    fn __enter__(&self, py: Python<'_>) -> PyResult<Py<PyAny>> {
        self.enter
            .call1(py, (&self.manager,))
            .map_err(|err| self.keep(py, err, "entry"))
    }

    fn __exit__(
        &self,
        py: Python<'_>,
        kind: &Bound<'_, PyAny>,
        value: &Bound<'_, PyAny>,
        traceback: &Bound<'_, PyAny>,
    ) -> PyResult<Py<PyAny>> {
        // A reading that raised ends here, inside the `with` that holds it.
        self.read.get().state().reading = false;
        let result = self
            .exit
            .call1(py, (&self.manager, kind, value, traceback))
            .map_err(|err| self.keep(py, err, "cleanup"))?;
        if !value.is_none() && !traceback.is_none() && result.bind(py).is_truthy()? {
            let raised: usize = traceback.getattr("tb_lineno")?.extract()?;
            let read = self.read.get();
            if let Some(body) = read.bodies.get(&self.line)
                && let Some(&skipped) = body.starts.iter().find(|start| **start > raised)
            {
                read.state().swallowed.push(Swallowed {
                    with: self.line,
                    skipped,
                    last: body.last,
                });
            }
        }
        Ok(result)
    }
}

/// A module that `load` made from a rewritten text.
pub struct Loaded<'py> {
    path: PathBuf,
    module: Bound<'py, PyModule>,
    code: Bound<'py, PyAny>,
    ran: Bound<'py, Ran>,
    read: Bound<'py, Read>,
    tests: Vec<Tested>,
}

/// One test of a loaded module, as [`Loaded::test`] gives it.
pub struct LoadedTest<'l> {
    tested: Option<&'l Tested>,
    expectations: &'l [Expectation],
}

impl LoadedTest<'_> {
    /// The line of its first expectation, when it states one.
    #[must_use]
    pub fn first_expectation(&self) -> Option<usize> {
        let tested = self.tested?;
        self.expectations
            .get(tested.expectations.clone())?
            .first()
            .map(|expectation| expectation.line)
    }

    /// Each `try` of it, in source order.
    #[must_use]
    pub fn tries(&self) -> &[Try] {
        self.tested.map_or(&[], |tested| &tested.tries)
    }

    /// Whether it states an expectation.
    #[must_use]
    pub fn states_expectation(&self) -> bool {
        self.tested
            .is_some_and(|tested| !tested.expectations.is_empty())
    }

    /// The text of each expectation of the module at one of `indexes`, once
    /// each, in the order of the indexes. An index of no expectation gives
    /// nothing.
    #[must_use]
    pub fn texts(&self, indexes: impl IntoIterator<Item = usize>) -> Vec<(usize, String)> {
        let mut indexes: Vec<usize> = indexes.into_iter().collect();
        indexes.sort_unstable();
        indexes.dedup();
        indexes
            .into_iter()
            .filter_map(|index| Some((index, self.expectations.get(index)?.text.clone())))
            .collect()
    }
}

/// What [`Loaded::watch`] saw of one call.
pub struct Watched<'a, T> {
    /// The result of the call.
    pub result: PyResult<T>,
    /// The text of the last marked assert inside the call that did not hold, or
    /// else of the last one that ran, beside whether it held.
    pub mark: Option<(&'a str, bool)>,
    /// What the expectations gave.
    pub readings: Readings,
    /// The first fault of each check of the form, each beside the name of its
    /// check, in the order they came: an assert before the first expectation,
    /// an assert inside a reading, a hook that the case replaced, an `all()`
    /// whose reading holds no item, or a name of an expectation that holds an
    /// iterator.
    pub form: Vec<(&'static str, String)>,
    /// Each exception that the entry or the cleanup of a `with` raised, in the
    /// order they raised, beside the text that names that `with`.
    pub raised: Vec<(Py<PyAny>, String)>,
    /// Each `with` that swallowed a raise and skipped a line.
    pub swallowed: Vec<Swallowed>,
    /// The words of each `try` of one path whose statement did not raise, in
    /// the order they ran.
    pub missed: Vec<String>,
}

impl<'py> Loaded<'py> {
    /// The module, whose namespace stays in Python.
    #[must_use]
    pub const fn module(&self) -> &Bound<'py, PyModule> {
        &self.module
    }

    /// The code object of the module's text, which holds the code of each
    /// function that the module defines.
    #[must_use]
    pub const fn code(&self) -> &Bound<'py, PyAny> {
        &self.code
    }

    /// The test that `name` names. A function that is not a test of the
    /// rewrite gives a test with no expectation and no `try`.
    #[must_use]
    pub fn test(&self, name: &str) -> LoadedTest<'_> {
        LoadedTest {
            tested: self.tests.iter().find(|tested| tested.name == name),
            expectations: &self.read.get().expectations,
        }
    }

    /// Runs `call` with no assert and no reading recorded, with `readers` for
    /// the expectations. Gives its result, the last mark, the readings, and
    /// what the steps, the marks and the `with`s of the case saw.
    ///
    /// When `call` bound a name of a hook to anything but the hook, the result
    /// is the error that [`load`] gives such a module, and the hook is back.
    /// When this process is a reader and `call` ends, the reader ends with the
    /// verdict that its expectation raised.
    pub fn watch<T>(
        &self,
        readers: &'static dyn Readers,
        call: impl FnOnce() -> PyResult<T>,
    ) -> Watched<'_, T> {
        self.observe(readers, None, call)
    }

    /// Runs `call` as a replay that reads only the expectation at `index`, and
    /// ends the process with its verdict through `readers`. When `call` ends
    /// before that expectation, the replay ends with the verdict that it
    /// raised.
    pub fn replay<T>(
        &self,
        readers: &'static dyn Readers,
        index: usize,
        call: impl FnOnce() -> PyResult<T>,
    ) -> Watched<'_, T> {
        self.observe(readers, Some(index), call)
    }

    fn observe<T>(
        &self,
        readers: &'static dyn Readers,
        reader: Option<usize>,
        call: impl FnOnce() -> PyResult<T>,
    ) -> Watched<'_, T> {
        let ran = self.ran.get();
        let read = self.read.get();
        let _ = ran.take();
        *read.state() = State {
            readers: Some(readers),
            reader,
            started: reader.is_some(),
            ..State::default()
        };
        let result = match (
            call(),
            check(&self.module, &self.ran, &self.read, &self.path),
        ) {
            (result, Ok(())) => result,
            (_, Err(err)) => bind(&self.module, &self.ran, &self.read).and(Err(err)),
        };
        let state = std::mem::take(&mut *read.state());
        if state.reader.is_some() {
            readers.end(Verdict::Raises);
        }
        let mark = ran
            .take()
            .and_then(|(index, held)| Some((ran.marks.get(index)?.text.as_str(), held)));
        Watched {
            result,
            mark,
            readings: state.readings,
            form: state.form,
            raised: state.raised,
            swallowed: state.swallowed,
            missed: state.missed,
        }
    }
}

/// Each name of a hook, beside the hook that it holds.
fn hooks<'py>(
    ran: &Bound<'py, Ran>,
    read: &Bound<'py, Read>,
) -> PyResult<[(&'static str, Bound<'py, PyAny>); 6]> {
    Ok([
        (RAN, ran.as_any().clone()),
        (READ, read.as_any().clone()),
        (READS, read.getattr("reads")?),
        (STEP, read.getattr("step")?),
        (WITH, read.getattr("wrap")?),
        (MISSED, read.getattr("missed")?),
    ])
}

/// Gives `module` each hook.
fn bind(
    module: &Bound<'_, PyModule>,
    ran: &Bound<'_, Ran>,
    read: &Bound<'_, Read>,
) -> PyResult<()> {
    for (name, hook) in hooks(ran, read)? {
        module.setattr(name, hook)?;
    }
    Ok(())
}

/// Loads the text of `rewrite` as the module at `path`, in this order:
///
/// 1. names the module after the file of `path`, without `.py`, refuses the
///    name when `sys.modules` holds it, and sets the module's `__file__` to
///    `path`;
/// 2. gives it `_purba_ran`, which holds the text and the line of each assert
///    of `rewrite` at the index that its mark records, and `_purba_read`,
///    `_purba_reads`, `_purba_step`, `_purba_with` and `_purba_missed`, the
///    hooks of its expectations and of its tests;
/// 3. puts it in `sys.modules` under that name;
/// 4. compiles the text, checks that no function holds a name of a hook as a
///    local name, and runs it in the module's namespace;
/// 5. checks that each name of a hook still holds its hook;
/// 6. when step 4 or 5 raises or panics, puts back what `sys.modules` held
///    under that name, or takes the name out when it held nothing.
///
/// # Errors
///
/// `ImportError` when `sys.modules` holds the module's name. The error that
/// step 4 raises, such as one from `compile()` or the module's body.
/// `NameError` when the body bound a name of a hook to anything but the hook.
///
/// # Panics
///
/// A panic of step 4 or 5 goes on after step 6.
pub fn load<'py>(py: Python<'py>, path: &Path, rewrite: &Rewrite) -> PyResult<Loaded<'py>> {
    load_shown(py, path, path, rewrite)
}

/// [`load`], where each error names the module by `shown` in place of `path`.
///
/// # Errors
///
/// Those of [`load`].
///
/// # Panics
///
/// Where [`load`] panics.
pub fn load_shown<'py>(
    py: Python<'py>,
    path: &Path,
    shown: &Path,
    rewrite: &Rewrite,
) -> PyResult<Loaded<'py>> {
    let text = rewrite.text.as_str();
    let name = path.file_stem().unwrap_or_default().to_string_lossy();
    let modules = py.import("sys")?.getattr("modules")?;
    if modules.contains(&*name)? {
        return Err(PyImportError::new_err(format!(
            "sys.modules already holds the name {name}"
        )));
    }
    let module = PyModule::new(py, &name)?;
    module.setattr("__file__", path.as_os_str())?;
    let read = Bound::new(
        py,
        Read {
            expectations: rewrite.expectations.clone(),
            bodies: rewrite.bodies.clone(),
            missed: rewrite.missed.clone(),
            state: Mutex::new(State::default()),
        },
    )?;
    let ran = Bound::new(
        py,
        Ran {
            marks: rewrite.marks.clone(),
            read: read.clone().unbind(),
            file: path.as_os_str().into_pyobject(py)?.into_any().unbind(),
            kept: Mutex::new(None),
        },
    )?;
    bind(&module, &ran, &read)?;
    let before = modules.get_item(&*name).ok();
    modules.set_item(&*name, &module)?;
    let ran_body = panic::catch_unwind(AssertUnwindSafe(|| {
        let code = run(py, (path, shown), text, &module)?;
        check(&module, &ran, &read, shown).map(|()| code)
    }));
    if !matches!(ran_body, Ok(Ok(_))) {
        let _ = before.map_or_else(
            || modules.del_item(&*name),
            |before| modules.set_item(&*name, before),
        );
    }
    let code = match ran_body {
        Ok(Ok(code)) => code,
        Ok(Err(err)) => return Err(err),
        Err(payload) => panic::resume_unwind(payload),
    };
    Ok(Loaded {
        path: path.to_owned(),
        module,
        code,
        ran,
        read,
        tests: rewrite.tests.clone(),
    })
}

/// Compiles `text` as the file at `path`, runs it in the namespace of
/// `module`, and gives its code. An error names the module by `shown`.
fn run<'py>(
    py: Python<'py>,
    (path, shown): (&Path, &Path),
    text: &str,
    module: &Bound<'py, PyModule>,
) -> PyResult<Bound<'py, PyAny>> {
    let builtins = py.import("builtins")?;
    let keywords = PyDict::new(py);
    keywords.set_item("optimize", 0)?;
    let code = builtins
        .getattr("compile")?
        .call((text, path.as_os_str(), "exec"), Some(&keywords))?;
    check_locals(&code, shown)?;
    builtins.getattr("exec")?.call1((&code, module.dict()))?;
    Ok(code)
}

/// Checks that each name of a hook in the namespace of `module` holds its hook.
///
/// # Errors
///
/// `NameError` that names the file at `path` when one does not.
fn check(
    module: &Bound<'_, PyModule>,
    ran: &Bound<'_, Ran>,
    read: &Bound<'_, Read>,
    path: &Path,
) -> PyResult<()> {
    let names = module.dict();
    for (name, hook) in hooks(ran, read)? {
        // A bound method is a new object at each lookup, so it holds when it
        // equals the hook.
        let holds = match names.get_item(name)? {
            Some(held) if name == RAN || name == READ => held.is(&hook),
            Some(held) => held.eq(&hook)?,
            None => false,
        };
        if !holds {
            return Err(bound(path, name));
        }
    }
    Ok(())
}

/// Checks that no code object in `code`, at any depth, holds a name of a hook
/// among the local names of its function. Such a name hides the hook from each
/// assert of that function.
///
/// # Errors
///
/// `NameError` that names the file at `path` when one does.
fn check_locals(code: &Bound<'_, PyAny>, path: &Path) -> PyResult<()> {
    let code_type = code.get_type();
    let mut waiting = vec![code.clone()];
    while let Some(current) = waiting.pop() {
        for names in ["co_varnames", "co_cellvars", "co_freevars"] {
            let names = current.getattr(names)?;
            for hook in [RAN, READ, READS, STEP, WITH, MISSED] {
                if names.contains(hook)? {
                    return Err(bound(path, hook));
                }
            }
        }
        for constant in current.getattr("co_consts")?.try_iter()? {
            let constant = constant?;
            if constant.is_instance(&code_type)? {
                waiting.push(constant);
            }
        }
    }
    Ok(())
}

/// The error of a module at `path` that binds `name`, the name of a hook.
fn bound(path: &Path, name: &str) -> PyErr {
    let hook = match name {
        RAN => "assert hook",
        STEP => "step hook",
        WITH => "with hook",
        MISSED => "try hook",
        _ => "expectation hook",
    };
    PyNameError::new_err(format!(
        "{} binds {name}, the name of purba's {hook}",
        path.display()
    ))
}
