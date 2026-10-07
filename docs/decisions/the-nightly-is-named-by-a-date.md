# The nightly is named by a date

## Context and Problem Statement

[purba meets the next trait solver before it stabilizes](purba-meets-the-next-trait-solver-before-it-stabilizes.md) puts purba on Rust nightly.

A second question arrives with the channel, because the channel has to be named in a file.
[One manifest declares each package](one-manifest-declares-each-package.md) already decided which file that is, and the tree contradicted it.
`rust-toolchain.toml` named a stable release under a comment describing a removed environment.
Leaving both files in place is not a neutral duplication.
Three of rustup's five precedence ranks decide why.

| rustup precedence | rank |
|---|---|
| `cargo +toolchain` on the command line | 1 |
| `RUSTUP_TOOLCHAIN`, which mise sets | 2 |
| `rust-toolchain.toml` | 4 |

A toolchain file that disagrees with mise loses inside a mise shell and wins in any job that does not run through mise.
Nothing reports the difference.

## Considered Options

- **A floating channel name.** Rejected. A floating name drifts, and no lockfile stops it.
- **`RUSTUP_TOOLCHAIN` under `[env]`.** Rejected. It overrides what the backend installs and says nothing when the two differ.
- **A toolchain file at the root.** Rejected. It loses inside a mise shell and wins outside one. Nothing reports the difference.
- **A date, named once in `.config/mise.toml`.** Chosen. A date is an exact version, so it resolves only to itself, and this is the form mise documents.

## Decision Outcome

**The nightly is named once, in `.config/mise.toml`, as a date.**
`.config/mise.lock` records the same string, because mise derives an exact request's lock row from the request.
So `.config/mise.lock` cannot disagree with `.config/mise.toml`.
A person moves the date.

⚠️ **A floating name drifts, and no lockfile stops it.**
[purba carries no tool manager beside mise, and no compiler of its own](purba-carries-no-tool-manager-beside-mise-and-no-compiler-of-its-own.md) holds the mechanism and what it costs.

**purba holds no `rust-toolchain.toml`.**
This follows from [one manifest declares each package](one-manifest-declares-each-package.md) and decides nothing new.
Its absence is what stops the tree contradicting that record.

**Downside:**

- **A person must move the date, or purba freezes on one compiler.** `.github/workflows/bump.yml` proposes a newer date every week for a person to sign.

## Confirmation

| property | check |
|---|---|
| the toolchain is the same everywhere | **true by construction, not by a check.** The request is exact, so every machine resolves the same version. `.github/workflows/build.yml` names the compiler each `rust` job built with, on three operating systems, so a reader can audit it; that step refuses nothing. ⚠️ **The claim excludes mise itself**, which cannot pin its own version, so each workflow names it and a developer machine does not |
