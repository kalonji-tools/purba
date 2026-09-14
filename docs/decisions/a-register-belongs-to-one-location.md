# A register belongs to one location

## Context and Problem Statement

purba writes into twelve locations.
Two of them state how to write there.

| location | states how to write there |
|---|---|
| commit message | yes, in [a commit message outlives its review](a-commit-outlives-its-review.md) |
| `docs/decisions/` | yes, in `docs/decisions/.template.md` |
| the other ten | no |

[A location inherits its readers](a-location-inherits-its-readers.md) says who reads each location.
It does not say what belongs there, and a reader who arrives at a location nobody has described writes whatever the last author wrote.

The prototype shows the cost of a rule that nothing carries.
It shipped two issue forms with required fields and one pull request template.

| prototype corpus | n | carry the template structure |
|---|---:|---:|
| issues | 1,317 | 0 |
| issues carrying the form's title prefix | 1,317 | 0 |
| pull requests carrying two of the template's three headings | 899 | 0 |
| the one pull request from an outside contributor | 1 | 0 |

The cause is mechanical rather than a failure of discipline.
A template is rendered by the client that opens it, and 1,317 of 1,317 issues plus 849 of 899 pull requests arrived through the API.
GitHub's own documentation says the setting that hides the blank-issue link "encourages" template use.

purba's two busiest locations already disagree with each other.

| | issues | pull requests |
|---|---:|---:|
| written so far | 74 | 11 |
| distinct second-level headings | | 33 |
| opening with `## Question` or `## Task` | 72 | |
| carrying no second-level heading at all | 0 | 3 |

The issue reached 72 of 74 with nothing enforcing it.
The pull request has no shape to preserve.

## Considered Options

- **One register, written as the same dimensions for every location.** Rejected. A commit subject, a record and an issue distribute information for different reasons, and forcing one vocabulary over them makes a table that reads well and decides nothing.
- **A register per location, written inside this record.** Rejected. [A record is rewritten, not amended](a-record-is-rewritten-not-amended.md) governs records and points at its template rather than restating it, and a rule kept three clicks from the person writing is a rule they do not read.
- **A template, relied on as the mechanism.** Rejected by the measurements above. A template reaches the web interface and reaches nothing else.
- **A required status check over the pull request body.** Rejected. It is the one location purba could gate this way, and the gate would buy a shape that costs nothing to state and refuse work over a heading. The project would rather lose the guarantee than spend a reviewer's merge on prose.
- **A workflow on every issue, checking its headings.** Rejected. An issue has no merge event, so the check can only label after the fact, and it would fire on every ticket of one author who already follows the rule.
- **A register per location, named here and written where that location's author reads it.** Chosen.

## Decision Outcome

A register belongs to one location, and this record names it rather than holding it.

**Two layers.**

Distribution belongs to purba.
It says what information belongs at a location.

Voice belongs to the language, and only where that language runs a tool that enforces it.
Elsewhere voice is convention.
An unpublished comment carries such a register: best practice rather than an enforced rule.
What belongs in one at all is held by [An unpublished comment carries what no other location carries](an-unpublished-comment-carries-what-no-other-location-carries.md).

**The published comment is split across both layers.**

The language's convention governs how it reads, and purba governs which artifact earns one.
A published comment covers every artifact purba exposes.
It also covers a private artifact that carries heavy traffic, because the reader arriving at one is in the same position as a reader arriving at a public one.
It is curated rather than accumulated, so an artifact that earns none carries none.

**Named, not held.**

The register is a file in the tree, or a convention named from outside.
The location's own author writes it.

**A derived location inherits its source's register.**

The CHANGELOG renders commit subjects, so a bad changelog line is a commit subject defect.
Python's `__doc__` and the type stub render the Rust doc comment.

**A register is written when the location needs one.**

A location does not open without a reader and a register.
Both prototype wikis opened with neither, and wrote 0 pages in 4 months.

**Nothing here is gated, with one exception.**

The pull request is the one location purba could gate, because it has a merge event and `main` already requires a status check.
A register is worth stating and is not worth refusing a merge over.

The exception is [An unpublished comment carries what no other location carries](an-unpublished-comment-carries-what-no-other-location-carries.md), which gates link liveness inside an unpublished comment.
Everything else about that location is convention.

The pull request's constraint is the one this record holds, because that register does not exist yet:
a pull request must carry a section addressed to the human reviewer that names what they must decide.
The section names belong to the ticket that writes the template.

A pull request holds the discussion of how the subject was implemented, between the author, the co-author and the approver.
That discussion is consumed by the review and does not travel, which is why [a commit message outlives its review](a-commit-outlives-its-review.md) keeps it out of the commit.
These are titles for the parties to one pull request, and the roster in [an actor is what it does, not what it is](an-actor-is-what-it-does-not-what-it-is.md) names readers rather than parties.

**Downside:**

- Every register depends on an author who reads it. The evidence is 72 of 74 issues by one author, and it says nothing about a second author or an outside contributor.
- A register can drift from the record that names it, because naming is a pointer and nothing compares the two.
- Six locations hold no register, so this record is revisited every time one opens.

## Confirmation

| location | its register | decidable | gated |
|---|---|---|---|
| commit message | [a commit message outlives its review](a-commit-outlives-its-review.md) | partly, and that record lists which rows | no |
| `docs/decisions/` | `docs/decisions/.template.md` | yes, four sections in order | no |
| pull request | `.github/PULL_REQUEST_TEMPLATE.md` | yes | no, refused |
| issue | `.github/ISSUE_TEMPLATE/` | yes | no, refused |
| published doc comments | the language's convention for voice, and this record for which artifact earns one | by that language's tooling, where it exists | no |
| unpublished comments | [An unpublished comment carries what no other location carries](an-unpublished-comment-carries-what-no-other-location-carries.md) | link liveness only | that one property, and nothing else |

One row is gated, and only for link liveness.
Two could be, and Considered Options says why.

Six locations are closed and hold no register.

| closed location | opens when |
|---|---|
| CHANGELOG | the release workflow lands, and `cliff.toml` is its register |
| public docs site | Pages opens, and Diátaxis is its register |
| local internals library | something is written into it |
| `AGENTS.md` | the file is written |
| wiki | never, unless a reader and a register are named first |
| discussions | never, unless a reader and a register are named first |

Nothing here checks whether a register is the right one for its location.
That fails as friction, and a reader reports it.
