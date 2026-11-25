# Rater 1's report

Word for word, as the rater returned it. In this rater's folder `PORT_R` = purba A, `PORT_S` = purba B, `PORT_T` = oxitest ([`KEY.txt`](KEY.txt)).

Rater 1 report. All work is under `<eval>/blind/rater1/_review/`

Summary: PORT_R and PORT_S contain the same test file. They differ only in the words the runner prints about the root and about `test-files`. The S wording tells you what to write; the R wording only names the fault. The purba ports (R/S) were best when a run starts in an unexpected folder, when the file-pattern setting is wrong, and when a test fails. Their weak point is writing tests: every case is its own function, helpers cannot be imported from a sibling module, and a missing exception shows up as a confusing project error. T (oxitest) has the clearest test intent and a clean type check (0 diagnostics). But with a wrong pattern or an empty list it runs no tests and still exits 0, and it does not say why it chose its rootdir. The original is the most compact, but its fixtures come from conftest without being named in the file, and it fails silently (exit 5, no reason given) when the pattern is wrong.

1. Scores (5 = best; on both complexity rows, 5 = simplest)

| criterion | ORIG | R | S | T |
|---|---|---|---|---|
| 1 Readability | 4 | 4 | 4 | 3 |
| 2 Clarity | 4 | 4 | 4 | 5 |
| 3 Ease of use | 4 | 3 | 3 | 4 |
| 4 Tester complexity | 3 | 3 | 3 | 2 |
| 5 Impl. complexity | 2 | 3 | 3 | 2 |
| 6 Min test | 5 | 4 | 4 | 4 |
| 7 Fidelity | — | 4 | 4 | 4 |
| 8 Run start | 3 | 4 | 5 | 2 |
| 9 Test files | 2 | 4 | 5 | 2 |
| 10 Seeded failures | 4 | 5 | 5 | 4 |

2. Per version

- **ORIGINAL:** 67 non-blank lines plus a 22-line conftest. Adding a case took one line (`("g", "{message}!", "g!\n")`).
  - Weak: the `writer` argument appears with no visible source.
  - Checker noise: pyright reports 9 errors in the conftest before any edit.
  - The failure output is 64 lines, because it reprints the whole parametrize decorator.
- **R:** 153 non-blank lines in 20 plain functions, with no framework import. A failure prints 4 lines: `expected 'Nope!\n'` / `reading 'Nope\n'`.
  - Weak: adding one case means a new 5-line function.
  - A helper module next to the tests fails to load (`ModuleNotFoundError: No module named 'tests'`).
  - pyright errors went from 26 to 28 after two new tests (`Cannot access attribute "read" for class "FunctionType"`).
- **S:** same as R, except the messages say how to fix the setting. Example: `test-files under [tool.purba] is a string; write a list, such as ["*_test.py"]`.
- **T:** named, typed cases (`no_fields=FormatCase(message="b", fmt="Nope", expected="Nope!\n")`). Each assert has a message, and ty and pyright both report 0.
  - Weak: setup cost in the file. A `TYPE_CHECKING`/`else` runtime stand-in, 4 per-checker ignore comments, a dataclass for each parametrize, and the special `__fixtures__.py` file.
  - Strict-mode errors are bare tags: `bare-assert line 78`, `dict-parametrize`.

3. Criteria 8 and 9

Criterion 8, run start:
- ORIG (3): from `tests/`, the output says `rootdir: …/O`, but the ids start at `test_add_option_format.py::`. With only `.git`, pytest ignores it, and nothing says why.
- R (4): `⚠ root …/git_R, from .git; PURBA_IMPLICIT_ROOT=1; no [tool.purba] found`. Ids start at the root in all 6 runs. Without the variable, the run stops with exit 3 and `add it to ../pyproject.toml`.
- S (5): `…none_S/tests is the working folder, …; the id of each test starts here, and moves with the working folder`. It gives the full reason.
- T (2): it prints only `rootdir:`. That is `git_T/tests` from tests/, with no reason given. The ids are always absolute paths of about 170 characters. Removing pyproject.toml also turns off `strict` without any notice.

Criterion 9, test files:
- ORIG (2): `["test*.py"]` and `[]` each give `collected 0 items`, exit 5, and no reason. The string form is accepted without comment.
- R (4): all three wrong settings are refused with exit 4, e.g. `test*.py is not a pattern purba takes; nearest: test_*.py`. But `an empty list` does not say what to write.
- S (5): `is an empty list, so no file is a test file; list at least one of test_*.py, *_test.py, tests.py and test.py`.
- T (2): `["test*.py"]` and `[]` both print `no tests ran` with **exit 0**. The string form gets a clear error (`expected a sequence`, exit 4). I found the key `python_files` only from the unknown-key error, which lists every valid key.

4. Ranking: **S > R > ORIGINAL > T**

Weighting: Clarity, Ease of use, Seeded failures and Fidelity count ×2; every other criterion counts ×1. Fidelity is left out of the cross-version total, because the original has no fidelity score.
- Totals out of 60: S 48, R 46, ORIG 43, T 41.
- Ports only, including fidelity: S 56, R 54, T 49.

5. Minimum working test

All four came to 6 non-blank lines and 1 import, using `out = []; logger.add(out.append, format="{message}"); logger.debug("a")`. All four pass.
- ORIG: the tester needs `test_` and `assert`.
- R/S: the tester also needs either `PURBA_IMPLICIT_ROOT=1` or `[tool.purba]`, and the expected value must be a literal in the last assert.
- T: under the port's `strict = "abort"`, the assert needs a message (`, "writes a"`). Without strict, the run passes and prints a tip.

6. Bugs and rough edges (not scored)

- **oxitest:** with a pattern that matches nothing, or an empty list, it exits 0 after running no tests.
- **oxitest:** a failure in a fixture teardown printed 21 "teardown failed" notices and still exited 0. This confirms T's NOTES.
- **oxitest:** a dict parametrize on unannotated parameters reports `valid keys: []`.
- **oxitest:** `-vv` prints only dots for tests that pass.
- **purba:** when the expected exception is not raised, it reports `[second-path] line 133 reaches line 39 of raised, which did not run` as a project error with exit 4. That is not a failure, and the message names the helper, not the missing exception. A wrong exception type, in contrast, gives a good message: `expected an instance of TypeError / reading ValueError(...)`.
- **purba:** a failure prints no file:line.
- **purba and oxitest:** neither captures loguru's default stderr output, so `| DEBUG | test_min:test_min:7 - a` showed up in the run. pytest captured it.
- The task says 9 tests, but the original file has 8 test functions (20 items).
- R/S NOTES mention `port/` and `../scratch/out/`, and neither exists in the folder.
