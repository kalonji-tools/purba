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

| fact | source |
|---|---|
| `ruff_python_parser` `0.0.18` and `0.0.19` are published from ruff's tags `0.17.1` and `0.17.2` | [the probe](https://github.com/kalonji-tools/purba/issues/1144#issuecomment-7064632646) |
| at `0.17.1`, `ruff_python_parser` names `stacker` as a plain dependency | [`Cargo.toml` L29 at `0.17.1`](https://github.com/astral-sh/ruff/blob/0.17.1/crates/ruff_python_parser/Cargo.toml#L29) |
| at `0.17.2`, `ruff_python_parser` names `stacker` as an optional dependency, behind a feature `stacker`, and its `[features]` names no `default` | [`Cargo.toml` L29 and L51-L52 at `0.17.2`](https://github.com/astral-sh/ruff/blob/0.17.2/crates/ruff_python_parser/Cargo.toml#L29-L52) |
| with the feature `stacker`, ruff's parser grows the stack through `stacker::maybe_grow` | [`parser/mod.rs` L950-L961 at `0.17.6`](https://github.com/astral-sh/ruff/blob/0.17.6/crates/ruff_python_parser/src/parser/mod.rs#L950-L961) |
| at `0.0.23` without `stacker`, the test of `test_deep_broken.py` in `crates/parsing/tests/parse.rs` aborts with a stack overflow, and with `stacker` it passes | [the probe](https://github.com/kalonji-tools/purba/issues/1144#issuecomment-7064632646) |
| the `.cargo_vcs_info.json` of `0.0.2`, `0.0.12`, `0.0.17` and `0.0.23` names the commit of ruff's tag `0.15.19`, `0.16.6`, `0.17.0` and `0.17.6` | [the probe of the tags](https://github.com/kalonji-tools/purba/issues/1144#issuecomment-7066152058) |
| ruff counts a tab to the next multiple of 2 columns | [`indentation.rs` L38 and L61-L69 at `0.17.6`](https://github.com/astral-sh/ruff/blob/0.17.6/crates/ruff_python_parser/src/lexer/indentation.rs#L38-L69) |
| CPython counts a tab to the next multiple of 8 columns | [`state.c` L9](https://github.com/python/cpython/blob/v3.14.4/Parser/lexer/state.c#L9) and [`lexer.c` L526](https://github.com/python/cpython/blob/v3.14.4/Parser/lexer/lexer.c#L526) at `v3.14.4` |
| at `0.0.23`, parsing refuses the module of the lines `if x:`, ` \t \tpass` and `  \t\tpass` | [the probe](https://github.com/kalonji-tools/purba/issues/1144#issuecomment-7064632646) |
| CPython 3.12, 3.13, 3.14 and 3.14t compile that module | [the probe of the tags](https://github.com/kalonji-tools/purba/issues/1144#issuecomment-7066152058) |
| `/Cargo.toml` pins `ruff_python_parser`, `ruff_python_ast`, `ruff_text_size` and `ruff_source_file` at `=0.0.23`, and turns on the feature `stacker` of `ruff_python_parser` | [`Cargo.toml` L19-L22 at `0fb42c27`](https://github.com/kalonji-tools/purba/blob/0fb42c27/Cargo.toml#L19-L22) |
| `purba-selection` names `ruff_text_size` | [`crates/selection/Cargo.toml` L13 at `af978a83`](https://github.com/kalonji-tools/purba/blob/af978a83/crates/selection/Cargo.toml#L13) |

## Considered Options

- **`rustpython-parser`.** It is not stale, it is abandoned, and it says so in three independent places. Its README now reads "superseded by". Its maintainer stated the repository will not be maintained and confirmed this covers the crate. The interpreter it was built for migrated onto a fork of ruff's parser and keeps no reference to it. The consequences are present rather than prospective: it declares no grammar version anywhere and fails outright on 7 of 774 CPython 3.14 source files.

Anyone re-checking the rejected crate will find a repository whose metadata says it is not archived, with a recent commit.
The evidence is in its issues and its README, not in repository metadata.

## Decision Outcome

**Reach:** `/Cargo.toml` `crates/**` `/.cargo/deny.toml` `/.cargo/clippy.toml` `tests/skeleton.bats`

**Area:** parsing

**purba parses with four ruff crates behind a thin parsing seam.**
The API of `ruff_python_parser` is not meaningfully unstable for this use.
Across six published versions over ten weeks the whole surface a prescan touches is byte-identical.
That surface is the parse entry point, its options and result types, all 25 statement and 33 expression variants, and every visitor trait.
The churn sits in peripheral helpers.

The lexer that parsing also calls, `lex` and `Lexer::current_range`, is outside that measured surface.
`[workspace.dependencies]` pins each crate it names exactly at `=0.0.23`.

`[workspace.dependencies]` turns on the feature `stacker` of `ruff_python_parser`.
With the feature, ruff's parser grows the stack while it parses a module that nests deep.
From `0.0.19` the feature is off by default.
Without it, purba's test of a module that nests deeper than its stack aborts with a stack overflow.

[Parsing refuses a module that opens a bracket inside 200 others](parsing-refuses-a-module-that-opens-a-bracket-inside-200-others.md#decision-outcome) holds where the feature does not keep a deep module from an overflow.

| crate | what the seam needs from it |
|---|---|
| `ruff_python_parser` | the parse entry point, and its lexer: `lex` and `Lexer::current_range` |
| `ruff_python_ast` | the AST the seam returns |
| `ruff_text_size` | the `Ranged` trait. `Expr::range()` does not resolve without it |
| `ruff_source_file` | `LineIndex`, so that a byte offset becomes a line and a column |

The manifest names them at the call site that first uses one, and not ahead of it.
Each ruff crate requires the others at its own version, so the pin also holds a crate that the manifest does not name yet.
Cargo reports most dependencies that nothing calls.
Nobody can suppress that report for one dependency alone.
Cargo misses some ruff crates that nothing calls, so the reviewer checks each one.

Parsing gives ruff's AST of a module through a thin seam, with the module's facts beside it.
Selection judges from the facts, and assertion takes the AST.
[Parsing matches the shape of each expectation in Rust, on ruff's AST](parsing-matches-the-shape-of-each-expectation-in-rust-on-ruffs-ast.md#decision-outcome) holds how parsing reads an expectation in that AST.
[Command lends one parse to selection and assertion](command-lends-one-parse-to-selection-and-assertion.md) holds who asks for the AST, and when.
[Each fact about a run is found once, by the part that sees it, and crosses as plain data](each-fact-is-found-once-by-the-part-that-sees-it.md#decision-outcome) holds why parsing finds the facts that selection and assertion read.
Holding that seam keeps even a late reversal a one-decision change.

Parsing returns a syntax error as an error, not a panic, because a file that fails to parse is a project error, not a crash.
[Parsing hands on no tree that ruff recovered](parsing-hands-on-no-tree-that-ruff-recovered.md#decision-outcome) says what that error holds.
MSRV is the upgrade clock, not the API.

**A coder raises the pin by hand, when one of three triggers fires.**

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
| `0.0.23` | 0.17.6 |

There is no `0.0.1`, and the mapping is not arithmetic.

**Downside:**

- **The publisher commits to nothing.** These crates first reached the registry eleven weeks before this decision, published so another project could consume them. No policy statement commits their publisher to a continuation. Confidence over the measured window is high and extrapolating is not.
- **MSRV moves about every six weeks**, three times in ten weeks, and each move is a pin somebody has to raise.
- **The dependency count is higher**, at 61 transitive against 45, and four direct crates to name against one.
- **Nothing notices a new ruff release.** A trigger reaches the coder only when they meet it.
- **ruff and CPython count a tab to different columns.** ruff counts a tab to the next multiple of 2 columns. CPython counts it to the next multiple of 8. Parsing refuses a module whose lines indent alike at 8 columns to a tab and unlike at 2, such as `if x:`, then ` \t \tpass`, then `  \t\tpass`. CPython compiles it.

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

`/Cargo.toml` pins `ruff_python_parser`, `ruff_python_ast`, `ruff_text_size` and `ruff_source_file` at `=0.0.23`.
It turns on the feature `stacker` of `ruff_python_parser`.

| pinned crate | the crates that name it |
|---|---|
| `ruff_python_parser` | `purba-parsing` |
| `ruff_python_ast` | `purba-parsing`, `purba-selection`, `purba-assertion` |
| `ruff_text_size` | `purba-parsing`, `purba-selection`, `purba-assertion` |
| `ruff_source_file` | `purba-parsing` |

`mise run test:rust` runs the tests in `crates/parsing/tests/parse.rs`, which parse a module through `purba-parsing`, and check that parsing refuses a module with a syntax error.
Without `stacker`, its test of a module that nests deeper than its stack and does not parse aborts with a stack overflow.
It also runs the tests in `crates/assertion/tests/rewrite.rs`, which rewrite a module at the ranges of its nodes.
`mise run lint:licences` reads the exceptions in `.cargo/deny.toml`, and refuses one that no crate uses.

Cargo refused an unused `itoa` in `purba`.
It passed `ruff_text_size` and `ruff_source_file` in `purba-parsing`, when nothing there called either.

The `.cargo_vcs_info.json` of each pinned version names the commit of a ruff tag.
That tag is the ruff release in the row of that pin.
A prescan turns the choice of parser into a fitness function, and counts parse failures against a corpus.
