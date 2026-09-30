# An artifact holds the minimum that conveys its point

## Context and Problem Statement

purba's artifacts are read twice to be understood.

A written rule changes what is written. Records forbid em-dashes and carry almost none.

| location | its written rule | em-dashes per thousand words |
|---|---|---:|
| decision records | `.template.md` | 0.13 |
| commit messages | [a commit outlives its review](a-commit-outlives-its-review.md) | 0.34 |
| issue bodies | none | 10.1 |

The complaint named padding, and padding is absent. A wordiness rule of 119 patterns finds nothing in the records. The corrections point instead at material another location already holds, such as reasoning the history carries.

So the defect is what an artifact admits, not how its sentences read.

## Considered Options

- **A reading level.** Rejected. Six formulas disagree by seven grades on one record, and appending backticked identifiers moves the score twelve grades without changing a word.
- **A wordiness linter.** Rejected. It finds nothing here, because the defect is admission rather than wording.
- **ASD-STE100, adopted whole.** Rejected. Fifty rules for aircraft maintenance to gain three ideas, and the standard sits where a reader must ask for it. Borrowing one mechanism stays open.
- **One voice standard across every location.** Rejected. No established project states one. Git requires an imperative subject and says nothing about its own code comments.
- **Subtraction, meaning remove whatever can be removed.** Rejected. A table, a diagram and an example are admissible when they carry the point.
- **Sufficiency, with named categories beneath it.** Chosen. It says what an artifact is for rather than how long it may be.

## Decision Outcome

<!-- OWED TO THE HUMAN. signoff.yml says a person writes this section. Adopt, edit or rewrite. -->

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

**Four rules are decidable, and a command owns them.**

| rule | shape |
|---|---|
| the `**Downside:**` label stands alone | an opening phrase |
| the label states no count | an opening phrase |
| bolded lead-ins equal the list items under the label | a count |
| a diagram may not raise the word count | a count |

**Evidence density is reported and refuses nothing.**
It counts the lines of running text carrying no number, no code span and no link, and `.template.md` drives that number up by moving numbers into tables.
A threshold would refuse the records that obey.

**Downside:**

- **The rule is a judgement.** Two reviewers can read one artifact differently, and this record calls neither wrong.
- **Naming a category costs a rewrite.** A record is replaced rather than appended to, so every category a review finds is paid for again.
- **A count can refuse an artifact whose every element carries the point.** Evidence density is reported for that reason, and a word budget carries the same flaw.
- **A reported number binds nobody.** It can be read and ignored indefinitely.

## Confirmation

| what proves it | where |
|---|---|
| `mise run records` refuses a `**Downside:**` label carrying a preamble or a count | not wired yet |
| the same command refuses a label whose bolded lead-ins do not equal its list items | not wired yet |
| evidence density is reported, and refuses nothing | not wired yet |
| the sufficiency test | no check, and there will not be one |

[Write the prose standard into the templates and gate what a command can decide](https://github.com/kalonji-tools/purba/issues/216) wires the first three. [Does CONTEXT.md close, so a word not on it may not be used?](https://github.com/kalonji-tools/purba/issues/218) asks whether the vocabulary closes.
