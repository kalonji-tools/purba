# loguru `test_add_option_format.py` in `record_strict/`

All 20 pytest items port, as 44 cases: **44 pass, 0 fail, 0 refused**. ty: clean. pyright: one warning, on the fixture's `1 / 0`, copied from loguru. Size: 174 + 50 lines, against 87 + 30. A chained writer silently reads `''`, so each case reaches the writer through a process global.

| original | cases (steps) | assays |
|---|---|---|
| `test_format` ×6 | 12: `Added` → `Logged` | `product`, `written` |
| `test_progressive_format` | 11: `Added`, `Logged` ×8, `Opted` ×2, one chain | `product`, `written` |
| `…_without_exception` | 3: `Added` → `WithError` → `Logged` | `product`, `written` |
| `…_with_exception` | 3: `Added` → `WithError` → `LoggedLines` | `first_line`, `last_line` |
| `test_invalid_format` ×3 | 3 `Added` raises | – |
| `test_invalid_markups` ×5 | 5 `Added` raises | – |
| `test_markup_in_field` ×2 | 6: `Added` → `Bound` → `Logged` | `written` |
| `…_invalid_format_builtin` | 1 `Added` raises | – |

Levels: `Logged` is typed, because 17 cases share `written`. Both checkers caught my seeded typos. The rest are small: `Added` has only `product`; the others have one or two cases.

## Refusals and forced changes (probed outside `proto/`)

1. **The act is one call.** `add` and `debug` became two steps. `opt(exception=e).error(…)` was refused, so `opt` is a separate step.
2. **The act cannot `try`.** In an act, `logger.exception` logs `NoneType: None`. A fixture supplies a raised exception, which goes through `opt(exception=…)` and `log("ERROR")`.
3. **`splitlines()[0]` is refused as "not one read".** `next(iter(…))` and `next(reversed(…))` pass. The rule checks syntax, not logic.
4. **Each case must check something.** A `bind` or `opt` step expects `written`. An `add` expects the handler id, `product=0`.
5. **Two inputs from one step make it replay twice, with no refusal.** Logger first: passes. Writer first: reads `''`. Through a `bind` view, both orders read `''`. A writer built per case also reads `''`. The fix is `use(sink)`, one `Writer` per process. It bypasses "a case gets only what its chain names".
6. **A single-input chain cannot undo `bind`.** The progressive test passes `noend` as a capture keyword. It also needs an extra step, `cooked = opt(raw=False)`.
7. **`Experiment[Logger]` raises `NameError`** (a stub-only type): write `Experiment["Logger"]`.
8. **`test_invalid_format` needs wrong-typed inputs.** ty ignores `# type: ignore[arg-type]`. Only a bare `# type: ignore`, which covers the whole line, silences both checkers.
9. `Logged` replaces `.debug`, `.info` and `.exception` with `log(level, …)`.

## Fidelity gaps

**Less:** No case tests that `logger.exception` captures `sys.exc_info()`. `raises` matches a substring, so `^…$` is lost. The progressive test does not test `bind`. No case calls `debug`, `info` or `exception`. The teardown is gone, which the fork makes safe. The parse does not read the logic in inputs (lambdas, `progressive`, `F.__format__`).

**More:** 11 handler-id checks, 13 intermediate sink readings.

**Differently:** The writer is process-global. The traceback starts in the fixture. A refusal applies to its whole experiment: one seeded typo refused all 17 `Logged` cases, from four original tests.
