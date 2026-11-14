# Port of loguru's tests/test_defaults.py

The whole file ported on the first natural attempt. All 14 cases passed, and the runner refused none. Only three kinds of change were needed: `pytest.raises` became `raises`, the `monkeypatch` fixture became `use(MonkeyPatch)`, and each parametrized value became one named `case(...)` line. I wrote negative controls to make sure the green run means something. A mutant, a check after the setup is undone, a type-strict `==`, a raise with the wrong message, a missing raise, an input of the wrong type and a test with no check all failed or were refused, as FORM.md says. I found no runner bug. I found one gap in FORM.md: it does not say that a `with raises(…)` alone counts as a check. The runner's refusal text says so.

## 1. Counts

| what | count |
|---|---|
| items pytest collects from the original | 14 (7 tests; 5 are parametrized) |
| cases in the port | 14 (12 `case(...)` lines + 2 tests with no inputs, which run under their own names) |
| passed / failed / refused / blocked | 14 / 0 / 0 / 0 (`14 checks held`) |
| tests accepted on the first natural attempt | 7 of 7 (14 of 14 cases) |
| tests refused or failed on the first attempt | 0 |
| items not ported | 0 |

Run: `cd tests && PY ROOT/run.py .`, exit 0.

## 2. Per test

The first natural attempt is the final form for every test. Nothing was changed after the first run. Runner's first response, verbatim (the whole run):

```
── test_defaults.py
 test_string  (test_defaults.py:12)
   ✓ string_test › env(key, str) == value   'test'
   ✓ string_empty › env(key, str) == value   ''
 test_bool_positive  (test_defaults.py:23)
   ✓ bool_positive_y › env(key, bool) is True   True
   ✓ bool_positive_1 › env(key, bool) is True   True
   ✓ bool_positive_TRUE › env(key, bool) is True   True
 test_bool_negative  (test_defaults.py:35)
   ✓ bool_negative_NO › env(key, bool) is False   False
   ✓ bool_negative_0 › env(key, bool) is False   False
   ✓ bool_negative_false › env(key, bool) is False   False
 test_int  (test_defaults.py:47)
   ✓ test_int › env(key, int) == 42   42
 test_invalid_int  (test_defaults.py:54)
   ✓ invalid_int_empty › raises   ValueError: Invalid environment variable 'INVALID_INT' (expected an integer): ''
   ✓ invalid_int_a › raises   ValueError: Invalid environment variable 'INVALID_INT' (expected an integer): 'a'
 test_invalid_bool  (test_defaults.py:69)
   ✓ invalid_bool_empty › raises   ValueError: Invalid environment variable 'INVALID_BOOL' (expected a boolean): ''
   ✓ invalid_bool_a › raises   ValueError: Invalid environment variable 'INVALID_BOOL' (expected a boolean): 'a'
 test_invalid_type  (test_defaults.py:84)
   ✓ test_invalid_type › raises   ValueError: The requested type 'float' is not supported
══ 14 cases: 14 passed · 0 failed · 0 refused · 0 blocked   (14 checks held)
```

| original test | first response | final form (changes from the original) | weaker? |
|---|---|---|---|
| `test_string[test]`, `[""]` | pass ×2 | `monkeypatch` → `monkeypatch: MonkeyPatch = use(MonkeyPatch)`; `value: str`; cases `string_test`, `string_empty` | **Stronger.** The form's `==` also requires the reading to be a `str`. pytest's `==` would accept any object that compares equal. |
| `test_bool_positive[y,1,TRUE]` | pass ×3 | same fixture change; cases `bool_positive_y/_1/_TRUE` | No. `is True` behaves as in Python. |
| `test_bool_negative[NO,0,false]` | pass ×3 | same; cases `bool_negative_NO/_0/_false` | No. |
| `test_int` | pass | same fixture change; no inputs, so it runs once under its own name | **Stronger.** `== 42` refuses `True` and `42.0`. The control `test_int_type_strict` shows that `True == 1` fails. |
| `test_invalid_int["", "a"]` | pass ×2 | `pytest.raises(...)` → `raises(...)`, same regex and same `re.search` meaning; cases `invalid_int_empty/_a` | No. |
| `test_invalid_bool["", "a"]` | pass ×2 | same; cases `invalid_bool_empty/_a` | No. |
| `test_invalid_type` | pass | same | No. |

## 3. Every `with` of the original

| original `with` | what it sets up | checks inside / after / both | in the port |
|---|---|---|---|
| `with monkeypatch.context() as context:` (all 7 tests) | a nested monkeypatch scope; `context.setenv(key, …)` sets one environment variable, which is undone at exit | **inside**, as the last statement(s) | unchanged: a setup of the form; checks stay at its end. The fork per case means the undo never protects another case, as it does in pytest. |
| `with pytest.raises(ValueError, match=…):` (test_invalid_int, test_invalid_bool, test_invalid_type), nested inside the one above | expects the one statement `env(key, …)` to raise | it *is* the check; it is the last statement of the act | `with raises(ValueError, match=…):`, still nested inside `monkeypatch.context()`. A multi-line call is accepted. |

The control `test_check_after_undo` shows that the form's "after the setup" reading is real. When the assert moves after the `with`, `env` reads `None` and the case fails with `a NoneType is not a str`.

