# Parsing refuses a module whose chain of operators reaches 6,000

## Context and Problem Statement

ruff does not grow the stack while it marks an assignment target, or a value before a subscript.
That mark follows each unary operand and each starred value.
A target that chains enough operators makes purba crash and print no line.
A target of unary operators alone opens no bracket, so the limit of 200 brackets does not refuse it.
Only the code, and no probe, names `set_expr_ctx` as the frame that overflows.

| fact | source |
|---|---|
| ruff marks the context of an assignment target with `set_expr_ctx`, which calls itself for each tuple, list, starred value and unary operand, and does not grow the stack | [`parser/helpers.rs` L10-L31 at `0.17.6`](https://github.com/astral-sh/ruff/blob/0.17.6/crates/ruff_python_parser/src/parser/helpers.rs#L10-L31) |
| before a subscript, ruff marks the value with `set_expr_ctx` | [`parser/expression.rs` L889-L891 at `0.17.6`](https://github.com/astral-sh/ruff/blob/0.17.6/crates/ruff_python_parser/src/parser/expression.rs#L889-L891) |
| on `781867ec`, purba prints nothing and exits 139 on a target of 100,000 `-`, of 200,000 `not`, of 50,000 `*`, of 200 rows of 5,999 `-` joined by 199 `(`, and on a value of 200,000 `-` before a subscript | [the probe of the exits](https://github.com/kalonji-tools/purba/pull/1199#issuecomment-7101046225) |
| on `781867ec`, a target of 50,000 `-` gives ruff's error | [the probe of the exits](https://github.com/kalonji-tools/purba/pull/1199#issuecomment-7101046225) |
| CPython's parser refuses a parse 6,000 levels deep | [`parser.c` L13 at `v3.12.0`](https://github.com/python/cpython/blob/v3.12.0/Parser/parser.c#L13), [L17 at `v3.13.0`](https://github.com/python/cpython/blob/v3.13.0/Parser/parser.c#L17), [L17 at `v3.14.4`](https://github.com/python/cpython/blob/v3.14.4/Parser/parser.c#L17) |
| each unary operator is one call of `factor_rule` or `inversion_rule`, and each call is one level | [`factor_rule`](https://github.com/python/cpython/blob/v3.12.0/Parser/parser.c#L13799) and [`inversion_rule`](https://github.com/python/cpython/blob/v3.12.0/Parser/parser.c#L11971) at `v3.12.0` |
| in the places that the probes tried, CPython 3.12.14, 3.13.13, 3.14.4 and 3.14.10t refuse a row of 5,955 to 5,970 unary operators with `MemoryError: Parser stack overflowed - Python source too complex to parse` | [the probe of the search, part B](https://github.com/kalonji-tools/purba/pull/1199#issuecomment-7100987388), [the probe of the levels, part A](https://github.com/kalonji-tools/purba/pull/1199#issuecomment-7101776390) |
| the four builds refuse `y = -×n x` and `y = a - -×n x` both from n = 5,968, so a binary operator before a row is no level; they refuse `y = [-×n x]` from 5,957 and `y = [*-×n x]` from 5,963, so a `*` there takes 6 levels off; a statement of `-×n x` alone needs 5,970 | [the probe of the levels, part A](https://github.com/kalonji-tools/purba/pull/1199#issuecomment-7101776390) |
| CPython's parser raises the overflow with no place | [`pegen_errors.c` L458-L463 at `v3.12.0`](https://github.com/python/cpython/blob/v3.12.0/Parser/pegen_errors.c#L458-L463) |
| the same four builds refuse each of 17 modules that this count puts at 6,000, and a module with a binary `+` before 5,999 `-` | [the probe of each module](https://github.com/kalonji-tools/purba/pull/1199#issuecomment-7099977780) |
| the same four builds refuse a second `*` in a row: in a target with `invalid syntax`, and in a list or a call with `Invalid star expression` | [the probe of each module](https://github.com/kalonji-tools/purba/pull/1199#issuecomment-7099977780) |
| the same four builds refuse 3,000 `-` then 3,000 `not`, and 3,000 `~` then 3,000 `*`, with `invalid syntax`, and purba refuses each with the `MemoryError` | [the probe of the rows, part A](https://github.com/kalonji-tools/purba/pull/1199#issuecomment-7100987388) |
| with 5,990 `-`, a target that is a subscript or an attribute reaches CPython's load, and its test is a blocked test; a target that is the chain itself gets ruff's error with 5,999 | [the probe of the rows, part A](https://github.com/kalonji-tools/purba/pull/1199#issuecomment-7100987388) |
| a value of 5,990 or 5,999 `-` in a def reaches CPython's load, and its test is a blocked test | [the probe of the value, part B](https://github.com/kalonji-tools/purba/pull/1199#issuecomment-7101776390) |
| CPython 3.12.14 and 3.13.13 refuse `x = t"{` + 6,000 `-` as `invalid syntax` at 1:6; CPython 3.14.4 and 3.14.10t refuse it with the `MemoryError` | [the probe of each module](https://github.com/kalonji-tools/purba/pull/1199#issuecomment-7099977780) |
| a debug build overflows a stack of 512 KiB on a target of 5,999 `-`, and parses it on 1 MiB; a release build parses it on 256 KiB | [the probe of the stack](https://github.com/kalonji-tools/purba/pull/1199#issuecomment-7099433380) |

## Considered Options

- **Parsing grows the stack to fit the length of the module, around ruff's parse.** The size of a frame of `set_expr_ctx` is a guess.
- **purba carries a ruff whose `set_expr_ctx` grows the stack.** purba would keep a fork of ruff's parser.
- **The error names `SyntaxError`, as the error of a bracket does.** CPython names `MemoryError` for such a module. CPython's load prints the `MemoryError` line for a shorter row too.

## Decision Outcome

**Reach:** `crates/parsing/**` `crates/command/**` `crates/selection/tests/**` `crates/assertion/tests/**` `tests/skeleton.bats`

**Area:** parsing

**Parsing refuses a module whose chain of operators reaches 6,000, before ruff parses it.**
Each unary operator of the chain is one level of CPython's parser.
A binary operator or a `*` that starts a row is no level.
The statement and each bracket around the row add more levels than that.
So CPython's parser refuses the same module, and purba refuses no module that CPython loads.

That is a reading of CPython's parser.
The probes found no module that purba refuses and CPython loads.
Parsing counts the chain in the pass of ruff's lexer that counts the brackets.

- **Parsing counts each `-`, `+`, `~`, `not` and `*` in a row.** It adds to the count the row before each bracket that is still open. A comment or a line break inside brackets keeps the row. Any other token ends it. A binary `-`, `+` or `*` before a row counts too.
- **`*` counts, though it is not a unary operator.** ruff's mark follows a starred value. CPython refuses a second `*` in a row.
- **The error is a project error with the words of CPython's parser, `MemoryError: Parser stack overflowed - Python source too complex to parse`.** The error holds the place of the operator that makes the chain 6,000 long. The line shows no place, as CPython's error has none.
- **Parsing names whichever error comes first in the tokens: this one, or the one that [parsing refuses a module that opens a bracket inside 200 others](parsing-refuses-a-module-that-opens-a-bracket-inside-200-others.md#decision-outcome) holds.**
- **This is an exception to [CPython alone refuses a module that its compiler refuses](cpython-alone-refuses-a-module-that-its-compiler-refuses.md#decision-outcome).** ruff can overflow the stack on a target that chains deep, before CPython loads the module.
- **This error comes before ruff's first syntax error.** [Parsing hands on no tree that ruff recovered](parsing-hands-on-no-tree-that-ruff-recovered.md#decision-outcome) holds the error that ruff finds.
- **This is an exception to [an error stops only the work that depends on it](an-error-stops-only-the-work-that-depends-on-it.md#decision-outcome).** Parsing names this error alone, and no error of ruff.
- **This is an exception to [CPython alone refuses the syntax that its version lacks](cpython-alone-refuses-the-syntax-that-its-version-lacks.md#decision-outcome).** On CPython 3.12 and 3.13, a t-string that chains 6,000 operators gets this error, and not CPython's `invalid syntax` at the load.

**Downside:**

- **A row just short of 6,000 operators can reach CPython's load.** Such a row reaches it in a value, and in a target that is a subscript or an attribute. CPython's load then blocks each test of the module, with the `MemoryError` line. A target that is the chain itself gets ruff's error first.
- **A module that parsing refuses gives no blocked test.** Parsing refuses it before CPython's load can block its test.
- **A small stack can still overflow below the limit.** A debug build on a small stack overflows on a target one operator short of it, as the Context shows.
- **A chain of 6,000 that CPython refuses as a syntax error gets the words of the `MemoryError`.** CPython names `invalid syntax` or `Invalid star expression` for such a chain. A second `*` in a row, `-` before `not`, and `~` before `*` each make one.
- **Parsing does more work on each token.** No probe measures it.
- **No test shows a walk over a long row of unary operators.** The tests of each walk nest through a row of `a ** -` or `1 ** -`, and through subscripts.

## Confirmation

| what it checks | where |
|---|---|
| 5,999 and 6,000 operators of `-`, `not`, a mix and `*`, 6,000 of `+`, of `~` and after a binary operator, and the place of the error | `mise run test:rust`, `crates/parsing/tests/parse.rs` |
| a row through brackets, and a row that a name or a closing bracket ends | `mise run test:rust`, `crates/parsing/tests/parse.rs` |
| a row split by a comment or a line break inside brackets, or by a backslash, and a row that the end of a line ends | `mise run test:rust`, `crates/parsing/tests/parse.rs` |
| a row in a field of an f-string or in a lambda, and none in a string or a comment | `mise run test:rust`, `crates/parsing/tests/parse.rs` |
| whichever of a chain and a bracket comes first names the error | `mise run test:rust`, `crates/parsing/tests/parse.rs` |
| a target, and a value before a subscript, that chain deep, on a small stack | `mise run test:rust`, `crates/parsing/tests/parse.rs` |
| command reports the error as a project error, and exits 4 | `mise run test:rust`, `crates/command/tests/run.rs` |
| `purba` prints the words of CPython's parser on a target that chains deep, and ruff's error one operator short | `mise run test:wheel`, `tests/skeleton.bats` |

[An assignment target that chains unary operators deep crashes purba with no output](https://github.com/kalonji-tools/purba/issues/1192), [a subscripted value that chains unary operators deep crashes purba with no output](https://github.com/kalonji-tools/purba/issues/1195) and [an assignment target that chains starred values deep crashes purba with no output](https://github.com/kalonji-tools/purba/issues/1198) hold the measurement.
