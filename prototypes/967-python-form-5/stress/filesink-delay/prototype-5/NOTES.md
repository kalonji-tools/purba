# filesink-delay → prototype 5: notes

All 10 pytest items are ported as 10 cases, and all 10 pass, with 20 checks (`OUTPUT.txt`). The file has no parametrize, so each test is one case under its own name. The form forced five changes. A mid-test assert becomes a local read in the act and a check at the end. `assert not x` becomes `assert x is False`. The `check_dir` helper becomes checks written out at each test's end. `with freeze_time(…)` becomes a call that starts the freeze and never stops it. `"7" * 20 + "\n"` becomes a 21-character literal. The biggest gap is cost, not meaning. `tmp_path` is a stock, so two checks may read it only after a checkpoint. The port's `TmpPath` copies the whole directory before each check and copies it back after, although every check only reads. Five runner bugs were found; the worst is that the "two checks read a stock with no checkpoint" refusal happens at run time and is counted as a failure.

## Original test → cases → checks

The original count is the number of asserts that run, including the ones inside `check_dir`.

| Original test | Cases | Original asserts | Port checks | What changed |
|---|---|---|---|---|
| `test_file_not_delayed` | 1 | 2 | 2 | The mid-test read is kept in `before`, then checked at the end |
| `test_file_delayed` | 1 | 2 | 2 | `not file.exists()` → `existed = file.exists()` … `assert existed is False` |
| `test_compression` | 1 | 3 (`check_dir`: count, name in dir, name not seen) | 1 | `sorted(os.listdir(tmp_path)) == ["file.log.gz"]` |
| `test_compression_early_remove` | 1 | 1 (`check_dir` size=0) | 1 | `list(tmp_path.iterdir()) == []` |
| `test_retention` | 1 | 1 | 1 | as above |
| `test_retention_early_remove` | 1 | 1 | 1 | as above |
| `test_rotation` | 1 | 7 (count; per file: in, not seen, content) | 3 | listing + 2 content checks; freeze started in the act |
| `test_rotation_early_remove` | 1 | 1 | 1 | as above |
| `test_rotation_and_retention` | 1 | 10 | 4 | listing + 3 content checks; literals written out |
| `test_rotation_and_retention_timed_file` | 1 | 10 | 4 | as above |
| **Total** | **10** | **38** | **20** | |

## First attempt: what the runner said

The first attempt is in `probes/first-attempt/` (`test_filesink_delay.py`, `loguru_lab.py`, output in `first-attempt.out.txt`). It kept the original's bodies. `check_dir` stayed a helper. `freeze_time` stayed a fixture that returns a context manager. `tmp_path` was a plain factory, `use(tmp_path)`. Result: `10 cases: 0 passed · 0 failed · 10 refused · 0 blocked`.

| Test | Runner text (verbatim) | What I changed |
|---|---|---|
| `test_file_not_delayed` | `line 20 holds an assert before the end: every check goes at the end of the test` | `before = file.read_text()` in the act; `assert before == ""` at the end |
| `test_file_delayed` | `line 28 holds an assert before the end: every check goes at the end of the test` | `existed = file.exists()` in the act; checked at the end |
| `test_compression`, `test_compression_early_remove`, `test_retention`, `test_retention_early_remove`, `test_rotation_early_remove` | `checks nothing: end it with \`assert reading == expected\`, or expect a raise with \`with raises(…):\`` | `check_dir(…)` replaced by asserts at the end |
| `test_rotation`, `test_rotation_and_retention`, `test_rotation_and_retention_timed_file` | `line 69 holds a \`with\` other than \`raises\`, \`warns\` or \`handling\`: call what it sets up, or make it a fixture with use(…)` (lines 91, 110 for the other two) | `frozen = freeze_time("…")` called in the act |

The runner did not name the second fault of `test_rotation` and the two others: each also checks nothing (see runner bug 3).

**Attempt 2** (`probes/attempts/attempt2_test_filesink_delay.py.txt`, output `attempt2.out.txt`): `TmpPath` became a `Samples[Path]` stock. Result: `5 passed · 0 failed · 5 refused`.

- `test_file_delayed`: `line 31: a check compares a reading with the value you expect, with one operator: \`assert reading == expected\`, \`in\`, \`<\`, \`is None\``. I wrote `assert not existed`. Changed to `assert existed is False`.
- `test_compression`, `test_rotation`, both `rotation_and_retention` tests: `line 40: the reading a comprehension over \`tmp_path.iterdir()\`: the act has one path, so it runs over literal data, with no \`if\``. I wrote `sorted(p.name for p in tmp_path.iterdir())`. FORM.md says a reading "may compute", so I did not expect the comprehension rule to reach the check. Changed to `sorted(os.listdir(tmp_path))`.
- Both `rotation_and_retention` tests, 3 times each: `line 102: the expected value computes \`'7' * 20 + '\n'\`; an expected value is a literal, so the test cannot compute what the code computes; write a literal, or take it as an input of the case`. Changed to `"77777777777777777777\n"`.

