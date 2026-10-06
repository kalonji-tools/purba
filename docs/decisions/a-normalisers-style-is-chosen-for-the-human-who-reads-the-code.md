# A normaliser's style is chosen for the human who reads the code

## Context and Problem Statement

[A gate owns the mechanical standard](a-gate-owns-the-mechanical.md) gives a normaliser one location for its style.
It does not decide which style.

## Considered Options

- **Tabs, so that each reader chooses the width.** It is the only indentation that adapts to its reader, and GitHub carries a per-account setting for it. Rejected on consistency: nothing in the tree is tab-indented, and YAML forbids tabs outright, so the file a script's reader arrives from cannot match it.
- **Two spaces, stated once for every language that admits the same value.** Chosen.

## Decision Outcome

**A normaliser's style is chosen for the human who reads the code**, because a machine reads any style the syntax admits.
Consistency across languages is part of that legibility: a reader should not change indentation systems at a file boundary.
purba therefore indents with two spaces, stated once for every language that admits the same value.

⚠️ **A language whose own rules make a different value load-bearing is stated per language, with the constraint as the reason.**
Markdown is that case: a table row is one line by its syntax, so no line length is stated for it.

**Downside:**

- **A reader cannot choose the width.** Tabs are the only indentation that adapts to its reader.

## Confirmation

`shfmt` and `editorconfig-checker` read the style from `.editorconfig`.
Under a style that demands tabs, `shfmt` rewrites every shell script and `editorconfig-checker` refuses the tree.
