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
| origin | the contributor | before the contribution goes for review | `Signed-off-by:` |
| authorship | the agent | the moment the commit is made | the author field and `Assisted-by:` |
| acceptance | a workflow, from the approval | the moment the code owner approves | an `Accepted-by:` trailer on every commit |
| stewardship | the code owner | whenever ownership changes | `CODEOWNERS` |

**The contributor writes the origin trailer, and no workflow writes one.**
A machine cannot hold a right to submit anything, so a machine never makes this statement.
`git commit -s` takes the name and address from the configured identity, which makes checking that identity the contributor's first task rather than their last.

**One exception, and it is a repository setting rather than a file.**
`web_commit_signoff_required` is on, so GitHub writes the trailer into a commit made in its web interface and names the account that made it.
The person is told before they commit, so the act is still theirs and the machine only transcribes it.
purba is the only repository in this organisation with the setting on, and a reader who audits files alone cannot see it.

**A workflow writes the acceptance trailer, and no person writes one.**
It is derived from the review approval and never authored, so it cannot disagree with the approval it reports.
It is written after the approval, because acceptance is not true before then.
A push that leaves the tree unchanged does not dismiss a review, so one approval is enough and recording it does not cost a second.
It is written into every commit the pull request adds, and never into the head alone.
A commit without the trailer is a commit from before this decision, and a reader who has to tell those two apart is back at the problem stated above.

**The committer field is not the acceptance record.**
The agent is truthfully the author and the committer, because it writes the change and it makes the commit.
A rebase merge overwrites the committer with whoever merged, and a design that depends on that would store one true fact by destroying another.

**Four eras sit on `main`, and only the fourth carries both statements.**

| commits | carry |
|---|---|
| before `6d88f99` | a `Signed-off-by:` trailer naming the code owner, written by the agent |
| from `6d88f99` until acceptance was recorded | nothing, and their answerability lives on their pull requests |
| from acceptance until the origin check | `Accepted-by:` from the approval |
| after the origin check | `Signed-off-by:` from the contributor, and `Accepted-by:` from the approval |

`main` refuses a non-fast-forward push and admits no bypass actor, so no era before the fourth can be repaired, and each is recorded instead.

The first era is the one to read carefully.
Its three trailers were written by the agent and they name the code owner, so they certify a right the writer could not hold.
That is the failure this decision exists to stop, and it is why the fourth era is not simply a return to the first.

**Downside:** five.

- **A contribution from a fork costs a manual step.** Such a pull request gives the workflow a read-only token, the push is refused, and setting `maintainer_can_modify` does not change it. Each of those three was measured rather than reasoned. A maintainer therefore applies an outside contribution to a branch here before it merges, so the mechanism covers it and the cost is an act rather than a gap.
- **Nothing separates a person from an agent holding that person's credentials.** An approval on a deployment environment was the one act an agent could not perform, and this decision removes it. Whoever gives an agent access to their credentials is answerable for what the agent does with them. No check replaces that, and the Confirmation section grades the row `none` rather than implying otherwise.
- **The first account to approve is the one the trailer names.** The mechanism skips a commit that already carries the trailer, because without that it would rewrite the branch on every push and never settle. So an approval that is dismissed, and then given by a different person, leaves the first name in place. The pull request holds the second approval and the commit does not.
- **The mechanism reports its own required check, and a pull request can change the mechanism.** GitHub runs the workflow from the head of the pull request, for the review event as well as for the push event, so the code that reports the check is code the change itself can edit. The approval an environment held could not be edited that way, and this decision removes it, so the loss is real rather than a restatement of the row above. `CODEOWNERS` covers `/.github/` for this reason, which makes the code owner read a change to the gate.
- **The mechanism forecloses commit signing.** It rewrites every commit the pull request adds, and an amended commit is a new commit object, so a signature a contributor made does not survive it. The job holds no key, so nothing signs again. GitHub breaks it a second time at the merge, documenting that Rebase and Merge adds commits without commit signature verification. A signature is the one record here that a person's agent could not forge, and this decision puts it out of reach for as long as the mechanism stands. [Can an act by a person be made impossible for their agent to forge?](https://github.com/kalonji-tools/purba/issues/106) owns what follows from that.

## Confirmation

