# A required check of `main` counts only from GitHub Actions

## Context and Problem Statement

The ruleset `protect-main` requires `Sign-off`, `Quality` and `Build` before a merge.
A required check can be a check run or a commit status.
An account with write access can post a commit status of any name on any commit.
Where a ruleset names no app for a required check, the ruleset accepts that status.
No diff shows the status.

## Considered Options

- **Each required check counts from any source.** An account with write access meets `Sign-off` with one commit status. A commit can then reach `main` without the acceptance trailer.

## Decision Outcome

**Reach:** `.github/workflows/sign.yml` `.github/workflows/quality.yml` `.github/workflows/build.yml` `.github/scripts/write-acceptance-trailer.sh` `.github/scripts/check-run.sh` `.github/scripts/carry-verdicts.sh` `/CONTRIBUTING.md`

**Area:** workflow

**`protect-main` accepts `Sign-off`, `Quality` and `Build` only from GitHub Actions.**
`Quality` and `Build` are the check runs of their jobs.
The `sign` job posts both again, with `GITHUB_TOKEN`, on the head that it writes.
It posts `Sign-off` with the same token.
Each comes from GitHub Actions, so `protect-main` refuses none of purba's own checks.
[Liability is recorded from the act that makes it true](liability-is-recorded-from-the-act-that-makes-it-true.md) says what `Sign-off` records.

**Downside:**

- **A workflow on any branch still reports a required check.** An account with write access pushes a branch with no pull request. A workflow on that branch posts `Sign-off` on the head of another pull request. That check run comes from GitHub Actions, so `protect-main` accepts it. No review reads that branch. A commit of the other pull request can then reach `main` without the acceptance trailer.

## Confirmation

These commands show that `protect-main` names GitHub Actions for each check.

| command | prints |
|---|---|
| `gh api repos/kalonji-tools/purba/rulesets/22774939 --jq '.rules[].parameters.required_status_checks // empty'` | `integration_id` 15368 for `Sign-off`, `Quality` and `Build` |
| `gh api apps/github-actions --jq .id` | 15368 |

No gate on a pull request reads `protect-main`, because no ruleset is in the tree.

[A commit status from any account with write access meets a required check of main](https://github.com/kalonji-tools/purba/issues/814) holds the measurements from a sandbox ruleset.
[A workflow on a branch with no pull request reports a required check of main](https://github.com/kalonji-tools/purba/issues/887) asks how `protect-main` can refuse a check run from a workflow on a branch other than the pull request's.