**Attempt 3** (`attempt3.out.txt`): `7 passed · 3 failed · 0 refused`. Each check of `test_rotation` and both `rotation_and_retention` tests printed `! … what      tmp_path is a stock with no checkpoint, and more than one check reads it`. I made `TmpPath` a `Checkpoints[Path, Path]` too: `checkpoint` copies the directory aside, `restore` copies it back.

**Attempt 4** (`attempt4.out.txt`, the port): `10 passed · 0 failed · 0 refused · 0 blocked (20 checks held)`.

ty then said `error[too-many-positional-arguments]: Too many positional arguments to \`object.__init__\`: expected 1, got 11` on the `reset_logger` line that calls `loguru.logger.__init__(…)`, and pyright said `error: Expected 0 positional arguments (reportCallIssue)`. loguru's stub declares no `__init__` arguments. The line carries both coded ignores. ty also said `invalid-return-type` on `freezegun.freeze_time(…).start()`, which returns a union of three factory types; the port uses `cast(FrozenDateTimeFactory, …)`. `CHECKERS.txt` now shows both clean.

## Refusals and forced changes

| Original | What forced a change | Port |
|---|---|---|
| `assert file.read_text() == ""` between `logger.add` and `logger.debug` | "an assert before the end" | `before = file.read_text()` in the act; `assert before == ""` at the end |
| `assert not file.exists()` mid-test | an assert before the end, and a check is one comparison | `existed = file.exists()`; `assert existed is False` |
| `check_dir(tmp_path, files=[…], size=…)`, a helper with `if`, `for` and asserts | "checks nothing"; a check is an assert at the test's end, one comparison each | one listing check, `sorted(os.listdir(tmp_path)) == [...]`, plus one content check per file whose content the original checks |
| `with freeze_time("…") as frozen:` around 3 to 5 statements | "a `with` other than `raises`, `warns` or `handling`" | `frozen = freeze_time("…")` in the act; the freeze is never stopped |
| `freeze_time` fixture that takes the date when it is used | `use(factory)` calls `factory()` with no argument, so a fixture cannot take the date | `freeze_time(date)` is a plain helper the act calls |
| `monkeypatch.context()` that undoes the patches | no pytest `monkeypatch`; nothing runs after a case | `mock.patch.object(…).start()`, never stopped; the case's process ends |
| `"7" * 20 + "\n"` as expected value | "an expected value is a literal" | `"77777777777777777777\n"` |
| pytest's `tmp_path` | no built-in; the form says state outside the process is a stock | `TmpPath(Samples[Path], Checkpoints[Path, Path])` in `loguru_lab.py` |
| more than one check reads `tmp_path` | "a stock with no checkpoint, and more than one check reads it" | `TmpPath.checkpoint` copies the directory with `shutil.copytree`; `restore` copies it back |
| autouse `reset_logger`, run before and after | — | `before_each(reset_logger)`; the after-reset is gone, the process ends |

## Fidelity gaps

**Less than the original**

- `freeze_time` keeps one date format (`%Y-%m-%d`) and drops the other two, the refusal of an aware date, `include_tm_zone`, `tm_gmtoff_override` and the Windows `struct_time` fix. This file uses none of them. The helper is called by the act, so it must have one path, and those options need `if` and `try`.
- The original's `check_dir` asserts that no expected name is listed twice (`filepath not in seen`). That checks the test's own data, not loguru. The port's list equality would fail on a duplicate expected name, so nothing about loguru is lost.
- No check was dropped.

**More than the original**

- Every check runs after another fails. The original stops at the first failing assert in `check_dir`. Seeded S12 shows a content check that raises `FileNotFoundError` and the listing check after it still runs and passes.
- A passing check prints its reading (`OUTPUT.txt`).
- Each case gets a new directory that is removed after it. pytest keeps the last three sessions' `tmp_path` directories.
- Each case runs in its own process, so `reset_logger` and the freeze cannot leak into another case.

**Differently from the original**

