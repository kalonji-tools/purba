# The benchmark of prototype 3 (#967). Never merge.

Prototype 3 ports the same loguru test as prototypes 1 and 2, and three new blind raters scored it beside pytest and oxitest.
Within this round it came second: 3.74, against 3.84 for pytest and 3.66 for oxitest.
It is the first prototype above oxitest, by 0.08, and the first within 0.10 of pytest.
Two raters of three ranked it first by their own judgement; by the numbers, it came second, second and third.

## The test

The test and the two other versions are prototype 1's, unchanged: [`benchmark/loguru-format/`](https://github.com/kalonji-tools/purba/tree/6591a105b6145c3c4afd40114d8e2085405401c1/prototypes/967-python-form/benchmark/loguru-format) on `proto/967-python-form-of-an-experiment`.

| folder | what it is | items | passes | ty | pyright |
|---|---|---|---|---|---|
| `pytest/` (prototype 1's branch) | loguru's `tests/test_add_option_format.py` at `28349be` (MIT), with its two conftest fixtures | 20 | 20 | 9 diagnostics in loguru's `conftest.py` | 9 errors in `conftest.py`, 2 warnings |
| `oxitest/` (prototype 1's branch) | the port to oxitest 4.0.0 | 20 | 20 | clean | clean |
| [`loguru-format/prototype-3/`](loguru-format/prototype-3/) | a port to this form, by an agent that read only `FORM.md`, `run.py`, `cases.py` and `examples/`, and changed no runner file | 20 cases | 20 | clean | clean |

The port is 106 + 50 lines (77 + 34 non-blank), against the original's 87 + 30 (67 + 22), prototype 2's 110 + 33 and prototype 1's 174 + 50.
Its [`NOTES.md`](loguru-format/prototype-3/NOTES.md) lists each change the rules forced, its probes, and the holes it found in the rules.
The porter found five faults of the runner. All five were fixed in `run.py` before the rating, and the port and its run did not change:

- a folder given with `..` found no test;
- a second test file of the same name was dropped without a word;
- the act ran as `__act__`, so loguru's `{function}` read that name;
- a computed expected value passed through an alias of the test, or through `**{…}`;
- `len(message)` hid that a check reads only an input.

## The numbers

Same set-up as rounds 2 and 3: three raters, each in a folder of their own, the two ports under shuffled letters (`ratings/KEY.txt`), and the same prompt word for word (`ratings/RATER_PROMPT.md`).
Each ran every version, added a case and a test to each, wrote the smallest passing test, probed failures, and chose their own weighting.
5 is best; for both complexity rows, 5 is simplest.
Each weighted score is recomputed from the rater's own scores and weights, with pytest's fidelity counted as 5, as in rounds 2 and 3. Rater 3's own sums differ (3.85 · 3.75 · 3.70); the recomputed ones are below.

| | readability | clarity | ease of use | complexity for the tester | complexity of the implementation | smallest test | fidelity | weighted, rater 1 · 2 · 3 | mean |
|---|--:|--:|--:|--:|--:|--:|--:|---|--:|
| pytest | 4.0 | 3.0 | 3.7 | 4.0 | 2.3 | 5.0 | reference | 3.80 · 3.95 · 3.78 | 3.84 |
| oxitest | 3.0 | 4.7 | 3.0 | 2.7 | 2.0 | 4.0 | 5.0 | 3.65 · 3.45 · 3.89 | 3.66 |
| **prototype 3** | 4.0 | 4.0 | 4.0 | 2.3 | 3.0 | 4.7 | 4.0 | 3.65 · 3.85 · 3.72 | **3.74** |

**Compare within a round.** The files of pytest and oxitest are the same in all three rounds, and so is the prompt, but each round's raters scored them differently: pytest 4.35, 4.07, 3.84; oxitest 4.15, 3.72, 3.66.

| | gap to pytest | gap to oxitest | rank by the numbers, rater 1 · 2 · 3 | rank the rater chose |
|---|--:|--:|---|---|
| prototype 1, round 2 | −2.38 | −2.18 | 3 · 3 · 3 | 3 · 3 · 3 |
| prototype 2, round 3 | −0.47 | −0.12 | 3 · 3 · 2 | 3 · 3 · 2 |
| prototype 3, round 4 | −0.10 | +0.08 | 2 (tied with oxitest) · 2 · 3 | 3 · 1 · 1 |

Rater 1 broke the tie with oxitest on fidelity. Rater 2 ranked it first by leaving pytest's fidelity out. Rater 3 ranked it first against their own numbers, because "M's errors were actionable every time I tested them, and they refused only the faulty case".

**By criterion, against prototype 2:** readability 3.3 → 4.0, the smallest test 3.0 → 4.7, clarity, ease of use and fidelity unchanged at 4.0, complexity for the tester unchanged at 2.3, complexity of the implementation 4.0 → 3.0.

**The smallest passing test** (`logger.add(writer, format="{message}")`, then `logger.debug("a")`, reads `"a\n"`): 6 non-blank lines in all three versions, one import, and no name from the form. Rater 2's is byte-for-byte the pytest test. With the port's own helpers (`use`, `Writer`, `fresh_logger`), it grows to 10.

**Adding a case** took one line in all three versions. For raters 1 and 2, a new test passed on the first try in prototype 3; in pytest it did for rater 1, and rater 2's failed once on their own wrong guess of the exception. oxitest aborted all 22 items on a bare assert for raters 1 and 2. In prototype 3, rater 1's other natural tests were refused twice: an `in` check, and a loop.

## What the three raters found in prototype 3

All three found these:

- **The errors of the runner:** `gives 'mesage', which is no input of test_format; did you mean 'message'?`, together with `gives expected=b'g!' (bytes), and test_format declares str`. Only that case was refused; the other 21 ran. A failure prints expected, reading, the first differing character, and the inputs.
- **The case lines read as a table, and the checkers read them:** a misspelled input and an expected value of the wrong type are caught by ty, by pyright and by the runner.
- **The rules refuse ordinary code:** `if`, loops, `in`. `test_progressive_format`'s format function holds an `if`, so it moved to `loguru_lab.py`, out of sight of its test; the hint "make each side a case of its own" does not fit a callback. Its loop became five lines. Complexity for the tester 2.3, the lowest of the three, as for prototype 2.
- **No fixture reaches every test of a file:** every signature carries `writer: Writer = use(Writer), logger: Logger = use(fresh_logger)`. `test_format`'s signature takes 7 lines, and 12 lines are over 100 characters. A fork resets what a case changes, but not loguru's default handler, which the import made; so the port needs `fresh_logger` on every test.
- **The runner never runs the test as written:** it parses the source, compiles the act again, and evaluates each check in a fork. 777 + 102 non-blank lines, against prototype 2's 512 + 83. Complexity of the implementation 3.0.
- **More than the original:** the runner proves that each `except` ran, and both checks of the exception test run when the first one fails.

One rater each found these:

- A loop written as `list(map(lambda …))` passes the rules (rater 3, confirming the porter's note).
- `RUN.md` gives no type-checker command; helpers must sit inside `tests/` (raters 2 and 3).
- `NOTES.md` cites probes outside the rater's folder, which cannot be verified (rater 2).

## Files

- `loguru-format/prototype-3/`: the port (`tests/`), the porter's `NOTES.md`, the real `OUTPUT.txt` and `CHECKERS.txt`.
- `ratings/loguru_rater{1,2,3}_scores.txt`: each rater's scores, weights and findings, unblinded.
- `ratings/KEY.txt`, `ratings/RATER_PROMPT.md`, `ratings/build_blind_loguru.sh`: how the blind folders were built and what each rater was asked.

The runs used Python 3.12.14, pytest 9.2.0, oxitest 4.0.0, loguru 0.8.0, ty 0.0.87 and pyright 1.1.411.
