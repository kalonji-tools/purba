# A loaded module joins sys.modules under its file's name, without .py

## Context and Problem Statement

Assertion's load makes a module from the rewritten text, as [the skeleton crosses the seam in execution and assertion](the-skeleton-crosses-the-seam-in-execution-and-assertion.md#decision-outcome) states.
It runs the text through `compile()` and `exec()`.
`importlib` does not run, so nothing puts the module in `sys.modules`.
`dataclasses`, `pickle`, `typing` and `inspect` look a module up there by its `__name__`.

| a test module that | out of `sys.modules` | in `sys.modules` |
|---|---|---|
| defines a `@dataclass` under `from __future__ import annotations` | `AttributeError` as it loads | loads |
| defines a `@dataclass` with plain annotations | loads | loads |
| pickles an instance of its own class | `PicklingError` | pickles |
| pickles its own function | `PicklingError` | pickles |
| calls `typing.get_type_hints` on a class with a forward reference | `NameError` | gives the hints |
| calls `inspect.getmodule` on its own function | `None` | the module |

[The probe](https://github.com/kalonji-tools/purba/issues/570#issuecomment-6311718092) gave the same results on Python 3.12.14 and 3.14.7.

## Considered Options

- **Keep the module out of `sys.modules`.** Rejected. A module with a `@dataclass` under `from __future__ import annotations` does not load. A test that pickles its own class or function fails.
- **Put the module in `sys.modules` under its file's name, without `.py`, before its body runs.** Chosen. Each row of the probe passes. The cost is one call before `exec()`.

## Decision Outcome

**Reach:** `crates/**`

**A loaded module joins `sys.modules` under its file's name, without `.py`, before its body runs.**

That name is also the module's `__name__`, so `dir/test_a.py` loads as `test_a`.

**Downside:**

- **A module that shares its name with a module already loaded replaces it in `sys.modules`.** A test module named `test.py` replaces the standard library's `test` package there. A later `import test` then gets the test module.
- **Two modules with one name share one key.** A run of the skeleton takes one module, so two never meet. The design in depth of execution decides the key for a worker that loads many.
- **A module that raises while it loads stays in `sys.modules`.** [`importlib`](https://github.com/python/cpython/blob/v3.12.14/Lib/importlib/_bootstrap.py#L936-L941) removes it. The design in depth of errors decides what purba does.

## Confirmation

`mise run test:rust` checks that a loaded module is in `sys.modules` as its body runs.
It also checks that a `@dataclass` under `from __future__ import annotations` loads.
It checks that an instance of that class pickles.
