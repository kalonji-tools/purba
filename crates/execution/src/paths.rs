//! The watch of the paths of a case: which lines of the test module's code run
//! before the first expectation, and which ways each branch there goes. It
//! takes a tool of `sys.monitoring` for the process:
//! <https://docs.python.org/3.12/library/sys.monitoring.html>
//!
//! Code is part of the case when the test starts it, or when code that is part
//! of the case calls it by name: the callable of the last call that the caller
//! made is its function, a bound method of it, or a class whose `__init__` or
//! `__new__` it is.
//! A generator is part of the case when code that is part of the case starts
//! it, since a `for` resumes it with no call. A generator under
//! `contextlib.contextmanager` is part of the case when code that is part of
//! the case enters it, through a `with` or through `ExitStack.enter_context`.
//! The lines after the `yield` where it waits at the first expectation are its
//! cleanup. The watch does not judge them.
//! Other code is free: a function of the test module that C code calls back,
//! such as `sorted` with `key=`, or that a library calls back, and a module that
//! the test imports, since only the test module's code is watched.

use std::collections::{HashMap, HashSet};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Mutex, MutexGuard, PoisonError};
use std::thread::{self, ThreadId};

use pyo3::sync::PyOnceLock;
use pyo3::types::{PyAnyMethods, PyDict, PyTuple, PyType};
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
    /// For each code object but the test's, the line of the test that ran
    /// when the code object first started, and the name of its function.
    reached: HashMap<usize, (usize, String)>,
    /// For each code object of the case, the callable of the last call that it
    /// made.
    calls: HashMap<usize, Py<PyAny>>,
    /// Each generator that a `with` of the case entered.
    managers: HashSet<usize>,
    /// The line of the `yield` where each such generator waits.
    waiting: HashMap<usize, usize>,
    /// The line of the `yield` where each such generator waited at the first
    /// expectation. The lines after it are its cleanup.
    cleanup: HashMap<usize, usize>,
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
    Call,
    Line,
    Branch,
    Yield,
    Resume,
}

/// The callable that `sys.monitoring` calls for one kind of event.
#[pyclass(frozen)]
struct Event {
    kind: Kind,
}

