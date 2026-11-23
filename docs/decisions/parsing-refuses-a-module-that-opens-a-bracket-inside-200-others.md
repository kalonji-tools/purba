# Parsing refuses a module that opens a bracket inside 200 others

## Context and Problem Statement

ruff's parser grows the stack while it parses a module that nests deep.
It does not grow the stack while it marks an assignment target, or a value before a subscript.
A target that nests deep makes purba die with no line.
Only the code, and no probe, names `set_expr_ctx` as the frame that overflows.

| fact | source |
|---|---|
| with the feature `stacker`, ruff's parser grows the stack through `stacker::maybe_grow` | [`parser/mod.rs` L950-L961 at `0.17.6`](https://github.com/astral-sh/ruff/blob/0.17.6/crates/ruff_python_parser/src/parser/mod.rs#L950-L961) |
| ruff marks the context of an assignment target with `set_expr_ctx`, which calls itself for each tuple, list, starred value and unary operand, and does not grow the stack | [`parser/helpers.rs` L10-L31 at `0.17.6`](https://github.com/astral-sh/ruff/blob/0.17.6/crates/ruff_python_parser/src/parser/helpers.rs#L10-L31) |
| before a subscript, ruff marks the value with `set_expr_ctx` | [`parser/expression.rs` L889-L891 at `0.17.6`](https://github.com/astral-sh/ruff/blob/0.17.6/crates/ruff_python_parser/src/parser/expression.rs#L889-L891) |
| at `0.0.23` with `stacker`, purba prints nothing and exits 139 on a module whose assignment target nests 50,000 tuples deep | [the probe](https://github.com/kalonji-tools/purba/issues/1163#issuecomment-7087813195) |
| purba also exits 139 on a chain of 200,000 unary operators as an assignment target, or as a value before a subscript | [the probe of the target](https://github.com/kalonji-tools/purba/issues/1192), [the probe of the value](https://github.com/kalonji-tools/purba/issues/1195) |
| on `f546586b`, a module whose brackets nest 50,000 deep in a value reaches the load, and its test is a blocked test | [the probe](https://github.com/kalonji-tools/purba/issues/1163#issuecomment-7087813195) |
| CPython's tokenizer refuses a bracket opened inside 200 others with `too many nested parentheses`, and counts `(`, `[`, `{` and the `{` of an f-string | [`tokenizer.h` L14 at `v3.12.0`](https://github.com/python/cpython/blob/v3.12.0/Parser/tokenizer.h#L14), [`state.h` L7 at `v3.13.0`](https://github.com/python/cpython/blob/v3.13.0/Parser/lexer/state.h#L7), [`state.h` L7](https://github.com/python/cpython/blob/v3.14.4/Parser/lexer/state.h#L7) and [`lexer.c` L1305-L1306](https://github.com/python/cpython/blob/v3.14.4/Parser/lexer/lexer.c#L1305-L1306) at `v3.14.4` |
| CPython 3.12.14, 3.13.13, 3.14.4 and 3.14.7t refuse the bracket 201 deep, start and end that error at the bracket, and compile 200 | [the probe of four builds](https://github.com/kalonji-tools/purba/issues/1163#issuecomment-7095967261) |
| the same four builds name their own error first for a module whose parser stack overflows (`MemoryError: Parser stack overflowed - Python source too complex to parse`), whose format specs nest 4 deep, or whose earlier line holds an error of the tokenizer | [the probe of four builds](https://github.com/kalonji-tools/purba/issues/1163#issuecomment-7095967261) |
| CPython 3.12.14 and 3.13.13 have no t-string, and refuse `x = t"{` + 200 `(` as `invalid syntax` at 1:6; CPython 3.14.4 and 3.14.7t refuse it as `too many nested parentheses` | [the probe of the t-string](https://github.com/kalonji-tools/purba/issues/1163#issuecomment-7096543745) |
| `ruff_python_parser::lexer::lex` and `Lexer::current_range` are public, and ruff hides them from its documentation | [`lexer.rs` L123-L127](https://github.com/astral-sh/ruff/blob/0.17.6/crates/ruff_python_parser/src/lexer.rs#L123-L127) and [L1557-L1561](https://github.com/astral-sh/ruff/blob/0.17.6/crates/ruff_python_parser/src/lexer.rs#L1557-L1561) at `0.17.6` |
| on a module of 1 MB, the median time of ten parses rises from 8.95 ms to 9.36 ms when parsing counts the brackets, and the two ranges overlap | [the probe of the cost](https://github.com/kalonji-tools/purba/issues/1163#issuecomment-7096543745) |

## Considered Options

- **Parsing asks CPython's compiler for the error of a module that nests too deep.** CPython gives a `MemoryError` and no syntax error for a module whose parser stack overflows before the bracket. ruff then parses that module, and can overflow the stack on it.
- **Parsing grows the stack to fit the length of the module, around ruff's parse.** The size of a frame of `set_expr_ctx` is a guess.

## Decision Outcome

**Reach:** `crates/parsing/**` `crates/selection/tests/**` `crates/assertion/tests/**` `tests/skeleton.bats`

**Area:** parsing

**Parsing refuses a module that opens a bracket inside 200 others, before ruff parses it.**
CPython refuses the same module.
So purba refuses no module that CPython loads.
Parsing counts the brackets in the tokens of ruff's lexer.

- **The error is a project error with the words of CPython's tokenizer, `too many nested parentheses`.** It starts at the bracket that opens inside 200 others. Where CPython's tokenizer reads the same tokens, its error starts at that bracket too.
- **This is an exception to [CPython alone refuses a module that its compiler refuses](cpython-alone-refuses-a-module-that-its-compiler-refuses.md#decision-outcome).** ruff can overflow the stack on an assignment target that nests deep, before CPython loads the module.
- **This error comes before ruff's first syntax error.** [Parsing hands on no tree that ruff recovered](parsing-hands-on-no-tree-that-ruff-recovered.md#decision-outcome) holds the error that ruff finds.
- **This is an exception to [an error stops only the work that depends on it](an-error-stops-only-the-work-that-depends-on-it.md#decision-outcome).** Parsing names this error alone, and no error of ruff.
- **This is an exception to [CPython alone refuses the syntax that its version lacks](cpython-alone-refuses-the-syntax-that-its-version-lacks.md#decision-outcome).** On CPython 3.12 and 3.13, a t-string that nests 200 brackets gets this error, and not CPython's `invalid syntax` at the load.
- **[purba parses Python with ruff's parser](purba-parses-python-with-ruff.md#decision-outcome) holds the feature `stacker`.** The feature does not grow the stack where ruff marks an assignment target or a value before a subscript. Parsing refuses such a mark when it nests through brackets, and does not refuse a deep chain of unary operators.

**Downside:**

- **A module that CPython refuses first for another reason gets the error of the bracket.** Such a module overflows the parser stack, nests its format specs too deep, or holds an earlier error of the tokenizer. On CPython 3.12 and 3.13, a t-string that nests the brackets is such a module too.
- **The error ends after the bracket.** CPython's error ends where it starts.
- **A module whose brackets nest past 200 gives no blocked test.** Parsing refuses it before CPython's load can block its test.
- **Parsing lexes each module once more before ruff parses it.** The Context measures the cost on a large module.
- **ruff hides `lex` and `Lexer::current_range` from its documentation.** The exact pin of [purba parses Python with ruff's parser](purba-parses-python-with-ruff.md#decision-outcome) holds them.
- **No test shows that a walk survives more than 200 nested brackets.** The modules of the tests of each walk nest deep through unary operators and subscript chains instead.
- **A chain of unary operators nests with no bracket.** Parsing does not refuse it, as an assignment target or as a value before a subscript.

## Confirmation

| what it checks | where |
|---|---|
| 200 and 201 brackets of each kind, and the place of the error | `mise run test:rust`, `crates/parsing/tests/parse.rs` |
| a target that nests deep, on a small stack | `mise run test:rust`, `crates/parsing/tests/parse.rs` |
| a bracket in a string or a comment | `mise run test:rust`, `crates/parsing/tests/parse.rs` |
| `purba` exits 4 with the words of the tokenizer on a target that nests deep | `mise run test:wheel`, `tests/skeleton.bats` |

[An assignment target that nests deep crashes purba with no output](https://github.com/kalonji-tools/purba/issues/1163) holds the measurement.
[An assignment target that chains unary operators deep crashes purba with no output](https://github.com/kalonji-tools/purba/issues/1192) and [a subscripted value that chains unary operators deep crashes purba with no output](https://github.com/kalonji-tools/purba/issues/1195) report the chains that parsing does not refuse.
