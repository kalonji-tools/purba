# A chain as a fork tree (#1010 Q1): throwaway prototype

This prototype answers Q1 of [#1010](https://github.com/kalonji-tools/purba/issues/1010) with numbers.
It starts from the runner of `../980-rules-at-run-time/` and changes only what Q1 needs.
Never merge it.

## The question

A chain is a later case that takes `earlier.product` as an input.
Today the runner replays: each later case runs every earlier case of its chain again first (#966).
Should a chain run as a fork tree instead?
Each stage's act runs once. The stage then forks once for its own checks and once for each later stage.
It falls back to replay when a thread other than the main one is alive after the act, or when the chain reads a stock.

## The falsifier

The answer "fork tree + fallback" falls if either is true:

1. A case's outcome differs between replay and the fork tree on the same corpus.
2. Replay caught a real divergence in the corpus: an act that read differently on replay, not just a failed earlier check.

## The verdict: it held, with one more fallback, and it saves time only on slow acts

Neither falsifier fired on the corpus.
Every outcome is the same in both modes, on all 12 targets.
Replay wrote "diverged on replay" on 3 checks, and the fork tree wrote the same 3. Each time the earlier stage's own check had failed. None was drift.
The fork tree ran 70 acts where replay ran 311.
It saved no time on this corpus, because its acts cost microseconds and a fork costs more.
When the root act (`create_app`) costs 1 ms or more, the fork tree is faster. At 20 ms it takes 0.50 s against replay's 1.22 s.

The corpus alone proves little about chains.
980's folders hold 183 cases and **no chain case**. The pipeline probe holds 9.
So this prototype adds `chains/`, 43 chain cases that mirror pytest fixtures that depend on fixtures.
In total the corpus holds **52 chain cases**.

Two injected controls go further than the corpus. They are not evidence about real tests.

- **An act that reads `random`** (`controls/test_coin.py`). Replay caught the drift: in 10 of 40 runs the stage passed and its later stage was blocked. Replay also missed the other way: in 6 of 40 runs the stage failed, and its later stage still passed on a second toss that the report never showed. The fork tree never disagreed with itself: the later stage passed exactly when the stage passed. This is what the fork tree gives up. It is a flaky earlier check, not a silent wrong pass.
- **A product that holds an open file** (`controls/test_handle.py`). This one falsified the fallback list as #1010 states it. A fork shares a file's offset. The first later stage read the file, and the second read `''` and failed. Replay passed both. Neither a thread nor a stock was involved.

So the runner here adds a third fallback: **a file, socket or pipe that the act opened and left open**.
It compares `/proc/self/fd` at the start of the case with the fork point.
With it, the control gives the same outcome as replay. It fired nowhere in the corpus. `FORM_FDS=off` turns it off, to show what it guards.

## Numbers

Outcomes and acts (`runs/outcomes.txt`, `runs/compare.txt`). An act is one call of the code under test, counted through `PIPE_LOG`. 980's folders log no act.
`runs/compare.txt` also compares the two outputs as text. Two add-option-catch targets differ there only in loguru's `elapsed` field, a time.

| target | cases | chain cases | outcomes | acts, replay → tree | fallbacks in the tree |
|---|---|---|---|---|---|
| 980's 9 folders of `runall.sh` | 183 | 0 | the same | — | 0 |
| `pipeline/test_pipeline.py` | 15 | 9 | the same | 27 → 14 | 0 |
| `pipeline/test_part.py` | 0 (the file is refused at import) | 1 | the same | 0 → 0 | 0 |
| `chains/` | 48 | 43 | the same | 284 → 56 | 6 later stages: 3 for loguru's writer thread, 3 for a SQLite stock |

Some acts per mode: `sign_up ann` ran 11 times under replay and 3 under the tree. `create_app` ran 32 times and 1. A replay runs each earlier act again for every later case, so its count grows with the depth of the chain times its width.

Every "diverged on replay" line, in both modes:

| later case | earlier check | kind |
|---|---|---|
| `from_wrong` (2 checks) | `wrong › len(…) == lines`, expected 9, read 1 | a failed earlier check |
| `after_wrong` | `wrong_total › response.body['total'] == total`, expected 999, read 1200 | a failed earlier check |

Wall time, best of 5, the modes taking turns (`runs/timing.txt`):

| target | replay | tree |
|---|---|---|
| 980's 9 folders (14.4 s of it is each `threading` folder) | 33.93 s | 33.92 s |
| `pipeline/test_pipeline.py` | 0.11 s | 0.13 s |
| `pipeline/test_part.py` | 0.05 s | 0.05 s |
| `chains/` | 0.46 s | 0.47 s |

A sensitivity, not corpus evidence (`runs/timing_cost.txt`): `chains/` with `create_app` made to sleep (`APP_COST_MS`). Replay runs `create_app` 32 times, the tree once.

| `create_app` costs | replay | tree |
|---|---|---|
| 0 ms | 0.46 s | 0.47 s |
| 1 ms | 0.50 s | 0.45 s |
| 5 ms | 0.65 s | 0.48 s |
| 20 ms | 1.22 s | 0.50 s |

## What the runner changes

- `--tree` runs each chain as a fork tree; without it, the runner replays.
- **A refused earlier stage blocks each later stage, in both modes.** The block names it: `blocked: quiet was refused`. Under 980's runner, `quiet_books` and `after_one_way` passed after a refused stage (`runs/*_980runner.txt`).
- A stage forks at its first check, or at its end if it reaches none. The fork for its own checks runs first. The continuation reads the checks as a replay does, so a failed check blocks the later stages with the replay's own words.
- A case that takes two products forks from the first, and replays the others. `bob_cannot_cancel` replays `bob_in`.
- On a fallback, the stage runs as any case. The runner then replays each of its later stages, from the root of the chain.
- Each case writes its results to a file of its own, so a process deep in a tree can write them. The runner runs the cases in chain order and prints them in file order.
- A forked later stage does not run `before_each` again, and restarts the watch's events, which the earlier act switched off.

## What surprised

- **The open-file hole.** The fork tree needs a fallback that #1010 does not name. 980 already refuses an open file that two checks read, for the same reason.
- **Replay is not stricter in every way.** On the coin, replay passed a later stage whose stage had failed in its own case.
- **The fallback for a stock always fires.** A stock's chain never forks. On `chains/test_store.py` the tree saved nothing.
- **A cross-file chain does not run.** `test_part.py` takes `ann.product.shop`, a part of a product. Importing it raises `AttributeError`, in both modes. Running `pipeline/` as one folder also imports `test_pipeline.py` unrewritten first, so each file runs alone here.

## Run it

```sh
uv venv -p 3.12 .venv && uv pip install -p .venv/bin/python loguru==0.8.0 freezegun==1.5.0 pytest==9.2.0
bash runall.sh                     # every target in both modes → runs/
.venv/bin/python compare.py > runs/outcomes.txt   # outcomes, chain cases, each divergence
bash controls.sh                   # the injected controls → runs/controls.txt
.venv/bin/python timing.py > runs/timing.txt             # best of 5
.venv/bin/python timing.py cost > runs/timing_cost.txt   # create_app made slower
cd chains && PYTHONPATH=../undertest_lib ../.venv/bin/python ../run.py --tree .
```

The code under test sits in `undertest_lib/`, outside each test folder. Otherwise 980's act scope would hold it to the one-path rule.

`runs/` was made on CPython 3.12.15 with loguru 0.8.0.
