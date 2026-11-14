# Port of loguru's tests/test_colorama.py to the #979 form

All 79 pytest items became cases, and all 79 pass. The runner did nothing that FORM.md does not describe, so
`probes/` is empty. The port was cheap where the original is "set up, call one pure function in the assert". It was
costly in three places. First, the `should_wrap` tests call the code in the first assert and read the mock's
`.called` in the next one. The form runs each check in its own process, so the later checks read the mock before the
call. Six tests failed for this reason, and two other tests (5 cases) PASSED while they checked nothing. Second, the
form has no skip, so 24 Windows-only items run on Linux with `os.name` faked to `"nt"`. Third, every bare `assert x` /
`assert not x` (27 of 36 asserts) had to become `x is True` / `x is False`.

Evidence: `runs/run1.txt` (natural first attempt, `runs/test_colorama.v1.py`), `runs/run2.txt` (mechanical second
attempt, `runs/test_colorama.v2.py`, made by `runs/step2.pl`), `runs/run3.txt` and `runs/final.txt` (the port in
`tests/`). Controls are in `runs/control/` and `runs/edge/` (see section 7).

## 1. Counts

| | count |
|---|---|
| Items pytest collects from the original | 79 (55 pass, 24 skipped on Linux) |
| Cases in the port | 79 (73 `case(...)` lines + 6 tests with no input) |
| Final run | `79 cases: 79 passed · 0 failed · 0 refused · 0 blocked   (103 checks held)` |
| Tests accepted on the first natural attempt | 7 of 24 tests (43 cases), all passed |
| Tests refused on the first natural attempt | 17 of 24 tests (36 cases) |
| Tests failed on the first natural attempt | 0 |
| Second attempt (mechanical rewrite) | `79 cases: 66 passed · 12 failed · 1 refused · 0 blocked   (85 checks held)` |
| Items not ported | 0 |

## 2. Per test

"First response" is the runner's output for that test in `runs/run1.txt`, verbatim. The natural first attempt kept
the original body. It changed only what FORM.md plainly says: each `parametrize` value became a `case(...)`, and
each fixture became an input default with `use(...)` (`monkeypatch=use(pytest.MonkeyPatch)`,
`patch_colorama=use(patch_colorama)`). Python requires defaulted parameters last, so `monkeypatch` moved behind
`patched`/`expected`. The `skipif` decorators were kept in the first attempt, because FORM.md gives nothing to
replace them with.

### test_stream_wrapped_on_windows_if_no_vt_support (2 cases)
First response:
```
 test_stream_wrapped_on_windows_if_no_vt_support  (test_colorama.py:45)
   ! refused:
       line 45: a test takes no decorator; give its cases with `name = case(test_stream_wrapped_on_windows_if_no_vt_support, …)`
       line 54: a check compares a reading with the value you expect, with one operator: `assert reading == expected`, `in`, `<`, `is None`
     so none of its 2 case(s) runs
```
Final: decorator removed; `context.setattr(os, "name", "nt")` first in the setup; `assert ….AnsiToWin32.called is True`.
Weaker: yes. It runs on Linux with a faked `os.name`, not on Windows (section 4).

### test_stream_not_wrapped_on_windows_if_vt_support (2 cases)
First response:
```
 test_stream_not_wrapped_on_windows_if_vt_support  (test_colorama.py:66)
   ! refused:
       line 66: a test takes no decorator; give its cases with `name = case(test_stream_not_wrapped_on_windows_if_vt_support, …)`
       line 75: a check compares a reading with the value you expect, with one operator: `assert reading == expected`, `in`, `<`, `is None`
     so none of its 2 case(s) runs
```
Final: as above, with `assert ….AnsiToWin32.called is False`. Weaker: the same faked `os.name`. Without the fake
`os.name`, this test passes anyway on Linux, because `should_wrap` returns `False` before it looks at colorama.

