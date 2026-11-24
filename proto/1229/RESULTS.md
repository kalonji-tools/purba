# #1229 prototype: how each marking strategy behaves on the 9 pinned suites

This branch is never merged.
Every count below comes from a run of purba's own selection, patched on this branch, over the 9 suites of [the cost of the rules](https://github.com/kalonji-tools/purba/blob/8e712dbf/prototypes/1015-rules-cost/projects.txt) at their pinned commits.
Nothing in the suites was imported or run.
Each prediction beside a count was committed in [`PREDICTIONS.md`](PREDICTIONS.md) at `b25237ed`, before the code of any strategy other than `baseline` existed.

Most predictions held.
`name`, `folder` and `list-and-check` found the files and the tests that #1225's counts give, to the unit.
Five counts differ from their prediction (✗ in the grid).
The near-miss check of `name-and-check` reported 6 functions in 189 files, not the 27 predicted: its 2 `tests_…` hits are true, and its 4 "asserts" hits are false.
`visibility` turned 956 helpers into tests: the 92 public functions, and 864 names that an import or an assignment binds, which selection then refuses for having no `def`.
None of the 92 public functions would run as a test: each is refused, for a parameter with no default, a returned value, a decorator or an import of pytest.
`folder` missed 17 test files without a line, and picked 141 other files; 125 of them, each `conftest.py` among them, gave no line either.
`list-and-check` reported 15 test files outside the list; all 15 are benchmarks that the suites keep apart on purpose.

## Wheels and runs

| what | commit | how |
|---|---|---|
| counting runs | `a5a7b530` (the code), main's code at `b3f58f5e` for the first `baseline` | `cargo build --release -p purba-command --example tally`, then [`suites.sh`](suites.sh) |
| wheel, main | `3093871b` | `mise run build` in a clone, installed with `pip install --no-index --find-links` into a scratch venv; sha256 `6e614b87…` |
| wheel, prototype | `a5a7b530` | the same, from the worktree with a clean tree under `crates/`, `Cargo.*` and `pyproject.toml`; sha256 `13b32100…` |

