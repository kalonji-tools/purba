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

A normaliser's style was weighed separately.

- **Tabs, so that each reader chooses the width.** It is the only indentation that adapts to its reader, and GitHub carries a per-account setting for it. Rejected on consistency: nothing in the tree is tab-indented, and YAML forbids tabs outright, so the file a script's reader arrives from cannot match it.
- **Two spaces, stated once for every language that admits the same value.** Chosen.

## Decision Outcome

**A gate owns every standard a reviewer would otherwise state by hand.**
The rule takes two clauses, because a formatter has no severity to set.

A tool that **detects** runs at the strictest setting it offers, with every optional check enabled.
A check is dropped only by naming it together with the reason it is dropped, at the location a reader meets the code.

A tool that **normalises** has one location for its style, and tolerates no deviation from it.
**The style is chosen for the human who reads the code**, because a machine reads any style the syntax admits.
Consistency across languages is part of that legibility: a reader should not change indentation systems at a file boundary.

⚠️ **A language whose own rules make a different value load-bearing is stated per language, with the constraint as the reason.**
Markdown is that case: a table row is one line by its syntax, so no line length is stated for it.

**A standard nothing can fix is still stated, and then it is only checked.**
Line length is that standard, because no formatter wraps a line.
Where a fixer exists it writes, and a checker is not put in its place.

⚠️ **A setting chosen because the tree already passes there is refused.**
It reports the tree's current state as though that were the standard, and the two are indistinguishable afterwards.

**A configuration file is owed only where purba deviates from a tool's default.**
[mise names every tool version](mise-names-every-tool-version.md) pins each one, so an accepted default already has a single location: the pinned tool.
Writing a default into a file makes a second copy, which drifts without saying so.
Absence is ambiguous, so this record names what runs unconfigured: `cargo fmt` and `typos` deviate in nothing.
`shfmt` does deviate, so the style is written down, and `.editorconfig` is where it lives because `shfmt` and every editor both read it.
⚠️ **A style flag would take that away:** `shfmt` ignores `.editorconfig` the moment one is passed, so the hook passes none.

Every lint level lives in `Cargo.toml`, which a task, a bare `cargo clippy` and an editor all read.
A flag on a command line reaches only that command, so `tasks.toml` carries no lint flag.

**Downside:** the strictest setting of a tool purba has not adopted is not derivable from this record, so every adoption costs its own measurement.
An exclusion is also somewhere a later contributor can widen quietly, and the reason above it is the only thing that makes widening visible.

An exclusion is also not always as narrow as its line.
Inside a YAML block scalar none can be: an indented one is posted as part of whatever the block writes, and one at the first column ends the block.
The tree holds no such exclusion, because the shell that needed one moved into a file where a line-scoped directive works.

⚠️ **What is generated is named in two files that cannot see each other.**
`.gitattributes` marks it for review and the style file excludes it from the standard, so a generated file added to one and not the other gains a check it will fail.

## Confirmation

Each gate is run green and red, because a check that passes on a clean tree and refuses nothing is a suggestion.

| gate | what it refuses |
|---|---|
| `clippy` | `cargo clippy --all-targets`, with no flag on the command line: a `pub fn` returning a value draws `must_use_candidate`, and the exit is non-zero |
| `rustdoc` | a broken intra-doc link in the crate-level block draws `unresolved link`, and the exit is non-zero |
| `cargo` | re-declaring a crate nothing calls draws `unused dependency`, which `-D warnings` never reached |

The normalising clause was exercised on purba's shell against the pinned tools, measured in [Should shellcheck and shfmt gate purba's shell, and at what severity and style?](https://github.com/kalonji-tools/purba/issues/186).

| gate | green | red |
|---|---|---|
| `shellcheck` at `enable=all` | the repaired tree reports nothing | a bare `$var` reinstated anywhere draws `SC2250` while `.shellcheckrc` is present, and nothing without it. Each suppression removed restores its own finding |
| `shfmt` reading `.editorconfig` | the tree needs no rewrite | a style demanding tabs rewrites every script |
| `editorconfig-checker` | the tree reports nothing | every exclusion removed restores **three** refusals, one per excluded line, and a style demanding tabs draws **seventy-two** |

⚠️ **One of the linter's own fixes was measured changing behaviour**, which is why only the normaliser writes.
`[[ ]]` evaluates `5+5` arithmetically where `[` refuses a value that is not an integer, so a guard on an issue number would have stopped refusing one.