#[pymethods]
impl Event {
    #[pyo3(signature = (code, at, to = None, _argument = None))]
    fn __call__(
        &self,
        code: &Bound<'_, PyAny>,
        at: usize,
        to: Option<&Bound<'_, PyAny>>,
        _argument: Option<&Bound<'_, PyAny>>,
    ) -> PyResult<Py<PyAny>> {
        let py = code.py();
        let disable = match self.kind {
            Kind::Start => {
                started(code)?;
                false
            }
            Kind::Call => to.is_some_and(|callable| !called(code, callable)),
            Kind::Line => line(code, at),
            Kind::Branch => {
                let to = to.map(PyAnyMethods::extract).transpose()?;
                branch(code, at, to.unwrap_or(at))?
            }
            Kind::Yield => {
                yielded(code, at)?;
                false
            }
            Kind::Resume => {
                let id = key(code);
                on_this_thread(|case| case.waiting.remove(&id));
                false
            }
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

/// The code of `__enter__` of a generator under `contextlib.contextmanager`,
/// and of `ExitStack.enter_context`.
static ENTERS: PyOnceLock<(Py<PyAny>, Py<PyAny>)> = PyOnceLock::new();

fn enters(py: Python<'_>) -> PyResult<&(Py<PyAny>, Py<PyAny>)> {
    ENTERS.get_or_try_init(py, || {
        let contextlib = py.import("contextlib")?;
        let code = |class: &str, method: &str| -> PyResult<Py<PyAny>> {
            Ok(contextlib
                .getattr(class)?
                .getattr(method)?
                .getattr("__code__")?
                .unbind())
        };
        Ok((
            code("_GeneratorContextManager", "__enter__")?,
            code("ExitStack", "enter_context")?,
        ))
    })
}

/// Makes `code` part of the case when the test or code that is part of the
/// case started it, or entered it as a generator under `contextmanager`.
/// Keeps the line that the test runs, and the name of `code`.
fn started(code: &Bound<'_, PyAny>) -> PyResult<()> {
    let id = key(code);
    let Some(Some(test)) =
        on_this_thread(|case| (case.on && !case.codes.contains_key(&id)).then_some(case.test))
    else {
        return Ok(());
    };
    let py = code.py();
    let mut frame = py.import("sys")?.getattr("_getframe")?.call0()?;
    while !frame.is_none() && !frame.getattr("f_code")?.is(code) {
        frame = frame.getattr("f_back")?;
    }
    if frame.is_none() {
        return Ok(());
    }
    let mut caller = frame.getattr("f_back")?;
    let mut manager = false;
    let (enter, enter_context) = enters(py)?;
    if !caller.is_none() && caller.getattr("f_code")?.is(enter.bind(py)) {
        manager = true;
        caller = caller.getattr("f_back")?;
        if !caller.is_none() && caller.getattr("f_code")?.is(enter_context.bind(py)) {
            caller = caller.getattr("f_back")?;
        }
    }
    if caller.is_none() {
        return Ok(());
    }
    let caller = key(&caller.getattr("f_code")?);
    let flags: u32 = code.getattr("co_flags")?.extract()?;
    if !manager && flags & RESUMES == 0 {
        let last = on_this_thread(|case| case.calls.get(&caller).map(|last| last.clone_ref(py)));
        let Some(Some(last)) = last else {
            return Ok(());
        };
        if !by_name(last.bind(py), code)? {
            return Ok(());
        }
    }
    let mut reached = None;
    while !frame.is_none() {
        if key(&frame.getattr("f_code")?) == test {
            reached = Some((
                frame.getattr("f_lineno")?.extract()?,
                code.getattr("co_name")?.extract()?,
            ));
            break;
        }
        frame = frame.getattr("f_back")?;
    }
    let code = code.clone().unbind();
    on_this_thread(|case| {
        if case.codes.contains_key(&caller) {
            case.codes.insert(id, code);
            if let Some(reached) = reached {
                case.reached.insert(id, reached);
            }
            if manager {
                case.managers.insert(id);
            }
        }
    });
    Ok(())
}

/// Keeps `callable` as the last call of `code`, when `code` is part of the
/// case. Gives whether it is.
fn called(code: &Bound<'_, PyAny>, callable: &Bound<'_, PyAny>) -> bool {
    let id = key(code);
    on_this_thread(|case| {
        if !case.on || !case.codes.contains_key(&id) {
            return false;
        }
        case.calls.insert(id, callable.clone().unbind());
        true
    })
    .unwrap_or(false)
}

/// The flags of a code object that runs as a generator, a coroutine or an
/// asynchronous generator: `CO_GENERATOR`, `CO_COROUTINE`,
/// `CO_ITERABLE_COROUTINE` and `CO_ASYNC_GENERATOR`.
/// <https://docs.python.org/3.12/library/inspect.html#code-objects-bit-flags>
const RESUMES: u32 = 0x20 | 0x80 | 0x100 | 0x200;

/// Whether `callable` calls `code` by name: it is the function of `code`, a
/// bound method of it, or a class whose `__init__` or `__new__` it is, which a
/// call of the class runs.
fn by_name(callable: &Bound<'_, PyAny>, code: &Bound<'_, PyAny>) -> PyResult<bool> {
    let runs = |function: &Bound<'_, PyAny>| -> PyResult<bool> {
        let function = function
            .getattr("__func__")
            .unwrap_or_else(|_| function.clone());
        Ok(function
            .getattr("__code__")
            .is_ok_and(|function_code| function_code.is(code)))
    };
    if runs(callable)? {
        return Ok(true);
    }
    if callable.is_instance_of::<PyType>() {
        for name in ["__init__", "__new__"] {
            if let Ok(method) = callable.getattr(name)
                && runs(&method)?
            {
                return Ok(true);
            }
        }
    }
    Ok(false)
}

/// Keeps the line of the `yield` at `at`, where a generator waits. It does
/// this only for a generator that a `with` of the case entered.
fn yielded(code: &Bound<'_, PyAny>, at: usize) -> PyResult<()> {
    let id = key(code);
    if on_this_thread(|case| case.on && case.managers.contains(&id)) != Some(true) {
        return Ok(());
    }
    let mut line = None;
    for entry in code.call_method0("co_lines")?.try_iter()? {
        let (start, end, at_line): (usize, usize, Option<usize>) = entry?.extract()?;
        if (start..end).contains(&at) {
            line = at_line;
        }
    }
    if let Some(line) = line {
        on_this_thread(|case| case.waiting.insert(id, line));
    }
    Ok(())
}

/// Records that `at` ran, when `code` is part of the case. A generator that
/// runs a line no longer waits: `throw()` resumes it with no `PY_RESUME`.
fn line(code: &Bound<'_, PyAny>, at: usize) -> bool {
    let id = key(code);
    on_this_thread(|case| {
        if !case.on || !case.codes.contains_key(&id) {
            return false;
        }
        case.waiting.remove(&id);
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
            for free in IDS {
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
            for (name, kind) in [
                ("PY_START", Kind::Start),
                ("CALL", Kind::Call),
                ("LINE", Kind::Line),
                ("PY_YIELD", Kind::Yield),
                ("PY_RESUME", Kind::Resume),
            ]
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

/// The tool ids that the watch can take, in the order it tries them.
const IDS: [u8; 2] = [3, 4];

/// Whether the watch tried to take a tool id already, in this process.
pub fn tried(py: Python<'_>) -> bool {
    TOOL.get(py).is_some()
}

/// Each tool id that the watch can take, and the name of the tool that holds
/// it, or `None` when it is free.
pub fn holders(py: Python<'_>) -> [(u8, Option<String>); 2] {
    let monitoring = py.import("sys").and_then(|sys| sys.getattr("monitoring"));
    IDS.map(|id| {
        let name = monitoring
            .as_ref()
            .ok()
            .and_then(|monitoring| monitoring.call_method1("get_tool", (id,)).ok())
            .filter(|name| !name.is_none())
            .map(|name| name.to_string());
        (id, name)
    })
}

/// The function that `sys.monitoring.set_local_events` held before [`wrap`]
/// replaced it. The watch sets its own events through it.
static SETTER: PyOnceLock<Py<PyAny>> = PyOnceLock::new();

/// Whether a call of the wrapper changed the events of the watch's tool since
/// [`events_changed`] last looked.
static CHANGED: AtomicBool = AtomicBool::new(false);

/// Takes the place of `sys.monitoring.set_local_events`. It notes a call that
/// changes the events of the watch's tool on a code object, then calls the
/// function that it replaced.
#[pyclass(frozen)]
struct Setter;

#[pymethods]
impl Setter {
    #[pyo3(signature = (*args, **kwargs))]
    fn __call__(
        _slf: &Bound<'_, Self>,
        args: &Bound<'_, PyTuple>,
        kwargs: Option<&Bound<'_, PyDict>>,
    ) -> PyResult<Py<PyAny>> {
        let py = args.py();
        if let Some(Some(tool)) = TOOL.get(py)
            && let Ok((id, code, events)) = args.extract::<(u8, Bound<'_, PyAny>, u32)>()
            && id == tool.id
        {
            // A call that the setter refuses raises in the setter's own words.
            let now = py
                .import("sys")
                .and_then(|sys| sys.getattr("monitoring"))
                .and_then(|monitoring| monitoring.call_method1("get_local_events", (id, code)))
                .and_then(|now| now.extract::<u32>());
            if now.is_ok_and(|now| now != events) {
                CHANGED.store(true, Ordering::Relaxed);
            }
        }
        setter(py)?.call(args, kwargs).map(Bound::unbind)
    }
}

/// The function that sets the events of a code object, as `sys.monitoring`
/// held it before [`wrap`].
fn setter(py: Python<'_>) -> PyResult<Bound<'_, PyAny>> {
    match SETTER.get(py) {
        Some(setter) => Ok(setter.bind(py).clone()),
        None => py
            .import("sys")?
            .getattr("monitoring")?
            .getattr("set_local_events"),
    }
}

/// Puts the wrapper in the place of `sys.monitoring.set_local_events`, so that
/// the watch hears each call that changes its events.
///
/// # Errors
///
/// `sys.monitoring` lacks `set_local_events`, or refuses the wrapper.
pub fn wrap(py: Python<'_>) -> PyResult<()> {
    let monitoring = py.import("sys")?.getattr("monitoring")?;
    SETTER.get_or_try_init(py, || {
        monitoring.getattr("set_local_events").map(Bound::unbind)
    })?;
    monitoring.setattr("set_local_events", Bound::new(py, Setter)?)
}

/// Whether a call changed the events of the watch's tool since this last
/// looked.
pub fn events_changed() -> bool {
    CHANGED.swap(false, Ordering::Relaxed)
}

/// Sets the events of the watch on each code object inside `code`, the code of
/// a module, at any depth. The module's own code stays unwatched. Gives
/// `false` when every tool id that the watch can take is taken.
pub fn arm(code: &Bound<'_, PyAny>) -> PyResult<bool> {
    let py = code.py();
    let Some(tool) = tool(py) else {
        return Ok(false);
    };
    let setter = setter(py)?;
    let code_type = code.get_type();
    let mut waiting: Vec<Bound<'_, PyAny>> = Vec::new();
    for constant in code.getattr("co_consts")?.try_iter()? {
        waiting.push(constant?);
    }
    while let Some(current) = waiting.pop() {
        if !current.is_instance(&code_type)? {
            continue;
        }
        setter.call1((tool.id, &current, tool.events))?;
        for constant in current.getattr("co_consts")?.try_iter()? {
            waiting.push(constant?);
        }
    }
    Ok(true)
}

/// Starts the watch of a case on this thread, whose test runs `test`, and
/// whose first expectation is at the line `before`.
pub fn start(py: Python<'_>, test: &Bound<'_, PyAny>, before: usize) {
    if tool(py).is_none() {
        return;
    }
    // A change of the events after an earlier case's first expectation, where
    // no step looks, is no fault of this case.
    CHANGED.store(false, Ordering::Relaxed);
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
            reached: HashMap::new(),
            calls: HashMap::new(),
            managers: HashSet::new(),
            waiting: HashMap::new(),
            cleanup: HashMap::new(),
            lines: HashMap::new(),
            ways: HashMap::new(),
        },
    );
}

/// Stops the watch of the case on this thread: it reached its first
/// expectation. The lines after the `yield` where each generator waits become
/// its cleanup.
pub fn stop() {
    if let Some(case) = cases()
        .as_mut()
        .and_then(|cases| cases.get_mut(&thread::current().id()))
    {
        case.on = false;
        case.cleanup = case.waiting.clone();
    }
}

/// A fault of the paths of a case: the line of the test, the line of the code
/// that the test reached when it is not the test's own, and the text.
pub type Fault = (usize, Option<usize>, String);

/// Ends the watch of the case on this thread, and gives each fault of its
/// paths, in the order of the test's lines, then of the lines that each
/// reached.
pub fn finish(py: Python<'_>) -> Vec<Fault> {
    let Some(case) = cases()
        .as_mut()
        .and_then(|cases| cases.remove(&thread::current().id()))
    else {
        return Vec::new();
    };
    let mut faults: Vec<Fault> = Vec::new();
    for (id, code) in &case.codes {
        let code = code.bind(py);
        let judged = |line: usize| {
            !(*id == case.test && line >= case.before)
                && case.cleanup.get(id).is_none_or(|waited| line <= *waited)
        };
        let fault = |line: usize, what: &str| match case.reached.get(id) {
            Some((test, name)) => (
                *test,
                Some(line),
                format!("line {test} reaches line {line} of {name}, which {what}, and {RULE}"),
            ),
            None => (line, None, format!("line {line} {what}, and {RULE}")),
        };
        let Ok(lines) = lines(code) else {
            continue;
        };
        let ran = case.lines.get(id);
        for line in lines {
            if judged(line) && !ran.is_some_and(|ran| ran.contains(&line)) {
                faults.push(fault(line, "did not run"));
            }
        }
        let Ok(branches) = branches(code) else {
            continue;
        };
        for ((owner, at), ways) in &case.ways {
            if owner != id || ways.len() > 1 {
                continue;
            }
            if let Some((line, _)) = branches.get(at)
                && judged(*line)
            {
                faults.push(fault(*line, "went one way only"));
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
