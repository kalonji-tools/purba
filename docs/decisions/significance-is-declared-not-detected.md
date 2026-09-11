# Architectural significance is declared, not detected

## Context and Problem Statement

Human attention is the scarce resource, so review has to be scoped to the changes that need it.
The prototype shows both ways this fails.

| prototype, 859 merged pull requests | |
|---|---|
| merged with zero review events | 770, or 89.6% |
| review events in total | 495 |
| of those, approvals | 14 |
| required approving reviews, final setting | 0 |

It required a review on everything, found the human was the bottleneck, and set the required count to zero.
That is worse than never having had the requirement, because it looks like a decision.

Scoping needs a signal CI can see without judgement.
Every candidate measured on the prototype fails.

## Considered Options

- **Diff size.** Rejected. Reviewability is the number of independent decisions, not lines. A signature change across 500 files is one decision. 55 lines across scopes is several.
- **A breaking-change marker in the commit message.** Rejected. It appears on 21 of 2,129 commits.
- **Detecting that a change touches a record.** Rejected as a detector. Of 89 commits touching the prototype's record directory, 61 touched no code at all.
- **Declaring it.** Chosen. The property becomes true by definition, so CI routes on it without judging anything.

The size numbers are worth keeping, because they kill the instrument outright:

| pull request size | lines | mean lifetime |
|---|---|---|
| median | 159 | 1.5 h |
| p90 | 1,109 | |
| p99 | 7,437 | 4.0 h above 1,000 lines |

The large ones were not read more carefully and the small ones were not read at all.
No threshold catches what mattered, because nothing ever required that a person had read anything.

## Decision Outcome

A change is architecturally significant if and only if it adds or changes a decision record.

CI routes on `docs/decisions/` being touched.
A pull request that touches it needs the code owner's approval, which is roughly one change in ten.
Every other pull request is unrouted.

The explain-back artifact is the record itself.
The human writes the Decision Outcome, or the change does not merge.
You cannot state what is true, and what it costs, about a change you have not understood.
The remedy on failure is teaching, not a waiver.

Where the human wrote the code the mechanism inverts instead of doubling.
An agent explains the change back to the human, and a mismatch is the signal.
The project has one comprehension mechanism, pointed in two directions.

A companion rule closes the gap the detector could not:

- **Gate.** A pull request touching a decision record must also touch code.
- **Suggestion.** A pull request touching code may be asked whether it needs a record.

**Downside:** two failure modes, both named, neither mechanical.

- **False negative.** A change can be significant and touch no record, and nothing catches it. Making the signal decidable means giving up detection, and this is the price.
- **False positive.** The companion gate refuses a pull request that only stands records. It cannot be satisfied while the tree holds no product code, which is the scaffold's whole destination. The pull request that stood these first records met the gate only because it also deleted a stale numbered citation from the manifest, and that does not repeat on demand.

This is also the one gate the human can fail on their own project, so it is the one they can quietly delete.
It costs a single Decision Outcome, and that is what makes it survivable.

## Confirmation

`CODEOWNERS` assigns `/docs/decisions/` to the human, and the branch ruleset requires code-owner review.

This is live and enforcing:

| evidence | |
|---|---|
| GitHub's CODEOWNERS validator | 0 errors |
| the first pull request to touch the directory | blocked |
| the review request GitHub generated | reported as coming from `CODEOWNERS` |

The companion gate is enforced by nothing.
It is a stated rule with no check, and a decided rule with no gate is not in force.

One limit no configuration removes.
The block on approving your own pull request keys on identity, and one human is in the organisation.
A required approval records who accepted a change and pins it to a commit.
It does not produce a second reader.