| check | strength |
|---|---|
| the commits a pull request adds carry a `Signed-off-by:` trailer naming an address | strong, the `sign` job refuses the branch before it rewrites anything |
| the trailer is parsed and not matched as text | strong, `.github/scripts/check-origin.sh` reads the trailer rather than the message |
| the origin trailer was written by a person and not by their agent | **none** — nothing stops an agent running the command that adds it |
| `Accepted-by:` names the account that approved the pull request | strong, it is derived from the approval and never authored |
| the approval survives the push that records it | strong, and measured: a push that leaves the tree unchanged does not dismiss a review |
| the acceptance trailer reaches `main` | strong, and measured: three commits of three reached `main` carrying it, from one approval |
| a person is distinguishable from their agent | **none, and this is deliberate** |
| a change to the gate reaches the code owner | strong, `CODEOWNERS` covers `/.github/` and a code owner review is required |
| the contributor read what they signed | none, and no check can make it |

The two origin rows are checked by the `sign` job, which stops before it rewrites anything.
An advisory check reports the same rule on every push, and it is deliberately not a required one.
This job pushes its own rewrite.
GitHub creates the pull request runs on the head it writes and concludes them `action_required` with no jobs, so a context reported by an ordinary job is absent there and no event arrives to report it again.
That was measured rather than reasoned.

A trailer is written into a commit message, and a commit message is not in the tree, so the rewrite leaves the content of the branch untouched.
This job carries each verdict forward on that basis.
Where the tree at the head it writes equals the tree at the head it replaced, it reports the same conclusion again on the new head.
Where the two differ it carries nothing, and a person restarts the run instead.
A run nobody restarts leaves a pull request blocked until somebody notices.

The two cases above, the trees matching and the trees differing, were the whole rule. A third case occurred twice: the verdict is not in yet.
This job rewrote the branch before `Quality` had concluded, so it read a check with no verdict and carried nothing forward, although every step of `Quality` passed.
This job now waits for every required context it does not post itself before it rewrites anything, so the third case cannot arise.
`.github/scripts/require-green.sh` holds that rule, and a gate concluding is one of the events that starts this job.

`.github/scripts/carry-verdicts.sh` holds the rule, and it is given the contexts it may carry rather than deciding which ones qualify.
A gate that reads a commit message can never be carried, because the rewrite edits the thing that gate reads.
That is a property of such a gate and not a state to revisit.
It is the second reason `Origin` stays advisory, beside the one that already stood: the `sign` job refuses on the origin rule before it rewrites anything, so requiring the check as well would buy nothing.

A required context is therefore possible beside this one, and the quality workflow holds one.

The row on who wrote the origin trailer is the ceiling of the whole mechanism.
An agent can run `git commit -s` as easily as a person can, so the check reads a trailer and never the act behind it.
It is graded `none` for the same reason the row below it is, and [can an act by a person be made impossible for their agent to forge?](https://github.com/kalonji-tools/purba/issues/106) is where that is decided.

The row on telling a person from their agent was once strong and is now none.
The environment approval was the only act here an agent could not perform, and removing it removes that separation.
An agent that holds a person's credentials can approve as that person, and whoever granted that access is answerable for what the agent does with it.

Any check that reads reviews must request every page.
The reviews endpoint returns 30 per page, and one pull request here reported zero approvals unpaginated while carrying one.

A check cannot bind the commit to the review by the reviewed commit id.
A rebase merge replays the commit under a new id, and the ids named by a review are absent from `main`.
The tree survives the replay.

A push that changes the tree dismisses the approval, and that was measured rather than reasoned.
So `dismiss_stale_reviews_on_push` reads the tree and not the commit, and the rule this job applies to a verdict is the rule GitHub applies to an approval.

The `sign` job in `.github/workflows/sign.yml` refuses a branch whose commits lack the origin trailer, then writes the acceptance trailer and reports the required status check `Sign-off`.
Read it back with `git log --format='%(trailers:key=Accepted-by)' <base>..HEAD`, which parses the trailer instead of matching it as text.

The measurements behind the rows above were made in a throwaway repository, and they are recorded on [Does the sign-off land in git, and in what order do liability, review and merge happen?](https://github.com/kalonji-tools/purba/issues/80) and on [Write the sign job that records acceptance in the commit](https://github.com/kalonji-tools/purba/issues/103), and on [check that every commit carries the origin trailer its contributor owes](https://github.com/kalonji-tools/purba/issues/105).

**The mechanism that writes `Accepted-by:` cannot run on a pull request from a fork.**
The token is read only there, the push is refused, and allowing a maintainer to modify the branch does not change it.
[An outside contribution is applied, not merged](an-outside-contribution-is-applied-not-merged.md) carries such a contribution to a branch here first, where this mechanism writes the trailer as it does on any other branch.

The row on whether the contributor read what they signed is the point of the whole mechanism, and no check can ever make it.
A detector good enough to suggest is not good enough to gate.
