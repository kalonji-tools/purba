# Port notes: loguru's test_add_option_format.py

All 20 pytest items are ported as 20 cases, and all pass. The runner, ty and pyright report nothing on `tests/`. The port makes the same checks as the original, with two exceptions. First, the form forced `try`/`except` into `with handling(…)`. That changes the traceback loguru logs, but the port's checks do not read that part. *Fixed in the form after the port: `handling` no longer changes the traceback (see "Differently" below).* Second, the form adds two small checks of its own: `handling` reports that `1 / 0` raised. The runner refused nothing during the port, because I wrote to FORM.md from the start. The type checkers forced three changes: the writer became a class, the reset needs one ignore, and `1 / 0` became `_ = 1 / 0`.

## Original test → cases → checks

| Original test (pytest items) | Cases in the port | Checks per case |
|---|---|---|
| `test_format` (6, parametrized) | `message_field`, `no_field`, `level_twice`, `level_attributes`, `function_format`, `function_reads_record` | `writer.read() == expected` |
| `test_progressive_format` (1) | runs once under its own name | `writer.read() == "[DEBUG] Start: .....\n[DEBUG] End\n"` |
| `test_function_format_without_exception` (1) | runs once under its own name | `handling` (the statement raised ZeroDivisionError); `writer.read() == "Error!\n"` |
| `test_function_format_with_exception` (1) | runs once under its own name | `handling`; `lines[0] == "Error!"`; `lines[-1] == "ZeroDivisionError: division by zero"` |
| `test_invalid_format` (3, parametrized) | `format_int` (-1), `format_float` (3.4), `format_object` (`object()`) | `raises(TypeError)`, with no match, as the original |
| `test_invalid_markups` (5, parametrized) | `open_red_only`, `close_red_only`, `close_before_open`, `close_any_only`, `unknown_tag` | `raises(ValueError, match=r"^Invalid format, color markups could not be parsed correctly$")` |
| `test_markup_in_field` (2, parametrized) | `colorized`, `not_colorized` | `writer.read() == "</>  <blue>  Test\n"` |
| `test_invalid_format_builtin` (1) | runs once under its own name | `raises(ValueError, match=r".* most likely a mistake")` |

Totals: 20 cases. 23 checks held: the original's 21 (12 asserts + 9 raises), plus 2 `handling` lines.

## Refusals and forced changes

| What the original does | What forced a change | What the port does instead |
|---|---|---|
| `try: 1 / 0` / `except ZeroDivisionError: logger.exception("Error!")` | The form refuses a `try` in the act (probe: "line 14 holds a `try`"). | `with handling(ZeroDivisionError, logger.exception, "Error!"): _ = 1 / 0` |
| `1 / 0  # noqa: B018` | pyright: "Expression value is unused (reportUnusedExpression)". | `_ = 1 / 0`, as `examples/test_5_except.py` does. |
| `@pytest.mark.parametrize(...)` | The form refuses a decorator on a test (probe: "a test takes no decorator"). | One `name = case(test, …)` line per item. Each case has a name instead of a pytest id. |
| `writer` fixture: a function with attributes `written`, `read`, `clear` | The runner accepts it. Both checkers reject it: 6 errors each (ty `unresolved-attribute`, pyright `reportFunctionMemberAccess`). Typed inputs need a typed writer. | `class Writer` in `loguru_lab.py`, with `__call__` and `read`. It must not have a `write` method: loguru would then treat it as a stream, not as a function. `clear` is dropped because this file never calls it. |
| `reset_logger` (autouse, before and after) | Both checkers reject the private `loguru.logger.__init__(…)` call. They also need an explicit `import loguru._logger`. | `before_each(reset_logger)`. One line carries `# pyright: ignore[reportCallIssue]  # ty: ignore[too-many-positional-arguments]`. Nothing runs after a case, because its process ends. |
| `test_invalid_format(writer, format)` passes -1, 3.4 and `object()` | A wrong type on purpose: the input is declared `object`. The act line carries `# pyright: ignore[reportArgumentType, reportCallIssue]  # ty: ignore[no-matching-overload]`. | The case lines stay checked. The order of the inputs changes to `(format, writer)`, because `writer` has a default. |
| `format` takes a str or a lambda | The annotation `str \| Callable[[Record], str]` needs `Record`. `Record` exists only in loguru's stub, so the port imports it under `TYPE_CHECKING` with `from __future__ import annotations`. | See "Differently" below. The runner does not read this annotation. |

## Fidelity gaps

**Less than the original**
- The reset no longer runs after each case, because the process ends. This changes nothing that a check reads. A probe removed `before_each(reset_logger)` entirely, and all 20 cases still passed, so no check in this file depends on the reset. The port keeps it so that it matches the autouse fixture.
- `writer.clear` is not ported. This file does not use it.

