# A record names the role, not the kind of entity, unless the kind is what it decides

## Context and Problem Statement

[An actor is what it does, not what it is](an-actor-is-what-it-does-not-what-it-is.md) decides that an actor names nobody.
It says nothing on how a record names who acts.

| on `7850ef1`, when this was decided | count |
|---|---:|
| records | 97 |
| lines the scan found to name a kind of entity for a role | 77 |
| of those, kept, because a person decides | 9 |
| files that hold the 77 lines | 33 |
| records with no line to fix or to keep | 54 |

"A person" stood for any reader, any author, or work done by hand.
"Agent" stood for the doer of a sentence.
"Session" stood for the work.
A rule in those words reads as if it binds one kind of entity alone.

## Considered Options

- **Each writer names the entity they picture.** That was the practice, and it wrote the lines above.

## Decision Outcome

**Reach:** `docs/decisions/*.md` `scripts/check-records.sh` `/CONTEXT.md` `/AGENTS.md`

**Area:** writing

**A record names the role, and names the kind of entity only where the kind is what the line decides.**
A rule that binds a role binds each [entity](../../CONTEXT.md#entity) that fills it, so its words name the role.

- **A line about responsibility or liability names the kind.** Who answers, decides, signs, accepts or holds a credential rests on the kind. [Liability is recorded from the act that makes it true](liability-is-recorded-from-the-act-that-makes-it-true.md), [purba records delegation and does not prevent it](purba-records-delegation-and-does-not-prevent-it.md) and [the person owns the Decision Outcome](the-person-owns-the-decision-outcome.md) decide those.
- **A line about the hierarchy of entities names the kind.** [An actor is what it does, not what it is](an-actor-is-what-it-does-not-what-it-is.md) says which entity occupies which actor.
- **A line that reports a measurement names the kind it measured.** A fact about agents stays a fact about agents.
- **"Agent" keeps one sense.** It names an AI agent, so a record writes "who acts" for the doer of a sentence.

**Downside:**

- **Whether the kind is what a line decides is a judgement.** No command reads it, so a reviewer reads each line.
- **A line that names the role hides who fills it today.** A reader who needs to know reads the actor record.
- **The active-voice rule drops the word of the standard.** A reader who holds the standard matches [that rule](purba-borrows-from-simplified-technical-english-rather-than-adopting-it.md#decision-outcome) by its meaning.

## Confirmation

The reviewer of each change to `docs/decisions/` reads whether each line names the role.
[A standard a gate cannot decide does not become a gate](a-standard-a-gate-cannot-decide-does-not-become-a-gate.md), and no command decides this one.
[`AGENTS.md`](../../AGENTS.md) carries the rule to an agent before it writes a record.
`.template.md` carries it to a writer who starts a record.