### test_stream_is_none, test_is_a_tty, test_is_not_a_tty, test_is_a_tty_exception (1 case each)
First responses:
```
 test_stream_is_none  (test_colorama.py:86)
   ! refused:
       line 87: a check compares a reading with the value you expect, with one operator: `assert reading == expected`, `in`, `<`, `is None`
     so none of its 1 case(s) runs
 test_is_a_tty  (test_colorama.py:90)
   ! refused:
       line 91: a check compares a reading with the value you expect, with one operator: `assert reading == expected`, `in`, `<`, `is None`
     so none of its 1 case(s) runs
 test_is_not_a_tty  (test_colorama.py:94)
   ! refused:
       line 95: a check compares a reading with the value you expect, with one operator: `assert reading == expected`, `in`, `<`, `is None`
     so none of its 1 case(s) runs
 test_is_a_tty_exception  (test_colorama.py:98)
   ! refused:
       line 99: a check compares a reading with the value you expect, with one operator: `assert reading == expected`, `in`, `<`, `is None`
     so none of its 1 case(s) runs
```
Final: `assert should_colorize(None) is False`, `… is True`, `… is False`, `… is False`. The act is empty; the
runner accepts that. Weaker: no. `is True` is stricter than the truthiness test of the original.

### test_pycharm_fixed (5 cases) — accepted first
First response:
```
 test_pycharm_fixed  (test_colorama.py:102)
   ✓ pycharm_fixed__stdout__ › should_colorize(stream) is expected   True
   ✓ pycharm_fixed__stderr__ › should_colorize(stream) is expected   True
   ✓ pycharm_fixed_stdout › should_colorize(stream) is expected   False
   ✓ pycharm_fixed_stderr › should_colorize(stream) is expected   False
   ✓ pycharm_fixed_none › should_colorize(stream) is expected   False
```
Final: unchanged. Weaker: no.

### test_github_actions_fixed (5 cases) — accepted first
First response:
```
 test_github_actions_fixed  (test_colorama.py:117)
   ✓ github_actions_fixed__stdout__ › should_colorize(stream) is expected   True
   ✓ github_actions_fixed__stderr__ › should_colorize(stream) is expected   True
   ✓ github_actions_fixed_stdout › should_colorize(stream) is expected   False
   ✓ github_actions_fixed_stderr › should_colorize(stream) is expected   False
   ✓ github_actions_fixed_none › should_colorize(stream) is expected   False
```
Final: unchanged. Weaker: no.

### test_mintty_fixed_windows (5 cases)
First response:
```
 test_mintty_fixed_windows  (test_colorama.py:134)
   ! refused:
       line 134: a test takes no decorator; give its cases with `name = case(test_mintty_fixed_windows, …)`
     so none of its 5 case(s) runs
```
Final: decorator removed; `context.setattr(os, "name", "nt")` first in the setup. Weaker: yes, the faked `os.name`.
Without it, the 3 `expected=False` cases pass on Linux anyway.

### test_dumb_term_not_colored (5 cases) — accepted first
First response:
```
 test_dumb_term_not_colored  (test_colorama.py:149)
   ✓ dumb_term_not_colored__stdout__ › should_colorize(stream) is expected   False
   ✓ dumb_term_not_colored__stderr__ › should_colorize(stream) is expected   False
   ✓ dumb_term_not_colored_stdout › should_colorize(stream) is expected   True
   ✓ dumb_term_not_colored_stderr › should_colorize(stream) is expected   True
   ✓ dumb_term_not_colored_none › should_colorize(stream) is expected   True
```
Final: unchanged. Weaker: no.

### test_honor_no_color_standard (10 cases) — accepted first
First response:
```
 test_honor_no_color_standard  (test_colorama.py:164)
   ✓ honor_no_color_1__stdout__ › should_colorize(stream) is expected   False
   ✓ honor_no_color_1__stderr__ › should_colorize(stream) is expected   False
   ✓ honor_no_color_1_stdout › should_colorize(stream) is expected   False
   ✓ honor_no_color_1_stderr › should_colorize(stream) is expected   False
   ✓ honor_no_color_1_none › should_colorize(stream) is expected   True
   ✓ honor_no_color_empty__stdout__ › should_colorize(stream) is expected   True
   ✓ honor_no_color_empty__stderr__ › should_colorize(stream) is expected   True
   ✓ honor_no_color_empty_stdout › should_colorize(stream) is expected   True
   ✓ honor_no_color_empty_stderr › should_colorize(stream) is expected   True
   ✓ honor_no_color_empty_none › should_colorize(stream) is expected   True
```
Final: unchanged. The original's two comments inside the parametrize list sit between the case lines. Weaker: no.

