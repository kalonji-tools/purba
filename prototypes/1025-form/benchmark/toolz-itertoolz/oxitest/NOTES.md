# Port notes: toolz `test_itertoolz.py` (12 tests) → oxitest 4.0.0

The port stays a single flat file of 12 plain test functions, in the original
order and with the original bodies. Strict mode required one change: every
`assert` now carries a message. The docs also require that the five
module-level helpers get a `_` prefix. oxitest 4.0.0 does not enforce that
rule (see Faults), but the port follows it. Pyright found one statement in
`test_diff` that compares two values and discards the result. That statement
is now an `assert`, so the port checks one thing the original did not.

## What maps to what

| Original test | Port test | Cases | Asserts (orig → port) |
|---|---|---|---|
| `test_isdistinct` | `test_isdistinct` | 1 | 6 → 6 |
| `test_nth` | `test_nth` | 1 | 7 → 7 |
| `test_tail` | `test_tail` | 1 | 3 → 3 |
| `test_iterate` | `test_iterate` | 1 | 2 → 2 |
| `test_accumulate_works_on_consumable_iterables` | same name | 1 | 1 → 1 |
| `test_sliding_window_of_short_iterator` | same name | 1 | 2 → 2 |
| `test_partition_all` | `test_partition_all` | 1 | 7 → 7 |
| `test_count` | `test_count` | 1 | 5 → 5 |
| `test_diff` | `test_diff` | 1 | 12 → 13 |
| `test_topk` | `test_topk` | 1 | 6 → 6 |
| `test_peek` | `test_peek` | 1 | 3 → 3 |
| `test_peekn` | `test_peekn` | 1 | 4 → 4 |
| **12 tests** | **12 tests** | **12 items** | **58 → 59** |

No test is parametrized, so each test is one item. The original has no
parametrize either.

## Changes that oxitest forced

| Change | Rule that forced it |
|---|---|
| Every one of the 58 original `assert` statements gets a message string, for example `assert count([]) == 0, "count of an empty list"`. | Strict check **bare assert** (`explanation/strict-mode.md`). Probe: the original file under `strict = "abort"` was refused with `bare-assert` on all 12 tests and exit 3. |
| `identity`, `iseven`, `isodd`, `inc` and `double` are renamed to `_identity`, `_iseven`, `_isodd`, `_inc` and `_double`. The call sites in `test_iterate` change to match. | Strict check **module-level definition** in the docs: a module-level `def` in a `test_*.py` file must be a test, a fixture or `_`-prefixed. **oxitest 4.0.0 did not enforce this rule.** The docs require it, so the port follows it (see Faults). |

What did not need to change:

* `toolz.utils.raises` is kept. It is an import, not a module-level `def`. It
  returns a bool, so `assert raises(...), "msg"` keeps the original form.
  `oxitest.raises` exists, but no rule forces it.
* The classes `NoCompare` and `ListWithBadLength`, and the function
  `indollars`, are nested inside tests. The strict check reads module level
  only.
* `no_default2` and the unused imports (`starmap`, `partial`, `Random`, `mul`
  and many `toolz.itertoolz` names) stay. No strict check covers them, and
  neither type checker reported them.
* The code under test is `../toolz-src`. It is not installed, so the port puts
  it on the path with `PYTHONPATH=../toolz-src` (see `RUN.md`). It is not
  copied or changed.

## Changes that the type checkers forced

| Checker | Finding | Change |
|---|---|---|
| pyright 1.1.411 | `tests/test_itertoolz.py:176:5 - warning: Expression value is unused (reportUnusedExpression)`. This is the last statement of `test_diff`, `list(diff(data1, data2, key=indollars)) == [...]`. The original computes this comparison and throws the result away. | The statement becomes `assert ... , "diff with a key compares the converted values"`. It passes. A probe that changed `300` to `301` in the expected value made `test_diff` fail on that line, so the assert does check. |
| ty 0.0.99 | None. | None. |

Both checkers are clean on the final port (`CHECKERS.txt`). A control run
without `[tool.ty.environment] extra-paths` gave ty 2 diagnostics (unresolved
`toolz`). So the clean result comes from resolving `toolz`, not from ty
skipping the import.

## Faults of oxitest

1. **The documented "module-level definition" strict check does not fire.**
   `explanation/strict-mode.md` lists it as one of "the ten checks". Its
   example says `def build_order(**kwargs)` in `test_orders.py` "triggers".
   On oxitest 4.0.0 with `strict = "abort"`, these runs gave `passed`, exit 0
   and no violation:
   * the five unprefixed helpers from this port;
   * a control file that holds only the docs' own `build_order` example;
   * the same runs with `--strict=abort` and `--strict=enforce` on the CLI.

   The docs and the tool disagree. A user who reads the docs renames helpers
   for no enforced reason. A user who trusts the tool keeps helpers that the
   docs call violations.

## Where the port checks less, more or differently

* **More:** `test_diff` has one more check. The original's final
  `list(diff(..., key=indollars)) == [...]` was not asserted and checked
  nothing. The port asserts it (see Changes that the type checkers forced).
* **Differently:** every assertion now has a message. The message only changes
  the text shown on failure. It does not change the condition.
* **Less:** nothing. Every original assertion is kept with the same condition
  and the same operands. `toolz.utils.raises` keeps its exact semantics.
  It catches only the named exception type. Any other exception propagates
  and fails the test, as in the original.
