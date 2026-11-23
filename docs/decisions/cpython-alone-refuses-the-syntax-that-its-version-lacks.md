# CPython alone refuses the syntax that its version lacks

## Context and Problem Statement

ruff's parser accepts the syntax of each Python version that it knows.
It lists the syntax that its target version lacks in `unsupported_syntax_errors()`, and not in `errors()`.
Parsing gives ruff no target, so ruff takes its default target.
Parsing reads only `errors()`.
A module in the syntax of a newer Python parses, and the CPython that loads it refuses it.

[Parsing hands on no tree that ruff recovered](parsing-hands-on-no-tree-that-ruff-recovered.md#decision-outcome) decides what parsing does with the errors that it reads.
[CPython alone refuses a module that its compiler refuses](cpython-alone-refuses-a-module-that-its-compiler-refuses.md#decision-outcome) decides the checks that come after a parse.

| fact | source |
|---|---|
| parsing calls `parse_unchecked_source(&source, PySourceType::Python)`, and keeps the first entry of `errors()` | [`crates/parsing/src/lib.rs` L427-L431](https://github.com/kalonji-tools/purba/blob/80691b24/crates/parsing/src/lib.rs#L427-L431) |
| with no target, ruff's target is 3.11 | [ruff's `ParseOptions`](https://github.com/astral-sh/ruff/blob/0.17.0/crates/ruff_python_parser/src/parser/options.rs#L51-L58), [ruff's `PythonVersion`](https://github.com/astral-sh/ruff/blob/0.17.0/crates/ruff_python_ast/src/python_version.rs#L99-L103) |
| ruff passes 7 constructs that a supported CPython refuses for its version, such as `t'{1}'` on 3.12 and `lazy import os` on each | [the research](https://github.com/kalonji-tools/purba/blob/25249e34f65d058e3f04eb5073479f82a8c10f1a/research/parsing-ruff-and-cpython.md?plain=1#L250-L269) |
| with a target of 3.12, 3.13 or 3.14, `unsupported_syntax_errors()` agrees with CPython's `compile()` on 36 of 36 pairs of a construct and a version | [the probe](https://github.com/kalonji-tools/purba/pull/1140#issuecomment-7062425778) |
| on `98de3a97`, `y = t"x"` on 3.12 prints `SyntaxError: invalid syntax (test_tstring.py, line 1)` as a project error, and its test as blocked, exit 4 | [the probe](https://github.com/kalonji-tools/purba/pull/1140#issuecomment-7062425778) |
| ruff's words name the version: `Cannot use t-strings on Python 3.12 (syntax was added in Python 3.14)` | [the probe on the issue](https://github.com/kalonji-tools/purba/issues/1130#issuecomment-7061770130) |

## Considered Options

- **Parsing gives ruff the version of the CPython that loads the module, and refuses the syntax that this version lacks.** Two parts then judge the same syntax. Where ruff judges a construct wrong, parsing refuses a module that CPython runs. The count then holds none of the module's tests.

## Decision Outcome

**Reach:** `crates/parsing/**` `/Cargo.toml`

**Area:** parsing

**CPython alone refuses the syntax that its version lacks.**
CPython is the one judge of the syntax that it runs.
Parsing gives ruff no target version.
Parsing reads only `errors()`.

- **A module in the syntax of a newer Python fails when CPython loads it.** It is a project error in CPython's words. Each test that selection names is blocked.
- **A t-string that nests 200 brackets is refused before the load, on CPython 3.12 and 3.13.** [Parsing refuses a module that opens a bracket inside 200 others](parsing-refuses-a-module-that-opens-a-bracket-inside-200-others.md#decision-outcome) holds why.

**Downside:**

- **CPython's words name no version.** For `y = t"x"` on 3.12 they read `invalid syntax`.
- **Selection and assertion read a tree that the CPython of the run cannot compile.** The failed load then wastes that work.

## Confirmation

`mise run test:rust` checks that parsing passes each construct of the research's table.
The `wheel` job of `ci.yml` runs `mise run test:wheel` on each OS and each Python version.
It checks that a module that opens with `lazy import os`, or with `f((a)=1)`, prints a project error in CPython's words.
It also checks that the module's test prints as blocked.
