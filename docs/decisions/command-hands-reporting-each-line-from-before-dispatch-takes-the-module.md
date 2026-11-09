# Command hands reporting each line from before dispatch takes the module

## Context and Problem Statement

[Reporting writes each line as dispatch hands it the test](reporting-writes-each-line-as-dispatch-hands-it-the-test.md#decision-outcome), but dispatch never sees a line from before command hands it the module.
Command hands reporting those lines itself.
Other records decide each of those lines.

| fact | source |
|---|---|
| command hands reporting the line of a refused test, of a module with a syntax error, and of a module whose parse, selection or rewrite panics | [`crates/command/src/lib.rs`](https://github.com/kalonji-tools/purba/blob/0646ed0/crates/command/src/lib.rs#L80-L109) |
| after a panic in the rewrite, the line of each refused test prints, then the module's internal error, then each selected test as blocked | [`tests/fault.bats`](https://github.com/kalonji-tools/purba/blob/0646ed0/tests/fault.bats#L159-L176) |

## Considered Options

- **Dispatch hands on a module whose rewrite panics, as a new state of its `Module`.** Dispatch then carries a module that it never loads. Command still hands reporting the line of a module that the parse or selection stops.

## Decision Outcome

**Reach:** `crates/**` `/LANDSCAPE.md`

**Area:** reporting

**Command hands reporting each line from before it hands dispatch the module, and dispatch hands it each line from the load to the last test.**
The boundary holds for each step before that hand-off, so a new step needs no exception of its own.

| the line | the record that decides it |
|---|---|
| each test that selection refuses | [A test that selection refuses prints before any test runs](a-test-that-selection-refuses-prints-before-any-test-runs.md#decision-outcome) |
| a module with a syntax error | [Parsing hands on no tree that ruff recovered](parsing-hands-on-no-tree-that-ruff-recovered.md#decision-outcome) |
| a module whose parse, selection or rewrite panics, and each test that a panic in the rewrite blocks | [A panic inside a test gives that test an internal error](a-panic-inside-a-test-gives-that-test-an-internal-error.md#decision-outcome) |

**Downside:**

- **Two parts hand reporting the line of a module.** A reader who looks for one module's line looks in command and in dispatch.

## Confirmation

`mise run test:wheel` runs `tests/skeleton.bats`, which checks the line of a refused test and of a module with a syntax error.
It runs `tests/fault.bats`, which checks the lines after a panic in the parse, in selection and in the rewrite.
No command checks that dispatch never takes a module that the rewrite stops, so a reviewer reads it.
