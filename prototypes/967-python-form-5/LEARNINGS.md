# What prototype 5, the stress test of prototype 4's form, taught (#967). Never merge.

Prototype 4's form ported all 28 items of three more loguru files, and all 28 pass. Only one check lost strength that the form forced: a timing check of `enqueue=True`.
The strictness that the runner applies while a case runs held on every axis. It cost the tester nothing, and it caught a fault that pytest only warns about.
The strictness that the runner reads from a test's syntax caused every refusal of the first attempt: 24 of 28 cases. In two places, the workaround it pushed the tester to is weaker than the original test. The third weaker workaround comes from the fork for each check.
The rest of what broke is two design questions that no prototype answers by itself, and runner bugs that a builder fixes.

Each finding below cites the porter's `NOTES.md` under `stress/<axis>/prototype-5/`, and a probe beside it. Each probe was run again before this file was written.

## What held

| what | evidence |
|---|---|
| **Every item ports, every port passes, and the checks keep their meaning** | 28 of 28; checks 20 (filesink-delay, for the original's 38 asserts, mostly inside `check_dir`), 31 (add-option-catch, 30 asserts and a `raises`), 14 (threading, 14 asserts) |
| **A case's own process replaces the reset after a test** | no port needs an undo after the case: the process ends |
| **Every check runs, after another fails** | filesink-delay seeded S12: a content check raises `FileNotFoundError`, and the next check still runs and passes |
| **An error in a thread fails the case** | threading probe e1: the runner reports "an error that no code saw: in a thread, RuntimeError". The same test under pytest 9.2.0 passes, with a `PytestUnhandledThreadExceptionWarning` only (`probes/pytest_twin/`) |
| **The case line is typed** | ty and pyright catch each misspelled or mistyped input that a porter seeded, in all three files, and are clean on all three ports |
| **A callback handed to the code may branch** | add-option-catch: `half_broken_sink` with `if`, `raise` and `nonlocal`, and a raising `__repr__`, were accepted at the first attempt |
| **Timing is kept** | threading: 14.3 s for the port, 14.2 s for the original; every sleep, barrier and join unchanged |

## What the first attempt met

Each porter first wrote each test as close to the original as `FORM.md` seemed to allow.
The runner accepted 4 of 28 cases. Each refusal comes from a rule that the runner reads from syntax.

| rule, as the runner reads it | cases refused (first attempt or later) | what the port does instead | weaker than the original? |
|---|---|---|---|
| a loop runs over literal data only | threading 7 (14 loops over `threads`); filesink-delay 4 (a comprehension in a check's reading) | `for k in range(2): threads[k].join()`; `sorted(os.listdir(…))` | **yes.** The count is written apart from the list. Seeded m8 joins 1 of 2 threads and passes 3 of 3 runs |
| a `with` other than `raises`, `warns`, `handling` is refused | filesink-delay 3 (`freeze_time`); add-option-catch 1 (`default_threading_excepthook`) | the setup is called in the act and never undone | **yes, once.** Swapping `threading.excepthook` switches off the runner's guard for a lost thread error (add-option-catch `probes/excepthook_hides_thread_error/`) |
| a check is one comparison | add-option-catch 6 (`out == err == ""`, `startswith`, a bare `re.match`); filesink-delay 1 (`assert not x`) | a tuple compare; `lines[-2][:18] == "ValueError: Error!"`, with the length counted by hand; `… is not None`; `x is False` | no, but a miscounted slice is caught only as a mismatch, and no checker sees it (add-option-catch s04) |
| the checks are the asserts at the end | filesink-delay 7 (2 asserts mid-test, 5 calls of the `check_dir` helper) | the mid-test reading kept in a name; 16 checks written out for 8 helper calls | no; longer |
| an expected value is a literal | filesink-delay 2 (`"7" * 20 + "\n"`, 3 times each) | a 21-character literal | no, but a miscount is caught only at run time (S11) |
| `raises` is the last statement of the act | add-option-catch 1 | the stderr read moved into the check | no |
| two checks read a stock only after a checkpoint | filesink-delay 3 (`tmp_path`) | the checkpoint copies the whole directory, and the restore copies it back: 32 copies in a run whose checks only read | no; it costs time |

A fourth weaker check comes from the fork, not from syntax: see the fork and a live thread below.

## What broke

### The fork for each check meets a live thread

[How does an experiment seal its product in Python?](https://github.com/kalonji-tools/purba/issues/964#issuecomment-6862174563) decided that each check but the last runs in a fork.
Python 3.12 warns at `os.fork()` when the process has a second thread: "use of fork() may lead to deadlocks in the child". loguru's `enqueue=True` keeps a worker thread alive.
The runner records its own fork's warning inside the case, and fails the case for a warning the test never raised. A test cannot expect it, because the fork comes after the act (add-option-catch `probes/bug_fork_warning/`).
The port stops the worker with `logger.remove()` at the end of the act. So it no longer checks that the original's 0.1 s sleep was enough for the worker to write.
The map kept "a thread that a product needs" in mind under the seal of a product. This is that case, in a real test. Blaming the test is a bug. The deadlock risk is a design question.

### The runner shares the process with what a test of logging tests

The runner uses three process-wide hooks of its own: fork, the capture of stdout and stderr, and `threading.excepthook`. A logging library writes to those same streams and hooks, and its tests patch them.

| what the test did | what happened | probe |
|---|---|---|
| a case built its capture at module level, `case(test, capsys=Capsys())`, which `FORM.md` allows | the capture swapped the runner's own streams at import; the run exits 1 and prints nothing | threading m2b |
| a file left out `before_each(reset_logger)` | loguru's default handler wrote to the runner's fd 2, not to the test's capture; `err == ""` held, and the case passed | threading m5 |
| a test swapped `threading.excepthook`, as the original does | the runner's guard for a lost thread error is off for that case | add-option-catch `excepthook_hides_thread_error/` |
| a test set `sys.stderr = None`, and has two checks | the runner flushes `sys.stderr` before each fork, and the case fails as "the act raised AttributeError" | add-option-catch `bug_stderr_none/` |

pytest has none of the first three: `capsys` and its thread hook are its own, and its reset is autouse.

### A setup that ends before the checks has no form

Six `with` blocks in 28 items set something up for a few statements and undo it: a frozen clock (3), a monkeypatch (2), a swapped excepthook (1).
The form refuses them, and nothing runs after the act. So the port makes the change and never undoes it, and the checks run under it.
None of the three files reads state after the undo, so no check changed. A test that checks the state after the undo cannot be ported.
A fixture cannot take an argument either: `use(factory)` calls `factory()`, so `freeze_time("2001-02-03")` became a helper that the act calls.

## Runner bugs

Each one contradicts `FORM.md`, and each was run again before this file was written.

| bug | found by | probe |
|---|---|---|
| a module-level or imported helper the test calls, and a module-level class it builds, are not held to one path. Prototype 4 fixed this only for a function the test defines inside itself | filesink-delay and threading, independently | `filesink-delay/…/probes/helper-paths/`; `threading/…/probes/test_e3_*`, `test_e4_*`, `test_e5_*` |
| the runner fails a case for its own fork's warning when a thread is alive | add-option-catch | `bug_fork_warning/` |
| "two checks read a stock with no checkpoint" is found after the act, and counted as failed, not refused | filesink-delay | `stock-two-checks/` |
| the stock rule reads the input's name only: a check through an alias (`same = d`) or a derived name (`f = d / "a"`) escapes it | filesink-delay | `stock-two-checks/` |
| a callback handed to the code by assignment (`stream.write = fn`) is read as part of the act; `setattr(stream, "write", fn)` passes | add-option-catch | `bug_handed_by_assignment/` |
| `sys.stderr = None` breaks the runner's flush before each fork | add-option-catch | `bug_stderr_none/` |
| a refusal does not always name every fault: a foreign `with` hides "checks nothing" | filesink-delay | `every-fault/` |
| a parameter default of the wrong type, `n: int = "3"`, is not refused; the same value from a case is | filesink-delay | `default-type/` |

The first bug is the second time a hole in the one-path rule was found after a fix. Each fix of a rule that reads syntax makes the runner larger.

## What the strategy learns

These are about designing strictness, not about this form.

1. **Strictness the runner applies while a case runs is cheap for the tester and held on every axis.** A process per case, every check run, an unexpected warning failed, a lost thread error failed, a typed case line. No porter had to change a test for any of them, and one caught a fault that pytest only warns about.
2. **Strictness read from syntax costs the tester at every refusal, and some refusals lead to a weaker test.** A rule that refuses a loop over the threads leads to `range(2)`, which can miss a thread. A rule that refuses a `with` leads to a patch that is never undone, and one that switched a guard off. A syntax rule should be judged by the test the tester writes next, not only by the test it refuses.
3. **A rule read from syntax keeps leaking.** Helpers, aliases and assignment each escaped a rule that `FORM.md` states in general words. The runner can name only what it parses.
4. **A runner that lives in the test's process must own its hooks apart from the code under test.** Its streams, its thread hook and its fork are the subject of a logging library's tests.
5. **Isolation by fork has a boundary at threads.** A product that keeps a thread alive is common: loguru's worker, executors, clients with a pool. The seal needs an answer for it that a tester does not have to know about.

## Another iteration, or close #967?

What another prototype of this form would measure: the same syntax rules, on new tests. The last three rounds gave complexity for the tester 2.3 each time. This round found the same kind of cost, on three new axes.

What a prototype cannot settle: the two design questions above (the fork and a live thread; a setup that ends before the checks), and which syntax rules move to run time or go. Those are decisions, and each fits a grilling issue of the map.

What a builder settles: the runner bugs. None changes the form.
