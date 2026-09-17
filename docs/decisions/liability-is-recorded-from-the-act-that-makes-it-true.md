# Liability is recorded from the act that makes it true

## Context and Problem Statement

A commit can be written by an agent, and nothing prevents that.
A person is answerable for it.

`main` records that answerability inconsistently, and the inconsistency is an ambiguity rather than a simple absence.

| | |
|---|---|
| commits carrying a `Signed-off-by:` trailer | 3 |
| the first commit without one | `6d88f99`, which created the sign-off workflow |
| commits from `6d88f99` until this decision | unsigned, and their answerability lives on their pull requests |

A reader cannot tell an unsigned commit from one signed by a mechanism that leaves no trace.

The obvious command makes this worse.
`git log --grep='Signed-off-by'` returns 4 against a true 3, because one commit body discusses the trailer in prose.
Only `git log --format='%(trailers:key=Signed-off-by)'` parses trailers.

purba is public and accepts pull requests from forks.
An outside contributor is therefore a reader this decision serves now, and not one it may serve later.

**Answerability is not one thing.**
Four of the five statements below are events, and the fifth is a standing role.

| liability | answers | attaches to | fixed or changing |
|---|---|---|---|
| origin | may this be submitted at all | a contribution | fixed at authorship |
| authorship | who or what wrote it | a commit | fixed |
| review | who read it and judged it sound | a change | fixed at review |
| acceptance | who let it onto `main` | a merge | fixed at merge |
| stewardship | who answers for this code now | a path | changes |

Stewardship never belongs in a commit.
A commit is immutable and an owner changes, so a commit that names an owner records a fact with an expiry date.
`CODEOWNERS` holds it, and [a location inherits its readers](a-location-inherits-its-readers.md) holds the same shape for readers.

**Acceptance had no home, and the obvious one is already occupied.**
Git carries an author and a committer.
The agent is truthfully both, because it writes the change and it makes the commit.
A rebase merge overwrites the committer with whoever merged, which stores one true fact by destroying another.
No signature is available as a third place: GitHub does not sign what a rebase merge replays.

## Considered Options

- **Keep answerability only at the approval.** Rejected. It does not survive a clone, and the mechanism fails on a pull request from a fork, which is the path an outside contributor uses.
- **Treat the committer field as the acceptance record.** Rejected. The field already states who made the commit, and a design that depends on a merge overwriting it is a coincidence rather than a mechanism.
- **Two human approvals, one on a deployment environment and one on the pull request.** Rejected. A push that leaves the tree unchanged does not dismiss a review, so one approval is sufficient and the second act records nothing the first does not.
- **Squash merge, to gain control of the commit message.** Rejected. It works, and the ruleset permits a linear history under squash, so this is refused on cost rather than on feasibility. It destroys the commit granularity that [a commit message outlives its review](a-commit-outlives-its-review.md) exists to protect.
- **Git notes.** Rejected. A default clone does not fetch notes, so the record fails at the one thing it exists to do.
- **A file mapping each unsigned commit to its pull request and approver.** Rejected. It is a second authored home for a fact GitHub already holds, and it grows for as long as the repository does.
- **Write each statement from the act that makes it true.** Chosen. Every other option asks one party to certify something it cannot know, or stores a fact in a field that means something else.

## Decision Outcome

Each liability is written from the act that makes it true, and it is written where that act can still be read once the pull request is gone.

| liability | written by | at | into |
|---|---|---|---|
| origin | the contributor | the moment the commit is made | `Signed-off-by:` |
| authorship | the agent | the moment the commit is made | the author field and `Assisted-by:` |
| acceptance | a workflow, from the approval | the moment the code owner approves | an `Accepted-by:` trailer on every commit |
| stewardship | the code owner | whenever ownership changes | `CODEOWNERS` |

**The contributor writes the origin trailer, and no workflow writes one.**
A machine cannot hold a right to submit anything, so a machine never makes this statement.
`git commit -s` takes the name and address from the configured identity, which makes checking that identity the contributor's first task rather than their last.

**A workflow writes the acceptance trailer, and no person writes one.**
It is derived from the review approval and never authored, so it cannot disagree with the approval it reports.
It is written after the approval, because acceptance is not true before then.
A push that leaves the tree unchanged does not dismiss a review, so one approval is enough and recording it does not cost a second.
It is written into every commit the pull request adds, and never into the head alone.
A commit without the trailer is a commit from before this decision, and a reader who has to tell those two apart is back at the problem stated above.

