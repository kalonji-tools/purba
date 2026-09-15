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
