# Contributing to purba

## What your sign-off certifies

Every commit you submit carries a `Signed-off-by:` trailer.

It certifies the [Developer Certificate of Origin, version 1.1](https://developercertificate.org/).

The version is named on purpose. A sign-off certifies the text as it stood
when it was written, so naming the version keeps that unambiguous if a later
one is ever published.

Write it with `git commit -s`. That takes the name and address from your
configured `user.name` and `user.email`, so check them before you start. A
sign-off names a person who can be reached.

No workflow writes this trailer for you. It certifies something only you know,
and it is written at the moment it becomes true.

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