### test_honor_force_color_standard (10 cases) — accepted first
First response:
```
 test_honor_force_color_standard  (test_colorama.py:186)
   ✓ honor_force_color_1__stdout__ › should_colorize(stream) is expected   True
   ✓ honor_force_color_1__stderr__ › should_colorize(stream) is expected   True
   ✓ honor_force_color_1_stdout › should_colorize(stream) is expected   True
   ✓ honor_force_color_1_stderr › should_colorize(stream) is expected   True
   ✓ honor_force_color_1_none › should_colorize(stream) is expected   False
   ✓ honor_force_color_empty__stdout__ › should_colorize(stream) is expected   False
   ✓ honor_force_color_empty__stderr__ › should_colorize(stream) is expected   False
   ✓ honor_force_color_empty_stdout › should_colorize(stream) is expected   False
   ✓ honor_force_color_empty_stderr › should_colorize(stream) is expected   False
   ✓ honor_force_color_empty_none › should_colorize(stream) is expected   False
```
Final: unchanged. Weaker: no.

### test_no_color_takes_precedence_over_force_color (1 case)
First response:
```
 test_no_color_takes_precedence_over_force_color  (test_colorama.py:208)
   ! refused:
       line 217 holds an assert inside the act: the act ends at the first check, and every check comes after it, at the end of the test or at the end of a `with`
       line 220: a check compares a reading with the value you expect, with one operator: `assert reading == expected`, `in`, `<`, `is None`
     so none of its 1 case(s) runs
```
Second response (`not` rewritten, act and checks still interleaved):
```
       line 217 holds an assert inside the act: the act ends at the first check, and every check comes after it, at the end of the test or at the end of a `with`
```
Final: each reading is captured in the act, while its own stream is `sys.__stderr__`. The two checks follow, at the
end of the setup:
```python
        context.setattr(sys, "__stderr__", stream_tty, raising=False)
        tty_colored = should_colorize(stream_tty)

        context.setattr(sys, "__stderr__", stream_not_tty, raising=False)
        not_tty_colored = should_colorize(stream_not_tty)
        assert tty_colored is False
        assert not_tty_colored is False
```
Weaker: no. Each call runs in the state that the original asserted in.

### test_mintty_not_fixed_linux (5 cases)
First response:
```
 test_mintty_not_fixed_linux  (test_colorama.py:224)
   ! refused:
       line 224: a test takes no decorator; give its cases with `name = case(test_mintty_not_fixed_linux, …)`
     so none of its 5 case(s) runs
```
Final: decorator removed, nothing else. Weaker: the skip on Windows is lost (section 4).

### test_jupyter_fixed (7 cases) — accepted first
First response:
```
 test_jupyter_fixed  (test_colorama.py:239)
   ✓ jupyter_fixed_stdout_false › should_colorize(stream) is expected   True
   ✓ jupyter_fixed_stderr_false › should_colorize(stream) is expected   True
   ✓ jupyter_fixed__stdout___false › should_colorize(stream) is expected   False
   ✓ jupyter_fixed__stderr___false › should_colorize(stream) is expected   False
   ✓ jupyter_fixed_stdout_true › should_colorize(stream) is expected   False
   ✓ jupyter_fixed_stderr_true › should_colorize(stream) is expected   False
   ✓ jupyter_fixed_none_false › should_colorize(stream) is expected   False
```
Final: unchanged. The `class Shell` and the `MagicMock` setup in the act were accepted. Weaker: no.

### test_jupyter_missing_lib (1 case) — accepted first
First response:
```
 test_jupyter_missing_lib  (test_colorama.py:270)
   ✓ test_jupyter_missing_lib › should_colorize(stream) is False   False
```
Final: unchanged. Weaker: no.

