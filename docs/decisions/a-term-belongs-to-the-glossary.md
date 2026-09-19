# A term belongs to the glossary

## Context and Problem Statement

purba's records are written in a vocabulary that no record defines.

| term | records that use it | defined by a record |
|---|---:|---|
| `reader` | 12 | no |
| `gate` | 11 | no |
| `record` | 11 | yes |
| `location` | 8 | yes |
| `actor` | 6 | yes |
| `register` | 5 | yes |

Measured against the fourteen records that stood before this one.

A session answering a single question about the issue register read four records end to end to learn six words.

| | lines |
|---|---:|
| read | 408 |
| the corpus | 1,491 |

The corpus also disagrees with itself.

| the collision | where |
|---|---|
| `issue` and `ticket` name one thing | both appear in one sentence of [a register belongs to one location](a-register-belongs-to-one-location.md) |
| `maintainer` was removed from the actor roster and is still used as a role | [an actor is what it does, not what it is](an-actor-is-what-it-does-not-what-it-is.md) removes it, and two records use it ten times |

[A register belongs to one location](a-register-belongs-to-one-location.md) says a location does not open without a reader and a register.
The vocabulary had neither.

## Considered Options

- **Leave each definition in the record that decided it.** Rejected. The two most-used terms are decided by no record, so there is nowhere for them to sit.
- **A glossary that points at the records.** Rejected. It sends the reader back into the corpus, and an index is not a source of truth.
- **Define a term where it is first used.** Rejected. A record is rewritten in place, so first use moves whenever the corpus is reordered.
- **Move every definition out, including a roster.** Rejected. A roster is the outcome of a decision and not a definition, so moving it wholesale empties the record that earned it.
- **One glossary holding every definition, with a record keeping its decision and naming the glossary.** Chosen. It matches what this project already does twice: a record names a register rather than holding it, and points at a template rather than restating it.

## Decision Outcome

A term is defined once, in `CONTEXT.md`, and a record names it rather than holding it.

**The split.**

| holds | |
|---|---|
| `CONTEXT.md` | what a word is |
| a record | what purba decided, why, and what it costs |

**A record keeps its reasoning.**
A mindset, a decision outcome and a rationale each read like a definition, and each stays where it was written.

**A roster stays in its record, whole.**
Every row keeps its place and every column keeps its content, and each row gains a link.
The set at a glance is the point of the table, and that set is a decision belonging to the record that took it.

**A term qualifies when it is specific to purba.**
`crate`, `wheel` and `trailer` are borrowed from Rust, Python and git, and purba defines none of them.

**The form is a heading, and that is a deliberate departure.**
The convention this glossary follows writes a term in bold.
GitHub assigns an identifier to a heading and not to bold text, so a bold term cannot be linked and an index cannot reach it.
A term is written as a third-level heading, followed by one or two sentences saying what it is.
Where the corpus uses a second word for the same thing, an `_Avoid_` line names it.
An entry links to another entry in the same file, and never out of it.
A definition that sends the reader somewhere else is not a definition, so other locations link in here and this one links nowhere.

**Downside:**

- **A definition can drift from the prose that uses it.** It takes an author who did not consult the glossary and a reviewer who did not catch it, and nothing mechanical prevents either.
- **Telling a purba term from a general one is judgement.** No test separates them, and a term admitted wrongly makes the glossary the place where general programming is explained.
- **This opens a location that cannot yet declare its readers.** The file that holds a reader binding is not in the tree, so this location names its reader in prose until it is.

## Confirmation

Nothing checks this.
One property is decidable and ungated: no link in `CONTEXT.md` leaves the file.

No check decides whether a reader found its word here rather than in a record.
That fails as friction, and a reader reports it.

The glossary is `CONTEXT.md` at the repository root.
Its reader is the stem, and its register is this record.

[Build the reader-binding gate](https://github.com/kalonji-tools/purba/issues/85) owes this location a binding.
[Audit the decision records for alignment, cohesion and truth](https://github.com/kalonji-tools/purba/issues/89) holds the corpus-level link graph that this rewrites part of.
