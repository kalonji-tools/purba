# Port of loguru's test_threading.py to prototype form 5

All 7 pytest items are ported as 7 cases, and all 7 pass with 14 checks (`OUTPUT.txt`). The original passes 7 of 7 in 14.2 s. The port takes about 14.3 s. The form forced one change: each `for thread in threads:` loop became `for k in range(2): threads[k].start()`, because a loop in the act must run over literal data. The form has no `capsys` and no conftest, so I wrote a stand-in `Capsys` fixture and moved `reset_logger` to a `before_each` in the file (`tests/loguru_lab.py`). The biggest gap is a new silent failure that the workaround brings. The loop count is now a literal that must match the length of the thread list, and nothing checks that. A join loop that is too short passes (probe m8). The runner also has one real bug. It checks for one path only in functions and classes defined inside the test body. A helper or class at module level, or one imported from another module, is not checked. FORM.md says that it must be.

## Original test → cases → checks

| Original test | Fixtures | Case | Checks (original → port) | Result |
|---|---|---|---|---|
| `test_safe_logging` | (reset_logger) | `safe_logging` | 1 → 1 (`sink.written in (…, …)`) | pass |
| `test_safe_adding_while_logging` | writer (unused) | `safe_adding_while_logging` | 2 → 2 | pass |
| `test_safe_removing_while_logging` | capsys | `safe_removing_while_logging` | 3 → 3 | pass |
| `test_safe_removing_all_while_logging` | capsys | `safe_removing_all_while_logging` | 2 → 2 | pass |
| `test_safe_slow_removing_all_while_logging` | capsys | `safe_slow_removing_all_while_logging` | 2 → 2 | pass |
| `test_safe_writing_after_removing` | capsys | `safe_writing_after_removing` | 2 → 2 | pass |
| `test_heavily_threaded_logging` | capsys | `heavily_threaded_logging` | 2 → 2 | pass |

There are no parametrize rows. The total is 14 checks in the original and 14 in the port.

## First attempt: what the runner said

The first attempt is in `probes/first_attempt/` (`test_threading.py`, `loguru_lab.py`). It is the original with three changes only: `before_each(reset_logger)`, `writer: Writer = use(Writer)` and `capsys: Capsys = use(Capsys)` as inputs, and one `case(...)` line under each test. `NonSafeSink` stayed at module level, as in the original. The nested `threaded`, `thread_1` and `thread_2` functions handed to `Thread(target=…)` were accepted. The comprehensions `[Thread(target=threaded) for _ in range(2)]` and the loops `for _ in range(1000):` were accepted too.

The runner refused all 7 tests for the same reason, twice each (`probes/first_attempt/run1.txt`):

```
 test_safe_logging  (test_threading.py:35)
   ! refused:
       line 48 holds a loop over `threads`: the act has one path, so a loop runs over literal data, such as `range(5)`, with no `else`; or make each round a case
       line 51 holds a loop over `threads`: the act has one path, so a loop runs over literal data, such as `range(5)`, with no `else`; or make each round a case
     so none of its 1 case(s) runs
 … the same two lines for each of the other six tests …
══ 7 cases: 0 passed · 0 failed · 7 refused · 0 blocked   (0 checks held)
```

What I changed: each pair of loops became a loop over a literal range that indexes the list:

```python
for k in range(2):        # range(10) in test_heavily_threaded_logging
    threads[k].start()
for k in range(2):
    threads[k].join()
```

The second run passed 7 of 7 (`probes/first_attempt/run2.txt`). I tried three other ways to write the loop. None of them is better:

| Rewrite | Example | Runner |
|---|---|---|
| Loop over a tuple of the threads | `for thread in (first, second): thread.start()` | refused: "holds a loop over `(first, second)`" (probe e6) |
| Comprehension over the list | `[thread.start() for thread in threads]` | refused: "holds a comprehension over `threads`: the act has one path, so it runs over literal data, with no `if`" (probe e6) |
| Helper that loops, defined inside the test | `def start_and_join(threads): for t in threads: …` | refused, as FORM.md says (probe e3b) |
| The same helper at module level, or imported | `start_and_join(threads)` | **accepted**, which FORM.md forbids (probes e3, e4; see Runner bugs) |

I did not use the module-level helper in the port, because the runner accepts it only because of a bug.

ty and pyright had one complaint against the first attempt. pyright read the type of `stop_time` from its default `0` as `int`, and refused `NonSafeSink(0.1, 0.1)`: `Argument of type "float" cannot be assigned to parameter "stop_time" of type "int"`. ty also did not accept the `# type: ignore[attr-defined]` on the `loguru.logger.__init__(…)` reset: `Too many positional arguments to object.__init__: expected 1, got 11`, and `Submodule _logger might not have been imported`. I annotated `NonSafeSink` with `float`, and wrote the reset with `cast(Any, loguru.logger).__init__(…)` after `import loguru._logger`. After that, both checkers are clean (`CHECKERS.txt`).

## Refusals and forced changes

