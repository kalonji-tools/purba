# The decision sentence is the winner, and Considered Options holds only what lost

## Context and Problem Statement

The comment in `.template.md` said that Considered Options holds what lost.
The template's form put the winner in that list, as `{Chosen. Why it won.}`.
Each record followed the form.
A reader who looked for the decision met the losers first.

| on `main` when this was decided | count |
|---|---:|
| records | 95 |
| records whose winner is the last option | 71 |
| records that mark two to eight winners among the losers | 21 |
| records a split into one question each would make | up to 42 |
| lost options that open with a form of `Rejected` | 297 of 302 |
| records that bold the decision sentence | 75 |

## Considered Options

- **The winner stays in the list, marked `Chosen.`** A reader meets each loser before the decision. In some records, several winners sit among the losers.
- **A heading for each sub-question, with its winner last under it.** A reader still scans each list for its winner.
- **A record for each sub-question, with one winner, last.** Each new record repeats a Context that it shares with another.
- **`Rejected.` on each lost option, as the records wrote it.** In a list of losers it marks nothing.
- **Strip only `Rejected.`, and let another form pass.** Each other form keeps the mark, and nothing refuses a new one.
- **The decision sentence in plain text, as the form wrote it.** The decision does not stand out. No rule said which form a record writes.

## Decision Outcome

**Reach:** `docs/decisions/*.md` `scripts/records.sh` `scripts/check-records.sh` `scripts/unmark-lost-options.sh` `scripts/test/check-records.bats` `scripts/test/unmark-lost-options.bats` `/.config/tasks.toml`

**Area:** writing, tooling

**The bold decision sentence is the winner, and Considered Options holds only what lost.**
A reader finds the decision in one place in each record, after the area.

- **Several winners are a list under the bold sentence.** A winner that the bold sentence states takes its reason on the next line. Each other winner that Decision Outcome does not already state is `- **{winner}.** {Why it won.}`.
- **A record with several winners stays one record.** [A new question gets a new record](a-new-question-gets-a-new-record.md) makes a new record only where an issue asks a question that no title answers.
- **A winner that Decision Outcome already states keeps its place there.** Its reason joins that sentence, and the record does not write the winner twice.
- **A lost option states why it lost, with no mark.**
- **`mise run lint:records` refuses `Chosen` after the lead-in of an option in Considered Options.**
- **`mise run fmt:records` strips `Rejected.` after the lead-in of an option.** [A gate owns the mechanical standard](a-gate-owns-the-mechanical.md) says that where a fixer exists, it writes.
- **`mise run lint:records` refuses any `Rejected` that opens a lost option.** The author rewrites any other form by hand, because only the author can keep what the form adds.
- **`mise run lint:records` refuses the first line after the area unless `**` wraps that line and its words end with `.`, `?` or `!`.** A reviewer would otherwise state the rule by hand, and a gate owns such a rule.
- **An option enters Considered Options only where its question weighed it.** [An artifact holds the minimum that conveys its point](an-artifact-holds-the-minimum-that-conveys-its-point.md#decision-outcome) says when an option is weighed.

**Downside:**

- **A winner sits apart from the losers it beat.** A reader compares them across two sections.
- **No word marks the winner for a search.** A search for `Chosen.` finds no winner, and the reader looks for the bold sentence after the area.
- **A record with several winners still answers several questions.** Its title can state one of them, and no command reads whether it states each.
- **Whether a reason reads right under the bold sentence is a judgement.** A reason written about an option can name the wrong subject there, and only a reviewer reads it.
- **No command reads whether the bold line holds one sentence.** A bold line of two sentences passes the gate, and only a reviewer sees it.

## Confirmation

`mise run lint:records` refuses `Chosen` after the lead-in of an option in Considered Options, and it refuses any `Rejected` there too.
It also refuses the first line after the area unless `**` wraps that line and its words end with `.`, `?` or `!`.
`mise run fmt:records` strips each `Rejected.` that a space or the end of the line follows.
`mise run lint` runs `lint:records`, and `Quality` runs it whenever a pull request changes a path that the lint reads.
