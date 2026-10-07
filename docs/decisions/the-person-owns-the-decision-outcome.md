# The person owns the Decision Outcome

## Context and Problem Statement

A record's Decision Outcome states what purba decided.
[Architectural significance is declared, not detected](significance-is-declared-not-detected.md) sends every change to a record through the code owner's review.
It does not decide who answers for the decision that review accepts.

When purba took this decision, the places that stated the rule disagreed.

| where | what it said |
|---|---|
| a thread posted on each pull request that changed a record | the Decision Outcome is written by a person, or the change does not merge |
| `significance-is-declared-not-detected` | an agent may draft it where the person has shown, in review, that the points are read and understood |
| the practice | the agent writes it, the person reviews and pushes back, and both rewrite it |

## Considered Options

- **The person writes it, or the change does not merge.** Rejected. A gate over a judgement measures only that someone clicked. [A standard a gate cannot decide does not become a gate](a-standard-a-gate-cannot-decide-does-not-become-a-gate.md).
- **An agent drafts it only where the person showed, in review, that the points are read and understood.** Rejected. Comprehension is the person's own goal, and not the project's.
- **The person owns it, and work an agent wrote is welcome.** Chosen. `.github/CODEOWNERS` routes the directory and the required approval pins it, so the rule is decidable and true.

## Decision Outcome

**The person owns the Decision Outcome.**
Work an agent wrote is welcome, and nothing about it changes what the reviewer reads for.

**Downside:**

- **Nothing shows that the person read the Decision Outcome they own.** The approving review is the only trace, and it does not say what was read.

## Confirmation

`.github/CODEOWNERS` assigns `/docs/decisions/` to the person.
The ruleset requires that owner's review on a pull request that touches it.
Read it back from the ruleset with `require_code_owner_review`.

The versions above were measured in [Is the Decision Outcome rule a project goal, and if not, what is it doing in the merge path?](https://github.com/kalonji-tools/purba/issues/96).
