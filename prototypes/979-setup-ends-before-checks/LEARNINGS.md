# What this prototype taught (#979)

Each answer was probed on purpose, and then held up against real tests that the person did not choose.
The setup answers (Q1, Q4, Q5, Q8, Q9) held with no change. The guard (Q2) gave a false fail on the first real file, and the person narrowed it.
The worst finding is a false pass that does not come from #979.

## Held

- **Q1 + Q5, any `with` is a setup and the act ends at the first check.** 33 of 34 original setups kept their shape. Checks inside (colorama 20, defaults 7, catch 2), after (delay 3, catch 1) and both (probe) all read the state they should. A porter's control moved a check after the undo, and it read `None` as it should (`porters/defaults/prototype/NOTES.md`, §3).
- **Q4, a swallowed raise.** `contextlib.suppress` that skips a line fails the case and names the line; a check it skips shows as blocked. A swallow at the last statement passes (`probes/test_setups.py`).
- **Q8, entry and undo.** Both fail the case and name the setup. A check inside the setup still runs before the undo; a check after it is blocked.
- **Q9.** Nested setups, two items on one line, and `raises` inside a setup all run (`test_setups.py`, the defaults port).
- **Q6, stdout is free.** A module that replaces `sys.stdout` at import muted prototype 5's runner (0 bytes). This runner prints through its own stream, and shows the full screen (`probes/test_guards_still_see.py`).

## Changed by the evidence

- **Q2 → Q2′, when a swap counts.** As first agreed ("the case fails at the swap"), the guard blamed freezegun. `freeze_time()` opens `with warnings.catch_warnings()` while it scans every module, and closes it before the test's next line (`freezegun/api.py:764`). Three loguru items failed for it (`runs/port_filesink-delay_swap.txt`). The person chose: **a hook counts as replaced when it is not the runner's own while a line of the test runs.** The runner notes where a hook was replaced, and checks after each statement of the test, at each check, and at the end of the act.
  - The hole it leaves: a library that replaces a hook and puts it back inside one call of the test, around code under test, hides that code's faults (`probes/test_guard_line.py`, `quietly(noisy)`).

## Found, not #979's

- **A check whose reading calls the code under test passes falsely.** `assert wrap(s) is True` then `assert s.flush.called is False`: pytest fails it. Here the first check runs in a fork (#964), so the call happens only there, and the second check reads the state from before it. Both pass, and nothing on the screen says so (`probes/test_reading_calls_the_code.py`, `probes/pytest_twin/`). The colorama porter met it naturally: 5 cases passed this way on its second attempt. It goes to #980: a reading that acts is a rule for run time or for the code.
- **The porters' costs were rules from #967, not setups:** one `case` per parametrized value, no decorator so no skip (colorama faked `os.name` for 24 Windows items), one comparison per check (27 bare `assert x`), and no place for a session-scoped fixture (defaults dropped `check_env_variables`).
- **FORM.md does not say that a lone `with raises(…)` counts as a check.** The runner treats it as one.

## Holes kept in mind

- A test that imports `signal.alarm` by name before the run holds the real function, and the guard does not see its call.
- A C library that sets a hook does not go through a module's `__setattr__`.
- Two port folders that each hold a `loguru_lab.py` cannot run in one call: Python keeps the first module of a name (inherited from prototype 5).
- #978's replay for each check (live thread) is not in this runner: it starts from prototype 5.
