# A module that binds a name purba adds is a project error

## Context and Problem Statement

Assertion puts a hook in each module that it loads.
Each marked assert calls the hook.
The hook shares the module's namespace with the tester's own names, so a name of the tester can replace it.

| fact | source |
|---|---|
| assertion puts `_purba_ran` in the module's namespace, beside the tester's own names | [`assertion`](https://github.com/kalonji-tools/purba/blob/f7c521b/crates/assertion/src/lib.rs#L199) |
| purba prints `unknown: no assert ran` for a module that defines `_purba_ran` and then fails an assert, and exits 2 | [How does purba keep its assert mechanics out of the reach of the tester's code?](https://github.com/kalonji-tools/purba/issues/660#issuecomment-6439623735), on a wheel from `f7c521b` |
| a check of the hook after the body catches a `def`, a `def` with a full-width letter, a `del`, an `import *` and a `globals().update` | [How does purba keep its assert mechanics out of the reach of the tester's code?](https://github.com/kalonji-tools/purba/issues/660#issuecomment-6444973004), on CPython 3.12, 3.14 and 3.14t |
| a check of the hook after each test catches a test that rebinds it | [How does purba keep its assert mechanics out of the reach of the tester's code?](https://github.com/kalonji-tools/purba/issues/660#issuecomment-6444973004), on CPython 3.12, 3.14 and 3.14t |
| a code object holds the local names of its function in `co_varnames`, `co_cellvars` and `co_freevars` | [the data model of Python](https://docs.python.org/3.12/reference/datamodel.html#code-objects) |
| Python normalises each name with NFKC when it compiles the module | [the reference of Python](https://docs.python.org/3.12/reference/lexical_analysis.html#identifiers) |
| execution gives a test an error when its call returns one that is not a failed assert | [`execution`](https://github.com/kalonji-tools/purba/blob/f7c521b/crates/execution/src/lib.rs#L207-L217) |

## Considered Options

- **A name that the module never writes.** Rejected. It needs a walk of every name, and a name that command, dispatch and execution carry to the load. It misses `import *` and `globals().update(…)`.
- **A module that writes `_purba_ran` is an error that parsing shows.** Rejected. Parsing sees only the text, and `import *` and `globals().update(…)` bind the name when the module runs.
- **A check of the hook in assertion, at the load and after each test.** Chosen. It stays inside assertion, and it sees what the module binds when it runs.

## Decision Outcome

**Reach:** `crates/assertion/**`

**A module that binds a name purba adds is a project error.**

- **The rule holds for each name that a rewrite of purba adds to a module,** `_purba_ran` and each helper that a later rewrite adds.
- **Before the body of the module runs, the load checks that no function holds such a name as a local name.**
- **After the body runs, the load checks that each such name still holds what purba put there.**
- **When either check fails, the load fails.** The error is `NameError: <file> binds _purba_ran, the name of purba's assert hook`.
- **After each test, assertion checks the hook again.** When the hook changes, that test has the outcome `raised`, with that `NameError`. Assertion then puts the hook back for the next test.

**Downside:**

- **A test module may not bind `_purba_ran`.** The name is not on the [public surface](../../CONTEXT.md#public-surface), and only the error names it.
- **A binding at the level of the module shows after the body ran.** The effects of the body stay.
- **A test that calls the hook on purpose can still forge a pass.** [A failure's kind is what the part that first sees it observes](a-failures-kind-is-what-the-part-that-first-sees-it-observes.md#decision-outcome) names that limit.
- **A class body that binds `_purba_ran`, and asserts in that body, still forges a pass.** The name is an attribute of the class, and neither check reads it.
- **A test that rebinds the hook gives its error as the result of its call.** It prints as `raised`, and not as a project error.

## Confirmation

`mise run test:rust` checks that a module that binds `_purba_ran` fails to load with that `NameError`.
It checks a `def`, a full-width `def`, a `del`, an `import *` and a `globals().update`.
It checks a local name, a parameter, a loop target, a full-width local name and a closure.
It checks that a test that rebinds `_purba_ran` gives that `NameError`, even when its assert fails.
It checks that the next test finds the hook back.
It checks, through command, that purba prints the error for a module that defines `_purba_ran`, and exits 4.
