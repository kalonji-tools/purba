# Notes on the port

The 12 original tests became 16 tests of the form and 24 cases, with 71 checks; all 24 cases pass, and
the runner refused none of them on the first run. Every value check of the original kept its own
`assert`, mostly word for word. What changed is the shape around the 12 expected raises (each became a
case of a raise test, because `with raises(…)` ends the act), a few expected values that the original
computed, and two iterators that the original checked directly.

## What maps to what

| Original test | Port test | Cases | Checks |
|---|---|---|---|
| `test_isdistinct` | `test_isdistinct` | `test_isdistinct` | 6 |
| `test_nth` (5 value asserts) | `test_nth` | `test_nth` | 5 |
| `test_nth` (2 `raises` asserts) | `test_nth_raises` | `nth_past_the_end`, `nth_negative_of_iterator` | 2 × 2 |
| `test_tail` | `test_tail` | `test_tail` | 3 |
| `test_iterate` | `test_iterate` | `test_iterate` | 2 |
| `test_accumulate_works_on_consumable_iterables` | same name | same name | 1 |
| `test_sliding_window_of_short_iterator` | same name | same name | 2 |
| `test_partition_all` (5 value asserts) | `test_partition_all` | `test_partition_all` | 5 |
| `test_partition_all` (2 `raises` asserts, issue 602) | `test_partition_all_of_invalid_length` | `too_long_list`, `too_short_list` | 2 × 2 |
| `test_count` | `test_count` | `test_count` | 5 |
| `test_diff` (5 value asserts + 1 bare comparison) | `test_diff` | `test_diff` | 6 |
| `test_diff` (7 `raises` asserts) | `test_diff_raises` | `diff_of_no_sequence`, `diff_of_one_sequence`, `diff_of_a_sequence_and_a_number`, `diff_of_an_empty_list`, `diff_of_a_list_of_one_empty_sequence`, `diff_of_a_list_of_one_sequence`, `diff_of_a_list_of_a_sequence_and_a_number` | 7 × 2 |
| `test_topk` | `test_topk` | `test_topk` | 6 |
| `test_peek` (2 value asserts) | `test_peek` | `test_peek` | 2 |
| `test_peek` (1 `raises` assert) | `test_peek_of_empty` | `test_peek_of_empty` | 2 |
| `test_peekn` | `test_peekn` | `test_peekn` | 4 |

Each raise case has two checks: the runner's `raises` line, and `str(caught.value) == message`.

## Changes that the form forced

1. **Each `assert raises(E, lambda: …)` became a case of a raise test** (12 raises → 4 tests, 12 cases). Rules: "`with raises(…)` … ends the act: a line of the act that runs after it caught is refused", and "Any other assert is refused: … any other call". A probe of `assert toolz.utils.raises(StopIteration, lambda: …)` was refused with "a check compares a reading with the value you expect, with one operator".
2. **Each raise also checks its message** (`assert str(caught.value) == message`). Rule: "A raise or a warning is not checked by its type alone". A probe of `with raises(StopIteration):` was refused with "checks the type alone". The messages came from running the code under test. For `StopIteration` the message is `""`.
3. **`test_nth_raises` takes the exception type as an input** (`error: type[Exception]`), because its two cases raise `StopIteration` and `ValueError`. `test_diff_raises` takes the arguments as a tuple (`seqs`) and calls `diff(*seqs)`. This let one test cover the raises of one original test. The rule is the same as in item 1.
4. **Computed expected values became literals.** In `test_tail`, `list('CDE')` became `["C", "D", "E"]` and `list((2, 1))` became `[2, 1]`. In `test_peek`, `alist[0]` became `"Alice"` and `alist` became the list literal. In `test_peekn`, `alist[:2]` became `("Alice", "Bob")` and `alist` became the tuple literal. Rule: "A computed expected value is refused". Probes of the original `test_tail` and `test_peek` were refused with "the expected value computes `list('CDE')`", "`alist[0]`" and "`alist`".
5. **`test_partition_all`: `obj` became a fixture input** (`obj: NoCompare = use(NoCompare)`), and `NoCompare` moved to module level. The local name `result` became the inline list `[(obj, obj, obj, obj), (obj, obj, obj)]`. Rule: "One side is the expected value: a literal, … an input of the test, or a list, tuple, set or dict of these". A local `obj` or `result` is a computed expected value.
6. **`peek` and `peekn` return an iterator; the port makes it a list or tuple in the act** (`whole = list(blist)`) and checks that. Rule: "A check never reads an iterator that one reading empties". A probe of `assert list(blist) == [...]` was refused with "line 28 reads blist, a chain that one reading empties".
7. **`test_peekn` calls `peekn` twice in the act, under two sets of names** (`elements, blist` and `all_elements, blist_after_all`). The original re-binds `elements, blist` after its first two asserts. Rule: "The act ends at the first check, and only checks follow it". A probe of the original shape was refused with "an assert ran in the act".
8. **`test_diff`: `data1`, `data2`, `conversions` and `indollars` moved above the first check.** In the original they come after the asserts. The rule is the same as in item 7.
9. **Unused module names were dropped:** `no_default2`, `identity`, `iseven`, `isodd`, and the unused imports (`starmap`, `partial`, `Random`, `pickle`, `mul`, `toolz.utils.raises`, and 25 of the 38 `toolz.itertoolz` names). No rule forced this. The form says to import "only what a test needs".
10. **Each test got a `-> None` return annotation, and each input of a raise test got an annotation.** The runner reads the annotations to check input types ("the runner checks each input's type when it can read the annotation"). The helpers `inc`, `double`, `NoCompare.__eq__`, `indollars` and `ListWithBadLength` stay as in the original, without annotations.

