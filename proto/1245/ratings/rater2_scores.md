# Rater 2's report

Word for word, as the rater returned it, save one quoted misspelling of `expected` that the typos hook would rewrite: "ty caught my S typo exactly". In this rater's folder `PORT_R` = purba B, `PORT_S` = oxitest, `PORT_T` = purba A ([`KEY.txt`](KEY.txt)).

I ran all four versions and every check myself; my scratch work is under `<eval>/blind/rater2/_review/`. PORT_R and PORT_T have the same test file and the same notes; only the runner's wording differs. Both ports keep every input and expected value, so the scores split on three things: how each runner explains where it starts, how it handles a bad test-file setting, and how much ceremony a tester must write. R's runner explains itself best on both setting checks (8 and 9). S has the clearest tests and failure output, but it has the most to learn, and it passes silently when nothing runs. The original pytest file is the most compact, but it hides its setup in conftest.

**1. Scores (5 = best; for both complexity rows, 5 = simplest)**

| Criterion | ORIGINAL | R | S | T |
|---|---|---|---|---|
| 1 Readability | 4 | 4 | 3 | 4 |
| 2 Clarity | 3 | 4 | 5 | 4 |
| 3 Ease of use | 4 | 3 | 4 | 3 |
| 4 Complexity for the tester | 3 | 4 | 2 | 4 |
| 5 Complexity of the implementation | 2 | 3 | 1 | 3 |
| 6 Minimum working test | 5 | 4 | 4 | 4 |
| 7 Fidelity | – | 5 | 5 | 5 |
| 8 Where a run starts | 2 | 5 | 2 | 4 |
| 9 Choosing the test files | 2 | 5 | 1 | 3 |
| 10 Seeded failures | 4 | 4 | 5 | 4 |

**2. Per version**

- **ORIGINAL (pytest)**
  - Strongest: it is compact. The test file is 85 lines, and a new case is one tuple row.
  - Weakest: `writer` and the autouse reset come from conftest and cannot be seen in the file.
  - Case ids are vague, for example `test_format[e-<lambda>-e]`.
  - The checkers start with 9 errors in conftest (ty and pyright both).

- **R (purba)**
  - Strongest: the runner's errors. Each one has a tag, a line and a reason, for example `[second-path] line 18 went one way only…`.
  - The root line and the test-files errors explain themselves fully.
  - Weakest: each case is a separate copied function, so the file is 222 lines and adding a case costs 6 lines.
  - Weakest: the conftest code moved into the test file, so pyright reports 26 errors and ty reports 12. A new error is lost in that noise.

- **S (oxitest)**
  - Strongest: clarity. Cases have names, and each assert carries a message.
  - A failure shows the case name, the line, a caret under the difference and the intent message (`tests/test_add_option_format.py:70`).
  - ty and pyright are both clean.
  - Weakest: the ceremony. A `TYPE_CHECKING`/`else` stand-in (lines 15–23), two-checker ignore comments (line 170), a dataclass for each parametrize, and the special `__fixtures__.py` file.
  - Weakest: `python_files = []` prints "no tests ran" and exits **0**.

- **T (purba, the other build)**
  - It has the same strengths as R.
  - Its messages are shorter, and on bad settings they lose the fix. `test-files: an empty list` does not say what to list. `test*.py is not a pattern purba takes; nearest: test_*.py` does not list `*_test.py`.

**3. Criteria 8 and 9, one line per version**

Criterion 8 (where a run starts):
- **ORIGINAL 2:** from `tests/` it prints `rootdir: …/ORIGINAL_pytest`, but the ids start `test_add_option_format.py::…`. In the `.git` copy, the root becomes `tests/` with no reason given.
- **R 5:** in each of the 6 runs, the ids match the root line, and the line gives the reason. With neither file it says "the id of each test starts here, and moves with the working folder".
- **S 2:** it prints only `rootdir:`. A passing run never shows ids (dots even at `--verbose=full`). `oxitest query tests` shows absolute ids, and the `.git` copy run from `tests/` gives `rootdir: …/tests` with no reason.
- **T 4:** it gives the same facts as R, but as a terse list: `from .git; PURBA_IMPLICIT_ROOT=1; no [tool.purba] found; ids start here`. It does not say that the ids move with the working folder.

