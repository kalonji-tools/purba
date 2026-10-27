# Reporting writes each line as dispatch hands it the test

## Context and Problem Statement

Dispatch put every outcome and error in a list, and reporting printed the lines after the last test.
A crash inside a test then left the tester no line, though the tests before it ran.
[Wayfinder: purba's errors, telling each kind of failure apart from the first part that sees it](https://github.com/kalonji-tools/purba/issues/652) ends with a panic inside a test, after which every other test keeps its line.

| fact | source |
|---|---|
| a test that aborts the process as test 3 of 5 leaves no line, and leaves the lines of tests 1 and 2 when reporting flushes each line | [the prototype](https://github.com/kalonji-tools/purba/blob/e381145a82127aa06dcd70c8d9d8f80f44d89d7f/proto-664/RESULTS.md) |
| a sink trait in dispatch builds and passes `lint:wiring` when dispatch re-exports execution's types | [the prototype](https://github.com/kalonji-tools/purba/blob/e381145a82127aa06dcd70c8d9d8f80f44d89d7f/proto-664/RESULTS.md) |
| `purba … \| head -1` with lines printed after the last test, in 20 runs: 200 tests of 10 ms exit 0 each time, 200 fast tests exit 0 in 19, and 3000 tests exit 3 each time | [the prototype](https://github.com/kalonji-tools/purba/blob/e381145a82127aa06dcd70c8d9d8f80f44d89d7f/proto-664/RESULTS.md) |
| `purba … \| head -1` with each line flushed, in 20 runs of each suite: it stops after 2 or 3 tests, and exits 3 each time | [the prototype](https://github.com/kalonji-tools/purba/blob/e381145a82127aa06dcd70c8d9d8f80f44d89d7f/proto-664/RESULTS.md) |
| six of seven surveyed test runners stop at once when stdout refuses a write, and none ignores the write | [the research of a refused write](https://github.com/kalonji-tools/purba/blob/b2b33df88e0a5bd02f138b6132fc33379b58bdce/research/refused-write.md) |

## Considered Options

- **Dispatch returns every result, and reporting prints after the last test.** Rejected. A crash inside a test leaves no line for any test.
- **A sink trait that dispatch declares.** Rejected. Command must name execution's types where it implements the trait. When command names them through a re-export of dispatch, no gate sees that use.
- **A function that dispatch calls once for each test.** Chosen. Command names no type of execution, and the parts diagram draws each call.

## Decision Outcome

**Reach:** `crates/**` `/LANDSCAPE.md`

**Reporting writes each line as dispatch hands it the test.**

- **Dispatch calls a function that command gives it, once for each test, as the worker returns the test's outcome or error.** The function returns `ControlFlow`, and no test runs after a `Break`.
- **Parsing and selection keep returning what they find, and command hands it on at once.**
- **Reporting writes and flushes each line as it takes it, and keeps only the tally until the count.**
- **A refused write stops the run at once, and purba exits 3.** [The record of the kinds](a-failures-kind-is-what-the-part-that-first-sees-it-observes.md#decision-outcome) names a refused write an invocation error, and [an error stops only the work that depends on it](an-error-stops-only-the-work-that-depends-on-it.md#decision-outcome) stops the run for one.

**Downside:**

- **The last test that runs before a refused write changes from run to run.** It depends on when the program that reads the pipe quits.
- **`purba … | head -1` exits 3.** When purba printed every line after the last test, a short output mostly exited 0, because no write came after `head` quit.

## Confirmation

`mise run test:rust` checks that dispatch hands each test on before it runs the next, and calls no test after a `Break`.
It checks that reporting flushes each line before the next test, and returns the error of a refused write.
It checks that command runs no test after a refused write.
`mise run test:wheel` checks that a test that aborts purba leaves the line of each test before it.
`mise run lint:wiring` checks that each call of the hand-off that the sequence of one run draws goes along an arrow.

[A module that did not load prints its error once for each of its tests](https://github.com/kalonji-tools/purba/issues/681) builds the lines of its stopped tests on this hand-off.
