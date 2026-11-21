# CPython alone refuses a module that its compiler refuses

## Context and Problem Statement

CPython parses a module, then builds its symbol table and compiles it.
Both later steps can refuse a module that parses.
ruff checks many of their rules in `SemanticSyntaxChecker`, outside its parser.
Parsing runs only ruff's parser.
[Parsing hands on no tree that ruff recovered](parsing-hands-on-no-tree-that-ruff-recovered.md#decision-outcome) decides what parsing does with the errors that ruff's parser reports.

| fact | source |
|---|---|
| parsing calls `parse_unchecked_source(&source, PySourceType::Python)`, and keeps the first entry of `errors()` | [`crates/parsing/src/lib.rs` L427-L431](https://github.com/kalonji-tools/purba/blob/54c3457e/crates/parsing/src/lib.rs#L427-L431) |
| ruff's parser passes a module for each of 64 messages of CPython's compiler and symbol table, such as `'return' outside function` | [the research](https://github.com/kalonji-tools/purba/blob/25249e34f65d058e3f04eb5073479f82a8c10f1a/research/parsing-ruff-and-cpython.md?plain=1#L338-L383) |
| `SemanticSyntaxChecker` asks its caller about scopes through `SemanticSyntaxContext`, and ruff's own test context answers 10 of its methods with a constant | [ruff's `SemanticSyntaxContext`](https://github.com/astral-sh/ruff/blob/5b27c1ccfaa230fb195855f5cf3404a21e3379cd/crates/ruff_python_parser/src/semantic_errors.rs#L2540-L2651), [ruff's test context](https://github.com/astral-sh/ruff/blob/5b27c1ccfaa230fb195855f5cf3404a21e3379cd/crates/ruff_python_parser/tests/fixtures.rs#L617-L719) |
| of 60 modules that ruff parses, CPython 3.14 refuses 58, and the checker, with a model of scopes, refuses 47 of them | [the probe](https://github.com/kalonji-tools/purba/issues/1131#issuecomment-7063614288) |
| the checker refuses `from .__future__ import braces`, which CPython 3.13 and later run | [the probe](https://github.com/kalonji-tools/purba/issues/1131#issuecomment-7063614288) |
| a module that parsing refuses prints its project error and `0 cases`, and names none of its tests | [parsing hands on no tree that ruff recovered](parsing-hands-on-no-tree-that-ruff-recovered.md#decision-outcome), [the control](https://github.com/kalonji-tools/purba/pull/1142#issuecomment-7063300104) |
| on `e61ce6c4`, a module of `return 1` and a test prints `SyntaxError: 'return' outside function (test_return.py, line 1)` as a project error, and its test as blocked, on each supported CPython | [the run of `e61ce6c4`](https://github.com/kalonji-tools/purba/actions/runs/44907738302/job/136856858523) |
| on `a9450782`, a module of `return 1` alone prints `0 cases`, and the run exits 2 | [the run of `a9450782`](https://github.com/kalonji-tools/purba/actions/runs/44908950510/job/136860476321) |

## Considered Options

- **Parsing runs `SemanticSyntaxChecker`, and refuses each module that it reports.** Two parts then judge the same rules. The checker misses some modules that CPython refuses. It also refuses a module that CPython runs. Parsing then needs a model of scopes. The count then holds none of the module's tests.
- **Parsing compiles each module with CPython's `compile()`, and refuses what CPython refuses.** Parsing then needs Python for each module. The count then holds none of the module's tests.

## Decision Outcome

**Reach:** `crates/parsing/**` `/Cargo.toml` `tests/skeleton.bats`

**Area:** parsing

**CPython alone refuses a module that its compiler refuses.**
CPython is the one judge of the rules that its compiler applies.
Parsing runs no `SemanticSyntaxChecker`, and no `compile()`.

- **Such a module fails when CPython loads it.** It is a project error in CPython's words, with its file and line. Each test that selection names is blocked.

**Downside:**

- **Selection and assertion read a module that the CPython of the run cannot compile.** The failed load then wastes that work.
- **A module in which selection names no test shows no error.** A module of `return 1` alone prints only the count.

## Confirmation

`mise run test:rust` checks that each module that [the research's table](https://github.com/kalonji-tools/purba/blob/25249e34f65d058e3f04eb5073479f82a8c10f1a/research/parsing-ruff-and-cpython.md?plain=1#L342-L357) writes out parses.
The `wheel` job of `ci.yml` runs `mise run test:wheel` on each OS and each Python version.
It checks that a module that opens with `return 1` prints a project error in CPython's words.
It also checks that the module's test prints as blocked.
It checks too that a module of `return 1` alone prints only the count.
