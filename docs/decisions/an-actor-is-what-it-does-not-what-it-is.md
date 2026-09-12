# An actor is what it does, not what it is

## Context and Problem Statement

purba writes information into about twelve kinds of location.
No location states who reads it.

[What replaces the 11-stage pipeline?](https://github.com/kalonji-tools/purba/issues/21) already requires that every stage produce an artifact with a named reader.
That rule cannot be applied, because purba has no list of readers to name.

The prototype failed at exactly this point, and the failure was measured rather than inferred.

| measure | prototype |
|---|---|
| implementation plans that open by addressing agents by name | 291 of 499 |
| pull requests merged with zero review events | 770 of 859 |
| review events across the whole repository | 495 |
| of those, approvals | 14 |
| artifacts addressed to the human | 0 |

The pipeline worked as an agent-to-agent protocol and produced nothing for the person who owned it.

Three actors that a roster would normally carry have no incumbent today, because purba publishes no artifact and has no product to extend.

## Considered Options

- **Sort actors by what they are: a person, a program, or an agent.** Rejected. No location is written differently on account of a reader being made of silicon. The question is one purba never needs the answer to, and asking it hides the distinction that matters, because one person occupies several actors in a single day.
- **Give each person and each program its own actor.** Rejected. This ties an actor to an incumbent, so the roster changes whenever a person changes role, and an unnamed future maintainer cannot appear on it at all.
- **Name an actor by its role, and record the mode it is in when it reaches for information.** Chosen. The role is stable, the mode says what the reader is doing at that moment, and neither names a substance. Diátaxis reached the same structure for documentation and states it plainly: the user it serves is "the practitioner in a domain of skill", and its four categories are four modes of one reader rather than four audiences. purba applies that across every location rather than across documentation alone.

## Decision Outcome

<!-- ⚠️ THE HUMAN WRITES THIS SECTION. An agent writing it satisfies the letter and voids the rule. -->

<!-- The proposed roster follows, for the human to accept, amend or replace. -->

| # | actor | mode | want/need |
|---|---|---|---|
| 1 | architect | chooses the direction of a feature before it is built | the alternatives, the cost of the one taken, what was refused and why |
| 2 | newcomer | starts cold and learns the rules before touching anything | what to do, what stops you, where things live |
| 3 | implementer | builds one thing that is already decided | how to build it in order, the interfaces it must meet |
| 4 | reviewer | judges a change and becomes answerable for it | what changed, whether it honours a decision, what the author discarded |
| 5 | maintainer | returns cold to something built long ago | why this approach, what would show it wrong, what depends on it |
| 6 | user | runs its own tests with the product | how to use it, what changed between releases |
| 7 | plugin author | writes a plugin that hooks into the product | plugin interface docs, changelog, roadmap |
| 8 | contributor | sends a change from outside the project | how work is accepted here, the rules a change must meet |

Actors 6, 7 and 8 have no incumbent.
purba names them and does not design for them.

An actor says nothing about who occupies it.
One person occupies actors 1, 4 and 5 today, and an agent occupies 2 and 3 alongside that person.

Actor 2 is occupied more often than any other.
Every agent session starts cold, so the project is read by a newcomer many times a day and by an architect a few times a month.

A program that reads a location is not an actor.
`git-cliff` reads a commit subject and `maturin` reads a doc comment, and neither wants anything.
Each carries information to an actor further along, so a program belongs in the route rather than in the roster.

## Downside

Three costs, and the first is structural.

- **A mode is a judgement, and nothing decides where one ends.** Whether arriving cold at a line of code makes a reader a newcomer or a maintainer is a call a person makes, and two people can make it differently. The roster gives no procedure for settling that.
- **Dropping the substance hides a real constraint.** A program that carries information to an actor still dictates the form a location must take, and the roster cannot express that. The next record carries it instead, which means the constraint is real and recorded somewhere this record does not point to.
- **Naming an actor with no incumbent invites designing for it.** The prototype wrote 20,705 tracked lines of documentation for a framework that had no users, so this is a demonstrated failure rather than a hypothetical one.

## Confirmation

**Nothing checks the roster today.**

The three absences are checkable now, and they are the only rows that are.

| claim | check | result on 2026-09-13 |
|---|---|---|
| purba has no users | `grep '^publish' Cargo.toml` and `gh release list` | `publish = false`, zero releases |
| purba has no plugin authors | `wc -l src/lib.rs` | 19 lines, a module stub, no interface to write against |
| purba has no contributors | `gh api repos/kalonji-tools/purba/contributors` | one account, `snregales-agent`, 8 contributions |

The rest becomes checkable only when a location names its actor, which is the next record and not this one.

⚠️ Expect that check to be weak when it arrives.
Every register rule that gates successfully is a token, a required phrase, a section order, or a count.
An actor is none of those, so a check can decide only whether a named actor is on the list, never whether the artifact truly serves it.
