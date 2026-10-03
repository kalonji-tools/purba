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
A rebase merge overwrites the committer with whoever merged, so it stores one true fact and destroys another.
No signature is available as a third place: GitHub does not sign what a rebase merge replays.

## Considered Options

- **Keep answerability only at the approval.** Rejected. It does not survive a clone, and the mechanism fails on a pull request from a fork, which is the path an outside contributor uses.
- **Treat the committer field as the acceptance record.** Rejected. The field already states who made the commit, and a design that depends on a merge overwriting it is a coincidence rather than a mechanism.
- **Two human approvals, one on a deployment environment and one on the pull request.** Rejected. A push that leaves the tree unchanged does not dismiss a review. One approval is therefore enough, and the second act records nothing the first does not.
- **Squash merge, to gain control of the commit message.** Rejected. It works, and the ruleset permits a linear history under squash, so this is refused on cost rather than on feasibility. It destroys the commit granularity that [a commit message outlives its review](a-commit-outlives-its-review.md) exists to protect.
- **Git notes.** Rejected. A default clone does not fetch notes, so the record fails at the one thing it exists to do.
- **A file mapping each unsigned commit to its pull request and approver.** Rejected. It is a second authored home for a fact GitHub already holds, and it grows for as long as the repository does.
- **Write each statement from the act that makes it true.** Chosen. Every other option asks one party to certify something it cannot know, or stores a fact in a field that means something else.

## Decision Outcome

Each liability is written from the act that makes it true.
It is written where that act can still be read once the pull request is gone.

| liability | written by | at | into |
|---|---|---|---|
| origin | the contributor | as the commit is made, or once the reviewer is satisfied | `Signed-off-by:` |
| authorship | the agent | the moment the commit is made | the author field and `Assisted-by:` |
| acceptance | a workflow, from the approval | the moment the code owner approves | an `Accepted-by:` trailer on every commit |
| stewardship | the code owner | whenever ownership changes | `CODEOWNERS` |

**The contributor writes the origin trailer, and no workflow writes one.**
A machine cannot hold a right to submit anything, so a machine never makes this statement.
`git commit -s` takes the name and address from the configured identity, which makes checking that identity the contributor's first task rather than their last.

**One exception, and it is a repository setting rather than a file.**
`web_commit_signoff_required` is on, so GitHub writes the trailer into a commit made in its web interface and names the account that made it.
The person is told before they commit, so the act is still theirs and the machine only transcribes it.
GitHub documents that as reaching every commit made there. The merge commit that brings a branch current is made there and carries no trailer.
purba is the only repository in this organisation with the setting on, and a reader who audits files alone cannot see it.

**A workflow writes the acceptance trailer, and no person writes one.**
It is derived from the review approval, and a trailer the branch already carried is replaced rather than kept.
What reaches `main` cannot disagree with the approval it reports.
It is written after the approval, because acceptance is not true before then.

A push that leaves the tree unchanged does not dismiss a review, so one approval is enough and recording it does not cost a second.
It is written into every commit the pull request adds, and never into the head alone.
A commit without the trailer is a commit from before this decision.
A reader who has to tell those two apart is back at the problem stated above.

**The committer field is not the acceptance record.**
The agent is truthfully the author and the committer, because it writes the change and it makes the commit.
A rebase merge overwrites the committer with whoever merged, and a design that depends on that would store one true fact and destroy another.

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

**Downside:**

- **A contribution from a fork costs a manual step.** Such a pull request gives the workflow a read-only token, the push is refused, and setting `maintainer_can_modify` does not change it. Each of those three was measured rather than reasoned. A gate finishing starts the same workflow from `main`, where the token is this repository's own. There the first step of the `sign` job refuses the fork instead. A maintainer therefore applies an outside contribution to a branch here before it merges. The mechanism covers it, and the cost is an act rather than a gap.
- **Nothing separates a person from an agent holding that person's credentials.** An approval on a deployment environment was the one act an agent could not perform, and this decision removes it. Whoever gives an agent access to their credentials is answerable for what the agent does with them. No check replaces that, and the Confirmation section grades the row `none` and implies nothing else.
- **The mechanism reports its own required check, and a pull request can change the mechanism.** GitHub runs the workflow from the head of the pull request, for the review event as well as for the push event. The code that reports the check is therefore code the change itself can edit. A gate finishing is the exception, because that run takes the workflow from `main`. A pull request changes the scripts the job calls, and not the file that calls them. The approval an environment held could not be edited that way, and this decision removes it. The loss is real rather than a restatement of the row above. `CODEOWNERS` covers `/.github/` and `/scripts/` for this reason, which makes the code owner read a change to the gate.
- **The mechanism forecloses commit signing.** It rewrites every commit the pull request adds, and an amended commit is a new commit object. A signature a contributor made does not survive it. The job holds no key, so nothing signs again. GitHub breaks it a second time at the merge, documenting that Rebase and Merge adds commits without commit signature verification. A signature would not separate a person from their agent in any case. It proves custody of a key, and a key kept where the agent runs is a key the agent uses. What follows from that is decided below.