- The mid-test reads (`file.read_text()`, `file.exists()`) are taken at the same point as in the original, but compared at the end.
- `len(set(dir.iterdir())) == size` and the per-name `in` checks become one list equality on sorted names. It says the same: exactly these files, no more.
- The original's checks run after the `with freeze_time` block has ended: the clock is real again, and `open`, `get_ctime`, `set_ctime` and `fake_localtime` are restored. In the port the freeze and the patches are still on while the checks run, and while `TmpPath` copies and removes its directory. The readings are file names and file contents, so the result does not change. A manual `cm.__enter__()` … `cm.__exit__(None, None, None)` in the act would end the freeze before the checks, but the runner's hint says to call what the `with` sets up, and nothing in this file reads time after the block.
- The expected strings are 21-character literals. A reader must count the digits; S11 shows that a miscount is caught only as a mismatch at run time.
- The temporary directory is `filesink-delay-XXXXXXXX` from `tempfile.mkdtemp`, not pytest's `test_rotation0`. No file name in this file depends on it.
- Each of the 16 checks that names `tmp_path` runs between a checkpoint and a restore, even a test's only check: one `copytree` before it, one `rmtree` and one `copytree` after. That is 32 directory copies in a run whose checks only read.
- Sleeps and ticks are unchanged: 9 × `time.sleep(0.05)` and 9 × `frozen.tick(0.05 s)` in each `rotation_and_retention` test.

## What the form cannot express

- **A reusable check helper.** `check_dir` is one assertion procedure that 8 tests share. The form has no way to say "these checks, for this directory": each check is a single comparison written at the test's end. The port writes out 16 checks by hand where the original makes 8 calls.
- **A conditional check.** `check_dir`'s `content=None` means "do not check this file's content". A check has no `if`, so the port says it by leaving the check out. The test author must make that choice per file, not the helper.
- **A fixture that takes an argument at the point of use.** The original's `freeze_time` fixture returns a function and takes the date where the test calls it. `use(factory)` calls `factory()` with no arguments, so the date can only reach a fixture by one fixture per date. The port moves the call into the act instead.
- **A read-only check of a stock.** The checks only read `tmp_path`, but the form cannot say so. A stock with more than one check needs a checkpoint, and the only general checkpoint for a directory is a full copy.
- **A scope that ends before the checks.** `with` is refused, and the act cannot end the freeze except through a manual `__exit__` call. The form has no form for "this setup ends here".

## Runner bugs

1. **The "stock with no checkpoint" fault is counted as failed, and found only at run time.** FORM.md: "with no checkpoint, two checks that read one stock are refused", and "The run ends with a count, `N cases: … passed · … failed · … refused · … blocked`". The runner runs the act first, on the stock, then prints `!` for each check with `tmp_path is a stock with no checkpoint, and more than one check reads it`, and counts the case as **failed**: attempt 3 ended `7 passed · 3 failed · 0 refused`. The fault is visible in the source, so it could be refused before the run. In `run.py`, `hook()` emits `status="refused"` (line 655), and `report()` turns every status that is not `pass` or `blocked` into `fail` (line 856). Probe: `probes/stock-two-checks/test_stock_two_checks.py` (`test_two_checks_name`, `run.out.txt`: `3 passed · 1 failed · 0 refused`). Port workaround: give `TmpPath` a checkpoint.
2. **The stock rule matches the input's name only.** A check that reads the stock through an alias (`same = d`) or a derived local (`f = d / "a"`) is not seen as reading it. Two such checks run with no checkpoint, and each but the last runs in a fork while the stock is shared. FORM.md says "a check that reads a stock". Probe: same file, `test_two_checks_alias` and `test_two_checks_derived`, both pass. The port is not hurt: `TmpPath` has a checkpoint. But `test_file_not_delayed`'s and `test_file_delayed`'s checks of `file` (derived from `tmp_path`) run as plain checks, with no checkpoint.
3. **A refusal does not always name every fault.** FORM.md: "A refusal names every fault at once." A test with a foreign `with` and no check is refused for the `with` only; "checks nothing" is not named. Probe: `probes/every-fault/test_every_fault.py` (`test_with_and_no_check`). The control in the same file (`test_mid_assert_and_not_form`) does name both of its faults. Seen first on `test_rotation` and the two `rotation_and_retention` tests of the first attempt.
4. **A function the test calls is not checked for one path.** FORM.md: "A function that the test calls itself, under any name, is part of the act, and has one path too." A local helper with `if`, and an imported helper with `if` and `assert`, both run unrefused. Probe: `probes/helper-paths/test_helper_paths.py`. Seeded S10 shows the effect on this file: the original `check_dir`, kept as a helper, runs and the case passes with its trivial last check. A failing assert in such a helper shows as `the act raised AssertionError`, not as a check. The port does not use this hole: `freeze_time` and `reset_logger` have one path.
5. **A default of the wrong type is not checked (arguable).** FORM.md: "The runner refuses, before the run: … an input of the wrong type", and "The runner checks each input's type too, when it can read the annotation at run time." `def test_default_of_wrong_type(n: int = "3")` runs, and fails only at its check (`a str is not an int`). The same value given by a case is refused. A `use()` fixture of the wrong type is caught when the case runs, as a failure: `the fixture of p gives a str, and the test declares Path`. Probe: `probes/default-type/test_default_type.py`. Seeded S9 (a forgotten `use(…)`) is the case of this file: the runner does not refuse it, and the act fails with `unsupported operand type(s) for /: 'ABCMeta' and 'str'`.