**The committer field is not the acceptance record.**
The agent is truthfully the author and the committer, because it writes the change and it makes the commit.
A rebase merge overwrites the committer with whoever merged, and a design that depends on that would store one true fact by destroying another.

**Three eras sit on `main`, and only the third is complete.**

| commits | carry |
|---|---|
| before `6d88f99` | a `Signed-off-by:` trailer naming the code owner, written by the agent |
| from `6d88f99` until this decision | nothing, and their answerability lives on their pull requests |
| after this decision | `Accepted-by:` from the approval |

`main` refuses a non-fast-forward push and admits no bypass actor, so the middle era cannot be repaired and is recorded instead.

The third era claims the acceptance trailer and not the origin one.
A contributor owes `Signed-off-by:` on every commit and nothing checks that they wrote it, so the table states what the repository does rather than what is asked of a contributor.
[Check that every commit carries the origin trailer its contributor owes](https://github.com/kalonji-tools/purba/issues/105) owns that gap.

**Downside:** three.

- **A contribution from a fork cannot carry the acceptance trailer.** Such a pull request gives the workflow a read-only token, the push is refused, and setting `maintainer_can_modify` does not change it. Each of those three was measured rather than reasoned. The mechanism therefore covers a contribution made inside this repository and not one made outside it, and the risk this record was built against sits outside it.
- **Nothing separates a person from an agent holding that person's credentials.** An approval on a deployment environment was the one act an agent could not perform, and this decision removes it. Whoever gives an agent access to their credentials is answerable for what the agent does with them. No check replaces that, and the Confirmation section grades the row `none` rather than implying otherwise.
- **The first account to approve is the one the trailer names.** The mechanism skips a commit that already carries the trailer, because without that it would rewrite the branch on every push and never settle. So an approval that is dismissed, and then given by a different person, leaves the first name in place. The pull request holds the second approval and the commit does not.

## Confirmation

| check | strength |
|---|---|
| the head commits carry a `Signed-off-by:` trailer naming a real account | none today |
| the trailer is parsed and not matched as text | none today |
| `Accepted-by:` names the account that approved the pull request | strong, it is derived from the approval and never authored |
| the approval survives the push that records it | strong, and measured: a push that leaves the tree unchanged does not dismiss a review |
| the acceptance trailer reaches `main` | strong, and measured: three commits of three reached `main` carrying it, from one approval |
| a person is distinguishable from their agent | **none, and this is deliberate** |
| the contributor read what they signed | none, and no check can make it |

The first two rows have no check.
[Check that every commit carries the origin trailer its contributor owes](https://github.com/kalonji-tools/purba/issues/105) owns one, and both rows say none until it lands.

The sixth row was once strong and is now none.
The environment approval was the only act here an agent could not perform, and removing it removes that separation.
An agent that holds a person's credentials can approve as that person, and whoever granted that access is answerable for what the agent does with it.

Any check that reads reviews must request every page.
The reviews endpoint returns 30 per page, and one pull request here reported zero approvals unpaginated while carrying one.

A check cannot bind the commit to the review by the reviewed commit id.
A rebase merge replays the commit under a new id, and the ids named by a review are absent from `main`.
The tree survives the replay.

The `sign` job in `.github/workflows/sign.yml` writes the trailer and reports the required status check `Sign-off`.
Read it back with `git log --format='%(trailers:key=Accepted-by)' <base>..HEAD`, which parses the trailer instead of matching it as text.

The measurements behind the rows above were made in a throwaway repository, and they are recorded on [Does the sign-off land in git, and in what order do liability, review and merge happen?](https://github.com/kalonji-tools/purba/issues/80) and on [Write the sign job that records acceptance in the commit](https://github.com/kalonji-tools/purba/issues/103).

**The mechanism that writes `Accepted-by:` cannot run on a pull request from a fork.**
The token is read only there, the push is refused, and allowing a maintainer to modify the branch does not change it.
An outside contribution therefore cannot carry this trailer.
[How does purba accept a contribution from a fork?](https://github.com/kalonji-tools/purba/issues/102) decides the path such a contribution takes.

The last row is the point of the whole mechanism, and no check can ever make it.
A detector good enough to suggest is not good enough to gate.
