# purba ranks an honest outcome first

## Context and Problem Statement

A [tester](../../CONTEXT.md#tester) needs a test run that tells the truth about their code and is fast enough to run often.
pytest is the most used Python test framework.
The backward compatibility that its users depend on holds its core.
Its collection is import, so a selection of ten tests still imports the whole suite.

Python lets a test rebind almost any name.
In evaluations, agents that write tests change the test instead of the code.
So a run can go green and check nothing, and its reader may not notice.

| fact | source |
|---|---|
| pytest is the choice of 50% to 53% of respondents, unittest of 23% to 25% | Python Developers Survey [2021](https://lp.jetbrains.com/python-developers-survey-2021/), [2022](https://lp.jetbrains.com/python-developers-survey-2022/), [2023](https://lp.jetbrains.com/python-developers-survey-2023/), [2024](https://lp.jetbrains.com/python-developers-survey-2024/) |
| pytest's strict options "are off by default for compatibility or other reasons" | [pytest good practices](https://docs.pytest.org/en/stable/explanation/goodpractices.html) |
| subtests were proposed in 2016 and shipped in pytest 9.0 on 2025-11-05 | [Support for Python 3.4 unittest subtests](https://github.com/pytest-dev/pytest/issues/1367), [pytest changelog](https://docs.pytest.org/en/stable/changelog.html) |
| 34,891 tests took 197.54 s to collect | [Would there be any interest if I rewrote the test collector in go/rust?](https://github.com/pytest-dev/pytest/issues/12813) |
| a builtin name can be rebound in a module | [Python execution model](https://docs.python.org/3/reference/executionmodel.html) |
| only `None`, `True`, `False` and `__debug__` cannot be reassigned | [Python built-in constants](https://docs.python.org/3/builtins/constants.html) |
| agents change the test instead of the code, in evaluations built to provoke it | [ImpossibleBench](https://arxiv.org/abs/2510.20270), [Natural Emergent Misalignment from Reward Hacking in Production RL](https://arxiv.org/abs/2511.18397) |

## Considered Options

- **Fast feedback first.** Rejected. A fast run that lies costs the tester more than a slow run that tells the truth.
- **Migration fidelity first, with each pytest suite run under purba unchanged.** Rejected. An unchanged suite keeps each test that checks nothing.
- **An honest outcome first, then explicit, legible result, reviewable core and fast feedback.** Chosen. Each step of the rank decides one collision.

## Decision Outcome

**Reach:** `/LANDSCAPE.md`

**purba ranks five qualities, and an honest outcome comes first.**
[The landscape](../../LANDSCAPE.md#the-quality-priorities) states each quality.

| rank | quality | it wins over the next one when |
|---|---|---|
| 1 | honest outcome | a strict check costs a migrated suite a rewrite: the check stays |
| 2 | explicit | the report would have to guess what an unnamed fixture held: it does not guess |
| 3 | legible result | the part that writes the report must grow to be legible: it may grow |
| 4 | reviewable core | a shortcut across a part boundary would save time: no shortcut |
| 5 | fast feedback | |

- **Type first is part of the explicit quality.** A type is how a test names an input.
- **Migration fidelity does not make purba a drop-in replacement for pytest.** A [candidate's](../../CONTEXT.md#candidate) tests do not run under purba unchanged.
  Its [desired-state suite](../../CONTEXT.md#desired-state-suite) restates the tests.
  Each test name keeps the outcome of a pytest control run.
  That match proves an honest outcome.
- **Readable test code comes from the explicit quality and from strictness.** It is not a quality of its own.
- **Lazy work is a mechanism of fast feedback, and not a quality.** It is an invariant, as [a concern is what no part can own](a-concern-is-what-no-part-can-own.md#decision-outcome) says.
- **A scenario uses a count of imports, not a time.** A count gives the same answer on any machine.
- **An agent reads the result, as a person does.** [Decide the distilled feature set](https://github.com/kalonji-tools/purba/issues/10) gave machine-readable output no design, and this reverses that.

**Downside:**

- **A pytest suite needs more change to migrate.** A pytest suite takes fixtures from `conftest.py` by name, and the explicit quality refuses that. The desired-state suite restates each one.
- **An upstream test that checks nothing needs a restatement.** It passes under pytest and is not a pass under purba. The desired-state suite gives it an expectation under the same name.
- **Speed loses every collision.** purba serves the tester who wants speed above all else last.
- **The result for an agent needs a design.** The distilled feature set planned none for machine-readable output.

## Confirmation

No command checks the rank, because [a standard a gate cannot decide does not become a gate](a-standard-a-gate-cannot-decide-does-not-become-a-gate.md).
Each scenario in [the landscape](../../LANDSCAPE.md#the-driver-scenarios) becomes a test when the parts it names exist.
No test runs one yet.
The walking skeleton is the first to run one.

[The landscape](../../LANDSCAPE.md#the-invariants) states lazy work.
[An invariant needs no design of its own](an-invariant-needs-no-design-of-its-own.md#decision-outcome) holds why.
[A pinned candidate decides when a milestone is done](a-pinned-candidate-decides-when-a-milestone-is-done.md#decision-outcome) holds how the person approves a desired-state suite.
