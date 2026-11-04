# An interrupt stops the run, and a second one exits at once

## Context and Problem Statement

[The concern interrupt handling](../../LANDSCAPE.md#the-cross-cutting-concerns) stops the run on Ctrl-C, and tears down what the run built.
No record says what an interrupt stops, what a second one does, or which signals count.

| fact | source |
|---|---|
| Ctrl-C ends only the test that runs, and SIGTERM ends purba with no line and no `finally` | [What purba does on an interrupt today](https://github.com/kalonji-tools/purba/issues/765#issuecomment-6626393301) |
| a Ctrl-C between two tests gives the next test `raised: KeyboardInterrupt:` before its first line runs | [the research of signal delivery](https://github.com/kalonji-tools/purba/blob/20b90a1a220c4f3b78f1a3ccf5f364c91dede7a8/research/signal-delivery.md?plain=1#L11) |
| a test that catches the `KeyboardInterrupt` passes, and the next test runs | [the research of signal delivery](https://github.com/kalonji-tools/purba/blob/20b90a1a220c4f3b78f1a3ccf5f364c91dede7a8/research/signal-delivery.md?plain=1#L120) |
| a handler that a test installs outlives its test | [the research of signal delivery](https://github.com/kalonji-tools/purba/blob/20b90a1a220c4f3b78f1a3ccf5f364c91dede7a8/research/signal-delivery.md?plain=1#L119) |
| a shell without job control starts a background job with SIGINT ignored, and CPython keeps the ignore | [the research of signal delivery](https://github.com/kalonji-tools/purba/blob/20b90a1a220c4f3b78f1a3ccf5f364c91dede7a8/research/signal-delivery.md?plain=1#L56) |
| twelve of thirteen runners stop the test that runs on the first Ctrl-C; only pytest, pytest-xdist and RSpec run the suite's teardown in every mode | [the research of interrupts in test runners](https://github.com/kalonji-tools/purba/blob/f5a9774123f965e9730d5f35f83d2b44040e3ea4/research/interrupt-runners.md?plain=1#L12-L14) |
| where a runner has a gentle first stage, a second Ctrl-C skips it | [the research of interrupts in test runners](https://github.com/kalonji-tools/purba/blob/f5a9774123f965e9730d5f35f83d2b44040e3ea4/research/interrupt-runners.md?plain=1#L19) |
| every Python runner surveyed dies on SIGTERM with no teardown; five of seven CI cancels and container stops send SIGTERM first | [the research of interrupts in test runners](https://github.com/kalonji-tools/purba/blob/f5a9774123f965e9730d5f35f83d2b44040e3ea4/research/interrupt-runners.md?plain=1#L21-L22) |
| GitHub Actions sends the step's entry process SIGINT, SIGTERM 7.5 s later, and SIGKILL 2.5 s after that | [the research of interrupts in test runners](https://github.com/kalonji-tools/purba/blob/f5a9774123f965e9730d5f35f83d2b44040e3ea4/research/interrupt-runners.md?plain=1#L168) |
| cargo-nextest, the JVM and RSpec handle SIGHUP as they handle SIGTERM | [the research of interrupts in test runners](https://github.com/kalonji-tools/purba/blob/f5a9774123f965e9730d5f35f83d2b44040e3ea4/research/interrupt-runners.md?plain=1#L125-L138) |

## Considered Options

- **What runs carries the interrupt, as today.** Rejected. A Ctrl-C in a load reads as a project error. One between two tests reads as the next test's raise.
- **The interrupt stops the run wherever it lands.** Chosen. No later work starts. No test or module carries it.
- **The test that runs finishes, and no later test starts, as RSpec does.** Rejected. A test that hangs never stops on the first interrupt.
- **The test that runs stops at once, and unwinds.** Chosen. Python raises inside the test, so its `finally` runs.
- **SIGTERM ends purba by its default action, as today.** Rejected. A CI cancel then leaves all that the run built.
- **SIGTERM and SIGHUP do what SIGINT does.** Chosen. A CI cancel and a closed terminal each get the cleanup.
- **A second interrupt raises again, as the first does.** Rejected. Each press ends one cleanup, so the tester presses once for each.
- **A second interrupt skips the cleanup left, and exits at once.** Chosen. A cleanup that hangs cannot trap the tester.
- **A test that catches the `KeyboardInterrupt` continues the run, as today.** Rejected. The test swallows the tester's Ctrl-C. The run can then exit 0.
- **A handler that a test installs outlives its test, as today.** Rejected. One test disarms Ctrl-C for each later test.
- **An `_Avoid_` line for `cancel` and `abort`.** Rejected. `typos` rewrites each word in every tracked file. GitHub reads `cancel-in-progress` as a key.

## Decision Outcome

**Reach:** `crates/**` `tests/**` `/CONTEXT.md`

**An [interrupt](../../CONTEXT.md#interrupt) stops the run, and a second one exits at once.**

- **The first interrupt stops the run wherever it lands.** No test starts after it, and no module loads. It is no [kind of failure](a-failures-kind-is-what-the-part-that-first-sees-it-observes.md#decision-outcome).
- **The test that runs stops at once.** Python unwinds it, so its `finally` runs.
- **SIGTERM, SIGHUP and a Ctrl-Break on Windows do what SIGINT does.**
- **A signal that the parent ignores stays ignored.**
- **The run stops even when a test catches the `KeyboardInterrupt`.** It stops when the test returns.
- **A handler that a test installs lives only for the call of that test.** purba puts its own back when the call returns.
- **A second interrupt skips the cleanup left, and exits at once.** Any two interrupts count, so SIGINT and then SIGTERM make a second.

**Downside:**

- **A second interrupt leaves built what the skipped cleanup would tear down.**
- **A Ctrl-C during a test that installed its own handler does not stop the run.** The test's handler takes it, as the test asked.
- **A CI that sends purba SIGINT and then SIGTERM ends the cleanup at the SIGTERM.** GitHub does so when purba is the step's entry process. The time before its SIGKILL goes unused.

## Confirmation

No command checks this yet.
The map [Wayfinder: purba's interrupt handling, stopping the run and tearing down what it built](https://github.com/kalonji-tools/purba/issues/765) builds it, and ends with a test that sends each case to an installed `purba`.

- [A Ctrl-C outside a test's call gives the next test raised, or vanishes](https://github.com/kalonji-tools/purba/issues/774)
- [Is a test that raises KeyboardInterrupt itself an interrupt?](https://github.com/kalonji-tools/purba/issues/770)
