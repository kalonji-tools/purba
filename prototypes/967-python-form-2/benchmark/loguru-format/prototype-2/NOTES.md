# Port of loguru's test_add_option_format.py to this form

All 20 items of the original are ported as 20 cases. All 20 pass, and ty and pyright report no errors. The port keeps every check the original makes. Four things changed shape. The two fixtures became explicit inputs on every act. The `try`/`1 / 0`/`except` became a `handling(...)` call with an exception the test builds itself. A loop was written out. Three deliberately wrong inputs need `# type: ignore`. The form refused nothing while I ported, because I read `FORM.md` and `run.py` first and wrote straight-line acts from the start. The probes below show what the form does with the original's own shapes. They also showed two faults in the runner, which the runner has since fixed.

## Original test → cases → assays

| Original test (pytest items) | Experiment | Cases | Assays |
|---|---|---|---|
| `test_format` (6) | `logged` | `message_field`, `no_field`, `level_twice`, `level_parts`, `function_format`, `function_record` | `product` |
| `test_progressive_format` (1) | `logged_progressively` | `progressive` | `product` |
| `test_function_format_without_exception` (1) | `logged_in_except` | `without_exception` | `product` |
| `test_function_format_with_exception` (1) | `logged_in_except` | `with_exception` | `first_line`, `last_line` |
| `test_invalid_format` (3) | `added` | `int_format`, `float_format`, `object_format` | `.raises(TypeError)` |
| `test_invalid_markups` (5) | `added` | `open_tag`, `close_tag`, `crossed_tags`, `bare_close`, `unknown_tag` | `.raises(ValueError, match=UNPARSED)` |
| `test_markup_in_field` (2) | `logged_with_field` | `markup_in_field_colorized`, `markup_in_field_plain` | `product` |
| `test_invalid_format_builtin` (1) | `added` | `builtin_format` | `.raises(ValueError, match=r".* most likely a mistake")` |

Two merges are my choice. The form did not force them. The two exception tests share one act, and only their format and their checks differ. The three "invalid" tests share one act, `logger.add(writer, format=format)`.

## Refusals and forced changes

The port met no refusal. Each row below is a change the rules force. The "Probe" column says what the form does if the original's shape is kept.

| Original | Port | Why | Probe of the original shape |
|---|---|---|---|
| `for _ in range(5): logger.opt(raw=True).debug(".")` | five identical lines | a loop in an act is refused | refused: "line 42 holds a loop: write each call on a line of its own…" |
| `try: 1 / 0` / `except ZeroDivisionError: logger.exception("Error!")` | `handling(ZeroDivisionError("division by zero"), logger.exception, "Error!")` | a `try` in an act is refused | refused: "line 59 holds a `try`: … run code inside an `except` with handling(…)". Plain `1 / 0` with no `try` is accepted, but the act raises and both cases fail. |
| `def formatter(record)` inside the test | `formatter` at module level, typed `Record` | a nested `def` is refused | refused: "a function inside: define it beside the cases" |
| `class F` inside the test | `class F` at module level | a nested `class` is refused | refused: "a class inside: define it beside the cases" |
| autouse `reset_logger` fixture | `logger: Logger = use(fresh_logger)` on each of the 5 acts | a fixture is a default of the act; the form has no autouse | — |
| `writer` fixture | `writer: Writer = use(Writer)` on each of the 5 acts; `Writer` is a class with `__call__` and `read` | same | — |
| `@pytest.mark.parametrize` rows | one named case per row: 20 names that I chose | a case is a named call | a loop of cases is refused ("is built by a loop") |
| `lines = writer.read().splitlines()`, then two `assert`s | assays `first_line` and `last_line` on the product | an `assert` in an act is refused, and a check is an assay | — |
| `format=-1`, `3.4`, `object()` | the same values, with `# type: ignore` on each case line | the act types `format` as `str \| FormatFunction`, so both checkers reject these inputs | — |
| conftest's `loguru.logger.__init__(loguru._logger.Core(), …)` | the same call, with one `# type: ignore` and a comment | `loguru.pyi` declares no `Logger.__init__`, and both checkers reject the call. A rewrite through `_logger.Logger.__init__` fails too, because the stub `Logger` and the private one are different types. This is a cost of type-checking, not a rule of the form. | — |
| no annotations | `from __future__ import annotations`, and `Logger`, `FormatFunction` and `Record` imported under `TYPE_CHECKING` | these names exist only in loguru's stub | without the future import, the file raises `NameError` on import. The runner prints "1 cases: … 1 refused", and both checkers pass. |

## Fidelity gaps

