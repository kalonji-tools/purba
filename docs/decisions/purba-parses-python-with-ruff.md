# purba parses Python with ruff's parser

## Context and Problem Statement

purba must parse Python source to find tests.
The parser is load-bearing, because a file that does not parse degrades to "no tests here". That is a wrong answer that looks like a correct one.

The choice was framed as maintained-but-unstable against stable-but-stale.
Neither half survives the sources.

This is also the cheapest moment to decide:

| decide | cost | reversible |
|---|---|---|
| now | one dependency line | yes, both directions |
| after a prescan exists | every statement and expression arm rewrites, 1 to 2 weeks | expensive, asymmetric |
| after a query language ships | AST facts have become a public contract | effectively one-way |

## Considered Options

- **`rustpython-parser`.** Rejected. It is not stale, it is abandoned, and it says so in three independent places. Its README now reads "superseded by". Its maintainer stated the repository will not be maintained and confirmed this covers the crate. The interpreter it was built for migrated onto a fork of ruff's parser and keeps no reference to it. The consequences are present rather than prospective: it declares no grammar version anywhere and fails outright on 7 of 774 CPython 3.14 source files.
- **`ruff_python_parser`, with its AST, text-size and source-file companions.** Chosen. Its API is not meaningfully unstable for this use. Across six published versions over ten weeks the whole surface a prescan touches is byte-identical. That surface is the parse entry point, its options and result types, all 25 statement and 33 expression variants, and every visitor trait. The churn sits in peripheral helpers.

Anyone re-checking the rejected crate will find a repository whose metadata says it is not archived, with a recent commit.
The evidence is in its issues and its README, not in repository metadata.

## Decision Outcome

purba parses with four ruff crates, pinned exactly at `=0.0.12`, behind a thin parsing seam.

| crate | what the seam needs from it |
|---|---|
| `ruff_python_parser` | the parse entry point |
| `ruff_python_ast` | the AST the seam returns |
| `ruff_text_size` | the `Ranged` trait. `Expr::range()` does not resolve without it |
| `ruff_source_file` | `LineIndex`, so that a byte offset becomes a line and a column |

The manifest names them at the call site that first uses one, and not ahead of it.
A dependency nothing calls is reported, and that report has nowhere to be suppressed one dependency at a time.

One module owns parsing and returns an AST, and everything else takes the AST.
Holding that seam keeps even a late reversal a one-decision change.
The prescan path parses and raises nothing on a syntax error, because a file that fails to parse is a diagnostic and not a crash.
MSRV is the upgrade clock, not the API.

The version mapping belongs here, because the patch number is a release counter and not a change signal:

| pin | ruff release |
|---|---|
| `0.0.2` | 0.15.19 |
| `0.0.12` | 0.16.6 |

There is no `0.0.1`, and the mapping is not arithmetic.

**Downside:**

- **The publisher commits to nothing.** These crates first reached the registry eleven weeks before this decision, published so another project could consume them. No policy statement commits their publisher to a continuation. Confidence over the measured window is high and extrapolating is not.
- **MSRV moves about every six weeks**, three times in ten weeks, and each move is a pin somebody has to raise.
- **The dependency count is higher**, at 61 transitive against 45, and four direct crates to name against one.

Being wrong here is survivable, and being wrong the other way is not:

| | the failure | loud | deferrable | bounded |
|---|---|---|---|---|
| ruff | an API break | yes, a compile error | yes, the pin is exact | yes, 6 symbols in 10 weeks |
| the rejected crate | a file does not parse | no | no, already true | no, nobody maintains it |

[One manifest declares each package](one-manifest-declares-each-package.md) names the file that holds the toolchain and [purba meets the next trait solver before it stabilizes](purba-meets-the-next-trait-solver-before-it-stabilizes.md) names the channel.
A raised MSRV is read against those, and not against a pin in this repository.

## Confirmation

The pin is exact.
Cargo treats a `0.0.x` requirement as `>=0.0.x, <0.0.(x+1)`, verified by an update that did not move a pinned requirement, so the forced upgrade cadence is none.
`=` therefore states that intent and does not change it.

This Confirmation is weak, and it is stated weakly on purpose.
`Cargo.toml` names no ruff crate.
Nothing in the tree exercises the choice.
No pin exists to move.
This record fixes the choice and the version.

The version mapping above is evidence, frozen at this decision.
Whoever first raises a pin reads it against `cargo tree`.
No issue owns that reading, because a check has nothing to run against until a pin exists.
The commit that first names a ruff crate carries the declaration and that check together.
A prescan turns this into a fitness function, and counts parse failures against a corpus.
