# A tuple assert keeps CPython's warning, and selection refuses a test that ends with one

## Context and Problem Statement

On `main` at `78232b7`, assertion rewrites each assert so that it records that it ran.
CPython warns that an assert whose expression is a tuple with items is always true.
The rewrite puts the tuple in a call, so CPython never warns.

| fact | source |
|---|---|
| on `main` at `78232b7`, a test with only `assert (1 == 2, "msg")` prints `passed`, the run exits 0, and stderr is empty | [The assert rewrite removes CPython's warning that a tuple assert is always true](https://github.com/kalonji-tools/purba/issues/812) |
| CPython warns when the expression of an assert is a tuple with items, and not when it is an empty tuple | [CPython 3.12 `compile.c`](https://github.com/python/cpython/blob/v3.12.0/Python/compile.c#L3865-L3875), [CPython 3.14 `codegen.c`](https://github.com/python/cpython/blob/v3.14.0/Python/codegen.c#L2934-L2944) |
| CPython 3.12.14 also warns for `assert (1,) * 2`, which it folds into a tuple, and not for `assert _purba_ran(0, (1,) * 2)`; CPython 3.14.7 warns for neither | the post pass of [the pull request of the issue](https://github.com/kalonji-tools/purba/pull/875) |
| on `main` at `794f075`, a test that ends with `assert (1 == 2, "msg")` gives the project error `no expectation stated`, and no assert above it is an expectation | [this record on `main` at `794f075`](https://github.com/kalonji-tools/purba/blob/794f075/docs/decisions/a-tuple-assert-keeps-cpythons-warning-and-counts-as-no-assert.md#decision-outcome) |

## Considered Options

- **Assertion rewrites a tuple assert as it rewrites any other assert.** The test then prints `passed`, and the tester never sees CPython's warning.
- **Selection refuses a test whose own body holds a tuple assert.** Selection does not judge an assert in a helper or at the module's level, so those asserts still lose the warning.
- **A tuple assert at a test's end counts as no expectation.** The line says `no expectation stated`, though the test states the tuple. No assert above the tuple is an expectation.

## Decision Outcome

**Reach:** `crates/**` `tests/**`

**Area:** assertion

**Assertion leaves an assert of a tuple with items as written, so CPython warns.**
At a test's end, a tuple assert is an [expectation](../../CONTEXT.md#expectation). Selection refuses its test, because an expectation is one comparison or one of four forms, unless a suppression of `not-one-comparison` lets it run.

- **Assertion leaves a tuple assert as written anywhere in a module.** CPython warns for an assert in a test, in a helper or at the module's level, when it compiles the module. Away from a test's end, the assert counts as no assert that ran.
- **The refused line names the tuple assert's line number.** [a test returns nothing, and the first part that sees a breach of a test refuses it](a-test-returns-nothing-and-the-first-part-that-sees-a-breach-refuses-it.md#decision-outcome) says how.
- **Assertion rewrites an assert of an empty tuple as it rewrites any other assert.** An assert of an empty tuple always fails, and CPython does not warn about it. At a test's end, selection refuses the test, because an empty tuple is no comparison, unless a suppression lets it run. Before the first expectation, execution refuses the case, as [a test returns nothing, and the first part that sees a breach of a test refuses it](a-test-returns-nothing-and-the-first-part-that-sees-a-breach-refuses-it.md#decision-outcome) says.

**Downside:**

- **A test with a tuple assert above a statement that is not an assert prints `passed` when its expectations hold.** Only the warning on stderr shows the tuple.
- **When selection refuses every test of a module, the module never loads, and CPython never warns.** The refused line still names the tuple's line number.
- **CPython 3.12 folds an expression such as `(1,) * 2` into a tuple, and the rewrite still removes its warning.** Assertion sees no tuple in that expression.

## Confirmation

`mise run test:wheel` checks that selection refuses a test that ends with a tuple assert.
It checks that the refused line names the tuple's line number.
It checks that a tuple assert in a helper keeps CPython's warning.
It checks that execution refuses an assert of an empty tuple above a statement that is not an assert, and that stderr is empty.

`mise run test:rust` checks that selection refuses a test that ends with a tuple assert or an assert of an empty tuple.
It checks that the rewrite leaves a tuple assert as written and gives it no index.
A property test of `mise run test:rust` checks the same for each tuple assert that it generates.
