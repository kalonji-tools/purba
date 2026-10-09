# PROTOTYPE #397: results (throwaway, never merges)

Question: does a reach that names only the locations whose content can break
its record still name the records the audited pull requests broke, and name
fewer records?

## What was built

- `data/narrow-reaches.tsv`: the 34 reaches of #388, narrowed under the rule.
  A subagent wrote them from the record texts, `CONTEXT.md`, a tree listing and
  the wide reaches with their notes. It never saw the breach table or #388's
  results. It could only remove entries, so the rule is the one variable. Its
  notes are in `data/narrow-notes.md`.
- `run.sh`: fetches the ten pull request heads, replays both reach sets
  through `proto/388/replay.sh`, and splits each run with `split.sh`.
- `split.sh`: counts the two lists the command prints. The every-PR list holds
  the records whose reach is `**` or carries `commit message`, `pull request`
  or `issue`.

The wide run reproduces every file in `proto/388/runs/out-w3` and
`proto/388/runs/nr-w3` with `LINKS=headers`.

## What the rule removed

3 of 34 reaches changed, and 5 entries went. No reach became empty.

| record | removed | why the subagent removed it |
|---|---|---|
| `an-implementation-follows-test-driven-development` | `scripts/**`, `.github/scripts/**`, `*.rs` | the paths say when test-first is owed, and the breach shows only in the pull request's heads |
| `significance-is-declared-not-detected` | `docs/decisions/*.md` | the rule breaks in `CODEOWNERS` and the ruleset |
| `the-person-owns-the-decision-outcome` | `docs/decisions/*.md` | the same |

## Measure 1: records named

Records the change names, then the every-PR list (11 records, of which a base
holds 6 to 11).

| pull request | wide: this change | narrow: this change | every-PR list |
|---|---:|---:|---:|
| 227 | 5 | 4 | 7 |
| 234 | 12 | 11 | 7 |
| 241 | 5 | 4 | 8 |
| 247 | 12 | 11 | 8 |
| 271 | 15 | 14 | 11 |
| 212 (no record) | 3 | 3 | 6 |
| 230 (no record) | 5 | 5 | 7 |
| 254 (no record) | 7 | 7 | 8 |
| 260 (no record) | 7 | 7 | 9 |
| 268 (no record) | 3 | 3 | 9 |

The one record that left each audited list is
`significance-is-declared-not-detected`. The test-first record was already on
the every-PR list through `Pull request`, so its three paths named nothing the
word did not.

## Measure 2: breaches named

| links in files | wide | narrow |
|---|---:|---:|
| header links (as #388) | 19 of 21 | 19 of 21 |
| none | 18 of 21 | 17 of 21 |

The narrow reach loses one breach: on 271, "A Downside edit sat inside the
Decision Outcome, and the body did not list it as drafted", which broke
`the-person-owns-the-decision-outcome`. With header links it is still named,
but only because 271 changed `AGENTS.md`, which links that record. The breach
is in record text: an edit inside a Decision Outcome. So the issue's premise,
"no record text can break either of them", is false for that record.

The subagent flagged this one removal as its weakest, blind: "If replay misses
a pull request that rewrote a record's Decision Outcome rule, this removal is
the cause."

## Verdict against the falsifiers

| falsifier | result |
|---|---|
| the narrow reaches name fewer than 19 of 21 | does not fire with header links (19), but only by chance; 17 against 18 on the reaches alone |
| the narrow reaches name about as many records as the wide ones | **fires**: one fewer on each audited pull request, none fewer on the others |

## Where the width is

The every-PR list is 6 to 11 records on every pull request, and the narrow
rule does not touch it. On a record change, `docs/decisions/*.md` still names
the records whose rules bind record text, and those are real.

## Missed by the wide reach

The subagent listed 8 entries that the wide reach lacks and the record text
supports. It did not add them. Examples: `scripts/test/check-links.bats` (the
record names it), and `.github/workflows/*.yml` for the trait-solver and
register records. Any reach written at build can miss what these missed.
