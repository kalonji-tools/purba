# A tuple assert keeps CPython's warning and counts as no assert

## Context and Problem Statement

On `main` at `78232b7`, assertion rewrites each assert so that it records that it ran.
CPython warns that an assert whose expression is a tuple with items is always true.
The rewrite puts the tuple in a call, so CPython never warns.

| fact | source |
|---|---|
| on `main` at `78232b7`, a test with only `assert (1 == 2, "msg")` prints `passed`, the run exits 0, and stderr is empty | [The assert rewrite removes CPython's warning that a tuple assert is always true](https://github.com/kalonji-tools/purba/issues/812) |
| CPython warns when the expression of an assert is a tuple with items, and not when it is an empty tuple | [CPython 3.12 `compile.c`](https://github.com/python/cpython/blob/v3.12.0/Python/compile.c#L3865-L3875), [CPython 3.14 `codegen.c`](https://github.com/python/cpython/blob/v3.14.0/Python/codegen.c#L2934-L2944) |
| CPython 3.12.14 also warns for `assert (1,) * 2`, which it folds into a tuple, and not for `assert _purba_ran(0, (1,) * 2)`; CPython 3.14.7 warns for neither | the post pass of [the pull request of the issue](https://github.com/kalonji-tools/purba/pull/875) |

## Considered Options

- **Assertion rewrites a tuple assert as it rewrites any other assert.** The test then prints `passed`, and the tester never sees CPython's warning.
- **Selection refuses a test whose own body holds a tuple assert.** Selection does not judge an assert in a helper or at the module's level, so those asserts still lose the warning.

## Decision Outcome

**Reach:** `crates/**` `tests/**`

**Area:** assertion

**Assertion leaves an assert of a tuple with items as written, so CPython warns, and the assert counts as no assert that ran.**

- **A test with no other assert prints `unknown`.** [purba is always strict](purba-is-always-strict.md#decision-outcome) shows and counts `unknown`.
- **The rule holds for an assert anywhere in a module.** CPython warns for an assert in a test, in a helper or at the module's level, when it compiles the module.
- **Assertion rewrites an assert of an empty tuple as it rewrites any other assert.** An assert of an empty tuple always fails, and CPython does not warn about it.

[A tester runs purba on one module, and reads one line for each test](a-tester-runs-purba-on-one-module-and-reads-one-line-for-each-test.md#decision-outcome) says what an `unknown` line says.

**Downside:**

- **A test with a tuple assert and an assert that holds prints `passed`.** Only the warning on stderr shows the tuple.
- **The `unknown` line says `no assert ran`, though the tuple assert ran.** The warning above it gives the line number of that assert.
- **CPython 3.12 folds an expression such as `(1,) * 2` into a tuple, and the rewrite still removes its warning.** Assertion sees no tuple in that expression.

## Confirmation

`mise run test:wheel` checks that a test with only a tuple assert prints `unknown`, and that stderr holds CPython's warning.
It checks that an assert of an empty tuple fails, and that stderr is empty.
`mise run test:rust` checks that the rewrite leaves a tuple assert as written and gives it no index.
A property test of `mise run test:rust` checks the same for each tuple assert that it generates.
