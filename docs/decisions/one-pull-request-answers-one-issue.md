# One pull request answers one issue

## Context and Problem Statement

[Architectural significance is declared, not detected](significance-is-declared-not-detected.md) rejects diff size, because reviewability is the number of independent decisions, not lines.
It never bounded how many decisions one pull request may carry.

Records touched is not decisions carried.

| pull request | records rewritten | decisions carried |
|---|---:|---:|
| one change | 2 | 1 |
| an audit repair | 3 | 0 |

| measured on `main` at `a8ed513` | |
|---|---:|
| merged pull requests | 54 |
| that name more than one issue in the title | 0 |
| that touched a record | 40 |
| that touched more than one record | 15 |

## Considered Options

- **One decision per pull request.** A repair carries no decision, so a bound on decisions never reaches it.
- **A gate on the number of records a pull request touches.** That count is decidable and counts records, which is the wrong thing.

## Decision Outcome

**Reach:** [pull request](../../CONTEXT.md#pull-request) [issue](../../CONTEXT.md#issue) `/AGENTS.md`

**Area:** workflow

**One pull request answers one issue.**
Nothing bounds the number of records it rewrites.
Every merged pull request already complied, so the rule states what the tree does.
A second issue rides along when one change answers both, and the pull request says so.

**Downside:**

- **No gate holds it.** A second issue may ride along when one change answers both. Only the reviewer can judge that.

## Confirmation

| what shows it | where |
|---|---|
| one issue is answered | `gh pr view <n> --json closingIssuesReferences` |

[A change is specified before it is built](a-change-is-specified-before-it-is-built.md) places this rule in the steps a change follows.
The measurements above were made on [May one pull request carry more than one decision?](https://github.com/kalonji-tools/purba/issues/116).
