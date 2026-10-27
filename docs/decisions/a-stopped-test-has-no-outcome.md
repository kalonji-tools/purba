# A stopped test has no outcome

## Context and Problem Statement

[An error stops only the work that depends on it](an-error-stops-only-the-work-that-depends-on-it.md#decision-outcome) says what each kind of error stops.
It does not say what a test that an error stops prints, or how the count counts it.
A module that does not load stops each of its tests, so one cause can count once for each test.

| fact | source |
|---|---|
| a module that raises while it loads gives its error to the first test, and `did not load` to each later test | [`run.rs`](https://github.com/kalonji-tools/purba/blob/e2e3c70/crates/execution/tests/run.rs#L102-L127) |
| each of the seven surveyed runners shows a module that fails to load as one item, and names none of its tests | [the research of test-runner failures](https://github.com/kalonji-tools/purba/blob/56d78a28c577c265e926eceb9ef510cfcb04ef55/research/test-runner-failures.md?plain=1#L53-L67) |
| selection selects a module's tests from the parse, before assertion loads the module | [`command`](https://github.com/kalonji-tools/purba/blob/e2e3c70/crates/command/src/lib.rs#L62-L73) |
| rustc's `ErrorGuaranteed` proves that the person was told, and cargo cuts an error that "does not add any new information" | [the research of error propagation](https://github.com/kalonji-tools/purba/blob/0c906fa23e9a4b8f87822ec8bf6abe5b3caf2b68/research/error-propagation.md?plain=1#L163-L168) |

## Considered Options

- **Each test carries the error, as purba does today.** Rejected. One cause shows once for each test. The count counts it each time.
- **One error, and its tests print nothing, as the surveyed runners do.** Rejected. The tester no longer reads one line for each test, and does not see what the error cost.
- **Each stopped test has the outcome `unknown`.** Rejected. Its line tells the tester to fix the test, and the fix is where the error is.
- **One error, and each test that it stops prints as stopped.** Chosen. The cause shows once. Selection already knows each test that the error cost.

## Decision Outcome

**Reach:** `crates/**` `tests/**` `/CONTEXT.md`

**A stopped test has no outcome.**

- **A test that an error stops prints as [stopped](../../CONTEXT.md#stopped-test), and names what stopped it.**
- **The error shows once.**
- **The rule holds for every error that stops a test, whatever its kind.**
- **The count of tests holds every selected test.** A stopped test has a column of its own, so the columns add up to the count.
- **The count of errors counts each cause once, separate from the count of tests.** An error that stops no test counts too.
- **A stopped test does not change the exit code.** The error that stopped it sets the code.

Reporting chooses the words of this shape:

```
test_b.py error: ValueError: x
test_b.py::test_one stopped: test_b.py did not load
test_b.py::test_two stopped: test_b.py did not load
2 tests: 0 passed, 0 failed, 0 unknown, 2 stopped; 1 error
```

**Downside:**

- **purba does not print a stopped test yet.**
- **A test that a panic stops prints no line yet.** [`_main`](https://github.com/kalonji-tools/purba/blob/e2e3c70/crates/purba/src/lib.rs#L33-L38) catches a panic for the whole run.
- **The records that hold the count still count an error for each test.** [A module that did not load prints its error once for each of its tests](https://github.com/kalonji-tools/purba/issues/681) rewrites [the tester record](a-tester-runs-purba-on-one-module-and-reads-one-line-for-each-test.md#decision-outcome) and [the reporting record](reporting-renders-the-exit-code-and-command-returns-it.md#decision-outcome).
- **A module of many tests prints a stopped line for each.** The surveyed runners print one line.

## Confirmation

No command checks this record.
[A module that did not load prints its error once for each of its tests](https://github.com/kalonji-tools/purba/issues/681) builds it for a module that does not load.
[Wayfinder: purba's errors, telling each kind of failure apart from the first part that sees it](https://github.com/kalonji-tools/purba/issues/652) holds the rest.
