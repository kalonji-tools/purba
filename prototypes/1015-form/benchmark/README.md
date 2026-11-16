# The blind rating of the 1015 form. Never merge.

Two files, each ported by an agent that read only the form's `FORM.md`, `cases.py` and `examples/`, and ran `run.py`. Three blind raters scored each port beside pytest and oxitest.
The set-up is the one #1010 used: shuffled letters (`ratings/KEY_*.txt`), the prompt of rounds 2 to 5 word for word for loguru (`ratings/RATER_PROMPT_loguru.md`), the flaskr prompt of #1010 with two ports in place of three (`ratings/RATER_PROMPT_flaskr.md`), each rater's own weighting, and each weighted score recomputed with the original's fidelity counted as 5.
5 is best; for both complexity rows, 5 is simplest. Compare within a round: each round's raters score pytest and oxitest differently.

The form changed in eight ways since #1010, as [the API strategy](https://github.com/kalonji-tools/purba/issues/1015#issuecomment-6911718208) agreed:
three more shapes of a check (`assert not x`, `isinstance`, `startswith`/`endswith`), each with its own failure line; a `try` with one statement and one `except` in place of `handling`; `@skip(reason)` and `@skip_if(condition, reason)`; a `raises` or `warns` that checks the type alone is refused; a private name and a name that the act replaced are reported, never refused; the seed of `random` on the summary and on each failure; the time of a slow case.

## Loguru `test_add_option_format.py`, the control

| | readability | clarity | ease of use | tester | implementation | smallest test | fidelity | weighted, rater 1 · 2 · 3 | mean |
|---|--:|--:|--:|--:|--:|--:|--:|---|--:|
| pytest | 4.33 | 3.0 | 4.33 | 4.0 | 3.0 | 5.0 | reference | 3.90 · 3.90 · 4.25 | 4.02 |
| **the form** | 4.0 | 4.33 | 4.0 | 2.0 | 2.0 | 5.0 | 4.0 | 3.75 · 3.55 · 3.55 | **3.62** |
| oxitest | 3.0 | 4.0 | 3.0 | 3.0 | 2.67 | 4.33 | 4.67 | 3.35 · 3.55 · 3.45 | 3.45 |

The gap to pytest is −0.40 and to oxitest +0.17. The form of #1010 had −0.65 and −0.38 with a reset in each signature, and −0.52 and +0.15 with `@before`; prototype 4 had −0.25 and +0.18.
By the recomputed numbers the form came second for each rater; rater 2 tied it with oxitest and broke the tie for it.
Rater 1 left fidelity out of its own sums, tied the form with pytest at 3.71, and broke that tie for the form on clarity; the table holds the sums recomputed with fidelity.

What the raters counted for the form: the failure screen (expected, reading, why they differ, inputs), the typed case line that ty, pyright and the runner each check ("did you mean 'message'?"), a refusal of one bad case while the others run.
Against it: the rules, 187 lines of `FORM.md` (3 of 3); `@before(reset_logger)` on all 8 tests (3 of 3); case names `format_a` to `format_f` that say nothing (2 of 3); a bare `assert writer.written` refused (raters 1 and 3); `with raises(KeyError):` refused for no `match=` (rater 1); an expected value computed from an input (rater 3).
Raters of #1010 said that `handling` reads backwards. No rater met it here: the port uses `try` and `except`, and no rater named it.

## Flaskr `test_blog.py`, fixtures

| | readability | clarity | ease of use | tester | implementation | smallest test | fidelity | weighted, rater 1 · 2 · 3 | mean |
|---|--:|--:|--:|--:|--:|--:|--:|---|--:|
| pytest | 5.0 | 3.67 | 4.33 | 4.0 | 3.67 | 5.0 | reference | 4.15 · 4.60 · 4.35 | 4.37 |
| oxitest | 4.0 | 4.67 | 2.33 | 3.0 | 2.67 | 3.33 | 5.0 | 3.65 · 3.55 · 3.80 | 3.67 |
| **the form** | 3.0 | 4.33 | 4.0 | 2.0 | 1.0 | 3.0 | 4.0 | 3.25 · 3.50 · 3.20 | **3.32** |

The gap to pytest is −1.05 and to oxitest −0.35. The form of #1010 had −0.61 and +0.34 with fixtures. Each rater ranked the form last.
Against #1010's form with fixtures, the form lost 1.0 on readability, 1.0 on complexity for the tester, 1.67 on complexity of the implementation and 1.0 on the smallest test; clarity, ease of use and fidelity did not move.

What the raters counted for the form: both checkers clean, and each planted mistake found by both ("did you mean 'path'?", "gives path=3 (int), and test_exists_required declares str"); the clearest failure line of the three (3 of 3).
Against it: the port moves code (test_index sends every request before its first check; test_create checks the GET status inside `with app.app_context()`, after the POST) (3 of 3); an assert before the last request is refused, so a test written as pytest writes it must be restructured (3 of 3); `FORM.md` at 187 lines and `run.py` at about 3,000 (3 of 3); a passing `in` check prints about 400 characters of HTML (2 of 3); the smallest test that does not leak its database file needs a stock, 20 or 21 lines (2 of 3).

