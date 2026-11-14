# A setup that ends before its checks (#979) — throwaway prototype

This prototype tests the answers to [How does a test set up something that ends before its checks?](https://github.com/kalonji-tools/purba/issues/979) on real tests before they are posted.
It starts from prototype 5 of #967 (`proto/967-python-form-5` at 7e2b40c), and changes only what #979 decides.
Never merge it.

The answers held on 114 real loguru items from four files. Every item passes but one, the item whose original swaps `threading.excepthook`, which Q2 fails on purpose. 33 of the 34 setups that the originals write as `with` kept that shape in the port; the one that did not is a fixture's own `with` around its `yield`.
The probe run found two defects. One is a false fail, where freezegun's own `catch_warnings` window inside `freeze_time()` tripped the warnings guard; the person then chose "a hook counts while a line of the test runs". The other is a false pass that comes from #964's fork for each check, and it goes to #980.

## What #979 decided, and where the runner does it

| # | decision | in the runner |
|---|---|---|
| Q1 | Any `with` in the act is a setup. Its body is act, under one path. | `is_setup`, `read_with` |
| Q2, Q2′ | A hook the runner sees faults through may not be replaced while a line of the test runs. The case fails, and names the line that replaced it. | `Guard` (`FORM_GUARD=line`, the default) |
| Q3 | A fixture takes no argument. A value that needs one is a `with`, or is built in the act. | nothing to build |
| Q4 | A setup that swallows a raise fails the case when a statement inside it never ran. A raise at its last statement may be swallowed. | `__form_mark__` / `__form_left__`, `left()` |
| Q5 | The act ends at the first check. Checks sit at the end of a setup, after it, or both. | `tail()`, `Rewrite` |
| Q6 | Guarded: `threading.excepthook`, `sys.unraisablehook`, the warning filters, the `SIGALRM` time limit. `sys.stdout` and `sys.stderr` are free; the runner prints through a stream of its own. | `GUARDED`, `say()` |
| Q7 | No form for "expect a lost error" until a test needs one. | nothing to build |
| Q8 | A setup whose entry or undo raises fails the case, and names it. | `setup_at()` |
| Q9 | Nested setups, and several items in one `with`, are allowed. | `read_with` recursion |

The markers add no frame to a setup's `__exit__`, so a library that reads its caller's frame (loguru's `logger.catch()`) still sees the test's frame. `probes/test_catch_reads_the_tests_frame.py` shows it.

## Run it

```sh
uv venv -p 3.12 .venv && uv pip install -p .venv/bin/python loguru==0.8.0 freezegun==1.5.0 pytest==9.2.0
cd probes && ../.venv/bin/python ../run.py .                 # one folder at a time: each port has its own loguru_lab.py
FORM_GUARD=swap ../.venv/bin/python ../run.py test_guard_line.py   # the guard as first agreed
```

## Results (`runs/`, Python 3.12.15, loguru 0.8.0)

| folder | what | cases | passed | notes |
|---|---|---|---|---|
| `probes/` | each answer, falsified on purpose | 27 | 13 | every verdict matches its docstring; 2 refused on purpose |
| `ports/filesink-delay/` | prototype 5's port, 3 `with freeze_time(…)` back | 10 | 10 | `FORM_GUARD=swap`: 3 fail on freezegun's window |
| `ports/add-option-catch/` | prototype 5's port, 3 `with` back, + 1 item without the hook swap | 12 | 11 | the thread-hook swap fails (Q2); without it, it passes |
| `porters/defaults/` | an independent porter, `test_defaults.py` | 14 | 14 | 7 of 7 tests on the first natural attempt |
| `porters/colorama/` | an independent porter, `test_colorama.py` | 79 | 79 | 7 of 24 tests on the first natural attempt; 20 of 21 `with` kept as written |

The porters had only `FORM.md`, the original, and `cases.py`. Their notes are `porters/*/prototype/NOTES.md`.

Runner size, non-blank lines: 1056, against 836 for prototype 5.
