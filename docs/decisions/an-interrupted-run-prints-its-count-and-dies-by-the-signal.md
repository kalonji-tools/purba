# An interrupted run prints its count, and dies by the signal

## Context and Problem Statement

[An interrupt stops the run, and a second one exits at once](an-interrupt-stops-the-run-and-a-second-one-exits-at-once.md#decision-outcome) says what an [interrupt](../../CONTEXT.md#interrupt) stops.
No record says what an interrupted run prints, or which code it exits.

| fact | source |
|---|---|
| the exit-code record takes 0, 1, 2, 3, 4 and 125, and gives 2 to `unknown`; pytest exits 2 for an interrupted run | [A run exits 0 only when it checked something and every test passed](a-run-exits-0-only-when-it-checked-something-and-every-test-passed.md#decision-outcome) |
| a line for a test goes to stdout, and a line for the run goes to stderr | [A line names its kind in the glossary's words](a-line-names-its-kind-in-the-glossarys-words.md#decision-outcome) |
| a Ctrl-C in a test gives it `raised: KeyboardInterrupt:`, and the run exits 4; SIGTERM ends purba with no line and no count | [What purba does on an interrupt today](https://github.com/kalonji-tools/purba/issues/765#issuecomment-6626393301) |
| after the last test, a Ctrl-C ends purba with a traceback and exit 130, after a count that promised exit 4 | [the research of signal delivery](https://github.com/kalonji-tools/purba/blob/20b90a1a220c4f3b78f1a3ccf5f364c91dede7a8/research/signal-delivery.md?plain=1#L12) |
| a test that catches the `KeyboardInterrupt` passes | [the research of signal delivery](https://github.com/kalonji-tools/purba/blob/20b90a1a220c4f3b78f1a3ccf5f364c91dede7a8/research/signal-delivery.md?plain=1#L120) |
| CPython ends itself by SIGINT when a `KeyboardInterrupt` goes unhandled, because otherwise "a calling process such as a shell may not know about the user's ^C" | [CPython's `main.c`](https://github.com/python/cpython/blob/v3.15.2/Modules/main.c#L793-L799) |
| on Windows, CPython exits `STATUS_CONTROL_C_EXIT` instead, which `cmd.exe` detects | [CPython's `main.c`](https://github.com/python/cpython/blob/v3.15.2/Modules/main.c#L808-L811) |
| `subprocess` reports a child that signal N ended as `-N` | [CPython's `subprocess.rst`](https://github.com/python/cpython/blob/v3.15.2/Doc/library/subprocess.rst?plain=1#L970-L971) |
| `git bisect run` aborts on any code above 127, and git counts a child that signal N ended as 128 plus N | [git's `git-bisect.adoc`](https://github.com/git/git/blob/v2.56.0/Documentation/git-bisect.adoc?plain=1#L333-L338), [git's `run-command.c`](https://github.com/git/git/blob/v2.56.0/run-command.c#L574-L583) |
| three of thirteen runners can exit 0 after an interrupt that left tests unrun | [the research of interrupts in test runners](https://github.com/kalonji-tools/purba/blob/f5a9774123f965e9730d5f35f83d2b44040e3ea4/research/interrupt-runners.md?plain=1#L18) |
| cargo-nextest counts the tests that an interrupt kept from running: `2/3 tests were not run due to interrupt` | [the research of interrupts in test runners](https://github.com/kalonji-tools/purba/blob/f5a9774123f965e9730d5f35f83d2b44040e3ea4/research/interrupt-runners.md?plain=1#L89) |
| pytest prints no count after a second Ctrl-C | [the research of interrupts in test runners](https://github.com/kalonji-tools/purba/blob/f5a9774123f965e9730d5f35f83d2b44040e3ea4/research/interrupt-runners.md?plain=1#L109) |
| GitHub Actions sends the step's entry process SIGINT, then SIGTERM 7.5 s later | [the research of interrupts in test runners](https://github.com/kalonji-tools/purba/blob/f5a9774123f965e9730d5f35f83d2b44040e3ea4/research/interrupt-runners.md?plain=1#L168) |

## Considered Options

- **An interrupted run prints as today.** A Ctrl-C reads as the test's own raise. The run exits 4. A CI cancel leaves no count.
- **An interrupted run exits 2, as pytest's does.** purba's 2 means `unknown`.
- **A test that catches the `KeyboardInterrupt` keeps its own outcome, as today.** The interrupt cut its body, so a pass checked less than it claims.
- **A test that the run never reached counts as `not run`.** purba does not run a blocked test either.

## Decision Outcome

**Reach:** `crates/**` `tests/**` `/CONTEXT.md`

**Area:** interrupt handling

**An interrupted run prints its count, and dies by the signal.**
A shell that runs purba then sees the interrupt, and `git bisect run` aborts.

- **purba prints one line on stderr as the interrupt arrives.** It names the signal, and what a second interrupt does. [An interrupt stops the run, and a second one exits at once](an-interrupt-stops-the-run-and-a-second-one-exits-at-once.md#decision-outcome) names each second interrupt that does not exit at once.
- **The test that the interrupt lands in has the [outcome](../../CONTEXT.md#outcome) `interrupted`.** The line names the test that ran, which is often the one that hangs. Its line prints in its place on stdout, after it unwinds. It has that outcome whether it then passes, fails or raises.
- **An [unreached test](../../CONTEXT.md#unreached-test) prints no line.** The run reached a blocked test, and never reached this one. The count names it. [A tester runs purba on one module, and reads one line for each test](a-tester-runs-purba-on-one-module-and-reads-one-line-for-each-test.md#decision-outcome) names this exception.
- **The count names `interrupted` after `unknown`, and `unreached` after `blocked`.** [A line names its kind in the glossary's words](a-line-names-its-kind-in-the-glossarys-words.md#decision-outcome) gives the order of the columns.

```
test_a.py::test_one passed
test_a.py::test_two passed
purba: interrupt: SIGINT; a second interrupt exits at once
test_a.py::test_three interrupted
5 tests: 2 passed, 1 interrupted, 2 unreached
```

- **After the count, purba dies by the first signal.** A shell sees 130 for SIGINT, 143 for SIGTERM and 129 for SIGHUP. The signal comes ahead of each code of [the exit-code record](a-run-exits-0-only-when-it-checked-something-and-every-test-passed.md#decision-outcome).
- **An interrupt after the last test still ends the run by the signal.** The count names each test by its outcome.
- **After an interrupt before selection finishes, purba prints no count.** It does not know the tests yet.
- **A second interrupt prints nothing more, and purba dies by that second signal.** The lines already written stay. When the parent's ignore or mask holds the second signal, purba dies by the first signal after Python finalizes, as [an interrupt stops the run, and a second one exits at once](an-interrupt-stops-the-run-and-a-second-one-exits-at-once.md#decision-outcome) says.
- **On Windows, purba exits `STATUS_CONTROL_C_EXIT` after a Ctrl-C or a Ctrl-Break.** The line names `Ctrl-C` or `Ctrl-Break`.

**Downside:**

- **Python's `subprocess` reports an interrupted run as minus the signal, such as -2, and not as 130.** An integrator that reads `returncode` sees a negative number.
- **The tester cannot read the name of an unreached test.** The count says how many.
- **A second interrupt leaves no count.**
- **An interrupt after the last test turns a run that passed into a failure.** A CI that cancels as the run ends fails it.

## Confirmation

`mise run test:wheel` checks that a first interrupt starts no later test, and that purba dies by SIGINT.
It checks that the test the interrupt lands in is `interrupted`, and that an interrupt in a load leaves each test unreached.
It checks that a signal handler of a test or a module lives only for its call or its load.
It checks that a second interrupt ends purba at once by that interrupt's signal, even while a cleanup runs C code.
It checks that SIGTERM and SIGHUP do what SIGINT does.
It checks that a Ctrl-C after the count ends Python's wait for threads, and lets a test's `atexit` function finish.
