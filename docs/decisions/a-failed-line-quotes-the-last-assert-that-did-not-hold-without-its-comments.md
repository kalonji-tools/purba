# A failed line quotes each expectation that did not hold, or else the last assert that did not hold, without its comments

## Context and Problem Statement

[A tester runs purba on one module, and reads a line for each case](a-tester-runs-purba-on-one-module-and-reads-one-line-for-each-test.md#decision-outcome) has a failed line quote the text of its assert, on one line.
[The skeleton crosses the seam in execution, assertion and parsing](the-skeleton-crosses-the-seam-in-execution-and-assertion.md#decision-outcome) keeps the traceback in Python, so execution learns which assert did not hold only from assertion's hook.
A test can run an assert after the one that did not hold, in a `finally` block or in the exit of a context manager.
An assert can hold a comment or a line continuation.
ruff's tree holds neither.

[How does the experiment shape meet the walking skeleton?](https://github.com/kalonji-tools/purba/issues/968#issuecomment-6884183158) makes the asserts at a test's end its [expectations](../../CONTEXT.md#expectation).
purba reads each one, so more than one can fail in one case.

## Considered Options

- **A failed line names the last assert that ran.** A cleanup assert that holds hides the assert that did not hold. The test then reads as `raised`. A run of only that test exits 4.
- **A failed line names the assert whose `AssertionError` the test raised.** Execution then needs the traceback, which stays in Python.
- **A failed line quotes the assert as the tester typed it, with its comments and line continuations.** On one line, the code after a `#` reads as part of the comment.
- **A failed line quotes the last assert that did not hold, an expectation too.** A fix of that one expectation leaves the tester to find the next one on the next run.

## Decision Outcome

**Reach:** `crates/assertion/src/rewrite.rs` `crates/assertion/src/load.rs` `crates/parsing/src/lib.rs` `crates/execution/src/lib.rs` `crates/errors/src/lib.rs` `crates/reporting/src/lib.rs` `crates/assertion/tests/rewrite.rs` `crates/execution/tests/run.rs` `tests/kinds.bats` `tests/rules.bats`

**Area:** assertion, execution, parsing, reporting

**A failed line quotes each expectation that did not hold, in order, or else the last marked assert that did not hold.**
Each quote takes a line of its own, without its comments and line continuations.
purba reads every expectation, so the tester sees each one that a fix must reach.
A cleanup assert that holds leaves the failure on the assert that found it.
A `with` whose entry or cleanup raised what ends the call fails the case, and its line names the `with` and what it raised.
The hook keeps no traceback, so no new Python object crosses the seam.

- **An expectation that did not hold fails the case after the test returns.** purba reads each later expectation too, and quotes each one that did not hold.
- **A `try` of one path whose statement did not raise comes before each expectation.** Its quote reads `the try at line 38 expects ValueError, and the statement did not raise it`.
- **The message of an expectation never runs.** The line quotes it, as part of the text of the assert.
- **A marked assert runs only in the cleanup of a `with`.** Execution refuses one that runs before the first expectation outside such a cleanup, and one inside a reading, as [a test returns nothing, and the first part that sees a breach of a test refuses it](a-test-returns-nothing-and-the-first-part-that-sees-a-breach-refuses-it.md#decision-outcome) says.
- **A raise from the entry or the cleanup of a `with` of the test fails the case when it ends the call.** Its line reads `the cleanup of the with at line 47 raised`, then the exception and its message. When the test catches the raise, the expectations decide the case.
- **A marked assert that holds never replaces a marked assert that did not hold.** A cleanup assert runs after the assert that found the failure, and the failed line names the assert that found it. A case fails when a marked assert that ran did not hold, and the test raised `AssertionError`. An `AssertionError` gives `raised` when no marked assert ran, or when each one that ran held.
- **Parsing hands assertion the range of each comment and each line continuation.** Parsing is the one caller of ruff's parser. Assertion leaves those ranges out of the text of each assert. Reporting joins the text that remains.

```
test_two_fail.py::test_one failed
    assert a == 1
      expected  1
      reading   2
    assert b == 2
      expected  2
      reading   3
test_m.py::test_cleanup failed
    the cleanup of the with at line 47 raised AssertionError: a handle leaked
test_shapes.py::test_comment failed
    assert (1 and 0)
```

[A failed expectation shows its expected value and its reading](a-failed-expectation-shows-its-expected-value-and-its-reading.md#decision-outcome) says what shows under each quote of an expectation.

**Downside:**

- **Execution refuses a case when a helper asserts inside a reading.** It refuses the case even when the assert holds. A tester guards the input of such a helper with a raise.

## Confirmation

`mise run test:rust` checks that an expectation inside a `with` whose cleanup asserts fails the case on itself.
It checks that execution refuses an assert in a `try` before the first expectation.
It checks that execution refuses an assert inside a reading, whether that assert holds or not.
It checks that a `with` whose cleanup raises fails the case, and that its line names the `with`.
It checks that the text of an assert leaves out its comment and its line continuation, and keeps a `#` inside a string.
It checks that the message of an expectation never runs.

`mise run test:wheel` checks that the line of an expectation that fails inside a `with` whose cleanup asserts reads `failed`, and that the run exits 1.
It checks that a test whose two expectations fail prints both under its line.
It checks that a `with` whose entry or cleanup raised prints its line.

[Which assert does a failed line name, and in what text, when the assert that failed is not the last to run, or holds a comment?](https://github.com/kalonji-tools/purba/issues/848) weighed the options.