| Original | What forced a change | Port |
|---|---|---|
| `for thread in threads: thread.start()` / `.join()` (14 loops) | The runner refuses a loop over a value the act built | `for k in range(2): threads[k].start()`, with the count written as a literal |
| `capsys` (a built-in pytest fixture) | The form has no capture fixture | `Capsys` in `loguru_lab.py`. Building it swaps `sys.stdout` and `sys.stderr` for `StringIO` buffers. `readouterr()` returns both buffers. |
| autouse `reset_logger` in conftest.py, which resets before and after each test | The form has no conftest and no autouse | `before_each(reset_logger)` in the test file. The reset after the test is dropped, because the case's process ends. |
| `writer` fixture (a function with attributes) | `use()` needs a factory; a type checker cannot follow a function with attributes | `Writer` class with `__call__`, `read`, `clear` |

The original's sleeps (1 s, 0.5 s, 0.1 s), barriers and joins are unchanged.

## Fidelity gaps

**Less than the original**

- The literal loop count. The original joins every thread in the list. The port joins `range(2)` or `range(10)` threads, and that number is written apart from the list. A count that is too high raises `IndexError`. A count that is too low is silent. Probe m8 joins only one of the two threads, and it passes 3 of 3 runs, because `logger.remove()` waits on the handler lock that the second thread holds. In `test_heavily_threaded_logging` the number 10 is written three times, and the three must agree.
- Forgetting the reset. In pytest, `reset_logger` is autouse in conftest.py, so a test cannot leave it out. In the form, each file must call `before_each(reset_logger)`. Probe m5 leaves it out. loguru's default handler then logs `Some message` to the `sys.stderr` it saw at import, which is the runner's own fd 2. `Capsys` does not see it, so `err == ""` holds and the case passes. The line is in the runner's capture, which it prints only for a failing case.
- `Capsys` is my own code. It is not pytest's. `readouterr()` does not clear the buffers as pytest's does. Each test calls it once, so the results are the same here.

**More than the original**

- An exception in a thread fails the case. Probe e1 makes a sink raise in a thread with `catch=False`. The runner reports `an error that no code saw: in a thread, RuntimeError: Can't write on stopped sink`. Under the original's setup (pytest 9.2.0, the given conftest, no `-W error`), the same test **passes** with only a `PytestUnhandledThreadExceptionWarning`. pytest's thread hook takes the traceback, so `err == ""` holds (`probes/pytest_twin/`, output at the end of `probes/runner.txt`). So the port catches a fault that the original misses.
- A warning that the test does not expect fails the case. In pytest it is only reported.
- Each case runs in its own forked process, and each check but the last in its own fork. A state that leaks from one test to the next cannot occur. The original depends on `reset_logger` for this.
- Each case has a 30 s alarm. The whole run of 7 cases takes about 14.3 s, so no case comes near it.

**Differently from the original**

- An exception in a thread now reaches the case through the runner's `lost error` report, not through `err`. The port's `err == ""` checks cannot see a thread's traceback, because the runner's `threading.excepthook` takes it first (e1: `err == ''` holds while the case fails). A print to `sys.stderr` from a thread still reaches `err` (control probe e2 fails on `err == ''`, and reads `'noise\n'`).
- `writer` is an input of `test_safe_adding_while_logging` that the test does not use, as in the original. The runner says nothing about it.
- `NonSafeSink` has `float` annotations, which pyright needs (see above).
- `Capsys` must be built only through `use(Capsys)`. FORM.md says a case may pass its own value. `case(test, capsys=Capsys())` builds the capture at import, in the runner's own process. The runner's report then goes into the buffer, and the run exits 1 with no output. The case's own `out` then reads the runner's report header (probe m2b, which `probes/show_swallowed_report.py` recovers).

## What the form cannot express

Everything this file needs can be written in the form, but two things only indirectly:

- **A loop over the threads that the act made.** The form has no loop over a value. A loop over a literal range that indexes the list is the closest the form accepts. That loop does not show that it covers the whole list (see Fidelity gaps).
- **A capture of `sys.stdout` and `sys.stderr` that is safe to build anywhere.** A fixture can act only when it is built. So a capture must swap the streams in its constructor, and that is harmful when a case builds it at module level (m2b). The form gives no point in the case's own process where a value given by a case is started.

## Runner bugs

1. **A helper function at module level, which the test calls itself, is not checked for one path.** FORM.md says: "A function that the test calls itself, under any name, is part of the act, and has one path too". `probes/test_e3_local_helper_loop.py` loops over its argument, and the runner accepts it. The same helper inside the test is refused (`probes/test_e3b_nested_helper_loop.py`, control).
2. **An imported helper is not checked either.** `probes/test_e4_imported_helper_loop.py` imports `start_and_join` from `probes/probe_helpers.py`, and the runner accepts it.
3. **The `__init__` and called methods of a class at module level are not checked.** FORM.md says "so are the `__init__` of a class the test builds, and each method the test calls". `probes/test_e5_method_with_if.py` has an `if` in `__init__` and an `if` and a `raise` in `write`, which the test calls. The runner accepts it. The same class inside the test is refused with three faults (lines 36, 41, 42).

