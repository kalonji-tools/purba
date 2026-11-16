# Notes on the port of loguru's `test_add_option_format.py`

The port passed on the runner's first try. The runner refused nothing: 20 cases passed and 21 checks held. The form forced two kinds of change. First, the two conftest fixtures became `@before(reset_logger)` and a `Writer` fixture. Second, the bare `pytest.raises(TypeError)` got a `match=`. The type checkers forced two `cast(Any, …)` calls and one explicit submodule import.

The original has 8 test functions and 20 items. The brief says 9 tests, but I count 8 `def test_…` in the original.

## What maps to what

| Original test | Port test | Cases |
|---|---|---|
| `test_format[a..f]` (6 items) | `test_format` | `format_a`, `format_b`, `format_c`, `format_d`, `format_e`, `format_f` |
| `test_progressive_format` | `test_progressive_format` | runs once under its own name |
| `test_function_format_without_exception` | `test_function_format_without_exception` | runs once under its own name |
| `test_function_format_with_exception` | `test_function_format_with_exception` | runs once under its own name |
| `test_invalid_format[-1, 3.4, object()]` (3 items) | `test_invalid_format` | `invalid_format_int`, `invalid_format_float`, `invalid_format_object` |
| `test_invalid_markups[5 markups]` (5 items) | `test_invalid_markups` | `invalid_markups_open` (`<red>`), `invalid_markups_close` (`</red>`), `invalid_markups_crossed` (`</level><level>`), `invalid_markups_bare_close` (`</>`), `invalid_markups_unknown` (`<foobar>`) |
| `test_markup_in_field[True, False]` (2 items) | `test_markup_in_field` | `markup_in_field_colorized`, `markup_in_field_plain` |
| `test_invalid_format_builtin` | `test_invalid_format_builtin` | runs once under its own name |
| conftest `reset_logger` (autouse) | `reset_logger`, put on each test with `@before` | — |
| conftest `writer` | the `Writer` class, taken as `writer: Writer = use(Writer)` | — |

Totals: 8 tests, 20 items, 20 cases, 21 checks (12 from the 6 assert statements, run over their cases, plus 9 expected raises).

## Changes that the form forced

1. **The autouse `reset_logger` fixture became a plain function under `@before(reset_logger)` on each of the 8 tests.** Rule: "`@before(fn)` on a test runs `fn()` at the start of each of its cases … Use it to reset global state; nothing runs after a case, as its process ends." The form has no autouse fixture and no teardown. So the reset after the test was dropped, because each case's process ends after it.
2. **The `writer` fixture became a `Writer` class with `__call__` and `read`.** Rule: "Share a setup through a fixture: the default of an input, `use(factory)`." The original adds attributes to a nested function (`w.written`, `w.read`). A class gives the same sink, and the type checkers can read it. The unused `clear` was dropped.
3. **`pytest.raises(TypeError)` in `test_invalid_format` got `match=r"^Invalid format, it should be a string or a function, not: "`.** Rule: "A raise or a warning is not checked by its type alone. Give `match=`, or bind `as caught` and check `caught.value`; `with raises(Closed):` alone is refused." The match is the part of loguru's message that does not depend on the case (`_logger.py:1029`).
4. **Each parametrize item became a named `case(…)` at module level.** Rule: "A case is `name = case(test, input=…, …)` at module level, named by its variable." The names are mine. pytest named them by id (`a`..`f`, `-1`, `<red>`, `True`).
5. **The `format` input of `test_invalid_format` is declared `object`.** Rule: "To pass a wrong type on purpose, declare the input `object`."
6. **`1 / 0  # noqa: B018` became `_ = 1 / 0`,** as in FORM.md's `try` example. The `try` already had the one shape the form allows: one statement and one `except ZeroDivisionError:`.
7. **Imports changed from `pytest` to `cases`** (`before`, `case`, `raises`, `use`).

These stayed as in the original, and the runner accepted each one:
- The `if` in `formatter` and `F.__format__`. loguru calls both, so their paths are free.
- The `for _ in range(5)` loop.
- `lines = writer.read().splitlines()` before the two checks.

## Changes that the type checkers forced

The annotations are new. The original had none. These changes were made after the first checker run (CHECKERS.txt has the final, clean run).

1. **`import loguru._logger` was added.** ty warned `possibly-missing-submodule` on `loguru._logger`. pyright said `"_logger" is not a known attribute of module "loguru"`, because loguru's `__init__.pyi` stub does not declare it.
2. **`loguru.logger.__init__(…)` became `cast(Any, loguru.logger).__init__(…)`.** The stub types `logger` as `Logger` and gives it no constructor. ty reported "expected 1, got 11", and pyright reported "Expected 0 positional arguments". The reset calls the real constructor, as conftest does.
3. **`logger.add(writer, format=format)` in `test_invalid_format` became `format=cast(Any, format)`.** Both checkers refused `object` for `format: str | FormatFunction`. That wrong type is what the test is about.
4. **`Record` is imported under `if TYPE_CHECKING:`.** It exists only in the stub, so the `format` and `formatter` annotations can name it. This was my choice, not a fault that a checker reported.

## Faults of the runner

None hit.

## Where the port checks less, more or differently

- **More:** `test_invalid_format` now also checks the start of the `TypeError` message. The original checked only the type.
- **More:** each expected raise is shown and counted as a check (`✓ … › raises`). pytest counts none of them. That is why there are 21 checks: 12 from asserts and 9 from raises.
- **Less:** the logger is not reset after each case. Only the case's own process could see that state, and the process ends after the case.
- **Differently:** the `format` input of `test_format` names `Record`, a type-only import. FORM.md says the runner checks an input's type "when it can read the annotation". The runner did not say whether it checked this one. All 6 cases passed.
- **Probe (not kept):** I ran a copy with `format_b` expecting `"Nope"` and the `TypeError` match changed to `…not: XX`. The runner failed `format_b` and the 3 `invalid_format` cases, as it should. So the checks and the matches can fail.
