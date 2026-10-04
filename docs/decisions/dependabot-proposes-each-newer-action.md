# Dependabot proposes each newer action

## Context and Problem Statement

Each workflow names an action by its major version, such as `actions/checkout@v4`.
Nothing in the tree proposes a newer one.
A person learns of an old action when GitHub warns about it.
By then GitHub already deprecated the runtime under it.

A wheel job, which uses both actions below, carried this warning:

> Node.js 20 is deprecated. The following actions target Node.js 20 but are being forced to run on Node.js 24: actions/checkout@v4, actions/setup-python@v5.

Dependabot has two halves.

| Dependabot | opens a pull request |
|---|---|
| security updates | when an advisory names a dependency |
| version updates | on the schedule `.github/dependabot.yml` sets |

A runtime deprecation is not an advisory.
Security updates never raise one.

## Considered Options

- **Bump each action that warns, by hand.** Rejected. It moves the actions that warned. The next deprecation arrives the same way.
- **Dependabot version updates.** Chosen. GitHub runs it under its own identity. purba writes one file.

## Decision Outcome

**Dependabot proposes each newer action the workflows use.**
It reads `.github/dependabot.yml` once a week.
Version updates move every action in one pull request.
A security update arrives on its own, whenever an advisory names an action.

**A person signs it, like any other branch.**
[Liability is recorded from the act that makes it true](liability-is-recorded-from-the-act-that-makes-it-true.md) says how.

**Dependabot never proposes the nightly.**
It cannot regenerate `mise.lock`.
`.github/workflows/bump.yml` proposes the nightly instead.

**Downside:**

- **Dependabot cannot write a subject in purba's form.** It writes a prefix and never a suffix. A security update also inserts `[security]` after the prefix. So each of its pull requests needs a reduction. The reduction rewrites each subject to end in the number of [Keep the workflow actions current with Dependabot](https://github.com/kalonji-tools/purba/issues/282).
- **Dependabot no longer rebases a branch once someone else pushes a commit to it.** The reduction is such a push.
- **A workflow that no pull request runs meets a newer action only on its next run.** `bump.yml` runs on a schedule. `release.yml` runs by hand.

| update | as Dependabot writes it | once reduced, where `N` is that issue's number |
|---|---|---|
| version | `ci(deps): bump the actions group with 2 updates` | `ci(deps): bump the actions group with 2 updates (#N)` |
| security | `ci(deps): [security] bump actions/checkout from 4 to 7` | `ci(deps): bump actions/checkout from 4 to 7 (#N)` |

## Confirmation

| property | check |
|---|---|
| Dependabot proposes each newer action | a pull request from `dependabot[bot]`. Nothing refuses a week without one |
| `.github/dependabot.yml` is a configuration Dependabot accepts | no command here checks it against Dependabot's schema. GitHub reads it on `main`. The Dependabot tab of the dependency graph lists each run |
| a reduced subject ends in an issue | `subject-form` in `prek.toml` refuses a subject without one at `git commit`. A rebase reword skips the hook |
