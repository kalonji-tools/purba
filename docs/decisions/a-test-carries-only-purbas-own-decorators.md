# A test carries only purba's own decorators

## Context and Problem Statement

On `main` at `b82e8b6`, selection judges only a `def` with no decorator, because a decorator can change what the call takes and returns.
Execution calls whatever the module binds to the name of the test.
A person who reads one test then cannot see what runs in its place.

| fact | source |
|---|---|
| `@keep`, a decorator whose wrapper returns what the test returns, on a test that returns `False` prints `passed`, and `test_rebound = keep(lambda: "x")` prints `unknown: no assert ran` | [Which decorators may a test carry?](https://github.com/kalonji-tools/purba/issues/707), on the wheel from `main` at `b82e8b6` |
| each decorator on the module-level tests of click and loguru is the test runner's, or a name that the project gives one of the runner's decorators | [Which decorators may a test carry?](https://github.com/kalonji-tools/purba/issues/707) |

## Considered Options

- **Execution gives `unknown` to a decorated test whose call returns a value.** Rejected. The decorator still changes the test in a way that a person who reads the test cannot see.
- **Selection refuses a test that carries a decorator that is not purba's.** Chosen. A person who reads the test sees all that purba does to it.
- **A decorator of purba under a name that the tester gives it, `needs_fd_capture = skip(…)`.** Rejected. The name does not say whether purba skips the test, hands it an input, or expects it to raise.
- **A decorator of purba under an import alias, `from purba import skip as needs_fd_capture`.** Rejected. The alias does not say what purba does either.
- **A decorator of purba on a class that holds the tests.** Rejected. A person who reads one method does not see the decorator on its class.
- **A mark for the whole module, such as pytest's `pytestmark`.** Rejected. A person who reads one test does not see the mark. A run that picks its files by platform leaves out a module that cannot load there.
- **Selection refuses a test when the module binds its name again after its last `def`, or its decorator's name after its import.** Chosen. A test's name bound again is a decorator in another form. A decorator's name bound again does not say what purba does.

## Decision Outcome

**Reach:** `crates/**` `tests/**` `/CONTEXT.md`

**A [test](../../CONTEXT.md#test) carries only purba's own decorators, under purba's own names.**

- **A decorator of purba belongs to purba's [public surface](../../CONTEXT.md#public-surface).** It joins with a [milestone](../../CONTEXT.md#milestone), as [purba's public surface starts empty and has no plugin API](purbas-public-surface-starts-empty-and-has-no-plugin-api.md#decision-outcome) says.
- **Selection refuses a test that carries any other decorator, as a project error for that test alone.**
- **A test names a decorator of purba through `purba`, or through a name that the module imports from purba with no alias.** `@purba.skip`, and `@skip` after `from purba import skip`, are the only forms. After `import purba as p`, selection refuses `@p.skip`.
- **Selection refuses a test whose decorator uses a name that the module binds again after its import.** After `from purba import skip`, `skip = keep` is one such binding.
- **A decorator of purba returns the function that it takes.** The body that selection judges is then the body that runs.
- **Selection judges each `def`, under any decorator.** A refused line names the decorator, then each reason in the body.
- **Selection refuses a test whose name the module binds again after its last `def`.** The refused line names where the module binds the name again. [Python's reference](https://docs.python.org/3/reference/executionmodel.html#binding-of-names) counts a `del` as a binding.
- **purba has no decorator for a class and no mark for a module.** A module that cannot load on one platform waits for a run that picks its files by platform.

**Downside:**

- **A test cannot carry a library's decorator.** Selection refuses Hypothesis's `@given` and `mock.patch` on a test. A tester writes `with mock.patch(…)` in the body instead. A tester puts `@given` on a function that the body defines and calls.
- **A suite from pytest loses each decorator until purba carries one that does its work.**
- **Each test repeats the decorator that it shares with other tests.** click names one `skipif` for several tests.
- **Selection does not see a name bound through `globals()`, `setattr` or `from … import *`.** Execution calls what import bound.
- **A module that cannot load on one platform is a [project error](../../CONTEXT.md#project-error) there, until a run picks its files by platform.**

## Confirmation

No command checks this record yet.
[Selection hands on a decorated test unjudged](https://github.com/kalonji-tools/purba/issues/708) builds what selection refuses here.
The milestone that adds purba's first decorator builds the two forms of its name, and the refusal of that name bound again.
The designs in depth of configuration and selection, which [the landscape](../../LANDSCAPE.md#the-order-of-design) ranks, decide how a run picks its files by platform.
[A test returns nothing, and the first part that sees a breach of a test refuses it](a-test-returns-nothing-and-the-first-part-that-sees-a-breach-refuses-it.md#decision-outcome) says what selection judges in the body.
[A test that selection refuses prints before any test runs](a-test-that-selection-refuses-prints-before-any-test-runs.md#decision-outcome) says when the refused line prints.
