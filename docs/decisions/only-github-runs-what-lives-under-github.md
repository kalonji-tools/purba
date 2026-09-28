# Only GitHub runs what lives under `.github`

## Context and Problem Statement

Eight scripts accumulated under `.github/scripts/`, and four of them were reached from somewhere that is not a workflow.
A person's sign-off command ran the replay check.
The command that carries an outside contribution in ran the origin check, and that command was itself run by nothing in this repository at all: no workflow, no task, no hook.

So the boundary was crossed in both directions, and nothing in the tree said which side a script belonged on.
[A location inherits its readers](a-location-inherits-its-readers.md) binds `.github/**` and `scripts/**` to the same actor, so it answers who reads a script and not where one goes.

The stakes are not tidiness.
`CODEOWNERS` covered `/.github/` and did not cover `scripts/`, so placement decided who reviews the code that gates every merge.

## Considered Options

- **One directory for every script.** Rejected. A workflow's own shell has no reader outside the workflow, and `.github/` is where the thing that runs it already lives.
- **Keyed on what a script does.** Rejected as undecidable. Whether a task belongs to continuous integration is a judgement, and this project has refused undecidable conditions repeatedly.
- **Keyed on what calls it.** Chosen. A caller is a fact a command can read.

## Decision Outcome

**Only GitHub runs what lives under `.github`.**
A script there is named by a workflow and by nothing else.

A script a person runs, or that this project's own tooling runs, lives in `scripts/`.
A sourced helper lives where its callers do, because it has no caller of its own to key on.

**`CODEOWNERS` covers both directories.**
Placement decides which directory a script sits in and never who approves it, and the gate's own code is the last thing that should lose a reviewer to a move.

**Downside:** a script moves when its callers change, and a caller is added by an edit somewhere else.
The rule is therefore not stable against a change it cannot see, and the only thing that makes such a move visible is the check below.
A sourced helper is the weakest part: it follows its callers, so two callers on opposite sides would leave it with no correct home.

## Confirmation

The rule was applied by reading every caller in the tree.

| script | named by | lives in |
|---|---|---|
| `check-origin.sh` | two workflows **and** the fork command | `scripts/` |
| `check-replayable.sh` | two workflows **and** the sign-off command | `scripts/` |
| `report.sh` | sourced by both of those | `scripts/` |
| `apply-fork-contribution.sh` | nothing in this repository | `scripts/` |
| `bump-nightly.sh`, `report-bump-failure.sh` | one workflow | `.github/scripts/` |
| `carry-verdicts.sh`, `require-green.sh`, `check-run.sh`, `write-acceptance-trailer.sh`, `post-record-thread.sh` | one workflow, or sourced by one | `.github/scripts/` |

⚠️ **No command reads this rule yet, and until one does it is a review comment.**
[Refuse a script that sits in the wrong directory](https://github.com/kalonji-tools/purba/issues/203) wires it, and both directions are decidable: a script under `.github/scripts/` that no workflow names, and a script under `scripts/` that only a workflow names.
