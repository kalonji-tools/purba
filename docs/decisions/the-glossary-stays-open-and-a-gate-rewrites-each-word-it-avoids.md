# The glossary stays open, and a gate rewrites each word it avoids

## Context and Problem Statement

[A term belongs to the glossary](a-term-belongs-to-the-glossary.md) gives each purba term one entry in `CONTEXT.md`.
An entry names a second word for the same thing on an `_Avoid_` line.
That record does not say whether a word missing from the glossary may be used.
No command reads an `_Avoid_` line.

| measured 2026-10-04, with `main` at `11222de` | count |
|---|---:|
| `_Avoid_` lines in `CONTEXT.md` | 1, for `ticket` |
| distinct word forms in the records, `CONTEXT.md`, `AGENTS.md` and `CONTRIBUTING.md` | 2,513 |
| uses of `ticket` and `tickets` in tracked files, each meaning an issue | 26 |
| comments on this repository that use `ticket`, in the two days before `AGENTS.md` named `issue` | 14 of 43 |
| the same, from that change to 2026-10-04 | 1 of 98 |

## Considered Options

- **Close the glossary.** Rejected. Over every word, it needs an approved list of every word form above. [purba borrows from Simplified Technical English rather than adopting it](purba-borrows-from-simplified-technical-english-rather-than-adopting-it.md) excludes the only ready-made list. Over purba's own words, no test separates a purba term from a general one.
- **Leave the `_Avoid_` line to `AGENTS.md`.** Rejected. A contributor who never reads `AGENTS.md` writes the avoided word. Nobody sees it.
- **Keep the glossary open, and let `typos` rewrite each word it avoids.** Chosen. The hook that already writes the fix for a misspelling writes this one.

## Decision Outcome

**The glossary stays open.**
A word missing from `CONTEXT.md` may be used.
No command holds a word on it to one sense.

**A gate rewrites each word an `_Avoid_` line names.**
`_typos.toml` names each form of the word and the word that replaces it.
`typos --write-changes` then rewrites it in every tracked file.
`typos` matches a whole word, so a plural is a form of its own.
Two patterns spare a mention: the `_Avoid_` line, and a code span that quotes the word.

**The word stays avoided outside a tracked file.**
An issue, a comment, a pull request and a commit message are outside the gate.
[`AGENTS.md`](../../AGENTS.md) tells an agent to avoid the word there.

**Downside:**

- **`CONTEXT.md` and `_typos.toml` cannot see each other.** An `_Avoid_` line added without its forms in `_typos.toml` goes unrefused. Nothing reports it.
- **A rewrite can break the sentence around it.** An article written before `ticket` stays `a`, so the result reads `a issue`. The commit stops, so the writer reads the diff. No gate catches it after that.
- **`typos` rewrites a mention outside a code span.** The two patterns spare nothing else.
- **The gate reaches tracked files only.** An issue, a comment, a pull request and a commit message rely on `AGENTS.md`. A contributor who never reads it writes the avoided word there.
- **A second sense of a glossary word passes.** `Record` is a noun in `CONTEXT.md` and a verb in [purba records delegation and does not prevent it](purba-records-delegation-and-does-not-prevent-it.md). No command tells the two apart.

## Confirmation

`typos` runs in the `pre-commit` hook, and in `Quality` through `prek run --all-files`.
[A gate owns the mechanical standard](a-gate-owns-the-mechanical.md) records its green run and its red run.

No command checks that each `_Avoid_` line in `CONTEXT.md` has its forms in `_typos.toml`.
`AGENTS.md` tells an agent that adds an `_Avoid_` line to add its forms there.
