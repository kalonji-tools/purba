# An error says its kind, and reporting takes five ways in

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
| a test that an interrupt leaves unreached has no line, and the count names it | [An interrupted run prints its count, and dies by the signal](an-interrupted-run-prints-its-count-and-dies-by-the-signal.md#decision-outcome) |

## Considered Options

- **Reporting reads `part()` as the kind, picks the counter of an internal error, and takes five ways in.** The rule that counts lives in two crates.

## Decision Outcome

**Reach:** `crates/**` `/LANDSCAPE.md`

**Area:** errors

**An error says its kind, and reporting takes five ways in.**
A new kind and the rule that counts it change in execution alone.

- **An error says its kind.** `error.kind()` gives `ErrorKind::Project`, or `ErrorKind::Internal` with the part that found it.
- **Any part builds a project error, through `Error::project`.** Command builds one for a test that selection refuses, for a module that does not parse, and for a fault of a module's suppression. It also builds one for each unused suppression.
- **Only execution builds an internal error.**
- **Execution's `Tally` counts each error by its kind.** Reporting picks no counter.
- **Reporting takes five ways in.** `finish` ends the report. The five ways in are these:
  - `case` takes a test with its outcome or its error, each [notice](../../CONTEXT.md#notice) of the test, and each verdict that a suppression turned into a warning. It writes the line, then each notice, then each warning that selection judged, then each warning that execution judged. A notice and a warning count no test.
  - `error` takes a module or a test, with its error. It counts no blocked test.
  - `stopped` takes a module that stopped before its tests ran, with its error, each thread's error, and its tests. It writes the error once, then each thread's error, then each test as blocked.
  - `ended` takes a module whose process ended after one of its tests ran. It also takes the error of that end, a flag that says whether that test's line named it, and each test left. It writes the error once, unless that line named it, then each test as blocked by the end.
  - `unreached` takes an [unreached test](../../CONTEXT.md#unreached-test). It writes no line.
- **Reporting's tests reach it through the five ways in, `finish`, and its private writers.** An outcome goes through the private `add`. An error, a notice and a warning go through the private `line`, `noticed` and `warned`.

**Downside:**

- **Command names a type of execution to build a project error.** The arrow from command to execution carries it.
- **Reporting keeps private ways in for its own tests.** `line`, which `case` calls, maps an outcome to its kind. Only command's tests run that map.
- **Command copies each warning that selection judged into execution's shape.** Reporting names no type of selection, so command builds an execution `Warning` from each.

## Confirmation

`mise run test:rust` checks that a tally counts an error of each kind, for a test and for a module.
It checks that an error says its kind.
It checks that reporting prints a project error and an internal error through `line` and through `stopped`, and a project error through `error`.
It checks that `stopped` writes the module's error, then each thread's error, then each test as blocked, and counts the module's error once.

`mise run test:rust` also checks that `ended` writes the error once, unless the line of a case named it, then each test left as blocked.
It checks that `unreached` writes no line, and that the count names `unreached`.
It checks that `case` writes the line, then each notice, then each warning that selection judged, then each warning that execution judged.

`mise run lint:wiring` checks that the manifest of command names execution along an arrow of the parts diagram.
