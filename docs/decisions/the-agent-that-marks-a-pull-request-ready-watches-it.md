# The agent that marks a pull request ready watches it

## Context and Problem Statement

[A change is specified before it is built](a-change-is-specified-before-it-is-built.md) says a pull request marked ready asks for the reviewer.
It does not decide who reads the review when it arrives.

On the pull request that wrote `AGENTS.md`, each review round ended with a person typing "feedback awaits on the pr" before the agent read it.

## Considered Options

- **A workflow that starts an agent on each review.** Rejected. It gives an agent a token of this repository, which is a separate decision.
- **The agent that marked it ready watches it.** Chosen. Nobody has to carry the review to the agent that holds the work.

## Decision Outcome

**The agent that marks a pull request ready watches it, and acts on each review before anyone asks.**
The rule names the act and no mechanism, because each tool that `AGENTS.md` reaches watches its own way.

**Downside:**

- **The watch ends when the agent stops.** A review that lands after that waits for a person to ask.

## Confirmation

No command decides this, and none is planned.
The pull request shows it: the agent answers each review before any comment asks it to.

[`AGENTS.md`](../../AGENTS.md) carries this rule to an agent before it acts.
