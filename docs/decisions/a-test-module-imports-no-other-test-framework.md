# A test module imports no other test framework, so selection refuses an import of pytest

## Context and Problem Statement

A test module can import pytest beside purba.
On `main` at `1dbae44`, selection lets such a module run, and its tests pass.

| fact | source |
|---|---|
| an import of pytest changes no hook, filter or handler that purba guards | [May a test module import another test framework, such as pytest?](https://github.com/kalonji-tools/purba/issues/1030#issuecomment-6928107634) |
| code can name a module in 45 ways that the research probes, 5 of which import nothing, and no single check sees every way | [the research of the refusal](https://github.com/kalonji-tools/purba/blob/f62081253d3eab303cab37779ba9a6a2ea1dac01/research/unwanted-import.md) |
| of the 189 test files of the 9 pinned suites, 119 import pytest, each on one line at the module's level | [the prototype of the refusal](https://github.com/kalonji-tools/purba/blob/ab88b2287570d8b8c68d75cb71a3611268792ee4/prototypes/1040-import-scan/runs/scan.txt) |
| of those 189 files, none imports pytest in a test, in a helper or under `TYPE_CHECKING`, and no call names it in a literal | the same |
| no test file of the 9 pinned suites imports nose, nose2, testtools or ward | [May a test module import another test framework, such as pytest?](https://github.com/kalonji-tools/purba/issues/1030#issuecomment-6928107634) |

## Considered Options

- **A test module that imports pytest passes.** `main` holds it. On [May a test patch a name inside its act, or does a stand-in enter only as an input?](https://github.com/kalonji-tools/purba/issues/1020#issuecomment-6918443908), the person said that "depending on a different test framework to import only one function is a big NO".
- **The refused line states the import alone, `line 1 imports pytest, a test framework that is not purba`.** [May a test module import another test framework, such as pytest?](https://github.com/kalonji-tools/purba/issues/1030#issuecomment-6928107634) chose it. It names no rule, as the lines of `computed-expected`, `patched-name` and `mock-without-spec` do.
- **Selection reads the import lines alone.** The same comment chose it. The research finds a call that names pytest in a literal, which the text of the module shows too.

## Decision Outcome

**Reach:** `crates/selection/**` `tests/framework.bats` `tests/framework/**`

**Area:** selection, strictness

**Selection refuses an import of pytest in a test module as `foreign-framework`, a check of the [practice tier](../../CONTEXT.md#practice-tier).**
A test module imports no other test framework.
A tester who departs from that names the departure in a [suppression](../../CONTEXT.md#suppression) with a reason.
The test then runs.

- **The target is `pytest` or `_pytest`, or a module under either.** Hypothesis and freezegun are libraries that a test calls, and no pinned suite imports another test framework. Selection allows `unittest` and `unittest.mock`, since [a stand-in enters a test as an argument](a-stand-in-enters-a-test-as-an-argument.md#decision-outcome) chose `unittest.mock`.
- **An import line names the target.** `import pytest`, `import pytest as pt`, `from pytest import *` and `from _pytest.monkeypatch import MonkeyPatch` are each one.
- **A known call imports the target when its first argument is a string that names it.**
  - The calls are `__import__`, `importlib.__import__`, `importlib.import_module`, `importlib.util.find_spec`, `pkgutil.resolve_name`, `runpy.run_module`, `importlib.resources.files` and `unittest.mock.patch`.
  - Selection resolves the called name through the imports, as it does for a patch. A name is a builtin when no import binds it, and neither the test nor the module binds it.
  - `find_spec` and `patch` import only the parent of a dotted name, so selection counts only a dotted name for them.
- **An import outside every test reaches every test of the module.** The import can sit at the module's level, in a block at that level, under `if TYPE_CHECKING:`, or in a helper or a class. It can also sit in a decorator, a default or an annotation of a `def` at the module's level. Each of them runs when the `def` is made. A test in a class is one of every test. An import in the body of a [test](../../CONTEXT.md#test) reaches that test only.
- **The refused line names the module as the code writes it.** It reads `[foreign-framework] line 5 imports pytest, and a test module imports no other test framework`. An import line that names two modules gives two reasons, and a module twice on one line gives one. The refused line gives each such reason in line order, among the other reasons of the test.
- **A patch whose target names pytest gives both `patched-name` and `foreign-framework`.**

**Downside:**

- **Selection reads the text of the test module only.** It does not see a computed name, or a call that imports and is not on the list. Nor does it see a module that the test module imports and that imports pytest. Nor does it see pytest when a module such as `sitecustomize` loaded it before the test. The research names each way, and the check that would see it.
- **Selection refuses an import under `if TYPE_CHECKING:`, though that import never runs.**
- **An import in a helper that no test calls blocks every test of the module.**
- **A suite that still runs under pytest needs a suppression in each module that imports pytest.**

## Confirmation

`mise run test:rust` runs `crates/selection/tests/select.rs`.
It checks that each import line outside a test reaches every test, in line order.
It checks that `unittest` gives no reason.
It checks that a call in a helper, and a call in a decorator or a default of a `def`, reach every test.
It checks that an import line or a known call in a test reaches that test only.
It checks that a module twice on one line gives one reason.

It checks that `find_spec` and `patch` of a name with no dot give no `foreign-framework`.
It checks that a call whose name another module or the module itself binds, and `pytest_mock`, give no reason.
It checks that a suppression of `foreign-framework` lets the test run, with each import in its warning.

`mise run test:wheel` runs `tests/framework.bats`.
It checks the refused lines and a suppressed import on the installed purba.

[A test carries only purba's own decorators](a-test-carries-only-purbas-own-decorators.md#decision-outcome) refuses a decorator of pytest.
[May a test module import another test framework, such as pytest?](https://github.com/kalonji-tools/purba/issues/1030) chose to refuse the import.
