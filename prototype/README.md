# Throwaway prototype: can the required context be posted by the job that creates the SHA?

**This branch is never merged.** It is captured as the primary source behind
[Write the sign job that records acceptance in the commit](https://github.com/kalonji-tools/purba/issues/103).

The workflows ran in `snregales/purba-proto-fork-token`, the same throwaway that
[#80](https://github.com/kalonji-tools/purba/issues/80) used. They sit under
`prototype/workflows/` rather than `.github/workflows/` so that nothing here runs.

Its ruleset was rewritten to mirror purba's `protect-main` before any arm ran:
`Sign-off` as the single required context, `strict_required_status_checks_policy`,
rebase-only, `dismiss_stale_reviews_on_push`, `require_last_push_approval: false`,
`require_extra_approval_for_unattributed_changes`, one approval.
**The admin bypass actor was removed**, so every merge below was a real merge.

| file | arms | question |
|---|---|---|
| `04-sign-fhi.yml` | F, G, H, I | can the job post the required context on the SHA it created? |
| `05-sign-j.yml` | J, K | does a second trigger clear the deadlock arm G found? |
| `06-sign-l.yml` | L | does replaying the branch put the trailer on every commit? |
| the shipped `.github/workflows/sign.yml` | M, N, O, P | does the file that ships behave, on both paths? |

## The control came first

Before any approval, pull request #5 read `mergeable_state: blocked`, carried
**zero** check runs, and `gh pr merge --rebase` was refused. Every positive
result below is measured against that.

## What it answered

**Arm F is positive, and the mechanism works on one human click.**
A check run the job posts under the name `Sign-off`, against the SHA the job
created by amending, satisfies the required context. `blocked` became `clean`,
the approval survived and re-pointed at the new head, and `gh pr merge --rebase`
returned 0 **without `--admin`**. `Accepted-by:` reached `main`.

**Arm G found the design's failure, and it is silent.**
Once `main` moves, `strict_required_status_checks_policy` reports `behind` and
refuses the merge. `update-branch --rebase` then gives the pull request new SHAs
carrying **zero check runs**, and **the approval is not dismissed** -- it survives
and re-points at the new head, even though the rebase changed the tree. No further
`pull_request_review` event can fire, so nothing re-posts the required context and
the pull request is stuck with an approval that looks current.

**Arm J clears it.** `synchronize` fires on `update-branch` and is the only event
left that sees the new head. The job finds the standing approval, sees the trailer
already present, re-posts the context, and the state returns to `clean`.

**Arm H is negative, and narrower than it looks.** An amend by an identity GitHub
cannot attribute (`sign@purba.invalid`, `committer: null` in the API) did **not**
trip `require_extra_approval_for_unattributed_changes`. One approval still read
`clean` and the merge returned 0. The *author* was an attributable account
throughout, so this measures the committer half only. It matters less than it
appears: the rebase merge replaces the committer with the merger, so whatever the
job writes there never reaches `main`.

**Arm I is a defect in the design, not in the mechanism.**
On a three-commit pull request only the head is amended, and that is what lands:

| commit on `main` | carries `Accepted-by:` |
|---|---|
| `316b181` arm I commit 1 of 3 | no |
| `ec06fa2` arm I commit 2 of 3 | no |
| `07abc8f` arm I commit 3 of 3 | **yes** |

Two of three commits reach `main` with no acceptance record, which is what an
era-two commit looks like. The decision record says acceptance attaches to a
merge; a rebase merge lands every commit.

**Arm K confirms it fails closed, and retires an old hazard.**
With no approval the job runs, finds nothing, and withholds the context: `Sign-off`
is absent, the state is `blocked`, and the merge is refused. Because the job is
named `sign-job` and the context is posted deliberately, **a job that succeeds or
skips can no longer satisfy the gate by writing a check run under its own name** --
which is the risk the current `signoff.yml` header spends a paragraph on.

**The job's own push does not loop.** It creates a `pull_request` run whose actor
is `github-actions[bot]`, with `conclusion: action_required`, **zero jobs and zero
check runs**. Measured under arm J's trigger set, where a loop was possible.

## What it is not

None of these files is the workflow purba should ship. The real job must also
refuse a pull request from a fork with a message naming why, and
[#102](https://github.com/kalonji-tools/purba/issues/102) decides that path.

**Two things were still not measured.** A pull request carrying more than one
approval, and a review dismissed and re-given while the trailer is already
present.

## Arm L, added after the head-only design was refused

Arm I refused the head-only design, so the job replays the branch instead of
amending its head:

```
git rebase "$base" --exec \
  'git log -1 --format="%(trailers:key=Accepted-by)" | grep -q . ||
     git commit --amend --no-edit --trailer "$TRAILER"'
```

**It is positive on every count.** A three-commit pull request was approved once.
All three commits took the trailer, the approval **survived the rewrite of every
SHA on the branch** and re-pointed at the new head, `Sign-off` was posted, and
`gh pr merge --rebase` returned 0 without `--admin`.

| commit on `main` | carries `Accepted-by:` |
|---|---|
| `eb86e54` arm L commit 1 of 3 | **yes** |
| `2590c66` arm L commit 2 of 3 | **yes** |
| `9fe9b65` arm L commit 3 of 3 | **yes** |

The `grep -q .` guard is what makes a re-run on `synchronize` idempotent: a commit
that already carries the trailer is replayed and not amended, so the SHAs settle
and the push stops.

`06-sign-l.yml` is the closest of these files to what purba should ship. It still
does not refuse a pull request from a fork.

## Arms M to P, run against the file that ships

**Arm M refused the obvious shape of the fork guard.** A fork pull request does
not fail where the design assumed. `actions/checkout` is given
`ref: head.ref`, that branch does not exist in this repository, and the job dies
there:

```
##[error]A branch or tag with the name 'arm-m' could not be found
```

The required context stayed absent and the merge was refused, so **the gate holds
with no refusal code at all**. But any refusal step placed after the checkout is
unreachable, and a contributor sees an opaque checkout error instead of a reason.

**Arm N: a second approval is a no-op.** The same account approved twice. Both
reviews were recorded, `last` selected the later one, and because the trailer was
already present the branch was not rewritten and the head did not move.

**Arms O and P ran the file this pull request ships, verbatim.**

| arm | shape | result |
|---|---|---|
| O | same repository, two commits | both commits carry the trailer, `Sign-off` posted |
| P | from a fork | step 2 refuses, **steps 3 and 4 are skipped**, `Sign-off` absent, `blocked` |

Arm P is the one that matters, because the refusal step had never run:

```
1.Set up job=success
2.Refuse a pull request from a fork=failure
3.Run actions/checkout@v4=skipped
4.Write the acceptance trailer into every commit=skipped
```

The refusal fires **before** the checkout, which is what arm M made necessary.
