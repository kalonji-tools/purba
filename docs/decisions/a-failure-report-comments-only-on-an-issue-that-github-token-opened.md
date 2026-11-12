# A failure report comments only on an issue that `GITHUB_TOKEN` opened

## Context and Problem Statement

`.github/scripts/report-run-failure.sh` reports a failed run of `bump.yml`, `release.yml` or `publish.yml` in an issue.
It comments on an open issue that carries the title it writes for that workflow.
When no such issue is open, it opens one.
The repository is public, so anyone can open an issue with that title.

## Considered Options

- **The first open issue with the title gets the comment, whoever opened it.** A stranger's issue then holds the report. Its author can close that issue.

## Decision Outcome

**Reach:** `.github/scripts/report-run-failure.sh` `scripts/test/report-run-failure.bats` `.github/workflows/bump.yml` `.github/workflows/release.yml` `.github/workflows/publish.yml`

**Area:** tooling

**`report-run-failure.sh` comments only on an open issue that `GITHUB_TOKEN` opened.**
Each step that runs the script holds `GITHUB_TOKEN`, so the script opens each of its issues under that token.
[A workflow is granted an identity, not a power](a-workflow-is-granted-an-identity-not-a-power.md) says why each of those steps keeps `GITHUB_TOKEN`.

**Downside:**

- **A step that runs the script under another token never finds the issues that token opened.** The script then opens a new issue at each failure, unless an issue from `GITHUB_TOKEN` is open. The reader of the open issues then finds one issue for each failure.

## Confirmation

For each of the three workflows, `scripts/test/report-run-failure.bats` gives the script an open issue with the title of that workflow and another author.
It expects the script to open an issue of its own.
`mise run test:shell` runs it, and `Scripts` runs that task on each pull request that changes a path not marked `-scripts`.

[The failure report of a workflow goes to any open issue that carries its title](https://github.com/kalonji-tools/purba/issues/813) holds the reproduction on `78232b7`.
