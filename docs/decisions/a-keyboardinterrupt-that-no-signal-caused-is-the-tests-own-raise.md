# A KeyboardInterrupt that no signal caused is the test's own raise

## Context and Problem Statement

[An interrupt stops the run, and a second one exits at once](an-interrupt-stops-the-run-and-a-second-one-exits-at-once.md#decision-outcome) says what an [interrupt](../../CONTEXT.md#interrupt) stops.
No record says whether a `KeyboardInterrupt` that a test raises with no signal stops the run.

| fact | source |
|---|---|
| a terminal's Ctrl-C, `raise KeyboardInterrupt`, `os.kill(os.getpid(), SIGINT)`, `signal.raise_signal` and `_thread.interrupt_main()` each give the line `raised: KeyboardInterrupt:` today | [the research of signal delivery](https://github.com/kalonji-tools/purba/blob/20b90a1a220c4f3b78f1a3ccf5f364c91dede7a8/research/signal-delivery.md?plain=1#L14) |
| a signal handler of purba's own tells a signal from a raise, and `os.kill` of its own process, `raise_signal` and `interrupt_main` reach it as signals | [the research of signal delivery](https://github.com/kalonji-tools/purba/blob/20b90a1a220c4f3b78f1a3ccf5f364c91dede7a8/research/signal-delivery.md?plain=1#L179) |
| `_thread.interrupt_main()` simulates "the effect of a signal arriving in the main thread", SIGINT unless told another, and does nothing when Python handles no such signal | [CPython's `_thread.rst`](https://github.com/python/cpython/blob/v3.15.2/Doc/library/_thread.rst?plain=1#L61-L72) |
| pytest re-raises a `KeyboardInterrupt` from a test, unless `--pdb` is set | [pytest's `runner.py`](https://github.com/pytest-dev/pytest/blob/9.2.0/src/_pytest/runner.py#L263-L267) |
| unittest re-raises a `KeyboardInterrupt` from a test | [CPython's `case.py`](https://github.com/python/cpython/blob/v3.15.2/Lib/unittest/case.py#L59-L60) |

## Considered Options

- **A `KeyboardInterrupt` that no signal caused stops the run, as in pytest and unittest.** Rejected. A test of code that lets a `KeyboardInterrupt` escape then hides each later test.
- **A `KeyboardInterrupt` that no signal caused gives the test the outcome `raised`, as today.** Chosen. Execution sees a raise, and no signal.

## Decision Outcome

**Reach:** `crates/**` `tests/**`

**Topic:** One section of the product design; 

**A `KeyboardInterrupt` that no signal caused is the test's own raise.**

- **A test whose call raises it, with no signal during the call, has the [outcome](../../CONTEXT.md#outcome) `raised`.** The next test runs.
- **A subclass of `KeyboardInterrupt` counts the same.** The line names the subclass.
- **A module whose load raises it with no signal is a [project error](../../CONTEXT.md#project-error).** Each of its tests is a [blocked test](../../CONTEXT.md#blocked-test).
- **An interrupt that a test sends to its own process stops the run.** A call to `_thread.interrupt_main()` simulates a SIGINT, so it is an interrupt too. Each reaches purba's signal handler as a terminal's Ctrl-C does.
- **A `KeyboardInterrupt` from a thread is a thread's exception.** [A failure's kind is what the part that first sees it observes](a-failures-kind-is-what-the-part-that-first-sees-it-observes.md#decision-outcome) holds its kind.

```
test_a.py::test_one raised: KeyboardInterrupt:
test_a.py::test_two passed
2 tests: 1 passed, 1 raised
```

**Downside:**

- **A tester who knows pytest expects the run to stop.** pytest ends its session on a `KeyboardInterrupt` from a test.
- **A Ctrl-C that a signal handler of the test turns into a `KeyboardInterrupt` gives the test the outcome `raised`.** purba's signal handler never sees the signal.
- **A test that sends its own process SIGINT stops the run.** A test of a program's Ctrl-C path must install its own signal handler for its call, or signal a child process.

## Confirmation

`mise run test:rust` checks that purba keeps no signal after a bare raise, and that a SIGINT that a test sends stops the run.
`mise run test:wheel` checks that a test that raises `KeyboardInterrupt` with no signal is `raised`, and that the next test runs.
