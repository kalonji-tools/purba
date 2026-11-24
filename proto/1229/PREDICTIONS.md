# #1229 predictions, written before any run of a patched strategy

This branch is never merged.
Each count below is a prediction for the counting run of one strategy over the 9 pinned suites.
It is committed before the strategy's code exists, so that a run can prove it wrong.
`baseline` was measured first, on main's code, at `b3f58f5e` ([`runs/suites/baseline.counts.tsv`](runs/suites/baseline.counts.tsv)).

## Sources

- **#1225** is [`selection-test-markers.md` at `93af24df`](https://github.com/kalonji-tools/purba/blob/93af24dfe1cf5f84151106ee625b5b7e09a0da6e/research/selection-test-markers.md), and its tables [`counts.txt`](https://github.com/kalonji-tools/purba/blob/93af24dfe1cf5f84151106ee625b5b7e09a0da6e/research/selection-test-markers/counts.txt) and `classes.tsv` at the same commit.
- **#1227** is [`selection-strategies.md` at `ceede296`](https://github.com/kalonji-tools/purba/blob/ceede296e32cc271ee090f2dcbcf08243d861be4/research/selection-strategies.md).
- **baseline∩set** is the baseline run, kept to the files that the #1225 set of the strategy names. Where stage two does not change, each file must give the same tests as on main. This source tests that the patch of stage one leaves stage two alone.
- **judgement** marks a count that no line states.

## The matchers of the run

The matchers are those of [`summarize.py`](summarize.py). The yardstick of a "helper" is the suite's own command: a module-level test is a helper caught as a test when its name does not start with `test`, or when its file is neither a test file nor a file that the suite's own command collects.

## What each strategy is in the run

| strategy | stage one, over a walk that skips each folder whose name starts with `.` | stage two |
|---|---|---|
| `baseline` | every `.py` file, each given alone, as main takes one path | `test` prefix |
| `name` | `test_*.py`, `*_test.py` | `test` prefix |
| `name-and-check` | as `name`; a line for each other file whose first or last `_` part is within one edit of `test`, case ignored | as `name`; a line for a test whose name is not `test` and does not start with `test_`; a line for a module-level `def` not taken whose first `_` part is within one edit of `test`, case ignored; a line for a public module-level `def` not taken whose own body holds an `assert` |
| `visibility` | `test_*.py`, `*_test.py` | each name that does not start with `_`, at the three places where main reads `test` |
| `folder` | each `.py` file with a folder named `test`, `tests` or `testing` on its path | `test` prefix |
| `list-and-check` | the `.py` files directly in each folder that `[tool.purba] tests` lists, not in subfolders; a line for each test file outside them | `test` prefix |

For the counting run, the list of each suite is the 15 folders that hold a file the suite's own command collects ([#1227 L373-L374](https://github.com/kalonji-tools/purba/blob/ceede296e32cc271ee090f2dcbcf08243d861be4/research/selection-strategies.md#L373-L374)).

## Counts over the 9 suites (totals)

"methods" are the refused lines of a `def` in a class, one for each.

