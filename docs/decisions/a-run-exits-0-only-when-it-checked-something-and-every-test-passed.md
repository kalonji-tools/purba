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

## Considered Options

- **`unknown` exits 1, as a failed test does.** Rejected. An integrator could not tell a test that found a defect from a test that checked nothing.
- **A run with no test exits 0.** Rejected. It reads as clean, and the run checked nothing.
- **An error that `_main` does not catch.** Rejected. It exits 1, so an error inside purba reads as a failed test. [The landscape](../../LANDSCAPE.md#the-risks) names that risk.
- **Four codes, which keep `unknown` and a run with no test apart from a failure.** Chosen. Only a run that checked at least one behaviour, with no failed or `unknown` test, exits 0.

## Decision Outcome

**Reach:** `crates/**`

**A run exits 0 only when at least one test ran and every test passed.**

| code | the run |
|---|---|
| 0 | at least one test ran, and every test passed |
| 1 | a test failed |
| 2 | no test failed, and a test is `unknown` or no test ran |
| 3 | the run did not finish: a usage error, a path that does not exist, or an error inside purba |

- **`_main` catches each error at its boundary, and returns 3.**
- **`_main` reads its argument without clap.** clap exits 2 on a usage error, and 2 means `unknown` here.
- **An interrupt is not an error here.** The design in depth of interrupt handling chooses its code.
- **The design in depth of errors may divide 3.**

**Downside:**

- **pytest exits 2 for an interrupted run, and purba exits 2 for `unknown`.** An integrator that reads pytest's codes misreads purba's 2.
- **An integrator cannot tell a usage error, a path that does not exist and an error inside purba apart.** All three exit 3 until the design in depth of errors divides them.
- **The table gives no code to a run that a suppression fails, or to a skipped test.** [A suppression is an outcome and not a switch](a-suppression-is-an-outcome-and-not-a-switch.md#decision-outcome) fails such a run. The design in depth of strictness chooses each code.

## Confirmation

No command checks it, because purba holds no product code.
The end-to-end test of [Wayfinder: purba's walking skeleton, running every part on main](https://github.com/kalonji-tools/purba/issues/533) runs `purba` on one module and reads its exit code.
[bats tests the installed wheel from outside purba](bats-tests-the-installed-wheel-from-outside-purba.md#decision-outcome) decides what runs that test.
