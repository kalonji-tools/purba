# A gate owns the mechanical standard

## Context and Problem Statement

purba welcomes a contribution written by an agent.
An agent cannot be assumed to have read a convention, and a reviewer who states one by hand states it again next time.

So a convention no tool enforces is not a convention.
It is a review comment, rewritten forever, by the reader whose attention is scarcest.
That reader's work is judging whether a change solves its problem, and everything mechanical competes with it.

[What does always-strict mean, and which checks can exist under it?](https://github.com/kalonji-tools/purba/issues/32) answered the product half: purba is strict for the suites it runs, and strict is not a dial.
It also drew the line this record needs — a defensible house rule for purba's own tests is a strong claim to make about someone else's.
This record answers the house half, which nothing answered: how purba configures the tools that read purba.

## Considered Options

Three, weighed against the published configuration of eighteen projects and against purba's own tree.

- **The tool's default severity.** Sixteen of the eighteen run their linter at its default, and four enable more. Rejected because it is not the strictest available, not because it is unusual.
- **A severity chosen where the tree is already clean.** Rejected. One of the eighteen documents the practice, and its reason is a backlog too large to repair now; purba's was four findings against nineteen lines of Rust. ⚠️ **The practice also fails on its own terms.** A floor at `warning` keeps `SC1090`, which is merely noisy, and drops `SC2102`, which is the class that put an unquoted glob into a workflow.
- **The strictest setting the tool offers, with every exclusion named.** Chosen.

## Decision Outcome

**A gate owns every standard a reviewer would otherwise state by hand.**
The rule takes two clauses, because a formatter has no severity to set.

A tool that **detects** runs at the strictest setting it offers, with every optional check enabled.
A check is dropped only by naming it together with the reason it is dropped, at the location a reader meets the code.

A tool that **normalises** has no severity, so its clause differs: the style has exactly one location, and the check tolerates no deviation from it.

⚠️ **A setting chosen because the tree already passes there is refused.**
It reports the tree's current state as though that were the standard, and the two are indistinguishable afterwards.

**A configuration file is owed only where purba deviates from a tool's default.**
[mise names every tool version](mise-names-every-tool-version.md) pins each one, so an accepted default already has a single location: the pinned tool.
Writing a default into a file makes a second copy, which drifts without saying so.
Absence is ambiguous, so this record names what runs unconfigured: `cargo fmt` and `typos` deviate in nothing.

Every lint level lives in `Cargo.toml`, which a task, a bare `cargo clippy` and an editor all read.
A flag on a command line reaches only that command, so `tasks.toml` carries no lint flag.

**Downside:** the strictest setting of a tool purba has not adopted is not derivable from this record, so every adoption costs its own measurement.
An exclusion is also somewhere a later contributor can widen quietly, and the reason above it is the only thing that makes widening visible.

## Confirmation

Each gate is run green and red, because a check that passes on a clean tree and refuses nothing is a suggestion.

| gate | what it refuses |
|---|---|
| `clippy` | `cargo clippy --all-targets`, with no flag on the command line: a `pub fn` returning a value draws `must_use_candidate`, and the exit is non-zero |
| `rustdoc` | a broken intra-doc link in the crate-level block draws `unresolved link`, and the exit is non-zero |
| `cargo` | re-declaring a crate nothing calls draws `unused dependency`, which `-D warnings` never reached |

⚠️ **The normalising clause has no application in the tree.**
Every tool purba runs that normalises accepts its default, so nothing here exercises the second clause.
[Should shellcheck and shfmt gate purba's shell, and at what severity and style?](https://github.com/kalonji-tools/purba/issues/186) is its first exercise, and it waits on this record rather than landing beside it.
