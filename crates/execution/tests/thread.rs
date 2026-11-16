use std::path::PathBuf;

use purba_execution::{Kind, Module, Owner, Rewrite};

fn module(name: &str, source: &str) -> Module {
    Module::new(
        PathBuf::from(name),
        Rewrite::of_source(source, &tests_in(source)).expect("the source parses"),
    )
}

/// The owner of a thread: the test, by the name of its function, or a load.
type Owned = Owner<&'static str>;

const fn test(function: &'static str) -> Owned {
    Owner::Test(function)
}

fn load(path: &str) -> Owned {
    Owner::Load(PathBuf::from(path))
}

/// The outcome or the error of a load or a test, and the error of each thread
/// that the worker took after it.
type Step = (Result<Option<Kind>, String>, Vec<(Owned, String)>);

/// A step that gave `result`, and the error of each thread that it took, from
/// its owner, the name of the thread and its exception.
fn step(result: Option<Kind>, threads: &[(Owned, &str, &str)]) -> Step {
    (
        Ok(result),
        threads
            .iter()
            .map(|(owner, thread, exception)| {
                (owner.clone(), format!("thread {thread} raised {exception}"))
            })
            .collect(),
    )
}

/// Threads that raise inside a test, which joins them.
const TEST_THREAD: &str = "import sys\nimport threading\n\n\
    class Quit(SystemExit):\n    pass\n\n\
    def leave():\n    raise Quit(1)\n\n\
    def run(name, target):\n    thread = threading.Thread(target=target, name=name)\n\
    \x20   thread.start()\n    thread.join()\n\n\
    def test_joins():\n    run(\"A\", lambda: 1 / 0)\n    assert True\n\n\
    def test_exits():\n    run(\"S\", lambda: sys.exit(3))\n\
    \x20   run(\"Q\", leave)\n    assert True\n\n\
    def test_two():\n    run(\"B\", lambda: [][1])\n    run(\"C\", lambda: {}[\"k\"])\n\
    \x20   assert True\n";

/// A thread that `test_leaves` leaves alive, and that raises during `test_releases`.
const TEST_LATE: &str = "import threading\n\ngo = threading.Event()\nleft = []\n\n\
    def late():\n    go.wait()\n    raise ValueError(\"late\")\n\n\
    def test_leaves():\n    thread = threading.Thread(target=late, name=\"T\", daemon=True)\n\
    \x20   thread.start()\n    left.append(thread)\n    assert True\n\n\
    def test_releases():\n    go.set()\n    left[0].join()\n    assert True\n";

/// A thread that raises while the module loads, which joins it.
const TEST_LOAD_THREAD: &str = "import threading\n\n\
    thread = threading.Thread(target=lambda: 1 / 0, name=\"L\")\n\
    thread.start()\nthread.join()\n\n\
    def test_one():\n    assert True\n";

/// A module that the test reads, which waits until a condition holds. `exec`
/// makes it, so its loop is no code of the test module, as a module that the
/// test imports is not.
const WAITS: &str = "import types\n\nwaits = types.ModuleType(\"waits\")\n\
    exec(\"import time\\n\\ndef until(ready):\\n    while not ready():\\n\
    \x20       time.sleep(0.01)\\n\", waits.__dict__)\n\n";

/// A thread that `_thread` starts in a test, which waits for `_thread._count()`
/// to fall, as `tests/skeleton.bats` does.
const TEST_RAW_THREAD: &str = "import _thread\n\nstarted = []\n\n\
    def body():\n    started.append(1)\n    raise ValueError(\"raw\")\n\n\
    def test_raw():\n    before = _thread._count()\n    _thread.start_new_thread(body, ())\n\
    \x20   waits.until(lambda: started and _thread._count() <= before)\n\
    \x20   assert True\n";

/// A thread that `_thread` starts in `test_leaves`, and that raises during
/// `test_releases`, which waits for `_thread._count()` to fall.
const TEST_RAW_LATE: &str = "import _thread\n\n\
    go = _thread.allocate_lock()\ngo.acquire()\nstarted = []\nbefore = []\n\n\
    def late():\n    started.append(1)\n    go.acquire()\n    raise ValueError(\"late raw\")\n\n\
    def test_leaves():\n    before.append(_thread._count())\n\
    \x20   _thread.start_new_thread(late, ())\n    waits.until(lambda: started)\n\
    \x20   assert True\n\n\
    def test_releases():\n    go.release()\n\
    \x20   waits.until(lambda: _thread._count() <= before[0])\n    assert True\n";

