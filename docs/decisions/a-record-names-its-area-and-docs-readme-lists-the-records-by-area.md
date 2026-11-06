# A record names its area, and docs/README.md lists the records by area

## Context and Problem Statement

A person looks for a record by the kind of work it decides, and an agent looks with `grep`.
[A decision record is rewritten, not amended](a-record-is-rewritten-not-amended.md) keeps the records in one flat folder.
GitHub lists every file of a folder before it renders the folder's `README.md`.

| on `main` when this was decided | count |
|---|---:|
| records, in one folder | 95 |
| records in two areas | 12 |
| rows GitHub lists in the folder before its `README.md` | 97 |

## Considered Options

- **The flat folder, with no index.** A person cannot find a record by its area in it.
- **A folder for each area.** Apart from the search by area, it adds only a code owner for each area, and purba has one owner. It breaks each link to a record from outside the tree.
- **An index page alone, written by hand.** It drifts each time a pull request adds, renames or deletes a record.
- **An index inside `docs/decisions/`.** GitHub renders it below every record.
- **The word kind for the line.** [Kind](../../CONTEXT.md#kind) already names the kind of an error.
- **Four areas, with no area for repository tooling.** None of the four fits a record that decides how a tool of the repository runs.

## Decision Outcome

**Reach:** `docs/decisions/*.md` `docs/README.md` `/LANDSCAPE.md` `/.config/tasks.toml` `scripts/records.sh` `scripts/check-records.sh` `scripts/index-of-records.sh` `scripts/test/check-records.bats` `scripts/test/index-of-records.bats` `.github/CODEOWNERS`

**Area:** writing, tooling

**Each record names the [area](../../CONTEXT.md#area) it decides something in, and `docs/README.md` lists the records by area.**
No record moves.
A gate keeps the `**Area:**` line and the index true.

- **The line.** It is its own paragraph after the reach: `**Area:**`, then each area of the record, with a comma between two areas.
- **The areas.** They are a closed list, and the index names each one in full:

| the line writes | the index names it |
|---|---|
| `design` | Product design |
| the name of a part or a concern in [the landscape](../../LANDSCAPE.md) | One part or concern, then the name |
| `workflow` | The pull-request workflow |
| `writing` | How a record is written |
| `tooling` | Repository tooling |

- **A record in two areas names each area.** The index lists it under each, and no area comes first.
- **A task writes the index.** `mise run fmt:index` writes `docs/README.md` from the lines, and leaves out an area that no record names.
- **The index has the code owner of the records.** `.github/CODEOWNERS` names `/docs/README.md`, so a change to it asks for the same review as a change to a record.
- **A reviewer judges whether an area fits the record.**

**Downside:**

- **Each record carries one more line.** A new record needs it before `lint:records` passes.
- **Each title lives in two files.** A new record or a new title needs `mise run fmt:index` before `lint:index` passes.
- **After a rename in the landscape, `lint:records` refuses each record that names the old name.** The pull request that renames a part or a concern rewrites those records too.
- **A wrong area passes.** No command reads whether an area fits, so only the reviewer catches one.
- **The folder stays flat.** `ls` and the folder page on GitHub still list every record in one list.

## Confirmation

`mise run lint:records` refuses a record with no `**Area:**` paragraph after its reach, and an area outside the list.
`mise run lint:index` refuses a `docs/README.md` that differs from the page `fmt:index` writes.
`mise run lint` runs both, and `Quality` runs `mise run lint` on each pull request.
