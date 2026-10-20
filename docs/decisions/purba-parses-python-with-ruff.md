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

**Reach:** `/Cargo.toml` `crates/**` `/.cargo/deny.toml` `/.cargo/clippy.toml`

purba parses with four ruff crates behind a thin parsing seam.
`[workspace.dependencies]` pins each crate it names exactly at `=0.0.17`.

| crate | what the seam needs from it |
|---|---|
| `ruff_python_parser` | the parse entry point |
| `ruff_python_ast` | the AST the seam returns |
| `ruff_text_size` | the `Ranged` trait. `Expr::range()` does not resolve without it |
| `ruff_source_file` | `LineIndex`, so that a byte offset becomes a line and a column |

The manifest names them at the call site that first uses one, and not ahead of it.
Each ruff crate requires the others at its own version, so the pin also holds a crate that the manifest does not name yet.
Cargo reports most dependencies that nothing calls.
Nobody can suppress that report for one dependency alone.
Cargo misses some ruff crates that nothing calls, so the reviewer checks each one.

One module owns parsing and returns an AST, and everything else takes the AST.
[Command lends one parse to selection and assertion](command-lends-one-parse-to-selection-and-assertion.md) holds who asks for the AST, and when.
Holding that seam keeps even a late reversal a one-decision change.
The prescan path parses and raises nothing on a syntax error, because a file that fails to parse is a diagnostic and not a crash.
MSRV is the upgrade clock, not the API.

**A person raises the pin, when one of three triggers fires.**

- A supported CPython has syntax that the pinned parser refuses.
- purba needs a fix in the parser.
- The minimum Rust version requires it.

Each raise moves the four crates together.
It takes these steps:

1. Compare the surface that purba calls between the two versions, as the [pin check](https://github.com/kalonji-tools/purba/issues/538#issuecomment-6146303766) did.
2. Edit each licence exception in `.cargo/deny.toml` that `mise run lint:licences` names.
3. List again in `.cargo/clippy.toml` each function of `ruff_python_parser` that parses.
4. Add the new pin to the table below.

The version mapping belongs here, because the patch number is a release counter and not a change signal:

| pin | ruff release |
|---|---|
| `0.0.2` | 0.15.19 |
| `0.0.12` | 0.16.6 |
| `0.0.17` | 0.17.0 |

There is no `0.0.1`, and the mapping is not arithmetic.

**Downside:**

- **The publisher commits to nothing.** These crates first reached the registry eleven weeks before this decision, published so another project could consume them. No policy statement commits their publisher to a continuation. Confidence over the measured window is high and extrapolating is not.
- **MSRV moves about every six weeks**, three times in ten weeks, and each move is a pin somebody has to raise.
- **The dependency count is higher**, at 61 transitive against 45, and four direct crates to name against one.
- **Nothing notices a new ruff release.** A trigger reaches the person only when they meet it.

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

`/Cargo.toml` pins `ruff_python_parser` and `ruff_python_ast` at `=0.0.17`.
`purba-parsing` names both.
`mise run test:rust` runs the tests in `crates/parsing/tests/parse.rs`, which parse a module through them.
`mise run lint:licences` reads the exceptions in `.cargo/deny.toml`, and refuses one that no crate uses.

Cargo refused an unused `itoa` in `purba`.
It passed `ruff_text_size` and `ruff_source_file` in `purba-parsing`, when nothing there called either.

`.cargo_vcs_info.json` in the published crate gives the row for `0.0.17`, because it names the commit of ruff's tag `0.17.0`.
The `.cargo_vcs_info.json` of `0.0.12` gives its row in the same way.
A prescan turns the choice of parser into a fitness function, and counts parse failures against a corpus.
