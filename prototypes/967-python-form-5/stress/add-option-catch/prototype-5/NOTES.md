# Port of loguru's test_add_option_catch.py to form 5

All 11 pytest items are ported as 11 cases. All 11 pass, with 31 checks (the original's 30 asserts plus one `raises`). The form forced five kinds of change: `capsys` became a helper class (`Capture`, used as `use(Capture)`), the chained `out == err == ""` became one tuple check, `startswith`/`endswith`/bare `re.match` became comparisons, the readout after `with raises` moved into the check, and the two `with` blocks (`monkeypatch.context()`, `default_threading_excepthook()`) became plain statements with no restore. Three runner bugs got in the way. The biggest gap comes from one of them. When the enqueue thread is alive, the runner's own fork before each check raises a `DeprecationWarning`, and the runner blames the test for it. To pass, the enqueued `test_broken_sink_message` case now calls `logger.remove()` before it reads stderr. That joins the worker, so the port no longer checks that the 0.1 s sleep alone was enough.

## Original test → cases → checks

| Original test (line) | Cases | Checks per case | Status |
|---|---|---|---|
| `test_catch_is_true` (16) | `catch_is_true` | 2 | pass |
| `test_catch_is_false` (24) | `catch_is_false` | 1 `raises` + 1 | pass |
| `test_no_sys_stderr` (32) | `no_sys_stderr` | 1 | pass |
| `test_broken_sys_stderr` (42) | `broken_sys_stderr` | 1 | pass |
| `test_encoding_error` (55) | `encoding_error` | 6 | pass |
| `test_unprintable_record` (73) | `unprintable_record` | 6 | pass |
| `test_broken_sink_message[False]` (94) | `broken_sink_message_direct` | 5 | pass |
| `test_broken_sink_message[True]` (94) | `broken_sink_message_enqueued` | 5 | pass, after a workaround |
| `test_broken_sink_caught_keep_working[False]` (110) | `broken_sink_caught_keep_working_direct` | 1 | pass |
| `test_broken_sink_caught_keep_working[True]` (110) | `broken_sink_caught_keep_working_enqueued` | 1 | pass |
| `test_broken_sink_not_caught_enqueue` (129) | `broken_sink_not_caught_enqueue` | 1 | pass |

The original passes 11 of 11 under pytest 9.2.0 (0.56 s). The port passes 11 of 11 (0.72 s wall clock).

## First attempt: what the runner said

The first attempt is in `probes/first_attempt/attempt1_test_add_option_catch.py.txt`, and its output is in `attempt1_output.txt`. I wrote it as close to the original as FORM.md seemed to allow: `capsys` became `capsys: Capture = use(Capture)`, `pytest.raises` became `raises`, `monkeypatch.context()` became a plain assignment, and the `with default_threading_excepthook():` block stayed. Result: `11 cases: 4 passed · 0 failed · 7 refused · 0 blocked`.

- **`test_catch_is_false`** (readout after `with raises`, chained check):
  ```
  line 30: `with raises(…):` is the last statement of the act; after a raise the act has nothing left to run
  line 33: a check compares a reading with the value you expect, with one operator: `assert reading == expected`, `in`, `<`, `is None`
  ```
  Change: removed `out, err = capsys.readouterr()` from the act. The check is now `assert capsys.readouterr() == ("", "")`.
- **`test_no_sys_stderr`** (chained check):
  ```
  line 45: a check compares a reading with the value you expect, with one operator: `assert reading == expected`, `in`, `<`, `is None`
  ```
  Change: `assert (out, err) == ("", "")`.
- **`test_broken_sys_stderr`** (monkeypatch by assignment, chained check):
  ```
  line 53 holds a `raise`: expect the code's own raise with `with raises(…):`
  line 60: a check compares a reading with the value you expect, with one operator: `assert reading == expected`, `in`, `<`, `is None`
  ```
  Line 53 is the `raise OSError` inside `broken_write`, a function the test hands to `sys.stderr`. This is runner bug 2. Change: `setattr(sys.stderr, "write", broken_write)` and a tuple check.
- **`test_encoding_error`** (`startswith`/`endswith` checks):
  ```
  line 78: a check compares a reading with the value you expect, with one operator: `assert reading == expected`, `in`, `<`, `is None`
  line 79: a check compares a reading with the value you expect, with one operator: `assert reading == expected`, `in`, `<`, `is None`
  line 80: a check compares a reading with the value you expect, with one operator: `assert reading == expected`, `in`, `<`, `is None`
  ```
  Change: `lines[1][:13] == "Record was: {"`, `lines[1][-1] == "}"`, `lines[-2][:19] == "UnicodeEncodeError:"`.
- **`test_broken_sink_message`** (bare `re.match`, `startswith`):
  ```
  line 121: a check compares a reading with the value you expect, with one operator: `assert reading == expected`, `in`, `<`, `is None`
  line 122: a check compares a reading with the value you expect, with one operator: `assert reading == expected`, `in`, `<`, `is None`
  ```
  Change: `re.match(…) is not None`, `lines[-2][:18] == "ValueError: Error!"`.
- **`test_broken_sink_not_caught_enqueue`** (`with default_threading_excepthook():`):
  ```
  line 162 holds a `with` other than `raises`, `warns` or `handling`: call what it sets up, or make it a fixture with use(…)
  ```
  Change: `loguru_lab.use_default_threading_excepthook()` is called in the act, and the three lines it wrapped follow it.
- Passed at once: `test_catch_is_true`, `test_unprintable_record`, and both `test_broken_sink_caught_keep_working` cases. The `if` and `raise` in `half_broken_sink`, the `nonlocal`, and the raising `Unprintable.__repr__` were all accepted as values handed to the code.

The second attempt (`attempt2_*`) gave `11 cases: 10 passed · 1 failed`. The failure is runner bug 1, reported four times, once for each check but the last:

```
   ✗ broken_sink_message_enqueued › warning   test_add_option_catch.py:126
       what      a warning that the test does not expect: DeprecationWarning: This process (pid=2072629) is multi-threaded, use of fork() may lead to deadlocks in the child.
                 at ../../../../run.py:667
       inputs    enqueue=True
```

Change: `logger.remove()` after `time.sleep(0.1)` in `test_broken_sink_message`. It runs in both cases. The direct case does not need it.

In `loguru_lab.py`, ty and pyright both rejected the conftest's `logger.__init__(Core(), None, 0, …)` call, because loguru's stub gives `Logger.__init__` no arguments. ty said `too-many-positional-arguments` and pyright said `reportCallIssue`. Both coded ignores are on the line with the arguments; pyright reports on that line, not on the `__init__(` line. ty also warned that `@contextmanager` with `-> Iterator[None]` is deprecated. That context manager is gone from the port, so the warning went with it.

## Refusals and forced changes

| What the original does | What forced a change | What the port does instead |
|---|---|---|
| `capsys` fixture | pytest built-in, not in the form | `Capture` in `loguru_lab.py`: swaps `sys.stdout`/`sys.stderr` for `StringIO` when the runner builds the input. `readouterr()` returns and clears, as in pytest. |
| autouse `reset_logger` (before and after) | no autouse fixture; nothing runs after a case | `before_each(reset_logger)`. The reset after the case is dropped, because the process ends. |
| `writer` fixture (function with attributes) | `use(factory)` wants a factory | `Writer` class; `writer: Writer = use(Writer)` |
| `@pytest.mark.parametrize("enqueue", [False, True])` | cases are named one by one | two `case(…, enqueue=…)` lines for each of the two tests |
| `assert out == err == ""` | one operator per check | `assert (out, err) == ("", "")`, or `capsys.readouterr() == ("", "")` |
| `readouterr()` after `with pytest.raises` | `raises` is the last statement of the act | the readout moves into the check |
| `assert s.startswith(x)`, `s.endswith(x)` | a check is a comparison | `s[:len] == x`, `s[-1] == x`, with each length counted by hand |
| `assert re.match(…)` | a check is a comparison | `re.match(…) is not None` |
| `with monkeypatch.context() as c: c.setattr(sys, "stderr", None)` | `with` refused | `sys.stderr = None` in the act. Never undone. |
| `c.setattr(sys.stderr, "write", broken_write)` | `sys.stderr.write = broken_write` refused (runner bug 2) | `setattr(sys.stderr, "write", broken_write)` |
| `with default_threading_excepthook(): …` | `with` refused | `use_default_threading_excepthook()` in the act. Never undone. |
| enqueued case reads stderr with the worker thread alive | runner bug 1: fork warning blamed on the test | `logger.remove()` at the end of the act |

## Fidelity gaps

**Less than the original**

- `broken_sink_message_enqueued`: `logger.remove()` joins the worker thread before the readout. The original reads after 0.1 s with the thread still running, so it also checks that 0.1 s is enough time for the worker to write the error. The port passes even with a slow worker. The direct case also runs `logger.remove()`, which has no effect there.
- `broken_sink_not_caught_enqueue`: `use_default_threading_excepthook()` replaces `threading.excepthook`, which is where the runner catches an error that no code sees in a thread. In this case the runner would no longer see such an error (probe `probes/excepthook_hides_thread_error/`). The original does the same thing to pytest's thread hook, so this is not less than pytest. It is less than what the form promises everywhere else. loguru 0.8.0 prints a sink error in the worker and never lets it escape, so nothing is lost here now.

**More than the original**

- Every check runs even when an earlier check fails. pytest stops at the first failing assert, so a failure in `lines[0]` would hide the other five lines of `test_encoding_error`.
- An unexpected warning, or an error in a thread or a finaliser, fails the case. pytest only warns. This applies to every case except `broken_sink_not_caught_enqueue`.
- `enqueue` is type-checked (`bool`) by ty, pyright and the runner.
- Each case runs in its own process. The original's after-test reset is not needed.

**Differently from the original**

- The `startswith` checks became slices with hand-counted lengths: `[:13]`, `[:19]`, `[:18]`. They have the same strength, but a wrong count fails with a message that does not show the mistake (probe s04). A failure shows the slice, not the whole line.
- `out == err == ""` became a single tuple check. A failure now names the element (`[1]: they differ…`) instead of a chained comparison.
- `test_catch_is_false` reads `capsys` inside the check, not in the act.
- `sys.stderr = None`, the broken `write` and the excepthook stay in place until the case's process ends. In the original they are restored before the test ends.
- The monkeypatch went from a `with` scope to `setattr(obj, "write", fn)`. ty checks `sys.stderr.write = fn` (and rejects the type), but it does not check a `setattr` with a string name. A typo in `"write"` passes both checkers (probe s14). Only the check's failure shows it.
- `Capture` is a class of mine, not pytest's `capsys`. It swaps `sys.*` like `capsys`, but it is built when the runner builds the input, after `before_each`.

## What the form cannot express

- **A setup that is undone inside the test.** `monkeypatch.context()` and `default_threading_excepthook()` both restore state after a few statements. The form allows no `with` other than `raises`/`warns`/`handling`, and nothing runs after the act. The port can make the change but cannot undo it. This file does not need the undo, because nothing reads the state afterwards. A test that checks the state after the restore could not be ported.
- **A check on a string prefix or suffix that names the expected text once.** `startswith(x)` must become `[:len(x)] == x`, with the length written by hand, or `startswith(x) is True`, which shows only `False` when it fails. The form can say this, but not with the original's single statement of the expected text.
- Nothing else. No item was left unported.

## Runner bugs

1. **The runner's own fork warning is blamed on the test.** FORM.md says "A warning that the test does not expect fails the case". The test raises no warning. `run.py:667` calls `os.fork()` for each check but the last, inside the case's `warnings.catch_warnings(record=True)`. When a second thread is alive, Python 3.12 warns `DeprecationWarning: This process (pid=…) is multi-threaded, use of fork() may lead to deadlocks in the child.`, and the case fails once for each fork. A test cannot expect this warning with `warns`, because `warns` wraps a statement of the act and the warning comes later (it fails with `expected DeprecationWarning matching 'multi-threaded'; no warning`). Any loguru test with `enqueue=True` and two or more checks hits this bug. Probes: `probes/bug_fork_warning/test_fork_warning.py` (two checks fail; one check passes; two checks after `join()` pass) and `test_fork_warning_expected.py`. Workaround: end the thread before the checks (`logger.remove()`).
2. **A function handed to the code by assignment counts as part of the act.** FORM.md says "A function … that the test defines and hands to the code (a callback, a format function, a sink) is a value: its body may hold any code but an assert." `called_inside()` in `run.py` counts a function as handed only when it is an argument of a call. `stream.write = broken_write` (a monkeypatch by assignment) is refused with `line 17 holds a \`raise\`: expect the code's own raise with \`with raises(…):\``, while `setattr(stream, "write", broken_write)` passes. Probe: `probes/bug_handed_by_assignment/test_handed_by_assignment.py`. Workaround: `setattr`. The cost is that ty no longer checks the attribute.
3. **`sys.stderr = None` in the act breaks the runner's own flush.** `run.py:665-666` calls `sys.stdout.flush(); sys.stderr.flush()` before each check's fork. With `sys.stderr = None` and two or more checks, the case fails with `the act raised AttributeError: 'NoneType' object has no attribute 'flush'`. That error points at the first check line, not at anything in the act. After the case, `run_case` makes the same flush outside its `try` (`run.py:803-804`). The case's process then leaves by an uncaught exception instead of `os._exit(0)`, so it runs the atexit handlers that `os._exit` skips. Its results still arrive. Probes: `probes/bug_stderr_none/test_stderr_none.py` (one check passes, two checks fail) and `test_stderr_none_atexit.py`: only the case with `sys.stderr = None` writes its atexit line to `ledger_atexit.txt`, and the control does not. In the port, `test_no_sys_stderr` avoids the bug only because the tuple check makes it a single check. Splitting the check into `out == ""` and `err == ""` would trigger the bug. loguru's own `atexit.register(logger.remove)` also runs in that case's process.

## Probes

The seeded mistakes are in `probes/seeded/`, one per file. Runner output is in `RUNNER.txt`, ty output in `TY.txt`, and pyright output in `PYRIGHT.txt` (run with `PYTHONPATH` / extra paths set to `F` and `tests/`).

| Seeded mistake | Runner | ty | pyright |
|---|---|---|---|
| s01 misspelled input `enqeue=True` | refused: `gives 'enqeue', which is no input …; did you mean 'enqueue'?` + missing `enqueue` | `unknown-argument` + `missing-argument` | `No parameter named "enqeue"` + `Argument missing` |
| s02 `enqueue="True"` (str) | refused: `gives enqueue='True' (str), and … declares bool` | `invalid-argument-type` | `reportArgumentType` |
| s03 `enqueue` left out | refused: `gives no 'enqueue', an input … with no default` | `missing-argument` | `reportCallIssue` |
| s04 slice one short, `lines[-2][:17] == "ValueError: Error!"` | fail: `they differ from character 17: expected '!', read ''` | silent | silent |
| s05 wrong handler id `#1` | fail: `they differ from character 37: expected '1 ---', read '0 ---'` | silent | silent |
| s06 input by position `case(test, False)` | refused: `passes 1 input(s) by position; name each input …` | silent | silent |
| s07 monkeypatch by assignment `sys.stderr.write = broken_write` | refused: `line 19 holds a \`raise\`…` (runner bug 2) | `invalid-assignment` on the attribute | silent |
| s08 `sys.stderr = None` left out | fail: `[1]: they differ from character 0: expected '', read '--- Logging error in'` | silent | silent |
| s09 `raises(TypeError, …)` for a `ValueError` | fail: `the act raised ValueError: Error!` (does not name the expected `TypeError`) | silent | silent |
| s10 enqueued, two checks, no `logger.remove()` | fail: the fork `DeprecationWarning` (runner bug 1); both checks pass | silent | silent |
| s11 enqueued, `time.sleep(0.1)` left out | fail: `expected 'A\nB\n', read ''` | silent | silent |
| s12 bare `assert re.match(…)` | refused: `a check compares a reading with the value you expect, with one operator …` | silent | silent |
| s13 `use(Writer())`, an instance, not the class | fail: `building an input raised TypeError: Writer.__call__() missing 1 required positional argument: 'message'` | `no-matching-overload` for `use` | `reportCallIssue` + `reportArgumentType` (`Extra parameter "message"`) |
| s14 `setattr(sys.stderr, "wirte", …)` | fail: the check reads loguru's error text | silent | silent |

Other probes: `probes/bug_fork_warning/`, `probes/bug_handed_by_assignment/`, `probes/bug_stderr_none/` (runner bugs 1-3), and `probes/excepthook_hides_thread_error/`. In that last probe, a thread error fails the case (`an error that no code saw: in a thread, ValueError: in the thread`). With the original's excepthook installed, the same case passes. Each folder has its `OUTPUT.txt`.

## Line counts

| | Original | Port |
|---|---|---|
| Test file | 144 (`test_add_option_catch.py`) | 170 (`tests/test_add_option_catch.py`) |
| Helpers / fixtures | 41: `default_threading_excepthook` 19, `reset_logger` 12, `writer` 10 (conftest imports not counted; `capsys` and `monkeypatch` are pytest built-ins, 0 lines) | 71 (`tests/loguru_lab.py`, with `Capture` 16 lines and module imports) |
| Total | 185 | 241 |

Non-blank lines: the original has 103 + 35 = 138, and the port has 111 + 51 = 162. Most of the test file's growth is the 11 `case(…)` lines and the blank lines around them. Most of the helper's growth is `Capture`, which replaces pytest's `capsys`.
