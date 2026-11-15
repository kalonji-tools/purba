# The test form as #1010 decided it: throwaway prototype

This folder is the form that the blind rating of #1010 scores. It is never merged.
A blind porter reads `FORM.md` alone, and ports tests into the form; blind raters then score the ports.
The runner is the stage-as-input prototype's (`../1010-d-use-stage/`, at 89315ce), with the fork tree of the fork-tree prototype (`1010-a-fork-tree/` on `proto/1010-a-fork-tree`, at 48aaede) ported in, and every decision of #1010's "Resolved" comment applied.

## Run it

```sh
uv venv -p 3.12 .venv && uv pip install -p .venv/bin/python flask==3.1.2 loguru==0.8.0
bash runall.sh          # probes, examples and regression targets → runs/
cd examples/tests && PYTHONPATH=../src ../../.venv/bin/python ../../run.py .
```

`runall.sh` reads the fork-tree prototype's `chains/` from its worktree; set `PA` to that folder if it sits elsewhere.
`runs/` was made on CPython 3.12.15, flask 3.1.2 and loguru 0.8.0.

## What the runner does for each decision

| # | decision | in `run.py` | probe |
|---|---|---|---|
| 1 | a chain runs as a fork tree, with replay when a thread is alive, the chain reads a stock, or the act left a file open | `run_chains`, `fork_stage`, `spawn_later`, `fallback_reason`; no flag | `test_tree.py`: each act runs once (`count(…) == 1` three stages down); each fallback prints `↺ … ran by replay: …`, and its act ran twice |
| 2 | a failed or refused stage blocks each later stage, and the line names it | `stage_gone`, `blocked_by`, `block_subtree`, `mark_refused` | `test_blocked.py`: `which failed`, `which was refused` (at run time and before the run), `which was blocked` two stages down, and under a replay |
| 3 | `use(stage)` is a default input | `resolve`, `stages_of` | `test_use_stage.py` |
| 4 | a stage of another test module is loaded rewritten, and every rule applies | `load`, and `visit` in `run_file`: the stage runs in this file's run, under its own module's shape and watch | `test_far.py` with `test_far_home.py`: a path refusal of the far stage blocks its user, also when `test_far.py` runs alone (`runs/probes_far-alone.txt`) |
| 5 | `before_each` leaves the form; `@before(fn)` replaces it | `before` and `BEFORE` in `cases.py`, and a `__getattr__` whose refusal points to `@before`; `build` runs each `@before` after the stages a case takes and before its fixtures; `is_before` refuses every other decorator | `test_before_each.py`: the file is refused at import; `test_before.py`: order, with a fixture, after a stage, a raising reset, another decorator refused; `test_reset.py`: a reset as a fixture still works |
| 6 | no one-act rule | none; the one-call fixture rule (`FORM_FIXTURE`) is removed too | `test_two_acts.py` |
| 7 | the watch covers only the test module, and its `@contextmanager` functions that a `with` of the test enters | `Paths.is_ours` compares the file; `Paths.py_start` steps over contextlib's frame; `arm` takes files | `test_watch.py`: code in the tests' folder passes, a one-way `@contextmanager` of the module is refused |
| 8 | an assert inside a reading is refused | `Paths.reading_of`, and its message in `Paths.line` | `test_reading_assert.py`: refused when the helper's assert holds and when it fails |
| 9 | a refusal in a helper names the test's line and the helper's line | `Paths.called_at`, `test_line`, `who` | `test_helper_lines.py`: a helper, a helper of a helper (the test's line), a helper in a reading (the check's line) |
| 10 | a fixture may use a fixture; each factory runs once in a case; a cycle, and a factory parameter with no `use()` default, are refused before the run | `resolve` keeps one value of each factory in `made`; `fixture_faults` walks every factory from `shape_of` and `refuse_case` | `test_composed.py`; `test_stock_in_a_fixture.py`: a stock used by a factory keeps its rules |
| 11 | stages are for probing a pipeline; fixtures share setup | `FORM.md` only | `examples/` |

## Probe results

Each probe's docstring begins with what the run must show, and may quote text with `Says "…"`.
`verify.py` reads the summary that the runner writes to `FORM_SUMMARY`, and compares.
Every probe matches: 94 tests and one refused file, of 19 files, in `runs/probes_verify.txt`, and 5 tests in `runs/probes_far-alone_verify.txt`.

