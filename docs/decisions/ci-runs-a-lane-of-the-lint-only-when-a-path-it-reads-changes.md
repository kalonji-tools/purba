# CI runs a lane of the lint only when a path it reads changes

## Context and Problem Statement

`Quality` ran every lint on every pull request.
`Quality-macOS`, `Scripts` and `Scripts-macOS` each took a runner on every pull request.
The two `Scripts` jobs skipped only their tests.
So a pull request that changed only `docs/` waited for the lint on macOS.

A repository of the prototype ran each option.

| median, measured when purba took this decision | every lint, every job | lanes `src`, `scripts`, `ci` and the rest | lanes `rust`, `scripts`, `docs` and `always` |
|---|--:|--:|--:|
| green pushes for each change | 5 | 5 | 5 |
| time from a push to `docs/agents/` to the end of `CI`, s | 124 | 79 | 63 |
| time from a push to the template of a record to the end of `CI`, s | 145 | 86 | 73 |
| time from a push to `.github/` to the end of `CI`, s | 129 | 76 | 65 |
| macOS jobs of a push to `docs/agents/` | 2 | 0 | 0 |
| macOS job-seconds of a push to `docs/agents/` | 139 | 0 | 0 |

## Considered Options

- **Every lint and every job on every pull request.** A pull request that changed no code held macOS runners, and waited for them.
- **Lanes `src`, `scripts`, `ci` and the rest.** `lint:records`, the slowest lint, ran on every pull request. So a pull request that changed no code waited longer than with lanes `rust`, `scripts`, `docs` and `always`.

## Decision Outcome

**Reach:** `/.config/gates` `/.config/lanes` `/.config/tasks.toml` `/prek.toml` `.github/workflows/ci.yml` `.github/scripts/lanes.sh` `scripts/test/lanes.bats` `/CONTRIBUTING.md`

**Area:** tooling

**`ci.yml` runs a lane of the lint, `Quality-macOS`, `Scripts` and `Scripts-macOS` only when a pull request changes a path that `.config/gates` marks for it.**
The job `changed` first decides whether each one runs.
Of the options that let no broken lint through, these lanes gave the verdict of `CI` soonest on a pull request that changed no code.

| work | starts on a path marked | `CI` requires |
|---|---|---|
| a lane of the lint, in `Quality` | `lint-<lane>` | that `Quality` succeeded |
| `Quality-macOS` | `lint-macos` | what `changed` asked |
| `Scripts` and `Scripts-macOS` | `scripts` | what `changed` asked |
| `rust` and `wheel` | `build` | what `changed` asked |

`.config/lanes` names the lanes of each lint.
`.github/scripts/lanes.sh` turns the lanes that start into one `mise run`.

- **The lane `always` runs on every pull request.** It holds each lint that reads every path, and the two lints of `.github/`, which take a few seconds.
- **A lint that `.config/lanes` does not name runs on every pull request.** Until it has a lane, a new lint costs time but never skips.
- **For a lane, `.config/gates` marks what its lints read, their own code and the files of mise.**
- **`lint-macos` marks the code of a lint, the files of mise and what clippy compiles.** The job `rust` runs only the tests, so clippy reads code under `cfg(target_os = "macos")` only in `Quality-macOS`. A change to `docs/` cannot break a script. It can only show a defect that the script already has.
- **`prek` runs on every pull request, and skips the hooks `shellcheck` and `editorconfig-checker`.** Each one runs a lint that the lane step of `Quality` runs when its lane starts.
- **A notice of `Quality` names each lane that started, and its log names each lint that skipped.**

[A path not marked to skip starts the gate](a-path-not-marked-to-skip-starts-the-gate.md) says what a path with no mark starts.
[A gate reads its marks on the base branch](a-gate-reads-its-marks-on-the-base-branch.md) says which copy of the marks decides.
[One workflow runs every gate of a pull request](one-workflow-runs-every-gate-of-a-pull-request.md) says which jobs `CI` waits for.

**Downside:**

- **A wrong mark skips a lint.** `lint:gates` refuses a path with no mark, and no command checks that a mark is right.
- **A path that no lane reads starts only the lane `always`.** `docs/agents/` is one, so a change there runs no lint of a lane.
- **The head decides which lints run.** `changed` and `Quality` run `lanes.sh` from the head, and `lanes.sh` reads `.config/lanes` there. A change to either starts every lane only because the base marks both for every lane. No command checks those two marks.
- **A defect that a script already has shows later on macOS.** A pull request that changes only `docs/` runs no lint on macOS.
- **Each job that can skip, and `Quality`, waits for `changed`.** It takes a runner and a few seconds before any of them starts.

## Confirmation

`scripts/test/lanes.bats` tests `lanes.sh`, and `mise run test:shell` runs those tests.
`mise run lint:gates` refuses a path that does not carry each mark.

The prototype broke one lint in each change below, and `CI` refused each one in that lint.

| change | the lint that refused it |
|---|---|
| an HTML comment in a record | `lint:records` |
| a shell defect in `scripts/` | `lint:shell` |
| a shell defect in `tests/` | `lint:shell` |
| a licence added to `.cargo/deny.toml` | `lint:licences` |
| a path with no mark | `lint:gates` |

[Which jobs of ci.yml may skip on a push whose paths cannot change their verdict?](https://github.com/kalonji-tools/purba/issues/934#issuecomment-6816451402) records where the prototype measured this.
