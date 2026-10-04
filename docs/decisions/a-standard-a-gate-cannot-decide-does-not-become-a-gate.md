# A standard a gate cannot decide does not become a gate

## Context and Problem Statement

[A gate owns the mechanical standard](a-gate-owns-the-mechanical.md) puts every standard a reviewer would otherwise state by hand into a gate.
It does not decide whether a standard that no command can decide becomes a gate.

When purba took this decision, the ruleset made the reviewer resolve a thread on each pull request that changed a record.
The thread said: "The Decision Outcome is written by a person, or the change does not merge."

| how the thread was discharged, on 28 merged pull requests | count |
|---|---:|
| the person wrote the Decision Outcome | 0 |
| the person authorised a draft with words in the thread | 0 |
| the thread resolved, with no words at all | 28 |

## Considered Options

- **Require the thread to be resolved before the merge.** Rejected. In the table above it records a click every time and a judgement never.
- **Post the questions, and block nothing.** Chosen. The reviewer reads them while they decide. Nothing claims that a judgement was made.

## Decision Outcome

**A standard a gate cannot decide does not become a gate.**
The record states it, the reader judges it, and nothing blocks on it.
A gate over a judgement measures only that someone clicked.
A reader who judged nothing satisfies it exactly as well.

**Downside:**

- **Where no gate stands over a judgement, nothing records that one was made.** The approving review is the only trace, and it does not say what was read. [Liability is recorded from the act that makes it true](liability-is-recorded-from-the-act-that-makes-it-true.md) took that trade knowingly, and a resolved thread never survived a clone either.

## Confirmation

The refusal of a gate over a judgement was measured on the only one this project built, in [Is the Decision Outcome rule a project goal, and if not, what is it doing in the merge path?](https://github.com/kalonji-tools/purba/issues/96).

What is checkable here is the endpoint.
A record comment arrives through `issues/{number}/comments`, which carries nothing to resolve.
`required_review_thread_resolution` therefore cannot reach it.
A review comment would be reached, and that is the whole distance between a message and a gate.
