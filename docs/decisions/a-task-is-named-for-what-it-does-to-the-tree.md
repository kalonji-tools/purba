# A task is named for what it does to the tree

## Context and Problem Statement

`mise tasks ls` sorts every task by name, whatever order `.config/tasks.toml` uses.
The name is therefore the only grouping a reader of that listing sees.
A group that lists its members by hand runs a new lint in CI only when its author also adds it to the list.

| measured on mise 2026.8.6 | |
|---|---|
| the order `mise tasks ls` prints | by name, A to Z |
| what `depends = ["lint:*"]` runs | every task whose name starts with `lint:` |
| `lint:cargo-scripts` in CI, cache warm | 0 to 1 s |
| `lint:cargo-scripts` in CI, cache cold | 15 to 23 s |

## Considered Options

- **The subject first, such as `shell:lint`.** Rejected. A check that reads the whole repository has no subject, so it needs an invented group.
- **The prefix names a topic, such as `fmt:check`.** Rejected. A reader cannot tell from the prefix whether a task changes their files.
- **One task for each tool, such as `lint:clippy`.** Rejected. A tool that reads two subjects splits one subject across two names.
- **A group with a list kept by hand, such as `quality`.** Rejected. A lint left off the list runs nowhere in CI, so it refuses nothing.
- **The verb first, the subject after the colon, and a bare verb as the group.** Chosen. The verb answers whether a task touches your files, and the wildcard keeps every group whole.

## Decision Outcome

**A task is named `<verb>:<subject>`.**

| verb | what the task does to the tree |
|---|---|
| `fmt` | rewrites files |
| `lint` | reads files and refuses, and never runs the code it reads |
| `test` | runs code and refuses on a failure |

**A subject keeps one word under every verb.**
`rust`, `shell` and `cargo-scripts` each name the same files under `fmt`, `lint` and `test`.
A check that reads the whole repository takes the name of its rule, such as `lint:links`.

**A bare verb runs every task under it.**
`fmt`, `lint` and `test` each depend on their prefix with a wildcard, and `check` runs `lint` and `test`.
A new `lint:` task runs in CI on the day it is written.
`build` is bare because it is the only task of its verb.

**`.config/tasks.toml` lists its tasks A to Z.**
That is the order `mise tasks ls` prints.

**A hidden task stays outside the scheme.**
`sign-off` and `apply-fork` never appear in `mise tasks ls`.

**Downside:**

- **A task that runs three tools stops at the first failure.** A clippy refusal in `lint:rust` hides a broken documentation link until clippy passes.
- **Every lint joins the required check.** A `lint:` task that takes minutes slows `Quality` on every pull request from the day it is written.
- **A guard that runs code cannot be a lint.** `cargo test --doc` refuses a crate with no `rlib`, so `Build` holds that guard and `Quality` does not.
- **No command refuses a name off the scheme.** The reviewer reads each new name.

## Confirmation

| what shows it | where |
|---|---|
| each visible task follows the scheme | `mise tasks ls` |
| each group runs every task under its prefix | `mise tasks deps fmt lint test check` |
| CI runs every lint | `.github/workflows/quality.yml` runs `mise run lint` |

The measurements above were made on [Which naming scheme sorts the command roster?](https://github.com/kalonji-tools/purba/issues/341).
