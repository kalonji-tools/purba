# A gate owns the mechanical standard

## Context and Problem Statement

purba welcomes a contribution written by an agent.
Nobody can assume an agent read a convention, and a reviewer who states one by hand states it again next time.

So a convention no tool enforces is not a convention.
It is a review comment, rewritten forever, by the reader whose attention is scarcest.
That reader's work is a judgement of whether a change solves its problem, and everything mechanical competes with it.

[What does always-strict mean, and which checks can exist under it?](https://github.com/kalonji-tools/purba/issues/32) answered the product half: purba is strict for the suites it runs, and strict is not a dial.
It also drew the line this record needs: a defensible house rule for purba's own tests is a strong claim to make about someone else's.
This record answers the house half, which nothing answered: how purba configures the tools that read purba.

## Considered Options

Three, weighed against the published configuration of eighteen projects and against purba's own tree.

- **The tool's default severity.** Sixteen of the eighteen run their linter at its default, and four enable more. Rejected because it is not the strictest available, not because it is unusual.
- **A severity chosen where the tree is already clean.** Rejected. One of the eighteen documents the practice, and its reason is a backlog too large to repair now. purba's was four findings against nineteen lines of Rust. ⚠️ **The practice also fails on its own terms.** A floor at `warning` keeps `SC1090`, which is merely noisy, and drops `SC2102`, which is the class that put an unquoted glob into a workflow.
- **The strictest setting the tool offers, with every exclusion named.** Chosen.

## Decision Outcome

**A gate owns every standard a reviewer would otherwise state by hand.**
The rule takes two clauses, because a formatter has no severity to set.

A tool that detects runs at the strictest setting it offers, with every optional check enabled.
A check is dropped only where it is named together with the reason it is dropped, at the location a reader meets the code.

A tool that normalises has one location for its style, and tolerates no deviation from it.
[A normaliser's style is chosen for the human who reads the code](a-normalisers-style-is-chosen-for-the-human-who-reads-the-code.md) says which style.

**A standard nothing can fix is still stated, and then it is only checked.**
Line length is that standard, because no formatter wraps a line.
Where a fixer exists it writes, and a checker is not put in its place.
`shellcheck` only checks, because one of its fixes changes behaviour.
`typos` writes in every file except a generated one.
The Downside below names what that costs.

[A standard a gate cannot decide does not become a gate](a-standard-a-gate-cannot-decide-does-not-become-a-gate.md) holds the converse.

⚠️ **A setting chosen because the tree already passes there is refused.**
It reports the tree's current state as though that were the standard, and the two are indistinguishable afterwards.

**A configuration file is owed only where purba deviates from a tool's default.**
[purba carries no tool manager beside mise, and no compiler of its own](purba-carries-no-tool-manager-beside-mise-and-no-compiler-of-its-own.md) pins each one, so an accepted default already has a single location: the pinned tool.
Writing a default into a file makes a second copy, and that copy drifts silently.
Absence is ambiguous, so this record names each deviation: `cargo fmt` has none, and `typos` has only [the words the glossary avoids](the-glossary-stays-open-and-a-gate-rewrites-each-word-it-avoids.md).
`shfmt` does deviate, so the style is written down, and `.editorconfig` is where it lives because `shfmt` and every editor both read it.
⚠️ **A style flag would take that away:** `shfmt` ignores `.editorconfig` the moment one is passed, so the hook passes none.

Where an owed file lives is a standard no gate holds, and [configuration takes the first location its tool reads](configuration-takes-the-first-location-its-tool-reads.md) says why.

Every lint level of the crate lives in `Cargo.toml`, which a task, a bare `cargo clippy` and an editor all read.
A cargo script carries its own, in the manifest inside it.
A flag on a command line reaches only that command, so `.config/tasks.toml` carries no lint flag.

**Downside:**

- **The strictest setting of a tool purba does not use is not derivable from this record.** Every adoption costs its own measurement.
- **`typos` rewrites a misspelling made on purpose, in code as well as in Markdown.** A test that misspells a name to prove the refusal holds one. The commit stops, so the writer sees each rewrite in the diff. To keep one, a writer adds an entry to `[tool.typos]` in `pyproject.toml`.
- **An exclusion is somewhere a later contributor can widen quietly.** The reason above it is the only thing that makes widening visible.
- **An exclusion is not always as narrow as its line.**
  Inside a YAML block scalar none can be. An indented one is posted as part of whatever the block writes, and one at the first column ends the block.
  The tree holds no such exclusion, because the shell that needed one moved into a file where a line-scoped directive works.
- ⚠️ **What is generated is named in files that cannot see each other.**
  `.gitattributes` marks it for review, the style file excludes it from the standard, and `prek.toml` excludes one that `typos` would otherwise read. A generated file added to one and not the others gains a check it will fail, or a fix nobody wrote.

## Confirmation

Each gate is run green and red, because a check that passes on a clean tree and refuses nothing is a suggestion.

| gate | what it refuses |
|---|---|
| `clippy` | `cargo clippy --all-targets`, with no flag on the command line: a `pub fn` returning a value draws `must_use_candidate`, and the exit is non-zero |
| `rustdoc` | a broken intra-doc link in the crate-level block draws `unresolved link`, and the exit is non-zero |
| `cargo` | re-declaring a crate nothing calls draws `unused dependency`, which `-D warnings` never reached |

The normalising clause was exercised on purba's shell against the pinned tools, measured in [Should shellcheck and shfmt gate purba's shell, and at what severity and style?](https://github.com/kalonji-tools/purba/issues/186).

| gate | green | red |
|---|---|---|
| `shellcheck` at `enable=all` | the repaired tree reports nothing | a bare `$var` reinstated anywhere draws `SC2250` while `.shellcheckrc` is present, and nothing without it. Each suppression removed restores its own finding |
| `shfmt` reading `.editorconfig` | the tree needs no rewrite | a style demanding tabs rewrites every shell script |
| `editorconfig-checker` | the tree reports nothing | every exclusion removed restores three refusals, one per excluded line, and a style demanding tabs draws seventy-two |

⚠️ **One of the linter's own fixes was measured changing behaviour**, which is why `shfmt` alone writes the shell.
`[[ ]]` evaluates `5+5` arithmetically where `[` refuses a value that is not an integer.
A guard on an issue number would then stop refusing one.

`typos` ran green and red through purba's own hooks, in [Let typos write its fix in every file except a generated one](https://github.com/kalonji-tools/purba/issues/251).

| gate | green | red |
|---|---|---|
| `typos --write-changes` | the tree needs no rewrite | it rewrites a misspelling in a `.md` or a `.sh` file and stops the commit |
| its exclusion of `CHANGELOG.md` | a misspelling in `CHANGELOG.md` stays as written | with the exclusion removed, `typos` rewrites it |

`Cargo.lock` and `.config/mise.lock` need no exclusion.
By default `typos` sets `check-file = false` for its `lock` file type.
A misspelling in either stayed as written.

Where a person fixed a real typo, `typos` wrote the same fix in 57 of 60 lines.
⚠️ **Some of its fixes in code change behaviour.**
On three trees that already spell-check, each of its 19 rewrites in code files was wrong.
Twelve of them renamed a name that a test misspells on purpose.

`[tool.typos]` in `pyproject.toml` ran green and red through the same hook.

| gate | green | red |
|---|---|---|
| `typos` reading `[tool.typos]` | the tree needs no rewrite | a planted `ticket` becomes `issue` and a planted `tickets` becomes `issues`, and the commit stops |
| its two `extend-ignore-re` patterns | the `_Avoid_` line and a `ticket` quoted in a code span stay as written | with the patterns removed, `typos` rewrites every mention, and `CONTEXT.md` reads `_Avoid_: issue` |

`cargo fmt` ran green and red through the same hooks, in [Let the Rust format hook write its fix](https://github.com/kalonji-tools/purba/issues/252).

| gate | green | red |
|---|---|---|
| `cargo fmt` | the tree needs no rewrite | it formats a misformatted `.rs` file and stops the commit |

`lint:rust` and `lint:shell` refuse unformatted code, because CI cannot write.

`lint:cargo-scripts` and the hook that formats a cargo script ran green and red on `scripts/check-prose/check-prose.rs`.

| gate | green | red |
|---|---|---|
| `lint:cargo-scripts` | the tree reports nothing | a planted unused variable exits 1, and a misformatted line exits 123 |
| the `rustfmt-cargo-scripts` hook | the tree needs no rewrite | it formats a misformatted cargo script and stops the commit |
