# Parsing hands on no tree that ruff recovered

## Context and Problem Statement

ruff's parser recovers from a syntax error, and gives one tree beside the errors it finds.
Parsing kept only whether ruff found an error, and handed the recovered tree to selection and assertion.
[purba parses Python with ruff's parser](purba-parses-python-with-ruff.md#decision-outcome) chose the parser, and says that a syntax error is a project error.

| fact | source |
|---|---|
| selection names `test_bad1` from the tree that ruff recovered for `def test_bad1(:`, and each test of the module prints as blocked | [the spec](https://github.com/kalonji-tools/purba/issues/671#issuecomment-6498427165) |
| a module with a syntax error and no test prints `0 tests`, and exits 2 | [the probe of `b82e8b6`](https://github.com/kalonji-tools/purba/issues/671#issuecomment-6491871031) |
| ruff passes `return` outside a function, and `nonlocal` with no binding, and CPython refuses both | [the probe of `b82e8b6`](https://github.com/kalonji-tools/purba/issues/671#issuecomment-6491871031) |
| ruff lists three errors for a module where CPython names one, and two of them come from one mistake | [the spec](https://github.com/kalonji-tools/purba/issues/671#issuecomment-6498427165) |
| ruff's own drop of a recovered tree overflows a 256 KiB stack on a module that nests deep | [the spec](https://github.com/kalonji-tools/purba/issues/671#issuecomment-6498427165) |

## Considered Options

- **`Module` keeps `has_syntax_error()`, and each part that reads the tree checks it.** Rejected. A part that omits the check reads a recovered tree, as selection did when it named `test_bad1`.
- **`parse` returns an error in place of a `Module`.** Chosen. No later part can hold a recovered tree.

## Decision Outcome

**Reach:** `crates/parsing/**` `crates/command/**` `crates/assertion/**` `tests/**` `/Cargo.toml`

**Parsing hands on no tree that ruff recovered.**

- **A module with a syntax error gives a [project error](../../CONTEXT.md#project-error) in place of a module.** Command prints the module's line, and hands nothing to selection, assertion or dispatch.
- **The error holds ruff's first syntax error: its message, and the line and column where it starts and ends.** A column counts Unicode scalars, in the text after a byte order mark.
- **Its text is `SyntaxError: ` and ruff's message, and holds no location.** The line reads `<path> error: SyntaxError: <message>`.

```
test_paren.py error: SyntaxError: Expected a parameter or the end of the parameter list
0 tests: 0 passed, 0 failed, 0 raised, 0 unknown, 0 blocked; 1 error
```

**Downside:**

- **The printed line names no line or column.** CPython's text names the line. Reporting renders the location in its design in depth.
- **ruff's words and lines are not CPython's.** For `x = (` on the last line, ruff names the line after it.
- **Two parts find syntax errors, and print two texts.** A syntax error that ruff passes, such as `return` outside a function, shows at load in CPython's words.
- **A second mistake in a module shows only after the first is fixed.** It is an exception to [an error stops only the work that depends on it](an-error-stops-only-the-work-that-depends-on-it.md#decision-outcome), where a part names every error that it finds.
- **The count names no test of a module that does not parse.** It reads `0 tests`, though the file holds tests.

## Confirmation

`mise run test:rust` checks the error, its location, and that the parse of a deep module that does not parse frees its tree.
It checks command's lines for a module that does not parse, with a test and with none.
`mise run test:wheel` checks the lines of a module that does not parse and holds tests, on the installed `purba`.
