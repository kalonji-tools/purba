# PROTOTYPE #388: results (throwaway, never merges)

Question: does a command that reads each record's reach name the records the
five audited pull requests broke, and name few others?

## What was built

- `data/reaches.tsv`: a reach for each of the 34 records at the five bases. A
  subagent wrote them from the record text, `CONTEXT.md` and a tree listing
  only. It never saw the breach table. Its notes are in `data/reach-notes.md`.
- `name-records.sh <base> <head> [records-dir]`: reads each `**Reach:**` line,
  matches its patterns with `git check-attr` in an empty repository, and names
  each record the pull request changes. Words in `$ALWAYS` match every pull
  request. `$LINKS` is `all`, `headers` (links from files outside
  `docs/decisions/`) or `none`.
- `replay.sh`: puts each reach on the records at each base, runs the command,
  and scores it against `data/breaches.tsv`, the 21 breaches from #374 that a
  record states.

## Measure 1 and 2: records named, breaches named

Five audited pull requests, all of which change a record:

| variant | 227 | 234 | 241 | 247 | 271 | breaches named |
|---|---|---|---|---|---|---:|
| records at base (a record the pull request adds is named, not counted here) | 20 | 22 | 23 | 23 | 33 | |
| control: every link (#374) | 8 | 11 | 11 | 9 | 27 | 12 of 21 |
| control: header links | 3 | 9 | 3 | 8 | 21 | 10 of 21 |
| reach paths + header links | 8 | 19 | 9 | 16 | 25 | 19 of 21 |
| + `commit message`, `pull request`, `issue` | 12 | 19 | 13 | 20 | 26 | 19 of 21 |
| + every word a pull request writes | 15 | 20 | 15 | 22 | 26 | 21 of 21 |

Five pull requests that change no record (no breach table, width only):

| variant | 212 | 230 | 254 | 260 | 268 |
|---|---|---|---|---|---|
| records at base | 19 | 20 | 23 | 24 | 25 |
| reach paths + header links | 5 | 7 | 11 | 11 | 6 |
| + the three words | 9 | 12 | 15 | 16 | 12 |

- 10 of the 34 records carry `commit message`, `pull request` or `issue`, and
  every pull request writes all three. So those 10 are named on every pull
  request. Two more carry `**`.
- The three words named no breach that the paths missed. Without them, 3
  commit-message breaches were named only by chance: the pull request changed
  the record, or touched `bump-nightly.sh`, whose header links it.
- The 2 misses are both `an unpublished comment carries what no other
  location carries`. Its reach word, `Unpublished comment`, names a comment in
  any file, so the word matches every pull request too.
- A change to any record names 5 or 6 records more: their reach is
  `docs/decisions/*.md`. Two of them (`significance`, `the person owns`) reach
  records because a record change routes to the code owner, not because the
  record text can break them.

## Measure 3: the agent's report on one pull request

`declare each package in one manifest` (#247), judged at `083508a` (the head
the pre-ready audit read) and at `85dfe79`. 20 records named each time.

| breach a review found | at the head | full records, `083508a` | outcomes only, `083508a` | full records, `85dfe79` |
|---|---|---|---|---|
| Considered Options told how the work arrived | `083508a` | missed | found | (fixed) |
| two commit bodies said the same thing | both | found | found | found |
| a comment restated the record | both | not named | not named | not named |
| the Decision Outcome opened in the passive | both | missed | missed | missed |
| two ideas joined with `, and` | both | missed | missed | missed |
| a Considered Option varied the chosen rule (no record) | both | found | found | found |
| a sentence went false (no record) | `083508a` | found | missed | (fixed) |

The agent also found one defect on `main` that no review found: filed as #390.
Today's `check-prose` tallies the passive voice and checks no `, and` join. So
no gate owns the two prose rules the agent missed.

## Verdict against the falsifiers

| falsifier | result |
|---|---|
| it names most of the set | **fires**: 56 to 86% on the audited pull requests, 47 to 66% on the others, with the three words |
| it names fewer than the 12 the links found | does not fire: 19 of 21 |
| the agent misses a breach a review found | **fires**: 2 prose breaches, in 3 of 3 runs |

## Bug found while building

An unquoted `**` in `replay.sh` expanded to the worktree's top-level files.
Every number above comes from the run after the fix (`set -f`).
