# A design in depth waits for what it assumes

## Context and Problem Statement

The landscape names purba's parts and concerns.
Each one gets its own design in depth.
A design in depth uses the interfaces of the parts around it.
When it assumes an interface that no design settles yet, the later design of that part can undo the assumption.
The order of design must also put the effort where a failure is likely and costs most.

| fact | source |
|---|---|
| the risk-driven methods design the riskiest area first, and write each risk as a failure a test can check | [What do established design methods say about drawing a landscape before a deep design?](https://github.com/kalonji-tools/purba/issues/462#issuecomment-6099871177) |
| those methods weigh a risk by its probability times its impact | [What do established design methods say about drawing a landscape before a deep design?](https://github.com/kalonji-tools/purba/issues/462#issuecomment-6099871177) |
| the landscape names each part, and draws which part depends on which | [purba divides by responsibility and not by language](purba-divides-by-responsibility-and-not-by-language.md#decision-outcome) |
| the landscape names each concern, and the parts it crosses | [a concern is what no part can own](a-concern-is-what-no-part-can-own.md#decision-outcome) |
| the five qualities, in rank order | [purba ranks an honest outcome first](purba-ranks-an-honest-outcome-first.md#decision-outcome) |
| where oxitest's fixes fell, part by part | [How did oxitest divide into parts, and what did each part cost?](https://github.com/kalonji-tools/purba/issues/464#issuecomment-6100239730) |
| oxitest amended its async strategy twice for the task and the teardown of an async fixture | [Async organizational strategy for oxitest, its third and fourth amendments](https://github.com/kalonji-tools/oxitest/blob/731c2b3f76c708125eb4984d9725652208f5ae81/docs/adr/0006-async-organizational-strategy.md?plain=1#L176) |
| RustPython moved onto a fork of ruff's parser | [purba parses Python with ruff's parser](purba-parses-python-with-ruff.md#considered-options) |

## Considered Options

- **Callers first.** Rejected. A caller would come before each part it uses, so its design in depth would assume them.
- **Used parts first, with the concerns last or in no order.** Rejected. The Rust/Python seam and strictness both cross fixtures, so a design of fixtures would assume both.
- **Every concern first, then the parts in layers of what they use.** Rejected. Parsing waits only for errors, but a layer holds it behind concerns with lower scores.
- **Order by the highest quality an item serves.** Rejected. Strictness and interrupt handling both serve an honest outcome, so that order cannot choose between them.
- **A design in depth waits for every part it uses and every concern that crosses it, and risk orders what nothing blocks.** Chosen. No design assumes what a later design can undo. The effort goes where a failure is likely and costs most.

## Decision Outcome

**Reach:** `/LANDSCAPE.md` `/CONTEXT.md` `scripts/order-of-design.sh`

**A design in depth waits for every part it uses and every concern that crosses it, and risk orders what nothing blocks.**
[The landscape](../../LANDSCAPE.md#the-order-of-design) ranks each part and concern.

The rule takes the next part or concern in these steps.

1. Take the item with the highest score among the items that nothing blocks. The score of an item is the sum of the scores of its risks in the landscape.
2. On a tie, add up the scores of the items that wait on each one directly, and take the highest.
3. On a tie, take the item whose line comes first in the landscape, parts before concerns. This step is arbitrary, and it exists only so that the order is complete.

A [risk](../../CONTEXT.md#risk) scores its probability times its impact.
A risk's impact comes from the quality it breaks.
Probability and impact take the same levels.

| level | value | impact, by the quality a risk breaks |
|---|:-:|---|
| high | 3 | an honest outcome, explicit |
| medium | 2 | a legible result, a reviewable core |
| low | 1 | fast feedback |

A risk stays out of the landscape when it scores less than a high impact at a low probability.

| risk | lands on | breaks | impact | probability | evidence |
|---|---|---|:-:|:-:|---|
| text selection disagrees with import | selection | honest outcome | high | high | oxitest's Python collection drew 21 fixes and 13 bug issues, and its prescan drew 10 fixes. A test made at import time has no text to read. |
| a fixture reaches a test that never named it | fixtures | explicit | high | high | oxitest's fixture model drew 57 fixes, and was redesigned twice |
| an outcome depends on the worker | dispatch | honest outcome | high | high | oxitest's worker protocol drew the most fixes per line among its large parts |
| purba's own error shows as a test failure | errors | honest outcome | high | medium | oxitest's shared errors drew 13 fixes and 10 bug issues |
| an interrupt leaves a fixture built | interrupt handling | honest outcome, because the next run starts from what the stopped run left | high | medium | a judgment, because no oxitest fix names an interrupt |
| an async fixture leaves its loop or its task | fixtures | honest outcome | high | medium | oxitest amended its async strategy twice for the task and the teardown |
| a change in one part edits another part | command | reviewable core | medium | high | oxitest's pipeline drew 59 fixes, and changed its internal shape three times in five weeks |
| strictness refuses a test that checks | strictness | legible result | medium | medium | a judgment. oxitest's narrower strict checks drew 2 fixes, but purba's explicit scenario asks more. |
| parsing refuses a file that CPython runs | parsing | honest outcome | high | low | a judgment. purba parses with ruff's parser, and RustPython moved onto a fork of it. |
| the Rust/Python seam costs more per test than pytest | the Rust/Python seam | fast feedback | low | medium | stays out |
| a failure needs a re-run to read | assertion | legible result | medium | low | stays out. oxitest's assert rewriter drew 1 fix. |

When the design in depth of a part or concern merges, its order line and its risk lines leave the landscape.
Its line in the parts or the concerns then links its design.

**Downside:**

- **The part that churned most in oxitest comes last.** Command wires the run, so it waits for every part it uses. Until then, the walking skeleton guards it, and so does the invariant [the diagram is the only wiring](../../LANDSCAPE.md#the-invariants).
- **The item with the highest score waits.** Fixtures waits for the Rust/Python seam, whose only risk stays out of the landscape, so items with lower scores come before it.
- **Some probabilities are judgments.** A wrong judgment carries into the order until a pull request with new evidence corrects it.
- **The last tie-break is arbitrary.** `mise run fmt:order` prints each step, and names a step that this tie-break decides.
- **Each design in depth edits the landscape.** Its pull request removes the risk and order lines of its part or concern, links the design, and rewrites each other line it makes untrue.
  [A script calculates the order of design, and only a changed input moves it](only-a-changed-input-moves-the-order-of-design.md#decision-outcome) says when the order changes.

## Confirmation

No check judges a probability, because [a standard a gate cannot decide does not become a gate](a-standard-a-gate-cannot-decide-does-not-become-a-gate.md) applies.
`mise run lint:order` calculates the order from those probabilities, and refuses an order section that differs.
