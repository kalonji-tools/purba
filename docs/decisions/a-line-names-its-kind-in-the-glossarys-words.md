# A line names its kind in the glossary's words

## Context and Problem Statement

[A run exits 0 only when it checked something and every test passed](a-run-exits-0-only-when-it-checked-something-and-every-test-passed.md#decision-outcome) lets a test that raised and a project error share exit 4, because the line that purba prints names each kind.
On `main` at `09618c5`, a project error prints `error`, and an invocation error prints `purba: ` and its reason with no kind.
A panic prints only the text of Rust's panic hook, and no line for its test.
So a [tester](../../CONTEXT.md#tester) cannot tell an internal error from the other kinds by the line.

| fact | source |
|---|---|
| a run writes no data for an [integrator](../../CONTEXT.md#integrator) yet, and the integrator reads the exit code | [A tester runs purba on one module, and reads one line for each test](a-tester-runs-purba-on-one-module-and-reads-one-line-for-each-test.md#decision-outcome) |
| purba shows and counts `unknown`, and never shows it as clean | [purba is always strict](purba-is-always-strict.md#decision-outcome) |
| pytest names a status in its count only when a report holds it | [pytest's `terminal.py`](https://github.com/pytest-dev/pytest/blob/9.1.1/src/_pytest/terminal.py#L1461-L1467) |
| libtest names each status in its count, even at 0 | [libtest's `pretty.rs`](https://github.com/rust-lang/rust/blob/1.99.1/library/test/src/formatters/pretty.rs#L290-L293) |
| of 12 tools surveyed for how they print an error of their own, 8 name a tracker and 6 print their version | [Which line does each kind of failure print, so that a tester tells the kinds apart?](https://github.com/kalonji-tools/purba/issues/714#issuecomment-6521169909) |
| ruff asks for a report once, "instead of repeating it for each diagnostic", on stderr "to prevent the message from being captured by tools parsing the normal output" | [ruff's `lib.rs`](https://github.com/astral-sh/ruff/blob/0.17.3/crates/ruff/src/lib.rs#L482-L494) |

## Considered Options

- **A project error prints `error`, as on `main`.** Rejected. A line of an internal error holds ` error:` too, so a search for ` error:` finds both kinds.
- **The line of an error names its kind in the glossary's words.** Chosen. The name says where the tester fixes the error.
- **An internal error prints only to stderr, as a panic does on `main`.** Rejected. Its test loses its line, and the columns of the count no longer match the lines.
- **A line goes where its error stops.** Chosen. A test that an internal error stops keeps its line among the others.
- **The count names every column even at 0, and one column for every error, as on `main`.** Rejected. A column at 0 tells the tester nothing. A count that keeps its shape helps only a parser. One column cannot tell a bug in purba from an error in the tester's code.
- **The count names each column only above 0.** Chosen. When nothing follows `blocked`, no error happened.
- **The line of an internal error names the part and the message.** Chosen. The tester learns the kind and the test, and the hook gives the rest.
- **purba names nowhere to report, as on `main`.** Rejected. The tester must find out where a bug in purba goes.
- **purba asks for a report once, on stderr.** Chosen. stdout keeps only the lines and the count.
- **purba keeps Rust's own panic hook.** Chosen. It writes the location and the way to a backtrace, and purba writes none of its text.

## Decision Outcome

**Reach:** `crates/**` `tests/**`

**A line names its kind in the glossary's words.**

- **The line of an error names its kind:** [project error](../../CONTEXT.md#project-error), [internal error](../../CONTEXT.md#internal-error) or [invocation error](../../CONTEXT.md#invocation-error). A test's line names its [outcome](../../CONTEXT.md#outcome) or `blocked`, as before.
- **A line goes where its error stops.** An error that stops one test or one module prints to stdout, in the place of that test or module. An error that stops the run prints one line to stderr, after `purba: `.

```
<test> project error: <reason>
<path> project error: <reason>
<test> internal error: <part>: <message>
purba: invocation error: <reason>
purba: internal error: <part>: <message>
```

- **The line of an internal error names the part that found it, and the message.** It names no place in purba's source.
- **A run that met an internal error ends with two lines on stderr, once.** The first names a bug in purba, purba's version, the Python, the OS and the architecture. The second links the defect template. They follow the count, or the line that stopped the run.
- **purba keeps the text of Rust's own panic hook.** The hook writes the location of the panic, and how to get a backtrace, to stderr. [A panic inside a test gives that test an internal error](a-panic-inside-a-test-gives-that-test-an-internal-error.md#decision-outcome) wraps it.
- **The count always names the tests, and names each other column only above 0.** The columns keep their order: passed, failed, raised, unknown and blocked, then a semicolon, project errors and internal errors. An invocation error stops the run before the count.

```
3 tests: 3 passed
3 tests: 2 passed, 1 blocked; 1 internal error
0 tests; 1 project error
```

**Downside:**

- **Of the lines this record adds, purba prints only those of a project error and of an invocation error.**
- **Until [a panic inside a test prints that test's internal error, and tests/kinds.bats checks each kind](https://github.com/kalonji-tools/purba/issues/729), every error that execution hands prints as a project error.** So does an error from a defect of purba, as when dispatch hands a worker a test whose module did not load.
- **The count changes its shape from run to run.** A script that reads a column by its place breaks. An integrator reads the exit code until reporting decides a format.
- **Rust's panic hook speaks Rust.** A tester sees `thread '<unnamed>' panicked at` and a path in purba's source.
- **The defect template is written for a contributor.** A tester who reports a bug fills in a task.

## Confirmation

`mise run test:rust` checks the line of a project error.
`mise run test:wheel` checks the line of a project error and the line of an invocation error on the installed `purba`.
[A case of each kind checks that no failure prints as another kind](a-case-of-each-kind-checks-that-no-failure-prints-as-another-kind.md#decision-outcome) names its gate.
Each slice that builds the rest of what this record decides adds its check: [the count names each column only above 0, with project and internal errors apart](https://github.com/kalonji-tools/purba/issues/726), [a panic that stops the run exits 125 and asks for a report](https://github.com/kalonji-tools/purba/issues/728) and [a panic inside a test prints that test's internal error, and tests/kinds.bats checks each kind](https://github.com/kalonji-tools/purba/issues/729).

[A failure's kind is what the part that first sees it observes](a-failures-kind-is-what-the-part-that-first-sees-it-observes.md#decision-outcome) holds the kinds.
[An error stops only the work that depends on it](an-error-stops-only-the-work-that-depends-on-it.md#decision-outcome) says what each kind stops.
