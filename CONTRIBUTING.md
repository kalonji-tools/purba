# Contributing to purba

## What your sign-off certifies

Every commit you submit carries a `Signed-off-by:` trailer.

It certifies the [Developer Certificate of Origin, version 1.1](https://developercertificate.org/).

The version is named on purpose. A sign-off certifies the text as it stood
when it was written, so naming the version keeps that unambiguous if a later
one is ever published.

**If you write your own commits**, add it as you go with `git commit -s`.

**If an agent writes them for you**, it cannot add this trailer, so you add it
to the whole branch yourself before you ask anyone to review it:

```
mise run sign-off
```

It lists every commit it will sign and who wrote each one, and signs them once
you agree. Nothing is signed if you do not, or if no terminal is attached. A
machine never writes this trailer, and whether a person is present is the one
part of that rule a file here can test.

It replays your branch on a copy before it asks, so a branch purba cannot
replay is refused and left where it is. The three shapes it refuses are named
under *What refuses your branch* below.

The list is measured from `origin/main`. Name another base when your pull
request targets one, with `mise run sign-off origin/<branch>`. Measured from a
base your pull request does not target, the range reaches back past that
branch and offers you commits somebody else wrote.

The replay lands your branch on that base, so it brings a branch that has
fallen behind current as well.

Running it again adds nothing. `git commit -s` on its own would: it adds a
trailer whenever the sign-off is not the last one, and purba writes its own
trailer after yours.

Either way the name and address come from your committer identity.
`user.name` and `user.email` set that identity unless you override them, so
check it before you start. A sign-off names a person who can be reached.

purba replaces the committer field on the way to `main`. Your sign-off is not
touched, so the address you sign with is the one that lasts.

No workflow writes this trailer for you, with one exception. GitHub adds it to
a commit you make in its web interface, because this repository asks it to. It
tells you so before you commit, so the act is still yours.

Everywhere else it certifies something only you know, and it is written at the
moment it becomes true.

A branch that reaches review without it is refused, and the refusal names the
commits that lack it.

## If a machine helped

State it, in the same commit, with an `Assisted-by:` trailer.

```
Assisted-by: AGENT:MODEL [TOOL] [TOOL]
```

The optional bracketed entries name specialised analysis tools, such as a
linter or a static analyser, that contributed to the change.

Disclosure and certification are different statements and neither replaces the
other. A machine cannot hold a right to submit anything, so a machine never
writes a `Signed-off-by:` trailer. You remain answerable for a contribution a
machine helped you write, on exactly the terms above.

## What purba records on its side

You certify the origin of your contribution. purba records that it accepted
it, and does so in two places.

| record | where |
|---|---|
| the person who accepted the change into `main` | an `Accepted-by:` trailer on every commit |
| the approval that released it | the pull request |

Both are purba's to write. Neither is yours to supply.

A workflow writes the trailer from the approval, so it cannot disagree with
the approval it reports. It rewrites your commits to add it, which is why the
branch you pushed and the branch that merges have different SHAs.

## How your work reaches `main`

If you can push to this repository, you push a branch here and your pull
request merges. Everything above applies to it unchanged.

If you work from a fork, your pull request is a proposal. It does not merge.

A workflow here holds a read-only token on a pull request from a fork, so it
cannot write purba's side of the record onto a branch that lives on your
account. It also refuses such a pull request outright, before it reads
anything, so the refusal does not depend on what that token allows. purba
carries your work in instead.

A maintainer copies your commits to a branch in this repository and opens a
pull request for it. That branch is what gets reviewed and what merges.

Your commits are not changed on the way. The author field stays yours and so
does your `Signed-off-by:` trailer. `Accepted-by:` is written on purba's
branch and never on yours.

A maintainer closes your pull request once the work has merged, and names the
branch that carried it.

### What refuses your branch

These rules apply to the branch that merges, whichever path carried it.

One approving review releases it. A code owner's review is required as well
where `CODEOWNERS` names the path, and it names `/docs/decisions/` and
`/.github/`.

Every review conversation must be resolved. A push that changes the tree
dismisses an approval already given, so an approval follows the code rather
than the branch.

Two checks are required. `Quality` refuses a lint, a formatting difference, a
broken link in the Rust documentation and a failing example. `Sign-off`
records that purba accepted the branch. Your branch must also be current with
`main`, so a branch that has fallen behind is rebased and reviewed against
what is there now.

Bring it current by replaying your commits onto the base, not by merging the
base into them. A merge commit carries no sign-off, and no trailer can be
added to one, so purba refuses it. The sign-off command above replays. So does
the **Update with rebase** entry GitHub offers on the pull request.

`Sign-off` arrives after the other checks rather than with them. purba signs
once an approval stands and every other required check is green. On a branch
whose `Quality` is still running, your approval lands and nothing appears to
happen until the gate finishes.

Where purba refuses your branch instead, `Sign-off` turns red and names what
it found. It is written before anything on your branch is rewritten, so the
branch you are looking at is the branch the refusal is about.

purba replays your branch onto its base to write its `Accepted-by:` trailer,
so every commit on the branch must survive that replay. Three kinds do not:

- a commit that replays empty
- a commit that no longer applies where the replay puts it
- a merge commit that made a change of its own, because the replay flattens
  the merge and the changes made in it are lost

A `Replay` check reports this once your pull request is open, and the refusal
names what it found.

`Quality` needs nothing from you after the approval. purba rewrites your
commits at that point and pushes them, and GitHub creates the checks on
that new commit without running them. A workflow here carries each verdict
across, because the rewrite edits commit messages, and purba refuses any
branch whose replay would change your content.

The runs GitHub holds on that commit carry no jobs, and a run with no jobs
reports no check, so nothing waits on them. The pull request offers to approve
them anyway, and marks that offer with a warning.

A commit GitHub cannot attribute to an account costs a further approval.

The merge is a rebase. Squash and merge commits are both off, so your commits
land one at a time and each message survives as you wrote it.

`main` keeps a linear history. It refuses a force push, and it cannot be
deleted.

If this is your first contribution here, GitHub holds your workflow run until
a maintainer approves it, and no check reports before that.

These rules live in the repository's settings and not in a file here, so read
them back rather than trusting this section:

```
gh api repos/kalonji-tools/purba/rules/branches/main
```

That call needs no token, because this repository is public.
