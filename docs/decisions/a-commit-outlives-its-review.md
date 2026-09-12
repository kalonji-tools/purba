# A commit message outlives its review

## Context and Problem Statement

purba writes commit bodies far longer than anyone else's.

| corpus | mean body words, among commits that have a body | have a body |
|---|---|---|
| cargo | 36 | 42% |
| hypothesis | 49 | 67% |
| pytest | 64 | 92% |
| the frozen prototype | 65 | 88% |
| uv | 85 | 89% |
| git | 125 | 100% |
| ruff | 127 | 86% |
| **purba** | **452** | 100% |

There is no standard to appeal to.
No ISO, IEEE or RFC governs commit messages, Conventional Commits specifies only the machine contract, and git and the kernel specify the human one.
Both of those assume the commit is the only durable artifact, because both are mailing-list projects with no issue tracker and no decision records.

The obvious fix was to let decision records absorb the rationale, and **measurement killed it**.
Across 47 repositories and 506,825 non-merge commits, projects that keep decision records write bodies of 42.2 words against 38.3 for those that do not: null, and pointing the wrong way.
Record corpora run at roughly one record per thousand commits, so they were never a reservoir large enough to absorb anything.

## Considered Options

- **Cap the body length in CI.** Rejected. No commit-message tool caps body length, three ship body *minimums* instead, and a four-hundred-line body passes checkpatch with zero warnings. A number in CI is also the shape of check that cost the prototype thousands of lines of scripts for two recorded fires.
- **Divide by artifact: the record holds rationale, the issue holds deliberation, the commit holds the rest.** Rejected by the measurement above. No project documents such a division, and the substitution it assumes happens nowhere.
- **Divide by permanence.** Chosen. It is the only seam with precedent: four projects separate the commit message from review material on whether the text must outlive the review.

## Decision Outcome

A commit message carries what must outlive the review that produced it, and nothing else.

Everything a review consumes and discards belongs in the pull request or the issue.

Permanence decides among the statements the committer can make truthfully.
A statement the committer cannot make belongs wherever the person who can make it acts.
Liability is the one such statement today, and [liability is recorded at approval, not in the commit](liability-is-recorded-at-approval-not-in-the-commit.md) holds it.

**Subject.**

| rule | |
|---|---|
| form | conventional type, optional scope |
| mood | imperative, no trailing full stop, lower case after the colon |
| length | at most 72 characters |
| reference | the issue number, in parentheses, at the end |

The subject is the changelog entry.
git-cliff renders it directly, so it is read by users who never open the repository.

The issue number stays in the subject rather than moving to a trailer.
`git blame` exposes the subject and no part of the body.
The reader who has forgotten the details arrives through blame, and a trailer is invisible there.

**Body.**

- Addressed to the reader who has long since forgotten the details, never to the reviewer.
- States the problem in the present tense, why this approach, and what would show it wrong.
- Does not restate the diff.
- Target near 150 words. Not enforced.

**Trailers.** `Assisted-by:` names the agent and model, plus any specialised analysis tool.

purba writes no `Signed-off-by:` trailer, for the reason given in [liability is recorded at approval, not in the commit](liability-is-recorded-at-approval-not-in-the-commit.md).

The `Assisted-by:` form names the agent and the model.
The kernel shipped that form, ran it for seven months, and replaced it with a bare `LLM`.
purba keeps the replaced form deliberately, because this project's premise is that provenance must be checkable, and a bare `LLM` is not a fact anyone can check against anything.

A session URL trailer is not used.
It was applied zero times across 2,129 prototype commits while `Assisted-by:` reached 84.7% under the same instruction, and it is a link no reader but its owner can follow.

**Downside:** three costs, and the first is structural.

- **The central rule is unenforceable by design.** Whether a sentence must outlive its review is a judgement, not a string test, so nothing can gate it.
- **Naming the agent and model diverges from the only upstream convention** for AI attribution, and the gap widens if that convention settles.
- **The issue number is verified by nothing.** A wrong number still renders as a working link, and it spends about six characters of a seventy-two character budget.

## Confirmation

| rule | decidable | checked today |
|---|---|---|
| conventional type and scope | yes | no |
| subject at most 72 characters | yes | no |
| imperative mood, no full stop | yes, by a word-list test | no |
| `Assisted-by:` present on agent work | yes | no |
| the referenced issue exists | yes | **no** |
| the body carries only what outlives review | **no** | no |

Every decidable row is unchecked.
No ticket owns any of them.

The last row is the important one.
It is the rule this record exists to state, and it is the one no check can ever enforce.
A detector good enough to suggest is not good enough to gate.
