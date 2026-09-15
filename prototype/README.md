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
