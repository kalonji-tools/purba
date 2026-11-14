# The benchmark of prototype 1 (#967). Never merge.

A later prototype of the Python form of an experiment is measured against these numbers.
It lives on a branch of its own, and ports the same loguru test.

## The test

`loguru-format/` holds one real test file, ported three times:

| folder | what it is | items | passes | ty | pyright |
|---|---|---|---|---|---|
| `pytest/` | loguru's `tests/test_add_option_format.py` at [`28349be`](https://github.com/Delgan/loguru/blob/28349be3cd740867bc947eef741c69370ca45735/tests/test_add_option_format.py) (MIT), with the two fixtures it uses from loguru's `conftest.py` | 20 | 20 | 9 diagnostics, all in loguru's own `conftest.py` | 9 errors in loguru's own `conftest.py`, and 2 warnings on two `1 / 0` lines |
| `oxitest/` | a port to oxitest 4.0.0, by an agent that read only oxitest's user docs | 20 | 20, strict mode `abort` | clean | clean |
| `prototype-1/` | a port to `record_strict/` at this commit, by an agent that followed the form's rules and changed no runner file; the port itself is `../record_strict/ex_loguru_format.py` and `loguru_lab.py` | 44 cases | 44 | clean | 1 warning |

Each folder holds the code, the real `OUTPUT.txt`, the real `CHECKERS.txt`, and the porter's `NOTES.md` where there is a port.
The runs used Python 3.12.14, pytest 9.2.0, oxitest 4.0.0, loguru 0.8.0, ty 0.0.87 and pyright 1.1.411.

## The numbers to beat

Three raters scored the three versions, each in a folder of its own, with the two ports under shuffled letters, and without the issue, the design history or any earlier score.
Each ran every version, added a case and a test to each, wrote the smallest passing test in each, and seeded failures.
5 is best; for both complexity rows, 5 is simplest.

| | readability | clarity | ease of use | complexity for the tester | complexity of the implementation | smallest test | fidelity to the original | weighted, rater 1 · 2 · 3 |
|---|--:|--:|--:|--:|--:|--:|--:|---|
| pytest | 5.0 | 4.0 | 4.0 | 4.0 | 3.7 | 5.0 | reference | 4.31 · 4.40 · 4.35 |
| oxitest | 4.0 | 5.0 | 4.0 | 3.0 | 2.7 | 4.0 | 5.0 | 4.20 · 4.15 · 4.10 |
| prototype 1 | 2.0 | 2.7 | 2.0 | 1.0 | 1.7 | 1.0 | 2.7 | 1.95 · 1.85 · 2.10 |

**The smallest passing test** checks that `logger.add(writer, format="{message}")` and then `logger.debug("a")` write `"a\n"`: 5 or 6 lines in pytest, 6 in oxitest, and 22 to 30 in prototype 1, which needs two classes and a chain.

Each rater chose a weighting; `ratings/loguru_rater{1,2,3}_scores.txt` hold each rater's scores, weights and findings, unblinded.

## The earlier rating: eight forms of seven written examples

Before the loguru test, three raters scored the eight forms of `../` on the examples `ex1`-`ex7`, blind.
`ratings/forms_rater{1,2,3}_scores.txt` hold the scores. The mean of the three:

| form | mean | ranks |
|---|--:|---|
| `call/` | 4.11 | 1 · 2 · 1 |
| `below/` | 3.94 | 2 · 3 · 2 |
| `record/` | 3.67 | 3 · 1 · 3 |
| `klasscall/` | 3.33 | 4 · 4 · 4 |
| `stacked/` | 3.28 | 7 · 6 · 5 |
| `module/` | 3.22 | 5 · 7 · 7 |
| `klass/` | 3.22 | 6 · 5 · 6 |
| `record_strict/`, with a typed `expect` in every experiment | 2.11 | 8 · 8 · 8 |

The typed `expect` became a level after this rating, at `100420d`; the loguru rating measured the form with both levels.

`ratings/build_blind_*.{py,sh}` built the blind copies, and `ratings/FORM_rules_given_to_raters.md` is the rules sheet the loguru raters got.
