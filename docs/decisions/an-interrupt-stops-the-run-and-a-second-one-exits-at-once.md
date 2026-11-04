# An interrupt stops the run, and a second one exits at once

## Context and Problem Statement

[The concern interrupt handling](../../LANDSCAPE.md#the-cross-cutting-concerns) stops the run on Ctrl-C, and tears down what was built.
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
| GitHub Actions sends SIGINT, SIGTERM 7.5 s later, and SIGKILL 2.5 s after that | [the research of interrupts in test runners](https://github.com/kalonji-tools/purba/blob/f5a9774123f965e9730d5f35f83d2b44040e3ea4/research/interrupt-runners.md?plain=1#L168) |
| cargo-nextest, the JVM and RSpec handle SIGHUP as they handle SIGTERM | [the research of interrupts in test runners](https://github.com/kalonji-tools/purba/blob/f5a9774123f965e9730d5f35f83d2b44040e3ea4/research/interrupt-runners.md?plain=1#L125-L138) |

## Considered Options

- **What runs carries the interrupt, as today.** Rejected. A Ctrl-C in a load reads as a project error, and one between two tests as the next test's raise.
- **The interrupt stops the run wherever it lands.** Chosen. No later work starts, and no test or module carries it.
- **The test that runs finishes, and no later test starts, as RSpec does.** Rejected. A test that hangs never stops on the first interrupt.
- **purba passes the signal on, waits, then kills the test, as cargo-nextest does.** Rejected. It needs each test in a process of its own, and dispatch holds that.
- **The test that runs stops at once, and unwinds.** Chosen. Python raises inside the test, so its `finally` runs.
- **SIGTERM ends purba by its default action, as today.** Rejected. A CI cancel then leaves all that the run built.
- **SIGTERM does what a second interrupt does.** Rejected. A CI cancel then skips every cleanup too.
- **SIGTERM and SIGHUP do what SIGINT does.** Chosen. A CI cancel and a lost terminal each get the cleanup.
- **SIGHUP stays out.** Rejected. A closed terminal then leaves all that the run built.
- **purba takes a signal that the parent ignores.** Rejected. It breaks `nohup`.
- **A signal that the parent ignores stays ignored.** Chosen. The parent decides, as CPython lets it decide for SIGINT.
- **A second interrupt is ignored, as `go test` does.** Rejected. A cleanup that hangs then ends only by SIGKILL.
- **A second interrupt raises again, as the first does.** Rejected. Each press ends one cleanup, so the tester presses once for each.
- **A second interrupt skips the cleanup left, and exits at once.** Chosen. A cleanup that hangs cannot trap the tester.
- **Only a second signal of the same kind counts.** Rejected. Under a GitHub cancel, SIGKILL cuts the cleanup anyway.
- **Any two interrupts make a second.** Chosen. One rule counts them.
- **A test that catches the `KeyboardInterrupt` keeps the run going, as today.** Rejected. The tester's Ctrl-C is lost, and the run can exit 0.
- **A handler that a test installs outlives its test, as today.** Rejected. One test disarms Ctrl-C for each later test.
- **An `_Avoid_` line for `cancel`, `abort` and `kill`.** Rejected. `typos` rewrites each word in every tracked file, and GitHub reads `cancel-in-progress` as a key.

## Decision Outcome

**Reach:** `crates/**` `tests/**` `/CONTEXT.md`

**An [interrupt](../../CONTEXT.md#interrupt) stops the run, and a second one exits at once.**

- **The first interrupt stops the run wherever it lands.** No test starts after it, and no module loads. It is no kind of error.
- **The test that runs stops at once.** Python unwinds it, so its `finally` runs.
- **SIGTERM and SIGHUP do what SIGINT does.**
- **A signal that the parent ignores stays ignored.**
- **A test that catches the `KeyboardInterrupt` does not keep the run going.** The run stops when the test returns.
- **A handler that a test installs lives only for the call of that test.** purba puts its own back when the call returns.
- **A second interrupt skips the cleanup left, and exits at once.** Any two interrupts count, so SIGINT and then SIGTERM make a second.

**Downside:**

- **A second interrupt leaves undone the cleanup that it skips.** What that cleanup would tear down stays built.
- **A Ctrl-C during a test that installed its own handler does not stop the run.** The test's handler takes it, as the test asked.
- **A CI that sends SIGINT and then SIGTERM, as GitHub does, ends the cleanup at the SIGTERM.** The time before its SIGKILL goes unused.

## Confirmation

No command checks this yet.
The map [Wayfinder: purba's interrupt handling, stopping the run and tearing down what it built](https://github.com/kalonji-tools/purba/issues/765) builds it, and ends with a test that sends each case to an installed `purba`.

- [A Ctrl-C outside a test's call gives the next test raised, or vanishes](https://github.com/kalonji-tools/purba/issues/774)
- [What does an interrupted run print, and which code does it exit?](https://github.com/kalonji-tools/purba/issues/769)
- [Is a test that raises KeyboardInterrupt itself an interrupt?](https://github.com/kalonji-tools/purba/issues/770)
