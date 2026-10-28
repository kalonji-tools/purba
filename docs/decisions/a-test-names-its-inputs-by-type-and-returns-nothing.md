# A test names its inputs by type and returns nothing

## Context and Problem Statement

[`CONTEXT.md`](../../CONTEXT.md) defines no test, so no part can refuse a function that is not one.
Selection takes each `def test…`, and execution calls it.

| fact | source |
|---|---|
| a test that runs an assert and then returns `False` prints `passed`, and an `async` test and a generator test print `unknown: no assert ran` | [the research of test interfaces, as checked again](https://github.com/kalonji-tools/purba/issues/688#issuecomment-6452745989), on the wheel from `main` at `d369fee` |
| `def test_c(x)` prints `raised: TypeError: …` | [the table of shapes](https://github.com/kalonji-tools/purba/issues/670), on the same wheel |
| only JUnit gives a test its inputs by type | [the research of test interfaces](https://github.com/kalonji-tools/purba/blob/d69fba8fd2c6a8705829aecd9af2a19ad52695ba/research/test-interface.md) |
| `unittest`, Ward and Vitest pass a test whose body never ran | the same |

## Considered Options

- **Selection takes each `def test…`, and execution ignores what the call returns.** Rejected. A test that returns `False` prints `passed`, and an `async` test prints `unknown`.
- **A test takes no input until fixtures has its design in depth.** Rejected. It contradicts the design of fixtures on the day that the design merges.
- **A test names its inputs by type.** Chosen. [The landscape](../../LANDSCAPE.md#the-quality-priorities) ranks Explicit second. The type also tells fixtures what to build.
- **Two inputs of one type share one object, and purba allows it.** Rejected. A test that compares two inputs then compares one object with itself.
- **Execution refuses a test that names two inputs of one type.** Chosen. No test then gets one object under two names.
- **Execution gives `unknown` to a test that returns a value.** Rejected for a test with no decorator. Selection sees the `return` first.
- **Selection refuses a `return` of a value in a test with no decorator.** Chosen. The text shows it before any test runs.
- **A test names a shared value whose content can change.** Rejected. Each test that names it then carries the risk.
- **A test reaches a shared value whose content can change only through a fixture.** Chosen. The risk then sits in one fixture.

## Decision Outcome

**Reach:** `crates/**` `tests/**` `/CONTEXT.md` `/LANDSCAPE.md`

**A [test](../../CONTEXT.md#test) names its [inputs](../../CONTEXT.md#input) by type, and returns nothing.**

| a test | a breach | the part that sees it |
|---|---|---|
| names each input by the type of a parameter with no default | a parameter with no default and no type | selection |
| | an input that fixtures cannot build | execution |
| | two inputs of one type | execution |
| | a [stock](../../CONTEXT.md#stock), which only a fixture takes | execution |
| returns nothing | a `return` of a value in the test's own body | selection |
| | a decorated test whose call returns a value, which gives `unknown` | execution |
| is not a generator | a `yield` in the test's own body | selection |
| can be `async`, and purba cannot run one yet | an `async def` | selection |

- **Selection judges only a `def` with no decorator.** A decorator can change what the call returns.
- **A `return` in a nested `def`, `lambda` or `class` belongs to it, and not to the test.**
- **Within one test, fixtures builds each input once.** A fixture that names an input gets the value that the test gets.
- **Selection leaves two inputs of one type to execution.** Only execution resolves an alias such as `Db = Database`.
- **The type of a [catalyst](../../CONTEXT.md#catalyst) proves it immutable.** Fixtures reads the type before it builds the value.
- **A test reaches a stock only through a [reagent](../../CONTEXT.md#reagent) that a fixture draws from it.** That fixture undoes what the test changes, such as through one transaction that it rolls back.
- **A parameter with a default, `*args` and `**kwargs` are not inputs.** purba passes nothing for them.

**Downside:**

- **Selection refuses a `return` of a value in a branch that never runs.** The call would return nothing.
- **A tester who knows pytest names each input again, by type.**
- **A decorated test reaches execution unjudged.** A decorator that hides an `async def` leaves the test `unknown`.
- **purba cannot check what a fixture does with a stock.** A fixture that undoes nothing lets a test change what the next test sees.
- **A value whose type does not prove it immutable is a stock.** click's `Command` is one, though no outcome of click depends on what a test leaves in it.

## Confirmation

No command checks this record yet.
[Selection takes an async test, a generator test and a test with an unfilled parameter as tests it can call](https://github.com/kalonji-tools/purba/issues/670) builds what selection refuses.
[A decorated test whose call returns a value prints passed](https://github.com/kalonji-tools/purba/issues/696) and [A test with a typed input prints raised](https://github.com/kalonji-tools/purba/issues/697) build what execution sees.
[A test names two values of one type by two roles](a-test-names-two-values-of-one-type-by-two-roles.md#decision-outcome) says how a test names two values of one type.
[What keeps a test from changing an input that other tests share outside its process?](https://github.com/kalonji-tools/purba/issues/692) decides which types prove a value immutable.

[A failure's kind is what the part that first sees it observes](a-failures-kind-is-what-the-part-that-first-sees-it-observes.md#decision-outcome) names the kind of each breach.
[An error stops only the work that depends on it](an-error-stops-only-the-work-that-depends-on-it.md#decision-outcome) says what each breach stops.
[Each test runs in a process of its own](each-test-runs-in-a-process-of-its-own.md#decision-outcome) says what may change around a test.
