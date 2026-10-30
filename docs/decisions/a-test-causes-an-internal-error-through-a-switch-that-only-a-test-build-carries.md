# A test causes an internal error through a switch that only a test build carries

## Context and Problem Statement

[Wayfinder: purba's errors, telling each kind of failure apart from the first part that sees it](https://github.com/kalonji-tools/purba/issues/652) ends with a test of the wheel.
That test runs purba on a project, and causes an [internal error](../../CONTEXT.md#internal-error) inside one of its tests on purpose.
No file of a tester causes one on demand, and the [public surface](../../CONTEXT.md#public-surface) has no switch for it.

| fact | source |
|---|---|
| a panic in the assert hook ends the run with exit 3, and prints no line for any test | [How does a test cause a defect inside purba during a run?](https://github.com/kalonji-tools/purba/issues/657#issuecomment-6415193768) |
| a module named `builtins.py` gives a project error | [How does a test cause a defect inside purba during a run?](https://github.com/kalonji-tools/purba/issues/657#issuecomment-6415193768) |
| a module that declares `coding: latin-1` and holds UTF-8 bytes gives `failed`, where CPython passes | [How does a test cause a defect inside purba during a run?](https://github.com/kalonji-tools/purba/issues/657#issuecomment-6415193768) |
| a cargo test that hands dispatch a broken rewrite reads the line and the exit code | [the prototype's cargo test](https://github.com/kalonji-tools/purba/blob/cb31802/crates/command/tests/proto_657.rs) |

## Considered Options

- **A file of a tester that breaks purba.** Rejected. `builtins.py` gives a project error, and the `latin-1` module gives `failed`. The fix of either defect turns the test red.
- **A test of a part that hands the next part a broken value.** Rejected. It never starts the console script, so it misses the exit 3 of a panic.
- **A switch in the wheel that a tester installs.** Rejected. The switch then adds to the public surface.
- **A switch that only a test build carries.** Chosen. It causes an internal error at a chosen point, through the console script that the wheel installs.

## Decision Outcome

**Reach:** `crates/**` `/.config/tasks.toml` `tests/**` `.github/workflows/build.yml` `.github/workflows/publish.yml` `/pyproject.toml`

**A test causes an internal error through a switch that only a test build carries.**

- **The cargo feature `fault` makes a part panic, or raise an error purba never expects, at the point that `PURBA_FAULT` names.** The wheel that a tester installs leaves the feature out, so the public surface does not change.
- **The test that causes an internal error runs on a second wheel, which `mise run test:wheel` builds with `fault` and installs.** Every other test of the wheel runs on the wheel without `fault`, as [bats tests the installed wheel from outside purba](bats-tests-the-installed-wheel-from-outside-purba.md#decision-outcome) decides.

**Downside:**

- **`mise run test:wheel` builds two wheels.** Each local `mise run check` waits for both.
- **The wheel under the test of an internal error differs from the wheel a tester installs.** The code that `fault` adds runs only in the test build.
- **A wrong answer inside purba stays out of reach.** Execution sees only a failed assert, so the tests of each part guard it.

## Confirmation

`mise run test:wheel`, and the `wheel` job of Build on each Linux and macOS cell, run `tests/fault.bats`.
It checks that the `fault` wheel panics under `PURBA_FAULT=assert`, that it passes without the variable, and that the wheel without `fault` does not panic.