The flaskr port holds no `raises`, no `try`, no skip, no private name and no patch; it meets the eight changes only through a longer `FORM.md` (165 → 187 lines) and a longer runner. One port by one porter stood for the form, so the next round asks how much of the drop is the port.

## Flaskr again: a second porter

A second porter, with the same brief and without sight of the first port, ported the file again (`flaskr-blog/form-2/`, port B; the first is port A). Three new raters scored four versions with #1010's flaskr prompt word for word (`ratings/RATER_PROMPT_flaskr2.md`, `ratings/KEY_flaskr2.txt`), as #1010 rated two ports of one form.

| | readability | clarity | ease of use | tester | implementation | smallest test | fidelity | weighted, rater 1 · 2 · 3 | mean |
|---|--:|--:|--:|--:|--:|--:|--:|---|--:|
| pytest | 5.0 | 4.0 | 5.0 | 4.0 | 3.33 | 5.0 | reference | 4.60 · 4.55 · 4.45 | 4.53 |
| **port B** | 4.0 | 4.33 | 4.0 | 2.0 | 2.0 | 3.33 | 4.0 | 3.60 · 3.50 · 3.70 | **3.60** |
| oxitest | 3.67 | 4.67 | 2.33 | 2.67 | 2.33 | 3.33 | 5.0 | 3.65 · 3.80 · 3.10 | 3.52 |
| **port A** | 3.0 | 4.0 | 4.0 | 2.0 | 2.0 | 3.0 | 4.0 | 3.35 · 3.35 · 3.35 | **3.35** |

Rater 1 rescaled the original without fidelity (4.53); the table holds 4.60, with fidelity counted as 5.

- **Port A is stable.** It scored 3.35 with each of the three new raters, and 3.32 in the first round.
- **The porter moves 0.25.** Each rater scored port B above port A, by 0.25, 0.15 and 0.35. Port B gained on readability (4.0 against 3.0) and the smallest test: its fixtures sit in a module of their own, and its checks come after the `with` block, where port A's sit inside `with app.app_context()` (3 of 3). Port A's notes call that layout forced; rater 3 found that `FORM.md` allows the checks after the block.
- **Port B came second, above oxitest by 0.08, and 0.93 below pytest.** The form of #1010 with fixtures was 0.34 above oxitest and 0.61 below pytest.
- **What remains against #1010's form is the tester's load.** Port B equals #1010's port on readability, clarity, ease of use and fidelity, and is 1.0 lower on complexity for the tester, 0.67 lower on the implementation and 0.67 lower on the smallest test. Each of the six flaskr raters gave each port of this form a 2 on complexity for the tester; one counted 58 rules. The eight changes add rules that flaskr's tests never use.
- **What the raters counted against both ports:** each test puts every request before its first assert, so `test_index` checks the anonymous page after the login, and `test_create` checks the GET's status after the POST (3 of 3); `FORM.md` at 187 lines (3 of 3); `use(App)`, a stock class, beside `use(client)`, a factory (2 of 3).

## Faults of this runner, not of the form

- A test whose second check posts `/1/delete` on the SQLite stock reports both checks failed with readings that never happened, where it should be refused (flaskr rater 2). The fork that reads a later check alone writes the shared file. The benchmark of #1010 lists the same fault.
- The runner skips the type check of an input whose annotation it cannot resolve, such as a name imported only for the checkers, and says nothing (loguru raters 1 and 2).
- When an assert is followed by another statement, the refusal says "a function handed to the code may swallow an assert", which names the wrong cause (loguru rater 2).
- A plain fixture that makes a temporary database file is not refused as a stock, and leaks the file (flaskr rater 3).
- Before the rating, two faults were fixed: a test that patched `json.dumps` crashed the whole run, because the runner wrote its results through it; and the watch of replaced names reported flask filling a private cache. Neither touched a port.

## Files

- `loguru-format/form/`, `flaskr-blog/form/` (port A), `flaskr-blog/form-2/` (port B): each port, with its `RUN.md`, `OUTPUT.txt`, `CHECKERS.txt` and `NOTES.md`; each flaskr port also has `OUTPUT_mutant-login.txt` (9 failed, 3 passed, as in each version).
- `loguru-format/pytest/`, `loguru-format/oxitest/`, `flaskr-blog/pytest/`, `flaskr-blog/oxitest/`, `flaskr-blog/flaskr-src/`: the originals and the oxitest ports, unchanged from #1010.
- `ratings/`: the build scripts of the blind folders, the keys, the prompts, and each rater's scores, condensed.

The runs used CPython 3.12.15, pytest 9.2.0, oxitest 4.0.0, loguru 0.8.0 and flask 3.1.2.
