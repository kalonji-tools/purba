# Prototype: which task runner purba carries

This branch is never merged. It exists so the numbers can be re-run.

It serves [#40](https://github.com/kalonji-tools/purba/issues/40). The decision
is the human's and is written on that ticket, not here.

## Why this ticket is open at all

#40 says "Write the justfile", and inherits that from
[#11](https://github.com/kalonji-tools/purba/issues/11), which sorted oxitest's
tooling into what copies and what is rebuilt.

⚠️ **#11 was decided on 2026-09-08, when the substrate was devenv.** It lists
`devenv.nix` and `devenv.yaml` among the files that copy wholesale. devenv ships
no task runner, so a separate one was not a choice anybody made — it was the
only option on the table.

[#94](https://github.com/kalonji-tools/purba/issues/94) removed devenv and
[#39](https://github.com/kalonji-tools/purba/issues/39) made mise the substrate
on 2026-09-22. **mise ships a task runner.** So #40's own admission test —
[#21](https://github.com/kalonji-tools/purba/issues/21)'s four questions, whose
second is *"does a builtin already exist?"* — now has a different answer to the
one available when #11 ran.

`just` and `justfile` appear nowhere in the tree: zero hits across `docs/`,
`README.md`, `CONTEXT.md`, `CONTRIBUTING.md` and `Cargo.toml`. No decision
record names either. This is an open question, not a settled one being reopened.

## Why this is measured rather than modelled

Both runners can express purba's recipes. Nothing separates them on a feature
list, and a feature list is what the author already believes written down twice.

What separates them is **where the toolchain comes from when the runner is
invoked**, and that differs per surface. The four surfaces below are the four
places a purba recipe is actually run, and each is measured on the real tree.

## The arms

| arm | what a developer types | where recipes live |
|---|---|---|
| **A — mise tasks alone** | `mise run check` | `tasks.toml`, included by `mise.toml` |
| **B — just alone** | `just check` | `justfile` |
| **C — both** | `just check` | `justfile` delegating to `mise run` |

Arm C is the arrangement people actually reach for, so it is measured rather
than dismissed: `just` stays the surface a developer types, and every recipe
shells to the mise task so the tool resolution is not lost.

In all three arms mise supplies the toolchain. `just` is named in `mise.toml`
in arms B and C, so no arm installs a tool by hand.

## The four surfaces

| surface | what is measured |
|---|---|
| **local** | can the runner resolve the pinned toolchain, and what must the developer have done first |
| **remote — CI** | how many actions bootstrap it, and does a failing gate fail the job |
| **git hooks** | does the runner resolve the toolchain inside a `prek` hook, where the environment is not the developer's shell |
| **editors** | can an editor discover and run the recipes, and read the pinned toolchain |

## The criterion

**`check` — every gate in one command — run on the bare scaffold.**

It is the command #40's Done-when names, it exercises every recipe worth having
on a tree with no product code, and it is the thing CI will eventually require.

## What this prototype predicts, written before the CI half ran

A prototype that cannot say no is decoration. These are the falsifiable claims.
The local and hook halves were measured first and are recorded below; claims
1–4 were written before `proto-40.yml` ran even once.

1. **CI is where the arms are closest to equal.** `jdx/mise-action` activates
   the environment for every later step, so `just` finds the toolchain there
   without any prefix. The gap measured locally and in hooks does not appear.
2. **Arm B fails on Windows**, because a justfile recipe runs under `sh` and a
   Windows runner does not carry one on `PATH` by default.
3. **No arm needs a second action.** `just` is named in `mise.toml`, so
   `mise-action` installs it and `extractions/setup-just` is never needed.
4. **A failing gate exits non-zero in all three arms.** If any arm reports zero
   on deliberately broken formatting, that arm cannot hold a required check.
5. ⚠️ **Arm C is the only arm that can disagree with itself**, because its
   recipes name `mise` by `PATH` lookup rather than by the lockfile. Already
   reproduced locally — see the local results.

## The confound this file exists to record

**A GitHub runner already ships cargo, rustc and a C compiler.** An arm can
appear to resolve a toolchain it never installed. `proto-40.yml`'s `inventory`
job records what a bare runner carries before any arm runs, so every later
result is read against what was already there.

A second confound is local. ⚠️ **The developer's machine runs mise 2026.8.6,
below purba's own `min_version` of 2026.9.7**, so `mise` on `PATH` refuses to
read `mise.toml` at all. Every local measurement here was taken with a
2026.9.12 binary invoked by absolute path. Where an arm failed because it
reached the system mise instead, that is recorded as an arm property only when
the arm is what chose the lookup.

## What is deliberately not varied

The tree is the bare cdylib scaffold with no product code. The recipe set is
the minimum #40 names — `build`, `fmt`, `check`, `test-rust`, `test-doc`,
`clean` — plus the two doc lints [#28](https://github.com/kalonji-tools/purba/issues/28)
settled. Nothing here decides which recipes purba keeps; that is #40's spec.

Nothing here tests the commit-grouping recipes #40's comment describes. They
run `git`, which every arm resolves identically, so they cannot separate the
arms.

---

# Results — local and hooks, measured 2026-09-22

## 1. Local: what the runner resolves, and what the developer must have done

The scaffold's own `check` commands all pass on the bare tree, so any
difference below is the runner's and not the tree's.

| developer state | arm A `mise run probe` | arm B `just probe` |
|---|---|---|
| nothing activated | ✅ `cargo 1.100.0-nightly (495c385d0)` | ❌ `just` is not even on `PATH` |
| `just` on `PATH`, mise not activated | ✅ | ❌ **exit 127**, `cargo: command not found` |
| `eval "$(mise activate bash)"` | ✅ | ✅ |

⚠️ **`just` is callable only once mise is activated, and its recipes resolve
the toolchain only once mise is activated.** `mise run` needs neither.

purba's `README.md` documents `mise install` and `maturin build`. **It never
tells a contributor to activate mise.** Arm B makes that instruction load-bearing.

## 2. The gates themselves, on the bare scaffold

Every command a `check` would run, measured through `mise exec`:

| command | exit | note |
|---|---|---|
| `cargo fmt --check` | 0 | |
| `cargo clippy --all-targets -- -D warnings` | 0 | ⚠️ see below |
| `cargo doc --no-deps` | 0 | |
| `cargo test --doc` | 0 | `0 tests` — honest, exactly as #36 predicts |
| `cargo test` | 0 | `0 tests` |

⚠️ **`check` passes with a warning `-D warnings` does not deny.** Nightly cargo
reports `unused dependency: ruff_python_parser` as a *manifest* warning, and
`-D warnings` is a rustc flag that does not reach it. The scaffold declares
four `ruff_*` dependencies that no code uses yet. **This is arm-independent and
belongs in #40's spec**, because "`just check` runs and passes" is satisfied by
a command that prints a warning on every run.

## 3. Git hooks — the surface that separates the arms

Measured with `prek` 0.5.3, both arms given **an identical environment** in
which `git`, `mise` and `just` are on `PATH` and **`cargo` is absent** — which
is what a commit launched from an editor gets.

| arm | hook entry | result |
|---|---|---|
| **A** | `mise run fmt:check` | ✅ **Passed** |
| **B** | `just fmt-check` | ❌ **exit 127**, `cargo: command not found` |

⚠️ **This is not a quirk of this machine.** mise's own issue tracker carries it
as [discussion #6830](https://github.com/jdx/mise/discussions/6830), *"Mise
tools not available in VSCode's git task"*, whose cause is that the editor
"doesn't load the full shell environment where mise activation occurs". The
workaround given upstream is to prefix hook commands with `mise x --`.

**Arm A is that prefix, structurally.** Arm B has to add it back by hand, in
every hook, or require that every contributor's editor launches from a profile
that activated mise.

## 4. Editors

Both runners are discoverable and both emit machine-readable output, so an
editor can list and run either.

| | arm A | arm B |
|---|---|---|
| machine-readable recipe list | `mise tasks ls --json` | `just --dump --dump-format json` |
| arguments shown in the plain listing | ❌ needs `mise tasks info` | ✅ `test-rust filter=""` |
| VS Code extension | [`hverlin.mise-vscode`](https://github.com/hverlin/mise-vscode/) — tasks, tools, `launch.json`, and it configures other extensions to use mise's tools | [`nefrob.vscode-just-syntax`](https://marketplace.visualstudio.com/items?itemName=nefrob.vscode-just-syntax) syntax, [`just-lsp`](https://github.com/terror/just-lsp) for completion, diagnostics and run-recipe code actions |
| language server | none; the extension supplies completion | `just-lsp` |
| JetBrains / Neovim / Emacs / Xcode | documented upstream at [mise IDE integration](https://mise.jdx.dev/ide-integration.html) | a Just plugin exists per editor |

⚠️ **This row is where arm B is genuinely ahead: `just-lsp` is a real language
server and mise has no equivalent.** `just --list` also shows parameters where
`mise tasks ls` does not.

⚠️ **But the editor's other job is finding the toolchain**, and that is the
same problem as §3. mise's own documentation is explicit that a shim in a login
profile does not configure "the extension host or every language server", and
that selecting an SDK path "does not load `[env]`". `hverlin.mise-vscode`
exists precisely to close that, and it closes it for arm A and arm B alike —
**but only arm A's recipes still work when it is absent.**

## 5. What `just` costs in the lockfile

| | before | after |
|---|---|---|
| `mise.lock` lines | 98 | **138** |
| tool blocks | 3 | **4** |

`just` 1.58.0 from `aqua:casey/just`, 8 platform entries, **each with a
sha256** — better provenance than `rust`, which delegates to rustup and records
no checksum at all.

## 6. Wall clock, warm

| arm | `check` |
|---|---|
| A | **0.2 s** — `depends` runs the five gates concurrently |
| B | 3 s — `just` runs dependencies in sequence |
| C | 3 s |

Not a reason to choose. Recorded because the difference is structural rather
than incidental: mise parallelises `depends` and `just` does not.

## 7. Arm C disagreed with itself, as predicted

Arm C's recipes call `mise run …`, which is a `PATH` lookup. On this machine
that resolved to the system mise 2026.8.6:

```
$ just check
mise ERROR mise version 2026.9.7 is required, but you are using 2026.8.6
error: recipe `check` failed on line 22 with exit code 1
```

With the correct mise first on `PATH` the same command exits 0 in 3 s.

⚠️ **Arm C reintroduces the exact failure the mise decision record was written
against** — *"The prototype purba succeeds had two answers and did not know
it."* Arm C has two runners and resolves one of them by `PATH`.

---

# Results — the feature battery, measured 2026-09-22

Run from `prototype/features/`. Both files express the same capabilities, so
every row below is a command that was executed rather than a documented claim.

| capability | `just` | mise tasks |
|---|---|---|
| a recipe in another language | ✅ shebang recipe | ✅ shebang in `run`, **and real files in `mise-tasks/`** |
| constants and variables | ✅ `crate := "purba"`, `+`, backticks | ✅ `[vars]` with `{{vars.crate}}` |
| exported environment | ✅ `export X := …` | ✅ `[env]` |
| a `.env` file | ✅ `set dotenv-load` | ✅ `[env] _.file` |
| conditional expression | ✅ `if … { } else { }` | ✅ Tera `{% if %}` |
| platform branch | ✅ `[linux]` / `[macos]` attributes | ✅ `{{os()}}`, `{{arch()}}` in the body |
| positional parameters | ✅ | ✅ `usage` spec |
| variadic parameters | ✅ `+args` | ✅ `var=#true` |
| **flags and options** | ❌ **positional only** — `--verbose` arrives as a positional string | ✅ **`--verbose`, `--jobs <n>`, with a generated `--help`** |
| dependency with arguments | ✅ `outer: (inner "x")` | ✅ `depends = ["inner x"]` |
| private recipe | ✅ `[private]` | ✅ `hide = true` |
| groups in the listing | ✅ `[group("gates")]` | ⚠️ none; `:` namespacing only |
| modules | ✅ `mod sub` | ⚠️ `includes`, and monorepo `--all` |
| **incremental skip** | ❌ | ✅ `sources` / `outputs` |
| **per-task tool version** | ❌ | ✅ `tools = { python = "3.11" }` ⚠️ does not auto-install a missing one |
| dependencies run | sequentially | concurrently |
| formatter | ✅ `just --fmt` | ✅ `mise fmt` |
| language server | ✅ `just-lsp` | ❌ |
| **resolves purba's toolchain unaided** | ❌ | ✅ |

⚠️ **On features this is close to a draw.** `just` has a nicer expression
language, real groups, real modules and a language server. mise has typed
command-line surfaces with generated help, build avoidance, per-task tool
pinning, and concurrent dependencies.

⚠️ **The one row that is not a preference is the last.** `just py` — a Python
shebang recipe — exits 127 here, and `mise run py` prints `python 3.11.16`.
The feature test reproduces §3's hook result by another route.

## When each one is the right answer

**Reach for `just` when the toolchain is not mise's.** If versions come from
nix, devenv, asdf, rustup or the system, then mise tasks would mean adopting
mise for tasks alone, which is the larger commitment. `just` is one static
binary that assumes nothing, and its recipes run for somebody who has never
heard of mise. Reach for it too when the task file is edited daily and its
ergonomics are the point — `just-lsp`, groups and modules are real gains that
mise has no answer to.

**Reach for mise tasks when mise already owns the toolchain.** Tasks then
inherit tool resolution for free, and the project keeps one file family, one
lockfile and one trust model. It is the right answer specifically when tasks
must run where the environment is not a developer's shell — git hooks, editor
commit buttons, a CI step that skipped activation — and when a task wants a
typed command line, build avoidance, or its own tool version.

**Reach for both only when `just` is already entrenched** and mise is being
introduced underneath it. Arm C is what that costs: every delegation is a
`PATH` lookup for `mise`, and §7 above is that seam failing. It buys nothing
that one runner does not already give.

**purba is the second case.** #39 made mise the substrate two days before #40
was picked up, and #41 is about to put these commands behind git hooks.

---

# Results — CI, measured 2026-09-22

Run [35735420631](https://github.com/kalonji-tools/purba/actions/runs/35735420631).
Bootstrap is `jdx/mise-action@v4.3.0` pinned to mise 2026.9.12, one action for
every arm.

| arm | ubuntu-latest | macos-latest | windows-latest |
|---|---|---|---|
| **A — mise tasks** | ✅ 48 s | ❌ 81 s | ✅ 137 s |
| **B — just** | ✅ 54 s | ❌ 71 s | ✅ 132 s |
| **C — both** | ✅ 86 s | ❌ 40 s | ✅ 116 s |

## The predictions, scored

| # | claim | verdict |
|---|---|---|
| 1 | CI is where the arms are closest to equal | ✅ **confirmed** — identical conclusions on all three operating systems |
| 2 | arm B fails on Windows | ❌ **FALSIFIED** — `just` is green on Windows. A `shell: bash` step gives the runner's Git Bash, and `just` finds the `sh` it wants |
| 3 | no arm needs a second action | ✅ **confirmed** — `just` is named in `mise.toml`, so `mise-action` installed it. `extractions/setup-just` was never needed |
| 4 | a failing gate exits non-zero in every arm | ✅ **confirmed** — on both green operating systems, in all three arms. The step fails the job if the gate returns zero, and no job failed that way |
| 5 | arm C can disagree with itself | ✅ **confirmed locally**, see §7 above. CI cannot reproduce it, because `mise-action` puts exactly one mise on `PATH` |

⚠️ **Prediction 2 was wrong and the arm it accused is fine.** Nothing about
Windows separates these runners.

## ⚠️ What CI found that has nothing to do with the choice

**`cargo test` fails on macOS, in all three arms, identically.**

```
dyld[3265]: Library not loaded: @rpath/libpython3.11.dylib
  Referenced from: target/debug/build/purba/…/out/purba-…
  Reason: tried: … '/usr/lib/libpython3.11.dylib' (no such file, not in dyld cache)
error: test failed, to rerun pass `--lib`
  process didn't exit successfully: … (signal: 6, SIGABRT)
```

`extension-module` is off for a plain `cargo test`, which is deliberate —
`Cargo.toml` says so, because the feature "omits libpython, which breaks
`cargo test --doc` and `cargo doc`". With the feature off the test binary
links libpython instead, and on macOS the dylib mise installed is not on the
binary's rpath. Linux and Windows both resolve it.

Note what still passed: **`cargo test --doc` is green on macOS.** It is only
the `--lib` test binary that aborts at load.

⚠️ **This means #40's Done-when — "`check` runs and passes" — is not currently
reachable on macOS, whichever runner is chosen.** It belongs in #40's spec and
it reaches [#42](https://github.com/kalonji-tools/purba/issues/42) as well,
because the test workflow will run the same binary on the same matrix.

The three candidate answers, none of them decided here: give the test binary an
rpath to mise's libpython; run `cargo test` with `extension-module` on and
accept that it covers less; or declare `test-rust` a Linux and Windows recipe
until [#42](https://github.com/kalonji-tools/purba/issues/42) settles the
matrix.

## The macOS abort has three fixes, and all three work

Measured on `macos-26-arm64` in run
[35737586961](https://github.com/kalonji-tools/purba/actions/runs/35737586961).
Each candidate ran on its own with `continue-on-error`, so a failing one could
not hide the others.

| candidate | result | what it changes |
|---|---|---|
| **1 — `DYLD_FALLBACK_LIBRARY_PATH` from `sysconfig`** | ✅ `test result: ok` | nothing that is compiled. One environment variable, read from `sysconfig.get_config_var("LIBDIR")` → `~/.local/share/mise/installs/python/3.11.16/lib` |
| 2 — `cargo test --features extension-module` | ✅ | ⚠️ what is tested. `Cargo.toml` turns this feature off on purpose, because it omits libpython and breaks `cargo doc` and `cargo test --doc` |
| 3 — `RUSTFLAGS="-C link-arg=-Wl,-rpath,$LIBDIR"` | ✅ | ⚠️ the link. It bakes one machine's path into the binary, and changing `RUSTFLAGS` forces a full rebuild |

**Candidate 1 is the one to take.** It is the only one that leaves both what is
compiled and how it is linked alone. `DYLD_*` is read on macOS only, so the
task sets it on every platform and needs no branch.
