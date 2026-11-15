# A process that a test forks has the signal handlers it has without purba

## Context and Problem Statement

[An interrupt stops the run, and a second one exits at once](an-interrupt-stops-the-run-and-a-second-one-exits-at-once.md#decision-outcome) gives each interrupt signal purba's signal handler.
No record says what a process that a test forks gets for a signal.

| fact | source |
|---|---|
| a process that a test forks inherits purba's signal handler | [A process that a test forks keeps purba's interrupt handling, and prints a second report](https://github.com/kalonji-tools/purba/issues/846) |
| on SIGTERM, that process prints `interrupted` and a second count, and stderr holds purba's interrupt line | the same |
| a process that `multiprocessing` forks raises `KeyboardInterrupt` when the test calls `terminate()`, so a test that asserts `exitcode == -signal.SIGTERM` fails | the same |
| a function that `os.register_at_fork` takes as `after_in_child` runs in the child of `os.fork`; a fork from C code skips it, unless that code calls `PyOS_AfterFork_Child` | [CPython's `os.rst`](https://github.com/python/cpython/blob/v3.15.2/Doc/library/os.rst?plain=1#L5426-L5450) |
| the guard that takes the place of `os._exit` tells a process that a test forks from purba by the process ID | [`execution/src/lib.rs`](https://github.com/kalonji-tools/purba/blob/8d5712a/crates/execution/src/lib.rs#L218) |

## Considered Options

- **A process that a test forks inherits purba's signal handler, as today.** A SIGTERM to it raises `KeyboardInterrupt`, and it prints a second count.
- **purba's signal handler compares the process ID with purba's.** The child keeps purba's signal handler, and `signal.getsignal` names it there. That handler acts as the one that each signal had before purba gave it purba's.

## Decision Outcome

**Reach:** `crates/**` `tests/**`

**Area:** interrupt handling

**A process that a test forks has the signal handlers it has without purba.**
In the child, `signal.getsignal` names the signal handler itself, and no handler of purba's acts in its place.

- **In the child of `os.fork`, each interrupt signal gets back the signal handler it had before purba gave it purba's.** A function that purba registers with `os.register_at_fork` gives it back.
- **A signal handler that the test set for a signal before the fork stays in the child.** The test asked for it.
- **The [zygote](../../CONTEXT.md#zygote) and each test's process keep purba's signal handlers.** purba forks them itself. Its at-fork function gives nothing back in a fork of purba's own.

**Downside:**

- **A process that C code forks without `PyOS_AfterFork_Child` keeps purba's signal handler.** Python runs no at-fork function in it. A SIGTERM during the test's call raises `KeyboardInterrupt` in it, in place of its default action. It ignores each later interrupt signal, so a second SIGTERM cannot end it.
- **A signal that had a signal handler from C code when Python started keeps purba's signal handler in the child.** `signal.getsignal` names no signal handler for it, so purba has none to give back. The child then ignores each interrupt signal after its first.

## Confirmation

`mise run test:wheel` checks that a process that a test forks dies by SIGTERM and prints no count of its own.
It checks that a process that `multiprocessing` forks dies by SIGTERM on `terminate()`.
It checks that the signal handlers of a process that a test forks equal the ones that CPython prints without purba: `default_int_handler`, then `SIG_DFL` twice.
It checks that a signal handler that a test sets for SIGTERM before it forks runs in the child.
It checks that a Ctrl-C makes the test that it lands in `interrupted`. That outcome shows that each test's process keeps purba's signal handler.
