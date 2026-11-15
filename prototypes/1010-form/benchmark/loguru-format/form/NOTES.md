# Notes on the port

## Map

| Original test (pytest id) | Port test | Port cases |
|---|---|---|
| `test_format[a-…]` | `test_format` | `message_field` |
| `test_format[b-…]` | `test_format` | `no_field` |
| `test_format[c-…]` | `test_format` | `level_twice` |
| `test_format[d-…]` | `test_format` | `level_attributes` |
| `test_format[e-<lambda>-e]` | `test_format` | `function_format` |
| `test_format[f-<lambda>-f DEBUG]` | `test_format` | `function_reads_record` |
| `test_progressive_format` | `test_progressive_format` | one run, under its own name |
| `test_function_format_without_exception` | `test_function_format_without_exception` | one run |
| `test_function_format_with_exception` | `test_function_format_with_exception` | one run |
| `test_invalid_format[-1]` | `test_invalid_format` | `negative_int` |
| `test_invalid_format[3.4]` | `test_invalid_format` | `float_number` |
| `test_invalid_format[format2]` | `test_invalid_format` | `bare_object` |
| `test_invalid_markups[<red>]` | `test_invalid_markups` | `unclosed_tag` |
| `test_invalid_markups[</red>]` | `test_invalid_markups` | `unopened_tag` |
| `test_invalid_markups[</level><level>]` | `test_invalid_markups` | `closed_before_opened` |
| `test_invalid_markups[</>]` | `test_invalid_markups` | `bare_closing_tag` |
| `test_invalid_markups[<foobar>]` | `test_invalid_markups` | `unknown_tag` |
| `test_markup_in_field[True]` | `test_markup_in_field` | `colorized` |
| `test_markup_in_field[False]` | `test_markup_in_field` | `not_colorized` |
| `test_invalid_format_builtin` | `test_invalid_format_builtin` | one run |

20 original cases, 20 port cases.

## Changes the form forced

- `@pytest.mark.parametrize` became one module-level `case(…)` per row. Each case has a name, so the pytest ids (`a`, `-1`, `format2`, …) became words.
- The `writer` and `reset_logger` fixtures became `use(Writer)` and `use(reset_logger)`, the defaults of two inputs of each test. The form has no autouse fixture, so each test names `_reset` in its signature.
- `reset_logger` runs only before a case. The form runs nothing after a case, because the case's process ends.
- The `try: 1 / 0` / `except ZeroDivisionError: logger.exception("Error!")` blocks became `with handling(ZeroDivisionError, logger.exception, "Error!"): _ = 1 / 0`. The form allows no `except` that goes one way only.
- `test_invalid_format` declares its input `format: object`, as the form asks for a value of the wrong type on purpose.
- The stand-in sink `Writer` and the fixture `reset_logger` sit in the test module. The rules sheet says that a sink of the test module that the code under test calls is a value, and its examples keep fixtures in the test module.

## Changes the type checkers forced

- The `writer` fixture was a function with `written`, `read` and `clear` set as attributes. Both checkers refuse those attributes (9 errors on the original `conftest.py`). The port uses a class `Writer` with `__call__` and `read`. loguru still takes it as a callable sink, because it has no `write` method.
- `Writer.clear` is not ported. No test of this file calls it.
- The original reset calls `loguru.logger.__init__(loguru._logger.Core(), …)` and `loguru._logger.context.set({})`. Both checkers flag those private calls. The port's reset calls only `logger.remove()`.
- `logger.add(writer, format=format)` in `test_invalid_format` takes an `object`, which the stub of `logger.add` refuses. The line carries a bare `# type: ignore`. ty 0.0.99 still reported `no-matching-overload` with `# type: ignore[arg-type]`.
- `1 / 0` became `_ = 1 / 0`. pyright warned `reportUnusedExpression` on the bare expression in the original.
- The port adds type annotations. `Message`, `Record` and `FormatFunction` exist only in loguru's stub, so the port imports them under `TYPE_CHECKING`. The runner cannot read those names at run time, so it does not check the type of the `format` input of `test_format`.

## Faults of the runner

- None hit. Each case passed on the first run with no refusal.

## Where the port checks less, more or differently

- Less: the port's reset does not rebuild the logger's core and does not clear its context. Each case runs in a forked process of its own, and no earlier case can change that state there.
- More: `handling` fails the case when `1 / 0` does not raise. The report also prints the caught `ZeroDivisionError` as a line of its own, and prints each caught raise of `raises`. The run reports 23 checks for 20 cases.
- Same: `raises(…, match=…)` uses `re.search`, as `pytest.raises` does.
- A mutated copy (one expected value changed, one invalid format made valid) failed 2 cases, so the checks can fail.