## Confirmation

| check | strength |
|---|---|
| the commits a pull request adds carry a `Signed-off-by:` trailer naming an address | strong, the `sign` job refuses the branch before it rewrites anything |
| the trailer is parsed and not matched as text | strong, `scripts/check-origin.sh` reads the trailer rather than the message |
| the origin trailer was written by a person and not by their agent | **none**, and nothing stops an agent running the command that adds it |
| `Accepted-by:` names the account that approved the pull request | strong, and measured: the job replaces a trailer that names anyone else, so a branch cannot carry one in |
| the approval survives the push that records it | strong, and measured: a push that leaves the tree unchanged does not dismiss a review |
| the rewrite leaves the content of the branch untouched | strong, `scripts/check-replayable.sh` refuses a branch whose replay changes the tree, and the `sign` job runs it before it rewrites |
| the acceptance trailer reaches `main` | strong, and measured: three commits of three reached `main` carrying it, from one approval |
| a person is distinguishable from their agent | **none, and no mechanism reachable here can make it**, because the agent runs where the credentials live |
| a change to the gate reaches the code owner | strong, `CODEOWNERS` covers `/.github/` and `/scripts/`, and a code owner review is required |
| a pull request from a fork never reaches the checkout in the signing job | strong by construction, and never exercised, because the refusal is the first step and no such pull request exists here |
| the contributor read what they signed | none, and no check can make it |

The two origin rows are checked by the `sign` job, which stops before it rewrites anything.
An advisory check reports the same rule on every push, and it is deliberately not a required one.
This job pushes its own rewrite.
GitHub creates the pull request runs on the head it writes, and concludes them `action_required` with no jobs.
A context reported by an ordinary job is absent there, and no event arrives to report it again.
That was measured rather than reasoned.

A trailer is written into a commit message, and a commit message is not in the tree.
That alone does not keep the content: the replay flattens a merge commit, and the changes made in that merge are lost.
So the job refuses a branch whose replay changes the content.
The rewrite leaves the content of the branch untouched, because nothing it admits can change it.

This job carries each verdict forward on that basis.
The tree at the head it writes may equal the tree at the head it replaced.
It then reports the same conclusion again on the new head.
A conclusion is reported again only where it answers for the tree.

GitHub draws a check run's conclusion from a fixed set, and `success` and `failure` are the two members of it that answer.
The rest report what happened to the run, so this job names the conclusion it refused and leaves the context absent.
A run nobody restarts leaves a pull request blocked until somebody notices.
Where the two trees differ it carries nothing.
The `sign` job cannot reach that case, because it refuses a branch whose replay changes the content before it rewrites anything.
This script is given two heads and cannot know what produced them, so it reads the trees and does not trust its caller.

The two cases above, the trees matching and the trees differing, were the whole rule. A third case occurred twice: the verdict is not in yet.
This job rewrote the branch before `Quality` concluded.
It read a check with no verdict and carried nothing forward, although every step of `Quality` passed.
This job now waits for every required context it does not post itself before it rewrites anything, so the third case cannot arise.
`.github/scripts/require-green.sh` holds that rule, and a gate concluding is one of the events that starts this job.

`.github/scripts/carry-verdicts.sh` holds the rule, and it is given the contexts it may carry and does not decide which ones qualify.
A gate that reads a commit message can never be carried, because the rewrite edits the thing that gate reads.
That is a property of such a gate and not a state to revisit.
It is the second reason `Origin` stays advisory, beside the one that already stood.
The `sign` job refuses on the origin rule before it rewrites anything, so the check would buy nothing as well.

A required context is therefore possible beside this one, and the quality workflow holds one.