**More than the original**
- `handling` adds a check that `1 / 0` raised. If it did not raise, the original fails at its assert, and the port fails at `handling` and blocks the assert.
- The type checkers read each case line against the test's parameters. A misspelled input, a missing input and a wrong type are errors before the run. pytest does not give these errors for a parametrize list.

**Differently from the original**
- **The traceback in `{exception}` is not the same.** In the original, loguru logs one frame: the test's `1 / 0`, marked `>`. Under `handling`, it logs the `with handling(…)` line, `contextlib.__exit__`, `cases.py`'s `yield` (now the `>` frame, where the exception was caught), and then `_ = 1 / 0` (probe `E/proto4_probes/show_traceback.py`). The port checks only the first and the last line, as the original does, so both checks still hold. A check that reads the full traceback would see the form's code. *Fixed in the form after the port: `handling` is now a class, not a generator, and the same probe logs the same traceback under both forms, with the `>` frame on the test's own line. The port and its run did not change.*
- Each case runs in a forked process of its own. Each check but the last runs in a fork too. The original runs all items in one process, with the reset between them.
- The runner does not type-check the `format` input: an annotation that names a stub-only type is "unread", and that includes its `str` half. A probe case `format=3` was not refused. It ran, and failed in the act with `TypeError`. ty and pyright did report it. *After the port, FORM.md says so: the runner checks an input's type only when it can read the annotation at run time.*

## Probes (in `E/proto4_probes/`, not in `tests/`)

| Seeded mistake | Runner | ty | pyright |
|---|---|---|---|
| Typo in an input: `mesage="a"` | refused: "did you mean 'message'?" + "gives no 'message'" | `unknown-argument` + `missing-argument` | "No parameter named" + "Argument missing" |
| Wrong expected value: `"Message: b\n"` | failed: "they differ from character 9" | nothing | nothing |
| Expected value of the wrong type: `b"Message: a\n"` | refused: "gives expected=b'…' (bytes), and test_format declares str" | `invalid-argument-type` | `reportArgumentType` |
| `format=3` (annotation names stub-only `Record`) | not refused; failed in the act: TypeError | `invalid-argument-type` | `reportArgumentType` |
| `colorize=1` for a `bool` | refused: "declares bool" | `invalid-argument-type` | `reportArgumentType` |
| Input by position: `case(test_markup_in_field, True)` | refused | nothing | nothing |
| Computed expected value: `"Message: " + "a\n"` (case line), `"[DEBUG] Start" + "\n"` (check) | refused, both | nothing | nothing |
| `match=re.escape("most likely a mistake")` | refused: "computes `re.escape(…)`" | nothing | nothing |
| `try`/`except` as the original writes it | refused | nothing | warning: unused expression `1 / 0` |
| `@pytest.mark.parametrize` | refused: "a test takes no decorator" | nothing | nothing |
| A statement after `with raises(…)` | refused | nothing | nothing |
| Fixture input with no `use(…)` default, case omits it | refused: "gives no 'writer'" | `missing-argument` | "Argument missing" |
| `for _ in range(count)` with `count` an input | refused: one path | nothing | nothing |
| Check reads only an input: `assert format == "{message}"` | refused | nothing | nothing |
| First of two checks wrong | first failed, second still ran and passed | nothing | nothing |
| conftest's writer, unchanged, as `use(writer_as_conftest)` | passed | 6 × `unresolved-attribute` | 6 × `reportFunctionMemberAccess` |
| A class the test defines **and calls itself**, with an `if` in `__init__` | **not refused**: one case passed, one failed (two paths) | nothing | nothing |
| A nested function with an `if`, called **through an alias** (`chosen = pick; chosen(flag)`) | **not refused**: two paths | nothing | nothing |
| Control: the same function called by its own name | refused: "line 39 holds an `if`" | nothing | nothing |

The last two runner results contradict FORM.md: "A function that the test calls itself is part of the act, and has one path too." `called_inside` collects only names called directly. It never treats a `ClassDef` as code the test runs. The probes are in `E/proto4_probes/test_path_probes.py`. *Fixed in the runner after the port: a function the test uses other than as an argument of a call (called, aliased, built) is part of the act, and so are the `__init__` of a class the test builds and each method it calls. All three probes are refused now; FORM.md names the class and the alias. The port and its run did not change.*

## Line counts

| | Port | Original |
|---|---|---|
| Test file | 103 (`tests/test_add_option_format.py`) | 87 |
| Helpers / fixtures | 24 (`tests/loguru_lab.py`) | 30 (`conftest.py`, the two fixtures) |
| Total | 127 | 117 |

The port is longer for three reasons. Each parametrize row is a named `case` line after the test. Each test has type annotations. The file has the `TYPE_CHECKING` import for `Record`.
