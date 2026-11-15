//! The watch of the paths of a case: which lines of the test module's code run
//! before the first expectation, and which ways each branch there goes. It
//! takes a tool of `sys.monitoring` for the process:
//! <https://docs.python.org/3.12/library/sys.monitoring.html>
//!
//! Code is part of the case when the test starts it, or when code that is part
//! of the case starts it. Code that a library calls back is free, and so is a
//! module that the test imports, since only the test module's code is watched.

use std::collections::{HashMap, HashSet};
use std::sync::{Mutex, MutexGuard, PoisonError};
use std::thread::{self, ThreadId};

use pyo3::sync::PyOnceLock;
use pyo3::types::PyAnyMethods;
use pyo3::{Bound, Py, PyAny, PyResult, Python, pyclass, pymethods};

/// The rule that each fault of a path names.
const RULE: &str = "the lines before the first expectation take one path";

/// The tool of `sys.monitoring` that purba took, its `DISABLE`, and the events
/// that it sets on each code object.
struct Tool {
    id: u8,
    disable: Py<PyAny>,
    events: u32,
}

static TOOL: PyOnceLock<Option<Tool>> = PyOnceLock::new();

/// What one case's watch saw, on the thread that runs the case.
struct Case {
    /// The code object of the test.
    test: usize,
    /// The line of the test's first expectation.
    before: usize,
    /// Whether the watch runs: from the call to the first expectation.
    on: bool,
    /// Each code object that is part of the case, the test's among them.
    codes: HashMap<usize, Py<PyAny>>,
    /// Each line that ran, for each code object.
    lines: HashMap<usize, HashSet<usize>>,
    /// Where each branch went, for each code object and offset.
    ways: HashMap<(usize, usize), HashSet<usize>>,
}

static CASES: Mutex<Option<HashMap<ThreadId, Case>>> = Mutex::new(None);

fn cases() -> MutexGuard<'static, Option<HashMap<ThreadId, Case>>> {
    CASES.lock().unwrap_or_else(PoisonError::into_inner)
}

/// For each code object, the line of each branch and whether it is free.
type Branches = HashMap<usize, (usize, bool)>;

/// The branches of each code object that the watch read, beside the code
/// object. Holding it keeps its address from naming another code object.
static BRANCHES: Mutex<Option<HashMap<usize, Read>>> = Mutex::new(None);

/// A code object, and the branches that the watch read from it.
type Read = (Py<PyAny>, Branches);

fn key(code: &Bound<'_, PyAny>) -> usize {
    code.as_ptr() as usize
}

/// The kind of event that an [`Event`] takes.
#[derive(Clone, Copy)]
enum Kind {
    Start,
    Line,
    Branch,
}

/// The callable that `sys.monitoring` calls for one kind of event.
#[pyclass(frozen)]
struct Event {
    kind: Kind,
}

#[pymethods]
impl Event {
    #[pyo3(signature = (code, at, to = None))]
    fn __call__(
        &self,
        code: &Bound<'_, PyAny>,
        at: usize,
        to: Option<usize>,
    ) -> PyResult<Py<PyAny>> {
        let py = code.py();
        let disable = match self.kind {
            Kind::Start => {
                started(code)?;
                false
            }
            Kind::Line => line(code, at),
            Kind::Branch => branch(code, at, to.unwrap_or(at))?,
        };
        Ok(match (disable, TOOL.get(py)) {
            (true, Some(Some(tool))) => tool.disable.clone_ref(py),
            _ => py.None(),
        })
    }
}

/// Gives `act` the case that runs on this thread, and its result, or `None`
/// when no case runs on it.
fn on_this_thread<R>(act: impl FnOnce(&mut Case) -> R) -> Option<R> {
    let thread = thread::current().id();
    cases()
        .as_mut()
        .and_then(|cases| cases.get_mut(&thread))
        .map(act)
}

/// Makes `code` part of the case when the test or code that is part of the
/// case started it.
fn started(code: &Bound<'_, PyAny>) -> PyResult<()> {
    let id = key(code);
    if on_this_thread(|case| case.on && !case.codes.contains_key(&id)) != Some(true) {
        return Ok(());
    }
    let py = code.py();
    let mut frame = py.import("sys")?.getattr("_getframe")?.call0()?;
    while !frame.is_none() && !frame.getattr("f_code")?.is(code) {
        frame = frame.getattr("f_back")?;
    }
    if frame.is_none() {
        return Ok(());
    }
    let caller = frame.getattr("f_back")?;
    if caller.is_none() {
        return Ok(());
    }
    let caller = key(&caller.getattr("f_code")?);
    let code = code.clone().unbind();
    on_this_thread(|case| {
        if case.codes.contains_key(&caller) {
            case.codes.insert(id, code);
        }
    });
    Ok(())
}

