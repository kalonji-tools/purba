# Configuration takes the first location its tool reads

## Context and Problem Statement

**Configuration files crowd the project root.**
Each tool purba runs reads its configuration from a file, and each tool looks at the root unless told otherwise.
GitHub lists the root's files above the README, so every configuration file there sits between a reader and the README.
[A gate owns the mechanical standard](a-gate-owns-the-mechanical.md) decides whether a file is owed, and not where it lives.

A file can leave the root only for a place its tool reads, and the places differ from tool to tool.
Run bare, with no flag and no environment variable, each pinned tool finds its file here:

| tool | `.config/` | `pyproject.toml` | the root |
|---|---|---|---|
| mise | ✅ | | ✅ |
| git-cliff | ✅ | ✅ | ✅ |
| typos | | ✅ | ✅ |
| lychee | | ✅ | ✅ |
| cargo-deny, prek, shellcheck | | | ✅ |
| EditorConfig, git | | | ✅ |

A Rust tool also reads other places. Each one finds its file here when it runs from the root:

| tool | `.cargo/` | the root |
|---|---|---|
| cargo | ✅ `config.toml` | |
| cargo-deny | ✅ `deny.toml` | ✅ |
| clippy | through `CLIPPY_CONF_DIR`, which `[env]` in `.cargo/config.toml` sets | ✅ |
| rustfmt | | ✅ |

Neither cargo-deny nor clippy reads `.config/`.

## Considered Options

- **Move each file its tool finds under `.config/`, and leave the rest at the root.** Rejected. The root keeps two files that `pyproject.toml` could hold.
- **Fold each file into `pyproject.toml` where its tool reads a table there, and move nothing.** Rejected. The root keeps every file that only `.config/` could take.
- **Move and fold, with no order between the two.** Rejected. git-cliff reads both locations, and no rule picks one.
- **A flag that names the file under `.config/`.** Rejected. Run bare or in an editor, typos, lychee and shellcheck fall back to their defaults and report nothing.
- **An environment variable that names the file, set outside the tool.** Rejected. Only git-cliff and shellcheck read one, and an editor outside mise sees neither. The variable that names clippy's file is set by cargo itself.
- **An order of three locations.** Chosen. One rule places every tool, and the root keeps only what no tool reads elsewhere.
- **The configuration of each Rust tool at the root.** Rejected. The root keeps two files that `.cargo/` can hold.
- **The configuration of each Rust tool under `.cargo/`.** Chosen. One directory holds the configuration of cargo, cargo-deny and clippy.

## Decision Outcome

**Reach:** `/*` `/.config/**` `/.cargo/**` `typos.toml` `_typos.toml` `.typos.toml` `lychee.toml` `scripts/check-gates.sh`

**A tool's configuration lives in the first location its tool reads: `.config/`, then `pyproject.toml`, then the root.**

"Reads" means the pinned tool finds the file run bare, with no flag and no environment variable. Clippy, below, is the one exception.
A flag reaches only the command that passes it, and an editor passes none.

**A file only GitHub reads stays outside the order.**
It lives under `.github/`, as [only GitHub runs what lives under `.github`](only-github-runs-what-lives-under-github.md) holds.

**A file that only purba's own scripts read takes the first location.**
`.config/readers` is one, because the script that reads it can look anywhere.

**A Rust tool that reads `.cargo/` is the exception to the order: its configuration lives there.**
Cargo reads `.cargo/config.toml`.
cargo-deny reads `.cargo/deny.toml`.
Clippy reads `.cargo/clippy.toml`, because `[env]` in `.cargo/config.toml` sets `CLIPPY_CONF_DIR` with `force = true`.
Cargo sets that variable itself, so an editor that runs `cargo clippy` sees it too.

`Cargo.toml` and `Cargo.lock` stay at the root, because cargo finds the workspace by its manifest there.
A Rust tool that reads no file under `.cargo/` follows the order above. rustfmt is one, so a `rustfmt.toml` would go to the root.

**No gate holds the rule, so it is an exception to [a gate owns the mechanical standard](a-gate-owns-the-mechanical.md).**
A gate knows only the tools already placed.
It refuses nothing when a new tool arrives, which is the one time the rule applies.

**Downside:**

- **`pyproject.toml` gains a reader.** A packager who opens it for the metadata also reads the settings of typos and lychee.
- **A dedicated file outranks its table.** typos reads `typos.toml` before `[tool.typos]`, and lychee reads `lychee.toml` before `[tool.lychee]`. A file of that name added later takes over, and nothing refuses it.
- **Some files stay at the root.** `prek.toml`, `.shellcheckrc`, `.editorconfig`, `.gitignore` and `.gitattributes` have no other location their tools read.
- **`.cargo/clippy.toml` needs a line in `.cargo/config.toml`.** Without the `[env]` line, clippy reads no list. `mise run lint:rust` still fails, because the `#[expect]` in `purba-parsing` then expects a warning that never comes.
- **A root `deny.toml` outranks `.cargo/deny.toml`.** cargo-deny reads the root first. A file of that name added at the root takes over, and nothing refuses it.
- **Nothing refuses a file placed below its first location.** The order holds only as far as a reviewer reads it.

## Confirmation

| property | check |
|---|---|
| each tool reads its moved configuration | run bare from the root: `typos` refuses a planted `ticket`, `lychee --dump` lists a URL inside a fenced block, `mise ls --current` lists every tool, and `git-cliff --bumped-version` names the next version |
| a change to the release configuration runs the release tests | `.config/gates` marks `.config/cliff.toml` `scripts`, and `mise run lint:gates` refuses the release configuration at a path the marks do not name |
| a citation of a moved file names where it lives | `mise run lint:links` refuses a path under `.config/` that git does not track |
| each Rust tool reads its file under `.cargo/` | run from the root: `mise run lint:licences` exits 4 with no `.cargo/deny.toml`, and 0 with it. `mise run lint:rust` refuses a call to `parse_module` from `purba` with "use of a disallowed method" |

No command checks the order itself, and none is planned.