| strategy | count | predicted | source |
|---|---|---|---|
| `name` | files picked | 189 | #1225 [L191](https://github.com/kalonji-tools/purba/blob/93af24dfe1cf5f84151106ee625b5b7e09a0da6e/research/selection-test-markers.md#L191) |
| `name` | other files picked | 0 | the matcher |
| `name` | test files missed | 0 | #1225 L191 |
| `name` | other collected files missed | 1 (dateutil's solution) | #1225 [L196](https://github.com/kalonji-tools/purba/blob/93af24dfe1cf5f84151106ee625b5b7e09a0da6e/research/selection-test-markers.md#L196) |
| `name` | collected module-level tests missed | 1 | the one test of that file, baseline |
| `name` | module-level tests found | 1,901: 109 selected, 1,792 refused | 1,899 of #1225 [L221](https://github.com/kalonji-tools/purba/blob/93af24dfe1cf5f84151106ee625b5b7e09a0da6e/research/selection-test-markers.md#L221), and `test_b`, `test_u` under `if` of #1227 [L365](https://github.com/kalonji-tools/purba/blob/ceede296e32cc271ee090f2dcbcf08243d861be4/research/selection-strategies.md#L365); the split is baseline∩set |
| `name` | helpers caught as tests | 0 | #1225 [L284](https://github.com/kalonji-tools/purba/blob/93af24dfe1cf5f84151106ee625b5b7e09a0da6e/research/selection-test-markers.md#L284) |
| `name` | methods | 2,505 | #1225 [L244](https://github.com/kalonji-tools/purba/blob/93af24dfe1cf5f84151106ee625b5b7e09a0da6e/research/selection-test-markers.md#L244) |
| `name` | check lines | 0 | the strategy has no check |
| `name-and-check` | every count of `name` | as `name` | stage one and two pick as `name` |
| `name-and-check` | `file-near` lines | 0 | judgement: no line counts file names near the pattern |
| `name-and-check` | `name-prefix` lines | 2, both true | #1225 [L224](https://github.com/kalonji-tools/purba/blob/93af24dfe1cf5f84151106ee625b5b7e09a0da6e/research/selection-test-markers.md#L224) and [L236](https://github.com/kalonji-tools/purba/blob/93af24dfe1cf5f84151106ee625b5b7e09a0da6e/research/selection-test-markers.md#L236): `tests_weakref_…`, which pytest collects |
| `name-and-check` | `name-near` lines | 0 | judgement: no line counts misspelled test names |
| `name-and-check` | `asserts` lines | 25, at most 92; most false | judgement, bounded by the 92 public non-test functions of #1225 [L225](https://github.com/kalonji-tools/purba/blob/93af24dfe1cf5f84151106ee625b5b7e09a0da6e/research/selection-test-markers.md#L225); a helper that asserts is called by the tests |
| `visibility` | files picked, missed | as `name` | stage one is `name`'s |
| `visibility` | module-level `def` tests found | 1,993 | 1,901 of `name`, and the 92 public helpers of #1225 L225 |
| `visibility` | … of the 92, selected | at most 9 | #1225 [L297](https://github.com/kalonji-tools/purba/blob/93af24dfe1cf5f84151106ee625b5b7e09a0da6e/research/selection-test-markers.md#L297): 18 fixtures and 5 decorated are refused, 60 take a parameter with no default; of the 9 others, each that returns a value is refused too |
| `visibility` | names bound with no `def`, refused | at least 724, at most 1,162; judgement 800 | 724 public `from` imports ([#1225 L246](https://github.com/kalonji-tools/purba/blob/93af24dfe1cf5f84151106ee625b5b7e09a0da6e/research/selection-test-markers.md#L246), [L302](https://github.com/kalonji-tools/purba/blob/93af24dfe1cf5f84151106ee625b5b7e09a0da6e/research/selection-test-markers.md#L302)); up to 159 public assignments ([`counts.txt` L68](https://github.com/kalonji-tools/purba/blob/93af24dfe1cf5f84151106ee625b5b7e09a0da6e/research/selection-test-markers/counts.txt#L68)), less those main holds never a function; up to 279 classes ([L242](https://github.com/kalonji-tools/purba/blob/93af24dfe1cf5f84151106ee625b5b7e09a0da6e/research/selection-test-markers.md#L242)), of which only a decorated one counts |
| `visibility` | helpers caught as tests | 92 `def`s, and each name of the row above | the two rows above |
| `visibility` | methods | at least 2,505, at most 2,693 | 2,505 `test*` methods, and 188 other methods in `classes.tsv`, of which only the public ones count |
| `folder` | files picked | 313: 172 test files, 141 others (14 `__init__.py`, 5 `conftest.py`, 122 modules) | #1225 [L190](https://github.com/kalonji-tools/purba/blob/93af24dfe1cf5f84151106ee625b5b7e09a0da6e/research/selection-test-markers.md#L190), [L192](https://github.com/kalonji-tools/purba/blob/93af24dfe1cf5f84151106ee625b5b7e09a0da6e/research/selection-test-markers.md#L192), [L209](https://github.com/kalonji-tools/purba/blob/93af24dfe1cf5f84151106ee625b5b7e09a0da6e/research/selection-test-markers.md#L209), [`counts.txt` L27](https://github.com/kalonji-tools/purba/blob/93af24dfe1cf5f84151106ee625b5b7e09a0da6e/research/selection-test-markers/counts.txt#L27) |
| `folder` | test files missed | 17, of which 2 collected | #1225 [L193](https://github.com/kalonji-tools/purba/blob/93af24dfe1cf5f84151106ee625b5b7e09a0da6e/research/selection-test-markers.md#L193), [L195](https://github.com/kalonji-tools/purba/blob/93af24dfe1cf5f84151106ee625b5b7e09a0da6e/research/selection-test-markers.md#L195) |
| `folder` | other collected files missed | 1 | #1225 L196 |
| `folder` | collected module-level tests missed | 66: six 65, dateutil 1 | #1225 L221 (six's 65), L196 |
| `folder` | module-level tests found | 1,824: 98 selected, 1,726 refused | 1,802 of #1225 [`counts.txt` L72](https://github.com/kalonji-tools/purba/blob/93af24dfe1cf5f84151106ee625b5b7e09a0da6e/research/selection-test-markers/counts.txt#L72), and 22 of #1225 [L212](https://github.com/kalonji-tools/purba/blob/93af24dfe1cf5f84151106ee625b5b7e09a0da6e/research/selection-test-markers.md#L212); the split is baseline∩set |
| `folder` | helpers caught as tests | 22: loguru 20, click 2 | #1225 L212 |
| `folder` | methods | 2,411 | baseline∩set |
| `folder` | lines for the 5 `conftest.py` | 5, one for each | #1225 [L323](https://github.com/kalonji-tools/purba/blob/93af24dfe1cf5f84151106ee625b5b7e09a0da6e/research/selection-test-markers.md#L323) says each becomes a test module that imports pytest. judgement from the code: 0, since main refuses an import of a framework only at a test, and a `conftest.py` holds no `test` name |
| `list-and-check` | files picked | 201: 175 collected, 26 others | #1227 L373; the 26 of #1225 [L204](https://github.com/kalonji-tools/purba/blob/93af24dfe1cf5f84151106ee625b5b7e09a0da6e/research/selection-test-markers.md#L204) sit in the same folders |
| `list-and-check` | test files missed | 15, of which 0 collected | #1225 L191 and L194: 189 − 174 |
| `list-and-check` | `list-outside` lines | 15, each a benchmark | #1225 [L198](https://github.com/kalonji-tools/purba/blob/93af24dfe1cf5f84151106ee625b5b7e09a0da6e/research/selection-test-markers.md#L198) |
| `list-and-check` | module-level tests found | 1,870: 91 selected, 1,779 refused | baseline∩set |
| `list-and-check` | helpers caught as tests | 0 | the 22 of #1225 L212 sit in deeper folders that the list does not name |
| `list-and-check` | methods | 2,502 | baseline∩set |
| `list-and-check` | collected module-level tests missed | 0 | the list names each collected file's folder |

## The probe on the wheel

The probe project `shop/` holds one case of each scenario of the issue. The tester runs `purba .` in it.

| scenario | `name` | `name-and-check` | `visibility` | `folder` | `list-and-check` |
|---|---|---|---|---|---|
| `tests/helpers.py` beside a test file | not picked, no line | not picked, no line | not picked, no line | picked; no test, no line | picked; no test, no line |
| `make_book()` in a test file, returns a value | no test, no line | no test, no line | a test, refused: returns a value | no test, no line | no test, no line |
| `test_root.py` at the root | runs | runs | runs | missed, no line | missed; a `list-outside` line |
| `tset_checkout()` that asserts | no test, no line | a `name-near` line | a test; it runs and passes | no test, no line | no test, no line |
| `tests/conftest.py` with a pytest fixture | not picked | not picked | not picked | picked; no line (judgement from the code) | picked; no line (judgement from the code) |
| `tests/samples/script.py` with `def test_sample()` that prints | not picked | not picked | not picked | a test; it runs, and is not reported as passed (judgement) | not picked: the list is not recursive |
| `extra/test_extra.py`, under no entry of the list | runs | runs | runs | missed, no line | missed; a `list-outside` line |

`baseline` takes no folder: `purba .` is the usage error, as on main.
