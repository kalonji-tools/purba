# A test names two values of one type by two roles

## Context and Problem Statement

Execution refuses a test that names two inputs of one type, as [a test names its inputs by type and returns nothing](a-test-names-its-inputs-by-type-and-returns-nothing.md#decision-outcome) decides.
A test of a copy needs two databases: the one that it copies from, and the one that it copies to.

| fact | source |
|---|---|
| with one value for each type, a copy that copies nothing passed its test | [the prototype](https://github.com/kalonji-tools/purba/blob/010dda4/proto-690/RESULTS.md) |
| each way that gave the test two objects failed that copy | the same |
| where a fixture built `Database`, a fixture that named it, in a test that named only roles of it, got a third database, and the broken copy passed | the same |
| mypy 2.4.0, pyright 1.1.414 and ty 0.0.93 passed that fixture | the same |
| with a fixture for each role and none for `Database`, the sketch refused that fixture, because nothing built a `Database` | the same |
| `Tx[SourceDb] == Tx[TargetDb]` is `False`, on Python 3.12 and 3.14 | the same |
| one fixture, `def tx[D: Database](db: D) -> Tx[D]`, built both reagents in the sketch | the same |
| a type that held both databases hid them from purba, and its fixture built each database itself, not through the fixture of `Database` | the same |
| at runtime `type SourceDb = Database` is unequal to `Database`, and mypy, pyright and ty read it as `Database` | the same |

The rows about Python and the type checkers report a measurement.
The other rows come from a sketch of the rules in [a test names its inputs by type and returns nothing](a-test-names-its-inputs-by-type-and-returns-nothing.md#decision-outcome), so each is a model.

## Considered Options

- **The refusal stays, and the tester writes a type that holds both values.** Rejected. The type hides its values from purba. Its fixture also skips the fixture of `Database`.
- **A role sits on the input that the test names.** Rejected for a stock. A test reaches a stock through a reagent, so each reagent then needs a role too.
- **A role sits on the stock, and one generic fixture draws a reagent for every role.** Chosen. A tester writes each role once.
- **Fixtures builds a role through the fixture of its type.** Rejected. A fixture that names the type then gets a third value. Only a refusal of execution stops it.
- **Each role has its own fixture.** Chosen. Where no fixture builds the type, a fixture that names it fails with no rule of its own.
- **Execution refuses a fixture that names one stock twice.** Rejected. Both names reach one shared stock.

## Decision Outcome

**Reach:** `crates/**` `tests/**` `/CONTEXT.md` `/LANDSCAPE.md`

**A [test](../../CONTEXT.md#test) names two values of one type by two [roles](../../CONTEXT.md#role).**

```python
SourceDb = NewType("SourceDb", Database)
TargetDb = NewType("TargetDb", Database)

@fixture
def source() -> SourceDb: ...

@fixture
def target() -> TargetDb: ...

@fixture
def tx[D: Database](db: D) -> Tx[D]: ...

def test_copy(src: Tx[SourceDb], dst: Tx[TargetDb]) -> None: ...
```

`source` and `target` build stocks.
`tx` draws a [reagent](../../CONTEXT.md#reagent) from each.
`@fixture` is a sketch.
purba has no public surface for it yet.

- **A role is a `NewType` of a type.** A `type` alias, such as `type SourceDb = Database`, is the type itself.
- **Each role has its own fixture.** Fixtures never builds a role through the fixture of its type.
- **The role of a [stock](../../CONTEXT.md#stock) sits on the stock.** A test names a reagent of the role.
- **One generic fixture draws the reagent for every role of a stock.**
- **A test names the role of any other input directly.** `alice: Alice` and `bob: Bob` name two roles of `User`.
- **A fixture that names one stock twice gets that stock twice.**
- **Execution refuses a fixture that names one reagent twice, as it refuses a test that does.**
- **Execution refuses a fixture that names a type, where the test names only roles of that type.**

**Downside:**

- **A tester writes a fixture for each role.**
- **No type checker sees a fixture that names a type in place of its role.** Only execution refuses it.
- **A value does not record its role.** purba trusts the return type of its fixture.
- **Execution refuses a test that names two reagents of one role.** The tester must declare a second role.

## Confirmation

No command checks this record yet.
The designs in depth of fixtures and execution, which [the landscape](../../LANDSCAPE.md#the-order-of-design) ranks, build it.
[A test names its inputs by type and returns nothing](a-test-names-its-inputs-by-type-and-returns-nothing.md#decision-outcome) says which part refuses two inputs of one type.
[What does a tester write to name an input and its role, so that a test's signature reads at a glance?](https://github.com/kalonji-tools/purba/issues/700) decides whether the syntax of the example stays.
[A failure's kind is what the part that first sees it observes](a-failures-kind-is-what-the-part-that-first-sees-it-observes.md#decision-outcome) names the kind of each refusal.
