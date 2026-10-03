# The markdown tooling prototype

Everything below answers the three questions that
[#219](https://github.com/kalonji-tools/purba/issues/219) left open. It was
measured on 2026-10-03, against `main` at `df08873`, with mise 2026.8.6.

This branch is never merged. Each script reproduces one table, and each `.out`
file is that script's last run.

No markdown linter earns a place in purba. The tree already refuses the one
structural defect purba shipped in a record. The second defect, #249, occurred
once in 107 commits, and no fixer repairs either one. `typos -w` writes the
correct fix for a real typo, but it rewrites every false positive as well. In
code, the false positives include deliberate misspellings that a test framework
writes on purpose, so `-w` belongs on Markdown alone. Harper found no error in
purba's Markdown, and no person writes that Markdown in an editor.

## 1. Which markdown linter, if any

### The candidates

From `maintenance.sh`. A release date and a commit count, never a star count.

| repository | language | latest release | released | commits, 90 days | in this prototype |
|---|---|---|---|---:|---|
| DavidAnson/markdownlint-cli2 | JavaScript | v0.23.3 (tag) | 2026-09-20 | 95 | yes |
| rvben/rumdl | Rust | v0.2.78 | 2026-09-29 | 716 | yes |
| akiomik/mado | Rust | v0.3.2 | 2026-09-06 | 107 | yes, check only |
| jackdewinter/pymarkdown | Python | v0.9.40 | 2026-09-13 | 19 | yes |
| hukkin/mdformat | Python | 1.0.0 (tag) | 2025-10-16 | 0 | yes, formatter |
| prettier/prettier | JavaScript | 3.9.9 | 2026-09-23 | 432 | yes, formatter |
| dprint/dprint-plugin-markdown | Rust | 0.25.0 | 2026-10-02 | 95 | yes, formatter |
| biomejs/biome | Rust | 2.5.15 | 2026-09-30 | 669 | no: it ignores a `.md` path |
| ekropotin/quickmark | Rust | 1.1.0 | 2025-09-05 | 0 | no: unmaintained |
| remarkjs/remark-lint | JavaScript | 4.0.2 | 2025-04-10 | 0 | no: unmaintained |
| markdownlint/markdownlint | Ruby | v0.18.1 (tag) | 2026-08-09 | 10 | no: needs Ruby |

Vale is out because [#216](https://github.com/kalonji-tools/purba/issues/216)
refused it.

### The defects purba shipped

From `q1-fixtures.sh`. Each checker runs on the defect and on its fix. A
planted control proves each one ran.

| checker | control | #216: a cost run onto the line before | #249: lines of the template that lose a placeholder, of 8 | a blank last line |
|---|---:|---|---|---|
| markdownlint-cli2 | 4 | not caught | 7 (MD033) | MD012 |
| rumdl | 4 | not caught | 2 (MD033) | MD012 |
| mado | 4 | not caught | 7 (MD033) | not caught |
| pymarkdown | 3 | not caught | 7 (MD033) | MD012 |

⚠️ **`scripts/check-records.sh` already refuses the #216 shape.** It counts the
bold lead-ins in a Downside list against its items. On the defect it reports
`4 lead-ins over 3 costs`, and on the fix it is silent.

The blank last line is the `end-of-file-fixer` that `prek` already runs.

### Today's 33 files

From `q1-corpus.sh`, with line length off as `.editorconfig` already has it for
Markdown.

| checker | findings | true defects | the rest |
|---|---:|---:|---|
| markdownlint-cli2 | 394 | 8, all #249 | 372 table pipe spacing (MD060) · 4 issue templates that open with `##` on purpose, and `CLAUDE.md` · 5 fences with no language · 2 `**Done when**` · 2 `$` prompts |
| rumdl | 14 | 2, both #249 | the same classes, without the table rule |
| mado | 41 | 8, all #249 | 11 ordered-list numbering · 6 bold lead-ins read as headings · 4 first headings that are not level one · the same classes |
| pymarkdown | 37 | 8, all #249 | 12 heading style · 4 headings with no blank line around them · the same classes |

### What each fixer rewrites

Each fixer runs on a copy, and GitHub renders each file it changed, before and
after. Identical HTML means the reader sees no change.

| fixer | files | lines | rendered HTML |
|---|---:|---:|---|
| pymarkdown | 0 | 0 | unchanged |
| rumdl | 4 | +7/−7 | `text` added to 5 bare fences, 2 `$` prompts removed |
| markdownlint-cli2 | 25 | +78/−78 | `\|---\|` padded to `\| --- \|`, 2 `$` prompts removed |
| prettier | 25 | +450/−450 | **unchanged**: every table re-padded, `*x*` becomes `_x_` |
| dprint | 27 | +452/−450 | **unchanged**: the same |
| mdformat | 8 | +30/−36 | 🔴 **damage**: the four issue templates lose their front matter |

`mdformat` rewrites the front matter of every issue template into a rule and a
heading, so GitHub stops reading `name`, `about` and `labels`:

```diff
----
-name: Defect
-about: A task to do, because a claim in the tree is false
-labels: "wayfinder:task"
----
+______________________________________________________________________
+
+## name: Defect about: A task to do, because a claim in the tree is false labels: "wayfinder:task"
```

Its own HTML check passes, because CommonMark reads that front matter as a
heading. No fixer repairs #216 or #249.

### Does #249 recur?

From `q1-history.sh`: MD033 over the Markdown tree of all 107 commits on `main`
that touched Markdown.

Every finding is one of the template's placeholders, and every one was born in
the template's first commit. The class never occurred anywhere else.

### What each candidate costs to adopt

From `lock.sh`. Each candidate is added alone to a clone, locked with purba's
settings, and run through `mise run lint:manifests`.

| candidate | adds | locked with a checksum | download, linux-x64 | manifest gate |
|---|---|---|---:|---|
| mado | one binary | 7 of 7 platforms | 2 MB | pass |
| rumdl | one binary | 7 of 7 | 6 MB | pass |
| dprint | one binary, and a wasm plugin fetched from a URL at run time | 7 of 7 | 10 MB | pass |
| markdownlint-cli2 | `node`, and 87 npm packages | ⚠️ **`node` only.** The npm entry holds a version and nothing else | – | pass |
| pymarkdownlnt | `uv`, and 12 Python packages | ⚠️ **`uv` only.** The pipx entry holds a version and nothing else | 21 MB | pass |

⚠️ **`locked = true` does not pin what an npm or pipx tool depends on.** Both
installed under it, and `markdownlint-cli2` resolved 87 packages at install
time. On NixOS, mise also builds Node from source unless `MISE_NODE_COMPILE=0`,
and that build needs `make`.

### The answer, as alternatives

**Add no linter** (recommended). #216's shape is already refused. #249 is fixed
once, by hand. Nothing enters `mise.toml`.

**Gate inline HTML with mado.** One rule, check only:

```toml
# mise.toml
"github:akiomik/mado" = "0.3.2"
```

```toml
# tasks.toml
["lint:markdown"]
description = "Refuse a placeholder GitHub would render as nothing"
shell = "bash -c"
run = "mado check $(git ls-files '*.md' ':!:CHANGELOG.md')"
```

It needs a `mado.toml` that turns every rule off except MD033. It would have
refused one defect in 107 commits, and it misses line 88 of the template, where
an HTML block swallows the line below it.

**Format with prettier or dprint.** About 450 lines rewritten on day one, with no
change a reader sees. Every later edit to a table then re-pads the whole table,
so a one-cell change shows as a whole-table diff in review.

## 2. Where `typos -w` writes

From `q2-typos.sh`: three hook entries, each committed through purba's real
`prek.toml`.

| scenario | `typos` today | `-w` on every file | `-w` on Markdown only |
|---|---|---|---|
| a misspelling in `.md` | refused, file untouched | refused, **fix written**, the second commit passes | the same as every file |
| a misspelling in a shell comment | refused | refused, fix written | refused, file untouched |
| a misspelled function, defined in a committed file and called from the staged one | refused | refused, and **the call is renamed alone**: `receive_all: command not found` | refused, file untouched |
| a fix in the staged half of a partly staged file | refused | refused, and prek **rolls the fix back**, keeping the unstaged edit | the same |
| a word with two corrections (`wich`) | refused | refused, nothing written | the same |

⚠️ **An exclusion does not hold when prek names the file.** typos checks a path
it is given, even when its config excludes that path. `extend-exclude =
["CHANGELOG.md"]` still let `-w` rewrite `CHANGELOG.md`. With `--force-exclude`,
the file was left alone.

### How often `-w` writes the right thing

From `q2-history.sh`: every commit whose message says it fixes a typo or a
spelling. The removed lines are the typo, and the added lines are the fix a
person wrote.

| tree | lines a person fixed | `-w` wrote the same | `-w` wrote something else | `-w` left alone |
|---|---:|---:|---:|---:|
| click | 159 | 26 | 2 | 131 |
| loguru | 74 | 31 | 1 | 42 |

On a real typo, `-w` matches the person in 57 of 60 lines. Most of the lines it
leaves alone changed more than one word.

From `q2-where.sh`: what `-w` would rewrite in trees that already run a spell
checker. These are the findings that remain once the real typos are gone.

| tree | one correction | in prose | in a code comment | in code |
|---|---:|---:|---:|---:|
| oxitest | 23 | 6 | 2 | 15 |
| click | 3 | 1 | 1 | 1 |
| loguru | 7 | 7 | 0 | 0 |

Every one of the 33 is wrong to rewrite:

- **12 are deliberate misspellings in test data.** oxitest tests that the fixture
  `sotre` is not found, nine times. click tests that `--bount` suggests
  `--bound`. Two keys `expeced` are already marked `codespell:ignore`.
- **7 rewrite a correct word.** `-w` writes `ERRORed` as `ERRORRed`, `missable` as
  `miscible`, and part of the class name `ZshComplet`.
- **14 in prose are names.** `ratatui` becomes `ratatouille` inside a code span,
  `serde::ser` becomes `set`, and loguru's coloured test output gains words.
- oxitest also vendors `mermaid.min.js`, where `-w` would rename 361 minified
  identifiers.

purba is a test framework. Its suites will test a misspelled fixture name on
purpose, as oxitest's do.

### The answer, as alternatives

**`-w` on Markdown only** (recommended):

```toml
[[repos.hooks]]
id = "typos"
name = "typos -w"
language = "system"
entry = "mise x -- typos -w --force-exclude"
files = '\.md$'
stages = ["pre-commit"]

[[repos.hooks]]
id = "typos-check"
name = "typos"
language = "system"
entry = "mise x -- typos --force-exclude"
exclude = '\.md$'
stages = ["pre-commit"]
```

**`-w` on every file.** One line changes:
`entry = "mise x -- typos -w --force-exclude"`. A deliberate misspelling in a
test is then rewritten, and the writer must notice it in the diff before staging
it again.

## 3. Where `harper-ls` is declared

From `q3-harper.sh`, on today's 33 files, 32,529 words.

| dialect | lints | per 1,000 words |
|---|---:|---:|
| us | 658 | 20.2 |
| gb | 617 | 19.0 |

purba writes British spelling, so `us` also raises `judgement` 20 times,
`licence` and `behaviour`.

| what Harper raises, gb | lints | true |
|---|---:|---:|
| capitalisation: `purba` in lower case, sentence-case headings | 293 | none. Both are purba's own conventions |
| spelling: `purba`, `mise`, `maturin`, `cdylib` and 49 more | 223 | none. Typos already gates spelling |
| split and compound words: `ruleset`, `non-zero` | 56 | style |
| grammar: agreement, articles, missing words | 24 | **none.** "every one of them" → "everyone", "an rlib" → "a rlib", "whose new solver" → "whose knew solver" |
| style: Oxford comma and others | 21 | style |

From `q3-drafts.sh`, on text no reviewer read first.

| writer | words | lints | grammar lints that are true |
|---|---:|---:|---|
| the agent, 40 issue comments | 16,216 | 595 | none of 20. Seven are `ruleset` split in two, and the rest are false or a matter of style |
| the person, 108 comments and reviews | 1,714 | 185 | about 4 of 10: "a inline table", and a comma after an opening "thus" or "otherwise" |

In the same text, Harper raised no spelling or grammar lint on "load baring",
"whats" or "When it runs it correct".

⚠️ **Every one of the 107 commits that touched Markdown is by
`snregales-agent`.** The person writes in GitHub's web form, and an editor's
language server does not run there.

| | |
|---|---|
| `mise.toml`, beside `rust-analyzer` | `jdx/mise-action` installs every tool by default, so every CI job that uses it installs 13 MB that it never runs. It locks on 7 of 7 platforms |
| a tracked `.harper-dictionary.txt` | `harper-ls` reads a workspace dictionary by that name (from the binary's strings, not exercised) |
| the dialect | an editor setting. No tracked file sets it |

### The answer, as alternatives

**Drop Harper from #219** (recommended). It found nothing true in purba's
Markdown, and nobody writes that Markdown in an editor.

**Keep it on the person's own machine.** `mise use -g harper-ls`, with
`dialect = "British"` in the editor. Nothing enters the repository.

## What was never varied

- Harper's code actions were not driven through the language server. harper-cli
  shares the engine, and its suggestions are what a code action would apply.
- No fixer was run on #216's defect to see whether it repairs it. None flagged it.
- The typos arms ran on Linux only.

## Scripts

| script | answers |
|---|---|
| `maintenance.sh` | is each candidate maintained? |
| `q1-fixtures.sh` | which checker catches the defects purba shipped? |
| `q1-corpus.sh` | what does each checker report, and each fixer rewrite, today? |
| `q1-history.sh` | does #249's class recur? |
| `lock.sh` | what does each candidate cost `mise.lock`? |
| `q2-typos.sh` | what does each typos hook do to a commit? |
| `q2-where.sh` | what would `-w` rewrite in a tree that already spell-checks? |
| `q2-history.sh` | when a person fixed a typo, did `-w` write the same fix? |
| `q3-harper.sh` | what does Harper raise on purba's Markdown? |
| `q3-drafts.sh` | what does Harper raise on text nobody reviewed? |
