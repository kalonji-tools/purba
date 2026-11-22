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
| macOS keeps one sender for each process, and records none for a signal that arrives while the handler masks it | [XNU's `kern_sig.c`](https://github.com/apple-oss-distributions/xnu/blob/xnu-12377.121.6/bsd/kern/kern_sig.c#L2253-L2256), [L2338-L2346](https://github.com/apple-oss-distributions/xnu/blob/xnu-12377.121.6/bsd/kern/kern_sig.c#L2338-L2346) |
| on `macos-latest`, a child that told purba's copy apart from a second interrupt by its sender took one Ctrl-C as two in 60 of 60 rounds | [the prototype's run](https://github.com/kalonji-tools/purba/actions/runs/43694218096) |
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
- **What a test or a module gives an interrupt signal beyond what `signal.getsignal` shows outlives its call or its load, as today.** After a test registers `faulthandler` for SIGINT, a Ctrl-C in a later test is lost, and the run can pass. A test that calls `signal.siginterrupt` with `False` makes a later Ctrl-C wait until a read returns.
- **The first interrupt gives the default action to a signal whose signal handler a test installed through Python's `signal` module.** The test asked for that signal handler. The default action would replace that signal handler for the rest of the call.
- **The first interrupt gives each interrupt signal its default action as the signal arrives.** purba's forwarded copy of a Ctrl-C would then end the test's process before its `finally` runs.
- **The zygote and each test's process take a process group of their own.** A test that reads the terminal then stops on SIGTTIN.
- **A zygote or a test's process tells purba's copy apart from a second interrupt by its sender.** macOS records no sender for a copy that arrives while the handler masks the signal, so the copy reads as a second interrupt.
- **A zygote or a test's process judges its own second interrupt.** One Ctrl-C reaches it twice, so it must count purba's copies on a page that purba shares with it. purba already judges each second interrupt itself.
- **purba forwards SIGUSR2, which the terminal never sends.** A test that uses SIGUSR2 then takes purba's copy.
- **purba forwards no interrupt to the test's process.** An interrupt that another process sends to purba alone then misses the test.
- **An `_Avoid_` line for `cancel` and `abort`.** `typos` rewrites each word in every tracked file. GitHub reads `cancel-in-progress` as a key.

## Decision Outcome

**Reach:** `crates/**` `tests/**` `/CONTEXT.md`

**Area:** interrupt handling

**An [interrupt](../../CONTEXT.md#interrupt) stops the run, and a second one exits at once.**

- **The first interrupt stops the run wherever it lands.** No test or module carries it. No test starts after it, and no module loads. It is no [kind of failure](a-failures-kind-is-what-the-part-that-first-sees-it-observes.md#decision-outcome).
- **The test that runs stops at once, unless its own signal handler, flag or mask holds the interrupt back.** Python unwinds it, so its `finally` runs.
- **SIGTERM and SIGHUP do what SIGINT does.** A CI cancel and a closed terminal each get the cleanup.
- **After each call and each load, each interrupt signal gets back the ignore and the mask that the parent gave it.**
- **The run stops even when a test catches the `KeyboardInterrupt`.** It stops when the test returns.
- **What a test's call or a module's load gives an interrupt signal lives only for that call or load.** This holds in Python and below it, and for the mask of the thread that runs the tests. When the call or the load returns, purba gives each interrupt signal back purba's signal handler, or the ignore that the parent gave it. purba's signal handler or the parent's ignore replaces each signal handler, ignore or flag that the operating system holds for the signal. purba then masks again each interrupt signal that the parent masks, and unmasks the rest. Then purba puts its native handler again in front of each interrupt signal that has purba's signal handler.
- **A second interrupt skips the cleanup left, and exits at once.** A cleanup that hangs cannot trap the tester, except in each case that the Downside list below names. Any two interrupts count, so SIGINT and then SIGTERM make a second. purba's native handler takes each interrupt signal as it arrives, and ends the process at the second. So the second ends purba even while purba or a test runs C code.
- **The [zygote](../../CONTEXT.md#zygote), each test's process and each [replay](../../CONTEXT.md#replay) stay in purba's process group.** A Ctrl-C reaches purba, the zygote, and the test's process or the replay at once.
- **purba forwards each first interrupt to the test's process or the replay, or else to the zygote.** An interrupt that another process sends to purba alone reaches the test too.
- **purba alone judges the second interrupt.** It gets one copy of each Ctrl-C, and the process that purba forwards to gets two. Those processes ignore each interrupt after their first.
- **A second interrupt ends the zygote and the test's process or the replay, and then it ends purba.** No process of the run outlives it.
- **purba takes as its own the first interrupt that a zygote, a test's process or a replay kept.** A mask of the parent's can hold the signal from purba alone. A zygote, a test's process or a replay that an interrupt signal ends does not end purba. purba dies by a signal only when it takes an interrupt as its own.

[A KeyboardInterrupt that no signal caused is the test's own raise](a-keyboardinterrupt-that-no-signal-caused-is-the-tests-own-raise.md#decision-outcome) says when a `KeyboardInterrupt` does not stop the run.
[An interrupt puts back no more than a normal end does](an-interrupt-puts-back-no-more-than-a-normal-end-does.md#decision-outcome) says what each part puts back.
[A process that a test forks has the signal handlers it has without purba](a-process-that-a-test-forks-has-the-signal-handlers-it-has-without-purba.md#decision-outcome) says what a process that a test forks gets for a signal.

**Downside:**

- **A second interrupt leaves built what the skipped cleanup would tear down.**
- **A Ctrl-C during a test that installed its own signal handler, in Python or below it, does not stop that test.** The test's signal handler takes it, as the test asked. A test that registers `faulthandler` for SIGINT can then pass. The run stops when the test returns.
- **A Ctrl-C during a test that called `signal.siginterrupt` with `False` waits until the system call that it lands in returns.** That flag holds for the call, as the test asked.
- **When a test's mask, and not the parent's, holds a Ctrl-C until the call returns, the run stops only then.** The test keeps its outcome, and is not `interrupted`.
- **An interrupt signal that the parent's ignore or mask holds from purba can end a test or a load that gives it its default action.** Under the mask, that process must also unmask it. The test has the outcome `raised`, or the module is a project error, and the run goes on. Its process dies before it can write that it kept the interrupt, so purba cannot tell the signal from that process's own. It excepts [the line of a load](a-tester-runs-purba-on-one-module-and-reads-one-line-for-each-test.md#decision-outcome), [the kind of an interrupt](a-failures-kind-is-what-the-part-that-first-sees-it-observes.md#decision-outcome) and [the outcome of an interrupted test](an-interrupted-run-prints-its-count-and-dies-by-the-signal.md#decision-outcome).
- **A second interrupt that the parent's ignore or mask holds does not exit at once.** It never arrives, so a cleanup that hangs traps the tester. purba dies by the first interrupt's signal once Python finalizes, as [an interrupted run prints its count, and dies by the signal](an-interrupted-run-prints-its-count-and-dies-by-the-signal.md#decision-outcome) says.
- **Two interrupts that a test's process sends itself count as one.** Only purba judges a second.
- **A second interrupt that reaches the test's process alone, and not purba, does not end it.** A Ctrl-C and an interrupt sent to purba both reach purba.
- **No process of the run keeps an interrupt that the parent's mask holds when it reaches only a fork that reads one expectation alone.** It reaches only that fork when a process sends it to that fork's pid alone. That fork ends with its exit code, and sends purba no interrupt that it kept. Its reading raises. So the case gives two readings that differ. The test's process keeps a Ctrl-C, because a Ctrl-C reaches it too.
- **A CI that sends purba SIGINT and then SIGTERM ends the cleanup at the SIGTERM.** GitHub does so when purba is the step's entry process. The time before its SIGKILL goes unused.
- **A C extension that gives an interrupt signal a signal handler when a test module imports it loses that signal handler before the first test.** One that a test imports in its call loses it when the call returns.
- **A test that installs its own signal handler for an interrupt signal can get one Ctrl-C twice.** The terminal sends one, and purba forwards one.

## Confirmation

`mise run test:wheel` checks that a first interrupt starts no later test, and that purba dies by SIGINT.
It checks that the test the interrupt lands in is `interrupted`, and that an interrupt in a load leaves each test unreached.
It checks that a load whose process exits after the interrupt gives no error.
It checks that a second interrupt ends purba at once by that interrupt's signal, even while a cleanup runs C code.
It checks that SIGTERM and SIGHUP do what SIGINT does.
It checks that a Ctrl-C ends the wait for a thread that a test left alive, and lets a test's `atexit` function finish.

`mise run test:wheel` also checks that an interrupt that another process sends to purba alone makes the test `interrupted`, and that its `finally` runs.
It checks that a second Ctrl-C in a test that catches the first leaves no process of the run alive.
It checks that a Ctrl-C that reaches a process of the run after purba forwarded it counts once.
It checks that two signals that wait at once in a test's process count as one interrupt.

`mise run test:wheel` also checks that a second interrupt ends purba at once when the first lands while a test runs C code.
It checks that SIGINT and then SIGTERM end purba at once by SIGTERM.

`mise run test:wheel` also checks that a signal handler of a test or a module lives only for its call or its load.
It checks that a signal handler of a test lives only for its call, even when the test patches `signal.signal`, `_signal.signal` and `_signal.getsignal`.
It checks that a mask of a test or a module lives only for its call or its load.
It checks that a Ctrl-C that a test's mask holds stops the run when the call returns, though the test has its own signal handler.
It checks that a signal that the parent ignores gets the ignore back after the call of a test that gives it a signal handler.
It checks that purba masks again a signal that the parent masks, after the call of a test that unmasks it.

`mise run test:wheel` also checks that a signal handler and an ignore that a test gives SIGINT below Python live only for its call.
It checks that the flag that a test gives SIGINT through `signal.siginterrupt` lives only for the test's call.
It checks that the parent's ignore comes back after a test gives SIGINT its default action below Python.

`mise run test:wheel` also checks that a test or a load that its own interrupt signal ends does not end purba, and that the run goes on.
It checks the same for an interrupt signal that the parent's mask holds, in a test or a load that unmasks it and sets `SIG_DFL`.
It checks the same for a SIGINT that the parent's ignore holds, in a test or a load that sets `SIG_DFL`.