**Less than the original**
- The `ZeroDivisionError` is built by the test, and `handling` raises it. In the original, `1 / 0` raises it. The traceback text in `with_exception` has other frames: it shows `handling` in `experiment.py` and `raise exception`. The two lines that the original checks (the first line and the last line) are the same.
- The reset runs before each act only. The original runs it before and after. The forked process is thrown away after each case, so "after" has nothing left to clean.
- The writer's `clear` is not ported. This file does not use it.

**More than the original**
- `with_exception` reports both readings when the first fails. A pytest test stops at its first failing `assert`.
- Both type checkers read each case against the act. A probe shows that a key typo inside a lambda input (`r["levle"]`) is flagged before the run. This comes from the annotations. No check was added.
- A warning that a case does not expect fails the case. pytest does not fail on a warning by default. No warning occurred here.

**Differently**
- Each case runs in a forked process. Because of the fork, the reset does not decide pass or fail. A probe with the reset removed still passes all 20 cases. The reset only keeps loguru's default stderr line out of a failure screen. I kept it for fidelity.
- `added` returns the handler id. The original discards it. No case reads it. It appears only in a failure screen ("no raise; the act returned 1").
- Case names are written by hand. pytest builds ids from the parameters, for example `test_format[a-Message: {message}-Message: a\n]`.
- The markup regex is bound once to `UNPARSED` and used five times. The original repeats it through `parametrize`. This is allowed because `match=` is not an expected value.

## Line counts

| | total lines | non-blank |
|---|---|---|
| port `experiments/ex_loguru_format.py` | 110 | 77 |
| helpers `experiments/loguru_lab.py` | 33 | 22 |
| **port + helpers** | **143** | **99** |
| original `test_add_option_format.py` | 87 | 67 |
| original `conftest.py` (the two fixtures) | 30 | 22 |
| **original + conftest** | **117** | **89** |

The port is 26 lines longer (+10 non-blank). Most of the extra lines are the four unrolled loop lines, the two assay functions, the `TYPE_CHECKING` block, and blank lines between experiments.

## Probes

Each probe copies the port to `../proto2_probes/experiments/` with one change. Then the runner, ty 0.0.87 and pyright 1.1.411 run on the copy.

| Seeded change | runner | ty | pyright |
|---|---|---|---|
| input typo `logged(mesage="a", …)` | refused, "did you mean 'message'?" | 2 errors | 2 errors |
| assay typo `.expect(first_lin=…)` | refused, "did you mean 'first_line'?" | silent | silent |
| wrong value `product="Message: a"` | fails, expected/reading shown | silent | silent |
| product of the wrong type `product=b"e"` | refused, "returns str" | error | error |
| assay value of the wrong type `last_line=1` | refused, "the assay last_line returns str" | silent | silent |
| input of the wrong type `message=1` | runs, and fails on the value `'Message: 1\n'` | error | error |
| key typo in a lambda input `r["levle"]` | fails: loguru catches the `KeyError`, the reading is `''`, and the failure screen shows loguru's stderr | error | error |
| typo on a `# type: ignore` line `added(fromat=-1)` | refused, "did you mean 'format'?" | **silent** | **silent** |
| wrong raise type `.raises(ValueError)` for `-1` | fails: "expected ValueError matching None", and the `TypeError` is shown | silent | silent |
| expected value computed inline `"…".format(message="a")` | refused, "computes `…`" | silent | silent |
| the same value bound to a name first, `EXPECTED_A = "Message: {message}\n".format(message="a")` | accepted, and passed; since the fix, refused: "computes `…`" | silent | silent |
| a loop, a `try`, a nested `def`, a nested `class` in an act | each refused, and the whole experiment stops | silent | silent (pyright warns about an unused `1 / 0`) |
| no logger reset | 20 pass. On a failure, the "output" line shows loguru's default stderr record. | silent | silent |
| a helper `broken_lab.py` that cannot be imported, in the folder | "21 cases: 20 passed · 1 refused", exit 1; since the fix, "the file is refused", counted apart from the cases | — | — |
| no `from __future__ import annotations` | "1 cases: … 1 refused", although the file has 20 cases; since the fix, "the file is refused … so none of its cases runs" | silent | silent |

## Runner faults (both fixed in the runner after the port; the port and its run did not change)

1. **A name bound to a computed value passes as a literal.** `FORM.md` says an expected value is "a literal, or a name bound to one". `run.py` line 171 checks only the type of the bound value, not how the name was bound. This input passes:
   ```python
   EXPECTED_A = "Message: {message}\n".format(message="a")
   message_field = logged(message="a", format="Message: {message}").expect(product=EXPECTED_A)
   ```
   The same expression written inline is refused.
2. **A file that cannot be imported counts as one refused case.** `run.py` line 531 adds 1 to `refused` for each such file. A broken helper adds a "case" that does not exist ("21 cases"). A broken test file with 20 cases is reported as "1 cases". The exit status is 1 in both, so the run still fails.
