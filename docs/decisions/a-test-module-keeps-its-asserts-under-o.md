# A test module keeps its asserts under -O, and a run under -O warns that no other assert runs

## Context and Problem Statement

`python -O`, `-OO` and a `PYTHONOPTIMIZE` above 0 make CPython strip each assert, and make `__debug__` `False`.
`-OO` strips each docstring too.
On `main` at `680e274`, assertion compiles the test module at the interpreter's optimize level.
So `-O` turns strictness off where [the landscape](../../LANDSCAPE.md#the-cross-cutting-concerns) says that no switch does.

| fact | source |
|---|---|
| on `main` at `680e274`, under `PYTHONOPTIMIZE=1`, purba prints `passed` for a test whose helper in the test module asserts, and the run exits 0 | [Keep the asserts of a test module under -O, and warn that the others do not run](https://github.com/kalonji-tools/purba/issues/1027) |
| a plain run refuses that test | [What does purba print when Python strips every assert?](https://github.com/kalonji-tools/purba/issues/683#issuecomment-6915316724) |
| at optimize level 0, under `-O`, each probe compiles to the same bytecode as in a plain run | the same |
| pytest 9.2.0 keeps the asserts of a test module under `-O`, and warns on each run, also for a path that does not exist | the same, and [the prototype of the warning](https://github.com/kalonji-tools/purba/tree/0064f4fb8cf61b97d5814ea96b7d74ed777c7642/prototypes/1027-optimize) |
| no test file named `test_*.py` or `*_test.py` in the 9 pinned suites of [the cost of the rules](https://github.com/kalonji-tools/purba/tree/8e712dbffa95985777551b4d9d5c743083c8ba29/prototypes/1015-rules-cost) reads `__debug__`: 0 of 189 | [What does purba print when Python strips every assert?](https://github.com/kalonji-tools/purba/issues/683#issuecomment-6915316724) |

## Considered Options

- **The test module follows the interpreter.** `-O` hides the project error of each assert before the first expectation, and the run exits 0 for a test that a plain run refuses.
- **A run under `-O` writes no warning.** The tester does not learn that the asserts of every other module do not run.

## Decision Outcome

**Reach:** `crates/assertion/**` `crates/purba/**` `tests/optimize.bats` `tests/optimize/**`

**Area:** assertion, command, strictness

**Assertion compiles the test module at optimize level 0, so `-O` hides no project error.**
Under `-O`, the test module keeps each assert, each docstring and `__debug__` as `True`.
Its code is the code of a plain run.
A module that the test imports follows the interpreter, as [purba runs inside the tester's interpreter](purba-runs-inside-the-testers-interpreter.md#decision-outcome) holds.

- **A run under `-O` writes one warning to stderr, before any other line.** The line is `purba: warning: Python runs with -O, so an assert outside a test module does not run`. It also comes before the line of each invocation error. [A tester reads a line for each case](a-tester-runs-purba-on-one-module-and-reads-one-line-for-each-test.md#decision-outcome) names those lines.
- **A warning is not an error, so it names no kind.** [A line names its kind in the glossary's words](a-line-names-its-kind-in-the-glossarys-words.md#decision-outcome) names the kinds of an error.
- **The warning leaves the exit code as the run sets it.**

**Downside:**

- **The test module reads `__debug__` as `True` under `-O`, and the code under test reads it as `False`.** A test that hands `__debug__` to the code under test fails where the code passes.
- **A helper of another module loses its asserts under `-O`.** A plain run gives the case `raised`, and the case passes under `-O`. The warning says so. No rule refuses it.
- **The warning comes above an error that stops the run, and in a run with no test.** No assert runs there.
- **An integrator that runs purba under `-O` and reads stderr as empty sees one line more.**

## Confirmation

`mise run test:wheel` runs `tests/optimize.bats`.
It checks that under `PYTHONOPTIMIZE=1` and `PYTHONOPTIMIZE=256` purba refuses a test whose helper in the test module asserts.
It checks that under `PYTHONOPTIMIZE=1` the test module reads `__debug__` as `True`.
It checks that under `PYTHONOPTIMIZE=2` the test module keeps its docstrings.
It checks that under `PYTHONOPTIMIZE=1` a helper of another module loses its asserts.

It checks that stderr holds the warning alone in each of these runs.
It checks that under `PYTHONOPTIMIZE=1` the warning comes before the line of a path that names no test file.
It checks that the warning comes before the line of an argument that is not UTF-8.
It checks that a plain run leaves stderr empty.
