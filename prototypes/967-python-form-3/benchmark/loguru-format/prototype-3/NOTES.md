# Port of loguru's tests/test_add_option_format.py

The port has 20 cases, one for each pytest item of the original. All 20 pass, and ty and pyright report nothing.
The form refused one test as first written: `test_progressive_format`. Its format function holds an `if`, and its act holds a `for`.
Every check of the original is kept. The form changed where two parts of the code live, and it added two fixture inputs to every test.

Files: `tests/test_add_option_format.py` (the port), `tests/loguru_lab.py` (helpers), `OUTPUT.txt`, `CHECKERS.txt`.
Probes are in `../proto3_probes/` (the first draft, the probe files, and the output of each run).

## Original test → cases → checks

| Original test (pytest items) | Cases | Checks per case |
|---|---|---|
| `test_format` (6) | `message_field`, `no_field`, `level_twice`, `level_attributes`, `function_format`, `function_of_level` | `writer.read() == expected` |
| `test_progressive_format` (1) | runs once under its own name | `writer.read() == "[DEBUG] Start: .....\n" "[DEBUG] End\n"` |
| `test_function_format_without_exception` (1) | runs once | `writer.read() == "Error!\n"`, and the runner proves that the `except` ran |
| `test_function_format_with_exception` (1) | runs once | `lines[0] == "Error!"`, `lines[-1] == "ZeroDivisionError: division by zero"`, and the runner proves that the `except` ran |
| `test_invalid_format` (3) | `negative_int`, `a_float`, `an_object` | `raises TypeError` (no match, as in the original) |
| `test_invalid_markups` (5) | `open_red`, `close_red`, `close_then_open`, `bare_close`, `unknown_tag` | `raises ValueError` matching `^Invalid format, color markups could not be parsed correctly$` |
| `test_markup_in_field` (2) | `colorized`, `uncolorized` | `writer.read() == "</>  <blue>  Test\n"` |
| `test_invalid_format_builtin` (1) | runs once | `raises ValueError` matching `.* most likely a mistake` |

The total is 20 cases and 21 checks that pass. The 21 checks are 12 equality checks and 9 `raises` checks.

## Refusals and forced changes

1. **The `if` in the format function of `test_progressive_format` was refused** ("line 38 holds a branch: make each side a case of its own").
   The hint does not fit this test: one sink must format both kinds of record in the same run.
   I moved the function to `loguru_lab.progressive_formatter`, unchanged. The `if` still exists, but now the test does not show it. A reader must open a second file to see the format.
2. **The `for _ in range(5)` loop in the same test was refused.** I wrote the five `logger.opt(raw=True).debug(".")` calls on five lines.
3. **There is no autouse fixture.** The original resets the logger before and after each test. Each test now takes `logger: Logger = use(fresh_logger)`, and `fresh_logger` runs the conftest's reset and returns `loguru.logger`.
   The reset after the test is dropped, because each case runs in a fork of its own.
   Without this input, loguru's default stderr handler stays, and every log call also writes to stderr (probe `test_s09_no_reset`).
4. **`writer` is a class, `Writer`**, given as `writer: Writer = use(Writer)`. The original's `clear()` is not ported, because no test in this file calls it.
5. **`test_invalid_format` passes a value of the wrong type on purpose.** The input is declared `object`, and the act line carries `# pyright: ignore[reportArgumentType, reportCallIssue]  # ty: ignore[no-matching-overload]`.
   Both pyright codes are needed. Without the comment, pyright reports both and ty reports one (probe `p09_no_ignore`).
6. **`1 / 0` became `_ = 1 / 0`.** pyright warns `reportUnusedExpression` on the bare expression. ruff's `# noqa: B018` in the original does not silence pyright. This change comes from pyright, not from the form.
7. **The conftest's reset needs a coded ignore in `loguru_lab.py`.** loguru's stub declares no `Logger.__init__`, so both checkers reject `loguru.logger.__init__(Core(), …)`. Both checkers also needed an explicit `import loguru._logger` (pyright: an error; ty: a warning).

## Fidelity gaps

**Less than the original**
- The reset after each test is not run. A fork makes it unnecessary, but the port does not do it.
- `test_progressive_format` no longer shows its format function. The function is the same code in `loguru_lab.py`.

**More than the original**
- In the two exception tests, the runner proves that `1 / 0` raised and the `except` ran. In the original, a missing raise fails only because the check then reads `""`.
- In `test_function_format_with_exception`, both checks run even when the first fails. pytest stops at the first failed assert.
- A check compares by type, then by value. Every reading here is a `str`, so this changes no verdict in this file.
- Each case gives a name (`open_red`, `negative_int`). pytest gives an id (`<red>`, `-1`, `format2`).