### test_dont_wrap_on_linux (2 cases)
First response:
```
 test_dont_wrap_on_linux  (test_colorama.py:280)
   ! refused:
       line 280: a test takes no decorator; give its cases with `name = case(test_dont_wrap_on_linux, …)`
       line 284: a check compares a reading with the value you expect, with one operator: `assert reading == expected`, `in`, `<`, `is None`
       line 285: a check compares a reading with the value you expect, with one operator: `assert reading == expected`, `in`, `<`, `is None`
     so none of its 2 case(s) runs
```
Second response (decorator removed, `not x` → `x is False`): **passed, but checked nothing in its second check.**
```
   ✓ dont_wrap_on_linux__stdout__ › should_wrap(stream) is False   False
   ✓ dont_wrap_on_linux__stdout__ › patch_colorama.win32.winapi_test.called is False   False
```
The second check ran in a process where `should_wrap` had never been called, because the first check's call ran
in a fork of its own. Final: `wrapped = should_wrap(stream)` in the act; `assert wrapped is False`;
`assert patch_colorama.win32.winapi_test.called is False`. Weaker: no, after the fix. The skip on Windows is lost.

### test_dont_wrap_if_not_original_stdout_or_stderr (3 cases)
First response:
```
 test_dont_wrap_if_not_original_stdout_or_stderr  (test_colorama.py:293)
   ! refused:
       line 293: a test takes no decorator; give its cases with `name = case(test_dont_wrap_if_not_original_stdout_or_stderr, …)`
       line 299: a check compares a reading with the value you expect, with one operator: `assert reading == expected`, `in`, `<`, `is None`
       line 300: a check compares a reading with the value you expect, with one operator: `assert reading == expected`, `in`, `<`, `is None`
     so none of its 3 case(s) runs
```
The second response is the same as for `test_dont_wrap_on_linux`: all 3 cases passed, and `winapi_test.called is False`
read the state before the call. Final: the call is in the act, and `os.name` is faked. Weaker: the faked `os.name`.

### test_dont_wrap_if_terminal_has_vt_support (2 cases)
First response:
```
 test_dont_wrap_if_terminal_has_vt_support  (test_colorama.py:309)
   ! refused:
       line 309: a test takes no decorator; give its cases with `name = case(test_dont_wrap_if_terminal_has_vt_support, …)`
       line 317: a check compares a reading with the value you expect, with one operator: `assert reading == expected`, `in`, `<`, `is None`
       line 318: a check compares a reading with the value you expect, with one operator: `assert reading == expected`, `in`, `<`, `is None`
     so none of its 2 case(s) runs
```
Second response (failed):
```
   ✗ dont_wrap_if_terminal_has_vt_support__stdout__ › patch_colorama.winterm.enable_vt_processing.called is True   test_colorama.py:319 · assert at line 316
       expected  True (bool)
       reading   False (bool)
       why       `False is True` does not hold
       inputs    patched='__stdout__'
```
Final: `wrapped = should_wrap(stream)` in the act, and `os.name` is faked. Weaker: the faked `os.name`.

### test_dont_wrap_if_winapi_false (2 cases)
The first response is the same refusal shape: the decorator at line 326, and the checks at lines 332 and 333. The
second response failed on `patch_colorama.win32.winapi_test.called is True` (reading `False`). The final form is the
same as above. Weaker: the faked `os.name`.

### test_wrap_if_winapi_true_and_no_vt_support, test_wrap_if_winapi_true_and_vt_check_fails (2 cases each)
First response (one of the two; the other is the same, at lines 359 and 367–369):
```
 test_wrap_if_winapi_true_and_no_vt_support  (test_colorama.py:341)
   ! refused:
       line 341: a test takes no decorator; give its cases with `name = case(test_wrap_if_winapi_true_and_no_vt_support, …)`
       line 349: a check compares a reading with the value you expect, with one operator: `assert reading == expected`, `in`, `<`, `is None`
       line 350: a check compares a reading with the value you expect, with one operator: `assert reading == expected`, `in`, `<`, `is None`
       line 351: a check compares a reading with the value you expect, with one operator: `assert reading == expected`, `in`, `<`, `is None`
     so none of its 2 case(s) runs
```
Second response: in each case, both `.called is True` checks failed, including the last check, which runs in the
case's own process. Final: the call is in the act, and `os.name` is faked. Weaker: the faked `os.name`.

