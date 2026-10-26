# bats tests the installed wheel from outside purba

## Context and Problem Statement

The walking skeleton needs a test that runs purba on a small project of two modules.
[A tester runs purba on one module, and reads one line for each test](a-tester-runs-purba-on-one-module-and-reads-one-line-for-each-test.md#confirmation) and [a run exits 0 only when it checked something and every test passed](a-run-exits-0-only-when-it-checked-something-and-every-test-passed.md#confirmation) each name that test as their check.
[purba writes its scripts in bash until purba can test them](purba-writes-its-scripts-in-bash-until-purba-can-test-them.md#decision-outcome) moves the scripts to Python when purba runs purba's own tests.

| fact | source |
|---|---|
| bats ran the installed `purba` on ubuntu and macOS, on 3.12, 3.13, 3.14 and 3.14t | [What runs the walking skeleton's end-to-end test, and on which interpreters?](https://github.com/kalonji-tools/purba/pull/544#issuecomment-6152302976) |
| all four Windows arms found no `bats`, and each loaded the wheel | [What runs the walking skeleton's end-to-end test, and on which interpreters?](https://github.com/kalonji-tools/purba/pull/544#issuecomment-6152302976) |
| `.config/mise.lock` locks `bats` for Linux and macOS only | [`.config/mise.lock`](https://github.com/kalonji-tools/purba/blob/f5fbbff9923b78ebf99e04aec732edb5aaaf1040/.config/mise.lock#L42-L68) |
| an import whose error a prototype `_main` swallowed left stdout and the exit code unchanged | [What runs the walking skeleton's end-to-end test, and on which interpreters?](https://github.com/kalonji-tools/purba/issues/535#issuecomment-6152849037) |
| a test of the sentinel alone passed while `purba` was not installed | [What runs the walking skeleton's end-to-end test, and on which interpreters?](https://github.com/kalonji-tools/purba/issues/535#issuecomment-6152849037) |

## Considered Options

- **A cargo test that calls `_main` in-process.** Rejected. It skips the wheel and the console script, which the test must confirm.
- **The test on `main` now, under `skip`.** Rejected. A skipped test checks nothing, and Build reports it as a pass.
- **The test on `main` now, against a stub `_main`.** Rejected. The stub turns the test green, and the test then proves nothing.
- **bats against the installed command, from outside purba.** Chosen. It crosses the console script and the wheel.

## Decision Outcome

**Reach:** `/.config/tasks.toml` `.github/workflows/build.yml` `tests/**` `crates/**` `/pyproject.toml`

**bats tests the `purba` command that the wheel installs, from outside purba.**

| file | holds |
|---|---|
| `tests/skeleton.bats` | the test |
| `tests/skeleton/test_a.py` | `test_one` passes, `test_two` fails, and `test_three` is `unknown` |
| `tests/skeleton/test_b.py` | a module that writes the file `$PURBA_SENTINEL` names, then raises |

- **`tests/skeleton.bats` runs `purba test_a.py` inside `tests/skeleton/`.** Each line then starts `test_a.py::`.
- **One test holds the run and every check.** A check of the sentinel in a test of its own passes when `purba` never ran.

| check | what it refuses |
|---|---|
| `diff -u` of stdout against the lines that the tester record shows, with the path `test_a.py` | a wrong line, a missing line, and a run that never started |
| exit code 1 | every other exit code |
| the sentinel file is absent | an import of `test_b.py`, even when purba swallows its error |

- **`mise run test:wheel` builds the wheel, installs it into a venv under `target/`, and runs `bats` over `tests/`.** `test` and `check` run it.
- **The test that causes an internal error runs on a second wheel.** [A test causes an internal error through a switch that only a test build carries](a-test-causes-an-internal-error-through-a-switch-that-only-a-test-build-carries.md#decision-outcome) says why.
- **Build's `wheel` job runs the test after it loads the wheel.** Every arm runs it except the Windows arms.
- **The test stays in bats after purba runs its own tests.** A broken purba could report its own test as passed.
- **The slice of the skeleton that turns the test green adds it.** `main` cannot hold the test red.

**Downside:**

- **Windows never runs the console script.** The Windows arms import the wheel's module, and never start the command that pip installs there.
- **Every local `mise run check` builds and installs the wheel.**
- **The sentinel sees an import, and nothing less.** A rewrite that reads `test_b.py` and never imports it passes the test.

## Confirmation

`mise run test:wheel` runs `bats` over `tests/`, against the `purba` that the wheel installs.
Build's `wheel` job runs `tests/skeleton.bats` on each Linux and macOS arm.
