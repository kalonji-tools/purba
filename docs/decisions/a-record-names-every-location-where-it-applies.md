# A record names every location where it applies

## Context and Problem Statement

A reader who checks a change against the records must first find the records that apply to it.
A title names a decision, and not the files that the decision binds.
A file can link the record that binds it, and many files do.

Five pull requests broke a rule that a record stated, and their audits found each breach.

| what named the record | breaches |
|---|---:|
| a link in a changed file, where the pull request did not change the record | 4 |
| of those, found by no other source | 1 |
| the pull request changed the record itself | 6 |
| `AGENTS.md` | 7 |
| nothing | 9 |

A breach that two sources found counts in both rows.
Each of the nine broke a record that binds a kind of text, such as prose, a comment or a commit body.
A link in a file cannot name that, and a file that a change adds carries no link at the base.

## Considered Options

- **A link in each file only.** Rejected. It misses each rule about a kind of text, and it misses a file that the change adds.
- **One map in `.config/records`.** Rejected. It is an index, and a change to it escapes the code owner.
- **A line in each record, read with the links in files.** Chosen. A pattern reaches a file that a change adds, and a word reaches a location with no path. A change to a reach is a change to a record, so [significance](../../CONTEXT.md#significance) routes it to the code owner.

## Decision Outcome

**Reach:** [unpublished comment](../../CONTEXT.md#unpublished-comment) `docs/decisions/*.md` `/CONTEXT.md` `scripts/check-records.sh` `scripts/check-prose/check-prose.rs` `scripts/test/check-records.bats` `/AGENTS.md`

**Each record opens its Decision Outcome with its [reach](../../CONTEXT.md#reach).**

| part | how it is written |
|---|---|
| the paragraph | `**Reach:**`, then each location |
| a path | a gitattributes pattern in a code span, matched by `git check-attr` in an empty repository |
| a location with no path | its glossary word, linked to its `CONTEXT.md` entry. Any glossary word for a location is allowed |
| `**` | allowed |
| a location that no pull request changes | not in a reach, because no change reaches it |

A ruleset, a repository setting and a GitHub App are such locations.

A reader of the reaches reads them at the base of the change.
It also reads each record that the change adds or changes, at the head.
At the head alone, the author's own edit chooses which records the reader reads.

The links in files stay, and a link names its record as a reach does.
Nothing reports a tracked file that no record reaches, because not every file has a rule.
A pattern that matches no tracked file is listed and never refused, because a record can come before its files.

**Downside:**

- **A reach can miss a location.** No command checks that a reach is complete, so the code owner reads each one.
- **Some patterns match no file on purpose.** A reach can name a file that exists only when a tool needs it, and `lint:records` lists that pattern on every run.
- **A renamed path leaves a pattern that matches nothing.** `lint:records` lists the pattern and refuses nothing, so the listing is a prompt that nothing enforces.

## Confirmation

`mise run lint:records` refuses a Decision Outcome that does not open with a `**Reach:**` paragraph, or one that holds anything but locations.
It lists each pattern that matches no tracked file, and refuses none of them.
Its prose rules do not read a reach, so a reach of any length passes them.
`scripts/test/check-records.bats` builds each case, and `mise run test:shell` runs it.
[A decision record is rewritten, not amended](a-record-is-rewritten-not-amended.md) lists these checks with the other checks of `lint:records`.

No command reads a reach to name the records a change must satisfy yet. [list:records names the records a change must satisfy](https://github.com/kalonji-tools/purba/issues/401) builds it.
The breaches above were counted on [Would the links from files to records have named the record each past audit found broken?](https://github.com/kalonji-tools/purba/issues/374).