### test_wrap_if_winapi_true_and_stream_has_no_fileno (2 cases)
First response: the same refusal shape (decorator at line 377; checks at lines 384–386). Second response: the
case failed on `winapi_test.called is True`, but `enable_vt_processing.called is False` **passed vacuously**, for
the same reason as `test_dont_wrap_on_linux`. Final: the call is in the act, and `os.name` is faked. Weaker: the
faked `os.name`.

### test_wrap_if_winapi_true_and_old_colorama_version (2 cases)
First response: the same refusal shape (decorator at line 398; checks at lines 406–407). Second response: failed on
`winapi_test.called is True`. Final: the call is in the act, and `os.name` is faked. The act's
`del patch_colorama.winterm.enable_vt_processing` was accepted. Weaker: the faked `os.name`.

## 3. Every `with` of the original

The original has 21 `with` statements, and all are `with monkeypatch.context() as context:`.

| where | count | what it set up | checks | in the port |
|---|---|---|---|---|
| fixture `patch_colorama` | 1 | `sys.modules["colorama"]`, `["colorama.win32"]` and `["colorama.winterm"]` → mocks, for the whole test | it holds the test (fixture `yield`) | Removed. The factory `patch_colorama()` assigns the three `sys.modules` items and returns the mock. Nothing undoes them; the case's process ends. |
| 18 tests | 18 | `setattr(sys, patched, stream)` plus environment items, mock return values, `builtins.__IPYTHON__`, or fake `IPython`/`ipykernel` modules | at the end of the setup only | Kept as written: the same `with monkeypatch.context()`, with checks at its end. In 10 tests, the first line inside is now `context.setattr(os, "name", "nt")`. |
| `test_no_color_takes_precedence_over_force_color` | 1 | `NO_COLOR`, `FORCE_COLOR`, and then `sys.__stderr__` twice | inside it, interleaved with the act | Kept. The two readings are captured in the act, and both checks sit at the end of the setup. |
| `test_jupyter_missing_lib` | 1 | `sys.stdout`, `builtins.__IPYTHON__` | at the end of the setup only | Kept as written. |

No check of the original sits after its `with`. The port adds no new `with`.

## 4. Skips, marks and parametrization

- **`@pytest.mark.parametrize`** (18 tests, 73 values) → 73 named `case(...)` lines at module level. Each name is
  `<test without "test_">` plus the value (`__stdout__`, `_stdout`, `_none` for `""`). The original's comments
  inside the lists stay between the case lines.
- **`@pytest.mark.skipif(os.name != "nt")`** (10 tests, 24 items, skipped on Linux) → refused as a decorator. The form
  has no skip. They run on every platform, with `context.setattr(os, "name", "nt")` first in their setup.
  **Different conditions:** the port checks loguru's branches under a faked `os.name` on Linux, not on Windows. The
  `colorama` modules are mocks in both. Control: without the fake, 16 of the 24 cases fail. The 8 that pass anyway are
  `stream_not_wrapped…` (2), `dont_wrap_if_not_original…` (3), and the three `expected=False` cases of
  `mintty_fixed_windows`.
- **`@pytest.mark.skipif(os.name == "nt")`** (2 tests, 7 items) → the decorator is removed and they run
  unconditionally. On Windows they would fail. The runner forks, so it does not run on Windows at all.
- **Autouse `clear_environment`** (test file) → `before_each(clear_environment)`, setup half only (`os.environ.clear()`).
- **conftest autouse `reset_logger`, `reset_multiprocessing_start_method`** → `before_each(...)`, setup halves only, in
  `tests/loguru_conftest.py`.
- **conftest session fixture `check_env_variables`** (warns if a `LOGURU_*` variable is set) → not ported. It
  guards the test environment and checks nothing of the code. `clear_environment` removes such variables before
  each case anyway.
- **Fixture `monkeypatch`** → `monkeypatch=use(pytest.MonkeyPatch)`. The port still imports `pytest` for this class.
- **Fixture `patch_colorama`** → `patch_colorama=use(patch_colorama)`, a factory with no undo.

## 5. Suspected runner bugs

None. Every refusal, failure and pass matched FORM.md. I tested two claims that this port relies on, in
`runs/edge/test_edge.py`. Both held:

