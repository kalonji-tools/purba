# The benchmark of prototype 2 (#967). Never merge.

Prototype 2 ports the same loguru test as prototype 1, and three new blind raters scored it beside pytest and oxitest.
It did not beat the numbers of round 2: 3.60 against pytest's 4.35 and oxitest's 4.15.
But the same raters scored pytest and oxitest lower this time, and within this round prototype 2 came 0.12 below oxitest, where prototype 1 came 2.18 below it.
One rater of three put it above oxitest.

## The test

The test and the two other versions are prototype 1's, unchanged: [`benchmark/loguru-format/`](https://github.com/kalonji-tools/purba/tree/6591a105b6145c3c4afd40114d8e2085405401c1/prototypes/967-python-form/benchmark/loguru-format) on `proto/967-python-form-of-an-experiment`.

| folder | what it is | items | passes | ty | pyright |
|---|---|---|---|---|---|
| `pytest/` (prototype 1's branch) | loguru's `tests/test_add_option_format.py` at `28349be` (MIT), with its two conftest fixtures | 20 | 20 | 9 diagnostics in loguru's `conftest.py` | 9 errors in `conftest.py`, 2 warnings |
| `oxitest/` (prototype 1's branch) | the port to oxitest 4.0.0 | 20 | 20 | clean | clean |
| [`loguru-format/prototype-2/`](loguru-format/prototype-2/) | a port to this form, by an agent that read only `FORM.md`, `run.py`, `experiment.py` and `examples/`, and changed no runner file | 20 cases | 20 | clean | clean |

The port is 110 + 33 lines (77 + 22 non-blank), against the original's 87 + 30 (67 + 22) and prototype 1's 174 + 50.
Its [`NOTES.md`](loguru-format/prototype-2/NOTES.md) lists each change the rules forced, and its probes.
The porter found two faults of the runner: a name bound to a computed value passed as a literal, and a file that failed to import counted as one refused case.
Both were fixed in `run.py` before the rating; the port and its run did not change.

## The numbers

Same set-up as round 2: three raters, each in a folder of their own, the two ports under shuffled letters (`ratings/KEY.txt`), and the same prompt word for word (`ratings/RATER_PROMPT.md`).
Each ran every version, added a case and a test to each, wrote the smallest passing test, seeded failures, and chose their own weighting.
5 is best; for both complexity rows, 5 is simplest.

| | readability | clarity | ease of use | complexity for the tester | complexity of the implementation | smallest test | fidelity | weighted, rater 1 · 2 · 3 | mean |
|---|--:|--:|--:|--:|--:|--:|--:|---|--:|
| pytest | 4.7 | 3.0 | 4.0 | 4.0 | 2.0 | 5.0 | reference | 3.95 · 4.15 · 4.10 | 4.07 |
| oxitest | 3.0 | 4.7 | 3.0 | 2.7 | 2.0 | 4.3 | 5.0 | 3.60 · 3.95 · 3.60 | 3.72 |
| **prototype 2** | 3.3 | 4.0 | 4.0 | 2.3 | 4.0 | 3.0 | 4.0 | 3.50 · 3.55 · 3.75 | **3.60** |

Round 2, for comparison (prototype 1's [`benchmark/README.md`](https://github.com/kalonji-tools/purba/blob/6591a105b6145c3c4afd40114d8e2085405401c1/prototypes/967-python-form/benchmark/README.md)):

| | readability | clarity | ease of use | complexity for the tester | complexity of the implementation | smallest test | fidelity | mean |
|---|--:|--:|--:|--:|--:|--:|--:|--:|
| pytest | 5.0 | 4.0 | 4.0 | 4.0 | 3.7 | 5.0 | reference | 4.35 |
| oxitest | 4.0 | 5.0 | 4.0 | 3.0 | 2.7 | 4.0 | 5.0 | 4.15 |
| prototype 1 | 2.0 | 2.7 | 2.0 | 1.0 | 1.7 | 1.0 | 2.7 | 1.97 |

**The anchors moved.** The files of pytest and oxitest are the same in both rounds, and so is the prompt, but this round's raters scored them 0.28 and 0.43 lower. Compare prototype 2 with the others inside a round:

| | gap to pytest | gap to oxitest | rank among the three, rater 1 · 2 · 3 |
|---|--:|--:|---|
| prototype 1, round 2 | −2.38 | −2.18 | 3 · 3 · 3 |
| prototype 2, round 3 | −0.47 | −0.12 | 3 · 3 · 2 |

**The smallest passing test** (`logger.add(writer, format="{message}")`, then `logger.debug("a")`, reads `"a\n"`): 6 non-blank lines in pytest and in oxitest, 8 or 9 in prototype 2, and 22 to 30 in prototype 1. It needs two imports and five names: `experiment`, the decorator, a `return` as the product, a named case, and `.expect(product=…)`.

**Adding a case** took one line in all three versions. A new test took 4 or 5 lines in pytest, 6 in oxitest, and 7 to 9 in prototype 2; rater 1's passed on the first run in prototype 2, and its first try in oxitest aborted all 22 items on a bare `assert`.

## What the three raters found in prototype 2

All three found these:

- **One case on one line, and refusals that suggest a fix** (`gives 'mesage', which is no input of logged; did you mean 'message'?`; `line 128 holds an assert: an assay checks, so return the value and expect it in the case`). Clarity 4, ease of use 4.
- **The runner is small enough to read:** 83 + 512 non-blank lines, standard library only. Complexity of the implementation 4, against 2 for pytest and oxitest. Prototype 2 does not build everything prototype 1's runner did; `../STRATEGY.md` lists what it leaves out.
- **`handling(ZeroDivisionError("division by zero"), …)` lowers fidelity.** The test builds the exception, so `last_line` reads back a message the test wrote. Rater 2 changed it to `"anything"` and the check followed. Fidelity 4 in all three.
- **The tester learns a vocabulary and a list of refusals:** `experiment`, act, case, `product`, assay, `use`, `handling`, and the constructs refused in an act: 14 or more by rater 1's count, about 25 by rater 2's. Complexity for the tester 2.3: the lowest of the three versions.
- **A bare `# type: ignore` on the three cases with a wrong-typed input hides any other typo on those lines** from both checkers. The runner still refuses it.
- **8 lines over 100 characters**, up to 156: four signatures of an act that carry both fixtures, `use(Writer)` and `use(fresh_logger)`, and four case lines.
- **The runner needs `os.fork`, and the test's source on disk.** None of the three scored it.

One rater each found these:

- A failure screen cuts a reading at 300 characters, so the `with_exception` traceback lost its last line, the line under test (rater 3).
- A case's refusals come one at a time: an assay value of the wrong type stays hidden behind a misspelled assay name on the same case (rater 1).
- One loop in an act refuses all seven cases of its experiment (rater 1).
- A lambda is refused in an act and accepted on a case line (rater 3).

## Files

- `loguru-format/prototype-2/`: the port (`experiments/`), the porter's `NOTES.md`, the real `OUTPUT.txt` and `CHECKERS.txt`.
- `ratings/loguru_rater{1,2,3}_scores.txt`: each rater's scores, weights and findings, unblinded.
- `ratings/KEY.txt`, `ratings/RATER_PROMPT.md`, `ratings/build_blind_loguru.sh`: how the blind folders were built and what each rater was asked.

The runs used Python 3.12.14, pytest 9.2.0, oxitest 4.0.0, loguru 0.8.0, ty 0.0.87 and pyright 1.1.411.
