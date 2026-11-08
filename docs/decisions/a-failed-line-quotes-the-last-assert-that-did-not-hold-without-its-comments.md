# A failed line quotes the last assert that did not hold, without its comments

## Context and Problem Statement

[A tester runs purba on one module, and reads one line for each test](a-tester-runs-purba-on-one-module-and-reads-one-line-for-each-test.md#decision-outcome) has a failed line quote the text of its assert, on one line.
[The skeleton crosses the seam in execution and assertion](the-skeleton-crosses-the-seam-in-execution-and-assertion.md#decision-outcome) keeps the traceback in Python, so execution learns which assert did not hold only from assertion's hook.
A test can run an assert after the one that did not hold, in a `finally` block or in the exit of a context manager.
An assert can hold a comment or a line continuation.
ruff's tree holds neither.

## Considered Options

- **A failed line names the last assert that ran.** A cleanup assert that holds hides the assert that did not hold. The test then reads as `raised`. A run of only that test exits 4.
- **A failed line names the assert whose `AssertionError` the test raised.** Execution then needs the traceback, which stays in Python.
- **A failed line quotes the assert as the tester typed it, with its comments and line continuations.** On one line, the code after a `#` reads as part of the comment.

## Decision Outcome

**Reach:** `crates/assertion/src/lib.rs` `crates/parsing/src/lib.rs` `crates/execution/src/lib.rs` `crates/assertion/tests/rewrite.rs` `crates/execution/tests/run.rs` `tests/kinds.bats`

**Area:** assertion, execution, parsing

**A failed line quotes the last assert that did not hold, without its comments and line continuations.**
A cleanup assert that holds then leaves the failure on the assert that found it.
The hook keeps no traceback, so no new Python object crosses the seam.

- **An assert that holds never replaces an assert that did not hold.** A cleanup assert runs after the assert that found the failure, and the failed line names the assert that found it. A test fails when an assert that ran did not hold, and the test raised `AssertionError`. An `AssertionError` gives `raised` when no assert ran, or when each assert that ran held.
- **Parsing hands assertion the range of each comment and each line continuation.** Parsing is the one caller of ruff's parser. Assertion leaves those ranges out of the text of each assert. Reporting joins the text that remains.

```
test_cleanup_assert.py::test_with_a_finally_check failed: assert 1 + 1 == 3
test_shapes.py::test_comment failed: assert (1 and 0)
```

**Downside:**

- **A test that catches the `AssertionError` of an assert that did not hold, and then raises `AssertionError` with no assert, fails on the caught assert.** The test fails on the caught assert even when an assert after it held. The hook records which assert did not hold, and not which error the test raised.

## Confirmation

`mise run test:rust` checks that a cleanup assert that holds leaves the test failed on the assert that did not hold.
It checks that a cleanup assert that does not hold fails the test on itself.
It checks the Downside: a test fails on the caught assert, after an assert that held and an `AssertionError` with no assert.
It checks that the text of an assert leaves out its comment and its line continuation, and keeps a `#` inside a string.
`mise run test:wheel` checks that the line of a test that fails before a cleanup assert that holds reads `failed`, and that the run exits 1.

[Which assert does a failed line name, and in what text, when the assert that failed is not the last to run, or holds a comment?](https://github.com/kalonji-tools/purba/issues/848) weighed the options.
