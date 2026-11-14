# What this prototype taught (#980)

Each answer was probed on purpose, then held against tests written by porters who had only the rules sheet.
The answers held, and the run found two facts that correct what was said during the grilling: the cost of the watch, and what the colorama porter's "five false passes" were.

## Held

- **A rule that moved to run time stops leaking.** The four leaks that prototype 5's porters found in the one-path rule are gone: a module-level helper, an imported helper, a module-level class, and a callback handed over by assignment (`probes/test_paths.py`). The stock rule sees a read through an alias or a derived name (`probes/test_stock_fingerprint.py`).
- **The test written next is the original.** `for t in threads: t.join()` passes as written, so the `range(2)` workaround, and the mistake it allowed, have no reason to exist (`first-attempts/threading/`, 0 → 7 accepted).
- **A rule that moved still holds.** filesink-delay's first attempt is still refused in all 10 cases, each by rule D: asserts mid-test, and asserts in the `check_dir` helper. The refusal now names the helper's line (`runs/first-attempts_filesink-delay_980.txt`).
- **Alone and in order.** A check whose reading calls the code under test is refused, and the refusal names the check before it. On the colorama port with the call moved back into the first check, #979's runner failed 12 cases without naming why; this one refuses the same 12 and names each cause (`regress/colorama-read-calls/`).
- **Nothing else changed.** Five folders of finished ports, 122 cases, give the same counts under both runners. #979's own probes change in three cases only, all on purpose: the false pass is refused, in its probe and in the pytest twin that the runner also collects, and an assert in the act is refused while the case runs instead of before it.
- **The forks cost nothing.** Reading each later check alone takes as many forks as #979's runner took (one for each check but one), and switching them off saves no time (`runs/timing_split_colorama.txt`).

## Changed by the evidence

- **The colorama porter's "five false passes" were not false passes.** On Linux, `should_wrap` returns before it calls `winapi_test`, so `winapi_test.called is False` holds in pytest too, in any order (`regress/colorama-v2/`, 66 passed under both runners). What #979's fork for each check hid is a regression of that kind: if `should_wrap` called `winapi_test` first, the check read alone would still hold. The probe `test_reading_calls_the_code` shows that this runner refuses it. The comment on #980 that reported five false passes overstated them.
- **The watch's cost is per case, not per round.** The grilling quoted a loop of a million rounds: 108 ms without the watch, 93 ms with it. That holds, but it misses a cost paid once in each case: every library function that a case runs for the first time instruments itself in the case's process. On colorama, 79 cases, that is 0.93 s → 1.81 s. Two first designs cost more: events set globally in each case (about 7 ms a case more on a trivial test), and events set on the test's own code in each case (colorama 3.2 s). Setting them once in the runner's process, before the forks, gave the cost above. A builder may cut it further; this prototype did not try.
- **On CPython 3.12, an exhausted `for` lands past its `END_FOR`,** two bytes beyond the jump target that `dis` shows. The runner tells a loop's round by its body's first instruction instead.

## Holes kept in mind

- **A callback that C code calls is read as act.** `sorted(["bb", "a"], key=key)` leaves no frame for `sorted`, so `key` looks called by the test, and its `if` is held to one path (`probes/test_paths.py`, `test_callback_called_by_c`). The same function, called by Python code, is free.
- **A module of the test's folder that is first imported inside a case is not watched:** the events are set when the file is loaded.
- **Rules that #980 did not weigh stay as they were:** an open file or socket that two checks read is still found by the names the checks read; `global`, `nonlocal`, `yield`, `await`, `async with` and a `with` inside a block are still read from code.
- **The fork with a thread still alive, inherited from prototype 5.** In one of the four saved runs of `first-attempts/threading/` (`runs/summary.txt`; the three of `runs/threading_repeats.txt` passed), 3 cases failed on Python's own `DeprecationWarning` about a fork in a process with threads. #978 answers it, with a replay of the act for each check; this runner, like #979's, does not build it.