The cause, from `run.py` (read only to diagnose): `called_inside` collects only the `FunctionDef`, `ClassDef` and lambda nodes found by `ast.walk(test)`, so it looks only inside the test's own body. The port does not depend on these bugs. Its `NonSafeSink` is at module level, but its `__init__` has one path, and the test does not call `write` or `stop` itself.

Not runner bugs, but confusing:

- Probe m3 passes `writer="writer"`. The runner gives the right fault ("gives writer='writer' (str), and test_writes declares Writer") and then a second one: "line 13 reads only writer, a value the act cannot change; read what the act made". The second fault follows from the first. FORM.md allows it, but it points at a check that is correct.
- Probe m2b: a fixture that redirects streams, built at module level, hides the runner's whole report. Exit 1 with no output.

## Probes

Run from `probes/` with `PYTHONPATH=../tests`. Full outputs: `probes/runner.txt`, `probes/checkers.txt`, `probes/m7_repeats.txt`, `probes/m8_repeats.txt`.

Seeded mistakes:

| Probe | Mistake | Runner | ty | pyright |
|---|---|---|---|---|
| m1 `test_m1_wrong_counter.py` | `sink_2.written == "ccc0ddd\n"` (wrong counter) | fail: "they differ from character 3: expected '0ddd\n', read '1ddd\n'". The first check still passes. | nothing | nothing |
| m2 `test_m2_misspelled_input.py` | `case(test_quiet, capsis=use(Capsys))` | refused: "gives 'capsis', which is no input of test_quiet; did you mean 'capsys'?" | `unknown-argument`: "Argument `capsis` does not match any known parameter of function `case`" | `reportCallIssue`: "No parameter named "capsis"" |
| m2b `test_m2b_fixture_built_in_case.py` | `case(test_quiet, capsys=Capsys())` | exit 1, **no output**. The swapped buffer holds a failing `out == ''` that reads the runner's own header. | nothing | nothing |
| m3 `test_m3_wrong_type_input.py` | `writer="writer"` | refused: "gives writer='writer' (str), and test_writes declares Writer" + "line 13 reads only writer, a value the act cannot change" | `invalid-argument-type`: "Expected `Writer`, found `Literal["writer"]`" | `reportArgumentType`: "Literal['writer']" is not assignable to "Writer" |
| m4 `test_m4_forgot_join.py` | starts the threads, never joins them | fail: `'___0___\n' in (…)` does not hold | nothing | nothing |
| m5 `test_m5_forgot_reset.py` | no `before_each(reset_logger)` | **pass** (the default handler writes to the runner's fd 2, not to `Capsys`) | nothing | nothing |
| m6 `test_m6_readouterr_not_called.py` | `out, err = capsys.readouterr` | fail: "the act raised TypeError: cannot unpack non-iterable method object". Both checks blocked. | `not-iterable` | `reportGeneralTypeIssues`: "() -> tuple[str, str]" is not iterable |
| m7 `test_m7_one_order_only.py` | expects only the order `___0___` then `___1___` | pass, 6 of 6 runs. This flaky check is not detected. The original would behave the same. | nothing | nothing |
| m8 `test_m8_join_range_too_short.py` | join loop `range(1)` for 2 threads | pass, 3 of 3 runs (silent, see Fidelity gaps) | nothing | nothing |

Experiments:

| Probe | What it tries | Runner | ty / pyright |
|---|---|---|---|
| e1 `test_e1_thread_error.py` | sink raises in a thread, `catch=False` | fail: "lost error … in a thread, RuntimeError: Can't write on stopped sink". `out == ''` and `err == ''` both hold. The pytest twin passes with a warning. | clean |
| e2 `test_e2_stderr_in_thread.py` | thread prints to `sys.stderr` (control for `Capsys`) | fail: `err == ''` reads `'noise\n'` | clean |
| e3 `test_e3_local_helper_loop.py` | module-level helper loops over its argument | **pass** (bug 1) | clean |
| e3b `test_e3b_nested_helper_loop.py` | the same helper inside the test | refused: loop over `threads` at lines 10 and 12 | clean |
| e4 `test_e4_imported_helper_loop.py` | the same helper, imported | **pass** (bug 2) | clean |
| e5 `test_e5_method_with_if.py` | module-level class with `if` in `__init__` and `write`, vs the same class inside the test | module-level: **pass** (bug 3). Inside the test: refused, with 3 faults. | clean |
| e6 `test_e6_tuple_loop.py` | loop over `(first, second)`; comprehension over `threads` | both refused | clean |

## Line counts

| | Original | Port |
|---|---|---|
| Test file | 225 (`test_threading.py`; 155 non-blank) | 255 (`tests/test_threading.py`; 168 non-blank) |
| Fixtures / helpers | 22 (conftest.py `reset_logger` 12, `writer` 10; `capsys` is pytest's, 0) | 50 (`tests/loguru_lab.py`: `reset_logger`, `Writer`, `Capsys`; 34 non-blank) |
| Total | 247 | 305 |

The test file is 30 lines longer. Most of this is the 7 `case(...)` lines and the blank lines around them (21), plus `before_each`, the imports and the docstring. The 14 loop rewrites do not change the line count. `Capsys` adds 14 lines that pytest gives for free.
