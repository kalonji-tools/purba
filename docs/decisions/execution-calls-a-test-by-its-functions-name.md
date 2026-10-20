# Execution calls a test by its function's name, and reporting prints the identity it is given

## Context and Problem Statement

The invariant one identity says that no part names a test except by the identity that selection gives it.
[Each part is a crate](each-part-is-a-crate.md#decision-outcome), so a part uses a type only from a crate that its manifest names.
In the parts diagram, execution and reporting use no interface of selection.

| fact | source |
|---|---|
| reporting's use of `purba_selection::TestId`, with no selection in its manifest, stopped at `E0433` | [the probe](https://github.com/kalonji-tools/purba/issues/563#issuecomment-6291610911) |
| command's `TestId { path, function }`, with private fields, stopped at `E0451` | [the probe](https://github.com/kalonji-tools/purba/issues/563#issuecomment-6291610911) |
| reporting's test, with a `&str` in place of the identity, passed with no selection in its manifest | [the probe](https://github.com/kalonji-tools/purba/issues/563#issuecomment-6291610911) |
| selection uses parsing, and parsing grows the build on Linux from 14 to 80 packages | [the landscape](../../LANDSCAPE.md#the-parts-and-their-dependencies), [the findings of How can purba depend on ruff's Python parser?](https://github.com/kalonji-tools/purba/issues/538#issuecomment-6145006439) |
| dispatch hands command "three outcomes, each beside its test" | [the landscape](../../LANDSCAPE.md#the-parts-and-their-dependencies) |
| the gate for one identity misses "a part that formats its own name for a test" | [a gate refuses each invariant a tool can see](a-gate-refuses-each-invariant-a-tool-can-see.md#decision-outcome) |

## Considered Options

- **Execution and reporting use selection, as two new arrows of the parts diagram.** Rejected. Only selection constructs an identity, so a test of either part parses a real module to get one. Each of those tests then builds ruff.
- **The invariant binds dispatch alone.** Rejected. Reporting then builds `path::function` itself, the breach that the gate for one identity misses.
- **Execution and reporting never name the identity's type.** Chosen. Each part then tests without a parser.

## Decision Outcome

**Reach:** `crates/**` `/LANDSCAPE.md`

**Execution calls a test by its function's name, and reporting prints the identity it is given.**

| question | answer |
|---|---|
| what execution takes | a module, and the name of one function in it |
| what a module holds | its path as the tester typed it, its rewritten text, and the text of each assert |
| how execution finds the function | its worker loads each module once, keyed by the path, and looks the name up in that module |
| what an in-process worker is | a value of execution, which dispatch holds |
| what an outcome holds | no name of its test |
| who puts each outcome beside its test | dispatch, which uses selection and execution |
| what reporting takes | each outcome, beside a value that prints itself |
| how selection's identity prints | `path::function`, as [a tester runs purba on one module](a-tester-runs-purba-on-one-module-and-reads-one-line-for-each-test.md#decision-outcome) lays out a line |
| who constructs an identity | selection alone, because its fields are private |
| who builds a part's input | the part that uses it. Command builds dispatch's input, and dispatch builds execution's module |

[A worker loads each module once, and command hands each module once](a-worker-loads-each-module-once.md#decision-outcome) holds why a worker loads each module once.

**Downside:**

- **Reporting prints any value that prints itself.** Command can hand it a name that command formatted. The gate for one identity misses it. The reviewer looks for it.
- **Execution takes the name of one function.** A test method of a class needs more than one name. The design in depth of execution decides how such a test crosses.
- **A module under two spellings of its path loads twice in one worker.** The design in depth of selection decides how an identity starts.
- **A field of a module crosses from command to dispatch to execution.** A new field edits each of them.

## Confirmation

`mise run test:rust` checks that an identity prints as `path::function`.
The `compile_fail` doc tests on `TestId` check that no crate but selection constructs an identity or reaches one of its fields.

No command checks the rest yet.
[Add purba-execution and purba-fixtures, which run a test and make its outcome](https://github.com/kalonji-tools/purba/issues/571) builds the worker.
[Add purba-dispatch, which hands each test to one worker in-process](https://github.com/kalonji-tools/purba/issues/572) puts each outcome beside its test, and builds execution's module.
[Add purba-command, which turns the arguments into an exit code](https://github.com/kalonji-tools/purba/issues/574) builds dispatch's input.
[Add purba-reporting, which prints one line for each test and a count](https://github.com/kalonji-tools/purba/issues/573) builds the line.
