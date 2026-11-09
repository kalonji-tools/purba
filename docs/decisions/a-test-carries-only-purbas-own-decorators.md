# A test carries only purba's own decorators

## Context and Problem Statement

On `main` at `b82e8b6`, selection judges only a `def` with no decorator, because a decorator can change what the call takes and returns.
Execution calls whatever the module binds to the name of the test.
A reader of one test then cannot see what runs in its place.

| fact | source |
|---|---|
| `@keep`, a decorator whose wrapper returns what the test returns, on a test that returns `False` prints `passed`, and `test_rebound = keep(lambda: "x")` prints `unknown: no assert ran` | [Which decorators may a test carry?](https://github.com/kalonji-tools/purba/issues/707), on the wheel from `main` at `b82e8b6` |
| each decorator on the module-level tests of click and loguru is the test runner's, or a name that the project gives one of the runner's decorators | [Which decorators may a test carry?](https://github.com/kalonji-tools/purba/issues/707) |
| a module that binds `test_rebound = keep(lambda: "x")`, `test_alias = helper` and `from os.path import join as test_join` with no `def`, beside `def test_real`, prints one line for `test_real`, then the count, and exits 0 | [A test name that the module binds with no def prints no line](https://github.com/kalonji-tools/purba/issues/811), on the wheel from `main` at `78232b7` |

## Considered Options

- **Execution gives `unknown` to a decorated test whose call returns a value.** The decorator still changes the test in a way that a reader of the test cannot see.
- **A decorator of purba under a name that the tester gives it, `needs_fd_capture = skip(…)`.** The name does not say whether purba skips the test, hands it an input, or expects it to raise.
- **A decorator of purba under an import alias, `from purba import skip as needs_fd_capture`.** The alias does not say what purba does either.
- **A decorator of purba on a class that holds the tests.** A reader of one method does not see the decorator on its class.
- **A mark for the whole module, such as pytest's `pytestmark`.** A reader of one test does not see the mark. A run that picks its files by platform leaves out a module that cannot load there.
- **Selection leaves a name bound under `global` in a function unseen, as it leaves `globals()`.** Execution then calls what the function bound, and a reader of the test cannot see it.
- **A test name that the module binds with no `def` prints no line.** A reader of the module sees a test name that holds a function, and purba prints no line for it.
- **Execution calls what the module binds to a test name with no `def`.** Selection then makes none of the checks that it makes of a `def`, such as the check of a decorator or of a `yield`.

## Decision Outcome

**Reach:** `crates/**` `tests/**` `/CONTEXT.md`

**Area:** selection

**A [test](../../CONTEXT.md#test) carries only purba's own decorators, under purba's own names.**

- **A decorator of purba belongs to purba's [public surface](../../CONTEXT.md#public-surface).** It joins with a [milestone](../../CONTEXT.md#milestone), as [purba's public surface starts empty and has no plugin API](purbas-public-surface-starts-empty-and-has-no-plugin-api.md#decision-outcome) says.
- **Selection refuses a test that carries any other decorator, as a project error for that test alone.** A reader of the test sees all that purba does to it.
- **A test names a decorator of purba through `purba`, or through a name that the module imports from purba with no alias.** `@purba.skip`, and `@skip` after `from purba import skip`, are the only forms. After `import purba as p`, selection refuses `@p.skip`.
- **Selection refuses a test whose decorator uses a name that the module binds again after its import.** A decorator's name bound again does not say what purba does. After `from purba import skip`, `skip = keep` is one such binding.
- **A decorator of purba returns the function that it takes.** The body that selection judges is then the body that runs.
- **Selection judges each `def`, under any decorator.** A refused line names the decorator, then each reason in the body.
- **Selection refuses a test whose name the module binds again after its last `def`, or under `global` in a function wherever it stands.** A test's name bound again is a decorator in another form. Selection already refuses a `yield` in a branch that never runs, so a binding that may never run counts too. The refused line names where the module binds the name again. [Python's reference](https://docs.python.org/3/reference/executionmodel.html#binding-of-names) counts a `del` as a binding.
- **Selection refuses a test name that the module binds with no `def`, where a binding of it can hold a function.** A binding that can hold a function is a decorator in another form too. At the module's level, an assignment of a literal, a comprehension or a generator expression to the name alone never holds a function. Nor does an `import` with no `from`, a `class` with no decorator or a `del`, so a name that only these bind prints no line. The refused line names each binding that can hold a function, and sits at the first of them.
- **Selection refuses a test whose `def` sits under a statement at the module's level, unless a `def` at the top binds the name.** The refused line names the statement, and says that purba cannot see whether import binds the test.
- **purba has no decorator for a class and no mark for a module.** A module that cannot load on one platform waits for a run that picks its files by platform.

**Downside:**

- **A test cannot carry a library's decorator.** Selection refuses Hypothesis's `@given` and `mock.patch` on a test. A tester writes `with mock.patch(…)` in the body instead. A tester puts `@given` on a function that the body defines and calls.
- **A suite from pytest loses each decorator until purba carries one that does its work.**
- **Each test repeats the decorator that it shares with other tests.** click names one `skipif` for several tests.
- **Selection does not see a name bound through `globals()`, `setattr` or `from … import *`.** Execution calls what import bound.
- **Selection refuses a test name that the module imports with `from`, whatever the name holds.** A tester who writes `from helpers import test_user` writes `from helpers import test_user as user` instead.
- **Selection refuses a test name that holds a value it cannot see, such as `test_dir = os.path.dirname(__file__)`.** The count names it as a test. Selection keeps a list of the forms that never hold a function, and refuses each form that the list leaves out.
- **A module that cannot load on one platform is a [project error](../../CONTEXT.md#project-error) there, until a run picks its files by platform.**

## Confirmation

`mise run test:rust` checks that selection refuses a test that carries a decorator that is not purba's, and a test whose name the module binds again.
`mise run test:rust` checks that selection refuses a test whose `def` sits under `if`, `try`, `with`, `for`, `while` or `match`.
`mise run test:rust` checks that selection refuses a test name that the module binds with no `def`.
`mise run test:wheel` checks the line of such a test on the installed `purba`.
`mise run test:rust` and `mise run test:wheel` check that a test name that never holds a function gives no line.

The milestone that adds purba's first decorator builds the two forms of its name, and the refusal of that name bound again.
The designs in depth of configuration and selection, which [the landscape](../../LANDSCAPE.md#the-order-of-design) ranks, decide how a run picks its files by platform.
[A test returns nothing, and the first part that sees a breach of a test refuses it](a-test-returns-nothing-and-the-first-part-that-sees-a-breach-refuses-it.md#decision-outcome) says what selection judges in the body.
[A test that selection refuses prints before any test runs](a-test-that-selection-refuses-prints-before-any-test-runs.md#decision-outcome) says when the refused line prints.
