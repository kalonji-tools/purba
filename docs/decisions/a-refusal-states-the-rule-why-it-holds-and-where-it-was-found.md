# A refusal states the rule, why it holds, and where it was found

## Context and Problem Statement

[`AGENTS.md`](../../AGENTS.md) leaves out every rule a command refuses, because the command names the breach when it fires.
Nothing said what naming the breach requires.

No file stated a form for a [refusal](../../CONTEXT.md#refusal).
Two forms were in use.

| form | refusals | says what to do | states the rule |
|---|---:|---:|---:|
| the rule, then why it holds | 25 | 3 | 25 |
| what happened, then what to do | 9 | 6 | 1 |

## Considered Options

- **A refusal also names the file that holds its rule.** Rejected. `mise run lint:links` reads a cited path only in a comment. A file named in a refusal goes stale when that file is renamed. Nothing reports it.
- **A refusal states the rule, why it holds, and where it was found.** Chosen.

## Decision Outcome

A refusal states the rule, why it holds, and where it was found.

| part | required |
|---|---|
| the rule that must hold | yes |
| why the rule holds | yes |
| where the check found the breach | yes, as the detail `report` takes |
| what to do | where the check knows the repair |

**A refusal names no record.**

[A register belongs to one location](a-register-belongs-to-one-location.md) names the file that holds this register.

**Downside:**

- **No command decides whether a refusal states why its rule holds.** [A standard a gate cannot decide does not become a gate](a-standard-a-gate-cannot-decide-does-not-become-a-gate.md), so a reviewer reads it.
- **A reason can be thin.** The prose rules borrowed from Simplified Technical English give that standard as their reason, because no record states a better one.
- **The register reaches only a check that sources `scripts/report.sh`.** A workflow writes its refusal inline.

## Confirmation

A test under `scripts/test/` pins the words of each refusal a script writes.
The refusals in `.github/workflows/` have no test, because no runner here executes a workflow.

No command reads a refusal for its reason.
None is planned.

A command can decide whether a refusal names a record.
None reads it yet.
[Refuse a refusal that names a record](https://github.com/kalonji-tools/purba/issues/272) wires it.