/// Records that `at` ran, when `code` is part of the case.
fn line(code: &Bound<'_, PyAny>, at: usize) -> bool {
    let id = key(code);
    on_this_thread(|case| {
        if !case.on || !case.codes.contains_key(&id) {
            return false;
        }
        case.lines.entry(id).or_default().insert(at);
        true
    })
    .unwrap_or(false)
}

/// Records where the branch at `at` went, when `code` is part of the case and
/// the branch is not free.
fn branch(code: &Bound<'_, PyAny>, at: usize, to: usize) -> PyResult<bool> {
    let id = key(code);
    let Some((test, before)) = on_this_thread(|case| {
        (case.on && case.codes.contains_key(&id)).then_some((case.test, case.before))
    })
    .flatten() else {
        return Ok(false);
    };
    let Some((line, free)) = branches(code)?.get(&at).copied() else {
        return Ok(true);
    };
    if free || (id == test && line >= before) {
        return Ok(true);
    }
    Ok(on_this_thread(|case| {
        let ways = case.ways.entry((id, at)).or_default();
        ways.insert(to);
        ways.len() > 1
    })
    .unwrap_or(false))
}

/// The line of each branch of `code`, and whether it is free: a loop's head,
/// whose body's lines speak for it; the exit of a `with`; the match of an
/// `except`; and the test of an assert, which assertion's mark judges. Each
/// is told by the instruction beside it, as `dis` names it:
/// <https://docs.python.org/3.12/library/dis.html#python-bytecode-instructions>
fn branches(code: &Bound<'_, PyAny>) -> PyResult<Branches> {
    let id = key(code);
    if let Some(known) = BRANCHES
        .lock()
        .unwrap_or_else(PoisonError::into_inner)
        .as_ref()
        .and_then(|known| known.get(&id))
    {
        return Ok(known.1.clone());
    }
    let py = code.py();
    let mut listed: Vec<(usize, String, Option<usize>)> = Vec::new();
    for instruction in py
        .import("dis")?
        .getattr("get_instructions")?
        .call1((code,))?
        .try_iter()?
    {
        let instruction = instruction?;
        listed.push((
            instruction.getattr("offset")?.extract()?,
            instruction.getattr("opname")?.extract()?,
            instruction
                .getattr("positions")?
                .getattr("lineno")?
                .extract()?,
        ));
    }
    let name = |index: Option<usize>| {
        index
            .and_then(|index| listed.get(index))
            .map(|(_, name, _)| name.as_str())
    };
    let mut found = Branches::new();
    for (index, (offset, opname, line)) in listed.iter().enumerate() {
        if opname != "FOR_ITER" && !opname.starts_with("POP_JUMP_IF") {
            continue;
        }
        let mut before = index.checked_sub(1);
        if name(before) == Some("TO_BOOL") {
            before = before.and_then(|before| before.checked_sub(1));
        }
        let mut after = index + 1;
        if name(Some(after)) == Some("NOT_TAKEN") {
            after += 1;
        }
        let free = opname == "FOR_ITER"
            || matches!(
                name(before),
                Some("WITH_EXCEPT_START" | "CHECK_EXC_MATCH" | "CHECK_EG_MATCH")
            )
            || matches!(
                name(Some(after)),
                Some("LOAD_ASSERTION_ERROR" | "LOAD_COMMON_CONSTANT")
            );
        found.insert(*offset, (line.unwrap_or(0), free));
    }
    BRANCHES
        .lock()
        .unwrap_or_else(PoisonError::into_inner)
        .get_or_insert_with(HashMap::new)
        .insert(id, (code.clone().unbind(), found.clone()));
    Ok(found)
}