**Differently from the original**
- `logger` is an input of each test, not the module global. Each signature carries two fixture inputs (`writer`, `logger`). Seven signatures are one line of 102 to 117 characters; the original's are 30 to 60.
- Every input has a type annotation. `test_format`'s signature takes 7 lines, and the file needs `if TYPE_CHECKING:` imports for `Logger` and `Record`.
- The five `.` calls are five lines, not a loop.
- The runner runs the act as a function named `__act__`. loguru records that name: `{function}` reads `__act__`, not the test's name (probe `test_probe_function`). No check in this file reads `{function}`, so no verdict changes here. *Fixed in the runner after the port: the act now keeps the test's name, and `{function}` reads it. The port and its run did not change.*

## Line counts

| | Lines | Non-blank |
|---|---|---|
| Original `test_add_option_format.py` | 87 | 67 |
| Original `conftest.py` (the two fixtures) | 30 | — |
| Port `tests/test_add_option_format.py` | 106 | 77 |
| Helpers `tests/loguru_lab.py` | 50 | 34 |
| Total, original | 117 | |
| Total, port | 156 | |

The port is 39 lines longer. The 16 case lines take about as many lines as the original's 4 `parametrize` decorators (14 lines). The extra lines come from `test_format`'s 7-line signature, the `TYPE_CHECKING` imports, the five unrolled calls, a blank line after each group of cases, and a longer helpers file (the `Writer` class and the moved format function).

## Probes

Each probe is in `../proto3_probes/p/`, with its output in `../proto3_probes/*.out`.

| What I tried | Runner | ty | pyright |
|---|---|---|---|
| Misspelled input `mesage=` | refused, with "did you mean 'message'?" | `unknown-argument` + `missing-argument` | `reportCallIssue` ×2 |
| Wrong expected value (`"Message: b\n"`) | fails, shows the first differing character | silent | silent |
| Expected `"Message: a"` without the `\n` | fails: "differ from character 10: expected '', read '\n'" | silent | silent |
| Expected value of the wrong type (`b"…"`, `10`) | refused: "gives expected=10 (int), and test_format declares str" | `invalid-argument-type` | `reportArgumentType` |
| `format=-1` into an input declared `str` | refused | `invalid-argument-type` | `reportArgumentType` |
| Key typo inside a lambda on a case line (`r["levle"]`) | fails; loguru catches the `KeyError` in the sink, and the runner shows it as captured output | `invalid-key` | `reportGeneralTypeIssues` |
| A valid markup in `test_invalid_markups` | fails: "the statement did not raise" | silent | silent |
| The `try` body does not raise (`1 / 1`) | fails: "the body of the `try` did not raise ZeroDivisionError"; the check is blocked | silent | silent |
| The loop as a list comprehension | refused, as the `for` | — | — |
| `with raises(…) as caught:` | refused, with a hint to drop `as` | — | — |
| `assert "ZeroDivisionError" in writer.read()` | refused: compare with `==` | — | — |
| An expected value computed in the check | refused | — | — |

### Holes in the rules

The rules read the syntax, so these constructs pass:

- **A loop as `map`:** `_ = list(map(lambda _: logger.opt(raw=True).debug("."), range(5)))` is accepted and passes (`test_s02_map_loop`).
- **A branch as arithmetic:** a format function `return "[{level.name}] {message}" + "\n" * ("noend" not in record["extra"])` is accepted inside the test (`test_s03_branch_as_arithmetic`). I did not use this in the port. It hides the branch from the rule, and moving the function to the helpers does the same.
- **A check that reads only an input, through a builtin:** `assert len(message) == 1` is accepted. On the next line, `assert message.upper() == "A"` is refused. The rule counts `len` as a second name, so it does not see that the check reads only `message` (`test_s06_reads_input_via_builtin`). *Fixed in the runner after the port: a builtin no longer counts, and both lines are refused.*
- **A computed expected value, through an alias of the test or through `**{…}`:** `alias = test_s08` and then `case(alias, message="a", expected="a".upper().lower())` runs. `case(test_s08, **{"message": "a", "expected": "a".upper().lower()})` also runs. The same value given directly is refused (`s08_direct`). Neither checker flags the alias. pyright gives an unrelated error on the `**{…}` case: a `str` cannot go to `writer` and `logger`. *Fixed in the runner after the port: a case not written `case(<the test's own name>, …)` is refused, and so is a case that passes inputs with `**`.*

### Runner bugs

- **A folder given with `..` finds no test.** `cd proto3 && python run.py ../proto3_probes/p` prints `0 cases` and exits 1, with no message. The same folder given as an absolute path runs 20 cases. `files_in` drops every path that has a part starting with `.`, and `..` is such a part. *Fixed in the runner after the port: only the parts below the given folder are read.*
- **A second file with the same name is dropped without a message.** `python run.py tests <abs>/proto3_probes/p2`, where `p2` holds a copy of `test_add_option_format.py`, prints one file and 20 cases, not 40. Run alone, `p2` gives 20 cases. `run_file` reuses `sys.modules[path.stem]`, so the second file gets the first file's module, and then no test matches its path. *Fixed in the runner after the port: each file is imported under a module name of its own, and the probe gives 40 cases.*
- **The runner refuses with a hint that does not fit.** "make each side a case of its own" cannot apply to a branch in a callback that the code under test calls once for each record.
