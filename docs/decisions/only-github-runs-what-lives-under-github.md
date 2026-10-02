# Only GitHub runs what lives under `.github`

## Context and Problem Statement

Eight scripts accumulated under `.github/scripts/`, and four of them were reached from somewhere that is not a workflow.
A person's sign-off command ran the replay check.
The command that carries an outside contribution in ran the origin check.
Nothing in this repository ran that command at all: no workflow, no task, no hook.

So the boundary was crossed in both directions, and nothing in the tree said which side a script belonged on.
[A location inherits its readers](a-location-inherits-its-readers.md) binds `.github/**` and `scripts/**` to the same actor, so it answers who reads a script and not where one goes.

The stakes are not tidiness.
`CODEOWNERS` covered `/.github/` and did not cover `scripts/`, so placement decided who reviews the code that gates every merge.

## Considered Options

- **One directory for every script.** Rejected. A workflow's own shell has no reader outside the workflow, and `.github/` is where the thing that runs it already lives.
- **Keyed on what a script does.** Rejected as undecidable. Whether a task belongs to continuous integration is a judgement, and this project refused undecidable conditions repeatedly.
- **Keyed on what calls it.** Chosen. A caller is a fact a command can read.

## Decision Outcome

**Only GitHub runs what lives under `.github`.**
A script is refused when every caller of it lives in the other half of the tree.

`.github/**` is GitHub's half.
Everything else is a person's, and `tasks.toml` and `prek.toml` are in it because a person reaches a task through them.
A script a person runs, or that this project's own tooling runs, lives in `scripts/`.

A sourced helper needs no rule of its own.
The script that sources it is a caller, so the same clause puts the helper where its callers are.

A script that no caller in the tree names is refused under `.github` and accepted in `scripts/`.
The runners under `.github` are enumerable, so nothing there reaches a script that no workflow and no sibling names.
A person is a caller this repository cannot see, which is why the fork command belonged in `scripts/` while nothing here ran it at all.

**`CODEOWNERS` covers both directories.**
Placement decides which directory a script sits in, and never who approves it.
The gate's own code is the last thing that should lose a reviewer to a move.

**Downside:**

- **A script moves when its callers change, and a caller is added by an edit somewhere else.**
  The rule is therefore not stable against a change it cannot see. The command below is what makes such a move visible, on a branch brought current and not before.
- **A sourced helper is the weakest part.**
  It follows its callers, so two callers on opposite sides would leave it with no correct home.
  The command stays silent there and does not choose one, because the rule names no home to choose.
- **A caller is read as a literal path and never as a call.**
  A line that merely writes one counts as a caller.

## Confirmation

**`mise run lint:placement` reads the rule, and `quality` depends on it.**
`scripts/check-placement.sh` keys on the callers a code line names.
A comment line naming a script it no longer runs counts for nothing, and neither does a record naming one in prose.

⚠️ **The table this section carried is deleted rather than corrected.**
It was a second reading of what the command now derives on every run, and one of its rows went wrong.
`require-green.sh` was filed under *"one workflow, or sourced by one"*.
No workflow names it and nothing sources it, because a sibling under `.github/scripts/` executes it.
That row is the case the clause above exists to admit, and it is also what a hand-written copy of a machine-readable fact becomes.

The command was exercised green and red.

| control | reading |
|---|---|
| the tree untouched | fifteen scripts pass, and the exit is 0 |
| the record command moved under `.github/scripts/` | refused, naming `tasks.toml` as its only caller |
| the record-thread script moved into `scripts/` | refused, naming `records.yml` |
| a new script under `.github/scripts/` that nothing names | refused |
| a new command in `scripts/` that nothing names | passes, because a person is its caller |
| the bump workflow keeps its header comment and loses the line that runs the script | refused, and the comment survives the edit |
| a duplicate basename across the two halves | the new file is refused and the placed one passes, so a collision cannot produce a silent pass |
| run outside a git repository | exits 2 rather than 1, because it cannot decide rather than refuse |
| run with `GITHUB_ACTIONS=true` | the refusal is an `::error::` annotation |

⚠️ **Nothing runs the command itself.**
The controls above were taken by hand, and [Test every gate script against the shape it refuses](https://github.com/kalonji-tools/purba/issues/201) owns the harness that would re-run them.
