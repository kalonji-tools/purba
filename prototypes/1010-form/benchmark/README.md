# The blind rating of the form that #1010 decides. Never merge.

Two files, each ported by agents that read only the form's `FORM.md`, `run.py`, `cases.py` and `examples/` (or oxitest's user docs), and scored by three blind raters beside pytest and oxitest.
The set-up is round 5's (`prototypes/967-python-form-4/benchmark/` on proto/980): shuffled letters (`ratings/KEY_*.txt`), the same prompt word for word for loguru (`ratings/RATER_PROMPT_flaskr.md` names what changed for flaskr), each rater's own weighting, and each weighted score recomputed with the original's fidelity counted as 5.
5 is best; for both complexity rows, 5 is simplest. Compare within a round: each round's raters score pytest and oxitest differently.

## Loguru `test_add_option_format.py`, the control

The same file as rounds 2 to 5. The form drops `before_each`, so each test names `_reset: None = use(reset_logger)`.

| | readability | clarity | ease of use | tester | implementation | smallest test | fidelity | weighted, rater 1 · 2 · 3 | mean |
|---|--:|--:|--:|--:|--:|--:|--:|---|--:|
| pytest | 4.67 | 3.0 | 4.33 | 4.0 | 3.0 | 5.0 | reference | 4.25 · 4.15 · 4.05 | 4.15 |
| oxitest | 3.67 | 4.67 | 3.33 | 3.0 | 2.0 | 4.0 | 5.0 | 4.00 · 3.75 · 3.90 | 3.88 |
| **the form** | 3.33 | 4.0 | 3.67 | 2.0 | 2.33 | 5.0 | 4.0 | 3.20 · 3.55 · 3.75 | **3.50** |

The gap to pytest is −0.65 and to oxitest −0.38. Prototype 4, with `before_each`, had −0.25 and +0.18 in round 5.
Each rater named `_reset` and `use(Writer)` in all 8 signatures. Rater 2 found that a test without `_reset` still passes, because each case runs in a fork of its own.

## Flaskr `test_blog.py`, a chain of fixtures

The tutorial's `app` → `client` → `auth` fixtures, with 8 tests and 12 items. The form was ported twice: once by a porter who chose fixtures, once by a porter told to share each arrange that acts through stages.

| | readability | clarity | ease of use | tester | implementation | smallest test | fidelity | weighted, rater 1 · 2 · 3 | mean |
|---|--:|--:|--:|--:|--:|--:|--:|---|--:|
| pytest | 5.0 | 4.0 | 4.67 | 4.0 | 2.67 | 5.0 | reference | 4.50 · 4.35 · 4.60 | 4.48 |
| **the form, fixtures** | 4.0 | 4.33 | 4.0 | 3.0 | 2.67 | 4.0 | 4.0 | 4.00 · 3.80 · 3.80 | **3.87** |
| oxitest | 3.67 | 4.33 | 2.33 | 2.67 | 1.67 | 3.0 | 5.0 | 3.65 · 3.25 · 3.70 | 3.53 |
| **the form, stages** | 3.0 | 3.67 | 3.67 | 2.0 | 2.33 | 3.0 | 3.0 | 3.25 · 2.95 · 3.05 | **3.08** |

Rater 2's sums for oxitest and the stages were arithmetic slips (3.15 and 3.0); the table holds the recomputed ones.
Each rater ranked the form with fixtures first among the ports and the form with stages last.
Rater 1 ran 8 mutants of the app: every version caught all 8, with the same tests failing.

On a broken login, the stage port shows 1 failure and 9 blocked cases that name the login stage; each other version shows the failure in each test.
The raters counted against the stages: 118 non-blank lines against 61, a second client for the anonymous page, a login before the author change, 4 more checks, a replay line on every case (the database is a stock, so no case forks from a stage), and a test that must know which stage to take.
Both form ports had to fold `app`, `client` and `auth` into one value, because a fixture takes no argument.

## Faults of this runner, not of the form

- A test whose second check posts `/1/delete` on the SQLite stock reports both checks failed, where it should be refused. The fork that reads a later check alone writes the shared file. Rater 2 reproduced it 4 times in 4.
- A passing `in` check prints the whole response body; a blocked case still prints a replay line; a refusal is labelled `› path` on a test with no `path` input; `run.py` fails its own `ty` check.

## Files

- `loguru-format/`, `flaskr-blog/`: the original, each port, and its `RUN.md`, `OUTPUT.txt`, `CHECKERS.txt` and `NOTES.md`. The repository's spelling gate rewrote one word of pyright's summary line in each `CHECKERS.txt`; no count changed.
- `ratings/`: the build scripts of the blind folders, the keys, the flaskr prompt, and each rater's scores, condensed.
