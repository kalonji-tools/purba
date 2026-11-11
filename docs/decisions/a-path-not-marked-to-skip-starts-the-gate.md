# A path not marked to skip starts the gate

## Context and Problem Statement

Build ran its jobs on every pull request, and many of those pull requests changed nothing Build reads.

| Build, measured when purba took this decision | value |
|---|---:|
| job-seconds in a run, the mean of 25 runs | 656 |
| jobs in a run | 15 |
| runs on a pull request that changed only docs, of the last 100 | 43 |

The `sign` job signs no head until `CI` is green, so a `CI` that never reports blocks the merge through `Sign-off`.
So a `paths:` filter cannot narrow `ci.yml`, and the workflow must start on every pull request.

## Considered Options

- **A path with no mark skips the gate.** A pull request that adds a path and its mark reads the base, where the path has no mark yet.

## Decision Outcome

**Reach:** `/.config/gates` `.github/workflows/ci.yml` `.github/scripts/changed.sh` `scripts/test/changed.bats` `.github/scripts/require-jobs.sh` `scripts/test/require-jobs.bats` `/.github/CODEOWNERS` `/CONTRIBUTING.md`

**Area:** tooling

**`ci.yml` skips the work of an attribute only when `.config/gates` marks every path the pull request changes to skip it.**
`CI` itself never skips.

| part | what it does |
|---|---|
| `.config/gates` | marks a path `<attribute>` or `-<attribute>` for each attribute, in the gitattributes format. `build` names the jobs `rust` and `wheel`, and `scripts` names the tests in `Scripts` and `Scripts-macOS` |
| `changed.sh <attribute> <base> <head>` | runs first in `Scripts`, in `Scripts-macOS` and in the job `changed`, and prints `<attribute>=true` unless every changed path is marked `-<attribute>` |
| the job `CI` | reports on every pull request, and refuses a skip that a failed `changed.sh` caused |

**A mark says whether a change to the path must start the work its attribute names.**
`.gitattributes` starts `rust` and `wheel`, though no step reads it, because git applies it at checkout.

[A gate reads its marks on the base branch](a-gate-reads-its-marks-on-the-base-branch.md) says which copy of the marks decides.
[One file marks every tracked path for each gate that can skip](one-file-marks-every-tracked-path-for-each-gate-that-can-skip.md) says what makes each tracked path carry a mark.
[purba writes its scripts in bash until purba can test them](purba-writes-its-scripts-in-bash-until-purba-can-test-them.md) says what a skipped run of the script tests costs.

**Downside:**

- **A wrong mark skips a job.** A path that `rust` or `wheel` reads, and that `.config/gates` marks `-build`, skips both. Only the code owner reads each mark.
- **A mark to skip stays when a job starts to read the path.** A Rust file can start to read a file under `docs/`, and the next change to the `docs/` file skips `rust` and `wheel`.

## Confirmation

`scripts/test/changed.bats` tests `changed.sh`, and `mise run test:shell` runs those tests.
`scripts/test/require-jobs.bats` tests that `require-jobs.sh` refuses a skip after the job `changed` failed.
The job `CI` runs that script.
`.github/CODEOWNERS` names `/.config/`, so a change to a mark waits for the code owner.

No command checks that a mark is right.
The Downside above carries that cost.
