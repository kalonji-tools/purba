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

## Considered Options

- **Move each file its tool finds under `.config/`, and leave the rest at the root.** Rejected. The root keeps two files that `pyproject.toml` could hold.
- **Fold each file into `pyproject.toml` where its tool reads a table there, and move nothing.** Rejected. The root keeps every file that only `.config/` could take.
- **Move and fold, with no order between the two.** Rejected. git-cliff reads both locations, and no rule picks one.
- **A flag that names the file under `.config/`.** Rejected. Run bare or in an editor, typos, lychee and shellcheck fall back to their defaults and report nothing.
- **An environment variable that names the file.** Rejected. Only git-cliff and shellcheck read one, and an editor outside mise sees neither.
- **An order of three locations.** Chosen. One rule places every tool, and the root keeps only what no tool reads elsewhere.

## Decision Outcome

**Reach:** `/*` `/.config/**` `typos.toml` `_typos.toml` `.typos.toml` `lychee.toml` `.github/scripts/changed-scripts.sh` `scripts/test/changed-scripts.bats`

**A tool's configuration lives in the first location its tool reads: `.config/`, then `pyproject.toml`, then the root.**

"Reads" means the pinned tool finds the file run bare, with no flag and no environment variable.
A flag reaches only the command that passes it, and an editor passes none.

**A file only GitHub reads stays outside the order.**
It lives under `.github/`, as [only GitHub runs what lives under `.github`](only-github-runs-what-lives-under-github.md) holds.

**A file that only purba's own scripts read takes the first location.**
`.config/readers` is one, because the script that reads it can look anywhere.

**No gate holds the rule, so it is an exception to [a gate owns the mechanical standard](a-gate-owns-the-mechanical.md).**
A gate knows only the tools already placed.
It refuses nothing when a new tool arrives, which is the one time the rule applies.

**Downside:**

- **`pyproject.toml` gains a reader.** A packager who opens it for the metadata also reads the settings of typos and lychee.
- **A dedicated file outranks its table.** typos reads `typos.toml` before `[tool.typos]`, and lychee reads `lychee.toml` before `[tool.lychee]`. A file of that name added later takes over, and nothing refuses it.
- **Some files stay at the root.** `deny.toml`, `prek.toml`, `.shellcheckrc`, `.editorconfig`, `.gitignore` and `.gitattributes` have no other location their tools read.
- **Nothing refuses a file placed below its first location.** The order holds only as far as a reviewer reads it.

## Confirmation

| property | check |
|---|---|
| each tool reads its moved configuration | run bare from the root: `typos` refuses a planted `ticket`, `lychee --dump` lists a URL inside a fenced block, `mise ls --current` lists every tool, and `git-cliff --bumped-version` names the next version |
| a change to the release configuration runs the release tests | `.github/scripts/changed-scripts.sh` matches `.config/cliff.toml`, and `scripts/test/changed-scripts.bats` holds it |
| a citation of a moved file names where it lives | `mise run lint:links` refuses a path under `.config/` that git does not track |

No command checks the order itself, and none is planned.
