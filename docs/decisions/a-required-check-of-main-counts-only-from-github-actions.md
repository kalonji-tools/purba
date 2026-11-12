# `main` requires only `Sign-off`, and accepts it only from `purba-sign`

## Context and Problem Statement

The ruleset `protect-main` names the checks that a merge into `main` needs.
A ruleset can name the app that must report a check, and it tells one app from another.
It cannot tell one workflow of GitHub Actions from another.
A workflow that may write checks posts a check run of any name on any commit, and GitHub Actions reports it.
An account with write access can push such a workflow to a branch that no pull request shows, so no reviewer sees it.
The `sign` job also read a gate by its name alone, so such a check run could stand for a gate.

## Considered Options

- **Each required check counts from GitHub Actions.** A workflow on any branch meets `Sign-off` with one call. A commit can then reach `main` without the acceptance trailer.
- **Leave that way open, and say that `Sign-off` stops no account with write access.** `Sign-off` then refuses no merge for that account, and the change rewrites each line that says it does.

## Decision Outcome

**Reach:** `.github/CODEOWNERS` `.github/workflows/sign.yml` `.github/workflows/ci.yml` `.github/scripts/check-run.sh` `.github/scripts/write-acceptance-trailer.sh` `.github/scripts/carry-verdicts.sh` `/CONTRIBUTING.md`

**Area:** workflow

**`protect-main` requires only `Sign-off`, and accepts it only from the GitHub App `purba-sign`.**
The key of `purba-sign` is a secret of the environment `sign`, and that environment lets in only a job that runs from `main`.
So a workflow on any other branch, or one that a pull request runs, cannot post a `Sign-off` that `protect-main` accepts.

- **The `sign` job runs only when a gate finishes.** That run takes `sign.yml` from `main`. It also runs the scripts of `main`. Those scripts only rewrite the commits of the branch.
- **The `sign` job reads the verdict of a gate only from a job that ran on a runner, or from `purba-sign`.** That job belongs to a run of the gate's workflow file, on `pull_request`, on the branch of the pull request. The name of the gate names that file: `CI` is a job of `ci.yml`. A check run that a workflow posts through the API joins the jobs of a run, and it has no runner.
- **`CI` stops a merge through `Sign-off`.** The `sign` job signs no head while `CI` is red, and `CI` is red while a job it requires is red.
- **`purba-sign` may write checks and read metadata, and nothing else.** For `purba-sign`, the environment is the boundary, because a check run is the very thing a forger writes.

[Liability is recorded from the act that makes it true](liability-is-recorded-from-the-act-that-makes-it-true.md) says what `Sign-off` records.
[A workflow is granted an identity, not a power](a-workflow-is-granted-an-identity-not-a-power.md) says why an App is the identity.

**Downside:**

- **A second pull request from the reviewed branch makes a run that reads as a gate.** An account with write access pushes a base branch whose gate workflow always passes. It then opens a pull request from the reviewed branch into that base. Its jobs run on a runner, in a run of the gate's file, on `pull_request`, on the reviewed branch. The API lists each open pull request of a run, and does not say which one started it. So the `sign` job cannot tell the two apart. The approval and the acceptance trailer still hold. [A pull request from the reviewed branch into another base makes a run that counts as a gate](https://github.com/kalonji-tools/purba/issues/899) holds it.
- **A pull request runs its own copy of each gate.** On `pull_request`, GitHub takes each workflow file from the pull request. A pull request that changes `ci.yml` changes its own `CI`, and the `sign` job reads that job like any other. `.github/CODEOWNERS` covers `/.github/`, so the code owner reads such a change.
- **Settings hold the rule, and a reader of the tree sees none of them.** An account with admin access can widen the environment or move the pin, and nothing in the tree contradicts it.
- **A person keeps the key.** An owner of the organization can uninstall the App or replace its key. Either one fails the mint, which fails the run, so no `Sign-off` comes.
- **Each `sign` run makes a deployment of the environment `sign`.** The deployments of the repository list one for each run.

## Confirmation

These commands read back the settings that hold the rule.

| command | prints |
|---|---|
| `gh api repos/kalonji-tools/purba/rulesets/22774939 --jq '.rules[].parameters.required_status_checks // empty'` | `Sign-off` alone, with `integration_id` 5688527 |
| `gh api apps/purba-sign --jq .id`, as an owner of `kalonji-tools` | 5688527 |
| `gh api repos/kalonji-tools/purba/environments/sign/deployment-branch-policies --jq '[.branch_policies[].name]'` | `["main"]` |
| `gh api repos/kalonji-tools/purba/environments/sign --jq .can_admins_bypass` | `false` |

`scripts/test/check-run.bats` holds the cases of a gate that counts and of one that does not.
No gate on a pull request reads `protect-main` or the environment, because neither is in the tree.

[The measurement of the environment](https://github.com/kalonji-tools/purba/issues/887#issuecomment-6753175210) shows a job on a branch and a job on a pull request refused.
[A commit status from any account with write access meets a required check of main](https://github.com/kalonji-tools/purba/issues/814) holds the measurements from a sandbox ruleset.
[A workflow on a branch with no pull request reports a required check of main](https://github.com/kalonji-tools/purba/issues/887) records the move of the pin of `Sign-off` to `purba-sign`.
