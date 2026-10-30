# A blocked test has no outcome

## Context and Problem Statement

[An error stops only the work that depends on it](an-error-stops-only-the-work-that-depends-on-it.md#decision-outcome) says what each kind of error stops.
It does not say what a test that an error blocks prints, or how the count counts it.
A module that does not load blocks each of its tests, so one cause can count once for each test.

| fact | source |
|---|---|
| a module that raises while it loads gives its error to the first test, and `did not load` to each later test | [`run.rs`](https://github.com/kalonji-tools/purba/blob/e2e3c70/crates/execution/tests/run.rs#L102-L127) |
| each of the seven surveyed runners shows a module that fails to load as one item, and names none of its tests | [the research of test-runner failures](https://github.com/kalonji-tools/purba/blob/56d78a28c577c265e926eceb9ef510cfcb04ef55/research/test-runner-failures.md?plain=1#L53-L67) |
| selection selects a module's tests from the parse, before assertion loads the module | [`command`](https://github.com/kalonji-tools/purba/blob/e2e3c70/crates/command/src/lib.rs#L62-L73) |
| rustc's `ErrorGuaranteed` proves that the person was told, and cargo cuts an error that "does not add any new information" | [the research of error propagation](https://github.com/kalonji-tools/purba/blob/0c906fa23e9a4b8f87822ec8bf6abe5b3caf2b68/research/error-propagation.md?plain=1#L163-L168) |

## Considered Options

- **Each test carries the error.** Rejected. One cause shows once for each test. The count counts it each time.
- **One error, and its tests print nothing, as the surveyed runners do.** Rejected. The tester no longer reads one line for each test, and does not see what the error cost.
- **Each test that an error blocks has the outcome `unknown`.** Rejected. Its line tells the tester to fix the test, and the fix is where the error is.
- **One error, and each test that it blocks prints as blocked.** Chosen. The cause shows once. Selection already knows each test that the error cost.
- **Each test that an error blocks prints as `stopped`.** Rejected. The word reads as an interruption from outside the tester's code, such as Ctrl-C.
- **A test whose own error blocks it prints as blocked, and names that error.** Rejected. `blocked` then sends the tester to the test on one line, and to a fix outside the test on another.
- **A test whose own error blocks it prints that error, and counts as blocked.** Chosen. Its line says that the fix is in the test.

## Decision Outcome

**Reach:** `crates/**` `tests/**` `/CONTEXT.md`

**A blocked test has no outcome.**

- **A test that an error outside the test blocks prints as [blocked](../../CONTEXT.md#blocked-test), and names what blocked it.** A test whose own error blocks it prints that error, and counts as blocked.
- **The error shows once.**
- **The rule holds for every error that blocks a test, whatever its kind.**
- **The count of tests holds every selected test.** A blocked test has a column of its own, so the columns add up to the count.
- **The count of errors counts each cause once, separate from the count of tests.** An error that blocks no test counts too.
- **A blocked test does not change the exit code.** The error that blocked it sets the code.

Reporting chooses the words of this shape:

```
test_b.py project error: ValueError: x
test_b.py::test_one blocked: test_b.py did not load
test_b.py::test_two blocked: test_b.py did not load
2 tests: 2 blocked; 1 project error
```

**Downside:**

- **A test that a panic stops prints no line yet.** [`_main`](https://github.com/kalonji-tools/purba/blob/e2e3c70/crates/purba/src/lib.rs#L33-L38) catches a panic for the whole run.
- **The blocked column can count more tests than the `blocked` lines show.** It also counts each test whose line names the kind of an error.
- **A module of many tests prints a blocked line for each.** The surveyed runners print one line.

## Confirmation

`mise run test:rust` checks the lines and the count when a module does not load, and when a test's own error blocks it.
`mise run test:wheel` checks them on the installed `purba`.
[Wayfinder: purba's errors, telling each kind of failure apart from the first part that sees it](https://github.com/kalonji-tools/purba/issues/652) holds the rest.
