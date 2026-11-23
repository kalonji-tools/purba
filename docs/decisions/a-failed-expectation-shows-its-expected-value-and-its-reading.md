# A failed expectation shows its expected value and its reading

## Context and Problem Statement

[A failed line quotes each expectation that did not hold, or else the last assert that did not hold, without its comments](a-failed-line-quotes-the-last-assert-that-did-not-hold-without-its-comments.md#decision-outcome) has a failed line quote the text of each expectation that did not hold.
The text holds the code and not the values.
A failed line loses the value that broke a check.
One side of each [expectation](../../CONTEXT.md#expectation) is a literal, or selection refuses its test.

[How does a test check a rule that has no literal value, such as a round trip or a property of every item?](https://github.com/kalonji-tools/purba/issues/1016#issuecomment-6913962314) decides that a failed expectation shows its [reading](../../CONTEXT.md#reading).
A flag joins the public surface only with a change that purba needs to work as a command, as [purba's public surface starts empty and has no plugin API](purbas-public-surface-starts-empty-and-has-no-plugin-api.md#decision-outcome) says.
No flag sets how much of a value shows.

## Considered Options

- **A failed line quotes the expectation alone.** The tester runs the test again with a `print` to see the value.
- **Each value shows its type, `[] (list)`, as [the prototype of the form](https://github.com/kalonji-tools/purba/blob/107b09a66bcdc7a4a8cba9cdd421beb96608e287/prototypes/1015-form/run.py#L1841) prints it.** Each line carries a type that the `repr` already shows.
- **The expected value shows alone for any operator, beside a line ``why  `3 < 2` does not hold``, as [the prototype of the form](https://github.com/kalonji-tools/purba/blob/107b09a66bcdc7a4a8cba9cdd421beb96608e287/prototypes/1015-form/run.py#L1830) prints it.** For `got < 2`, `expected  2` reads as the value that the reading must equal.
- **The failed line cuts a value to its head and its tail, as [the prototype of the form](https://github.com/kalonji-tools/purba/blob/107b09a66bcdc7a4a8cba9cdd421beb96608e287/prototypes/1015-form/run.py#L93) cuts it.** The cut can drop the item that broke the check. purba has no flag to show the value whole.

## Decision Outcome

**Reach:** `/CONTEXT.md` `crates/parsing/src/lib.rs` `crates/selection/src/lib.rs` `crates/assertion/src/rewrite.rs` `crates/assertion/src/load.rs` `crates/execution/src/lib.rs` `crates/errors/src/lib.rs` `crates/execution/src/zygote.rs` `crates/reporting/src/lib.rs` `crates/assertion/tests/load.rs` `crates/assertion/tests/rewrite.rs` `crates/assertion/tests/rewrite_property.rs` `crates/execution/tests/run.rs` `crates/errors/tests/tally.rs` `crates/dispatch/tests/hand.rs` `tests/kinds.bats` `tests/skeleton.bats` `tests/iterable.bats`

**Area:** assertion, execution, parsing, reporting

**Under each expectation that did not hold, a failed line shows its expected value and its reading.**
A failed `all()` or a failed `any()` shows a heading and item lines in place of the expected value and the reading.
Each takes a line of its own, two spaces in from the quote of its expectation.

- **The expected value of a comparison is the side that is a literal.** When both sides are literals, it is the right side.
- **Python's own operator compares the two sides.** Assertion hands the hook each side. The hook compares them by the operator of the expectation, so each verdict is the one that Python gives.
- **Each of the four other forms of an expectation names its expected value in words.** Assertion hands the hook the reading, and the type or the argument. Python decides each, by `not`, `isinstance`, `startswith` or `endswith`.
- **Each value shows as its `repr`, taken at the check, and only when the check fails.** A process that reads the expectation alone ends with its verdict before any `repr`.
- **When the two texts of a comparison are the same, each also shows the name of its class.** The reading of `isinstance` always shows its class, as `1 (int)`. Otherwise no type shows.
- **The `expected` line names the rule of the operator, from the side of the reading.**

  | expectation | `expected` line |
  |---|---|
  | `got == 2` | `2` |
  | `got != 2`, `got < 2`, `got in (0, 1)`, `got is None` | `!= 2`, `< 2`, `in (0, 1)`, `is None` |
  | `2 < got` | `> 2` |
  | `"z" in got`, `"z" not in got` | `to contain 'z'`, `not to contain 'z'` |
  | `not got` | `empty or false` |
  | `isinstance(got, str)` | `an instance of str` |
  | `got.startswith("ann")`, `got.endswith("lee")` | `to start with 'ann'`, `to end with 'lee'` |

- **A value shows whole, at any length.** No cut drops the item that broke the check.
- **A `repr` that raises leaves the case failed.** Its line reads `(repr raised ValueError: no repr)`.
- **A failed `all()` stops at the first item that breaks its check, as Python's own `all()` does.** It shows that item under a heading. A failed `any()` shows a heading, then a line for each item. Each line holds the index, then the text and the value of what the check reads, then the target and the item: `[2]  len(c)  1     c  'e'`. When the check reads the item itself, the line holds the item once: `[0]  '/usr/share'`. The heading reads `the first item that breaks  len(c) == 2`, or `none of 2 items holds  p == "/opt"` for an `any()`.
- **A failed marked assert, and a `with` whose entry or cleanup raised, show no reading.**

```
reading.py::test_contains failed
    assert "z" in got
      expected  to contain 'z'
      reading   'ab'
reading.py::test_same_text failed
    assert got == 1
      expected  1 (int)
      reading   1 (One)
reading.py::test_no_repr failed
    assert got == 1
      expected  1
      reading   (repr raised ValueError: no repr)
iterable.py::test_each failed
    assert all(len(c) == 2 for c in parts)
      the first item that breaks  len(c) == 2
      [2]  len(c)  1     c  'e'
```

**Downside:**

- **A failed `is not`, and a failed `!=` whose two values print alike, show the type of each value.** Their two texts are the same, so the line carries a type that says nothing new.
- **A long value fills one long line.**
- **A failed `any()` over many items prints a line for each item.**
- **A failed `all()` names only its first item that breaks.** Each run names one item that breaks, so three such items take three runs.
- **The `repr` of a value runs code of the test module after its check.** A `repr` that changes state changes it after the check read the value.

## Confirmation

`mise run test:rust` checks that the rewrite hands the hook each side of a comparison as the test writes it, for each operator.
It checks that the rewrite keeps the number of each later line.
It checks the expected value and the reading for each operator, and for a literal on either side.
It checks two texts that are the same, and a `repr` that raises.

It checks that Python's own operator gives the verdict.
An `__eq__` that gives a list holds.
A comparison that raises ends the call with its own raise.
It checks that what a failed expectation shows crosses the fork whole, even with a NUL in it.
A quote of a `with` crosses with no reading.
It checks that reporting prints the two lines under each quote that shows a reading, and none under a quote that shows none.

`mise run test:rust` checks that the items of a failed `any()` cross the fork whole. It checks that reporting prints the heading and the item under the quote of a failed `all()`. It checks in `crates/execution/tests/run.rs` that a failed `all()` takes the `repr` of its item after its reading closes. It checks there that an `any()` of no item fails. It checks that an `any()` that holds stops at its first true item. It checks that a failed `all()` stops at its first item that breaks.

`mise run test:wheel` checks that purba prints the two lines under each expectation that did not hold, in a module whose tests each fail.
It checks that the run exits 1.
`tests/iterable.bats` checks the lines of a failed `all()` and a failed `any()`.

[A failed expectation shows its reading](https://github.com/kalonji-tools/purba/issues/1024) built it.
