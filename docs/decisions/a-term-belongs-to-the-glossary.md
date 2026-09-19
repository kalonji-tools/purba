# A term belongs to the glossary

## Context and Problem Statement

purba states its decisions in records.
Those records are written in a vocabulary that no record defines.

| term | records that use it | defined by a record |
|---|---:|---|
| `reader` | 12 | no |
| `gate` | 11 | no |
| `record` | 11 | yes |
| `location` | 8 | yes |
| `actor` | 6 | yes |
| `register` | 5 | yes |

Measured against the fourteen records that stood before this one.
The two most-used terms are load-bearing everywhere and owned nowhere.

A reader who needs one word pays for that.
A session answering a single question about the issue register read four records end to end to learn six words.

| | lines |
|---|---:|
| read to learn six words | 408 |
| the record corpus | 1,491 |

The corpus also disagrees with itself, and a reader cannot tell which word is meant.

| the collision | where |
|---|---|
| `issue` and `ticket` name one thing | both appear in one sentence of [a register belongs to one location](a-register-belongs-to-one-location.md) |
| `maintainer` was removed from the actor roster and is still used as a role | [an actor is what it does, not what it is](an-actor-is-what-it-does-not-what-it-is.md) removes it, and two records use it ten times |

A rule that nothing carries is the failure this project has already recorded once.
[A register belongs to one location](a-register-belongs-to-one-location.md) says a location does not open without a reader and a register.
The vocabulary had neither.

## Considered Options

- **Leave each definition in the record that decided it.** Rejected. The two most-used terms are decided by no record, so there is nowhere for them to sit, and a reader still opens a record to learn a word.
- **A glossary that points at the records.** Rejected. It sends the reader back into the corpus, which is the cost this decision exists to remove, and an index is not a source of truth.
- **Define a term where it is first used.** Rejected. First use is not stable. A record is rewritten in place, so the definition moves whenever the corpus is reordered.
- **Move every definition out, including a roster.** Rejected. A roster is the outcome of a decision and not a definition, so moving it wholesale empties the record that earned it.
- **One glossary holding every definition, with a record keeping its decision and naming the glossary.** Chosen. It matches what this project already does twice: a record names a register rather than holding it, and points at a template rather than restating it.

## Decision Outcome

A term is defined once, in `CONTEXT.md`, and a record names it rather than holding it.

**The split.**

| holds | |
|---|---|
| `CONTEXT.md` | what a word is |
| a record | what purba decided, why, and what it costs |

**A decision outcome is not a definition.**
A record states what purba decided, and that sentence can read like a definition.
It stays where it is, because a decision belongs to the record that took it, and the glossary carries the entry a reader looks up.

**A roster stays in its record.**
Every row keeps its place and each row gains a link.
The set at a glance is the point of the table, and that set is a decision belonging to the record that took it.
The definition of one row is a word, and that belongs in the glossary.

**A term qualifies when it is specific to purba.**
A general programming concept does not, even where purba uses it constantly.
`crate`, `wheel` and `trailer` are borrowed from Rust, Python and git, and purba defines none of them.

**The form is a heading, and that is a deliberate departure.**
The convention this glossary follows writes a term in bold.
GitHub assigns an identifier to a heading and not to bold text, so a bold term cannot be linked and an index cannot reach it.
A term is written as a third-level heading, followed by one or two sentences saying what it is.
Where the corpus uses a second word for the same thing, an `_Avoid_` line names it.
An entry links to another entry in the same file, and links outward for anything else.

**Downside:**

- **A definition can drift from the prose that uses it.** It takes an author who did not consult the glossary and a reviewer who did not catch it, so nothing mechanical prevents it and two people have to miss it.
- **Telling a purba term from a general one is judgement.** No test separates them, and a term admitted wrongly makes the glossary the place where general programming is explained.
- **This opens a location that cannot yet declare its readers.** The file that holds a reader binding is not in the tree, so this location names its reader in prose until it is.

## Confirmation

Nothing checks this, and the departure above is the only part a tool could see.

**This is wrong if a reader still opens a record to learn a word.**
That is the behaviour the decision exists to remove, and it is observed rather than measured.
A reader reports it as friction.

The glossary is `CONTEXT.md` at the repository root.
Its reader is the stem, and its register is this record.

[Build the reader-binding gate](https://github.com/kalonji-tools/purba/issues/85) owes this location a binding.
[Audit the decision records for alignment, cohesion and truth](https://github.com/kalonji-tools/purba/issues/89) holds the corpus-level link graph that this rewrites part of.
