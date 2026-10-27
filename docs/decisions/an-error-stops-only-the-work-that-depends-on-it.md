# An error stops only the work that depends on it

## Context and Problem Statement

[A failure's kind is what the part that first sees it observes](a-failures-kind-is-what-the-part-that-first-sees-it-observes.md#decision-outcome) names each kind, and the part that first sees it.
It does not say what each kind stops.
A run that stops too much hides outcomes that the tester needs.
A run that stops too little runs tests on a wrong input.

| fact | source |
|---|---|
| pytest 9.0.3 runs no test when one module raises at import, and exits 2 | [the probe of pytest](https://github.com/kalonji-tools/purba/issues/608#issuecomment-6318260548) |
| oxitest reported "every bad Target in the invocation … rather than only the first" | [oxitest's record of target validation](https://github.com/kalonji-tools/oxitest/blob/731c2b3f76c708125eb4984d9725652208f5ae81/docs/adr/0014-target-validation.md) |
| a check that stops at the first violation "would reintroduce the run-fix cycle per offender" | [oxitest's registrar](https://github.com/kalonji-tools/oxitest/blob/731c2b3f76c708125eb4984d9725652208f5ae81/python/tests/test_module_source_registrar.py#L546-L548) |
| on a wheel built from `9e35ee0d6d5913dae931ea8473e06a2478c87ffd`, a panic inside a test ends the run with exit 3, and no test prints a line | [How does a test cause a defect inside purba during a run?](https://github.com/kalonji-tools/purba/issues/657#issuecomment-6415193768) |

## Considered Options

- **A run stops at the first failed test, as pytest's `-x` does.** Rejected. It answers a different question: when to stop a run that works.
- **Every error stops the run, as pytest does for a module that raises at import.** Rejected. One broken module hides the outcome of every other test.
- **An error stops only the work that depends on it.** Chosen. Every test that can still run gives its outcome, and no test runs on a wrong input.
- **A part names the first error, and stops.** Rejected. The tester then fixes one error for each run, as oxitest's registrar found.
- **An internal error stops the run.** Rejected. A panic inside one test costs the printed line of every test.
- **The rule is an invariant of the landscape.** Rejected. [An invariant needs no design of its own](an-invariant-needs-no-design-of-its-own.md#decision-outcome) says that a rule that a concern states is not an invariant.

## Decision Outcome

**Reach:** `crates/**`

**An error stops only the work that depends on it.**

| kind | it stops | for example |
|---|---|---|
| [invocation error](../../CONTEXT.md#invocation-error) | the run | a missing path, or a closed stdout that command sees at start |
| [project error](../../CONTEXT.md#project-error) | the tests that depend on it | a module that does not load stops its tests, and a test that purba cannot call stops itself |
| [internal error](../../CONTEXT.md#internal-error) | the work that depends on what failed | a panic inside a test stops that test, a panic in selection while it reads `test_b.py` stops the tests of `test_b.py`, and a panic in command or reporting stops the run |
| [outcome](../../CONTEXT.md#outcome) | nothing | |

- **An error shows in the earliest part that can see it.** No part does work outside the selection only to see an error sooner.
- **Before a part stops, it names every error that it finds.** Parsing decides in its design in depth how many syntax errors it reports for one file.
- **The error of a file that does not parse shows, though the query selects no test in it.** Selection cannot know whether the file holds a selected test.
- **An error that no part sees before a write shows at the write that fails.** A full disk is one.

**Downside:**

- **The work that goes on after a panic trusts that the panic broke no state that another part reads.** The `UnwindSafe` bound of `catch_unwind` checks some of it. `AssertUnwindSafe` opts out of that check.
- **An invocation error that reporting finds discards every outcome of the run.** Each test ran, and the tester reads no outcome.
- **A tester who knows pytest expects a module that raises at import to stop the run.** purba runs every other module.
- **A runtime that puts a file that takes a write on fd 1 hides a closed stdout from purba.** macOS logs at each start of the free-threaded Python from python.org, because its `Python.app` lists an executable it does not hold. The log write opens a socket on the lowest free fd, which is then fd 1. purba writes its lines into that socket.

## Confirmation

`mise run test:rust` checks that command stops the run before any test when it takes an error in place of stdout.
No command checks the rest of this record.
[Selection takes an async test, a generator test and a test with an unfilled parameter as tests it can call](https://github.com/kalonji-tools/purba/issues/670) and [A file that does not parse shows when assertion loads it, and not when parsing reads it](https://github.com/kalonji-tools/purba/issues/671) build some of it.
[Wayfinder: purba's errors, telling each kind of failure apart from the first part that sees it](https://github.com/kalonji-tools/purba/issues/652) holds the rest.
Its Destination promises what a panic inside a test stops.
[Does a module that failed to load give one error, or one for each of its tests?](https://github.com/kalonji-tools/purba/issues/659) asks what each test of a module that does not load prints.
