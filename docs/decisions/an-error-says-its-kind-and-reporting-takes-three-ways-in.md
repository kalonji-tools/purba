# An error says its kind, and reporting takes three ways in

## Context and Problem Statement

An error that execution hands to reporting is a project error or an internal error.
[Reporting renders the exit code, and command returns it](reporting-renders-the-exit-code-and-command-returns-it.md#decision-outcome) puts the rule that counts in execution.
On `main` at `bc83b7a`, reporting decoded the kind and counted an internal error itself.

| fact | source |
|---|---|
| `Tally::add` counts any error as one blocked test and one project error | [`execution`](https://github.com/kalonji-tools/purba/blob/bc83b7a/crates/execution/src/lib.rs#L323-L337) |
| reporting counts an internal error itself, and picks the counter of a module | [`reporting`](https://github.com/kalonji-tools/purba/blob/bc83b7a/crates/reporting/src/lib.rs#L99-L126) |
| reporting and the tests of dispatch and execution read `part()` as the kind | [`reporting`](https://github.com/kalonji-tools/purba/blob/bc83b7a/crates/reporting/src/lib.rs#L17-L27), [`dispatch`'s tests](https://github.com/kalonji-tools/purba/blob/bc83b7a/crates/dispatch/tests/hand.rs#L32-L38), [`execution`'s tests](https://github.com/kalonji-tools/purba/blob/bc83b7a/crates/execution/tests/run.rs#L294) |
| reporting has five ways in, and no test calls three of them | [Execution counts each error by its kind, and command's tests check only command's rules](https://github.com/kalonji-tools/purba/issues/748) |
| an outcome has a private field and no public constructor. Outside execution, only its `Deserialize` builds one | [`execution`](https://github.com/kalonji-tools/purba/blob/bc83b7a/crates/execution/src/lib.rs#L35-L50) |

## Considered Options

- **Reporting reads `part()` as the kind, picks the counter of an internal error, and takes five ways in.** Rejected. The rule that counts lives in two crates.
- **The error says its kind, execution counts it, and reporting takes three ways in.** Chosen. A new kind and the rule that counts it change in execution alone.

## Decision Outcome

**Reach:** `crates/**` `/LANDSCAPE.md`

**An error says its kind, and reporting takes three ways in.**

- **An error says its kind.** `error.kind()` gives `ErrorKind::Project`, or `ErrorKind::Internal` with the part that found it.
- **Any part builds a project error, through `Error::project`.** Command builds one for a test that selection refuses, and for a module that does not parse.
- **Only execution builds an internal error.**
- **Execution's `Tally` counts each error by its kind.** Reporting picks no counter.
- **Reporting takes three ways in, and `finish` ends the report.** The three ways in are these:
  - `line` takes a test with its outcome or its error.
  - `module` takes a module with its error.
  - `blocked` takes a test that its module blocked.
- **Reporting's tests reach it through the three ways in and `finish`.** An outcome goes through the private `add`.

**Downside:**

- **Command names a type of execution to build a project error.** The arrow from command to execution carries it.
- **Reporting keeps one private way in for its own tests.** `line` maps an outcome to its kind, and only command's tests run that map.

## Confirmation

`mise run test:rust` checks that a tally counts an error of each kind, for a test and for a module.
It checks that an error says its kind.
It checks that reporting prints a project error and an internal error through `line` and through `module`.
`mise run lint:wiring` checks that the manifest of command names execution along an arrow of the parts diagram.
