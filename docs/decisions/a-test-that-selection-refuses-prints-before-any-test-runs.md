# A test that selection refuses prints before any test runs

## Context and Problem Statement

[A test returns nothing, and the first part that sees a breach of a test refuses it](a-test-returns-nothing-and-the-first-part-that-sees-a-breach-refuses-it.md#decision-outcome) says which tests selection refuses.
[Reporting writes each line as dispatch hands it the test](reporting-writes-each-line-as-dispatch-hands-it-the-test.md#decision-outcome), and dispatch never takes a test that selection refuses.

| fact | source |
|---|---|
| `async def test_a` prints `unknown: no assert ran` and a `RuntimeWarning`, and `def test_c(x)` prints `raised: TypeError: …` | [the spec of the selection change](https://github.com/kalonji-tools/purba/issues/670#issuecomment-6467653807), on the wheel from `main` at `0484ef3` |

## Considered Options

- **Each refused line prints at its place among the lines of the selected tests.** Rejected. When a test above the refused test hangs or ends the process, the refused line never prints.
- **Each refused line prints before any test runs.** Chosen. Selection judges each test before any test runs.

## Decision Outcome

**Reach:** `crates/**` `tests/**` `/LANDSCAPE.md`

**A test that selection refuses prints before any test runs.**
The refused lines keep source order, and the lines of the selected tests follow them.

- **Selection judges the last `def` of a name, because import binds that one.** The line keeps the place of the first `def`, as for a selected test.
- **A refused line names every reason that selection sees.** A part names every error it finds, as [an error stops only the work that depends on it](an-error-stops-only-the-work-that-depends-on-it.md#decision-outcome) says. An `async def` with a `yield` names only that the test is async, because purba cannot call it either way.
- **A module does not load when selection refuses every test of it.**

**Downside:**

- **The lines do not follow the order of the file.** The line of a refused test prints above the line of a test that comes before it.
- **When selection refuses every test of a module, an error in its body does not show.** The module never loads.
- **Selection does not see a statement other than `def` that binds the name after its last `def`.** It judges the `def`, and execution calls what import bound.
- **Selection can refuse a test on a tree that ruff recovered from a module that does not parse.** [Does parsing ever hand a later part a tree that ruff recovered?](https://github.com/kalonji-tools/purba/issues/675) asks whether that can happen.

## Confirmation

`mise run test:rust` checks which tests selection refuses.
It also checks that their lines print before any test runs.
`mise run test:wheel` checks the lines and the exit code on the installed `purba`.
