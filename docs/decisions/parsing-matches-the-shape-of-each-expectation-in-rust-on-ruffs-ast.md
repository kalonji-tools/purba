# Parsing matches the shape of each expectation in Rust, on ruff's AST

## Context and Problem Statement

Selection and assertion both read the shape of an [expectation](../../CONTEXT.md#expectation).
Parsing matches that shape for both.
[purba parses Python with ruff's parser](purba-parses-python-with-ruff.md#decision-outcome) chose the parser.

| fact | source |
|---|---|
| parsing holds what selection and assertion share: the asserts at the end of a test, the blocks of a statement, and each assert's shape | [the landscape, L59](https://github.com/kalonji-tools/purba/blob/7b0139f2/LANDSCAPE.md?plain=1#L59) |
| no part but parsing turns Python source into an AST | [the landscape, L146](https://github.com/kalonji-tools/purba/blob/7b0139f2/LANDSCAPE.md?plain=1#L146) |
| a prototype wrote each shape as an ast-grep rule, a tree pattern on tree-sitter's tree | [the prototype](https://github.com/kalonji-tools/purba/issues/1128#issuecomment-7046130155) |
| on the 79 modules under `tests/`, the rules and purba give the same 14 findings | [the prototype](https://github.com/kalonji-tools/purba/issues/1128#issuecomment-7046130155) |
| on 21 test modules that the prototype wrote, the rules and purba disagree on 7 | [the prototype](https://github.com/kalonji-tools/purba/issues/1128#issuecomment-7046130155) |
| tree-sitter gives `def test_sum(:` a tree with no error node, and ruff refuses it | [the prototype](https://github.com/kalonji-tools/purba/issues/1128#issuecomment-7046130155), which names no version of ast-grep |
| the rules find a breach in a comment in a list or a comparison, a `\` in a comparison, or the `( )` around a generator, and purba passes each of them | [the prototype](https://github.com/kalonji-tools/purba/issues/1128#issuecomment-7046130155), which names no version of ast-grep |
| the rules restate which `def` is a test, and disagree with selection on 2 test modules | [the prototype](https://github.com/kalonji-tools/purba/issues/1128#issuecomment-7046130155) |

## Considered Options

- **Tree patterns, as ast-grep rules on tree-sitter's tree.** ast-grep reads each module through a second tree, tree-sitter's. Its rules and purba disagree where the two trees differ.

## Decision Outcome

**Reach:** `crates/parsing/**` `crates/selection/**` `crates/assertion/**`

**Area:** parsing

**Parsing matches the shape of each expectation in Rust, on ruff's AST.**
The shape and the test that selection names come from one tree, the tree that ruff gives.

- **Selection and assertion read the shape of each expectation that parsing gives.** Neither part matches the shape of an expectation on its own.

**Downside:**

- **A reader of a shape reads Rust.** A tree pattern reads more like the Python it matches than Rust does. That is a judgement.

## Confirmation

`.cargo/clippy.toml` refuses a call of ruff's parser outside parsing.
The reviewer reads each tool that a change adds for a second parser.
`crates/parsing/tests/parse.rs` checks the shapes, the asserts at the end of a test, and the blocks of a statement.
`mise run test:rust` runs it.

[How do the shapes of an expectation look as tree patterns?](https://github.com/kalonji-tools/purba/issues/1128) holds the measurement.
