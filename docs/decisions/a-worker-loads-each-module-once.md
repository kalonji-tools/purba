# A worker loads each module once, and command hands each module once

## Context and Problem Statement

Dispatch hands execution one test at a time.
The tests of `test_a.py` share one module.
Each slice of the walking skeleton needs to know what crosses to dispatch, and how often a module loads.

| fact | source |
|---|---|
| pytest 9.0.3 runs the body of a test module once for its three tests | [the probe](https://github.com/kalonji-tools/purba/issues/564#issuecomment-6252106214) |
| a selected test, the rewritten text of its module and the text of each assert cross as plain data | [command lends one parse to selection and assertion](command-lends-one-parse-to-selection-and-assertion.md#decision-outcome) |

## Considered Options

- **Execution loads the module again for each test.** Rejected. The module body then runs once for each test. An outcome can then differ from pytest's.
- **Each selected test carries the rewritten text of its module.** Rejected. Each test of a module then carries a copy of the same text.
- **One load of each module in a worker, from one hand-off by command.** Chosen. The module body runs once for its tests, as under pytest. Command hands one text for each module.

## Decision Outcome

**Reach:** `crates/**` `/LANDSCAPE.md`

**A worker loads each module once, and command hands each module once.**

| question | answer |
|---|---|
| what command hands dispatch | each module once: its rewritten text, the text of each assert, and its selected tests |
| what dispatch hands execution | a function's name, and its module |
| how often execution loads a module | once in a worker. The first test of a module loads it, and the next tests use it. The worker does not load again a module that failed to load. Each later test of that module gets an error that names the module |
| where it is drawn | the sequence of one run in [the landscape](../../LANDSCAPE.md#the-parts-and-their-dependencies) |

[Execution calls a test by its function's name, and reporting prints the identity it is given](execution-calls-a-test-by-its-functions-name.md#decision-outcome) holds why execution takes a function's name.

**Downside:**

- **A worker keeps each module it loaded.** A module-level state of one test reaches the next test of that module, as it does under pytest. The test does not name that state. The cost falls on the quality explicit.
- **With many workers, an outcome can depend on the tests that ran before it in its worker.** The landscape's risk "an outcome depends on the worker" lands this cost on dispatch.

## Confirmation

`mise run test:rust` checks that a worker of execution loads a module once for two of its tests.
It checks that a module that fails to load runs its body once for two of its tests.
It checks that the next test of such a module gets an error that names the module.
It checks that dispatch hands the tests of one module to one worker.

No command checks the rest yet.
[Add purba-command, which turns the arguments into an exit code](https://github.com/kalonji-tools/purba/issues/574) builds it.
