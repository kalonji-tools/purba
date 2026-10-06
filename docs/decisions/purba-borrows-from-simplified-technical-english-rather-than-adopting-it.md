# purba borrows from Simplified Technical English rather than adopting it

## Context and Problem Statement

[An artifact holds the minimum that conveys its point](an-artifact-holds-the-minimum-that-conveys-its-point.md) put the defect in what an artifact admits.
It left how a sentence reads open.

The corpus was measured against the standard's rules before any rule was chosen.

| the records, measured | sentences |
|---|---:|
| read | 1,441 |
| over 25 words | 104 |
| with an `-ing` verb form | 63 |
| with a perfect or continuous tense | 28 |
| in the passive voice | 265 |

Two of these rules were already purba's.
`.template.md` says *"One idea in one sentence"* and *"Active voice"*, and names no source for either.

## Considered Options

- **The standard, adopted whole.** Rejected. Fifty rules written for aircraft maintenance, to gain the few that bind prose here.
- **A tool that checks the standard.** Rejected. The Vale package registry carries no entry for it, and every project that offers one calls itself an approximation.
- **A rule for every property that moved.** Rejected. A reading level moves twelve grades when a writer appends a backticked name, so the number falls and the prose does not improve.
- **The rules a command can decide, quoted and borrowed.** Chosen.

## Decision Outcome

purba borrows a prose rule from the standard, and never adopts the standard.

A borrowed rule is quoted in the words the standard uses.
No public source carries the rule numbers, so purba cites none.

**A rule refuses only where a command decides it.**

| the borrowed rule | force |
|---|---|
| no more than 25 words in descriptive text | refuses |
| no more than six sentences in a paragraph | refuses |
| the `-ing` form only as a technical noun | refuses |
| the simple tenses only | refuses |
| the active voice | reports |
| no wordy or formal word | reports |
| no part of a sentence left out | reports |
| no more than 20 words in a procedure | neither |

[A part-of-speech tagger decides the tense rule](a-part-of-speech-tagger-decides-the-tense-rule.md) says which command refuses a tense.

**The active voice is reported and never refused.**
The rule admits the passive where the agent is unknown.
A command cannot decide whether an agent is unknown.
[A standard a gate cannot decide does not become a gate](a-standard-a-gate-cannot-decide-does-not-become-a-gate.md).

**A list item carries a sentence, and a table cell does not.**
A cost under a Downside obeys the length rule, so a bullet is no way past it.
A table holds numbers on purpose, and a length rule there would push them back into prose.

**A link counts as one word.**
Record prose carries no issue number, so a link is titled with the question its issue asks.
That title is owed rather than chosen, and charging its words to the sentence would refuse the sentence that obeys.

**No record is a procedure, so the shorter limit reaches nothing.**

**Downside:**

- **The approved dictionary is excluded.** The list is not obtainable here, so the largest rule in the standard binds nothing.
- **The rule numbers are absent.** A reader holding the specification matches these rules to it by their words.
- **An `-ing` form reaches the gate in two positions only.** The detector reads what follows a form of `be` and what follows a preposition, so a gerund elsewhere passes.
- **A reported number binds nobody.** The passive voice is the largest deviation in the corpus and nothing refuses it.
- **The file stating these rules is not checked against them.** `.template.md` sits outside the glob, and its prose lives inside a comment the same command refuses elsewhere.

## Confirmation

| what proves it | where |
|---|---|
| `mise run records` refuses a sentence over 25 words | `scripts/check-records.sh` |
| the same command refuses a paragraph over six sentences | `scripts/check-records.sh` |
| the same command refuses an `-ing` form after `be` or a preposition | `scripts/check-records.sh` |
| the same command refuses `has`, `have` or `had` with a participle | `scripts/check-records.sh` |
| the same command reports the passive voice and refuses nothing | `scripts/check-records.sh` |
| the approved dictionary | a reader, and there will be no check |

Each refusal names the file and the line it found.