Python 3.12.14, Linux.
`baseline` was measured first on main's code, at `b3f58f5e`.
The patched code under `baseline` gives a byte-identical tally ([control](#controls)).

```sh
proto/1229/suites.sh target/release/examples/tally <suites dir> proto/1229/runs/suites <strategy>...
proto/1229/run.sh <venv>/bin/purba proto/1229/runs/probe <work> "wheel from a5a7b530"
proto/1229/run.sh <main venv>/bin/purba proto/1229/runs/probe <work> "wheel from 3093871b (main)" main
```

## The grid, over the 9 suites

A cell reads `measured (predicted)`. ✗ marks a count that differs from its prediction. `—` means the strategy has no such line.
A **helper caught as a test** is a module-level test whose name does not start with `test`, or whose file is neither a test file nor a file that the suite's own command collects.
**Lines the tester sees** are the lines printed before any test runs: one for each refused test, each refused method, and each line of a check. The line of each test that runs comes on top, one for each selected test.

| strategy | files found | module-level tests found | helpers caught as tests | test files missed (of 189), of them collected | near-miss hits, true / false | lines the tester sees |
|---|---|---|---|---|---|---|
| `baseline`, every `.py` given alone | 485 | 1,927 | 25 | 0 | — | 4,314: 1,809 tests, 2,505 methods |
| `name` | 189 (189) | 1,901 (1,901) | 0 (0) | 0 (0) | — | 4,297 (4,297): 1,792 tests, 2,505 methods |
| `name-and-check` | 189 (189) | 1,901 (1,901) | 0 (0) | 0 (0) | 2 / 4 ✗ (2 / about 25) | 4,303 ✗ (4,324): 4,297 and 6 check lines |
| `visibility` | 189 (189) | 2,857 ✗ (2,793; bound 2,717 to 3,155) | 956 ✗: 92 `def`s (92), 864 names (800; bound 724 to 1,162) | 0 (0) | — | 5,360 (bound 5,104 to 5,739): 2,748 tests, 2,612 methods |
| `folder` | 313 (313): 172 test files, 141 others | 1,824 (1,824) | 22 (22) | 17, of them 2 (17, 2) | — | 4,137 ✗ (4,142): 1,726 tests, 2,411 methods; 0 lines for `conftest.py` (5) |
| `list-and-check` | 201 (201): 175 collected, 26 others | 1,870 (1,870) | 0 (0) | 15, of them 0 (15, 0) | 0 / 15 `list-outside` (15) | 4,296 (4,296): 1,779 tests, 2,502 methods, 15 check lines |

The predicted "lines the tester sees" is the sum of the predicted rows of `PREDICTIONS.md`; no row predicted it whole.
Per suite, each count is in `runs/suites/<strategy>.counts.tsv`, and each file, test and line is in `runs/suites/<strategy>.tsv.gz`.

### What each strategy did in the probe

The probe project is [`runs/probe/tree.txt`](runs/probe/tree.txt). The tester ran `purba .`, and `purba tests/test_lend.py`.
Each cell names its run file `runs/probe/<strategy>.dot.txt`. Every cell matched its prediction, except the rows that the run added (marked "added").

| scenario | `name` | `name-and-check` | `visibility` | `folder` | `list-and-check` |
|---|---|---|---|---|---|
| `tests/helpers.py` beside a test file | not picked | not picked | not picked | picked, no line | picked, no line |
| `make_book()`, a public helper | no line | no line | refused: `[returned-value] the test returns a value` | no line | no line |
| `test_root.py` at the root | passed | passed | passed | missed, no line | missed; `list-outside: test_root.py` |
| `tset_checkout()` that asserts | no line | `name-near: tests/test_lend.py:13: tset_checkout is not a test…` | passed, as a test | no line | no line |
| `tests/conftest.py` with a pytest fixture | not picked | not picked | not picked | picked, no line | picked, no line |
| `tests/samples/script.py`, `def test_sample()` that prints | not picked | not picked | not picked | its print on stdout, then `project error: [no-expectation] no expectation stated` | not picked |
| `extra/test_extra.py`, no entry lists it | passed | passed | passed | missed, no line | missed; `list-outside: extra/test_extra.py` |
| added: `tests/tset_typo.py`, a misspelled file name | missed, no line | missed; `file-near: tests/tset_typo.py` | missed, no line | `test_typo` passed | `test_typo` passed |
| added: `tests_return()`, as attrs's `tests_weakref_…` | passed | passed; `name-prefix` line | passed | passed | passed |
| added: `assert_titled(book)`, a helper that asserts | no line | `asserts: … assert_titled is not a test, and it asserts` | refused: `the parameter book has no default` | no line | no line |
| added: `from json import dumps` in a test file | no line | no line | refused: `[name-without-def] line 1 binds dumps with no def` | no line | no line |
| exit code of `purba .` | 0 | 0 | 4 | 4 | 0 |

`baseline` and main's wheel print the same for both commands: `purba .` is `usage: purba <path to one .py file>` with exit 3, and `purba tests/test_lend.py` runs 3 tests ([`main.dot.txt`](runs/probe/main.dot.txt), [`baseline.dot.txt`](runs/probe/baseline.dot.txt)).
On `purba tests/test_lend.py`, every strategy gives `baseline`'s 3 tests, except `visibility` (7 cases, 3 refused) and `name-and-check` (the 3 lines of stage two).

## Where a measurement differs from its prediction

1. **`name-and-check`, `asserts` lines: 4, predicted about 25.** A public helper that holds a bare `assert` is rarer than I judged. All 4 are false hits: `compare_exception` (loguru), `assert_init_annotations` (attrs), `assert_warned_from_this_file` and the fixture `NYC` (dateutil). Each is read elsewhere in its suite: 6, 14, 5 and 58 times (`grep -rnw`).
2. **`name-and-check`, lines: 4,303, predicted 4,324.** It follows from the row above.
3. **`visibility`, names bound with no `def`: 864, predicted 800 (bound 724 to 1,162).** 724 are the public `from` imports of #1225, as predicted. 72 are assignments (of 159), and 68 are names that #1225's `bindings.tsv` does not hold: decorated classes and bindings in a block. The count sits inside the bound; the point judgement was low.
4. **`visibility`, module-level tests found: 2,857, predicted 2,793.** It follows from the row above.
5. **`folder`, lines: 4,137, predicted 4,142.** #1225 [L323](https://github.com/kalonji-tools/purba/blob/93af24dfe1cf5f84151106ee625b5b7e09a0da6e/research/selection-test-markers.md#L323) says each `conftest.py` becomes a test module that imports pytest. It is picked, and it gives no line: selection refuses an import of another framework only at a test ([`lib.rs` L1042](https://github.com/kalonji-tools/purba/blob/3093871b/crates/selection/src/lib.rs#L1042)), and a `conftest.py` holds no `test` name. My judgement from the code, also in `PREDICTIONS.md`, predicted 0.
6. **`name` and `folder`, "collected module-level tests missed": 0 and 65, predicted 1 and 66.** This is my matcher, not the strategy. The matcher reads #1225's `functions.tsv`, which holds only the functions of test files, so it cannot see the one test of dateutil's `docs/exercises/solutions/mlk_day_rrule_solution.py`. That file is missed under both, as predicted ("other collected files missed": 1). Under `folder`, the 2 tests of six under `if` are missed too, and the matcher does not see them either; with them, `folder` misses 68 collected tests.

Each other predicted count matched to the unit, including the 109 selected and 1,792 refused of `name`, the 98 and 1,726 of `folder`, and the 91 and 1,779 of `list-and-check`. Stage two did not change for them, and the counts show that the patch of stage one left it alone.

## Each near-miss hit, judged

[`judged.tsv`](judged.tsv) holds each hit and its verdict. I judged each by reading its `def` and by `grep -rnw <name>` over its suite.
A hit is **true** when it is a test whose marker is off: nothing in the suite reads the name, and the suite runs it or means it as a test.
A hit is **false** when it is a helper or a fixture that the suite reads, or a file that the suite keeps out of its run on purpose.

| check | hits | true | false | what they are |
|---|---|---|---|---|
| `name-prefix` | 2 | 2 | 0 | `tests_weakref_does_not_add_…` in `attrs/tests/test_slots.py`; pytest collects them, and only their `def` names them |
| `asserts` | 4 | 0 | 4 | the 4 helpers of the row 1 above |
| `name-near` | 0 | — | — | no misspelled test in the 189 files |
| `file-near` | 0 | — | — | no misspelled test file in the 485 files |
| `list-outside` | 15 | 0 | 15 | `attrs/bench/test_benchmarks.py` and 14 files of `toolz/bench/`; no command of either suite collects them |

## Other findings

- **`visibility` refuses 956 helpers, and runs none of them.** Of the 92 public functions, 71 take a parameter with no default, 43 return a value, 23 carry a decorator that is not purba's, 11 are generators and 5 are `async`; a function can give more than one reason. The 5 that give none of these are refused only because their module imports pytest. A missed `_` shows as a refused test, never as a silent pass, in these suites. In the probe, `tset_checkout`, which a tester meant as a test, passed.
- **`visibility` also refuses 107 more methods** (2,612 against 2,505): every public method of a class in a test file is now a test that purba cannot call.
- **`folder` adds 141 files, and 125 of them give no line.** They hold no `test` name, so a helper module, an `__init__.py` and a `conftest.py` pass in silence. The other 16 hold the 22 helpers that become tests: 20 in sample scripts of loguru, 2 in typing checks of click. In the probe, the sample script's `print` reached the tester's stdout before its refusal.
- **`folder` picks a misspelled test file** (`tests/tset_typo.py`) and runs it. `list-and-check` does the same. Under `name` and `visibility`, the file is missed with no line; only `name-and-check` names it.
- **`list-and-check` needs 15 entries for the 9 suites** and reports the 15 benchmark files. Each report is a file the suite keeps apart. A suite that adds an entry for `bench` turns them into test files.
- **A near-miss check prints 6 lines over 1,901 tests.** 2 of the 6 are true.

## What this prototype did not vary

- **The list is not recursive.** A folder entry picks the `.py` files directly in it. A recursive list would pick loguru's 101 sample scripts and click's 12 typing files under `tests`, as `folder` does.
- **Each suite's list** is the 15 folders that hold a file its own command collects. No other list was tried.
- **The near-miss predicate**: one edit from `test`, on the first or the last `_` part of a name; "asserts" means a bare `assert` in the function's own body. Methods are not checked. A wider predicate was not tried.
- **Where a check line goes**: stderr, before any test, and it never changes the exit code. A check that refuses was not tried.
- **A file named alone** is taken as given under every strategy; stage one applies only to a walk of a folder.
- **The folder names** `test`, `tests`, `testing`; **the file names** `test_*.py`, `*_test.py`; **the function prefix** `test`, not `test_`.
- **`visibility`** treats a dunder as private, and changes all three places where main reads `test`, so a public name with no `def` is refused, and a public method is a refused test.
- **The walk** skips each folder and file whose name starts with `.`, and each linked folder. The suites hold no virtual environment or build folder.
- **One Python (3.12.14), one OS (Linux), the `dev` profile of the wheel.** The counting runs read selection's verdicts; no selected test of a suite was run.
- **No form was rated blind.** `visibility`, `folder` and `list-and-check` change a form that a tester writes, and each owes a rating before it is decided.

## Controls

- `baseline` on the patched code gives a tally identical to main's, byte for byte: `cmp <(zcat runs/suites/baseline.tsv.gz) <(... tally of a5a7b530 under baseline)`.
- `baseline` on the wheel of `a5a7b530` prints what main's wheel prints, for both commands.
- After `cargo fmt`, the 6 tallies of `a5a7b530` are identical to the runs committed here.

## Files

| path | what it holds |
|---|---|
| `PREDICTIONS.md` | each prediction and its source, committed before the strategies' code |
| `summarize.py` | the matchers, and the counts of one tally |
| `suites.sh` | the counting run over the 9 suites |
| `run.sh` | the probe on a wheel |
| `lists/<suite>.toml` | the `[tool.purba] tests` list of each suite for `list-and-check` |
| `ground/files.tsv`, `ground/functions.tsv.gz` | #1225's tables at `93af24df`, the yardstick of the matchers |
| `judged.tsv` | each near-miss and list hit, and its verdict |
| `runs/suites/<strategy>.tsv.gz`, `.counts.tsv` | each tally, and its counts per suite |
| `runs/probe/<strategy>.<target>.txt`, `tree.txt` | each probe run, and the probe project |
