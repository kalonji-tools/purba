# The prek prototype

Everything [#41](https://github.com/kalonji-tools/purba/issues/41) claims was
measured here, on 2026-09-22, with mise 2026.8.6 and prek 0.5.3.

This branch is never merged. `prek.toml`, `mise.toml` and `mise.lock` are the
candidate files; this directory holds what proves them.

| script | what it answers |
|---|---|
| `minenv.sh` | does a hook run where a commit is started from an editor? |
| `corpus.sh` | what does the commit-message gate refuse, on purba's own history? |

## A gate fits a rule that is discrete, measurable and finite

That test decided more here than any other, and it removed a check this project
had already graded as buildable. Each rule below is answered against it.

## The hook entry is the whole question

A commit started from an editor's button gets `git` and whatever the editor's
own environment holds. It does not get a shell that activated mise, so it does
not get `cargo`. mise records this as
[discussion 6830](https://github.com/jdx/mise/discussions/6830).

Both candidate entries were written as hooks and run under `minenv.sh`, which
puts `git`, `mise` and `prek` on `PATH` and nothing else.

| entry | result |
|---|---|
| `cargo fmt --check` | ❌ **exit 2**, `Failed to run hook ... No such file or directory` |
| `mise run fmt:check` | ✅ **Passed** |

The first number is the bad one. prek exits **2**, which is prek reporting that
it could not run the hook, not the hook reporting a verdict. A gate that cannot
run is not a gate that passed, and nothing in the output says which happened
unless somebody reads the wording.

`mise x -- typos` passes the same test, so the rule generalises: **a hook that
needs one of purba's tools reaches it through mise, never through `PATH`.**

## The commit-message gate

[A commit message outlives its review](../docs/decisions/a-commit-outlives-its-review.md)
ends its Confirmation with *"Every decidable row is unchecked. No ticket owns
any of them."* prek's `require-pattern` and `deny-pattern` builtins carry three
of those rows with no script. A fourth was built, measured and removed.

| rule | discrete | measurable | finite | built |
|---|---|---|---|---|
| conventional type, optional scope | a closed list of 11 | exact | the list is owned here | ✅ |
| lower case after the colon | a character class | exact | one position | ✅ |
| the issue number in parentheses at the end | a pattern | exact | one position | ✅ |
| subject at most 72 characters | a count | exact | one number | ✅ |
| no full stop before the reference | `.` `!` `?` | exact | three characters | ✅ |
| **imperative mood, by a word list** | yes | yes | ⚠️ **no** | ❌ |
| **`Assisted-by:`, if a machine helped** | the trailer, yes | yes | yes | ❌ — ⚠️ **the antecedent is not in the message** |

### `corpus.sh`: what it refuses on purba's own history

```
51 accepted, 3 refused
refused  75c  docs: name mise as a prerequisite, not as the thing that follows them (#94)
refused  76c  docs(decisions): stop .gitattributes naming a file that does not exist (#94)
refused  74c  docs(decisions): write the outcome, and give a stem its prerequisite (#94)
```

**Three real violations, no false refusals, across the project's whole history.**
The record's own budget is 72 characters and three subjects went over it. One of
them was caught by a hand audit on [#117](https://github.com/kalonji-tools/purba/pull/117)
and the other two were never caught at all.

### Why the mood check was removed

The record grades this row *"yes, by a word-list test"*. **The measurement
falsifies that grading.**

A word list is finite. The rule it stands for is not: English verbs are an open
class, so the list can only refuse the words somebody thought of.

Its size was measured against the verbs purba already uses. The 54 subjects on
`main` open with **28 distinct verbs**. Each was written back as a past-tense
and a third-person subject — 56 messages that every one of these rules is meant
to refuse — and run through the gate.

| | |
|---|---|
| refused | **4** |
| slipped through | **52** |

`named`, `recorded`, `gave`, `dropped`, `corrected`, `stated`, `stood`, `said`,
`bound`, `signed`, `chose`, `froze` and forty more all pass. The list also
caught nothing at all on the real corpus, so it was pure risk against no
measured yield.

A detector good enough to suggest is not good enough to gate. This is the rule
`oxitest#2205` states and [#21](https://github.com/kalonji-tools/purba/issues/21)
adopts as the fourth admission question, and it removes a row the record says is
decidable.

### Why `Assisted-by:` is not gated either

⚠️ **The trailer is optional, and a gate cannot tell when it is owed.**

`Assisted-by:` is a disclosure, written only when a machine helped. A commit
nobody's tooling touched correctly carries none, so **absence is not a defect**
and a hook that required the trailer would refuse a legitimate unaided commit.

The rule's consequent is discrete, measurable and finite. Its **antecedent is
not in the message at all**. Whether a machine helped is a statement only the
committer can make, on the same ground that
[liability is recorded from the act that makes it true](../docs/decisions/liability-is-recorded-from-the-act-that-makes-it-true.md)
gives for `Signed-off-by:`, and nothing in `prek.toml` can read it.

The history measures the antecedent rather than the rule:

| | |
|---|---|
| commits on `main` authored by `snregales-agent` | 54 of 54 |
| of those, carrying `Assisted-by:` | **54 — 100%** |

⚠️ **That is not 100% compliance with the rule.** It is 100% of a population
that happens to be entirely agent-assisted, which is what a repository with no
unaided commit yet looks like. The first human commit would move both numbers
and break nothing.

### Hand-written shapes

| message | verdict |
|---|---|
| `feat: add a thing (#41)` | ✅ |
| `feat(bridge)!: drop the old wire (#41)` | ✅ |
| `feat: add a thing (#1234)` | ✅ |
| `feat: add a thing, e.g. a widget (#41)` | ✅ abbreviation is not a terminator |
| `chore: move the floor to 2026.8.6 (#41)` | ✅ |
| `docs: name .gitattributes as the owner (#41)` | ✅ |
| `fixup!`, `squash!`, `amend!` | ✅ allowed explicitly |
| a subject followed by a body and trailers | ✅ |
| a subject followed by git's `#` comment block | ✅ |
| `feat: add a thing` | ❌ no reference |
| `feat: add (#41) a thing` | ❌ reference not at the end |
| `feat: Add a thing (#41)` | ❌ upper case after the colon |
| `feat: add a thing. (#41)` · `...!` · `...?` | ❌ terminator before the reference |
| `wip: add a thing (#41)` · `Feat: ...` · `feat:add ...` | ❌ not a conventional subject |
| `Added the prek config` | ❌ not a conventional subject |
| `Revert "feat: add a thing (#41)"` | ❌ refused, deliberately |
| `Merge branch main into x` | ❌ refused, deliberately |

### The two deliberate refusals

git's own default subjects for a revert and a merge cannot match a conventional
subject. The set of them is finite, so a gate fits either answer, and the
history decides which.

| | across **every ref** in this repository |
|---|---|
| merge commits | **0** |
| subjects beginning `Revert "` or `Merge ` | **0** |

`main` is rebase-only, and Conventional Commits already has a `revert` type, so
`revert: undo the thing (#41)` is the subject a changelog wants. Refusing both
costs nothing that has ever happened here.

⚠️ **`fixup!` support is the one row with no measurement behind it.** No
`fixup!` subject appears in this repository's history or reflog, and none can:
autosquash consumes them before they reach `main`. They are allowed on the
argument that a commit which cannot reach the changelog should not be judged as
a changelog entry — an argument, not a number.

## Traps, each one hit here

1. ⚠️ **`prek install` does not install the `commit-msg` shim.** A bare
   `prek install` writes `pre-commit` and nothing else, so every commit-message
   hook silently never fires. `default_install_hook_types` in `prek.toml` fixes
   it at the config rather than at the command, which is where it has to be —
   nobody re-reads an install command.
2. ⚠️ **A hook with no `stages` key runs at every stage.** `typos`, `cargo fmt` and
   `no-commit-to-branch` each ran a second time during `commit-msg` before they
   were scoped.
3. ⚠️ **Git hooks live in the common directory, so they are installed once per
   clone and not once per worktree.** `git rev-parse --git-path hooks` resolves
   to `purba/.git/hooks` from inside every worktree.
4. ⚠️ **That sharing bites during this ticket's own life.** With the shims
   installed, a commit on any branch that predates `prek.toml` is refused
   outright: *"No `prek.toml` ... found"*, exit 1.
5. ⚠️ **prek refuses to run at all when `prek.toml` is unstaged**, and exits
   **2** to say so. `corpus.sh` read that as 54 refusals before it learned to
   separate a verdict from a failure to reach one.
6. ⚠️ **A `repo` block takes `hooks = [...]` or `[[repos.hooks]]`, never both.**
   Mixing them is a TOML duplicate-key error.

## The spell checker: codespell cannot be admitted, typos can

[#41](https://github.com/kalonji-tools/purba/issues/41) names `codespell`.
[mise names every tool version](../docs/decisions/mise-names-every-tool-version.md)
is what decides whether a tool can be admitted, and all three of codespell's
routes were run.

| route | result |
|---|---|
| the mise registry | **codespell is not in it.** `cspell` and `typos` are |
| `ubi:codespell-project/codespell` | ❌ **does not install** — *"could not find a release asset"*. codespell publishes no binaries. The `ubi` backend is also deprecated and removed in mise 2027.1.0 |
| `pipx:codespell` | ✅ installs, **but only after `uv` or `pipx` is admitted too** — *"pipx is required ... but was not found"* |

So codespell is reachable. It costs two tools rather than one, and the second
of them reads nothing in purba's tree, which is the admission rule
[#39](https://github.com/kalonji-tools/purba/issues/39) settled.

What the lockfile then records:

| tool | backend | platform rows | checksums |
|---|---|---|---|
| `codespell` | `pipx:codespell` | **0** | **0** |
| `codespell` | `ubi:...` | **0** | **0** |
| `typos` | `aqua:crate-ci/typos` | **7** | **7** |

codespell locks a version string and nothing else — no URL, no checksum, no
attestation. Every one of its seven platform rows is skipped.

### Both were then installed and run

Neither tool is better at finding a typo. Both found all five seeded prose
misspellings and all six seeded identifier misspellings, and both were clean on
purba's real tree and on `mise.lock`'s sha256 strings.

They separate on three things that are not about spelling.

| | codespell | typos |
|---|---|---|
| honours `.gitignore` | ❌ **no** — reported three typos inside `target/` | ✅ yes |
| run with no arguments | scans the working directory, build output included | scans the repository |
| whole tree, three runs | 0.35 s *(given an explicit file list)* | **0.15 s** |
| false positives on the identifier fixture | 3 of 4 short identifiers | 3 of 4, a different three |

⚠️ **A spell checker needs an escape hatch, and the first thing that needed one
was this file.** Both tools refuse the table above when it quotes the
identifiers verbatim. typos reads `_typos.toml`, and
`[default.extend-words]` suppresses a word while leaving every real
misspelling in the same file refused. It has no inline directive:
`spellchecker:off` is not honoured by 1.50.2. purba needs no such file today,
because the tree is clean without one.

The `.gitignore` row is the one that matters for a hook. `target/` is a Rust
build directory, and a checker that walks into it is both slow and wrong.

**typos is chosen.** It is one tool rather than two, it locks with checksums the
way every other purba tool does, it is the only one of the two in mise's
registry, and it is a spell checker written for source code rather than for
prose. The cost is a different dictionary and an issue body that no longer names
the tool the file uses.

## What the tree does not contain

`prek.toml` names no Python hook. purba has **zero** `.py` files, and
[#39](https://github.com/kalonji-tools/purba/issues/39) settled the admission
rule as *a tool enters the environment when the thing it reads is in the tree.*
`ruff`, `ruff-format` and `ty` have nothing to read here and are not admitted.

It also names no `pre-push` stage. [#21](https://github.com/kalonji-tools/purba/issues/21)
calls a `pre-push` stage that never runs at pre-push a naming trap, and purba's
expensive gates already hang off worktrunk's `pre-merge`, at
`.config/wt.toml`.

## One defect found on `main`

`docs/decisions/mise-names-every-tool-version.md` ends with two newlines, which
`end-of-file-fixer` refuses. It is the only file in the tree that does, and it
is the reason `prek run --all-files` does not pass on `main` today.
