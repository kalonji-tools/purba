# One manifest declares each package

## Context and Problem Statement

purba needs one answer to the question of which version of a package it uses.
[purba carries no tool manager beside mise, and no compiler of its own](purba-carries-no-tool-manager-beside-mise-and-no-compiler-of-its-own.md) chose mise to give it.

**One manager cannot declare every package.**
Only cargo links a crate into purba.
A Python package that imports purba must share its environment.
mise installs a Python tool into an environment of its own.

## Considered Options

Each answer below places the line where mise stops.

- **mise declares whatever it can fetch.** Rejected. mise installs a Python tool such as pytest into an environment of its own, where it cannot import purba.

- **A package declared where a foreign tool reads it, and in mise as well.** Rejected, because a package that two managers declare is a conflict in the making. It keeps maturin in `pyproject.toml` for pip and in `.config/mise.toml` for everyone else.

- **`pyproject.toml` declares maturin, and mise drops it.** Rejected. A source build would work, but no lockfile would hold the build backend.

- **A package that shares purba's process or environment goes in its language's manifest, and mise declares the rest.** Chosen. One reason places a linked crate and a test runner alike.

## Decision Outcome

**One manifest declares each package.**

| package | manifest |
|---|---|
| a crate purba links | `Cargo.toml` |
| a Python package that imports purba or shares its environment | `pyproject.toml`, under `[dependency-groups]` |
| every other package, which runs over the files | `.config/mise.toml` |
| a crate a cargo script links | the manifest inside that script |

A name in two manifests is a conflict.
A mise backend that installs through cargo or pip is one declaration, because one file names the package.
A crate shares the process of the cargo script that links it, so the script declares it.
[A script that needs a Rust library is a cargo script](a-script-that-needs-a-rust-library-is-a-cargo-script.md) says when a script is one.

`.config/mise.toml` declares maturin.
`pyproject.toml` keeps its `[build-system]` table with an empty `requires` list, because maturin refuses a file without that table.
A frontend such as pip or uv installs what that list names from PyPI, so the list stays empty.
purba chooses its Python manager with the first Python package it declares.

**Downside:**

- **Nobody builds purba from source with pip or uv.** Each stops with an import error that does not name mise.
- **The gate compares names, not packages.** A package published under a different name in each registry passes `mise run lint:manifests`. A mise key that names a URL passes it as well.
- **The first Python package owes the gate a reader.** Until then the gate refuses any package in `pyproject.toml`.

## Confirmation

| property | check |
|---|---|
| each package is declared in one manifest | ✅ `mise run lint:manifests`, through `lint`. It compares `.config/mise.toml`, `Cargo.toml` and the manifest inside each cargo script by name, in lower case with each run of `-`, `_` and `.` read as one `-`. It refuses any package in `pyproject.toml` |
| a crate a cargo script links is declared in that script alone | ✅ the same command. `cargo metadata -Zscript` reads the manifest inside each tracked cargo script |
