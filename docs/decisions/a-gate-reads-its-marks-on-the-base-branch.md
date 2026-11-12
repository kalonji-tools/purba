# A gate reads its marks on the base branch

## Context and Problem Statement

[A path not marked to skip starts the gate](a-path-not-marked-to-skip-starts-the-gate.md).
`.config/gates` holds the marks.
A pull request can change `.config/gates` as it changes any other file.
So the base branch and the head can hold different marks, and the gate reads one copy.

## Considered Options

- **The head of the pull request.** A pull request could mark the paths it changes to skip, and skip its own gate.

## Decision Outcome

**Reach:** `/.config/gates` `/.config/lanes` `.github/workflows/ci.yml` `.github/scripts/changed.sh` `.github/scripts/lanes.sh` `.github/scripts/require-jobs.sh` `scripts/attributes.sh` `scripts/test/changed.bats` `/.github/CODEOWNERS` `/CONTRIBUTING.md`

**Area:** tooling

**`changed.sh` reads `.config/gates` on the base branch of the pull request.**
A pull request cannot mark the paths it changes to skip its own gate.
A pull request to another branch reads the marks of that branch.
`.config/gates` marks itself `-<attribute>` for each attribute, because no gate that can skip reads the copy a pull request changes.
[CI runs a lane of the lint only when a path it reads changes](ci-runs-a-lane-of-the-lint-only-when-a-path-it-reads-changes.md) says what each attribute starts.

**Downside:**

- **A changed mark does not apply to the pull request that changes it.** The new mark applies from the next pull request.
- **A pull request can change what decides its own skip.** The workflow, `changed.sh`, the scripts it sources, `lanes.sh` and `require-jobs.sh` run from the head. `lanes.sh` reads `.config/lanes` there. `require-jobs.sh` decides whether the skip passes. The code owner of `/.github/`, `/.config/` and `/scripts/` reads a change to any of those.

## Confirmation

`scripts/test/changed.bats` tests that a head which marks its own change to skip still starts the gate, and `mise run test:shell` runs those tests.
`.github/CODEOWNERS` names `/.github/`, `/.config/` and `/scripts/`.
So a change to the workflow, to a script it runs or one of those sources, or to `.config/lanes` waits for the code owner.
