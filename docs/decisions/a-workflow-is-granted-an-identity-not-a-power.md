# A workflow is granted an identity, not a power

## Context and Problem Statement

`.github/workflows/bump.yml` proposes a newer compiler every week for a person to sign.
Its first scheduled run pushed the branch and then stopped.

```
pull request create failed: GraphQL: GitHub Actions is not permitted to
create or approve pull requests (createPullRequest)
```

The repository setting that would permit the call is one switch over two powers.
It permits Actions to create a pull request, and it permits Actions to approve one.

[Acceptance](../../CONTEXT.md#acceptance) here is derived from the code owner's approval, so an approval is what makes a [liability](../../CONTEXT.md#liability) true.
A workflow that can approve makes every approval say less than it says, including the ones it does not write.
[An outside contribution is applied, not merged](an-outside-contribution-is-applied-not-merged.md) already refused a mechanism on that reasoning.

So the question is not whether the bump may open a pull request.
It is which identity opens it.

## Considered Options

- **Permit Actions to create and approve pull requests.** Rejected, and measured before it was. The switch buys the approve power, and it does not even finish the job. A pull request created with `GITHUB_TOKEN` puts its workflow runs in an approval-required state. Every weekly bump would then wait for a person to press a button before `Quality` and `Build` ran.
- **Open no pull request, and leave a person a link to open one.** Rejected on the ruleset. The person who opens it becomes its author. `protect-main` requires one approving review and a code owner's review, and GitHub refuses an approval from the author. purba has one maintainer, so every bump would merge through an override. The gate would never bind the one change nobody wrote.
- **Open it with an account's token.** Rejected. An account token is named by what it is rather than by what it does. It carries whatever that account may do, wherever that account may do it. It also expires on a date nobody records. A bump that stops because a token lapsed is the same silence this workflow exists to break.
- **Mint an installation token for the act, and grant Actions nothing further.** Chosen. The token is bounded by an installation rather than by a person, it lives one hour, it is minted per run, and nobody rotates it.

## Decision Outcome

An act `GITHUB_TOKEN` may not perform earns an identity scoped to that act.
Actions' own token is never widened to reach it.

The identity is a GitHub App. It holds no code, no server and no webhook.

| the App | |
|---|---|
| named | `purba-bump`, and the author it writes reads `purba-bump[bot]` |
| webhook | none, because nothing calls it |
| installed on | this repository, and no other |
| may | read and write pull requests |
| may also | read contents and metadata |
| may not | write contents, touch an issue, or approve anything |
| lives | one hour, minted by `actions/create-github-app-token` in the job that uses it |

**The narrowing is the boundary, and no rule sits over the secret.**
A secret cannot be bound to one workflow file, so a rule saying which workflow may name it would be a convention that nothing enforces.
A workflow that named these two could open a pull request and do nothing else, which is the bound worth having.

**The token that reports a failure is not this one.**
`.github/scripts/report-bump-failure.sh` opens or comments on an issue, which this identity may not do, so that step keeps `GITHUB_TOKEN`.
A change that points it at the App token would stop the only thing that reports a scheduled run.

**Downside:**

- **An App can be uninstalled, and its key can be replaced.** Either fails the mint, which fails the run, which reports. That is the loud behaviour, and purba chose it rather than a quiet stand down. A bump that stands down quietly freezes the compiler and says nothing.
- ⚠️ **A repository setting is invisible to a reader of this tree.** The switch this record refuses is read back through the API and appears in no file here. A later maintainer can turn it on, and nothing in the tree will contradict them.
- **This adds a third thing the workflow trusts at run time**, beside the checkout and the tool installer. It is published by GitHub's own organization and pinned the way purba pins the other two.

## Confirmation

The prototype that measured this lives on a branch that is never merged, so the numbers live here.

Taken 2026-09-29, one call, one branch, one minute apart, differing only in the token that held it.

| measured | result |
|---|---|
| `gh pr create` under `GITHUB_TOKEN` | refused, and the refusal names the setting |
| `gh pr create` under the installation token | opened, and the author reads `purba-bump[bot]` with type `Bot` |
| workflow runs GitHub created on that pull request | six, of which `Quality` and `Build` are two, and zero waiting for a person |
| a mint given an app id that is not the App | the step fails, so a lost identity is loud rather than quiet |

⚠️ **The author reads two ways, and a check written against the wrong one refuses a pull request the App did open.**
The REST payload reports `purba-bump[bot]` and a type of `Bot`.
The GraphQL payload, which `gh pr view` reads, reports `app/purba-bump` instead.
A test asserts on the type.

Nothing checks which workflow names the secret, and nothing needs to: the token carries its own limit.

The weekly run is what reports on all of it.
A pull request authored by the App is the working state, and a red run comments where somebody reads it.
[Bump the pinned nightly on a schedule, and regenerate the lockfile with it](https://github.com/kalonji-tools/purba/issues/190) holds that reporting path.
