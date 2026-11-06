# Every crate lives under crates/

## Context and Problem Statement

[Each part is a crate](each-part-is-a-crate.md#decision-outcome) makes purba a workspace of crates.
Today the crate that builds the extension module is the package at the root, with its code in `src/`.
Records reach product code through `src/**`.
`.config/gates` marks `src/**` for Build.
`.config/readers` binds `src/**` to its readers.
Each predates any product code outside `src/`.

| fact | source |
|---|---|
| with a package at the root, bare `cargo test` ran that crate alone, and `test:rust` ran none of the three tests and exited 0 | [How do three layouts of a part's boundary compare when the walking skeleton and purba's own tasks run on them?](https://github.com/kalonji-tools/purba/issues/556#issuecomment-6168468953) |
| with a root that holds only the workspace, bare `cargo test` ran every crate | [How do three layouts of a part's boundary compare when the walking skeleton and purba's own tasks run on them?](https://github.com/kalonji-tools/purba/issues/556#issuecomment-6168468953) |
| `cargo clippy` and `cargo fmt` refused a breach inside a member crate | [How do three layouts of a part's boundary compare when the walking skeleton and purba's own tasks run on them?](https://github.com/kalonji-tools/purba/issues/556#issuecomment-6168468953) |
| once `.config/gates` and `.config/readers` named each directory, `src/<part>/` and `crates/` passed the same tasks, and a change inside a part named as many records | [How do three layouts of a part's boundary compare when the walking skeleton and purba's own tasks run on them?](https://github.com/kalonji-tools/purba/issues/556#issuecomment-6168468953) |
| a member without `[lints] workspace = true` took the default lint levels, and cargo warned of it and exited 0 | [the probes' output](https://github.com/kalonji-tools/purba/issues/556#issuecomment-6170716506) |
| at a root that holds only the workspace, with another member that has a library, `cargo test --doc` warned on a bare cdylib and exited 0, and `cargo test --doc -p purba` exited 101 | [the probes' output](https://github.com/kalonji-tools/purba/issues/556#issuecomment-6170716506) |
| `release.sh` reads purba's version from the `[package]` table of `Cargo.toml` | [`.github/scripts/release.sh` at the base](https://github.com/kalonji-tools/purba/blob/499a8c8d777921b4e035ee35e4ff208dbc7e1bf7/.github/scripts/release.sh#L25) |
| `check-links.sh` checks a cited path only under `docs`, `scripts`, `src`, `.github` and `.config` | [`scripts/check-links.sh` at the base](https://github.com/kalonji-tools/purba/blob/499a8c8d777921b4e035ee35e4ff208dbc7e1bf7/scripts/check-links.sh#L120) |

## Considered Options

- **`src/<part>/`, beside the crate of the extension module in `src/`.** Rejected. Bare `cargo test` reaches the parts only through `default-members`.
- **`crates/<part>/`, with the crate of the extension module in `src/`.** Rejected. Each record about product code names two patterns.
- **The registration in the pull request that records this.** Rejected. `crates/**` matches no file until the code moves. Until then, `src/lib.rs` falls outside each record that names `crates/**`.
- **Every crate under `crates/`, beside a root that holds only the workspace.** Chosen. Bare `cargo test` reaches every crate with no setting.

## Decision Outcome

**Reach:** `/Cargo.toml` `/.cargo/config.toml` `crates/**` `/pyproject.toml` `/.config/gates` `/.config/readers` `/.config/tasks.toml` `scripts/check-links.sh` `.github/scripts/release.sh`

**Every crate lives under `crates/`, beside a root manifest that holds only the workspace.**

| file | what it holds |
|---|---|
| `/Cargo.toml` | `[workspace]`, `[workspace.package]`, `[workspace.dependencies]` and `[workspace.lints]` |
| `crates/purba/Cargo.toml` | the `[package]` of the crate that builds the extension module, with its version, `crate-type` and `extension-module` |
| `crates/<part>/Cargo.toml` | the `[package]` of a part, and `[lints] workspace = true` |

- **The lint tables of today's `Cargo.toml` move to `[workspace.lints]`.** Each member takes them with `[lints] workspace = true`.
- **`.cargo/config.toml` sets `warnings = "deny"` under `[build]`.** Cargo warns of a member that leaves out `[lints] workspace = true`, so each cargo command refuses that member.
- **The slice that moves `src/lib.rs` into `crates/purba/` registers `crates/`.** In the same pull request, each change below lands.
  - `src/` leaves the tree, because no file stays in it.
  - Each reach that names `src/**` names `crates/**`.
  - Each reach that names `/Cargo.toml` for a fact of the extension module names `crates/purba/Cargo.toml`.
  - `.config/gates` and `.config/readers` name `crates/**` in place of `src/**`.
  - `scripts/check-links.sh` checks a cited path under `crates/`.
  - `test:rust` also runs the doc tests of `purba` by name, so that a bare cdylib still fails.
  - `release.sh` reads the version from `crates/purba/Cargo.toml`.
  - `pyproject.toml` names `manifest-path` under `[tool.maturin]`.

**Downside:**

- **A member with its own `[lints]` table takes only the levels it names.** Cargo does not warn of it, so no gate refuses it.
- **Each new warning of cargo refuses each cargo command.** A newer nightly that warns of something new turns every cargo task red.
- **The facts of the extension module leave the root manifest.** A reader of `/Cargo.toml` finds the version, `crate-type` and features in `crates/purba/Cargo.toml`.

## Confirmation

`/Cargo.toml` names `members = ["crates/*"]`, so a crate elsewhere is not a member of the workspace.
Three commands refused a crate at `tools/stray/`:

| what | result |
|---|---|
| `mise run lint:gates` | exit 1: `.config/gates` marks none of its files |
| `mise run lint:readers` | exit 1: its `src/lib.rs` reaches no actor |
| `cargo check` inside it | exit 101: "current package believes it's in a workspace when it's not" |

`mise run lint:rust` and `mise run test:rust` exited 0, because neither builds a crate that is not a member.
A person who marks its directory in `.config/gates` and `.config/readers` passes `lint:gates` and `lint:readers`.
The reviewer then reads where the crate lives.

Each cargo command refused a member of the workspace without `[lints] workspace = true`, and passed it once it had the table:

| what | without the table | with it |
|---|---|---|
| `mise run lint:rust` | exit 101: "missing `[lints]` to inherit `[workspace.lints]`", then "warnings are denied by `build.warnings` configuration" | exit 0 |
| `mise run test:rust` | exit 101: the same two lines | exit 0 |

`missing_lints_inheritance = "deny"` under `[workspace.lints.cargo]` refused nothing, because cargo reads that level from the member's own `[lints]` table.
