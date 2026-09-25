# A commit message outlives its review

## Context and Problem Statement

Commit body length varies several-fold across comparable projects.

| corpus | mean body words, among commits that have a body | have a body |
|---|---|---|
| cargo | 36 | 42% |
| hypothesis | 49 | 67% |
| pytest | 64 | 92% |
| the frozen prototype | 65 | 88% |
| uv | 85 | 89% |
| git | 125 | 100% |
| ruff | 127 | 86% |

Length is no longer the defect.
A commit that adds a decision record repeats the record it adds.

| commits on `main` when this was decided | n | mean body words |
|---|---:|---:|
| add or change a decision record | 8 | 101 |
| do not | 5 | 200 |

Four of the five record commits then on `main` opened with the Context of the record in the same commit.
Two of those five went on to list the changes.
The commit is a second copy of a document that sits in the same tree.

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

Permanence decides what is eligible. A second question decides what is written.

- **A named element list, with an exception for a record commit.** Rejected. Every new kind of commit needs another clause, and the list already collides with itself: a body cannot both state the problem and not restate the diff when the diff is the record.
- **Subtraction: the body carries what no other location already carries.** Chosen. It states the value of a commit rather than its length, and the two rules the list already carries become instances of it.

## Decision Outcome

A commit message carries what must outlive the review that produced it, and nothing else.

Everything a review consumes and discards belongs in the pull request or the issue.

Permanence decides among the statements the committer can make truthfully.
A statement the committer cannot make belongs wherever the person who can make it acts.
[Liability is recorded from the act that makes it true](liability-is-recorded-from-the-act-that-makes-it-true.md) divides liability along that line.
The origin statement is the part a contributor can make, so it belongs in the commit.
The review statement is the part no committer can make, so it stays where the reviewer acts.

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

The number is an instruction to the writer as well as an anchor for the reader.
Depth belongs on the issue, so the body stops at the conclusion and the number carries the reader the rest of the way.

**Body.**

**A body is owed when the change makes a claim.**
A change that makes none carries a subject and nothing else.

Where a body is written, it carries what no other location already carries.

- Addressed to the reader who has long since forgotten the details, never to the reviewer.
- The pull request, the issue and the decision record are locations, and [a register belongs to one location](a-register-belongs-to-one-location.md) names the register of each. What a location already holds is not written in the body again, and neither is what the diff already shows.
- It states what would show the change wrong. No other location holds that sentence.
- It names what a change relates to, and never its position in a planned series. A series can shrink. A body cannot be corrected.

**A rewrite carries more than a new record.**

[A decision record is rewritten, not amended](a-record-is-rewritten-not-amended.md) replaces a record in place, so the tree never says what that record used to say.

| the commit adds | the record holds | the body carries |
|---|---|---|
| a new record | the problem, the options, the cost | what would show the decision wrong |
| a rewrite | the new state only | what was wrong before, why, and what would show the decision wrong |

The commit body is the amendment history that record removes.

**Trailers.** `Assisted-by:` names the agent and model, plus any specialised analysis tool.

A contributor writes a `Signed-off-by:` trailer and no workflow writes one, for the reason given in [liability is recorded from the act that makes it true](liability-is-recorded-from-the-act-that-makes-it-true.md).

The `Assisted-by:` form names the agent and the model.
The kernel shipped that form, ran it for seven months, and replaced it with a bare `LLM`.
purba keeps the replaced form deliberately, because this project's premise is that provenance must be checkable, and a bare `LLM` is not a fact anyone can check against anything.

A session URL trailer is not used.
It was applied zero times across 2,129 prototype commits while `Assisted-by:` reached 84.7% under the same instruction, and it is a link no reader but its owner can follow.

**Downside:** four costs, and the first is structural.

- **The central rule is unenforceable by design.** Whether a sentence must outlive its review is a judgement, not a string test, so nothing can gate it.
- **Subtraction fails silently.** A writer who judges wrongly writes nothing, the information is simply absent, and no reader learns that it was owed.
- **Naming the agent and model diverges from the only upstream convention** for AI attribution, and the gap widens if that convention settles.
- **The issue number is verified by nothing.** A wrong number still renders as a working link, and it spends about six characters of a seventy-two character budget.

## Confirmation

| rule | decidable | checked |
|---|---|---|
| conventional type and scope | yes | yes, `subject-form` |
| lower case after the colon | yes | yes, `subject-form` |
| the issue number in parentheses at the end | yes | yes, `subject-form` |
| subject at most 72 characters | yes | yes, `subject-length` |
| no full stop before the reference | yes | yes, `subject-full-stop` |
| imperative mood | no, because English verbs are an open class | no |
| `Assisted-by:` present on agent work | no, because the message never says whether a machine helped | no |
| the referenced issue exists | yes | **no** |
| the body carries only what outlives review | **no** | no |
| the body carries only what no other location carries | **no** | no |
| the body carries no count of a planned series | weak: a pattern test suggests it and cannot confirm it | no |

Five rows are checked, by hooks `prek.toml` carries.

One decidable row is unchecked and no ticket owns it.
Nothing resolves the issue a subject names, so a wrong number still renders as a working link.

The rule this record exists to state is that the body carries only what outlives review.
It is the one no check can ever enforce.
A detector good enough to suggest is not good enough to gate.
