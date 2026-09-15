# Prototype: which environment manager becomes purba's first-class path

This branch is never merged. It exists so the numbers can be re-run.

It serves [#94](https://github.com/kalonji-tools/purba/issues/94). The decision
is the human's and is written on that ticket, not here.

## Why this is measured rather than modelled

The question is how two real runtimes behave on three operating systems. A
model would encode what the author already believes and confirm it by
construction. Both halves below patch the real tree and measure it.

The decisive claims sit on opposite machines, so neither half decides alone:

| claim | only measurable on |
|---|---|
| mise cannot link, because NixOS carries no `cc` outside a shell | the developer's machine |
| devenv cannot serve Windows, and mise can | a GitHub runner |

## The arms

| arm | what supplies the toolchain | what supplies the C compiler |
|---|---|---|
| **devenv only** | devenv, through `rust-toolchain.toml` | devenv |
| **mise only** | mise, through `mise.toml` | the host, whatever it has |
| **mise plus devenv** | mise | devenv |

The devenv-only arm is the baseline and is measured first, because *this
changed* is only a measurement when there is a before.

It gets no Windows run. WSL2 is not usable on a Windows runner, and a job that
confirms a documented impossibility buys nothing. That arm fails on Windows by
documentation, and this file says so in those words rather than implying a run
happened.

## The criterion

**A wheel, built from a floating `nightly`.**

`maturin build` exercises the linker, libpython, the toolchain and maturin in
one command, and it is the artifact purba ships. Every arm on every operating
system records the same manifest:

- the resolved toolchain string
- whether `clippy`, `rustfmt`, `rust-analyzer` and `rust-src` are all present
- whether a C compiler was found, and which one
- whether a wheel exists
- cold setup wall-clock

## What this prototype predicts, written before it ran

A prototype that cannot say no is decoration. These are the falsifiable claims.

1. The mise-only arm fails to link on the developer's machine, because there is
   no `cc` on `PATH` there.
2. The devenv-only arm cannot run on Windows at all.
3. mise's lockfile carries no Windows entries until `mise lock --platform` is
   run for Windows.
4. On the same day, a floating `nightly` resolves to *different* toolchain
   strings under mise and under devenv.

## The confound this file exists to record

GitHub's runners ship a Rust toolchain and a C compiler already. An arm can
appear to succeed on tools it never installed. `proto-94.yml`'s baseline job
inventories a bare runner before any arm runs, so every later result is read
against what was already there.

## What is deliberately not varied

The tree is the bare cdylib scaffold with its 93 dependency crates and no
product code. Nothing here tests free-threaded Python, which is
[#63](https://github.com/kalonji-tools/purba/issues/63).

---

# Results, measured 2026-09-15

Run [34959419025](https://github.com/kalonji-tools/purba/actions/runs/34959419025)
is the baseline. Run
[34960016644](https://github.com/kalonji-tools/purba/actions/runs/34960016644)
is the arms. All eight jobs are green, which is itself a result rather than a
pass: read every row against the confound below.

## The criterion

| arm | NixOS, the developer's machine | ubuntu | macos | windows |
|---|---|---|---|---|
| devenv only | wheel. 38.8 s cold shell | wheel, 125 s | wheel, 271 s | **not run. Impossible by documentation** |
| mise only | ⚠️ **fails: `linker cc not found`** | wheel, 39 s | wheel, 45 s | **wheel, 103 s** |
| mise plus a nix system layer | wheel. 2.9 s | not distinguishable from the row above, see the confound | | |

Times are whole-job wall-clock, cold, including checkout and install.

## The predictions, graded

| # | prediction | verdict |
|---|---|---|
| 1 | mise only fails to link on the developer's machine | **confirmed.** `error: linker 'cc' not found`, on the first build script |
| 2 | devenv only cannot run on Windows | **not run.** Recorded from documentation, not measured |
| 3 | mise's lockfile carries no Windows entries until `mise lock --platform` runs | ⚠️ **falsified** |
| 4 | a floating nightly resolves differently under the two managers | **confirmed, same machine, same day** |

### Prediction 3 was wrong, and wrong in mise's favour

No `mise lock --platform` was run. `mise install` downloaded and recorded seven
platform triples on its own, `x86_64-pc-windows-msvc` among them, each with a
checksum and `provenance = "github-attestations"`.

### Prediction 4, measured on one machine within one hour

Both read one word, `nightly`.

| | rustc | cargo |
|---|---|---|
| devenv, pinned by `devenv.lock` | `1.100.0-nightly (0fc141305 2026-09-11)` | `1.100.0-nightly (3c0b53475 2026-09-04)` |
| mise, pinned by `mise.lock` | `1.100.0-nightly (574ff7d98 2026-09-14)` | `1.100.0-nightly (7941be6fb 2026-09-11)` |

Four days apart on rustc and a week apart on cargo, from the same word on the
same day.

## What the prototype found that nobody predicted

1. ⚠️ **mise resolved `nightly-2026-09-15` on all four machines.** NixOS, ubuntu,
   macos and windows each reported `RUSTUP_TOOLCHAIN: nightly-2026-09-15` from
   the one word `nightly`. This is the property that has blocked the channel
   ticket twice, and it holds across operating systems.
2. ⚠️ **`[[tools.rust]]` in `mise.lock` carries no checksum and no platform
   entries.** The date is pinned, the artifact is not verified, because
   `core:rust` delegates to rustup. Reproducibility differs by backend inside
   one lockfile.
3. ⚠️ **mise's Python install failed on NixOS by default.** It ran python-build,
   which compiles from source, although the documentation says precompiled is
   the default. `MISE_PYTHON_COMPILE=0` installed it in 2.4 s.
4. ⚠️ **The two arms ship different macOS wheels.** devenv produced
   `macosx_14_0_arm64` and mise produced `macosx_11_0_arm64`. The devenv wheel
   excludes macOS 11 through 13.
5. ⚠️ **The confound is larger than expected.** Every runner already carries
   `cc`, `gcc`, `clang`, `ld`, rustup and Rust 1.98.1. No CI job can show that
   devenv supplies a compiler, because the runner supplied one first.
6. ⚠️ **A bare runner already honours `rust-toolchain.toml`.** The baseline's
   `rustc --version` printed `syncing channel updates for 1.98.1`, which is
   rustup's proxy reading the checked-out file with no manager present.
7. **devenv is 3 to 6 times slower cold in CI**: 125 s against 39 s on ubuntu,
   271 s against 45 s on macos.
8. ⚠️ **`MISE_DATA_DIR` does not redirect rustup's home.** The toolchain landed
   in `~/.rustup`, outside the scratchpad this prototype was confined to.

## The dimension never varied

The tree is the bare cdylib scaffold with its 93 dependency crates and no
product code. Nothing here tests free-threaded Python.

---

# Second question: can `rust-toolchain.toml` be dropped?

Asked after the arms ran. Measured the same day, on the same branch.

## Who reads the file

| reader | how |
|---|---|
| devenv | `devenv.nix:9`, `toolchainFile = ./rust-toolchain.toml` |
| rustup, anywhere | any `rustc` or `cargo` resolving through a rustup proxy, including the developer's own `~/.cargo/bin` |
| `Cargo.toml:19`, `README.md:20` | prose. No functional dependency |
| mise | not at all, unless `idiomatic_version_file_enable_tools` names rust |

## Both managers build a wheel without it

| arm | file present | file deleted |
|---|---|---|
| mise | wheel | **wheel.** `RUSTUP_TOOLCHAIN=nightly-2026-09-15`, unchanged |
| devenv, inline `channel` plus a complete `components` list | wheel | **every binary from one nightly** |

## ⚠️ The trap that cost this prototype a wrong finding

A first attempt at the devenv arm wrote `components = [ "clippy" "rustfmt"
"rust-analyzer" "rust-src" ]` and produced a shell that looked fine and was not.

| binary | resolved to | version |
|---|---|---|
| `rustc`, `cargo` | ⚠️ `~/.cargo/bin`, the developer's own rustup | **1.95.0** |
| `rustfmt`, `clippy-driver` | the nix store | `1.10.0-nightly`, 2026-09-11 |

1.95.0 is the version this project's README says cannot build its pinned ruff
crates, which declare `rust-version = "1.96"`.

The cause is one word meaning two things:

| file | `components` means |
|---|---|
| `rust-toolchain.toml` | **additive.** `profile = "minimal"` supplies `rustc`, `cargo` and `rust-std`, and `components` adds to it |
| `devenv.nix`, inline | ⚠️ **the complete list.** Omit `rustc` and the toolchain has none |

⚠️ **An incomplete list is silent.** PATH falls through to the developer's own
rustup and the shell still works, with the wrong compiler in it.

⚠️ **This prototype published that wrong finding before the human challenged
it.** It is the strongest evidence here for the asymmetry below, because it is
an instance of it.

## The asymmetry the two managers have

| manager | how it holds a toolchain | what a gap does |
|---|---|---|
| devenv | PATH ordering | ⚠️ **falls through to the developer's rustup default, silently** |
| mise | sets `RUSTUP_TOOLCHAIN` | no fall-through exists. It overrides a rustup default and the file alike |

## `rust-src` is not needed when the file is gone

Measured with `rust-src` absent from the components list. `RUST_SRC_PATH` still
resolves, to a `rust-src` matching the toolchain's own nightly.

Three signals rule out a stale evaluation:

| signal | evidence |
|---|---|
| devenv re-evaluated every run | the `rustc` store path changed each time |
| `rust-analyzer` is its own derivation | `rust-analyzer-preview-1.100.0-nightly-2026-09-12` |
| `rust-src` is not part of the toolchain | ⚠️ **it is not a reference of the toolchain derivation** |

devenv assembles the shell from separate per-component derivations and supplies
a matching `rust-src` on its own.

### ⚠️ Two lines the substrate ticket records do not hold for an inline channel

1. `toolchain.rust-src = config.languages.rust.toolchainPackage;` is not merely
   redundant. With `channel` and `components` set, devenv evaluation fails with
   `error: infinite recursion encountered`.
2. Its stated reason does not reproduce. The ticket records that without it
   devenv aims `RUST_SRC_PATH` at nixpkgs' `rustLibSrc`, from a different
   release than the rustc beside it. Under an inline nightly channel the default
   already resolves to the matching nightly.

⚠️ **Measured against an inline channel only.** It says nothing about the
`toolchainFile` arrangement the substrate pull request actually ships, where
that line may still earn its place.

## ⚠️ A finding that belongs to neither manager

The source distribution ships the file. Verified by building one:

```
purba-0.0.0/rust-toolchain.toml
```

Anyone building from source is put on purba's toolchain, and under a floating
nightly their build installs a nightly.

⚠️ **The sdist also ships `devenv.nix`, `devenv.lock`, `mise.toml`, `mise.lock`,
`prototype/README.md` and `.github/workflows/`.** No ticket owns this.

## The three options this leaves

| option | consequence |
|---|---|
| keep it | a bare runner needs no setup, and a developer's own rustup cannot shadow the project |
| remove it | works under both managers. The sdist problem goes with it |
| keep it, exclude it from the sdist | both protections survive and a source build is freed |

---

# Q2 and Q3: git hooks, and what replaces `enterShell`

Asked after mining the frozen prototype, whose environment the scaffold cannot
exercise. `prek.toml` and `prototype/hookprobe.sh` on this branch reproduce the
frozen prototype's shape: one `language = "system"` hook, so prek installs
nothing and the hook must find its tools on whatever PATH the caller provides.

## Q2. What a git hook can see

A commit was made under four environments. The probe recorded what each hook saw.

| | 1 bare | 2 mise shims, **not** activated | 3 mise activate | 4 devenv shell |
|---|---|---|---|---|
| `prek` | ⚠️ absent | shim | mise install | ⚠️ **absent** |
| `cargo` | ⚠️ absent | shim | ⚠️ `~/.cargo/bin` | nix store |
| `rustc` | ⚠️ absent | shim | ⚠️ `~/.cargo/bin` | nix store |
| `just` | ⚠️ absent | shim | mise install | ⚠️ **absent** |
| `rustc --version` | ⚠️ not found | **2026-09-14 nightly** | **2026-09-14 nightly** | 2026-09-11 nightly |
| `RUSTUP_TOOLCHAIN` | unset | unset | `nightly-2026-09-15` | unset |

**The hook itself always ran.** prek writes the absolute path of its own binary
into `.git/hooks/pre-commit` and falls back to `prek` on PATH, so the gate fires
even where nothing else is present. In the bare column a real `cargo fmt` hook
fails loudly, which is the right failure.

### ⚠️ Column 3 is the interesting one

Under `mise activate`, `cargo` and `rustc` resolve to the developer's **own**
`~/.cargo/bin`, and the version is still the project's nightly. `RUSTUP_TOOLCHAIN`
is doing the work, and it does not care about PATH order.

⚠️ **This is the same mechanism that made the earlier devenv failure silent, seen
from the other side.** devenv holds a toolchain by winning PATH, so losing PATH
loses the toolchain. mise sets an environment variable that steers whatever
rustup proxy is found first.

### ⚠️ Column 4 found a gap in the substrate

`prek` and `just` are **absent** from the devenv shell, because the substrate
pull request's `devenv.nix` does not list them. The two tickets that need them
are not written yet.

## Q3. When each manager's entry hook fires

`mise.toml` gained `[hooks] enter`, and `devenv.nix` gained an `enterShell`.
Both wrote to a log.

| invocation | mise `enter` | devenv `enterShell` |
|---|---|---|
| non-interactive command, `mise exec` or `devenv shell -- cmd` | ⚠️ **never** | **fires** |
| shims on PATH, no activation | ⚠️ **never** | not applicable |
| activated shell started **inside** the project | ⚠️ **never** | not applicable |
| activated shell, then `cd` into the project | **fires once** | not applicable |

⚠️ **A shell that starts inside the project does not fire mise's enter hook.**
That covers a CI checkout, an agent running `bash -c` in the repo, and an editor
opening a terminal in the project directory. The config was trusted, so trust is
not the cause.

### ⚠️ devenv's `enterShell` fires twice per invocation

Measured: one `devenv shell -- true` produced **2** firings, a second produced
**4**. Not investigated further. It matters because the frozen prototype's
`enterShell` calls `just build` and `just health`, so a side effect written
there runs twice for every non-interactive command.

## What this means for the decision

**Q2 favours mise, against expectation.** mise's own documentation warns that
under shims *"most hooks won't trigger"*, which was read here as a risk to git
hooks. It is not: that sentence is about mise's own hooks. Shims made the git
hook work with **no activation at all**, which is the hardest case.

**Q3 favours devenv, and the gap is real.** An `enterShell` runs everywhere,
including non-interactively. mise's `enter` hook runs in one situation only.
Anything that must happen before every command has no home in mise.

⚠️ **The scaffold has no `enterShell` today**, so Q3 prices a capability purba
does not yet use. The frozen prototype used it to rebuild a stale extension, and
a stale extension there was a known hazard that hid Rust check failures.

---

# Q4: worktrees and Worktrunk

Every repository here is driven by Worktrunk, so a manager that is slow or
heavy per worktree is slow and heavy every working day.

## Cost of a fresh worktree

A worktree was created with `wt switch --create`, both managers' files copied
into it, and each asked for a working `rustc`.

| | mise | devenv |
|---|---|---|
| cold, first call in a brand-new worktree | **0.105 s** | **4.48 s** |
| what it installed | **0 tools. 6 already installed** | reused the nix store, built a profile |
| what it wrote into the worktree | **nothing** | `.devenv/`, 916 KB for this scaffold |

`wt switch --create` itself is 0.032 s. The cost is entirely in what the
manager does afterwards.

## Per-worktree footprint at maturity

Measured on the frozen prototype's live tree.

| directory | size | what it is |
|---|---:|---|
| `.devenv/state/venv` | 225 MB | the uv virtualenv. ⚠️ **Not devenv's fault: a venv is per directory under any manager** |
| `.devenv/shell-*.sh` | ⚠️ **35 MB across 411 files** | cached shell scripts, accumulated |
| `.devenv` total | 260 MB | per worktree |

⚠️ **The 411 cached scripts are devenv-specific accumulation and nothing prunes
them.** Excluding the venv, devenv's own per-worktree cost in that tree is
essentially those files.

mise installs into one shared store, so a second worktree adds nothing.

## Both managers need a per-worktree trust step

| manager | what is untrusted | measured |
|---|---|---|
| devenv | `.envrc` | already handled by `.config/wt.toml`'s `pre-start.direnv`, which runs `direnv allow` |
| mise | `mise.toml` | ⚠️ **an untrusted config errors:** `Config files ... are not trusted` |

⚠️ **Every new worktree is a new path, so mise needs `mise trust` in each one.**
The remedy is symmetrical with the one already in the tree: a `pre-start` hook.

## ⚠️ The Worktrunk config is not manager-neutral

`.config/wt.toml` carries `post-switch = "direnv reload"` and a `pre-start`
hook that runs `direnv allow`. Both exist **because devenv is reached through
`.envrc`**. Under mise neither is needed in that form, and mise needs its own
trust hook instead.

⚠️ **That file was decided as copying wholesale.** Choosing mise rewrites it.

## ⚠️ Installing git hooks from a worktree hits every worktree

Found while measuring Q2 and repeated here because it belongs to this question:
`prek install` run inside a worktree wrote `pre-commit` into the **shared**
`.git/hooks`, which every worktree of the repository uses. The frozen prototype
handles this with an ordered task that sets `core.hooksPath`. Any ticket that
installs hooks inherits the problem, under either manager.

---

# Q5: the two dimensions where devenv wins. Can they be closed?

devenv's wins were the C toolchain and an entry hook that runs everywhere.
Both were probed. Both have solutions, and one of them improves the artifact.

## Gap 1: the C toolchain

Four candidates, three of them already measured elsewhere in this file.

| candidate | verdict |
|---|---|
| devenv as the system layer only | **works.** Wheel in 2.9 s. This is arm 3 |
| the host's own toolchain | **works** on all three runners, which already ship one. On NixOS it means adding a compiler to the system configuration |
| nix without devenv, `nix shell nixpkgs#gcc` | **works.** Also arm 3, and it is what produced that number |
| ⚠️ **zig, from mise** | **works, and produces a better wheel** |

### zig, measured strictly

`zig` is a first-class mise backend, `core:zig`, installed in 6.5 s. maturin
carries a `--zig` flag for manylinux compliance.

| PATH | result |
|---|---|
| mise shims, a two-line `cc` shim calling `zig cc`, `maturin build --zig` | ✅ **wheel** |
| mise shims only, no `cc` anywhere, `maturin build --zig` | ❌ `error: linker cc not found` |

⚠️ **`--zig` redirects the target build and not the host one.** Build scripts
compile for the host and still call `cc` by name, so zig must also be reachable
under that name. That is a two-line shim the repository would have to ship, and
it is a real cost.

### ⚠️ The wheel zig produced is more portable than either other arm

| arm | wheel tag |
|---|---|
| devenv | `manylinux_2_34_x86_64` |
| mise plus a nix compiler | `manylinux_2_34_x86_64` |
| **mise plus zig** | **`manylinux_2_17_x86_64.manylinux2014_x86_64`** |

glibc 2.17 against glibc 2.34. The zig wheel installs on far more Linux systems,
and nobody asked for that. It is the second time in this prototype that the
choice of environment silently decided which machines an artifact runs on, after
the macOS tag difference recorded above.

## Gap 2: an entry hook that runs everywhere

mise's `enter` hook fires only when an activated shell changes directory into
the project. The question is what the hook was **for**.

In the frozen prototype it rebuilt a stale extension, and a stale extension
there was a known hazard that hid failing Rust checks. ⚠️ **That is a missing
build-dependency expressed as a shell hook.** A stale artifact is a job for the
task graph, not for directory entry.

Measured, in the exact context where the enter hook never fires, with no
activation, no directory change and no interactive shell:

```
[build-guard] $ echo GUARD-RAN
[check]       $ echo CHECK-RAN
Finished in 8.6ms
```

⚠️ **A task dependency runs where the entry hook does not, including CI and an
agent's `bash -c`.** The entry hook never ran in CI under either manager, so
moving the work into the task graph is strictly wider coverage rather than a
workaround.

| candidate | verdict |
|---|---|
| **a task dependency** | **works, non-interactively, 8.6 ms** |
| do nothing | the scaffold has no `enterShell` today |
| mise `watch_files` hook | ⚠️ **not tested.** Likely needs activation, like `enter` |
| direnv with `use mise` | ⚠️ **not tested** |

## What survives

Gap 1 is closable four ways and one of them improves the artifact. Gap 2 is
closable by moving the work to where it belonged.

⚠️ **Neither closure is free.** The zig route ships a `cc` shim, and the task
route requires that every side effect have a task that depends on it, which is
a discipline rather than a mechanism.

---

# The two shortlisted arms, across architecture and operating system

Run [34972030646](https://github.com/kalonji-tools/purba/actions/runs/34972030646).
Configs in `prototype/arm-a/` and `prototype/arm-b/`, each shipping the
lockfile its own `mise.toml` produced.

**Arm A** is mise alone, with zig asked to supply the C toolchain.
**Arm B** keeps devenv as the system layer only: its `devenv.nix` deliberately
omits `languages.rust`, so the toolchain is demonstrably mise's.

## Coverage

| platform | A, mise plus zig | B, mise plus devenv |
|---|---|---|
| ubuntu x86_64 | **wheel, 50 s** | **wheel, 64 s** |
| ubuntu aarch64 | **wheel, 21 s** | **wheel, 53 s** |
| macOS arm64 | **wheel, 67 s** | **wheel, 217 s** |
| macOS x86_64 | **wheel, 195 s** | ⚠️ **cannot run** |
| Windows x86_64 | **wheel, 85 s** | ⚠️ not possible, WSL2 only |
| Windows arm64 | **wheel, 88 s** | ⚠️ not possible, WSL2 only |
| **total** | **6 of 6** | **3 of 6** |

### ⚠️ devenv's reach is narrower than "not Windows"

Arm B did not fail on Intel macOS for a configuration reason. It failed on an
upstream fact, quoted from the job:

```
error: Nixpkgs 26.11 has dropped support for x86_64-darwin.
```

Arm A built a wheel on that same runner.

## The criteria, met identically where both ran

| criterion | A | B |
|---|---|---|
| resolved toolchain | `rustc 1.100.0-nightly (574ff7d98 2026-09-14)` on **all six** | the same, on all three |
| clippy, rustfmt, rust-analyzer, rust-src | **all present, every platform** | **all present, every platform** |
| wheel | yes, on every platform that ran | yes, on every platform that ran |

⚠️ **One nightly, six platforms, two architectures, three operating systems,
from the single word `nightly`.**

## ⚠️ zig worked on three platforms of six

| platform | compiler actually used |
|---|---|
| ubuntu x86_64 | **zig** |
| macOS arm64 | **zig** |
| macOS x86_64 | **zig** |
| ⚠️ ubuntu aarch64 | **host toolchain. zig refused** |
| Windows x86_64 | host toolchain. Expected: rustc drives `link.exe` on the msvc target and never consults `cc` |
| Windows arm64 | host toolchain, same reason |

⚠️ **The arm64 Linux refusal is unexplained and was not predicted.** The job
falls back to the host compiler and still produces a wheel, so the arm passes
while its distinguishing feature is silently absent on that platform.

## ⚠️ The environment decides which machines the artifact runs on

| platform | A | B |
|---|---|---|
| linux x86_64 | **`manylinux_2_17`** | `manylinux_2_34` |
| linux aarch64 | `manylinux_2_34` | `manylinux_2_34` |
| macOS arm64 | **`macosx_11_0`** | `macosx_14_0` |
| macOS x86_64 | **`macosx_10_12`** | not built |

Where zig ran, the floor drops by years: glibc 2.17 against 2.34, macOS 11
against 14, and macOS 10.12 on Intel. Where zig refused, the two arms agree.

## What it took to get here, recorded because it is the expensive part

Three full matrix runs failed before any of the above was measured, none of them
for a reason either arm is responsible for.

| failure | cause |
|---|---|
| all ten legs | mise-action detects a lockfile and runs `--locked`; the lockfile at the repository root belonged to neither arm |
| one leg, on the platform the lock was generated on | the lock held **two** `python` entries, one carrying a stale `compile = "true"` option, and only one was matchable |
| all four arm B legs | a patch applied to the workflow ate the dollar signs in its build step |

⚠️ **A local `mise install --locked` that reports "already installed" validates
nothing.** It never resolves the lockfile. The working lock was checked against
an empty store, where it installed 7 of 7 in 22.1 s.

⚠️ **`actionlint` rejected `macos-13` before it cost a run.** The Intel runner
is now `macos-15-intel`.

---

# Crossing the PyO3 bridge, which building a wheel does not prove

Every leg up to here proved a wheel **compiles and links**. None loaded one into
a Python interpreter. The crate carries a real `#[pymodule]`, and the crate's own
doc comment already says a verified Rust example covers the core and never the
bridge. So the bridge was untested, and it is untested exactly where it matters
most: in the arm that changes the linker.

Run [34978397756](https://github.com/kalonji-tools/purba/actions/runs/34978397756).
Each arm installs its own wheel into a fresh virtual environment and runs
`prototype/bridge_check.py`.

## ⚠️ The first attempt at this check was too weak, and would have passed

It reported `purba.__file__`, which is `purba/__init__.py`, because maturin ships
a package whose first line is `from .purba import *`. The extension did load, but
the probe could not show it and **would have reported success for a package with
no Rust in it at all.**

The check now asserts on `purba.purba.__file__` and requires a `.so`, `.pyd` or
`.dylib` suffix.

## The result

| arm | platform | extension loaded |
|---|---|---|
| A | ubuntu x86_64 | `purba.abi3.so` |
| A | ubuntu aarch64 | `purba.abi3.so` |
| A | macOS arm64 | `purba.abi3.so` |
| A | macOS x86_64 | `purba.abi3.so` |
| A | Windows x86_64 | `purba.pyd` |
| A | Windows arm64 | `purba.pyd` |
| B | ubuntu x86_64 | `purba.abi3.so` |
| B | ubuntu aarch64 | `purba.abi3.so` |
| B | macOS arm64 | `purba.abi3.so` |

**Six of six for arm A, three of three for arm B, on Python 3.12.14.**

Loading the extension runs PyO3's module initialisation, which is Rust, so this
is the first evidence in this prototype that Rust code executed inside a Python
process rather than merely compiling.

## ⚠️ abi3 proved itself by accident

Locally the same zig-built wheel, built against Python 3.12, loaded into
**Python 3.14.4**. Nobody asked for that and it is what `abi3` is for.

## What is still not tested

The module is empty, so nothing calls a Rust function and returns a value.
**Module initialisation running is the strongest claim available until the
crate holds product code**, which is the scaffold's whole premise.
