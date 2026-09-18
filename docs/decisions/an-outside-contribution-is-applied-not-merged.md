# An outside contribution is applied, not merged

## Context and Problem Statement

purba is public and `allow_forking` is `true`, so a pull request from a fork is the path an outside contributor takes.

[Liability is recorded from the act that makes it true](liability-is-recorded-from-the-act-that-makes-it-true.md) records acceptance as a trailer that a workflow writes from the approval.
That workflow holds a read-only token on a pull request from a fork.
It cannot write the trailer there, so the `sign` job refuses the branch and never posts the required status check.

The pull request therefore cannot merge, and nothing says what happens instead.
`sign.yml` tells the contributor that purba takes their contribution another way.
`CONTRIBUTING.md` does not say what that way is.

Five prototypes measured what a workflow can and cannot do across the fork boundary, and they are held by [How does purba accept a contribution from a fork?](https://github.com/kalonji-tools/purba/issues/102).

## Considered Options

- **Merge the pull request from the fork and write no acceptance trailer.** Rejected. A record with a documented hole has to be read beside its documentation before a reader can trust it, and the risk this mechanism was built against sits outside this repository.
- **Let a workflow write the trailer on the fork.** Rejected on measurement. The token is read only there whatever `permissions:` declares, and `maintainer_can_modify` grants a person rather than the Actions token.
- **Let a merge queue build the commits here, and have a job rewrite them.** Rejected on measurement. The queue branch refuses the push with `protected branch hook declined`, and the queue then merges the commit it built.
- **Have a workflow mirror the branch and open the pull request.** Rejected. A workflow opens a pull request only where the repository allows Actions to create and approve them, and that is one setting rather than two. Acceptance is derived from an approval here, so a workflow that can approve voids the record it would write.
- **Give the contributor write access to one branch prefix.** Rejected. Write access also carries the power to approve a pull request.
- **Have the maintainer write the trailer onto the fork branch, and a job verify it against the approval.** Measured, and it works. Not taken. It stops when the contributor clears "Allow edits from maintainers", and that option is absent on a fork owned by an organization, so it cannot stand alone. Running it beside the chosen option would give a maintainer two paths, and the better of the two fails on a condition purba does not control. purba has no outside contributor today. A contributor who wants the merge that GitHub records is what would take this up.
- **Have the maintainer apply the work to a branch in this repository.** Chosen. It needs no new mechanism, because the branch is an ordinary branch here and the gate treats it as one.

## Decision Outcome

An outside contribution is applied to a branch in this repository, and the pull request from the fork is a proposal.

| | |
|---|---|
| the pull request from the fork | a proposal, and it never merges |
| what merges | `accepted/pr-<n>` in this repository |
| who applies it | a maintainer, with `.github/scripts/apply-fork-contribution.sh` |
| who opens `accepted/pr-<n>` | the account that opens pull requests here, and never a code owner |
| the author field | the contributor's, unchanged |
| `Signed-off-by:` | the contributor's, unchanged |
| `Accepted-by:` | written by the `sign` job from the approval, as on any branch |

**The script refuses before it writes anything.**

It refuses a code owner, because GitHub refuses a self-approval and a code owner who opens the branch leaves a pull request nobody can approve.
It refuses a branch whose commits carry no origin trailer.

That second refusal is the only certain reading of the origin rule on this path.
A first-time contributor's workflow run waits for approval and writes no check at all.
The `sign` job reads the approval before it reads the trailers, so it writes nothing on a branch nobody has approved.

**A closing keyword links an issue and never a pull request.**
The maintainer closes the contribution after the merge, and the script prints the command that does it.

**Downside:** five.

- The contributor's pull request closes and does not merge. GitHub records no merge against their account.
- Each outside contribution costs the maintainer two acts. One command applies the work, and one closes the contribution after the merge.
- Nothing runs the script. A maintainer who does not use it carries in work that no gate has read.
- Until the gate rewrites the commits, the fork pull request and `accepted/pr-<n>` are one commit. A check run belongs to a SHA, so the mirror's green `sign` appears on the fork pull request. The gate is honest and the display is not.
- **Applying a contribution raises what its code is allowed to do.** A pull request from a fork runs a workflow with a read-only token, and a branch here runs one with this repository's own token, from the head of the pull request and before anyone approves it. A contribution that changes `.github/` therefore gains that token the moment it is applied. The script warns and does not refuse, because refusing would close this path to every contribution that improves the gate. `main` is unreachable from there, because its ruleset admits no bypass actor.

## Confirmation

**A pull request from a fork cannot merge.**
`sign.yml` refuses it before the step that posts `Sign-off`, the ruleset requires that context, and its `bypass_actors` is empty.
Measured: `gh pr merge --admin` on such a pull request returns `Repository rule violations found` and merges nothing.

**Nothing requires the script.**
The act it would check is the maintainer's own, and no check reaches it.

Read the result back on the applied branch with `git log --format='%(trailers:key=Accepted-by)' <base>..HEAD`, which parses the trailer rather than matching it as text.

⚠️ A contributor whose `author_association` is `NONE` was not measured.
Two accounts cannot hold the three roles this path needs at once, and the path reads no such field.