Criterion 9 (choosing the test files):
- **ORIGINAL 2:** `["test*.py"]` and `[]` fail silently. The output is either the wrong files or "no tests ran" (exit 5), with no hint. A string is accepted under `ini_options`; under `[tool.pytest]` the error is clear.
- **R 5:** each bad value is refused with exit 4. Each error names the item, the closed list of patterns, and an example fix, for example `write a list, such as ["*_test.py"]`.
- **S 1:** `[]` and a non-matching pattern both print "no tests ran" and exit **0**. `python_files` is not in `--help`; I found it only through an unknown-field error.
- **T 3:** each bad value is refused with exit 4, but the messages do not list the allowed patterns.

**4. Overall ranking**

Weights: clarity, ease of use and seeded failures count ×2; every other criterion ×1. I left fidelity out because the three ports tie at 5 and the original has no score. The total weight is 12.

1. **R**, 47/60 (3.92)
2. **T**, 44/60 (3.67)
3. **S**, 41/60 (3.42)
4. **ORIGINAL**, 40/60 (3.33)

**5. Minimum working tests**

Each test is in `_review/min_<version>/tests/test_min.py`, and each passed.

| | Non-blank lines | Imports | Names the tester must know |
|---|---|---|---|
| pytest | 6 | 1 (`from loguru import logger`) | the `test_` prefix |
| purba (R, T) | 6 | 1 | the `test` prefix; `[tool.purba]` or `PURBA_IMPLICIT_ROOT=1` (otherwise exit 3); the expected value must be a literal |
| oxitest (S) | 6 | 1 | the `test_` prefix; an assert message, because the project sets `strict="abort"` (otherwise `bare-assert`, exit 3) |

**Evidence for the other criteria**

- **Fidelity (7):**
  - R and T: the same data. `pytest.raises` became `raised()` plus `isinstance` and `re.search`, which matches pytest's `match=`. The reset happens only before each test, inside `make_writer()`, but each test runs in its own process, so nothing is lost. The unused `w.clear` is kept.
  - S: the same data. `format` is renamed `fmt`, assert messages are added (they document, they check nothing new), and `Writer` has no `clear`. The notes say a teardown exception is only a notice; I did not test that.
- **Ease of use (3):**
  - In all four, the "g" case and my test (a removed handler receives no later message) passed the first time.
  - S stopped my test at first for having no assert message: `bare-assert line 80`, exit 3.
  - ty caught my S typo exactly. pytest reported a short tuple row clearly at collection.
  - purba's refusals of a fixture parameter, an `if`, an assert in the middle and a computed expected value were all clear and gave line numbers.
- **Seeded failures (10):** all versions printed "2 failed, 18 passed".
  - purba shows the assert text, the expected value and the reading, but no file:line and no difference marker. I had to compare `.....` and `....` by eye.
  - pytest shows a diff with `^` and the line of the assert, not the line of the failing row.

**6. Bugs and rough edges (not scored)**

- oxitest exits 0 when no test ran. Its failure headers are not consistent: `message_field - test_format` versus `FAILED <path>::test_progressive_format`.
- purba does not capture output: loguru's default stderr line appeared among the results of the minimum test.
- purba printed project errors out of source order: the static refusals came first.
- R and T's NOTES.md says that `[foreign-framework]` and `[computed-expected]` are missing from the rules, but `rules/TESTS.md` in the folder documents both. The notes are out of date.
- The brief says the file has 9 tests; it has 8 test functions (20 items).
- R runs from venv-b; T, S and ORIGINAL run from venv-a.
