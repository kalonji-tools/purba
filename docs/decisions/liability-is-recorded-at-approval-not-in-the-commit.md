# Liability is recorded at approval, not in the commit

## Context and Problem Statement

An agent writes every commit in this repository, and a person is answerable for each one.
The Developer Certificate of Origin convention records that with a `Signed-off-by:` trailer in the commit message.

`git commit -s` writes that trailer at the moment the commit is made.
The agent makes the commit, so the agent writes the trailer, and the trailer names a person who has not read the code yet.

| | |
|---|---|
| commits on `main` carrying a premature trailer before this decision | 3 |
| checks that verified any of them | 0 |

The first commit written after the rule was agreed omitted the trailer.
A person found that by reading trailers by hand.

[A commit message outlives its review](a-commit-outlives-its-review.md) divides commit content by permanence, and liability must outlive its review, so that rule alone would place liability in the commit message.
This record is the exception to it.
The cause is that the committer cannot make the statement truthfully.

This is not the divide-by-artifact option that record rejected.
That option moved rationale out of the commit, and it lost to a measurement of commit body length across 47 repositories.
This record moves one certification out, and the cause is truthfulness rather than volume.

## Considered Options

- **Gate the trailer in CI.** Rejected. It makes the premature claim mandatory, and it does not make the certification true.
- **Write the trailer after review, from a workflow.** Rejected. It needs a force-push for every pull request. A push made with `GITHUB_TOKEN` starts no workflow run, so the required check never reports again and the pull request blocks forever. A personal access token would fix that and would add a second long-lived credential.
- **A required review thread that asks the reviewer.** Rejected. A pull request with zero threads satisfies the rule, so the mechanism fails open. The agent opens every pull request and holds write, so the agent can resolve its own thread.
- **A job that compares the landed tree with the signed tree after a merge.** Rejected. The ruleset admits no bypass actor, an administrator can never bypass it, every commit arrives through a pull request, and a current branch merged by rebase replays without changing the tree. The job could only fire when the configuration changed, so it watched a consequence of drift rather than drift itself.
- **A required reviewer on a deployment environment.** Chosen. It computes nothing, it waits for a person, and it names the person who acted.

## Decision Outcome

A commit carries no `Signed-off-by:` trailer, and a human approval on the `signoff` environment is the liability record.

| | |
|---|---|
| environment | `signoff` |
| reviewers | the human code owner |
| `prevent_self_review` | `false` |
| required check | `Sign-off` |

`prevent_self_review` stays `false` because it blocks the actor who triggered the run.
The agent is not a reviewer, so the agent can never approve.
Setting it `true` would only deadlock a pull request that the human opens.

The record names the tree, and not the commit alone.
A rebase merge replays the commit under a new SHA, and the tree survives that replay only when the branch is already current with `main`.
Rebasing onto a moved `main` rebuilds the tree, and that is correct rather than a fault.
The ruleset requires a branch to be current before it merges, so the replay changes nothing and the tree names what the reviewer read.
A tree that differs after a merge means the configuration changed or somebody bypassed it, and the sign-off then covers nothing that landed.

**Downside:** the record stops travelling with the code.
An approval lives in GitHub and a trailer lives in git, so a clone of this repository carries no approval.

A clone is not silent about people.
The author field names the agent account, and the committer field names a person, because a rebase merge sets the committer to whoever pressed merge.
That is a record of the merge and not of the review.
Git has no field that separates the person who reviewed from the person who merged, so a reader outside GitHub can learn who released the change and never who accepted responsibility for it.

## Confirmation

| check | strength |
|---|---|
| `Sign-off` is in `required_status_checks` on the `protect-main` ruleset | strong |
| the `signoff` environment names a reviewer who is not the agent | strong |
| the approval names a person | strong |
| the branch is current before it merges, so the tree is stable | strong |
| the tree that merged is the tree that was signed | strong, and it follows from the rows above rather than from a check |
| the person who merged is the person who approved | none |
| the person read the code | none |

Every strong row rests on the ruleset, and nothing watches the ruleset itself.
A person can change it without a pull request, without an approval and without a record, and no row above would notice.

The merge row is decidable and unchecked.
GitHub reports who merged a pull request and who approved the deployment, so comparing them is a string test.
Nothing compares them today, and one person holds both roles, so the gap stays invisible until a second person can merge.

The last row is the point of the whole mechanism, and no check can ever make it.
A detector good enough to suggest is not good enough to gate.

A pull request from a fork receives a read-only token, so the job fails after the approval is given.
The workflow header records that limit, and nothing fixes it.
