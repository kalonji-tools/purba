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
A record does not.

Four sections are required, in order:

1. Context and Problem Statement
2. Considered Options
3. Decision Outcome
4. Confirmation

Consequences is optional. The downside is required, inside Decision Outcome.

The title is a proposition.
Filenames carry no numbers and there is no index.
`ls` and `grep` are the whole interface.

Records cite each other by proposition, never by number.
A proposition survives a rename and is falsifiable at a glance.
A number cannot be checked against the thing it names, and one prototype record was cited inverted twice.

When one record excepts another, the general record names the exception and links it, and the exception links back.
A general rule that hides its exception is a trap, because the reader who applies the rule never opens the record that would correct them.
The link is prose inside the section it bears on, and never a status field.

Issue numbers never appear in record prose.
The record states what is true, the issue states what happened, and the prototype leaked 417 issue references the wrong way across that line.

A link titled by the question its ticket asks is the exception, and it is a citation of the question rather than of the number.
A title is falsifiable where it stands and a bare number is not, so a reader who never opens the link still knows what was claimed.

A programme is a milestone, not a record.

**Downside:**

- **The structural cap is gone.** Four sections absorb twenty amendments where one sentence could not, so the 1,226-line record can happen again. What replaces the cap is weaker: a minimal template, the milestone, and a command that cannot see content.
- **Making Consequences optional costs a real check.** Two prototype amendments found genuine drift by auditing a large Consequences section.
- **A record must be rewritten whenever a ticket it waits on closes.** Nothing prompts that rewrite. The Confirmation states the obligation and says no check decides it, and a ticket can close for reasons that have nothing to do with the record that named it.

## Confirmation

`mise run records`, over `docs/decisions/`, checking only what is decidable:

| check | strength |
|---|---|
| the four sections are present, in order | strong |
| record prose carries no bare issue number | strong |
| source carries no numbered-record citation | strong |
| a Confirmation that says a gate is unwired names the ticket that will wire it | strong for the link, weak for the admission, which is matched as a phrase |

The command refuses nothing at a commit and nothing at a merge.
Whether a sentence admits an unwired gate is a judgement, so the check that finds one may suggest and may not gate.
It is run by hand before a pull request opens.
[Audit the decision records for alignment, cohesion and truth](https://github.com/kalonji-tools/purba/issues/89) repairs the records it refuses, and the command joins the quality gate once they land.

Whether a ticket a record names as owing work is still open is read by the same reader, and no check decides it.
A record cites two kinds of ticket: one it waits on, which must be open, and one recording where a measurement was made, which is closed by the time it is cited.
One record here cites both, so a state check alone refuses it wrongly.

A merged record links a ticket that closed while the gate it owned stayed unwired, and nothing read the two together.
The fourth check reads the link and never the claim, because the link is the part a command can decide.

Two things are never gateable: whether a record states one decision, and whether it is still true.
The code owner judges both, and is the only reader who can.
