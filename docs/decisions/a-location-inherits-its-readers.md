# A location inherits its readers

## Context and Problem Statement

purba writes information into twelve kinds of location.
[An actor is what it does, not what it is](an-actor-is-what-it-does-not-what-it-is.md) named the eleven readers those locations serve.
No location says which of them it serves.

| | count |
|---|---|
| locations purba writes into | 12 |
| locations that name a reader | 0 |
| actors on the roster | 11 |

[What replaces the 11-stage pipeline?](https://github.com/kalonji-tools/purba/issues/21) requires that every artifact have a named reader.
The roster made that rule expressible.
Nothing has applied it to a location.

The prototype shows what an unanswered location question costs.

| prototype location | state | pages written | months open |
|---|---|---|---|
| wiki | enabled | 0 | 4 |
| discussions | enabled | 0 | 4 |

`CODEOWNERS` already maps a path to a person, and a reader will assume it is this map.
It is not.
`CODEOWNERS` protects the integrity of what is written, and this record decides who needs to read it.

## Considered Options

- **Bind each location to one reader.** Rejected. `docs/decisions/` serves the architect and the technical writer at the same time, and the roster already says so: the architect wants what was refused and why, and the technical writer wants the decisions a reader outside the work will read.
- **Let a child location override its parent, as `CODEOWNERS` does.** Rejected. The last matching pattern wins there, so `/docs/` naming the technical writer followed by `/docs/decisions/` naming the architect dismisses the technical writer from the record directory. Override is correct for approval and wrong for reading.
- **Author a reader list for every location.** Rejected. A commit, a pull request and a changelog hold no content of their own, so a second authored list can disagree with the thing it describes. [#28](https://github.com/kalonji-tools/purba/issues/28) settled the general form: a check that detects drift is worse than a design where drift cannot happen.
- **Route by the stage of work rather than by the location.** Rejected. The pipeline is unsettled and will change, and a record keyed to its stages would be rewritten on every revision. The roster was rejected for the same reason when it was keyed to incumbents.
- **Accumulate readers down the tree, and derive every location that has a source.** Chosen. Inheritance is the one rule that covers both, because a derived location inherits from what it is made of exactly as a path inherits from its parent.

## Decision Outcome

A location inherits its readers.

Inheritance runs two ways, and one rule therefore covers every location purba writes into.

**Down the path tree.**
A directory names the actors it adds.
A file's readers are the union of every binding on the path from the project root to that file.
A directory that binds nothing is transparent rather than empty: `src/config/pyproject.toml` reads the actors bound at `src/` even when `config/` binds none.

| location | adds |
|---|---|
| `README.md` | stem |
| `CLAUDE.md` | every contributor |
| `.github/`, `justfile`, `devenv.nix` | toolsmith |
| `docs/` | technical writer |
| `docs/decisions/` | architect |
| `src/` | coder, plugin author, tester |
| `tests/` | tester, coder |

The project root binds nothing.
A reviewer and a maintainer reach every location by what their role is, and a role that spans everything is not a binding.

**Along the derivation chain.**
A location built from another inherits that source's readers and is never authored twice.

| location | derives from |
|---|---|
| commit message | the paths it touches |
| pull request | the paths its diff touches |
| changelog | commit subjects |
| docs site | the stub |

The issue is the single authored exception, because it exists before there is a diff to derive from.

**A file that reaches no actor is flagged.**
The flag is not a rule violation.
It is three questions the file has failed to answer: should it exist, what does it serve, and for whom.
Asking where a new file lives forces the question of who it is for, and the answer is the binding, so the tree stays organised as a side effect of being readable.

This is scoped to the files purba tracks.
It never applies to a milestone, an issue or a pull request.

This record routes information to the reader who needs it.
It never withholds information from one.

**Downside:** three costs, and the first blocks the rest.

- **The language that expresses the tree is not chosen.** Nothing can read a binding until it is, so the flag is a design and not yet a check. `.gitignore` and `.gitattributes` are the shapes worth studying first.
- **The flag is only as sharp as the bindings above it.** A binding placed high and loosely silences every file beneath it, and nothing detects a binding that is technically true and useless.
- **A reader who wants to know when an actor reaches a location needs a second record.** This one binds an actor to a location and declines to enumerate the pipeline, because the pipeline is unsettled.

## Confirmation

**One half of this record is decidable, and the other half is not.**

The decidable half is the flag.

| the check | every tracked file resolves to at least one bound actor by inheritance |
|---|---|
| scope | files purba tracks, never a milestone, an issue or a pull request |
| wired today | no |
| what wires it | the representation language, once it is chosen |

The undecidable half is whether a location truly serves the actor it names.
Every register rule that gates successfully is a token, a required phrase, a section order, or a count, and this is none of those.
It fails as friction rather than as a violation, and friction is observed when a reader hits it.

| failure | what it looks like | the flag catches it |
|---|---|---|
| a location reaches nobody | a stem cannot reach a first success without asking a person | yes, once wired |
| a location reaches the wrong reader | an architect cannot find the prior decision and writes one that contradicts it | no |
| a reader is stranded | a reviewer cannot tell what the author discarded, and approves anyway | no |

The last of those is the one already measured.
[An actor is what it does, not what it is](an-actor-is-what-it-does-not-what-it-is.md) holds the count.
