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
| at a root that holds only the workspace, `cargo test --doc` warned on a bare cdylib and exited 0, and `cargo test --doc -p purba` exited 101 | [the probes' output](https://github.com/kalonji-tools/purba/issues/556#issuecomment-6170716506) |
| `release.sh` reads purba's version from the `[package]` table of `Cargo.toml` | [`.github/scripts/release.sh`](../../.github/scripts/release.sh) |
| `check-links.sh` checks a cited path only under `docs`, `scripts`, `src`, `.github` and `.config` | [`scripts/check-links.sh`](../../scripts/check-links.sh) |

## Considered Options

- **`src/<part>/`, beside the crate of the extension module in `src/`.** Rejected. Bare `cargo test` reaches the parts only through `default-members`.
- **`crates/<part>/`, with the crate of the extension module in `src/`.** Rejected. Each record about product code names two patterns.
- **The registration in the pull request that records this.** Rejected. `crates/**` matches no file until the code moves. Until then, `src/lib.rs` falls outside each record that names `crates/**`.
- **Every crate under `crates/`, beside a root that holds only the workspace.** Chosen. Bare `cargo test` reaches every crate with no setting.

## Decision Outcome

**Reach:** `/Cargo.toml` `crates/**` `/pyproject.toml` `/.config/gates` `/.config/readers` `/.config/tasks.toml` `scripts/check-links.sh` `.github/scripts/release.sh`

**Every crate lives under `crates/`, beside a root manifest that holds only the workspace.**

| file | what it holds |
|---|---|
| `/Cargo.toml` | `[workspace]`, `[workspace.package]`, `[workspace.dependencies]` and `[workspace.lints]` |
| `crates/purba/Cargo.toml` | the `[package]` of the crate that builds the extension module, with its version, `crate-type` and `extension-module` |
| `crates/<part>/Cargo.toml` | the `[package]` of a part, and `[lints] workspace = true` |

- **The lint tables of today's `Cargo.toml` move to `[workspace.lints]`.** Each member takes them with `[lints] workspace = true`.
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

- **A member that leaves out `[lints] workspace = true` takes only the default lint levels.** Cargo warns of it and exits 0, so no gate refuses it. The slice that adds the second crate decides whether a gate refuses it.
- **The facts of the extension module leave the root manifest.** A reader of `/Cargo.toml` finds the version, `crate-type` and features in `crates/purba/Cargo.toml`.

## Confirmation

No command checks it, because no crate lives under `crates/` yet.
The slice of [Wayfinder: purba's walking skeleton, running every part on main](https://github.com/kalonji-tools/purba/issues/533) that moves `src/lib.rs` into `crates/purba/` registers `crates/`.
