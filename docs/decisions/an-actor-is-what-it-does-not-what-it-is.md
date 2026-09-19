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

## Considered Options

- **Sort actors by what they are: a person, a program, or an agent.** Rejected. No location is written differently on account of a reader being made of silicon, and one person occupies several actors in a single day.
- **Give each person and each program its own actor.** Rejected. This ties an actor to an incumbent, so the roster changes whenever a person changes role, and an unnamed future reader cannot appear on it at all.
- **Sort actors by the language they work in.** Rejected on measurement. In the prototype, 15 of 681 Python files sit outside the Python tree and their reader is the toolsmith. The same language reaches three readers, so the location carries the distinction and the roster does not need to.
- **List what each actor wants.** Rejected. A list of wants is never complete, and a reader checks whether it is on the list rather than whether it is in the position.
- **Name an actor by its role, and state the mindset that role is in.** Chosen. A mindset covers what a list of wants leaves out, and it is what a reader recognises itself by.

## Decision Outcome

An actor is a role that reads, stated as the mindset it reads from.

The roster is flat.

| actor | mindset |
|---|---|
| [**stem**](../../CONTEXT.md#stem) | I do not know this part of purba, and I need footing before I can do anything |
| [**architect**](../../CONTEXT.md#architect) | I must choose a direction, and I will be answerable for what it costs |
| [**coder**](../../CONTEXT.md#coder) | the decision is made, and I must build it correctly |
| [**toolsmith**](../../CONTEXT.md#toolsmith) | a mistake must be stopped before a person has to catch it |
| [**handler**](../../CONTEXT.md#handler) | an agent will act on what I write, and on nothing else |
| [**reviewer**](../../CONTEXT.md#reviewer) | I am about to become answerable for work I did not write |
| [**technical writer**](../../CONTEXT.md#technical-writer) | someone outside this work must understand it without asking |
| [**tester**](../../CONTEXT.md#tester) | I need purba to tell me the truth about my own code |
| [**plugin author**](../../CONTEXT.md#plugin-author) | I build against purba, and I need its shape to hold still |
| [**packager**](../../CONTEXT.md#packager) | I must get purba into an environment I do not control, and prove I am allowed to ship it |
| [**integrator**](../../CONTEXT.md#integrator) | purba must tell a machine what happened, the same way every time |

`contributor` and `user` are not actors.
Each was a heading over the rows beneath it, and a heading is taxonomy rather than a reader.
They are abbreviations, and [A location inherits its readers](a-location-inherits-its-readers.md) holds them.

**An actor is admitted, and two actors merge, by one test.**

An actor earns its place when its mindset is not contained by another's.
Two merge when one is contained, and the survivor is the larger.

Running it removed the maintainer.
A reader returning cold does not know this part of purba, which is the stem's mindset, and it becomes whichever actor the work then needs.
⚠️ A want list kept that role alive by naming a need no other row named. Stating the mindset instead made it visible as the stem.

**A reader does not leave the stem once.**
It returns whenever it meets a part of purba it does not know, so a coder sent into the pipeline is a stem again, and so is a coder sent into a language it does not write.

⚠️ The stem is the only actor purba can lose by doing nothing.
A stem that finds no way in leaves, and nothing records that it was ever there.

**A mode is not a second axis.**
An actor is a role stated as a noun and a mode is the same role stated as a verb, so `reviewer` and `reviewing` name one thing twice.

**An actor says nothing about who occupies it.**
One person occupies the architect, the reviewer and the toolsmith today, and an agent occupies the coder alongside that person.
A program is not an actor: `git-cliff` reads a commit subject and wants nothing, so it belongs in the route.
An agent is not a program in that sense and not an actor either, because it occupies an actor and reads what that actor reads.

⚠️ Three actors sit in both branches of the work, and one sits in both permanently.
A plugin author becomes a contributor when it sends a hook upstream, a packager when it sends a build fix, and a tester is both at once because purba tests itself with itself.

**Downside:** three costs, and the first is structural.

- **The merge test compares sentences a person wrote.** A mindset phrased narrowly survives a merge it should lose, and one phrased broadly swallows a role it should not. Rewording a single row can change the roster, and rewording one did.
- **The overlaps are recorded and not resolved.** Anything that later routes on an actor must decide which branch wins, and this record does not decide it.
- **The stem is scoped to a part of purba, and this record names no parts.** [A location inherits its readers](a-location-inherits-its-readers.md) names them, so a reader needs both records to know whether it is a stem right now.

## Confirmation

**One check reads this roster, and it lives in the other record.**

[A location inherits its readers](a-location-inherits-its-readers.md) rejects any name that is not on this list, so a typo cannot invent a twelfth actor.

⚠️ It is not wired yet, and [Build the reader-binding gate](https://github.com/kalonji-tools/purba/issues/85) owns it.

⚠️ No check decides whether a location truly serves the actor it names.
Every register rule that gates successfully is a token, a phrase, a section order or a count, and this is none of those.
