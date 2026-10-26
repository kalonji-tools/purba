# A module whose load fails leaves sys.modules as it found it

## Context and Problem Statement

[A loaded module joins sys.modules under its file's name, without .py](a-loaded-module-joins-sys-modules-under-its-files-name.md#decision-outcome) puts the module there before its body runs.
Until this record, the module stayed there when a later step of load raised.
A run takes one module.
Execution keeps the failed load in its own map, so purba never reads the entry again.
Only code that the module started before it raised, such as a thread or an `atexit` hook, still reads the entry.
A worker that loads more than one module, or a test's folder on `sys.path`, lets more code read the entry.

| what later code that reads the name gets | keep | remove | put back | `None` |
|---|---|---|---|---|
| a new name | the half-made module, with no error | `ModuleNotFoundError` | `ModuleNotFoundError` | the import halts |
| a new name, its folder on `sys.path` | the half-made module | the file runs again, and raises again | the file runs again, and raises again | the import halts |
| `abc.py`, which shadows a module Python loads at startup | the test module, so `abc.ABCMeta` raises `AttributeError` | a second `abc`, whose `ABCMeta` differs from the one at startup | the `abc` of startup | each import of `abc` halts |
| a body that calls `sys.exit(3)` | the module | no entry | no entry | `None` |

[The prototype](https://github.com/kalonji-tools/purba/issues/658#issuecomment-6421294717) gave each row on a wheel built from `4f572c2` with the prototype's switch.
purba printed the same lines and the same exit code under each choice.

## Considered Options

- **Keep the module under its name.** Rejected. A later import gets a half-made module, and no error.
- **Remove the name.** Rejected. A failed module that shadowed a loaded module makes the next import load a second copy, whose classes differ from the first. Once assertion refuses a module named after a loaded module, as [a failure's kind is what the part that first sees it observes](a-failures-kind-is-what-the-part-that-first-sees-it-observes.md#decision-outcome) decides, remove gives what put back gives.
- **Put `None` under the name.** Rejected. Each later import of a shadowed name halts for the rest of the run.
- **Leave it to the design in depth of execution alone.** Rejected. Until that design, a later import gets a half-made module.
- **Put back what the name held.** Chosen. A new name gives what [`importlib`](https://github.com/python/cpython/blob/v3.12.14/Lib/importlib/_bootstrap.py#L936-L941) gives, and a shadowed name keeps its module. oxitest chose it after a half-made module broke later imports, in [`70fbc27a`](https://github.com/kalonji-tools/oxitest/commit/70fbc27a).

## Decision Outcome

**Reach:** `crates/assertion/**`

**A module whose load fails leaves `sys.modules` as it found it.**

- **Load puts back the module that the name held before.** A name that held nothing leaves `sys.modules`.
- **This holds for any exception after the module joins `sys.modules`, `SystemExit` included.**
- **The design in depth of execution decides which part puts the entry back, if any, once a worker loads many modules.**

**Downside:**

- **A later import runs the file again, without purba's rewrite, once the test's folder is on `sys.path`.** Code that imports the module then sees its body run twice, as under `importlib`.
- **Load puts back only the entry under the module's name.** A module that the body imported before it raised stays in `sys.modules`, as under `importlib`.

## Confirmation

`mise run test:rust` checks that a body that raises leaves no entry under a new name.
It checks that a shadowed name holds its old module after the body raises.
It checks that a body that calls `sys.exit()` leaves no entry.

[How does a test module import a module beside it, or the package it belongs to?](https://github.com/kalonji-tools/purba/issues/637) decides when a test's folder is on `sys.path`.
[Once a worker loads many modules, which part puts back what sys.modules held after a load fails?](https://github.com/kalonji-tools/purba/issues/669) holds that decision.