Smaller frictions, not contradictions of FORM.md:

- `python run.py --help` treats `--help` as a file and ends in a `FileNotFoundError` traceback.
- The comprehension refusal reads `the reading a comprehension over …`: a word is missing.
- Seeded S5 gives `tmp_path="/nonexistent"`. Besides the type fault, the runner adds `line 56 reads only tmp_path, a value the act cannot change; read what the act made`. The check reads a directory that the act changes; the rule judged it by the wrong value's type.
- A stock that is both `Samples` and `Checkpoints` takes its value from `sample()`; `open()` is never called. FORM.md says a stock may be "both" but not which method gives the value. Probe: `test_both_which_value` in `probes/stock-two-checks/`.

## Probes

Seeded mistakes are in `probes/seeded/test_seeded.py` (run with `PYTHONPATH=../../tests`; outputs in `run.out.txt`, `ty.out.txt`, `pyright.out.txt`).

| # | Seeded mistake | Runner | ty | pyright |
|---|---|---|---|---|
| S1 | wrong expected file name (`300000` for `350000`) | ✗ failed: `[0]: they differ from character 26: expected '00000.log', read '50000.log'` | silent | silent |
| S2 | `assert existed == 0` (a bool as an int) | ✗ failed: `a bool is not an int` | silent | silent |
| S3 | forgot `logger.remove(i)` before checking for `.gz` | ✗ failed: reading `['file.log']` | silent | silent |
| S4 | case input misspelled `tmp_pth=` | ! refused: `gives 'tmp_pth', which is no input of test_s4_s5_case_inputs; did you mean 'tmp_path'?` and `gives no 'tmp_path', an input of test_s4_s5_case_inputs with no default` | `unknown-argument` + `missing-argument` | `No parameter named "tmp_pth"` + `Argument missing for parameter "tmp_path"` |
| S5 | case input of the wrong type, `tmp_path="/nonexistent"` | ! refused: `gives tmp_path='/nonexistent' (str), and test_s4_s5_case_inputs declares Path`, plus the misleading "reads only tmp_path" line | `invalid-argument-type` | `reportArgumentType` |
| S6 | `freeze_time(datetime.date(2001, 2, 3))` | ✗ failed in the act: `TypeError: strptime() argument 1 must be str, not datetime.date`; the check is blocked | `invalid-argument-type` | `reportArgumentType` |
| S7 | bare `assert not file.exists()` | ! refused: `line 76: a check compares a reading with the value you expect, with one operator: …` | silent | silent |
| S8 | `tmp_pth / "file.log"` in the act | ✗ failed in the act: `NameError: name 'tmp_pth' is not defined` | `unresolved-reference` | `reportUndefinedVariable` |
| S9 | forgot `use(…)`: `tmp_path: Path = TmpPath` | ✗ failed in the act: `TypeError: unsupported operand type(s) for /: 'ABCMeta' and 'str'` (not refused: bug 5) | `invalid-parameter-default` | `reportArgumentType` |
| S10 | original `check_dir` kept as a helper (`if`, `for`, `assert`) | ✓ **passed**: not refused (bug 4) | silent | silent |
| S11 | `"7" * 20` miscounted as 19 sevens | ✗ failed: `they differ from character 19: expected '\n', read '7\n'` | silent | silent |
| S12 | content check of `file.lg` (typo of `file.log`) | ✗ failed: `reading it raised FileNotFoundError: …`; the next check still runs and passes | silent | silent |

Totals: runner `12 cases: 1 passed · 8 failed · 3 refused · 0 blocked`; ty 6 diagnostics; pyright 6 errors. Only S10 passes every tool.

Runner-bug probes: `probes/stock-two-checks/`, `probes/every-fault/`, `probes/helper-paths/`, `probes/default-type/`, each with its `run.out.txt`.

## Line counts

Non-blank lines. The original's fixtures count only what this file uses from `conftest.py`: its imports (13), `check_dir` (14), `reset_logger` (11) and `freeze_time` (82). `writer` and `default_threading_excepthook` are not counted.

| | Original | Port |
|---|---|---|
| Test file | 86 (`test_filesink_delay.py`, 117 lines in all) | 91 (`tests/test_filesink_delay.py`, 124 lines in all) |
| Helpers / fixtures | 120 (`conftest.py` parts) | 73 (`tests/loguru_lab.py`, 95 lines in all) |
| Total | 206 | 164 |

The test file grows by 5 lines: 16 written-out checks replace 8 `check_dir` calls. The helpers shrink because `freeze_time` drops the options this file does not use, and because no undo is needed; `TmpPath` adds about 15 lines for its checkpoint and restore.
