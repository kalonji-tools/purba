# A concern is what no part can own

## Context and Problem Statement

The landscape names purba's cross-cutting concerns.
The order of design must rank each concern beside the parts.
If a concern is anything that two parts touch, almost anything is a concern.
The landscape needs a test that keeps out what a part can own.

| fact | source |
|---|---|
| the parts record names the Rust/Python seam, strictness, configuration and interrupt handling as concerns | [purba divides by responsibility and not by language](purba-divides-by-responsibility-and-not-by-language.md#decision-outcome) |
| the parts record puts the timeout of a test inside dispatch, and async inside execution | [purba divides by responsibility and not by language](purba-divides-by-responsibility-and-not-by-language.md#decision-outcome) |
| arc42 says "Pick only the most-needed topics for your system" | [What do established design methods say about drawing a landscape before a deep design?](https://github.com/kalonji-tools/purba/issues/462#issuecomment-6099871177) |
| rust-analyzer lists nine cross-cutting concerns, among them Cancellation, Error Handling and Configurability | [What do established design methods say about drawing a landscape before a deep design?](https://github.com/kalonji-tools/purba/issues/462#issuecomment-6099871177) |
| pytest, cargo-nextest and unittest each handle an interrupt, each in a different way: as an exception, as an event or as a flag | [Which parts and cross-cutting concerns do comparable test runners have?](https://github.com/kalonji-tools/purba/issues/465#issuecomment-6100161159) |
| pytest has an exit code for an internal error, and cargo-nextest models "the full space of errors" in its events | [Which parts and cross-cutting concerns do comparable test runners have?](https://github.com/kalonji-tools/purba/issues/465#issuecomment-6100161159) |
| of the oxitest parts outside the distilled feature set, the `inspect` TUI and configuration were the largest by lines, and configuration held 2,395 Rust production lines | [How did oxitest divide into parts, and what did each part cost?](https://github.com/kalonji-tools/purba/issues/464#issuecomment-6100239730) |
| oxitest's shared errors held 1,026 Python production lines, and drew 12.7 `fix` commits per 1,000 | [How did oxitest divide into parts, and what did each part cost?](https://github.com/kalonji-tools/purba/issues/464#issuecomment-6100239730) |
| `unknown` is shown and counted, and never confused with clean | [What does always-strict mean, and which checks can exist under it?](https://github.com/kalonji-tools/purba/issues/32#issuecomment-5600829294) |
| a value that names a fact about the project is not a switch | [What does always-strict mean, and which checks can exist under it?](https://github.com/kalonji-tools/purba/issues/32#issuecomment-5600829294) |

## Considered Options

- **A concern is anything that two or more parts touch.** Rejected. Output capture, the timeout of a test, a selected test's identity and async would each join the list, though each has an owner.
- **A concern is what two or more parts follow, what no part can own, and what is designed in depth on its own.** Chosen. It keeps out what a part owns, and what has no design of its own.

## Decision Outcome

**Reach:** `/LANDSCAPE.md` `/CONTEXT.md`

**A [concern](../../CONTEXT.md#concern) is what no part can own.**
[The landscape](../../LANDSCAPE.md#the-cross-cutting-concerns) names each concern and the parts it crosses.

| concern | why no part can own it |
|---|---|
| the Rust/Python seam | execution, fixtures and assertion each hold a live Python object |
| strictness | each part that makes or shows an outcome must refuse to show as passed what was not checked |
| configuration | the part that reads a value owns what it means, and every reader follows one rule: a value states a fact about the project, and never turns a check off |
| interrupt handling | each part that holds a test while it runs must stop the test, and each part that holds what was built for it must tear that down |
| errors | every part can fail, and each failure is one of three kinds: a test's outcome, a diagnostic about the tester's project, or an error inside purba |

**A part crosses the seam when its job needs a live Python object.**
Command takes the arguments and returns an exit code, both data, so it does not cross the seam.
Dispatch and reporting pass data, so neither crosses it.

**What is not a concern:**

- **Output capture, warnings and stray exceptions.** They belong inside execution, which wraps the test and its fixtures and owns the outcome.
- **The timeout of a test.** It belongs inside dispatch, as [purba divides by responsibility and not by language](purba-divides-by-responsibility-and-not-by-language.md#decision-outcome) says.
- **A log of purba's own work.** No decision admits one yet.
- **Lazy work.** It has no design of its own, so it is not a concern.
  It is an invariant.
  [The landscape](../../LANDSCAPE.md#the-invariants) states it.
  [An invariant needs no design of its own](an-invariant-needs-no-design-of-its-own.md#decision-outcome) holds why.
  [purba ranks an honest outcome first](purba-ranks-an-honest-outcome-first.md#decision-outcome) makes it a mechanism of fast feedback.

**Downside:**

- **The errors concern crosses every part.** The interface of each part waits on the kinds of failure that errors keeps apart.
- **The seam rule rests on the invariant that an outcome is data.** [The landscape](../../LANDSCAPE.md#the-invariants) states that invariant. A breach of it puts each part that passes an outcome across the seam.
- **The line between a fact about the project and a switch is thin.** [What does always-strict mean, and which checks can exist under it?](https://github.com/kalonji-tools/purba/issues/32#issuecomment-5600829294) names it a risk.

## Confirmation

No check reads the concerns, because [a standard a gate cannot decide does not become a gate](a-standard-a-gate-cannot-decide-does-not-become-a-gate.md).

[The landscape](../../LANDSCAPE.md#the-order-of-design) ranks these concerns.
[A design in depth waits for what it assumes](a-design-in-depth-waits-for-what-it-assumes.md#decision-outcome) holds why.
