# A path not marked to skip starts the gate

## Context and Problem Statement

Build ran its jobs on every pull request, and many of those pull requests changed nothing Build reads.

| Build, measured when purba took this decision | value |
|---|---:|
| job-seconds in a run, the mean of 25 runs | 656 |
| jobs in a run | 15 |
| runs on a pull request that changed only docs, of the last 100 | 43 |

The ruleset requires `Build`, and a required check that never reports blocks the merge.
So a `paths:` filter cannot narrow Build, and the workflow must start on every pull request.

## Considered Options

- **A path with no mark skips the gate.** Rejected. A pull request that adds a path and its mark reads the base, where the path has no mark yet.
- **A path with no mark starts the gate.** Chosen.

## Decision Outcome

**Reach:** `/.config/gates` `.github/workflows/build.yml` `.github/workflows/scripts.yml` `.github/scripts/changed.sh` `scripts/test/changed.bats` `/.github/CODEOWNERS` `/CONTRIBUTING.md`

**A gate skips a pull request only when `.config/gates` marks every path the pull request changes to skip that gate.**

| part | what it does |
|---|---|
| `.config/gates` | marks a path `<gate>` or `-<gate>` for each gate that can skip, in the gitattributes format |
| `changed.sh <gate> <base> <head>` | runs first in the gate, and prints `<gate>=true` unless every changed path is marked `-<gate>` |
| the check the gate reports | reports on every pull request, and refuses a skip that a failed `changed.sh` caused |

**A mark says whether a change to the path must start the gate.**
`.gitattributes` starts Build, though no step reads it, because git applies it at checkout.

[A gate reads its marks on the base branch](a-gate-reads-its-marks-on-the-base-branch.md) says which copy of the marks decides.
[One file marks every tracked path for each gate that can skip](one-file-marks-every-tracked-path-for-each-gate-that-can-skip.md) says what makes each tracked path carry a mark.
[purba writes its scripts in bash until purba can test them](purba-writes-its-scripts-in-bash-until-purba-can-test-them.md) says what a skipped run of the script tests costs.

**Downside:**

- **A wrong mark skips the gate.** A path that Build reads and that `.config/gates` marks `-build` skips Build. Only the code owner reads each mark.
- **A mark to skip stays when the gate starts to read the path.** A Rust file can start to read a file under `docs/`, and the next change to the `docs/` file skips Build.

## Confirmation

`scripts/test/changed.bats` tests `changed.sh`, and `mise run test:shell` runs those tests.
The `Build` job refuses a skipped matrix when the job `changed` failed.
`.github/CODEOWNERS` names `/.config/gates`, so a change to a mark waits for the code owner.

No command checks that a mark is right.
The Downside above carries that cost.

[.config/gates marks itself build, but a change to it need not start Build](https://github.com/kalonji-tools/purba/issues/510) asks that `.config/gates` mark itself `-build`.
