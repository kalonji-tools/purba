# purba writes its scripts in bash until purba can test them

## Context and Problem Statement

purba's own scripts are shell, and nothing in the tree said what language they had to be written in.
The question governs every script added from now on.

Nothing tested any of that shell when purba took this decision.
Every defect in it was found by continuous integration or by a throwaway pull request.
One of them refused the pull request that introduced it.
A language is worth no more here than the runner that can test it.

Two arguments pull toward a typed language, and they select different files.
The calls into GitHub's API sit in the thinnest scripts in the tree.
The scripts holding real branch logic make none of those calls.
Whichever argument is taken, the other set stays in shell.

| calls the scripts made when purba took this decision | n |
|---|---:|
| `git` | 73 |
| `gh` | 19 |
| `jq` | 12 |

In another language each of those git invocations becomes a subprocess call carrying more ceremony than the line it replaces.

## Considered Options

- **Rust, for every script.** Rejected. A script written in Rust must be compiled before it can gate, so it cannot gate the build that compiles it.
- **Python, now.** Rejected. It costs a linter and a test runner this project does not use, and it buys nothing shell does not already have.
- **Bash.** Chosen. Three tools already gate it, and it fits the plumbing above.

The trigger for a later change was weighed separately.

- **Once purba is mature.** Rejected. Nothing decides it, and [Decide the Rust/Python placement rule](https://github.com/kalonji-tools/purba/issues/6) is out of scope for the same reason.
- **Once purba runs purba's own test suite.** Chosen. One command answers it.

So was the runner.

- **`shellspec`.** Rejected. `shfmt` strips every nested level from a spec body, because `Describe` and `It` are plain commands to a shell parser.
- **`bats`.** Chosen. It survives the formatter, and `shellcheck` reads its test blocks as brace groups.

## Decision Outcome

**purba writes its scripts in bash.**
A change of language is earned by something the new language brings that bash cannot.

**A change to Python is earned when purba runs purba's own test suite.**
purba runs Python tests and nothing else, so at that point the scripts' tests become the product's own first user.

**The language the product is written in is not evidence for the language its scripts are written in.**
A script and the product share no user.
That excludes a pull toward Rust because the product is Rust, and a pull toward Python because the product tests Python.
The trigger above turns on which runner executes a script's tests, so the exclusion leaves it standing.

**`bats` is the runner, and a test of a script ends in `.bats`.**
[A gate owns the mechanical standard](a-gate-owns-the-mechanical.md) puts a writer ahead of a checker, and `shfmt` is a writer.
A `shellspec` file ends in `.sh`, which is the glob every shell task already reads.
The formatter therefore reaches every spec the moment it is written.

**The Python inside `build.yml` is one standing exception.**
A `run:` block there asserts the built extension is a real shared object, and no linter reads it.

**A script that needs a Rust library is the other.**
[A script that needs a Rust library is a cargo script](a-script-that-needs-a-rust-library-is-a-cargo-script.md) says what that script is.

**Downside:**

- **Nothing checks this.** An extension shows the language of a script, and no command reads it. A reviewer decides whether a `.rs` script needs its library.
- **The trigger cannot be confirmed until purba can meet it.** No reader can test half of this record today.
- **A test suite written in bash is rewritten when the trigger fires.** The tests move with the scripts they cover.
- **Adopting `bats` widens a glob in two files.** A narrowed glob removes a gate and nothing fails.
- **A pull request runs the tests only when it changes a script or a setting a script test copies.** A new version of `bats`, `jq` or git can break a script. The next change to a script is what finds the break.

## Confirmation

**No command refuses a script for its language.**
A reviewer reads the extension.
The Downside above carries that cost.

`mise.toml` names `bats`.
`mise run test:scripts` runs every test under `scripts/test`.
`mise run check` runs that task.
`.github/workflows/scripts.yml` runs it when a pull request changes a file that `.github/scripts/changed-scripts.sh` names.
`.github/workflows/sign.yml` waits for the `Scripts` check, so purba does not sign a branch whose test fails.

`shellcheck`, `shfmt` and `editorconfig-checker` gate the shell this record keeps, through `mise run quality`.
The first two read a glob that names `*.sh` and `*.bats`, so both reach a test file.
`editorconfig-checker` reads every tracked file.

This record refuses `shellspec`, and `mise.toml` does not name it.
