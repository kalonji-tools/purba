# A test names the fixture of each input

## Context and Problem Statement

A test names the inputs that fixtures build for it.
With `src: Tx[SourceDb]`, a reader found the fixture of `src` only through another file.

| fact | source |
|---|---|
| a reader of `src: Tx[SourceDb]` "took me a moment to realize that src and dst were correlated with tx target and source" | the review of the record of two values of one type, as [the question of the syntax](https://github.com/kalonji-tools/purba/issues/700) quotes it |
| of the four forms below, three agents that read blind ranked `= use(tx, source)` first in 12 of their 21 rankings | [the readers' answers](https://github.com/kalonji-tools/purba/tree/30147b4/proto-700/runs/readers) |
| with a fixture of the wrong type in `use(tx, …)`, mypy 2.4.0, pyright 1.1.414 and ty 0.0.93 each reported it; in `@using`, in `= use_any(…)` and in `Annotated[…, From(…)]`, none did | [the type checkers](https://github.com/kalonji-tools/purba/blob/30147b4/proto-700/runs/cases_checkers.txt) |
| over the eight forms of the prototype, the mean rank of `= use(tx, source)` was 2.38, and of `= use_any(source)` 2.90 | [the readers' answers](https://github.com/kalonji-tools/purba/tree/30147b4/proto-700/runs/readers) |
| the five inputs of one test took 159 characters in `= use(…)` | [the measures of each line](https://github.com/kalonji-tools/purba/blob/30147b4/proto-700/runs/cases_measure.jsonl) |

## Considered Options

- **A test names the fixture of each input in a decorator, `@using(src=source)`.** Rejected. A fixture of the wrong type passed every type checker.
- **A test names the fixture of each input in a default typed `Any`, `= use_any(source)`.** Rejected. A fixture of the wrong type passed every type checker.
- **A test names the fixture of each input in `Annotated[Tx[Database], From(source)]`.** Rejected. A fixture of the wrong type passed every type checker.
- **A test names the fixture of each input in a typed default, `= use(tx, source)`.** Chosen. The readers gave it the best mean rank of the forms above. A type checker sees a fixture of the wrong type.

## Decision Outcome

**Reach:** `crates/**` `tests/**` `/CONTEXT.md` `/LANDSCAPE.md`

**A [test](../../CONTEXT.md#test) names the fixture of each of its [inputs](../../CONTEXT.md#input) in a typed default.**

```python
def test_copy(src: Tx[Database] = use(tx, source), dst: Tx[Database] = use(tx, target)) -> None: ...
```

`use(source)` names a fixture with no input.
`use(tx, source)` names the fixture that draws a [reagent](../../CONTEXT.md#reagent), and the [stock](../../CONTEXT.md#stock) that it draws from.
`use` is a sketch.
purba has no public surface for it yet.

- **Within one test, fixtures builds each `use(…)` once.** A fixture that names the same `use(…)` as the test gets the value that the test gets.
- **A parameter with any other default, `*args` and `**kwargs` are not inputs.** purba passes nothing for them.

**Downside:**

- **A tester writes two names for each input, where pytest takes one.** One names the parameter, and one names its fixture.
- **A test with many inputs wraps its line.**
- **A test reads like a function with defaults.** A call by hand gets the markers of `use`, not the values.

## Confirmation

`mise run test:rust` checks that selection refuses a parameter with no default.
The designs in depth of fixtures and execution, which [the landscape](../../LANDSCAPE.md#the-order-of-design) ranks, build the rest.
[A test returns nothing, and the first part that sees a breach of a test refuses it](a-test-returns-nothing-and-the-first-part-that-sees-a-breach-refuses-it.md#decision-outcome) says which part refuses a test that breaks this record.
[A test names two values of one type by two fixtures](a-test-names-two-values-of-one-type-by-two-fixtures.md#decision-outcome) says how a test names two values of one type.
