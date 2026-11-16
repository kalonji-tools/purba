# A stand-in enters a test as an argument, so selection refuses a patch and a mock with no spec

## Context and Problem Statement

A test can replace a name that its act reaches, and then check its own [stand-in](../../CONTEXT.md#stand-in).
On `main` at `88a2c9d`, selection lets each [patch](../../CONTEXT.md#patch) and each mock run.

| fact | source |
|---|---|
| a test that patches the function its act calls passes | [Refuse a patch, and a Mock with no spec, as strict checks of the practice tier](https://github.com/kalonji-tools/purba/issues/1031) |
| a test passes that reads a misspelled attribute of the mock that a patch makes | [May a test patch a name inside its act, or does a stand-in enter only as an input?](https://github.com/kalonji-tools/purba/issues/1020#issuecomment-6918443908) |
| the standard library's `unittest.mock` gives each change that pytest's `MonkeyPatch` gives | the same |
| of the 16 lines in the test files of the 9 pinned suites that open `with …patch(`, 10 give no value | the same |
| of the 45 lines of a test file there that call `Mock`, `MagicMock`, `NonCallableMock` or `AsyncMock`, 1 gives `spec=` on the same line | the same |

## Considered Options

- **A patch passes as a setup.** `main` holds it. A test that patches the function its act calls checks only its stand-in, and passes.
- **A patch that gives a written value passes, and selection refuses only the mock that a patch makes.** The person asked to weigh it. A written value still hides a test that replaces the very function it calls.
- **A mock with no spec passes.** `main` holds it. Such a mock answers any name, so a check that reads a misspelled attribute passes.
- **A tester patches with pytest's `MonkeyPatch.context()`, or with a patch of purba's own.** The person asked whether purba should give a `MonkeyPatch` of its own on its API. Then the person said that "depending on a different test framework to import only one function is a big NO". The standard library already gives each change, so a patch of purba's own would copy `unittest.mock`.
- **Selection also reads the calls of pytest's `MonkeyPatch` as patches.** The first spec of [Refuse a patch, and a Mock with no spec, as strict checks of the practice tier](https://github.com/kalonji-tools/purba/issues/1031) held it. The spec that the person agreed drops it, since purba reads no API of another test framework.
- **A fourth tier for these checks.** The person said "I thought this was non of the three". A suppressed patch still lets each expectation read a real value, so the outcome stands on what purba checked. That puts both checks in the [practice tier](../../CONTEXT.md#practice-tier).

## Decision Outcome

**Reach:** `crates/selection/**` `tests/patch.bats` `tests/patch/**`

**Area:** selection, strictness

**Selection refuses a patch in a test's body as `patched-name`, and a mock with no spec as `mock-without-spec`, each in the practice tier.**
A stand-in enters a test as an argument.
A tester who departs from that names the departure in a [suppression](../../CONTEXT.md#suppression) with a reason.
The test then runs.

- **A patch is any of these, in the test's body, a nested `def`, `lambda` or `class` included.**
  1. A call of `unittest.mock`'s `patch`, `patch.object`, `patch.dict` or `patch.multiple`, in a `with` or not.
  2. An assignment or a `del` of an attribute or an item of an imported name, such as `lending.Library = Stub`. An augmented assignment counts too.
  3. A call of the builtin `setattr` or `delattr` on an imported name.
- **A mock with no spec is either of these.**
  1. A call of `unittest.mock`'s `Mock`, `MagicMock`, `NonCallableMock`, `NonCallableMagicMock` or `AsyncMock`. It gives no spec as its first argument, and no `spec_set`.
  2. A call of `patch` or `patch.object` that gives no value, no `spec`, no `spec_set`, no `autospec` and no `new_callable`. A `DEFAULT` gives no value. A keyword of `patch.multiple` with `DEFAULT` gives none either.
- **Selection resolves each name through the imports of the test module and of the test's body.** A local name that hides an import is no import. An attribute of any imported name counts as the target of a patch, since syntax does not tell a module from a class. A `spec`, `spec_set`, `autospec` or `new_callable` of `None` or `False` gives nothing. A call with a `*` or a `**` argument may give a spec, so selection does not count it as a mock with no spec.
- **The refused line names each target of a patch through the imports.** It reads `[patched-name] line 7 patches lending.borrow, and a stand-in enters a test as an argument`. A string gives its value. `patch.multiple` gives one target for each name it patches. A name of `patch.object` that is not a string leaves the object alone as the target.
- **The line of a mock with no spec reads `[mock-without-spec] line 7 makes a mock with no spec, and a mock has the spec of what it stands for`.** Selection gives it once for each line of the test module.
- **Selection refuses a patch that gives no value twice, once by each name.** A suppression of `patched-name` alone leaves the `mock-without-spec` refusal.
- **Selection refuses a patch of a name that the act calls as `patched-name`, like any other patch.**
- **A patch as a decorator stays `foreign-decorator`,** as [a test carries only purba's own decorators](a-test-carries-only-purbas-own-decorators.md#decision-outcome) says.
- **A replaced hook is a patch too.** Selection refuses `threading.excepthook = print` before execution sees it. With `patched-name` suppressed, execution refuses it as `hook-replaced`, which no suppression reaches.
- **A tester patches with `unittest.mock`.** purba has no patch of its own, and reads no call of another test framework.

**Downside:**

- **Selection does not see a patch or a mock in a function that the test calls.** It reads only the test's body, as it does for a private name.
- **Selection does not see a patch that a library makes,** such as freezegun's `freeze_time`, nor one in a module that the test imports.
- **Selection does not see a call that changes an imported value in place,** such as `os.environ.update(…)` or `sys.path.insert(…)`.
- **Selection does not see a patch through pytest's `MonkeyPatch`.** Neither check refuses it.
- **A tester who sets an attribute of an imported class gets a refusal,** since selection cannot tell the class from a module.
- **Selection does not read the `mock` package on PyPI, which copies `unittest.mock`.**

## Confirmation

`mise run test:rust` runs `crates/selection/tests/select.rs`.
It checks that selection refuses each form of a patch by its target.
It checks that selection refuses each call of a mock class with no spec, once for each line.
It checks that selection refuses a patch with no value twice.
It checks that a suppression of one of the two checks leaves the other.
It checks that a change to a local name, a patch that is not `unittest.mock`'s and a patch as a decorator give no `patched-name`.

`mise run test:wheel` runs `tests/patch.bats`.
It checks that the installed purba refuses a patch and a mock with no spec, and runs a suppressed patch with its warning.

[A test returns nothing, and the first part that sees a breach of a test refuses it](a-test-returns-nothing-and-the-first-part-that-sees-a-breach-refuses-it.md#decision-outcome) holds the other rules of a test's code.
