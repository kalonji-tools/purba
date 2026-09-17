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
git rebase "$(git merge-base origin/main HEAD)" --exec \
  'git log -1 --format="%(trailers:key=Signed-off-by)" | grep -q . ||
     git commit --amend --no-edit -s'
```

That is one act and it covers every commit on the branch.

The guard matters. `git commit -s` adds nothing only when the sign-off is the
last trailer. purba adds its own trailer after yours, so a second run without
the guard leaves you with two sign-off lines.

Either way the name and address come from your configured `user.name` and
`user.email`, so check them before you start. A sign-off names a person who
can be reached.

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
