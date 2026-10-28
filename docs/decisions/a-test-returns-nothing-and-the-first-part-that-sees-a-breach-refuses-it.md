# A test returns nothing, and the first part that sees a breach of a test refuses it

## Context and Problem Statement

[`CONTEXT.md`](../../CONTEXT.md) defines no test, so no part can refuse a function that is not one.
Selection takes each `def test…`, and execution calls it.

| fact | source |
|---|---|
| a test that runs an assert and then returns `False` prints `passed`, and an `async` test and a generator test print `unknown: no assert ran` | [the research of test interfaces, as checked again](https://github.com/kalonji-tools/purba/issues/688#issuecomment-6452745989), on the wheel from `main` at `d369fee` |
| `def test_c(x)` prints `raised: TypeError: …` | [the table of shapes](https://github.com/kalonji-tools/purba/issues/670), on the same wheel |
| `unittest`, Ward and Vitest pass a test whose body never ran | [the research of test interfaces](https://github.com/kalonji-tools/purba/blob/d69fba8fd2c6a8705829aecd9af2a19ad52695ba/research/test-interface.md) |

## Considered Options

- **Selection takes each `def test…`, and execution ignores what the call returns.** Rejected. A test that returns `False` prints `passed`, and an `async` test prints `unknown`.
- **A test takes no input until fixtures has its design in depth.** Rejected. It contradicts the design of fixtures on the day that the design merges.
- **Two inputs that name one fixture share one object, and purba allows it.** Rejected. A test that compares two inputs then compares one object with itself.
- **Execution refuses a test with two inputs that name one fixture.** Chosen. No test then gets one object under two names.
- **Execution gives `unknown` to a test that returns a value.** Rejected for a test with no decorator. Selection sees the `return` first.
- **Selection refuses a `return` of a value in a test with no decorator.** Chosen. The text shows it before any test runs.
- **A test names a shared value whose content can change.** Rejected. Each test that names it then carries the risk.
- **A test reaches a shared value whose content can change only through a fixture.** Chosen. The risk then sits in one fixture.

## Decision Outcome

**Reach:** `crates/**` `tests/**` `/CONTEXT.md` `/LANDSCAPE.md`

**A [test](../../CONTEXT.md#test) returns nothing, and the first part that sees a breach of a test refuses it.**

| a test | a breach | the part that sees it |
|---|---|---|
| names the fixture of each [input](../../CONTEXT.md#input) in `use(…)` | a parameter with no default | selection |
| | an input that fixtures cannot build | execution |
| | two inputs that name one fixture | execution |
| | an input whose fixture builds a [stock](../../CONTEXT.md#stock), which only a fixture takes | execution |
| returns nothing | a `return` of a value in the test's own body | selection |
| | a decorated test whose call returns a value, which gives `unknown` | execution |
| is not a generator | a `yield` in the test's own body | selection |
| can be `async`, and purba cannot run one yet | an `async def` | selection |

- **Selection judges only a `def` with no decorator.** A decorator can change what the call returns.
- **A `return` in a nested `def`, `lambda` or `class` belongs to it, and not to the test.**
- **Selection leaves two inputs that name one fixture to execution.** Only execution resolves the names in `use(…)`.
- **The type of a [catalyst](../../CONTEXT.md#catalyst) proves it immutable.** Fixtures reads the type before it builds the value.
- **A test reaches a stock only through a [reagent](../../CONTEXT.md#reagent) that a fixture draws from it.** That fixture undoes what the test changes, such as through one transaction that it rolls back.

**Downside:**

- **Selection refuses a `return` of a value in a branch that never runs.** The call would return nothing.
- **A decorated test reaches execution unjudged.** A decorator that hides an `async def` leaves the test `unknown`.
- **purba cannot check what a fixture does with a stock.** A fixture that undoes nothing lets a test change what the next test sees.
- **A value whose type does not prove it immutable is a stock.** click's `Command` is one, though no outcome of click depends on what a test leaves in it.

## Confirmation

`mise run test:rust` checks that selection refuses an `async def`, a `yield` and a `return` of a value.
[Selection passes a test with a typed parameter and no default](https://github.com/kalonji-tools/purba/issues/703) builds the refusal of a parameter with no default.
[A decorated test whose call returns a value prints passed](https://github.com/kalonji-tools/purba/issues/696) builds what execution sees of a decorated test.
The designs in depth of fixtures and execution, which [the landscape](../../LANDSCAPE.md#the-order-of-design) ranks, build the rest.
[A test names the fixture of each input](a-test-names-the-fixture-of-each-input.md#decision-outcome) says how a test names its inputs.
[What keeps a test from changing an input that other tests share outside its process?](https://github.com/kalonji-tools/purba/issues/692) decides which types prove a value immutable.

[A failure's kind is what the part that first sees it observes](a-failures-kind-is-what-the-part-that-first-sees-it-observes.md#decision-outcome) names the kind of each breach.
[An error stops only the work that depends on it](an-error-stops-only-the-work-that-depends-on-it.md#decision-outcome) says what each breach stops.
[Each test runs in a process of its own](each-test-runs-in-a-process-of-its-own.md#decision-outcome) says what may change around a test.
