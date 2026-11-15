# Notes on the port

## Map

| Original test (pytest) | Port test | Cases |
|---|---|---|
| `test_format` (6 parametrized) | `test_format` | `message_field`, `no_field`, `level_twice`, `level_attributes`, `function_without_newline`, `function_reads_the_record` |
| `test_progressive_format` | `test_progressive_format` | runs once under its own name |
| `test_function_format_without_exception` | `test_function_format_without_exception` | runs once under its own name |
| `test_function_format_with_exception` | `test_function_format_with_exception` | runs once under its own name |
| `test_invalid_format` (3 parametrized) | `test_invalid_format` | `format_int` (-1), `format_float` (3.4), `format_object` (`object()`) |
| `test_invalid_markups` (5 parametrized) | `test_invalid_markups` | `open_tag_alone` (`<red>`), `close_tag_alone` (`</red>`), `close_before_open` (`</level><level>`), `bare_close_tag_alone` (`</>`), `unknown_tag` (`<foobar>`) |
| `test_markup_in_field` (2 parametrized) | `test_markup_in_field` | `colorized` (True), `not_colorized` (False) |
| `test_invalid_format_builtin` | `test_invalid_format_builtin` | runs once under its own name |
| conftest fixture `reset_logger` (autouse) | `reset_logger`, given to each test by `@before(reset_logger)` | - |
| conftest fixture `writer` | class `Writer`, taken as `writer: Writer = use(Writer)` | - |

The original has 20 parametrized runs. The port has 20 cases. All 20 pass.

## Changes the form forced

- The `try` / `except ZeroDivisionError` in the two exception tests became `with handling(ZeroDivisionError, logger.exception, "Error!"):`. An `except` in the act always goes one way, and the form refuses that.
- The autouse fixture `reset_logger` became `@before(reset_logger)` on each of the 8 tests. The form has no autouse fixture.
- The original resets the logger before and after each test. The port resets it only before. The form runs nothing after a case, and each case runs in its own process.
- The `writer` fixture was a function with attributes. It became the class `Writer`, because `use(…)` takes a factory. The class keeps `written` and `read()`. It drops `clear()`, which no test in this file uses.
- Each parametrize id became a named module-level case. The names are new; pytest showed the values.
- In `test_invalid_format`, `test_invalid_markups` and `test_markup_in_field`, the input `writer` moved after the parametrized input. An input with a fixture default must come after an input with no default.
- `Writer` and `reset_logger` sit in the test module. The rules sheet says a function of the test module that the code under test calls (a sink) is a value, and the examples keep fixtures in the test module. The code under test, loguru, sits in the venv, outside the tests' folder.

## Changes the type checkers forced

- Each input has a type annotation. `format` in `test_format` is `str | FormatFunction`. `FormatFunction` and `Record` exist only in loguru's stub, so they are imported under `TYPE_CHECKING`. The runner cannot evaluate that annotation, so it does not check the type of that input at run time.
- `test_invalid_format` passes wrong types on purpose. Its input is declared `object`, as the rules sheet says. The call `logger.add(writer, format=format)` then fails both checkers, so it carries a bare `# type: ignore`. ty did not honour `# type: ignore[arg-type]`.
- `reset_logger` calls `logger.__init__` with 10 arguments. loguru's stub declares no `__init__` for `Logger`, so both checkers refuse the call. It carries a bare `# type: ignore`. This is conftest code, not a change of the form.
- `reset_logger` imports `Core` and `context` from `loguru._logger`. The original reads them as `loguru._logger.Core`, and ty warned that the submodule might not be imported.
- `1 / 0` became `_ = 1 / 0`. A bare expression statement is a checker warning (the original carries `# noqa: B018`).
- The nested `formatter` in `test_progressive_format` and `F.__format__` in `test_markup_in_field` have annotations. Their code is the same.

## Faults of the runner

- None hit. The first run passed 20 of 20, and nothing was refused.

## Where the port checks differently

- The `handling` block is reported as a check of its own. It fails the case if `1 / 0` does not raise `ZeroDivisionError`. In the original, no raise means `logger.exception` never runs, and the assert fails instead. Both fail; the port names the cause.
- Each `raises` is reported as a check. `pytest.raises` was not an assert, but it failed the test the same way. `raises(..., match=…)` uses `re.search`, as `pytest.raises` does.
- `==` in a check compares type, then value. The original's `==` compared value only. Every reading here is a `str`, and every expected value is a `str`, so no check changes.
- The report counts 23 checks over 20 cases. The original has 6 asserts in 5 tests, and 3 `pytest.raises` blocks in 3 tests.