The row on who wrote the origin trailer is the ceiling of the whole mechanism.
An agent can run `git commit -s` as easily as a person can, so the check reads a trailer and never the act behind it.

The row that tells a person from their agent was once strong and is now none.
The environment approval was the only act here an agent could not perform, and removing it removes that separation.
An agent that holds a person's credentials can approve as that person.
Whoever granted that access is answerable for what the agent does with it.

**No act recorded here can be made impossible for that person's agent to forge.**
Three routes lead out of it, and each one is closed.
A key kept where the agent runs is a key the agent uses, so a signature proves custody rather than personhood.
No credential is out of the agent's reach either, because the agent runs on the machine that holds them.
The block GitHub documents on an approval of one's own pull request keys on the author rather than on the approver.

A hardware key that demands a touch for each signature is genuinely out of reach, and git cannot ask for one.
The allowed-signers format `ssh-keygen(1)` documents admits `cert-authority`, `namespaces`, `valid-after` and `valid-before`.
None of them concerns the presence of a person.
Each route was measured rather than reasoned, and [an act by a person cannot be made unforgeable here, because the agent holds the human's credentials](https://github.com/kalonji-tools/purba/issues/204) records every measurement beside the source it rests on.

**So the question is not whether a trailer can be forged, but whether the procedure preserves the intent it records.**
git's own answers to frequently asked questions refuse a `commit.signoff` setting for that reason.
They hold that an automated sign-off would let someone argue later that the trailer was added out of habit rather than to certify anything.
`scripts/sign-branch.sh` is the answer this project already holds.
It refuses a caller with no terminal, prints every commit and who wrote it, and waits to be answered.
It cannot tell who typed the answer, and it claims no more than that.

**purba therefore records delegation and does not prevent it.**
An agent acts for a person here, and that person remains answerable for what it does.
This record says so, and implies no separation that no check makes.

One consequence is a property of this repository and of no other.
Its own git config names the agent as the committer, so `git commit -s` run here writes the agent's sign-off.
`CONTRIBUTING.md` addresses a contributor whose committer identity is already their own.
That is how this repository is worked rather than a defect in the document, and nothing here repairs it.

Any check that reads reviews must request every page.
The reviews endpoint returns 30 per page, and one pull request here reported zero approvals unpaginated when it carried one.

A check cannot bind the commit to the review by the reviewed commit id.
A rebase merge replays the commit under a new id, and the ids named by a review are absent from `main`.
The tree survives the replay.

A push that changes the tree dismisses the approval, and that was measured rather than reasoned.
So `dismiss_stale_reviews_on_push` reads the tree and not the commit.
The rule this job applies to a verdict is the rule GitHub applies to an approval.

The `sign` job in `.github/workflows/sign.yml` refuses a branch whose commits lack the origin trailer.
It then writes the acceptance trailer and reports the required status check `Sign-off`.
Read it back with `git log --format='%(trailers:key=Accepted-by)' <base>..HEAD`, which parses the trailer and does not match it as text.

The measurements behind the rows above were made in a throwaway repository, and they are recorded on [Does the sign-off land in git, and in what order do liability, review and merge happen?](https://github.com/kalonji-tools/purba/issues/80) and on [Write the sign job that records acceptance in the commit](https://github.com/kalonji-tools/purba/issues/103), and on [check that every commit carries the origin trailer its contributor owes](https://github.com/kalonji-tools/purba/issues/105).

The exception above was measured on this repository, because the setting is a property of it, and it is recorded on [Two claims about bringing a branch current are false](https://github.com/kalonji-tools/purba/issues/166).

**The mechanism that writes `Accepted-by:` cannot run on a pull request from a fork.**
On the pull request event the token is read only, and the push is refused.
A maintainer allowed to modify the branch does not change it.
On the gate-completed event none of that holds, because the run starts from `main` and takes the permissions the workflow declares.

What refuses the fork there is the first step of the `sign` job.
It reads the repository the head belongs to, and stops before the checkout.
That step never ran, because no pull request from a fork exists in this repository.
[An outside contribution is applied, not merged](an-outside-contribution-is-applied-not-merged.md) carries such a contribution to a branch here first, where this mechanism writes the trailer as it does on any other branch.

The row on whether the contributor read what they signed is the point of the whole mechanism, and no check can ever make it.
A detector good enough to suggest is not good enough to gate.
