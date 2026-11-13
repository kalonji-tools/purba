# The `sign` job signs only the pull request into `main`

## Context and Problem Statement

The ruleset `protect-main` requires `Sign-off` on a pull request into `main`.
A check run belongs to a commit.
So a `Sign-off` on a commit shows on each pull request whose head is that commit.

A workflow run of a gate lists open pull requests at its head.
GitHub states no order for that list.
GitHub allows no more than one open pull request for each head branch and base.
So two branches at one commit can each hold an open pull request into `main`.

A probe in a sandbox repository measured which pull request each read in the table gives.

| pull requests from one branch | the first that the workflow run lists | the open pull request from the branch into `main` |
|---|---|---|
| A into `main`; then B into another base | B | A |
| C into another base | C | none |
| C, then its base moved to `main` | C | C |

The probe also opened a pull request into `main` from a second branch at the head of A.

## Considered Options

- **Sign the first pull request that the workflow run lists.** A `Sign-off` for a pull request into another base then stands on the pull request into `main` at the same head. When the first is not the pull request into `main` and no approval of it stands, the job writes nothing for the one into `main`.
- **Refuse a head that another open pull request shares.** A pull request into another base at the same head holds back the one into `main`. A pull request whose base moves to `main` keeps the `Sign-off` from its old base.

## Decision Outcome

**Reach:** `.github/workflows/sign.yml` `.github/scripts/write-acceptance-trailer.sh` `.github/scripts/check-run.sh` `scripts/test/write-acceptance-trailer.bats` `scripts/test/check-run.bats` `scripts/test/fixture.sh` `/CONTRIBUTING.md`

**Area:** workflow

**The `sign` job signs only the open pull request into `main` at the head of the workflow run.**
`protect-main` reads `Sign-off` only on a pull request into `main`.

- **Two open pull requests into `main` with one head get a failed `Sign-off` from the `sign` run of either.** A `Sign-off` on that head stands on each of them.

`sign.yml` gives the job the default branch as `BASE_REF`.
`write-acceptance-trailer.sh` asks GitHub for the open pull request from the branch of the workflow run into `BASE_REF`.
It signs that pull request.
When no such pull request is open, it writes nothing.

It also reads the pull requests whose branch holds the head.
It counts only those whose own head is that commit.
When it counts more than one open pull request into `BASE_REF`, the job posts a failed `Sign-off` that names them.

[`main` requires only `Sign-off`, and accepts it only from `purba-sign`](a-required-check-of-main-counts-only-from-github-actions.md) says which check `protect-main` requires.

**Downside:**

- **A pull request into another branch gets no `Sign-off`.** A probe of the `sign` job needs a pull request into `main`.
- **A move of the base of a pull request to `main` starts no workflow run.** That pull request has no `Sign-off` until a push, or a close and a reopen.
- **One open pull request into `main` can hold back another.** The failed `Sign-off` stays on a pull request until it gets a push. It also goes when the other one closes or leaves that commit, and this one gets a close and a reopen.
- **A new pull request into `main` at a signed head shows that `Sign-off` until its own `sign` run.** A pull request from a fork gets no `sign` run of its own. So no run of its own replaces that `Sign-off`. `protect-main` still requires an approval of the new one.
- **The `Sign-off` of a closed pull request stands on a new pull request into `main` at the same commit.** `protect-main` still requires an approval of the new one. The `Accepted-by:` trailer names the approver of the closed one.

## Confirmation

`scripts/test/write-acceptance-trailer.bats` checks that the job asks GitHub for the open pull request into `main`, and never reads the pull request that the environment names.
It also gives the job no open pull request into `main`, and expects the job to write nothing.
It gives the job two open pull requests into `main` with one head, and expects a failed `Sign-off`.
It gives one more whose branch only holds the head, and expects the job to sign.
`mise run test:shell` runs it.

[A Sign-off that the sign job writes for one pull request stands on each pull request at the same head](https://github.com/kalonji-tools/purba/issues/949) holds the probe.
