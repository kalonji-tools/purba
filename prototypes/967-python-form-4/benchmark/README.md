# The benchmark of prototype 4 (#967). Never merge.

Prototype 4 ports the same loguru test as prototypes 1, 2 and 3, and three new blind raters scored it beside pytest and oxitest.
Within this round it came second: 3.82, against 4.07 for pytest and 3.63 for oxitest.
Against oxitest it gained (+0.18, where prototype 3 had +0.08); against pytest it lost ground (−0.25, where prototype 3 had −0.10), because this round's raters scored pytest higher.
By the numbers it came second for all three raters; by their own judgement, rater 3 ranked it first.

## The test

The test and the two other versions are prototype 1's, unchanged: [`benchmark/loguru-format/`](https://github.com/kalonji-tools/purba/tree/6591a105b6145c3c4afd40114d8e2085405401c1/prototypes/967-python-form/benchmark/loguru-format) on `proto/967-python-form-of-an-experiment`.

| folder | what it is | items | passes | ty | pyright |
|---|---|---|---|---|---|
| `pytest/` (prototype 1's branch) | loguru's `tests/test_add_option_format.py` at `28349be` (MIT), with its two conftest fixtures | 20 | 20 | 9 diagnostics in loguru's `conftest.py` | 9 errors in `conftest.py`, 2 warnings |
| `oxitest/` (prototype 1's branch) | the port to oxitest 4.0.0 | 20 | 20 | clean | clean |
| [`loguru-format/prototype-4/`](loguru-format/prototype-4/) | a port to this form, by an agent that read only `FORM.md`, `run.py`, `cases.py` and `examples/`, and changed no runner file | 20 cases | 20 | clean | clean |

The port is 103 + 24 lines (71 + 16 non-blank), against the original's 87 + 30 (67 + 22), prototype 3's 106 + 50, prototype 2's 110 + 33 and prototype 1's 174 + 50.
The runner refused nothing during the port. Its [`NOTES.md`](loguru-format/prototype-4/NOTES.md) lists each change the form or the type checkers forced, and its probes.
The porter found two faults of the runner, and one fidelity gap of `handling`. All three were fixed before the rating, and the port and its run did not change:

- an `if` in a class that the test builds itself was not refused;
- an `if` in a function that the test calls through another name was not refused;
- `handling`, written as a generator, added two frames of the form's own code to the traceback that loguru logs; as a class it logs the original's traceback.

Before the rating, the runner's check function was renamed from a name that held the project's name.

## The numbers

Same set-up as rounds 2, 3 and 4: three raters, each in a folder of their own, the two ports under shuffled letters (`ratings/KEY.txt`), and the same prompt word for word (`ratings/RATER_PROMPT.md`).
Each ran every version, added a case and a test to each, wrote the smallest passing test, probed failures, and chose their own weighting.
5 is best; for both complexity rows, 5 is simplest.
Each weighted score is recomputed from the rater's own scores and weights, with pytest's fidelity counted as 5, as in rounds 2 to 4. Rater 1 and rater 3 left pytest's fidelity out of their own sums; rater 2's sums match.

| | readability | clarity | ease of use | complexity for the tester | complexity of the implementation | smallest test | fidelity | weighted, rater 1 · 2 · 3 | mean |
|---|--:|--:|--:|--:|--:|--:|--:|---|--:|
| pytest | 4.0 | 3.0 | 4.0 | 4.0 | 3.7 | 5.0 | reference | 4.05 · 4.10 · 4.05 | 4.07 |
| oxitest | 3.0 | 4.7 | 2.7 | 2.7 | 2.7 | 4.0 | 4.7 | 3.65 · 3.70 · 3.55 | 3.63 |
| **prototype 4** | 4.0 | 4.3 | 4.0 | 2.3 | 2.3 | 4.7 | 4.0 | 3.70 · 3.80 · 3.95 | **3.82** |

**Compare within a round.** The files of pytest and oxitest are the same in all four rounds, and so is the prompt, but each round's raters scored them differently: pytest 4.35, 4.07, 3.84, 4.07; oxitest 4.15, 3.72, 3.66, 3.63.

| | gap to pytest | gap to oxitest | rank by the numbers, rater 1 · 2 · 3 | rank the rater chose |
|---|--:|--:|---|---|
| prototype 1, round 2 | −2.38 | −2.18 | 3 · 3 · 3 | 3 · 3 · 3 |
| prototype 2, round 3 | −0.47 | −0.12 | 3 · 3 · 2 | 3 · 3 · 2 |
| prototype 3, round 4 | −0.10 | +0.08 | 2 (tied with oxitest) · 2 · 3 | 3 · 1 · 1 |
| prototype 4, round 5 | −0.25 | +0.18 | 2 · 2 · 2 | 2 · 2 · 1 |

**By criterion, against prototype 3:** clarity 4.0 → 4.3, complexity of the implementation 3.0 → 2.3, readability, ease of use and fidelity unchanged at 4.0, the smallest test unchanged at 4.7, complexity for the tester unchanged at 2.3, for the third round in a row.
The criterion that moved against pytest is complexity of the implementation: this round's raters gave pytest 3.7, where round 4's gave it 2.3.

**The smallest passing test** (`logger.add(writer, format="{message}")`, then `logger.debug("a")`, reads `"a\n"`): 6 non-blank lines in all three versions, one import, and no name from the form. All three raters wrote the same text in pytest and in prototype 4. oxitest's needs a message on its assert under the port's `strict = "abort"`, and a `pyproject.toml`.

**Adding a case** took one line, on the first try, in all three versions, for all three raters. A new test passed on the first try in prototype 4 for raters 1 and 3 (rater 2 did not say); rater 3's new test took 5 lines in pytest, 9 in prototype 4 and 17 in oxitest.

## What the three raters found in prototype 4

All three found these:

- **The errors of the runner are the best of the three:** a misspelled input refused with `did you mean 'message'?`, an expected value of the wrong type refused with `gives expected=1 (int), and test_format declares str`, and only the faulty case stopped. A failure prints expected, reading, the first differing character, and the inputs.
- **The case lines read as a table, and the checkers read them** against the test's signature.
- **The rules sheet is long:** "28 dense lines of rules" (rater 1), "28 dense rules" (rater 2), "about 15 rules" (rater 3). Between them, they named the refusals they met with their own code: a `try`, an `and` inside an assert, a computed expected value (`level + ": x\n"`), an `if`.
- **The runner is large:** `run.py` 975 lines and `cases.py` 211, which rewrite asserts, read the act's paths from its syntax, and fork for each case and each check.
- **`handling` replaces `try`/`except`:** all three checked that the logged traceback is now the original's.
- **Less than the original:** the reset runs before each case, not after.
- **Lines over 100 characters:** 6 of them, and one signature over 120.
- **The folder's own files fall short:** `RUN.md` gives no type-checker command, and `CHECKERS.txt` cites `E/proto4` paths that are not in the folder.

One rater each found these:

- Under `handling`, loguru records the log call as coming from the form's own code: `cases:__exit__:157`, where the `try` gives the test's function and line (rater 3). The ported checks do not read it. A probe after the rating: `logger.opt(depth=1).exception` moves the record into the test, at the line of the `with`, not the line of the original `except`.
- The runner forks, so it runs only on POSIX (rater 2).

## Files

- `loguru-format/prototype-4/`: the port (`tests/`), the porter's `NOTES.md` with each fix after the port marked, the real `OUTPUT.txt` and `CHECKERS.txt`, run again after the fixes and unchanged.
- `ratings/loguru_rater{1,2,3}_scores.txt`: each rater's scores, weights and findings, unblinded.
- `ratings/KEY.txt`, `ratings/RATER_PROMPT.md`, `ratings/build_blind_loguru.sh`: how the blind folders were built and what each rater was asked.

The runs used Python 3.12.14, pytest 9.2.0, oxitest 4.0.0, loguru 0.8.0, ty 0.0.87 and pyright 1.1.411.
