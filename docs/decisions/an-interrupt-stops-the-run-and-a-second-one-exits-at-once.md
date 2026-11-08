# An interrupt stops the run, and a second one exits at once

## Context and Problem Statement

[The concern interrupt handling](../../LANDSCAPE.md#the-cross-cutting-concerns) stops the run on Ctrl-C, and tears down what the run built.
No record says what an interrupt stops, what a second one does, or which signals count.

| fact | source |
|---|---|
| Ctrl-C ends only the test that runs, and SIGTERM ends purba with no line and no `finally` | [What purba does on an interrupt today](https://github.com/kalonji-tools/purba/issues/765#issuecomment-6626393301) |
| a Ctrl-C between two tests gives the next test `raised: KeyboardInterrupt:` before its first line runs | [the research of signal delivery](https://github.com/kalonji-tools/purba/blob/20b90a1a220c4f3b78f1a3ccf5f364c91dede7a8/research/signal-delivery.md?plain=1#L11) |
| a test that catches the `KeyboardInterrupt` passes, and the next test runs | [the research of signal delivery](https://github.com/kalonji-tools/purba/blob/20b90a1a220c4f3b78f1a3ccf5f364c91dede7a8/research/signal-delivery.md?plain=1#L120) |
| a signal handler that a test installs outlives its test | [the research of signal delivery](https://github.com/kalonji-tools/purba/blob/20b90a1a220c4f3b78f1a3ccf5f364c91dede7a8/research/signal-delivery.md?plain=1#L119) |
| a shell without job control starts a background job with SIGINT ignored, and CPython keeps the ignore | [the research of signal delivery](https://github.com/kalonji-tools/purba/blob/20b90a1a220c4f3b78f1a3ccf5f364c91dede7a8/research/signal-delivery.md?plain=1#L56) |
| twelve of thirteen runners stop the test that runs on the first Ctrl-C; only pytest, pytest-xdist and RSpec run the suite's teardown in every mode | [the research of interrupts in test runners](https://github.com/kalonji-tools/purba/blob/f5a9774123f965e9730d5f35f83d2b44040e3ea4/research/interrupt-runners.md?plain=1#L12-L14) |
| where a runner has a gentle first stage, a second Ctrl-C skips it | [the research of interrupts in test runners](https://github.com/kalonji-tools/purba/blob/f5a9774123f965e9730d5f35f83d2b44040e3ea4/research/interrupt-runners.md?plain=1#L19) |
| every Python runner surveyed dies on SIGTERM with no teardown; five of seven CI cancels and container stops send SIGTERM first | [the research of interrupts in test runners](https://github.com/kalonji-tools/purba/blob/f5a9774123f965e9730d5f35f83d2b44040e3ea4/research/interrupt-runners.md?plain=1#L21-L22) |
| GitHub Actions sends the step's entry process SIGINT, SIGTERM 7.5 s later, and SIGKILL 2.5 s after that | [the research of interrupts in test runners](https://github.com/kalonji-tools/purba/blob/f5a9774123f965e9730d5f35f83d2b44040e3ea4/research/interrupt-runners.md?plain=1#L168) |
| cargo-nextest, the JVM and RSpec handle SIGHUP as they handle SIGTERM | [the research of interrupts in test runners](https://github.com/kalonji-tools/purba/blob/f5a9774123f965e9730d5f35f83d2b44040e3ea4/research/interrupt-runners.md?plain=1#L125-L138) |

## Considered Options

- **What runs carries the interrupt, as today.** A Ctrl-C in a load reads as a project error. One between two tests reads as the next test's raise.
- **The test that runs finishes, and no later test starts, as RSpec does.** A test that hangs never stops on the first interrupt.
- **SIGTERM ends purba by its default action, as today.** A CI cancel then leaves all that the run built.
- **A second interrupt raises again, as the first does.** Each press ends one cleanup, so the tester presses once for each.
- **purba's Python signal handler ends purba on a second interrupt.** It runs only at the next line of Python, so a cleanup that loops in C code keeps purba alive.
- **The first interrupt gives each interrupt signal its default action in purba's Python signal handler, as today.** That handler runs only at the next line of Python, so a second interrupt waits while purba or a test runs C code.
- **A test that catches the `KeyboardInterrupt` continues the run, as today.** The test swallows the tester's Ctrl-C. The run can then exit 0.
- **A signal handler that a test or a module installs outlives its call or its load, as today.** One test or one module disarms Ctrl-C for each later test.
- **An `_Avoid_` line for `cancel` and `abort`.** `typos` rewrites each word in every tracked file. GitHub reads `cancel-in-progress` as a key.

## Decision Outcome

**Reach:** `crates/**` `tests/**` `/CONTEXT.md`

**Area:** interrupt handling

**An [interrupt](../../CONTEXT.md#interrupt) stops the run, and a second one exits at once.**

- **The first interrupt stops the run wherever it lands.** No test or module carries it. No test starts after it, and no module loads. It is no [kind of failure](a-failures-kind-is-what-the-part-that-first-sees-it-observes.md#decision-outcome).
- **The test that runs stops at once.** Python unwinds it, so its `finally` runs.
- **SIGTERM and SIGHUP do what SIGINT does.** A CI cancel and a closed terminal each get the cleanup.
- **An interrupt signal keeps the ignore and the mask that the parent gave it.**
- **The run stops even when a test catches the `KeyboardInterrupt`.** It stops when the test returns.
- **What a test's call or a module's load gives an interrupt signal through Python's `signal` module lives only for that call or load.** When the call or the load returns, purba gives each interrupt signal back purba's signal handler, or the ignore that the parent gave it. purba then masks again each interrupt signal that the parent masks, and unmasks the rest. purba reads and sets only the mask of the thread that runs the tests. Then, after the first interrupt, purba gives each interrupt signal that has purba's signal handler its default action.
- **A second interrupt skips the cleanup left, and exits at once.** A cleanup that hangs cannot trap the tester. Any two interrupts count, so SIGINT and then SIGTERM make a second. As the first interrupt signal arrives, purba gives each interrupt signal that has purba's signal handler its default action. So the second ends purba even while purba or a test runs C code.

[A KeyboardInterrupt that no signal caused is the test's own raise](a-keyboardinterrupt-that-no-signal-caused-is-the-tests-own-raise.md#decision-outcome) says when a `KeyboardInterrupt` does not stop the run.
[An interrupt puts back no more than a normal end does](an-interrupt-puts-back-no-more-than-a-normal-end-does.md#decision-outcome) says what each part puts back.
[A process that a test forks has the signal handlers it has without purba](a-process-that-a-test-forks-has-the-signal-handlers-it-has-without-purba.md#decision-outcome) says what a process that a test forks gets for a signal.

**Downside:**

- **A second interrupt leaves built what the skipped cleanup would tear down.**
- **A Ctrl-C during a test that installed its own signal handler does not stop the run.** The test's signal handler takes it, as the test asked.
- **When a test's mask, and not the parent's, holds a Ctrl-C until the call returns, the run stops only then.** The test keeps its outcome, and is not `interrupted`.
- **A second interrupt that the parent's ignore or mask holds does not exit at once.** It never arrives, so a cleanup that hangs traps the tester. purba dies by the first interrupt's signal once Python finalizes, as [an interrupted run prints its count, and dies by the signal](an-interrupted-run-prints-its-count-and-dies-by-the-signal.md#decision-outcome) says.
- **A CI that sends purba SIGINT and then SIGTERM ends the cleanup at the SIGTERM.** GitHub does so when purba is the step's entry process. The time before its SIGKILL goes unused.
- **A cleanup that installs its own signal handler after the first interrupt takes a second interrupt that comes before the call or the load returns.**
- **A second interrupt can wait for C code after a test or a module gives an interrupt signal back purba's Python signal handler.** Until the call or the load returns, only the C handler of CPython takes that signal. purba's Python signal handler then runs at the next line of Python.

## Confirmation

`mise run test:wheel` checks that a first interrupt starts no later test, and that purba dies by SIGINT.
It checks that the test the interrupt lands in is `interrupted`, and that an interrupt in a load leaves each test unreached.
It checks that a second interrupt ends purba at once by that interrupt's signal, even while a cleanup runs C code.
It checks that SIGTERM and SIGHUP do what SIGINT does.
It checks that a Ctrl-C after the count ends Python's wait for threads, and lets a test's `atexit` function finish.

`mise run test:wheel` also checks that a second interrupt ends purba at once when the first lands while a test runs C code.
It checks that SIGINT and then SIGTERM end purba at once by SIGTERM.

`mise run test:wheel` also checks that a signal handler of a test or a module lives only for its call or its load.
It checks that a signal handler of a test lives only for its call, even when the test patches `signal.signal`, `_signal.signal` and `_signal.getsignal`.
It checks that a mask of a test or a module lives only for its call or its load.
It checks that a Ctrl-C that a test's mask holds stops the run when the call returns, though the test has its own signal handler.
It checks that a signal that the parent ignores gets the ignore back after the call of a test that gives it a signal handler.
It checks that purba masks again a signal that the parent masks, after the call of a test that unmasks it.
