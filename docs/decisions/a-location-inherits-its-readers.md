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
- **Let a child override its parent, as GitHub's `CODEOWNERS` does.** Rejected. The last matching pattern wins there, so a child naming the architect would dismiss the technical writer. Override is correct for approval and wrong for reading.
- **Author a reader list for every location.** Rejected. A commit and a changelog hold no content of their own, so a second authored list can disagree with the thing it describes. [#28](https://github.com/kalonji-tools/purba/issues/28) settled the form: a check that detects drift is worse than a design where drift cannot happen.
- **Route by the stage of work rather than by the location.** Rejected. The pipeline is unsettled, and a record keyed to its stages would be rewritten on every revision.
- **Invent a binding syntax.** Rejected after a prototype. The smallest thing that works is directory prefixes, and prefixes cannot express `*.toml`, which in the prototype reached 956 of 1,109 files in four patterns.
- **Implement part of gitattributes syntax by hand.** Rejected. A file that looks like gitattributes while supporting less than gitattributes lies about itself, and the subset is invisible until someone writes a pattern it silently ignores.
- **Accumulate readers down the tree, written in gitattributes syntax and parsed by gitattributes' own parser.** Chosen.

## Decision Outcome

A location inherits its readers.

**Down the path tree.**
A directory names the actors it adds, and a file's readers are the union of every binding from the project root down to it.
A directory that binds nothing is transparent: `src/config/pyproject.toml` reads `src/`'s actors when `config/` binds none.

The bindings live in `.readers`, in gitattributes syntax:

```
[attr]contributor  architect coder toolsmith handler reviewer technical-writer
[attr]user         tester plugin-author packager integrator

README.md          stem
CLAUDE.md          handler contributor
.claude/**         handler
docs/agents/**     handler
.github/**         toolsmith
docs/**            technical-writer
docs/decisions/**  architect
src/**             coder plugin-author tester
```

The project root binds nothing.
A reviewer reaches every location by what its role is, and a role that spans everything is not a binding.
⚠️ Bind anything at the root and every file inherits it, which leaves the check below unable to fail.

**Along the derivation chain.**
A location built from another inherits that source's readers and is never authored twice.
A commit message and a pull request derive from the paths they touch.

**A tracked file that reaches no actor fails the build.**
The failure asks three questions rather than reporting a broken rule: should this file exist, what does it serve, and for whom.
Adding a file forces the question of who it is for, and the answer is the binding, so the tree stays organised as a side effect of being readable.

This is scoped to tracked files.
It never applies to a milestone, an issue or a pull request, each of which carries its own record and its own template.

**purba borrows the syntax entire, and never the machinery.**
Git reads attributes from four named places and `.readers` is not one of them, so git never sees this file.
`gix-attributes` parses an arbitrary buffer under an arbitrary filename, so nothing is reimplemented and nothing is approximated.

| the syntax gives | what it does here |
|---|---|
| override is per attribute | accumulation is the default, so a nearer line silent about an actor leaves the farther one standing |
| `**`, `*`, `?`, `[abc]`, escapes | a reader is assigned by what a file is, not only by where it sits |
| `-actor` | inheritance stops for one actor rather than for all of them |
| `[attr]name a b c` | where `contributor` and `user` live |

**Downside:** three costs, and the first is paid every month.

- **The parser costs 12 packages and a breaking release roughly monthly**, six in the last nine months, against a project that pins `ruff_*` exactly. They belong to the gate rather than to the product crate, so the wheel is untouched and the cost is a recurring upgrade. The record pays it rather than hand-write a subset.
- ⚠️ **The check cannot tell a file that correctly has no reader from one nobody has thought about.** A lock file may be read by nobody, and both states look identical.
- **A binding placed high and loosely silences every file beneath it**, and nothing detects a binding that is technically true and useless.

## Confirmation

**The check reads `.readers` and fails the build on a tracked file that reaches no actor.**

It rejects any actor name, in a binding or in a macro, that is not on the roster.

⚠️ It is not wired yet, and [Build the reader-binding gate](https://github.com/kalonji-tools/purba/issues/85) owns it.

Both halves were exercised against real trees before this record was written.

| measured | result |
|---|---|
| this repository, against the bindings above | 7 of 19 tracked files reach no actor |
| the prototype repository, 1,109 files, 288 directories, 8 levels deep | one pattern reaches 688 files, four reach 956 |

⚠️ The seven are root-level configuration and licence files.
**The first output of this decision is an edit to those bindings, not a build.**

⚠️ No check decides whether a location truly serves the actor it names.
That fails as friction, and friction is observed when a reader hits it: a stem that cannot reach a first success without asking, an architect that writes a record contradicting one it never found, a reviewer that cannot tell what the author discarded.