/// `text` with the number that names a thread `_thread` started replaced, so
/// that two runs compare equal.
fn without_ident(text: String) -> String {
    let Some(rest) = text.strip_prefix("thread ") else {
        return text;
    };
    let digits = rest.bytes().take_while(u8::is_ascii_digit).count();
    if digits == 0 || !rest[digits..].starts_with(" raised") {
        return text;
    }
    format!("thread <ident>{}", &rest[digits..])
}

/// A thread that the load leaves alive, and that raises during the test.
const TEST_LOAD_LEAVES: &str = "import threading\n\ngo = threading.Event()\n\n\
    def late():\n    go.wait()\n    raise ValueError(\"late\")\n\n\
    thread = threading.Thread(target=late, name=\"M\", daemon=True)\nthread.start()\n\n\
    def test_releases():\n    go.set()\n    thread.join()\n    assert True\n";

// The hooks on `threading.excepthook` and `sys.unraisablehook` keep one record
// for the process, and cargo runs the tests of one binary on several threads.
// The `threads` of another test could take the record of an exception raised
// here. So each thread that raises in this process lives in this one test,
// alone in its binary.
#[test]
fn a_threads_exception_charges_the_test_or_the_load_that_left_the_thread() {
    let test_thread = module("test_thread.py", TEST_THREAD);
    let test_late = module("test_late.py", TEST_LATE);
    let test_load_thread = module("test_load_thread.py", TEST_LOAD_THREAD);
    let test_load_leaves = module("test_load_leaves.py", TEST_LOAD_LEAVES);
    let test_raw_thread = module("test_raw_thread.py", &format!("{WAITS}{TEST_RAW_THREAD}"));
    let test_raw_late = module("test_raw_late.py", &format!("{WAITS}{TEST_RAW_LATE}"));
    // A step with no function is a `load` alone, as dispatch makes it.
    let steps: Vec<Step> = purba_execution::attach(|worker| {
        [
            (&test_thread, None),
            (&test_thread, Some("test_joins")),
            (&test_thread, Some("test_exits")),
            (&test_thread, Some("test_two")),
            (&test_late, None),
            (&test_late, Some("test_leaves")),
            (&test_late, Some("test_releases")),
            (&test_load_thread, None),
            (&test_load_leaves, None),
            (&test_load_leaves, Some("test_releases")),
            (&test_raw_thread, None),
            (&test_raw_thread, Some("test_raw")),
            (&test_raw_late, None),
            (&test_raw_late, Some("test_leaves")),
            (&test_raw_late, Some("test_releases")),
        ]
        .iter()
        .map(|(module, function)| {
            let result = match function {
                Some(function) => worker
                    .run(module, function, &[], *function)
                    .map(|outcome| Some(outcome.kind().clone())),
                None => worker.load(module).map(|()| None),
            }
            .map_err(|error| error.to_string());
            let threads = worker
                .threads()
                .into_iter()
                .map(|(owner, error)| (owner, without_ident(error.to_string())))
                .collect();
            (result, threads)
        })
        .collect()
    });
    let passed = || Some(Kind::Passed);
    let zero = "ZeroDivisionError: division by zero";
    assert_eq!(
        steps,
        [
            step(None, &[]),
            step(passed(), &[(test("test_joins"), "A", zero)]),
            step(passed(), &[(test("test_exits"), "Q", "Quit: 1")]),
            step(
                passed(),
                &[
                    (test("test_two"), "B", "IndexError: list index out of range"),
                    (test("test_two"), "C", "KeyError: 'k'"),
                ]
            ),
            step(None, &[]),
            step(passed(), &[]),
            step(passed(), &[(test("test_leaves"), "T", "ValueError: late")]),
            step(None, &[(load("test_load_thread.py"), "L", zero)]),
            step(None, &[]),
            step(
                passed(),
                &[(load("test_load_leaves.py"), "M", "ValueError: late")]
            ),
            step(None, &[]),
            step(
                passed(),
                &[(test("test_raw"), "<ident>", "ValueError: raw")]
            ),
            step(None, &[]),
            step(passed(), &[]),
            step(
                passed(),
                &[(test("test_releases"), "<ident>", "ValueError: late raw")]
            ),
        ],
        "each test keeps its outcome and each load its result. Each exception of a thread, \
         but SystemExit itself, charges the test or the load that left the thread alive, and \
         otherwise the one that ran when it raised",
    );
}

/// The name of each `def test_…` at the top of `source`, as selection names
/// the tests of a module.
fn tests_in(source: &str) -> Vec<&str> {
    source
        .lines()
        .filter_map(|line| line.strip_prefix("def test_").map(|_| &line[4..]))
        .filter_map(|line| line.split_once('(').map(|(name, _)| name))
        .collect()
}