/// Takes a tool of `sys.monitoring` for the process, once, and sets its
/// callables. Gives `None` when every free tool is taken.
fn tool(py: Python<'_>) -> Option<&Tool> {
    TOOL.get_or_init(py, || {
        let take = || -> PyResult<Option<Tool>> {
            let monitoring = py.import("sys")?.getattr("monitoring")?;
            let events = monitoring.getattr("events")?;
            let mut id = None;
            // 0 to 2 and 5 are named for a debugger, coverage, a profiler and
            // an optimizer; 3 and 4 have no name.
            for free in [3_u8, 4] {
                if monitoring
                    .call_method1("use_tool_id", (free, "purba"))
                    .is_ok()
                {
                    id = Some(free);
                    break;
                }
            }
            let Some(id) = id else {
                return Ok(None);
            };
            let branches: Vec<&str> = if events.hasattr("BRANCH_LEFT")? {
                vec!["BRANCH_LEFT", "BRANCH_RIGHT"]
            } else {
                vec!["BRANCH"]
            };
            let mut mask = 0_u32;
            for (name, kind) in [("PY_START", Kind::Start), ("LINE", Kind::Line)]
                .into_iter()
                .chain(branches.into_iter().map(|name| (name, Kind::Branch)))
            {
                let event = events.getattr(name)?;
                mask |= event.extract::<u32>()?;
                monitoring.call_method1(
                    "register_callback",
                    (id, event, Bound::new(py, Event { kind })?),
                )?;
            }
            Ok(Some(Tool {
                id,
                disable: monitoring.getattr("DISABLE")?.unbind(),
                events: mask,
            }))
        };
        take().ok().flatten()
    })
    .as_ref()
}

/// Sets the events of the watch on each code object inside `code`, the code of
/// a module, at any depth. The module's own code stays unwatched.
pub fn arm(code: &Bound<'_, PyAny>) -> PyResult<()> {
    let py = code.py();
    let Some(tool) = tool(py) else {
        return Ok(());
    };
    let monitoring = py.import("sys")?.getattr("monitoring")?;
    let code_type = code.get_type();
    let mut waiting: Vec<Bound<'_, PyAny>> = Vec::new();
    for constant in code.getattr("co_consts")?.try_iter()? {
        waiting.push(constant?);
    }
    while let Some(current) = waiting.pop() {
        if !current.is_instance(&code_type)? {
            continue;
        }
        monitoring.call_method1("set_local_events", (tool.id, &current, tool.events))?;
        for constant in current.getattr("co_consts")?.try_iter()? {
            waiting.push(constant?);
        }
    }
    Ok(())
}

/// Starts the watch of a case on this thread, whose test runs `test`, and
/// whose first expectation is at the line `before`.
pub fn start(py: Python<'_>, test: &Bound<'_, PyAny>, before: usize) {
    if tool(py).is_none() {
        return;
    }
    // Each case sees each event again, though an earlier case disabled it.
    drop(
        py.import("sys")
            .and_then(|sys| sys.getattr("monitoring")?.call_method0("restart_events")),
    );
    cases().get_or_insert_with(HashMap::new).insert(
        thread::current().id(),
        Case {
            test: key(test),
            before,
            on: true,
            codes: HashMap::from([(key(test), test.clone().unbind())]),
            lines: HashMap::new(),
            ways: HashMap::new(),
        },
    );
}

/// Stops the watch of the case on this thread: it reached its first
/// expectation.
pub fn stop() {
    if let Some(case) = cases()
        .as_mut()
        .and_then(|cases| cases.get_mut(&thread::current().id()))
    {
        case.on = false;
    }
}

/// Ends the watch of the case on this thread, and gives each fault of its
/// paths, beside its line, in the order of the lines.
pub fn finish(py: Python<'_>) -> Vec<(usize, String)> {
    let Some(case) = cases()
        .as_mut()
        .and_then(|cases| cases.remove(&thread::current().id()))
    else {
        return Vec::new();
    };
    let mut faults: Vec<(usize, String)> = Vec::new();
    for (id, code) in &case.codes {
        let code = code.bind(py);
        let Ok(lines) = lines(code) else {
            continue;
        };
        let ran = case.lines.get(id);
        for line in lines {
            if (*id == case.test && line >= case.before)
                || ran.is_some_and(|ran| ran.contains(&line))
            {
                continue;
            }
            faults.push((line, format!("line {line} did not run, and {RULE}")));
        }
        let Ok(branches) = branches(code) else {
            continue;
        };
        for ((owner, at), ways) in &case.ways {
            if owner != id || ways.len() > 1 {
                continue;
            }
            if let Some((line, _)) = branches.get(at) {
                faults.push((*line, format!("line {line} went one way only, and {RULE}")));
            }
        }
    }
    faults.sort();
    faults.dedup();
    faults
}

/// Each line of `code` that holds an instruction, but the line of its `def`.
fn lines(code: &Bound<'_, PyAny>) -> PyResult<Vec<usize>> {
    let first: usize = code.getattr("co_firstlineno")?.extract()?;
    let mut found: Vec<usize> = Vec::new();
    for entry in code.call_method0("co_lines")?.try_iter()? {
        let (_, _, line): (usize, usize, Option<usize>) = entry?.extract()?;
        if let Some(line) = line
            && line != first
        {
            found.push(line);
        }
    }
    found.sort_unstable();
    found.dedup();
    Ok(found)
}