```
 test_untyped  (test_edge.py:7)
   ! misspelled refused   test_edge.py:13
       gives 'pached', which is no input of test_untyped; did you mean 'patched'?
       gives no 'patched', an input of test_untyped with no default
   ! missing refused   test_edge.py:14
       gives no 'patched', an input of test_untyped with no default
 test_os_name_left_patched  (test_edge.py:17)
   ✓ test_os_name_left_patched › flag.called is True   True
   ✓ test_os_name_left_patched › os.name == 'nt'   'nt'
```
The parameters are unannotated, but misspelled and missing inputs are still refused at run time. A case that sets
`os.name = "nt"` does not disturb the runner.

The vacuous passes in section 2 are not a runner bug: FORM.md says "no check can change what another reads". They are
the form's largest hazard for this file (section 6).

## 6. The rules that cost the most

1. **"Each check runs in a process of its own" together with "the act ends at the first check."** The original
   calls the code in the first assert (`assert not should_wrap(stream)`) and reads the mock's side effect in the next
   one (`assert patch_colorama.win32.winapi_test.called`). Under the form, the call in a check is not act, and its
   effect does not reach the next check. This cost 8 tests. On the second attempt, 6 tests (12 cases) failed. The
   costly part was the 2 tests (5 cases) that **passed** while their `.called is False` check read the state before
   the call, plus one more vacuous check inside a failing case. Nothing in the run marks a check that reads a
   value that another check changed. A porter who only rewrote `not x` would ship 5 cases that check less than the
   original. The fix is a capture in the act:
   ```python
   wrapped = should_wrap(stream)
   assert wrapped is False
   assert patch_colorama.win32.winapi_test.called is False
   ```
2. **No decorator, and no skip in the form.** 12 tests carry a `skipif`. The form offers no way to say "not on this
   platform". So I had to choose between dropping 24 items and running them under a faked `os.name`. The port runs
   them under different conditions, and the 7 Linux-only items lose their guard.
3. **A check is one comparison.** 27 of the 36 asserts are bare truthiness (`assert x`, `assert not x`). They became
   `x is True` / `x is False`. This was mechanical and stricter, but it touched 15 of the 24 tests and caused most of
   the 36 first-attempt refusals.
4. (Smaller) **A parametrize value becomes one named line.** This adds 73 case lines and 73 names to choose.

## 7. What the original checks that the port cannot

- **The Windows-only behaviour on a real Windows host** (24 items). The port checks the same branches with a faked
  `os.name`. Anything else in the call path that reads the platform at import time (for example `os.path`) stays
  POSIX.
- **That the Linux-only tests are skipped on Windows** (7 items). The port has no platform guard.
- **Undo of the environment and the monkeypatches after each test.** pytest checks implicitly that the next test
  starts clean. The form ends the process instead, so no undo of `patch_colorama`, `clear_environment` or
  `reset_logger` is exercised.
- The session warning on `LOGURU_*` variables (`check_env_variables`).

Controls that show the port still catches defects (scratch copies in `runs/control/`; the copy left there has the
last mutation):

| mutation | result |
|---|---|
| none, with a hostile environment (`NO_COLOR=1 FORCE_COLOR=1 CI=1 GITHUB_ACTIONS=1 PYCHARM_HOSTED=1 TERM=dumb`) | `79 passed` |
| `before_each(clear_environment)` removed, with the same environment | `65 passed · 14 failed` |
| `context.setattr(os, "name", "nt")` removed | `63 passed · 16 failed` |
| `pycharm_fixed_stdout` given `expected=True` | `78 passed · 1 failed` |

## 8. Size (non-blank lines)

| file | lines |
|---|---|
| original `tests/test_colorama.py` | 313 |
| port `tests/test_colorama.py` | 327 |
| helper `tests/loguru_conftest.py` (only the 5 stream classes and 2 resets the file uses) | 30 |

The port is 14 lines longer. The 18 parametrize decorators take 106 lines. They became 89 lines of `case(...)` calls (65 one-line,
8 wrapped over 3 lines). The port also drops the 12 `skipif` lines and the fixture's `with`. It gains lines from test
signatures wrapped by length, the 8 `wrapped = should_wrap(stream)` captures, the `before_each` registrations, the
imports and the platform comment. Type checking of the case lines was not exercised: no type
checker is installed in this environment, and the port's parameters are unannotated, like the original's.
