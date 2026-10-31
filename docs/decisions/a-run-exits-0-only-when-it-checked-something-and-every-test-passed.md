# A run exits 0 only when it checked something and every test passed

## Context and Problem Statement

An [integrator](../../CONTEXT.md#integrator) reads the exit code of a [run](../../CONTEXT.md#run), and reads 0 as clean.
[The landscape](../../LANDSCAPE.md#the-quality-priorities) says that purba never shows `unknown` as clean.
The exit code joins the [public surface](../../CONTEXT.md#public-surface).
[A pinned candidate decides when a milestone is done](a-pinned-candidate-decides-when-a-milestone-is-done.md#decision-outcome) leaves it to review alone.
[A tester runs purba on one module, and reads one line for each test](a-tester-runs-purba-on-one-module-and-reads-one-line-for-each-test.md) holds how a run starts.

| fact | source |
|---|---|
| a console script passes the integer its function returns to `sys.exit` | [the entry points specification](https://github.com/pypa/packaging.python.org/blob/d1ad4ad3d6d6f3a3582e4c6b8a1b6f8284f4cb3b/source/specifications/entry-points.rst?plain=1#L125-L142) |
| CPython exits 1 when an exception leaves the file it runs | [CPython's `main.c`](https://github.com/python/cpython/blob/v3.15.0/Modules/main.c#L437-L439), [its exit code](https://github.com/python/cpython/blob/v3.15.0/Modules/main.c#L122-L128) |
| PyO3 turns a Rust panic into `PanicException`, which derives from `BaseException` and not from `Exception` | [PyO3's `PanicException`](https://github.com/PyO3/pyo3/blob/v0.29.2/src/panic.rs#L6-L16) |
| clap exits 2 on a usage error | [clap's exit codes](https://github.com/clap-rs/clap/blob/v4.6.7/clap_builder/src/util/mod.rs#L26-L32) |
| pytest exits 0 when its tests pass, 1 when a test fails, 2 when it is interrupted, 3 on an internal error, 4 on misuse, and 5 when it finds no test | [pytest's `ExitCode`](https://github.com/pytest-dev/pytest/blob/9.1.1/src/_pytest/config/__init__.py#L101-L120) |
| pytest 9.0.3 reports a test that raises `ValueError` as failed, and exits 1 | [the probe](https://github.com/kalonji-tools/purba/issues/608#issuecomment-6318260548) |
| `os._exit` exits at once, with no cleanup handler, and should normally be called only in the child of a fork | [CPython's `os.rst`](https://github.com/python/cpython/blob/v3.15.0/Doc/library/os.rst?plain=1#L4955-L4963) |
| a child that `multiprocessing` forks ends with `os._exit` | [CPython's `popen_fork.py`](https://github.com/python/cpython/blob/v3.15.0/Lib/multiprocessing/popen_fork.py#L81-L90) |
| a test that calls `os._exit(0)`, `std::process::exit(0)` or `process.exit(0)` after another test failed ends pytest, `unittest`, `cargo test` and Jest with 0 | [the research of exit codes](https://github.com/kalonji-tools/purba/blob/9b3fcc44897ffff09a876f00034adf6fe88a81dd/research/exit-codes.md?plain=1#L120-L123) |
| Go panics when a test calls `os.Exit(0)`, and on no other code | [Go 1.16 notes](https://go.dev/doc/go1.16#go-test), [Go's `proc.go`](https://github.com/golang/go/blob/go1.27.3/src/os/proc.go#L62-L68) |
| Vitest throws when a test calls `process.exit`, with any code | [Vitest's `base.ts`](https://github.com/vitest-dev/vitest/blob/v5.0.5/packages/vitest/src/runtime/workers/base.ts#L33-L38) |
| `faulthandler` calls the C `_exit(1)` when its timer fires with `exit=True` | [its docstring](https://github.com/python/cpython/blob/v3.15.0/Modules/faulthandler.c#L798-L799), [the call](https://github.com/python/cpython/blob/v3.15.0/Modules/faulthandler.c#L713-L723) |
| CI, a pre-commit hook and `make` fail on any code other than 0, and `git bisect run` skips a commit on 125 | [the research of exit codes](https://github.com/kalonji-tools/purba/blob/9b3fcc44897ffff09a876f00034adf6fe88a81dd/research/exit-codes.md?plain=1#L9-L13) |
| in a measured bisect, a runner fault that exited 4 made `git bisect run` blame the wrong commit, and 125 found the right one | [the research of exit codes](https://github.com/kalonji-tools/purba/blob/9b3fcc44897ffff09a876f00034adf6fe88a81dd/research/exit-codes.md?plain=1#L14) |
| no surveyed checking tool gives a digit of an exit code a meaning | [the research of exit-code encodings](https://github.com/kalonji-tools/purba/blob/c3cdcec48fda70673fce384aaa373cf5abee3390/research/exit-code-encodings.md?plain=1#L263) |
| SMTP gives its second digit a meaning, and only 5 codes were left for mail system errors | [the research of exit-code encodings](https://github.com/kalonji-tools/purba/blob/c3cdcec48fda70673fce384aaa373cf5abee3390/research/exit-code-encodings.md?plain=1#L161) |
| Bazel says that infrastructure decides from the exit code whether to retry a request | [the research of exit-code encodings](https://github.com/kalonji-tools/purba/blob/c3cdcec48fda70673fce384aaa373cf5abee3390/research/exit-code-encodings.md?plain=1#L217) |
| `timeout` and `env` exit 125 when they fail themselves | [the research of exit-code encodings](https://github.com/kalonji-tools/purba/blob/c3cdcec48fda70673fce384aaa373cf5abee3390/research/exit-code-encodings.md?plain=1#L122) |

## Considered Options

- **`unknown` exits 1, as a failed test does.** Rejected. An integrator could not tell a test that found a defect from a test that checked nothing.
- **A run with no test exits 0.** Rejected. It reads as clean, and the run checked nothing.
- **An error that `_main` does not catch.** Rejected. It exits 1, so an internal error reads as a failed test. [The landscape](../../LANDSCAPE.md#the-risks) names that risk.
- **A test that raised exits 1, as a failed test does.** Rejected. An integrator could not tell a test whose assert found a defect from a test whose code raised.
- **A test that raised exits 3.** Rejected. 3 says that the run did not finish, but the run goes on past the test.
- **Codes that keep `unknown`, a run with no test, and a test that raised or gave no outcome apart from a failure.** Chosen. Only a run that checked at least one behaviour, in which every test has an outcome and none failed, raised or is `unknown`, exits 0.
- **A test that calls `os._exit` ends the run with its code, and the Downside names it.** Rejected. With 0, the run reads as clean though an earlier test failed.
- **Only `os._exit(0)` makes the test raise, as in Go.** Rejected. Another code becomes the exit code of the run, so `os._exit(1)` reads as a failed test.
- **The guard raises and records nothing, and the Downside names a call whose error a thread or the test catches.** Rejected. A test that hides the call passes, and the run can exit 0. Before the guard, the run ended with the code of the call.
- **Off the test's thread, a code other than 0 calls the real `os._exit`.** Rejected. `faulthandler` already ends a hung test, and the guard would need to know which thread runs the test.
- **A guard takes the place of `os._exit` in purba's process.** Chosen. A test that calls it has the outcome `raised`. A child that a test forks still exits.
- **A digit names the kind, and a second digit names the failure.** Rejected. No surveyed tool gives a digit a meaning. SMTP's second digit ran out of codes.
- **Each kind has a code of its own.** Rejected. No surveyed tool that reads an exit code tells a test that raised apart from a project error.
- **Only the line that purba prints names the kind, and 3 and 4 each hold several kinds.** Rejected. An internal error then reads as an invocation error or as a test that raised.
- **Only an internal error gets a new code.** Chosen. `git bisect run` and a system that retries each ask whether the tool itself failed.
- **A failed test comes first in the order.** Rejected. A run with a failed test and a panic inside another test then exits 1, which reads as a failed test.
- **The order puts an internal error first, then an invocation error, then a failed test.** Chosen. An internal error never comes after a failed test. A failed test still comes before what the run could not check.
- **An internal error exits 125.** Chosen. `git bisect run` skips the commit, and does not blame it.

## Decision Outcome

**Reach:** `crates/**`

**A run exits 0 only when at least one test ran and every test passed.**

| code | the run |
|---|---|
| 0 | at least one test ran, and every test passed |
| 1 | a test failed |
| 2 | a test is `unknown`, or no test ran |
| 3 | an invocation error |
| 4 | a test raised or gave no outcome, or a part saw a project error |
| 125 | an internal error |

- **When a run meets more than one row, its code is the first of 125, 3, 1, 4, 2 and 0.**
- **An internal error has a code of its own, 125.** The line that purba prints names each kind.
- **`_main` catches each error at its boundary.** It returns 3 for an invocation error, and 125 for a panic.
- **Command reads its argument without clap.** clap exits 2 on a usage error, and 2 means `unknown` here.
- **This record chooses no code for an interrupt.** The design in depth of interrupt handling chooses it.
- **A test that calls `os._exit` has the outcome `raised`.** The guard records the first call since the last test. The worker takes that call when the test returns. So a call from a thread, or one whose error the test catches, counts too. In a process that a test forks, `os._exit` still exits.

[Reporting renders the exit code, and command returns it](reporting-renders-the-exit-code-and-command-returns-it.md#decision-outcome) holds which part renders the code.
[A failure's kind is what the part that first sees it observes](a-failures-kind-is-what-the-part-that-first-sees-it-observes.md#decision-outcome) holds each kind of failure.

**Downside:**

- **pytest exits 2 for an interrupted run, and purba exits 2 for `unknown`.** An integrator that reads pytest's codes misreads purba's 2.
- **An integrator cannot tell a test that raised from a project error by the code.** A run with either exits 4. The line tells them apart.
- **A crash before `_main` exits 1, as a failed test does.** CPython exits 1 when `import purba` fails.
- **A wrapped run's 125 can be the wrapper's.** `timeout` and `env` exit 125 when they fail themselves.
- **Ctrl-C ends only the test that runs.** Python raises `KeyboardInterrupt` inside the test, so the test has the outcome `raised`. Dispatch hands on the next test. A tester presses Ctrl-C once for each test left, and the run exits 4.
- **An internal error that reaches dispatch exits 4, as a test that raised does.** [A panic inside a test gives that test an internal error](a-panic-inside-a-test-gives-that-test-an-internal-error.md#decision-outcome) tells the two apart, and [a panic inside a test prints that test's internal error, and tests/kinds.bats checks each kind](https://github.com/kalonji-tools/purba/issues/729) builds what it decides.
- **The table gives no code to a run that a suppression fails, or to a skipped test.** [A suppression is an outcome and not a switch](a-suppression-is-an-outcome-and-not-a-switch.md#decision-outcome) fails such a run. The design in depth of strictness chooses each code.
- **A test that exits by another route ends the run with its own code.** `posix._exit`, `ctypes` and the C code of an extension are such routes. With 0, the run reads as clean though an earlier test failed.
- **A thread that calls `os._exit` after its test returned charges the test that runs when it calls.** After the last test, no test takes the call, and the run can exit 0.
- **A thread that calls `os._exit` while its test hangs does not end the run.** The run waits on the test. `faulthandler` with `exit=True` still ends it, because it exits from C.

## Confirmation

`mise run test:rust` checks that reporting renders 0, 1, 2 and 4 from a tally of the run.
It also checks that a call to `os._exit` in a test gives the outcome `raised`, from a thread or under `except BaseException` too.
It checks that a child that a test forks exits with its own code, whether the call names `status` or not.
It checks that command returns 1 for a module with a failed test, and returns 2 for a module with no test.
`mise run test:wheel` checks that the installed `purba` exits 1 on `test_a.py`, 3 on an invocation error, and 125 when the `fault` wheel panics.
[bats tests the installed wheel from outside purba](bats-tests-the-installed-wheel-from-outside-purba.md#decision-outcome) decides what `mise run test:wheel` runs.

No command checks the rest.
[A panic inside a test prints that test's internal error, and tests/kinds.bats checks each kind](https://github.com/kalonji-tools/purba/issues/729) builds exit 125 for a run in which a panic inside a test stops only that test.
It adds its check.
