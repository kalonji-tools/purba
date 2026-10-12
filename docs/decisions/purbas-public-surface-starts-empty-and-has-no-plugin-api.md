# purba's public surface starts empty and has no plugin API

## Context and Problem Statement

A tester writes each test file against purba, so each name that purba exposes becomes a promise.
[Decide the distilled feature set](https://github.com/kalonji-tools/purba/issues/10#issuecomment-5593350290) settled the surface and refused a plugin API.
No record holds its reasons.
That issue found that xdist and pytest-asyncio are plugins only because pytest's core refuses to grow.
A runner that refuses plugins must say how an integration reaches a test.

| fact | source |
|---|---|
| 2,149 `pytest-*` packages exist | [Decide the distilled feature set](https://github.com/kalonji-tools/purba/issues/10#issuecomment-5593350290) |
| xdist is in 15.2% of pytest installs, and pytest-asyncio in 27.6% | [Decide the distilled feature set](https://github.com/kalonji-tools/purba/issues/10#issuecomment-5593350290) |
| a pytest maintainer: "About any kind of external extension to pytest, in any language, is at principle a no-go for the pytest core" | [Decide the distilled feature set](https://github.com/kalonji-tools/purba/issues/10#issuecomment-5593350290) |
| no named third-party project was found that would prove a plugin API | [Decide the distilled feature set](https://github.com/kalonji-tools/purba/issues/10#issuecomment-5593350290) |
| an up-front scope written as paths and nodeids was refused | [Distinguish a milestone from a feature](https://github.com/kalonji-tools/purba/issues/16#issuecomment-5561583582) |
| the extension module added no name, and `pyproject.toml` declared no command | [`src/lib.rs`](https://github.com/kalonji-tools/purba/blob/c215783c48e04543ba502ec2ea8d80f2a8188441/src/lib.rs#L15-L19), [`pyproject.toml`](https://github.com/kalonji-tools/purba/blob/c215783c48e04543ba502ec2ea8d80f2a8188441/pyproject.toml) |

## Considered Options

- **A plugin API, as pytest's hooks.** Rejected. A hook runs for tests that never named it, so it breaks the [explicit quality](../../LANDSCAPE.md#the-quality-priorities).
  It also breaks the invariant that [the diagram is the only wiring](../../LANDSCAPE.md#the-invariants).
  purba carries what xdist and pytest-asyncio do.
  [purba divides by responsibility and not by language](purba-divides-by-responsibility-and-not-by-language.md#decision-outcome) puts parallel runs in dispatch, and makes async a property of execution.
- **A first surface decided up front, as oxitest's `__all__`.** Rejected. It writes down names that no candidate asked for.
  [Decide the distilled feature set](https://github.com/kalonji-tools/purba/issues/10#issuecomment-5593350290) refused it, because it re-imports the up-front scope that [Distinguish a milestone from a feature](https://github.com/kalonji-tools/purba/issues/16#issuecomment-5561583582) refused.
- **An empty surface, with each integration as a library that a test imports by name.** Chosen. Each name joins with the issue that needs it.
  A test names each integration that affects it.

## Decision Outcome

**Reach:** `/LANDSCAPE.md` `/CONTEXT.md` `/pyproject.toml` `src/**`

**purba's [public surface](../../CONTEXT.md#public-surface) starts empty, and nothing joins it except through the issue that needs it.**

- **A name that a test uses joins with a [milestone](../../CONTEXT.md#milestone).** A flag or an exit code joins with a change that purba needs to work as a command.
  [A pinned candidate decides when a milestone is done](a-pinned-candidate-decides-when-a-milestone-is-done.md#decision-outcome) holds the rule for a flag and an exit code.
- **purba never loads a plugin.** A third-party integration, such as Django or Hypothesis, reaches a test only as a library that the test imports by name.
  Its author is the [library author](../../CONTEXT.md#library-author).

**Downside:**

- **Each pytest plugin that a candidate uses becomes a library or a fixture that the test names.**
- **A library author has nothing to build against until the first milestone.**
- **An integration that changes the whole run must become a fixture that each test names.** A Django database for each session is one example.

## Confirmation

No command checks it.
A gate could check that the surface is empty today, and the first milestone would make that check false.
No gate can decide whether an issue needs a name, so [a standard a gate cannot decide does not become a gate](a-standard-a-gate-cannot-decide-does-not-become-a-gate.md) applies.
The review of each pull request that adds a name checks that its issue needs the name.
[purba runs inside the tester's interpreter](purba-runs-inside-the-testers-interpreter.md#decision-outcome) keeps a plugin off the context diagram.
[purba divides by responsibility and not by language](purba-divides-by-responsibility-and-not-by-language.md#decision-outcome) keeps a plugin API off the parts diagram.
