# Rater 2 (PORT_R = purba, PORT_T = oxitest)

The rater's final report, word for word, with its opening label dropped, the absolute path of its `_review/` folder shortened, and its section titles set as headings.

Everything below was run on copies under `_review/`.

The scores are close. PORT_R is the most direct on the questions a tester asks the runner: "what did I misspell?" and "which name is near?". It costs more boilerplate and has unusual assert rules. PORT_T is the clearest and most faithful on the page, and it has the best rerun. Its runner silently misreads a misspelled mark query, and its strict mode adds friction. ORIGINAL is the shortest and easiest to edit, but it says nothing about a misspelled test name.

## 1. Table (1-5; 5 = best, and 5 = simplest for both complexity rows)

| Criterion | ORIGINAL | PORT_R | PORT_T |
|---|---|---|---|
| 1 Readability | 4 | 4 | 3 |
| 2 Clarity | 3 | 4 | 5 |
| 3 Ease of use | 5 | 3 | 4 |
| 4 Complexity for the tester | 3 | 3 | 2 |
| 5 Complexity of the implementation | 2 | 4 | 2 |
| 6 Minimum working test | 5 | 4 | 3 |
| 7 Fidelity | (ref) | 4 | 5 |
| 8 Tagging | 4 | 4 | 2 |
| 9 Misspelled name | 1 | 5 | 4 |
| 10 Rerun failures | 4 | 3 | 5 |

## 2. Per version

ORIGINAL (pytest):

- Strongest: compact and easy to change. The suite is 111 non-blank lines. Adding case "g" was one tuple line, and my new test (`test_format_extra_field`) was 4 lines; both passed first time (23 passed).
- Weakest: the injected names are invisible. `writer` and the autouse reset come from conftest.py, and nothing in the test file points there.
- Parametrize IDs are opaque: `test_invalid_format[format2]`, `test_format[e-<lambda>-e]`.
- Checker noise is high: ty reports 9 diagnostics and pyright 11 errors, all in conftest, so a new type error is buried.
- A wrong-arity tuple is caught only by pytest at collection, though with a clear message ("number of names (3) must be equal to the number of values (2)").

PORT_R (purba):

- Strongest: the runner's own messages. It names the nearest tag, flags a name near `test`, and shows every failure as "expected / reading".
- Weakest: boilerplate and novel rules. It has 20 near-duplicate functions; for example the five `test_invalid_markups_*` tests (lines 151-178) are one body repeated.
- Every new test must call `reset_logger()` and `make_writer()` by hand, and forgetting the reset fails silently.
- Natural Python is refused. I ran the original dateutil test under purba: it got `[not-one-comparison] line 30`, and a for-loop with an assert got `[assert-before-expectation]`.
- `pytest.raises` became `add_error()` plus `isinstance` (line 43), which hides the "this should raise" intent.
- Each `@purba.tag` is flagged by both checkers as "Module `purba` has no member `tag`".

PORT_T (oxitest):

- Strongest: clarity. Cases have names (`unclosed_tag`), and every assert carries a message.
- The failure card shows the parameter values and the message, for example "format 'Message: {message}' should render 'a' as 'Message: A\n'".
- ty is clean, and pyright has only the 2 dateutil errors that the original also has.
- Typed cases help: a field typo (`expect=`) was caught by ty ("unknown-argument", "missing-argument").
- Weakest: the ceremony in the file and the runner's behaviour. Lines 15-23 hold a TYPE_CHECKING/else stand-in. Two lines carry dual-checker ignore comments (lines 22 and 137). There is one frozen dataclass per parametrize.
- Strict mode stops the whole suite for one missing assert message: "bare-assert line 187 … aborting (exit 3)".
- Mark filtering is silently loose (see criterion 8).

## 3. Criteria 8, 9, 10: score and the deciding evidence

Criterion 8 (tagging):

- ORIGINAL, 4: `-m smoke` gives "2 passed, 19 deselected". `-m smok` gives "21 deselected", exit 5, with no hint. Misspelling the decorator as `@pytest.mark.smok` warns "Unknown pytest.mark.smok - is this a typo?".
- PORT_R, 4: `'tag(smoke)'` runs 5 cases. `'tag(smok)'` stops with exit 3 and "`tag(smok)` picks no test; the nearest tag is `smoke`". I also had to add `import purba`. A misspelled tag in the decorator is silent (4 cases, exit 0).
- PORT_T, 2: `-E 'mark(smok)'` still ran 2 tests, "2 passed", exit 0. mark() matches a substring: `mark(s)` also matched, and `mark(xyz)` printed "no tests ran" with exit 0.
  - With `@oxi.mark.smok` on the decorator, the filtered run silently dropped the test (1 passed). Only the unfiltered run refused it ("unknown marker 'smok'", exit 3).
  - No test names were printed even with `-vv`.

