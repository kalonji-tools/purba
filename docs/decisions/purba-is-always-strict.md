# purba is always strict

## Context and Problem Statement

Strictness is one of purba's cross-cutting [concerns](../../CONTEXT.md#concern).
[The landscape](../../LANDSCAPE.md#the-cross-cutting-concerns) says that no switch turns it off.
A runner with no switch must still be able to change its checks.
rustc and clippy buy that right with a hatch.
purba buys that right with a [suppression](../../CONTEXT.md#suppression).
[A suppression is an outcome and not a switch](a-suppression-is-an-outcome-and-not-a-switch.md#decision-outcome) holds that decision.

| fact | source |
|---|---|
| oxitest shipped three modes: `abort`, `enforce` and `off` | [What does always-strict mean, and which checks can exist under it?](https://github.com/kalonji-tools/purba/issues/32) |
| an audit found 7 of oxitest's 11 strict mechanisms defective | [Decide the distilled feature set](https://github.com/kalonji-tools/purba/issues/10#issuecomment-5597159743) |
| the audit's rule: "a detector good enough to suggest is not thereby good enough to gate" | [What does always-strict mean, and which checks can exist under it?](https://github.com/kalonji-tools/purba/issues/32) |
| oxitest had no suppression at a site and no configuration for a check, so its mode was its only hatch | [What does always-strict mean, and which checks can exist under it?](https://github.com/kalonji-tools/purba/issues/32#issuecomment-5600829294) |
| clippy's `correctness` lints "should be free of false positives", and `eq_op`, a correctness lint, carries an open false positive filed 2017-01-22 | [What does always-strict mean, and which checks can exist under it?](https://github.com/kalonji-tools/purba/issues/32#issuecomment-5600829294) |
| `manual_clamp` was restricted to trigger only where it can "guarantee" that `clamp` does not panic | [What does always-strict mean, and which checks can exist under it?](https://github.com/kalonji-tools/purba/issues/32#issuecomment-5600829294) |
| rustc and clippy stay silent where a lint cannot see, and `#[allow]` makes that silence safe | [What does always-strict mean, and which checks can exist under it?](https://github.com/kalonji-tools/purba/issues/32#issuecomment-5600829294) |
| rustc's suggestion-only idiom groups "did not seem to work very well" | [What does always-strict mean, and which checks can exist under it?](https://github.com/kalonji-tools/purba/issues/32#issuecomment-5600829294) |
| oxitest's strict judged each test by its `node_id`, so a migration by path worked | [What does always-strict mean, and which checks can exist under it?](https://github.com/kalonji-tools/purba/issues/32) |

## Considered Options

- **A mode switch, as oxitest's `abort`, `enforce` and `off`.** Rejected. A switch makes a broken check survivable, so the broken check survives.
- **Strict by default, with a silent opt-out for each test or each check.** Rejected. Each silent opt-out is the same switch at a smaller scale.
- **Every strict check decides every test.** Rejected. A lint in rustc or clippy stays silent where it cannot see. Even a correctness lint carries an open false positive.
- **Always strict, and a strict check blocks only when its predicate guarantees each verdict it gives.** Chosen. The bar is a property of the predicate, and the design of the check sets it.

## Decision Outcome

**Reach:** `/LANDSCAPE.md` `/CONTEXT.md`

**purba is always strict: no mode, flag or opt-out turns strictness off.**

- **A [strict check](../../CONTEXT.md#strict-check) blocks only when its predicate guarantees each violation or clean that it gives.** A check that blocks answers `unknown` where its predicate cannot guarantee the verdict.
- **purba shows and counts `unknown`, and never shows it as clean.**
- **A value that names a fact about the project is not a switch.** [A concern is what no part can own](a-concern-is-what-no-part-can-own.md#decision-outcome) holds that rule for configuration.
- **Selection is not a switch.** Strictness judges only the tests that a run selects, so a tester can migrate one path at a time.
- **A `skip` or an `xfail` with a reason is an outcome, and not a switch.** It shows as skipped or as an expected failure, and never as passed.

**Downside:**

- **A false positive costs the tester a suppression and its reason until a fix ships.** [The landscape](../../LANDSCAPE.md#the-risks) names it as the risk that strictness refuses a test that checks its result through a helper.
- **A check that warns and does not block needs a category beside the checks that block, and not a separate channel.** A separate channel of suggestions failed in rustc.
- **Each demotion states its trigger.**
- **A removed check keeps its name and states why it went.**
- **A suggested fix carries its own confidence, apart from the verdict.**
- **A bar measured on a corpus of real suites waits for a pinned [candidate](../../CONTEXT.md#candidate).** Until then, the bar on the predicate is the only one purba can run.
- **Whether a style rule has the same weight as a contract rule stays open.** The design of the checks decides it.

## Confirmation

`mise run test:wheel` runs the explicit scenario of [the landscape](../../LANDSCAPE.md#the-driver-scenarios): purba prints `unknown` for a test that states no expectation.
No command checks the rest.
[The landscape](../../LANDSCAPE.md#the-order-of-design) ranks the design in depth of strictness.
That design answers each cost above.
