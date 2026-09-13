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

- **Sort actors by what they are: a person, a program, or an agent.** Rejected. No location is written differently on account of a reader being made of silicon. The question is one purba never needs the answer to, and asking it hides the distinction that matters, because one person occupies several actors in a single day.
- **Give each person and each program its own actor.** Rejected. This ties an actor to an incumbent, so the roster changes whenever a person changes role, and an unnamed future reader cannot appear on it at all.
- **Name an actor by its role, and record the mode it is in when it reaches for information.** Chosen. The role is stable, the mode says what the reader is doing at that moment, and neither names a substance. Diátaxis reached the same structure for documentation and states it plainly: the user it serves is "the practitioner in a domain of skill", and its four categories are four modes of one reader rather than four audiences. purba applies that across every location rather than across documentation alone.

## Decision Outcome

Three actors sit at the top.
A contributor changes purba.
A user runs purba without changing it.
A stem is neither yet, and every contributor and every user begins there.

The roster is one level deep beneath the contributor and the user.

| actor | mode | want/need |
|---|---|---|
| **stem** | has arrived, is neither yet, and can become any of them | what purba is and whether it fits, how to reach a first success, where things are, how to stand up a workspace, which issues are safe to start on, and which known problems nobody has had time to fix |
| **contributor** | changes purba | the six below |
| architect | chooses the direction of a feature before it is built | the alternatives, the cost of the one taken, what was refused and why |
| coder | writes the product code for one thing already decided | how to build it in order, the interfaces it must meet |
| toolsmith | builds and keeps the pipelines, the hooks and the developer tooling | what each gate decides, how to run the gates locally, and what breaks when one changes |
| reviewer | judges a change and becomes answerable for it | what changed, whether it honours a decision, what the author discarded |
| maintainer | returns cold to something built long ago | why this approach, what it assumed, and what depends on it |
| technical writer | writes what a reader outside the work will read | the decisions, the public interfaces, what changed and why it matters |
| **user** | runs purba without changing it | the four below |
| tester | writes tests and runs them with purba | how to use it, how to configure it, what changed between releases |
| plugin author | writes a plugin that hooks into purba | the plugin interfaces, the changelog, the roadmap |
| packager | distributes purba to a platform's own users | build requirements, supported platforms, licences, the release artifacts |
| integrator | wires purba into another project's pipeline | exit codes, machine-readable output, configuration, the supported version range |

The stem takes its name from the stem cell and carries that cell's property, which is pluripotency: it can become any actor beneath either parent, and often several.

⚠️ A reader does not leave the stem once.
It returns to the stem whenever it meets a part of purba it does not know.
A coder who is asked to change the pipeline that afternoon is a stem again, and wants what a stem wants.

⚠️ The stem is also the only actor purba can lose by doing nothing.
A stem that finds no way in leaves, and nothing records that it was ever there.

purba designs for every actor on this list.
An actor with no incumbent today still shapes what purba builds, in the same way a product line is designed with its customers and its factories in mind.

The mode column names the mode an actor is characteristically in.
A mode is not a second axis.
An actor is a role stated as a noun and a mode is the same role stated as a verb, so `reviewer` and `reviewing` name one thing twice.

The roster is fine-grained only to the point where each actor is distinct and can be assembled with the others.
A change passes through several actors, and the line it walks is that change's life.
An actor earns a place when it is a brick that line cannot be built without.
The line's nodes are the stages of purba's pipeline, and no record enumerates them, because the pipeline is unsettled and a record keyed to it would be rewritten on every revision.
[A location inherits its readers](a-location-inherits-its-readers.md) binds an actor to a location instead, which is the binding that holds whatever the pipeline turns out to be.

Three actors sit in both branches, and one of them sits in both permanently.
A plugin author becomes a contributor the moment it sends a hook upstream.
A packager becomes a contributor the moment it sends a build fix.
⚠️ A tester of purba is a contributor and a user at the same time, because purba is a test framework that tests itself with itself.
That overlap is not an event that happens to a reader. It is what dogfooding means, and it holds for every test purba runs on its own code.

An actor says nothing about who occupies it.
One person occupies the architect, the reviewer and the maintainer today, and an agent occupies the coder alongside that person.

A program that reads a location is not an actor.
`git-cliff` reads a commit subject and `maturin` reads a doc comment, and neither wants anything.
Each carries information to an actor further along, so a program belongs in the route rather than in the roster.

## Downside

Three costs, and the first is structural.

- **The stem is scoped to a part of the project, and the roster names no parts.** Whether a reader is a stem depends on which part of purba it is looking at, so the question cannot be answered from this record alone. [A location inherits its readers](a-location-inherits-its-readers.md) names the parts, so a reader needs both records to answer it.
- **The intersections are recorded and not resolved.** Three actors sit in both branches, and the tester sits in both permanently rather than on an event. Anything that later routes on an actor has to decide which branch wins, and this record does not decide it.
- **Distinctness is a judgement.** An actor earns a place when it is distinct and the assembly cannot be built without it, and nothing decides distinctness except a person. Eleven actors stand here, and the rule refuses a twelfth only as well as that judgement holds.

## Confirmation

**Nothing enforces the roster today.**

[A location inherits its readers](a-location-inherits-its-readers.md) names the actor each location serves, and it carries the only check either record proposes.
That check flags a tracked file that reaches no actor on this list.
It reads the roster, so the roster becomes enforceable the day the check is wired, and it is not wired yet.

⚠️ A check cannot decide the thing that matters most.
Every register rule that gates successfully is a token, a required phrase, a section order, or a count.
Whether a location truly serves the actor it names is none of those, so that half stays a judgement and the record says so rather than inventing a check for it.