Criterion 9 (misspelled test name):

- ORIGINAL, 1: completely silent. It printed "collected 20 items … 20 passed", exit 0; only the count drops from 21.
- PORT_R, 5: it printed "purba: name-near: tests/test_add_option_format.py:88: tset_progressive_format is not a test, and its name is near the prefix test". The count dropped to 23. The exit code stays 0, so CI would not catch it.
- PORT_T, 4: strict mode stopped the run with "tset_progressive_format module-level-def … line 76", exit 3. The message names a generic rule, not the typo. With `strict = "off"` it is silent (20 passed), and a public helper `def helper()` triggers the same violation.

Criterion 10 (rerun failures):

- ORIGINAL, 4: `pytest --lf` shows "rerun previous 2 failures". The ID of the parametrized case contains the expected value. After I fixed it, `--lf` ran only 1 item, so the fixed case never re-ran to show it passing.
- PORT_R, 3: I had to copy the last stderr line, a 105-character query, and quote it: `purba 'tests/…::test_format_a | tests/…::test_progressive_format'`. After the fix it printed "1 passed, 1 failed" and a shorter query. Nothing is stored between runs.
- PORT_T, 5: `--lf` printed "FF". After the fix it printed ".F" and "1 failed · 1 passed", then just "F". The case names are stable, so there is nothing to copy.

## 4. Overall ranking

Weights (they sum to 100): Readability 15, Clarity 15, Ease of use 15, Tester complexity 10, Implementation complexity 5, Minimum test 5, Fidelity 10 (the ORIGINAL is given 5 as the reference), Tagging 8, Misspelled name 8, Rerun 9.

| Rank | Version | Weighted score | With fidelity excluded (out of 90) |
|---|---|---|---|
| 1 | PORT_R | 3.74 | 3.71 |
| 2 | ORIGINAL | 3.71 | 3.57 |
| 3 | PORT_T | 3.68 | 3.53 |

The margins are small. Moving one point in Ease of use or in Tagging would reorder them.

## 5. Minimum working test (files under `_review/min/{O,R,T}/tests/test_min.py`, all pass)

| Version | Non-blank lines | Imports | Names the tester must know | Notes |
|---|---|---|---|---|
| ORIGINAL | 5 | 1 (`loguru.logger`) | `logger`, the `writer` fixture (from conftest), the `test_` prefix | |
| PORT_R | 6 | 1 | `logger`, the `test` prefix | Uses an inline `written` list, because `make_writer` lives in another test module. purba printed loguru's default stderr line: it does not capture output. |
| PORT_T | 6 | 1 | `logger`, the `test_` prefix, the rule that every assert needs a message | The first attempt stopped with "bare-assert line 8, exit 3"; the assert needed a message string. |

## 6. Bugs and rough edges (not scored)

PORT_T (oxitest):

- `mark(X)` is a substring match, and a filter that selects nothing exits 0 ("no tests ran").
- A filtered run skips the unknown-marker check.
- `-vv` and `--verbose=full` never list the names of passing tests.
- A dataclass field typo gives a raw traceback through `oxitest/_bridge/_loader.py:285`.
- The notes claim "ty and pyright are clean", but pyright reports 2 errors (their own CHECKERS.txt shows them).
- The notes' teardown claim is confirmed: raising after `yield` gave "21 passed, 21 warnings", exit 0.

PORT_R (purba):

- `purba.tag` exists at runtime (it is a lambda), but neither type checker sees it.
- A refused test is left out of the "failures as one query" line, so the rerun line can be incomplete.
- `[not-one-comparison]` hid `[assert-before-expectation]` on the same test, so the rules surface one at a time.
- In this copy, purba exits 4 on project errors but exits 0 on a name-near.

ORIGINAL (pytest):

- The `--lf` cache keys a case by an ID that contains its expected value, so changing an expected value changes which tests `--lf` runs.

My ty run first resolved the venv's python symlink to the base interpreter, which gave false import errors. Pointing `--python` at `.venv/bin/python` itself fixed it.