## Changes that the type checkers forced

None. `ty check` (ty 0.0.99) and `pyright` (1.1.411) both report no errors on the port (`CHECKERS.txt`). A control file with `case(test_nth_raises, n="x", …)` was flagged by both checkers. This shows that both resolve `cases.py`, and that both read a `case(…)` call against the test's parameters.

## Faults of the runner

None hit by the port. Three things that the probes showed, outside the port:

- **A type-only raise check gets past the runner through a helper of the code under test.** `stops = toolz.utils.raises(StopIteration, lambda: nth(10, {10: "foo"}))` in the act, then `assert stops is True`, ran and passed. This checks the type alone, which the form refuses for its own `raises`. The `try` is in toolz's code, which the runner does not watch, so this follows the written rules. The port does not use it.
- **An assert in the act is reported under the label `path`:** `! test_peekn_original › path … an assert ran in the act at test_probe.py:34`. The text is correct; the label names another rule (a path of the act that a case did not take).
- **A test with no act passes.** The original `test_isdistinct` shape, with each call to the code under test inside the check, ran unchanged. This matches FORM.md's example `assert list(numbers(text)) == [1, 2]`. The port keeps this shape wherever the original had it.

## Where the port checks less, more or differently

- **More: each of the 12 raises also checks its message.** The original checks only the type (`toolz.utils.raises` returns `True` on any instance of the type). If toolz changes a message, for example `'int' object is not iterable` in a future CPython, or the misspelled `parition_all` in the `LookupError`, the port fails and the original does not.
- **More: the last comparison of `test_diff` is a check.** In the original it is a bare expression (`list(diff(data1, data2, key=indollars)) == [...]` with no `assert`), so it checks nothing. The port asserts it, and it holds.
- **Differently: `==` in the form compares by type, then by value.** `nth(1, (3, 2, 1)) == 2` and `count(...) == 3` must give an `int`, and `topk(...)` must give a `tuple`, not a list. The original `==` would accept `2.0` or a list of the same items. All held.
- **Differently: in `test_partition_all`, one `NoCompare` object comes from a fixture,** not a local. The checks compare the same object, so the regression check (no compare against a sentinel, which would raise `ValueError`) is unchanged.
- **Differently: `test_peek` and `test_peekn` compare with literals,** not with `alist`. The literals equal `alist`, so the result is the same unless `peek` mutates `alist`. Neither test checked that before, and the port does not check it either.
- **Same: each case runs in its own process.** The raise cases each start from a new process, but the original raise lambdas shared no state, so this does not change what is checked.
