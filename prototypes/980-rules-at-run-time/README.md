# Rules at run time (#980) — throwaway prototype

This prototype tests the answers to [Which rules of the form does purba check while a case runs, rather than read from a test's code?](https://github.com/kalonji-tools/purba/issues/980) on real tests before they are posted.
It starts from the prototype of #979 (`proto/979-setup-ends-before-checks` at 44c5abc), and changes only what #980 decides.
Never merge it.

The answers held. Prototype 5's porters first wrote 28 tests as close to the originals as its rules seemed to allow, and its runner refused 24. #979's runner refuses 23 of them, and this one 16. The 7 it no longer refuses are all of `test_threading.py`, whose loops over the threads it takes as written; the 16 it still refuses break rules that stay (C) or that moved and still hold (D). Five folders of finished ports, 122 cases, give the same count under both runners.
The watch of the act's paths costs about 11 ms a case on colorama, whose cases call much library code: 0.93 s became 1.81 s. On the other three files it added 0.05 to 0.18 s, and on the slowest, threading, 0.1 s of 14.7.

## What #980 decided, and where the runner does it

| round | decision | in the runner |
|---|---|---|
| 1, Q1 | A rule moves to run time when the run can see its fault, goes when the test written next is weaker, and stays in the code otherwise. | how each rule below was placed |
| 1, Q2 | Every check reads in place, in order, and each later check is also read alone. A check whose two verdicts differ is refused. | `fork_alone`, `hook` |
| 2, Q1 | **A** The act has one path in each case: each line runs, each branch goes both ways or is not there, each loop runs a round. The act is the test's own code that the test calls; who calls a function decides whether it is a value. | `Paths`, `arm` |
| 2, Q2 | **G** A stock is watched by its type: a fingerprint, or else a checkpoint, around each check. | `Fingerprints` in `cases.py`, `read_guarded`, `shape_of` |
| 2, Q3 | **D** An assert that runs in the act, a helper's or a callback's too, is refused. **F** A line of the act that runs after an expected raise is refused. | `Paths.line`, `RAISED` in `cases.py` |
| 2, Q4 | **C** one comparison, **E** a literal expected value, **H** no decorator stay in the code. **E** now takes operators on literals. | `read_check`, `literal_or_why`, `shape_of` |
| 3, Q1 | **I** a case in which no check ran, and **J** a case that no name or two names hold, are found at run time. | `in_case`, `run_file` |
| 3, Q2 | **K** a check of a value the act cannot change, **M** an expected input that takes a literal, **P9** public names stay in the code. | `refuse_case` (P9 is not built, as before) |
| 3, Q3 | A fault of the form found while a case runs is **refused**, not failed: refused is the test's fault, failed is the code's. | `report` |

## Run it

```sh
uv venv -p 3.12 .venv && uv pip install -p .venv/bin/python loguru==0.8.0 freezegun==1.5.0 pytest==9.2.0
cd probes && ../.venv/bin/python ../run.py .        # each answer, falsified on purpose
bash runall.sh                                       # every folder below, under the #979 runner and this one
.venv/bin/python timing.py                           # the two runners, taking turns, best of seven
```

## Results (`runs/`, CPython 3.12.14, loguru 0.8.0)

| folder | what | #979 runner | this runner |
|---|---|---|---|
| `probes/` | each answer, falsified on purpose; the verdict is in each docstring | — | 33 cases: 11 passed, 22 refused, every verdict as its docstring says |
| `first-attempts/threading/` | prototype 5's first attempt at `test_threading.py` | 0 of 7 accepted | 0 refused; 7 passed in each of the 3 runs of `runs/threading_repeats.txt`. In the run saved as `runs/first-attempts_threading_980.txt`, 3 cases failed on Python's own warning about a fork in a process with threads, which prototype 5 found and #978 answers; this runner does not build #978 |
| `first-attempts/filesink-delay/` | its first attempt at `test_filesink_delay.py` | 0 of 10 | 0 of 10: asserts mid-test and in the `check_dir` helper (rule D, moved, not dropped) |
| `first-attempts/add-option-catch/` | its first attempt at `test_add_option_catch.py`, with #979's `loguru_lab.py` (the first attempt's own was not kept) | 4 passed, 6 refused, 1 failed | the same: 6 by rule C, which stays, and 1 by #979's hook guard, on purpose |
| `regress/{filesink-delay,add-option-catch,threading,defaults,colorama}/` | the finished ports of #979 and of prototype 5 | 122 cases: 121 passed, 1 failed | the same counts |
| `regress/colorama-read-calls/` | the colorama port with its 8 `wrapped = should_wrap(stream)` lines moved back into the first check | 67 passed, 12 failed, for a reason it does not name | 67 passed, 12 refused, each naming the check before it |
| `regress/colorama-v2/` | the colorama porter's own second attempt | 66 passed, 12 failed, 1 refused | 66 passed, 13 refused |
| `../979-setup-ends-before-checks/probes/` | #979's probes | 13 passed | 11 passed: its false pass is now refused (in the probe and in its pytest twin), and its assert in the act is refused at run time instead of before it |

Time, best of seven, the two runners taking turns (`runs/timing.txt`): colorama 0.93 s → 1.81 s, defaults 0.31 → 0.36, filesink-delay 1.33 → 1.51, threading 14.69 → 14.79 (best of three). A second run that switches each part off (`runs/timing_split_colorama.txt`, best of five) puts colorama's extra second on the watch: 0.98 s for #979's runner, 1.90 s for this one, 1.03 s with the watch off, 1.91 s with the alone forks off.

Runner size, non-blank lines: 1252, against 1056 for #979's.
