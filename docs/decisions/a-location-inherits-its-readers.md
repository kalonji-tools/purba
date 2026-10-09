# A location inherits its readers

## Context and Problem Statement

[An actor is what it does, not what it is](an-actor-is-what-it-does-not-what-it-is.md) named purba's readers.
No location says which of them it serves, so [What replaces the 11-stage pipeline?](https://github.com/kalonji-tools/purba/issues/21)'s rule that every artifact have a named reader cannot be applied.

The prototype shows what an unanswered location question costs.

| prototype location | state | pages written | months open |
|---|---|---|---|
| wiki | enabled | 0 | 4 |
| discussions | enabled | 0 | 4 |

## Considered Options

- **Bind each location to one reader.** Rejected. `docs/decisions/` serves the architect and the technical writer at once, and the roster already says so.
- **Let a child override its parent, as GitHub's `CODEOWNERS` does.** Rejected. The last matching pattern wins there, so a child naming the architect would dismiss the technical writer. Override is correct for an approval and wrong for a reader.
- **Author a reader list for every location.** Rejected. A commit and a changelog hold no content of their own, so a second authored list can disagree with the thing it describes. [Decide purba's documentation harness and example verification](https://github.com/kalonji-tools/purba/issues/28) settled the form: a check that detects drift is worse than a design where drift cannot happen.
- **Route by the stage of work rather than by the location.** Rejected. A stage can be reordered or removed while every location stays, so a record keyed to stages is rewritten on each revision.
- **Invent a binding syntax.** Rejected after a prototype. The smallest thing that works is directory prefixes, and prefixes cannot express `*.toml`, which in the prototype reached 956 of 1,109 files in four patterns.
- **Implement part of gitattributes syntax by hand.** Rejected. A file that looks like gitattributes and supports less than gitattributes lies about itself. The subset is invisible until someone writes a pattern it silently ignores.
- **Accumulate readers down the tree, written in gitattributes syntax and parsed by gitattributes' own parser.** Chosen.

## Decision Outcome

**Reach:** `**`

A location inherits its readers.

**The words this record uses are defined once.**
[A term belongs to the glossary](a-term-belongs-to-the-glossary.md) holds them.

**Down the path tree.**
A directory names the actors it adds, and a file's readers are the union of every binding from the project root down to it.
A directory that binds nothing is transparent: `src/config/pyproject.toml` reads `src/`'s actors when `config/` binds none.

The bindings live in [`.config/readers`](../../.config/readers), in gitattributes syntax.

The project root binds nothing.
A reviewer reaches every location by what its role is, and a role that spans everything is not a binding.
⚠️ Bind anything at the root and every file inherits it, which leaves the check below unable to fail.

**Along the derivation chain.**
A location built from another inherits that source's readers and is never authored twice.
A commit message and a pull request derive from the paths they touch, and a [refusal](../../CONTEXT.md#refusal) from what it refuses.
`CLAUDE.md` derives from `AGENTS.md`.
Gitattributes syntax cannot say so.
Both files name one macro, so their readers are still written once.

**A tracked file that reaches no actor fails the build.**
The failure states the rule, as [a refusal states the rule, why it holds, and where it was found](a-refusal-states-the-rule-why-it-holds-and-where-it-was-found.md) requires.
It then asks three questions: should this file exist, what does it serve, and for whom.
Adding a file forces the question of who it is for, and the answer is the binding.
The tree stays organised as a side effect of being readable.

This is scoped to tracked files.
It never applies to a milestone, an issue or a pull request, each of which carries its own record and its own template.

**purba borrows the syntax entire, and git's own parser with it.**
`git check-attr` reads `.config/readers` when `core.attributesFile` names it, so nothing is reimplemented and nothing is approximated.
In this repository, `info/attributes` and a tracked `.gitattributes` outrank that file.
The check therefore runs git in an empty repository, where `.config/readers` is the only attribute file.

| the syntax gives | what it does here |
|---|---|
| override is per attribute | accumulation is the default, so a nearer line silent about an actor leaves the farther one standing |
| `**`, `*`, `?`, `[abc]`, escapes | a reader is assigned by what a file is, not only by where it sits |
| `-actor` | inheritance stops for one actor rather than for all of them |
| `[attr]name a b c` | a list of actors, written once and named by every line that uses it, as `contributor` is |

**Downside:**

- **The roster check reads the names off each line itself, and git does not.** A quoted pattern that holds a space splits in two. The check refuses the second half as a name off the roster, while git resolves the same line correctly.
- ⚠️ **The check cannot tell a file that correctly has no reader from one nobody considered.** A lock file may be read by nobody, and both states look identical.
- **A binding placed high and loosely silences every file beneath it**, and nothing detects a binding that is technically true and useless.
- **A refusal of a file that reaches no actor inherits no reader.** A branch that adds only that file reaches nobody.

## Confirmation

**The check reads `.config/readers` and fails the build on a tracked file that reaches no actor.**

It rejects any actor name, in a binding or in a macro, that is not on the roster.
It also rejects a macro named after an actor.

`mise run lint:readers` runs the check.
`mise run lint` runs that task.

Both halves were exercised against real trees before this record was written.

| measured | result |
|---|---|
| this repository as it then stood, against the bindings this record first held | 7 of 19 tracked files reached no actor |
| the prototype repository, 1,109 files, 288 directories, 8 levels deep | one pattern reaches 688 files, four reach 956 |

⚠️ The seven were root-level configuration and licence files.

⚠️ No check decides whether a location truly serves the actor it names.
That fails as friction, and friction is observed when a reader hits it.
