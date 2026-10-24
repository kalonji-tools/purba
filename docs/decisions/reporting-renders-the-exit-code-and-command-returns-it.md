# Reporting renders the exit code, and command returns it

## Context and Problem Statement

A [run](../../CONTEXT.md#run) ends in an exit code, and in a count line for the [tester](../../CONTEXT.md#tester).
[A run exits 0 only when it checked something and every test passed](a-run-exits-0-only-when-it-checked-something-and-every-test-passed.md#decision-outcome) gives each code of a finished run.
Each of those codes asks whether a kind of [outcome](../../CONTEXT.md#outcome) or an error is present, and never asks how many there are.
So the code names the worst of the run: failed, then an error, then `unknown` or no test, then passed.
Some part counts each outcome and each error, and some part turns that count into the code.

| fact | source |
|---|---|
| command starts a run, and reporting renders each outcome as data for an [integrator](../../CONTEXT.md#integrator) | [the landscape](../../LANDSCAPE.md#the-parts-and-their-dependencies) |
| purba hands the integrator an exit code and data | [the landscape](../../LANDSCAPE.md#the-context) |
| the parts diagram draws no arrow from command to execution, or from reporting to dispatch | [the landscape](../../LANDSCAPE.md#the-parts-and-their-dependencies) |
| a crate that names `purba_execution::Kind`, with only dispatch in its manifest, does not compile: `E0433` | [the spec of dispatch](https://github.com/kalonji-tools/purba/issues/572#issuecomment-6316969184) |

## Considered Options

- **Reporting returns the tally, and command maps it to the code.** Rejected. Command then renders the outcomes for an integrator. The landscape says that reporting renders each outcome for an integrator.
- **The parts diagram gains an arrow from command to execution, and command counts.** Rejected. Command then names the kind of an outcome.
- **Dispatch counts while it hands each test.** Rejected. Reporting cannot name a type of dispatch, so it would count again for its count line.
- **Execution holds the rule that counts, reporting renders the code, and command returns it.** Chosen. Execution defines each kind, so a new kind and the rule that counts it change in one crate. Command names no type of execution.

## Decision Outcome

**Reach:** `crates/**` `/LANDSCAPE.md`

**Reporting renders the exit code, and command returns it.**

| question | answer |
|---|---|
| which part holds the rule that counts | execution: `Tally`, and `Tally::add` for each test |
| which part renders the count line and the exit code | reporting, from the tally that it adds while it prints each line |
| what `purba_reporting::print` returns | the exit code: 0, 1, 2 or 4. When stdout refuses a write, `print` returns the error in place of the code |
| what command does with the code | it returns the code to `_main`. A refused write is an error, so `_main` returns 3 |
| where it is drawn | the sequence of one run in [the landscape](../../LANDSCAPE.md#the-parts-and-their-dependencies), as `reporting -->> command: exit code 1` |

**Downside:**

- **The table of the exit code lives in two places.** The exit-code record holds it, and reporting repeats it in `exit_code`. A change to one must change the other.

## Confirmation

`mise run test:rust` checks that a tally adds each kind and each error.
It checks that the exit code names the worst of the run.
It checks that the count line matches the lines that reporting prints.
`mise run lint:wiring` checks that reporting answers command back along the arrow from command to reporting.

No command checks the rest yet.
[Add purba-command, which turns the arguments into an exit code](https://github.com/kalonji-tools/purba/issues/574) returns the code to `_main`.
