# Throwaway prototype: can a workflow write the acceptance trailer?

**This branch is never merged.** It is captured as the primary source behind
[Liability is recorded from the act that makes it true](https://github.com/kalonji-tools/purba/blob/main/docs/decisions/liability-is-recorded-from-the-act-that-makes-it-true.md),
[Does the sign-off land in git?](https://github.com/kalonji-tools/purba/issues/80)
and [How does purba accept a contribution from a fork?](https://github.com/kalonji-tools/purba/issues/102).

The workflows ran in a throwaway repository, `snregales/purba-proto-fork-token`,
which exists only to be deleted. They sit under `prototype/workflows/` rather
than `.github/workflows/` so that nothing here runs.

| file | arm | question |
|---|---|---|
| `01-token-probe.yml` | A | is a fork pull request's `GITHUB_TOKEN` writable? |
| `02-sign-probe.yml` | D | can a workflow amend a commit and push the result? |
| `03-sign-on-approval.yml` | E | does that push dismiss the review it was derived from? |

## Why it could not run in purba

The question needs a contributor with **no write access**. Both accounts on purba
are OWNER or COLLABORATOR, so the permission relationship would have been
confounded, exactly as the C toolchain was in the mise prototype. The throwaway
was owned by the human, with the agent account deliberately not a collaborator,
which produced `author_association: NONE`.

## What it answered

**Five findings, three of which changed the design.**

1. **A fork pull request's token is read only**, whatever `permissions:` declares:
   `403 Resource not accessible by integration`. A `git push` to the fork branch
   is refused with exit 128. `maintainer_can_modify: true` does **not** help,
   because it grants a person and not the Actions token.
2. **A workflow can do it on a same-repo branch.** The push succeeded and the
   trailer landed. The design had deleted this job on the claim that it was
   impossible; the measured claim is narrower.
3. **`dismiss_stale_reviews_on_push` is tree-sensitive, not commit-sensitive.** A
   force-pushed amend leaving the tree identical keeps the review `APPROVED`, and
   GitHub re-points the review's `commit_id` at the new head. This removed the
   second human approval from the design.
4. **An amended head carries zero check runs.** The old head carried two. A job
   that amends must post its own check run against the SHA it created, or a
   required context never reports and the merge is refused forever.
5. **A first-time contributor's run is blocked** with `conclusion: action_required`
   and produces no check runs at all.

Rebase merge was measured separately: the tree survives the replay unchanged, the
author is preserved, the committer becomes the merger, trailers survive byte for
byte, and the replayed commit is **not signed**.

## What it is not

Each workflow reports a refusal rather than failing, because a refusal is the
measurement. None of them is a design for purba: the real job must also verify
the approver, post its own check run, and refuse a fork pull request with a
message naming why.

**Three things were not measured.** `main` never moved, so rebasing onto a moved
`main` is untested. Every pull request carried one commit. And no arm tested a job
posting a check run under a **required** context name, which is the load-bearing
step of the design.
