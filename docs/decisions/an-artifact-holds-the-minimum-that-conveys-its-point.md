# An artifact holds the minimum that conveys its point

## Context and Problem Statement

purba's artifacts are read twice to be understood.

A written rule changes what is written. Records forbid em-dashes and carry almost none.

| location | its written rule | em-dashes per thousand words |
|---|---|---:|
| decision records | `.template.md` | 0.13 |
| commit messages | [a commit outlives its review](a-commit-outlives-its-review.md) | 0.34 |
| issue bodies | none | 10.1 |

The complaint named padding, and padding is absent. A wordiness rule of 119 patterns finds nothing in the records. The corrections point instead at material another location already holds, such as the reasons the history carries.

So the defect is what an artifact admits, not how its sentences read.

## Considered Options

- **A reading level.** Rejected. Six formulas disagree by seven grades on one record, and appending backticked identifiers moves the score twelve grades while the words stay the same.
- **A wordiness linter.** Rejected. It finds nothing here, because the defect is admission rather than wording.
- **ASD-STE100, adopted whole.** Rejected. Fifty rules for aircraft maintenance to gain three ideas, and the standard sits where a reader must ask for it. [purba borrows from Simplified Technical English rather than adopting it](purba-borrows-from-simplified-technical-english-rather-than-adopting-it.md) names the rules purba took instead.
- **One voice standard across every location.** Rejected. No established project states one. Git requires an imperative subject and says nothing about its own code comments.
- **Subtraction, meaning remove whatever can be removed.** Rejected. A table, a diagram and an example are admissible when they carry the point.
- **Sufficiency, with named categories beneath it.** Chosen. It says what an artifact is for rather than how long it may be.

## Decision Outcome

An artifact holds the minimum that conveys its point.

**The test is a judgement.**
No command decides whether an element carries the point.
A reviewer decides, and this record does not pretend otherwise.

**Provenance crosses every location.**
What a session leaves behind does not enter an artifact.
Discarded alternatives, intermediate edits and the order the work happened in stay where they happened, unless a reader needs them to understand the result.

**A category is named by a review, and enters when this record is rewritten.**

| material that never carries the point |
|---|
| what a session leaves behind |
| a count of the costs on the `**Downside:**` label |

**The rules below are decidable, and a command owns them.**

| rule | shape |
|---|---|
| the `**Downside:**` label stands alone | an opening phrase |
| the label states no count | an opening phrase |
| the costs are a list | a count |
| bolded lead-ins equal the list items under the label | a count |
| no em-dash outside a code span or a fence | a character |
| bold opens a sentence | a position in a sentence |

A record carrying no label has no list of costs either, so the third rule is what a missing Downside breaks.

**A prose rule is decided on the sentence, not the line.**
The two rules above are written about sentences and a detector reads lines.
A line-kind test refuses a second lead-in that opens a sentence in the middle of its line, and the corpus writes two of those.
Reading sentence position drops the line classifier and refuses neither.

**Bold is for a lead-in, and a reviewer decides whether a bolded phrase is one.**
The command refuses bold inside a sentence, which is the half it can decide.
The purpose of bold stays where the diagram rule sits, and the Downside names what that costs.

A diagram may not raise the record's word count.
That rule is decidable and a reviewer decides it, because the length of a record without its diagram is not a number the tree holds.

**Evidence density is reported and refuses nothing.**
It counts the prose lines that carry no number, no code span and no link.
`.template.md` drives that number up, because it moves numbers into tables.
A threshold would refuse the records that obey.

**Downside:**

- **The rule is a judgement.** Two reviewers can read one artifact differently, and this record calls neither wrong.
- **Naming a category costs a rewrite.** A record is replaced rather than appended to, so every category a review finds is paid for again.
- **A count can refuse an artifact whose every element carries the point.** Evidence density is reported for that reason, and a word budget carries the same flaw.
- **A reported number binds nobody.** It can be read and ignored indefinitely.
- **The file stating the prose rules is not checked against them.** `.template.md` sits outside the glob, and its rules live inside a comment the same command refuses elsewhere.
- **Bold used for emphasis at the head of a sentence passes.** A bolded table row is not a lead-in, and no rule here refuses it.

## Confirmation

| what proves it | where |
|---|---|
| `mise run lint:records` refuses a `**Downside:**` label carrying a preamble or a count | `scripts/check-records.sh` |
| the same command refuses a Downside that states no list of costs | `scripts/check-records.sh` |
| the same command refuses a label whose bolded lead-ins do not equal its list items | `scripts/check-records.sh` |
| the same command reports evidence density and refuses nothing on it | `scripts/check-prose/check-prose.rs` |
| the same command refuses an em-dash, and exempts one inside a code span or a fence | `scripts/check-prose/check-prose.rs` |
| the same command refuses bold that does not open a sentence | `scripts/check-prose/check-prose.rs` |
| the diagram rule, and the sufficiency test under it | a reviewer, and there will be no check |
| whether a bolded phrase is a lead-in | a reviewer, and there will be no check |

Each refusal names the file and the line it found.
The command reports every rule before it exits, so a writer fixing one refusal finds the next in the same run.

[The glossary stays open, and a gate rewrites each word it avoids](the-glossary-stays-open-and-a-gate-rewrites-each-word-it-avoids.md) answers whether the vocabulary closes.
