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

| pull request size | lines | mean lifetime |
|---|---|---|
| median | 159 | 1.5 h |
| p90 | 1,109 | |
| p99 | 7,437 | 4.0 h above 1,000 lines |

No threshold catches what mattered, because nothing ever required that a person had read anything.

## Decision Outcome

A change is architecturally significant if and only if it adds or changes a decision record.

`CODEOWNERS` assigns `/docs/decisions/` to the human, and the ruleset requires that owner's review on a pull request touching it.
The same ruleset requires one approving review on every pull request, under no path condition.

The reviewer reads for one thing: whether the decision, the spec and the implementation are the same thing.
That is the question no check can ask, and routing exists to put it in front of a person rather than to choose which person.

A pull request that touches a record carries all three links in one diff, and the code owner reads it.
A pull request that touches no record is further down a chain whose decision already merged, and the reviewer reads the change against the record that governs it.
Where no record governs it, the reviewer asks whether one is owed.
That question is a suggestion and never a gate, and nothing enforces it.

The explain-back artifact is the record itself.
The human owns the Decision Outcome.
An agent may draft it where the human has shown, in review, that the points are read and understood.
Work an agent wrote is welcome and is held to the harder standard.
You cannot state what is true, and what it costs, about a change you have not understood.
The remedy on failure is teaching, not a waiver.

Where the human wrote the code the mechanism inverts instead of doubling.
An agent explains the change back to the human, and a mismatch is the signal.

**Downside:**

- **A change can be significant and touch no record, and nothing catches it.** Making the signal decidable means giving up detection, and this is the price.
- **The question above is all that reaches that gap.** A companion gate stood beside it and required a pull request touching a record to also touch code. It is deleted because a record merges before the thing it decides is built, so a pull request carrying a record alone is the ordinary shape here and not an exception. The gate never said what counts as code either. What it was written to close stays open.

## Confirmation

The routing is live and enforcing:

| evidence | result |
|---|---|
| GitHub's CODEOWNERS validator | 0 errors |
| the first pull request to touch the directory | blocked |
| the review request GitHub generated | reported as coming from `CODEOWNERS` |
| every pull request merged to `main` | carried an approving review |

Read the routing back from the ruleset, not from here: `required_approving_review_count`, `require_code_owner_review` and the absence of a path condition are what decide it.

The block on approving your own pull request keys on identity, and one human is in the organisation.
A required approval records who accepted a change and pins it to a commit.
It does not produce a second reader.

[Liability is recorded from the act that makes it true](liability-is-recorded-from-the-act-that-makes-it-true.md) reads the same ruleset from the other side, and states what the approval this rule requires can and cannot prove.