| file | what it proves |
|---|---|
| `test_tree.py` | 1: three stages, each act once; the three fallbacks replay |
| `test_blocked.py` | 2 |
| `test_use_stage.py`, `test_use_far.py` | 3 and 4, from the stage-as-input prototype |
| `test_far.py`, `test_far_home.py` | 4 |
| `test_before_each.py`, `test_before.py`, `test_reset.py` | 5 |
| `test_two_acts.py` | 6 |
| `test_watch.py`, `flat.py` | 7 |
| `test_reading_assert.py` | 8, with one hole: an asserting helper of another module passes |
| `test_helper_lines.py` | 9 |
| `test_composed.py`, `test_stock_in_a_fixture.py` | 10 |
| `test_paths.py`, `test_reading_acts.py`, `test_stock_fingerprint.py`, `test_case_names.py`, `helpers.py` | prototype 980's probes |

The whole run: `runs/probes.txt`, 113 cases: 55 passed, 8 failed, 40 refused, 10 blocked, and 1 file refused.
A cycle of fixtures cannot be written with `def`s alone, since a default names only an earlier function: `test_composed.py` builds one by setting `__defaults__`.
`runs/probes_acts.txt` counts the shop's acts over `test_use_stage.py`: `sign_up ann` runs 3 times, where the stage-as-input prototype's replay ran it 9 times.

A control (`runs/control_previous-runner.txt`) runs six of the new probe files under the stage-as-input runner.
There each later stage of `test_tree.py` is blocked, because a replay runs the act again; `test_before_each.py` imports; code in the tests' folder is refused; and no refusal names the test's line.
So the probes tell the two runners apart.

## Regression: the flaskr ports and the fork tree's `chains/`

The outcome of every case is the same as before, on all five targets.

| target | before | now |
|---|---|---|
| `blog-fixtures` | 13 passed | 13 passed |
| `blog-fixtures_mutant-login` | 4 passed, 9 failed | 4 passed, 9 failed |
| `blog-stages` | 15 passed | 15 passed |
| `blog-stages_mutant-login` | 5 passed, 1 failed, 9 blocked | 5 passed, 1 failed, 9 blocked |
| `chains` (under `--tree`) | 44 passed, 1 failed, 1 refused, 2 blocked | the same |

Each difference in the text, and why:

- **`blog-stages`, both runs: each later stage prints `↺ … ran by replay: the act left a file, socket or pipe open (fd 6 → …)`** (item 1). `test_app_with_data` calls `tempfile.mkstemp()` and never closes the descriptor. That fallback also protects the port: its database is a file that a fork does not copy, and the later stages write to it.
- **`chains`: a blocked line names the stage and what it did** (item 2). `blocked: one_way_in was refused` became `blocked: it takes the product of one_way_in (test_web.py:208), which was refused`. `blocked: wrong_total › … diverged on replay: expected 999, read 1200` became `… which failed`. A blocked case prints one line, not one line for each of its checks.
- **`chains`: the thread fallback's reason** reads `a fork copies only the thread that forks`, not an issue number.
- `blog-fixtures_mutant-login` differs only in the address of a `sqlite3.Row`. The fd path in the `↺` lines changes with each run.

Prototype 980's probes, under this runner (`runs/probes.txt`):

- **`test_assert_in_a_helper` passes; it was refused** (item 7). `check_dir` sits in `helpers.py`, which is not the test module.
- **`no_threads` names 3 faults, not 5** (item 7). The two in `start_all`, of `helpers.py`, are gone.
- **Each refusal in a helper adds `reached from line N of the test`** (item 9).

## Holes kept in mind

- **An asserting or branching helper of another module is not watched** (items 7 and 8). `test_asserting_helper_of_another_module` and `test_assert_in_a_helper` pass. #1010 keeps this in mind, and decides no rule for it.
- **A stage of another module runs again in each file that takes it,** and counts in that file's tally too. In a folder run, `test_far_home.py`'s cases show twice in the count.
- **A callback that C code calls is read as the test's act** (`test_callback_called_by_c`): C leaves no frame. This is prototype 980's hole, unchanged.
- **The watch now compares files**, so code that Python generates (a dataclass's `__init__`, whose file is `<string>`) is never the test's. The stage-as-input runner compared folders, and read `<string>` as a file of the tests' folder: in the control, `Shop()` built in an act was refused for its generated `__init__`.
- The open-file fallback reads `/proc/self/fd`, so it works on Linux only.

## Files

- `FORM.md`: the rules sheet the porter reads.
- `run.py`, `cases.py`: the runner and what a test imports. `verify.py`: probe docstrings against a run. `runall.sh`: every run into `runs/`.
- `examples/tests/`: `test_members.py` (a fixture, a fixture that uses it, a stage, `use(stage)`), `test_loans.py` (a pipeline of three stages on a stage of `test_members.py`, `@before`, `raises`, a `with` setup). `examples/blocked/test_due_dates.py`: a stage that fails on purpose, and the stage it blocks. `examples/src/lending.py`: their code under test.
- `probes/`: the probes above. `undertest/`: their code under test (`shop.py`, `chain.py`, `registry.py`, `undertest.py`).
- `runs/`: each output, folded at 100 bytes.