## 4. Skips, marks and parametrization

- The original has no skips and no marks other than `parametrize`.
- Each of the 4 `@pytest.mark.parametrize("value", [...])` decorators became one named `case(test, value=<literal>)` line per value, 12 in all. The `value` input carries the annotation `str`, so the runner checks its type (control `wrong_input_type`: `value=1` is refused).
- Case names are mine. pytest ids `test_string[]` and `test_invalid_int[]` became `string_empty` and `invalid_int_empty`, because an empty id cannot be a variable name.
- Autouse fixtures from conftest: `reset_logger` and `reset_multiprocessing_start_method` became `before_each(...)`, from the helper `tests/loguru_conftest.py`. The probe `probes/before_each_order/` shows that two `before_each` calls in one file both run, in order. The session-scoped autouse `check_env_variables`, which warns when a `LOGURU_*` variable is present, has no counterpart in the form (no session scope). I left it out. None of these tests reads a `LOGURU_*` variable.

## 5. Suspected runner bugs

None. Each behaviour I tested matched FORM.md:

`probes/controls/test_controls.py`: every case must fail or be refused. Output:

```
── test_controls.py
 test_string_mutant  (test_controls.py:9)
   ✗ string_mutant › env(key, str) == value   test_controls.py:16 · assert at line 13
       expected  'test' (str)
       reading   'testx' (str)
       why       they differ from character 4: expected '', read 'x'
       inputs    value='test'
 test_check_after_undo  (test_controls.py:19)
   ✗ check_after_undo › env(key, str) == value   test_controls.py:26 · assert at line 23
       expected  'test' (str)
       reading   None (NoneType)
       why       a NoneType is not a str
       inputs    value='test'
 test_int_type_strict  (test_controls.py:29)
   ✗ test_int_type_strict › env(key, bool) == 1   test_controls.py:29 · assert at line 33
       expected  1 (int)
       reading   True (bool)
       why       a bool is not an int
 test_raises_wrong_match  (test_controls.py:36)
   ✗ test_raises_wrong_match › raises   test_controls.py:36
       what      expected ValueError matching "^Invalid environment variable 'INVALID_INT' \\(expected an integer\\)$"; it raised ValueError: Invalid environment variable 'INVALID_INT' (expected an integer): 'a'
                 at test_controls.py:40  with raises(ValueError, match=r"^Invalid environment variable 'INVALID_INT' \(expected an integer\)$"):
 test_raises_no_raise  (test_controls.py:44)
   ✗ test_raises_no_raise › raises   test_controls.py:44
       what      expected ValueError; the statement did not raise
                 at test_controls.py:48  with raises(ValueError):
 test_wrong_input_type  (test_controls.py:52)
   ! wrong_input_type refused   test_controls.py:59
       gives value=1 (int), and test_wrong_input_type declares str
 test_checks_nothing  (test_controls.py:62)
   ! refused:
       checks nothing: end it with `assert reading == expected`, or expect a raise with `with raises(…):`
     so none of its 1 case(s) runs
══ 7 cases: 0 passed · 5 failed · 2 refused · 0 blocked   (0 checks held)
```

`probes/before_each_order/test_before_each_order.py`: passes (`reading == 'first,second'`).

**A gap in FORM.md (not a runner bug):** FORM.md refuses "a test that checks nothing" and says that a check is an `assert`. It never says that a `with raises(…)` with no assert counts as a check. The runner accepts it: it counts `raises` as one of the "checks held", and its refusal text says "or expect a raise with `with raises(…):`". Three of the 7 tests here depend on that rule, which only the runner's output states.

## 6. The rules that cost the most

The costs were small. In order:

1. **One named `case` per parametrized value.** 4 decorator lines became 12 `case(...)` lines, and I named each value by hand. An empty-string value needs a made-up name:
   ```python
   string_test = case(test_string, value="test")
   string_empty = case(test_string, value="")
   ```
2. **A fixture is an input default, `use(factory)`.** pytest's built-in `monkeypatch` has no form counterpart, so the port imports `pytest.MonkeyPatch` as a library and names its type on each test:
   ```python
   def test_int(monkeypatch: MonkeyPatch = use(MonkeyPatch)):
   ```
   In the form, `monkeypatch.context()` is redundant: each case forks, so an environment change cannot reach another case. I kept it because it is the original's code and the form accepts it.
3. **Autouse fixtures become `before_each`, and session scope has no place.** The two resets became a helper module and two `before_each` lines. `check_env_variables` was dropped (see section 4).

## 7. What the original checks that the port cannot

- **The `LOGURU_*` environment warning** (`check_env_variables`, session scope). pytest prints it once per session as a warning. It never fails a test. The port has no session scope, so it does not check this.
- Nothing else. Every assert and every `pytest.raises` of the original has a check of the same meaning in the port. Two checks are stricter (section 2).

## 8. Size (non-blank lines)

| file | lines |
|---|---|
| original `tests/test_defaults.py` | 51 |
| port `tests/test_defaults.py` | 63 |
| helper `tests/loguru_conftest.py` | 14 |
| port + helper | 77 |

The port is 12 lines longer. The 12 `case` lines, the attribution header and the extra import and `before_each` lines outweigh the 4 decorator lines they replace.
