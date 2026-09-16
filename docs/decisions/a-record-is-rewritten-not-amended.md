# A decision record is rewritten, not amended

## Context and Problem Statement

A decision record has to be trustworthy on one reading, without replaying the project's history.
The prototype's records were not.

| prototype corpus | |
|---|---|
| records | 19 |
| total lines | 3,373 |
| largest record | 1,226 lines, 36% of the corpus |
| amendment headings | 40, of which one record held 22 |
| records ever superseded | 0 |

The largest record broke its own container.
Amendments 6 to 20 sat underneath the Consequences heading, and nobody noticed.
The record says so in its own text and tells the reader to skip the rule body below it.

Two causes are separable and both are real.

| the title names | records | amendment headings | mean |
|---|---|---|---|
| a proposition | 10 | 6 | 0.6 |
| a topic | 9 | 34 | 3.8 |

Granularity is the primary cause. Mutability is the amplifier.

No-duplication discipline does not rescue this.
The prototype's context document pointed at records instead of restating them, and pointing at an unreadable record does not make it readable.

## Considered Options

- **Append-only, with a status field and amendments.** Rejected. The prototype had exactly this. Its supersede path was chosen zero times in nineteen records, and its status field became nine nested prose paragraphs.
- **A single-sentence record.** Rejected. It is structurally capped, because a sentence cannot absorb twenty amendments. It lost on tooling: the most-starred record tool has made no release since 2018.
- **A public Markdown convention, cut to its minimum, plus a fitness function.** Chosen. It has no runtime, so nothing can go dead, and its Confirmation section already is the staleness mechanism.

## Decision Outcome

A record is a Markdown file at `docs/decisions/<proposition-slug>.md`, it reads as current state, and it is replaced rather than annotated.

This is the exception to [an artifact is rewritten until its direction is agreed](an-artifact-is-rewritten-until-its-direction-is-agreed.md).
Every other artifact this project writes freezes once its direction is agreed.
A record does not, because a record that reads as history is a record nobody can trust on one reading.

Four sections are required, in order:

1. Context and Problem Statement
2. Considered Options
3. Decision Outcome
4. Confirmation

Consequences is optional. The downside is required, inside Decision Outcome.

The title is a proposition.
Filenames carry no numbers and there is no index.
`ls` and `grep` are the whole interface, and a generated index is declined knowingly.

Records cite each other by proposition, never by number.
A proposition survives a rename and is falsifiable at a glance.
A number cannot be checked against the thing it names, and one prototype record was cited inverted twice.

When one record excepts another, the general record names the exception and links it, and the exception links back.
A general rule that hides its exception is a trap, because the reader who applies the rule never opens the record that would correct them.
The link is prose inside the section it bears on, and never a status field.

Issue numbers never appear in record prose.
The record states what is true, the issue states what happened, and the prototype leaked 417 issue references the wrong way across that line.

A programme is a milestone, not a record.

**Downside:** the structural cap is gone. Four sections absorb twenty amendments where one sentence could not, so the 1,226-line record can happen again. What replaces the cap is weaker: a minimal template, the milestone, and a linter that cannot see content. Making Consequences optional costs a real check too, because two prototype amendments found genuine drift by auditing a large Consequences section.

## Confirmation

A linter over `docs/decisions/`, checking only what is decidable:

| check | strength |
|---|---|
| the four sections are present | strong |
| record prose carries no issue-number citation | strong |
| source carries no numbered-record citation | strong |
| cited paths exist | weak: 4 of 67 prototype paths were dead, and 2 of those were illustrative prose |

The linter does not exist.
The decision-record linter ticket wires it, and that ticket is blocked on these records existing to lint.
Until then the checks run by hand before a pull request opens.

Two things are never gateable: whether a record states one decision, and whether it is still true.
A detector good enough to suggest is not good enough to gate.
The code owner judges both, and is the only reader who can.
