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

- **A project error prints `error`, as on `main` at `09618c5`.** A line of an internal error holds ` error:` too, so a search for ` error:` finds both kinds.
- **An internal error prints only to stderr, as a panic inside a test does on `main` at `09618c5`.** Its test loses its line, so the columns of the count no longer match the lines.
- **The count names every column even at 0, and one column for every error, as on `main` at `09618c5`.** A column at 0 tells the tester nothing. A count that keeps its shape helps only a parser. One column cannot tell a bug in purba from an error in the tester's code.
- **purba names nowhere to report, as on `main` at `09618c5`.** The tester must find out where a bug in purba goes.
- **purba names the Python by its version alone.** A free-threaded build then reads as the build with a GIL.

## Decision Outcome

**Reach:** `crates/**` `tests/**`

**Area:** errors

**A line names its kind in the glossary's words.**

- **The line of an error names its kind:** [project error](../../CONTEXT.md#project-error), [internal error](../../CONTEXT.md#internal-error) or [invocation error](../../CONTEXT.md#invocation-error). The name says where the tester fixes the error. A test's line names its [outcome](../../CONTEXT.md#outcome) or `blocked`, as before.
- **A line goes where its error stops.** A test that an internal error stops keeps its line among the others.

  ```mermaid
  flowchart LR
    error[an error stops] -->|one test or one module| place[one line on stdout, in the place of that test or module]
    place -->|a module| blocked[then each of its selected tests, as blocked]
    error -->|the run| stderr["one line on stderr, after purba: "]
    error -->|nothing| own[a line of its own]
  ```

  The line names the test or the module that the error charges.

```
<test> project error: <reason>
<path> project error: <reason>
<test> internal error: <part>: <message>
<path> internal error: <part>: <message>
purba: invocation error: <reason>
purba: internal error: <part>: <message>
```

- **The line of an internal error names the part that found it, and the message.** The line names no place in purba's source. The tester learns the kind and the test, and Rust's panic hook gives the rest.
- **A module that an internal error stops prints its line once, and each of its selected tests as blocked.** It takes the shape of a module's project error.
- **A run that met an internal error ends with two lines on stderr, once.** The first names a bug in purba, purba's version, the Python, the OS and the architecture. The Python carries its ABI flags, so a free-threaded 3.14.0 reads `3.14.0t`. A tester's report then names the build that met the error. The second links the defect template. They follow the count, or the line that stopped the run. stdout keeps only the lines that name a test or a module, and the count.
- **purba keeps the text of Rust's own panic hook.** The hook writes the location of the panic, and how to get a backtrace, to stderr. purba writes none of that text. [A panic inside a test gives that test an internal error](a-panic-inside-a-test-gives-that-test-an-internal-error.md#decision-outcome) wraps the hook.
- **The count always names the tests, and names each other column only above 0.** When no semicolon follows the tests, no error happened. The columns keep their order: passed, failed, raised, unknown, interrupted, blocked and unreached, then a semicolon, project errors and internal errors. A comma parts two columns on one side of the semicolon. An invocation error stops the run before the count.

```
3 tests: 3 passed
3 tests: 2 passed, 1 blocked; 1 internal error
0 tests; 1 project error
```

**Downside:**

- **The count changes its shape from run to run.** A script that reads a column by its place breaks. An integrator reads the exit code until reporting decides a format.
- **Rust's panic hook speaks Rust.** A tester sees `thread '<unnamed>' panicked at` and a path in purba's source.
- **The defect template is written for a contributor.** A tester who reports a bug fills in a task.

## Confirmation

`mise run test:rust` checks the line of a project error.
`mise run test:wheel` checks the line of a project error and the line of an invocation error on the installed `purba`.
`mise run test:rust` checks that the count names each column only above 0.
`mise run test:rust` checks that the count names project errors apart from internal errors.
`mise run test:wheel` checks the count on the installed `purba`.

On the `fault` wheel, `mise run test:wheel` checks the line of an internal error that stops the run, and the two lines after that line.
`mise run test:rust` checks the line of an internal error for a test and for a module.
On the `fault` wheel, `mise run test:wheel` checks the line of an internal error inside a test, and the line of each test after it.

[A case of each kind checks that no failure prints as another kind](a-case-of-each-kind-checks-that-no-failure-prints-as-another-kind.md#decision-outcome) names its gate.

[A failure's kind is what the part that first sees it observes](a-failures-kind-is-what-the-part-that-first-sees-it-observes.md#decision-outcome) holds the kinds.
[An error stops only the work that depends on it](an-error-stops-only-the-work-that-depends-on-it.md#decision-outcome) says what each kind stops.
