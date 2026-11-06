# A test names two values of one type by two fixtures

## Context and Problem Statement

Execution refuses a test with two inputs that name one fixture, as [a test returns nothing, and the first part that sees a breach of a test refuses it](a-test-returns-nothing-and-the-first-part-that-sees-a-breach-refuses-it.md#decision-outcome) decides.
A test of a copy needs two databases: the one that it copies from, and the one that it copies to.

| fact | source |
|---|---|
| with one value for each type, a copy that copies nothing passed its test | [the prototype of two values of one type](https://github.com/kalonji-tools/purba/blob/010dda4/proto-690/RESULTS.md) |
| each way that gave the test two objects failed that copy | the same |
| where a fixture built `Database`, a fixture that named it, in a test that named only `NewType`s of it, got a third database, and the broken copy passed | the same |
| mypy 2.4.0, pyright 1.1.414 and ty 0.0.93 passed that fixture | the same |
| one fixture, `def tx[D: Database](db: D) -> Tx[D]`, built both reagents in the sketch | the same |
| a type that held both databases hid them from purba, and its fixture built each database itself, not through the fixture of `Database` | the same |
| in `src: Tx[SourceDb]`, no agent that read blind named the fixture of `src` from the test's line alone; in `= use(tx, source)`, two of three did | [the readers' answers](https://github.com/kalonji-tools/purba/tree/30147b4/proto-700/runs/readers) |

The rows of the first prototype that do not name a type checker come from a sketch of the rules, so each is a model.

## Considered Options

- **The refusal stays, and the tester writes a type that holds both values.** Rejected. The type hides its values from purba. Its fixture also skips the fixture of `Database`.
- **Two `NewType`s of the type, each with a fixture of its own, and one generic fixture draws a reagent for each.** Rejected. A reader of `src: Tx[SourceDb]` finds its fixture only through another file.
- **A `NewType` sits on the input that the test names.** Rejected for a stock. A test reaches a stock through a reagent, so each reagent then needs a `NewType` too.
- **Fixtures builds a `NewType` through the fixture of its type.** Rejected. A fixture that names the type then gets a third value. Only a refusal of execution stops it.
- **Two fixtures, each named in a typed default of the test.** Chosen. The line names where each value comes from. A fixture names its inputs in the same way, so no fixture gets a third value.
- **Execution refuses a fixture that names one stock twice.** Rejected. Both names reach one shared stock.

## Decision Outcome

**Reach:** `crates/**` `tests/**` `/CONTEXT.md` `/LANDSCAPE.md`

**A [test](../../CONTEXT.md#test) names two values of one type by two fixtures.**

```python
@fixture
def source() -> Database: ...

@fixture
def target() -> Database: ...

@fixture
def tx[D: Database](db: D) -> Tx[D]: ...

def test_copy(src: Tx[Database] = use(tx, source), dst: Tx[Database] = use(tx, target)) -> None: ...
```

`source` and `target` build stocks.
`tx` draws a [reagent](../../CONTEXT.md#reagent) from each.
`@fixture` and `use` are sketches.
purba has no public surface for them yet.

- **One generic fixture draws the reagent from every [stock](../../CONTEXT.md#stock).** `use(tx, source)` names that fixture, and the stock that it draws from.
- **A fixture that draws a reagent takes its stock as its one parameter with no default.** `use(tx, source)` hands `tx` the stock that `source` builds.
- **A fixture names each other input as a test does.** A fixture that names `use(tx, source)` gets the reagent that the test gets.
- **A test names two values of any other type by two fixtures too.** `alice: User = use(alice)` and `bob: User = use(bob)` name two users.
- **A fixture that names one stock twice gets that stock twice.**
- **Execution refuses a fixture that names one reagent twice, as it refuses a test that does.**

**Downside:**

- **A tester writes a fixture for each value of one type.**
- **A test names `tx` again for each reagent.**
- **Execution refuses a test that names one reagent twice.** The tester must write a second fixture.

## Confirmation

No command checks this record yet.
The designs in depth of fixtures and execution, which [the landscape](../../LANDSCAPE.md#the-order-of-design) ranks, build it.
[A test returns nothing, and the first part that sees a breach of a test refuses it](a-test-returns-nothing-and-the-first-part-that-sees-a-breach-refuses-it.md#decision-outcome) says which part refuses two inputs that name one fixture.
[A test names the fixture of each input](a-test-names-the-fixture-of-each-input.md#decision-outcome) says how a test names its inputs.
[A failure's kind is what the part that first sees it observes](a-failures-kind-is-what-the-part-that-first-sees-it-observes.md#decision-outcome) names the kind of each refusal.
