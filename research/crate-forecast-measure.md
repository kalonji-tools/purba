# Measuring the levels of a crate by script, for the forecast

Research for [Can a script measure the levels of a crate that purba does not lock, as research/future-crates.md measured them by hand?](https://github.com/kalonji-tools/purba/issues/995).
Measured on 2026-11-15, against purba `main` at `af20414`.
The baseline is [`research/future-crates.md` at `8607015`](https://github.com/kalonji-tools/purba/blob/8607015ff23c21b1c41fea9d17c6f693502f6679/research/future-crates.md), the hand measurement.
The scales are those of [a crate joins when its relief exceeds its pain](https://github.com/kalonji-tools/purba/blob/af20414/docs/decisions/a-crate-joins-when-its-relief-exceeds-its-pain.md).
The script is in [`research/crate-forecast-measure/`](crate-forecast-measure/), and `run.sh` runs it again.
This file changes no criterion, no scale and no formula.

## Summary

Yes, a script can measure blast area and the seven risks of a crate that purba does not lock, with few exceptions.
The script reads crates.io, builds the crate in a probe crate, and reads GitHub.
It measured the 41 crates in about six minutes.
Its level equals the hand-measured level in 323 of 328 cells. It gives a different level in 3 cells, and no level in 2.
No difference comes from a wrong count.
Each one comes from a source that the script does not read, or from upstream data that changed after the hand measurement.
Two parts of the scales stay with a person: "another part names the crate's types" in blast area, and "a record names when a person raises it" in owner in purba.
Relief and trust stay with a person, as the record says.

- **Six inputs agree for all 41 crates:** blast area, upkeep, maintainers, `unsafe`, API churn and owner in purba. The blast count equals the file's count for all 41. The `unsafe` count equals the file's count for 39. For `imara-diff` and `rustix`, a count differs and the level does not.
- **Upstream review agrees for 39.** `tokio` moves from 1 to 2: five of its pull requests return HTTP 404 today, so no script can count their reviews. `chumsky` gets no level, because its repository is on Codeberg.
- **Tests agree for 38.** `postcard` moves from 1 to 2, because its workflow runs the tests through `ci.sh`. `seccompiler` moves from 1 to 2, because its CI runs on Buildkite, not on GitHub Actions. `chumsky` gets no level.
- **Time moves a level.** Measured 21 hours after the file, `nix`'s API churn moves from 2 to 3, because `nix` 0.32.0 was published at 19:28 UTC on 2026-11-14.
- **Cost:** about 8.5 s for each crate. Each crate needs 2 crates.io requests at 1 per second. Each repository needs 2 GitHub GraphQL queries and 0 to 5 REST requests, with a GitHub token. The script uses no model tokens. The build runs the build scripts and proc macros of each crate.
- **A trap:** when probes share a target dir, cargo does not show the warnings of a crate that an earlier probe built. The first run gave `os_pipe` and `winnow` `unsafe` level 1 for this reason. The script now cleans the crate before each check.

## How to read this file

- **Tiers.** [A] = fetched this session, with the command in [the appendix](#appendix-commands). [B] = relayed and not re-verified. [C] = could not establish.
- **The baseline levels are [B].** They are copied from Appendix B7 of the file into [`baseline.tsv`](crate-forecast-measure/baseline.tsv). This research compares with them, and does not measure them again by hand.
- **Scope.** As the file: target `x86_64-unknown-linux-gnu`, default features, and the version the file measured ([`crates.tsv`](crate-forecast-measure/crates.tsv)). purba's pinned toolchain, nightly-2026-11-09, through `mise exec`.
- **The cut.** The script reads crates.io versions and commit history up to `2026-11-14T10:18:47Z`, the commit time of the file. [§ 5](#5-what-the-normalisations-erase) says what the cut erases. [§ 2.6](#26-one-day-later) gives the levels with the cut at the time of the run.
- **Readings.** The script uses the file's primary reading of each input ("The primary reading of each input"). It uses none of the other readings of the file's § 3.
- **The parts.** The script reads the parts of each crate from [`needs.tsv`](crate-forecast-measure/needs.tsv), which copies the file's needs table. It gives the same parts as the file for all 41 crates [A].

## 1. The commands

One command or API gives the number for each input. A short rule maps the number to the level.

| input | command | number | level |
|---|---|---|---|
| blast area | `cargo tree -e normal,build --prefix none --target x86_64-unknown-linux-gnu` in a probe crate that depends on `<crate> = "=<version>"` ([`probe.sh`](crate-forecast-measure/probe.sh)) | distinct `name version` lines, without the probe and the crate; the parts from `needs.tsv` | 3 when > 30 crates; 2 when ≥ 2 parts or > 10 crates; else 1 |
| upkeep | `GET https://crates.io/api/v1/crates/<crate>` ([`fetch-cratesio.sh`](crate-forecast-measure/fetch-cratesio.sh)) | the date of the newest version that is not yanked and not a prerelease, before the cut | 1 within 12 months; 2 within 2 years; else 3 |
| maintainers | `GET https://crates.io/api/v1/crates/<crate>/owners` | owners with `kind` `team`, and owners with `kind` `user` | 1 for a team or ≥ 3 users; 2 for 2 users; 3 for 1 user |
| upstream review | one GitHub GraphQL query: `history(first: 30, until: <cut>)` on the default branch, with `associatedPullRequests` and `reviews(states: APPROVED)` ([`review.graphql`](crate-forecast-measure/review.graphql)) | commits whose first merged pull request has an approval from a login other than its author | 1 for ≥ 27; 2 for ≥ 15; else 3 |
| tests | one GraphQL query for `.github/workflows` at the HEAD of the cut ([`workflows.graphql`](crate-forecast-measure/workflows.graphql)); `GET repos/<r>/actions/runs?head_sha=<sha>`; `GET …/runs/<id>/jobs`; `GET …/workflows/<file>/runs?per_page=10` ([`github.sh`](crate-forecast-measure/github.sh)) | a workflow whose text runs `cargo test`, `nextest`, `cargo hack … test`, `cargo miri test`, `make test` or `just … test`, on push, pull_request or merge_group, with a run in `success`, or a job in `success` whose name or step names "test"; else `#[test]` in the published source | 1 for an observed run; 2 for `#[test]` and no observed run; 3 for neither |
| `unsafe` | `RUSTC_WRAPPER=<clippy-driver> RUSTFLAGS="--force-warn unsafe_code --force-warn clippy::undocumented_unsafe_blocks" cargo check --message-format json` in the probe | warnings on the crate's own package id: "usage of an `unsafe` block", and "unsafe block missing a safety comment" | 1 for no block; 2 for blocks and no missing comment; else 3 |
| API churn | the same crates.io record | the first version of each compatibility key (`N`, `0.N`, `0.0.N`) that is not yanked and not a prerelease | for ≥ 1.0: 3 for a major release within 3 months, 2 within 12 months, else 1. For 0.x: 3 for a breaking release within 3 months, else 2 |
| owner in purba | `grep` of `.github/dependabot.yml` for `package-ecosystem: cargo`, of `.github` and `.config` for `renovate`, and of `docs/decisions` for the crate's name in backticks ([`levels.sh`](crate-forecast-measure/levels.sh)) | a bot entry; the records that name the crate | 3 when there is no bot and no record names the crate; otherwise no level, and a person reads the record |

[`levels.sh`](crate-forecast-measure/levels.sh) holds each rule, except the rules for upkeep and API churn, which are in [`versions.jq`](crate-forecast-measure/versions.jq). [`compare.sh`](crate-forecast-measure/compare.sh) compares the levels with the file's.

## 2. The comparison

### 2.1 Agreement for each input

With the cut at the file's commit time [A] ([`out/compare-cut-baseline.tsv`](crate-forecast-measure/out/compare-cut-baseline.tsv)):

| input | agree | differ | no level |
|---|:-:|:-:|:-:|
| blast area | 41 | 0 | 0 |
| upkeep | 41 | 0 | 0 |
| maintainers | 41 | 0 | 0 |
| upstream review | 39 | 1 | 1 |
| tests | 38 | 2 | 1 |
| `unsafe` | 41 | 0 | 0 |
| API churn | 41 | 0 | 0 |
| owner in purba | 41 | 0 | 0 |
| **all** | **323** | **3** | **2** |

### 2.2 The 41 crates

Each cell is the level. A bold cell is a difference: the file's level, then the script's.

| crate @ version | blast | upkeep | maint. | review | tests | unsafe | churn | owner |
|---|---|---|---|---|---|---|---|---|
| `anstream` 1.0.0 | 1 | 1 | 1 | 3 | 1 | 3 | 2 | 3 |
| `bpaf` 0.9.29 | 1 | 1 | 3 | 3 | 1 | 1 | 2 | 3 |
| `chumsky` 0.14.1 | 2 | 1 | 1 | **3 → none** | **1 → none** | 2 | 3 | 3 |
| `clap` 4.6.7 | 1 | 1 | 1 | 3 | 1 | 1 | 1 | 3 |
| `difflib` 0.4.0 | 2 | 3 | 3 | 3 | 2 | 1 | 2 | 3 |
| `encoding` 0.2.33 | 1 | 3 | 3 | 3 | 2 | 3 | 2 | 3 |
| `encoding_rs` 0.8.44 | 1 | 1 | 3 | 3 | 1 | 3 | 2 | 3 |
| `gag` 1.0.0 | 2 | 3 | 3 | 3 | 2 | 1 | 1 | 3 |
| `globset` 0.4.21 | 1 | 1 | 3 | 3 | 1 | 1 | 2 | 3 |
| `ignore` 0.4.34 | 2 | 1 | 3 | 3 | 1 | 1 | 2 | 3 |
| `imara-diff` 0.2.0 | 2 | 2 | 3 | 2 | 1 | 3 | 2 | 3 |
| `junit-report` 0.9.1 | 2 | 1 | 3 | 3 | 1 | 1 | 2 | 3 |
| `lexopt` 0.3.2 | 1 | 1 | 3 | 3 | 1 | 1 | 2 | 3 |
| `nix` 0.31.3 | 2 | 1 | 1 | 2 | 1 | 3 | 2 | 3 |
| `nom` 8.0.0 | 1 | 2 | 3 | 2 | 2 | 3 | 1 | 3 |
| `normpath` 1.5.2 | 1 | 1 | 3 | 3 | 1 | 2 | 1 | 3 |
| `os_pipe` 1.2.3 | 1 | 2 | 3 | 3 | 1 | 3 | 1 | 3 |
| `owo-colors` 4.4.0 | 1 | 1 | 2 | 3 | 1 | 1 | 1 | 3 |
| `path-clean` 1.0.1 | 1 | 3 | 3 | 3 | 2 | 1 | 1 | 3 |
| `pep508_rs` 0.9.2 | 3 | 2 | 3 | 3 | 1 | 1 | 2 | 3 |
| `pest` 2.9.6 | 2 | 1 | 1 | 2 | 1 | 1 | 1 | 3 |
| `petgraph` 0.8.3 | 1 | 2 | 1 | 3 | 1 | 3 | 2 | 3 |
| `pico-args` 0.5.0 | 1 | 3 | 3 | 3 | 2 | 1 | 2 | 3 |
| `polling` 3.11.0 | 1 | 2 | 1 | 2 | 1 | 3 | 1 | 3 |
| `postcard` 1.1.3 | 2 | 2 | 3 | 2 | **1 → 2** | 2 | 1 | 3 |
| `pyo3-async-runtimes` 0.30.0 | 2 | 1 | 3 | 1 | 1 | 1 | 3 | 3 |
| `pyproject-toml` 0.13.7 | 3 | 2 | 1 | 3 | 1 | 1 | 2 | 3 |
| `quick-junit` 0.8.0 | 2 | 1 | 2 | 3 | 1 | 1 | 3 | 3 |
| `quick-xml` 0.42.0 | 1 | 1 | 1 | 2 | 1 | 1 | 3 | 3 |
| `rmp-serde` 1.3.1 | 2 | 1 | 2 | 3 | 1 | 1 | 1 | 3 |
| `rustix` 1.1.6 | 2 | 1 | 3 | 3 | 1 | 3 | 1 | 3 |
| `seccompiler` 0.5.0 | 1 | 2 | 1 | 1 | **1 → 2** | 2 | 2 | 3 |
| `serde_json` 1.0.154 | 2 | 1 | 1 | 2 | 1 | 3 | 1 | 3 |
| `similar` 3.2.1 | 2 | 1 | 3 | 3 | 1 | 1 | 2 | 3 |
| `supports-color` 3.0.2 | 1 | 2 | 3 | 3 | 2 | 1 | 1 | 3 |
| `textcode` 0.5.1 | 1 | 1 | 3 | 3 | 2 | 3 | 3 | 3 |
| `tokio` 1.54.1 | 1 | 1 | 1 | **1 → 2** | 1 | 3 | 1 | 3 |
| `toml` 1.2.1+spec-1.1.0 | 1 | 1 | 1 | 3 | 1 | 1 | 2 | 3 |
| `toml_edit` 0.25.19+spec-1.1.0 | 1 | 1 | 1 | 3 | 1 | 1 | 2 | 3 |
| `walkdir` 2.5.0 | 1 | 3 | 3 | 3 | 1 | 1 | 1 | 3 |
| `winnow` 1.0.5 | 1 | 1 | 3 | 3 | 1 | 3 | 2 | 3 |

The numbers behind each level are in [`out/evidence-cut-baseline.md`](crate-forecast-measure/out/evidence-cut-baseline.md).

### 2.3 Each difference and its cause

| crate | input | file → script | pain, file → script | cause | tier |
|---|---|---|---|---|---|
| `tokio` | review | 1 (29/30) → 2 (25/30) | 1.29 → 1.36 | Five commits in the window have no merged pull request in GraphQL or in REST. Their subjects name #8651, #8674, #8690, #8743 and #8747, and `GET repos/tokio-rs/tokio/pulls/<n>` returns HTTP 404 for each. The control #8761 returns `closed`, merged. The five commits are ancestors of `ab0814fc8c`, the HEAD the file read (`compare/<sha>...ab0814fc8c` gives `ahead`), so the file read them. The file's window holds those 5, 24 commits that are reviewed today, and `63fd694966` (#8736), which is reviewed today. The file counts 29, so it counted four of the five reviewed. With four of them, the script would count 29 too | 404s, ancestry, today's verdicts [A]; "four of the five", by subtraction, because the file keeps no verdict for each commit [C]; why GitHub no longer serves the pull requests [C] |
| `postcard` | tests | 1 → 2 | 2.00 → 2.07 | `ci.yml` runs on push and pull_request, and its test step is `run: ./ci.sh`. The test command is in `ci.sh`, which the script does not read. So it finds no test workflow, and falls back to the 67 `#[test]` of the published source | [A] |
| `seccompiler` | tests | 1 → 2 | 1.36 → 1.43 | The HEAD of the cut (`c05f207989`) has no `.github/workflows` tree. The file says its CI is Buildkite, through the `rust-vmm-ci` submodule [B]. The script reads GitHub Actions only, and falls back to the 32 `#[test]` | [A] |
| `chumsky` | review, tests | 3 → none, 1 → none | — | The crates.io `repository` is `https://codeberg.org/zesterer/chumsky`. The script reads GitHub only. The file mapped Codeberg commits to pull requests through each merge commit, with a script of its own (Appendix B3, `chumsky-map.sh`) [B] | [A] |

The new `tokio` commit `f5c5b57dde` (2026-11-14T09:54:58Z, #8761, reviewed) is before the cut. It enters the window, and `63fd694966`, also reviewed, leaves it. This does not change the count. The five 404s do [A].

### 2.4 Counts that differ under the same level

| crate | input | file | script | cause | tier |
|---|---|---|---|---|---|
| `imara-diff` | `unsafe` | 9 blocks, 8 without SAFETY | 9 blocks, 9 without | `src/sources.rs:63` has `// safety invariant: this struct may only contain valid utf8` two lines above it. The file's `/safety/i` window accepts it. clippy's lint asks for `SAFETY:` | [A] |
| `rustix` | `unsafe` | 31 blocks, 14 without | 29 blocks, 14 without | rustc reports 29 blocks for the target. The 14 without SAFETY are in `src/backend/linux_raw/io/syscalls.rs` in both. Which 2 blocks the file's walker counted and rustc did not report | [C] |

Every other count agrees [A]: the blast count of all 41; the `unsafe` counts of the other 39, among them `tokio` (28 blocks, 13 without, as the file's red-team found through rustc's expansion), `encoding_rs` (144, 110) and `anstream` (2, 2); and the review counts of all GitHub repositories except `tokio`'s.

### 2.5 Agreement by another path

`supports-color` agrees at tests level 2, but not for the file's reason.
Its `miri.yml` runs `actions-rs/cargo@v1` with `command: miri` and `args: test`, on two lines. The script's pattern does not match it, so the script finds no test workflow and falls back to `#[test]` [A].
The file found the workflow, and no run [B]. The level is the same.

### 2.6 One day later

With the cut at `2026-11-15T07:40:23Z`, the time of the second run [A] ([`out/compare-cut-now.tsv`](crate-forecast-measure/out/compare-cut-now.tsv)):

| what moved | from | to |
|---|---|---|
| `nix` API churn | 2 | **3**: 0.32.0 was published at `2026-11-14T19:28:01Z`, a breaking 0.x release within 3 months. Pain 1.93 → 2.00 |
| `nix` review | 24/30 | 23/30, level 2 still |
| `petgraph` review | 1/30 | 0/30, level 3 still |
| the window of `tokio` and `clap` | | moved, level the same |

A forecast run on 2026-11-15 would also pick `nix` 0.32.0 as the newest stable version. Nobody measured its blast area or `unsafe` [C].

### 2.7 Controls

- **`unsafe`.** Three path crates in [`control.sh`](crate-forecast-measure/control.sh): no `unsafe`, one block with `// SAFETY:`, and one bare block with an `unsafe fn` and an `unsafe impl`. The two forced lints give one "usage of an `unsafe` block" to the second and third, and one "unsafe block missing a safety comment" to the third only. The `unsafe fn` and `unsafe impl` get messages of their own, so the script does not count them as blocks [A] ([`out/control-warnings.tsv`](crate-forecast-measure/out/control-warnings.tsv)).
- **Review: GraphQL against REST.** [`rest-control.sh`](crate-forecast-measure/rest-control.sh) applies the file's REST rule (Appendix B3, `measure.sh`) to the same 30 commits of `nix-rust/nix`, `smol-rs/polling` and `serde-rs/json`. The verdict is the same for 90 of 90 commits [A]. For the five `tokio` commits, REST also finds no pull request [A].
- **Tests.** For `os_pipe`, the script finds the run the file cites, [run 20493330400](https://github.com/oconnor663/os_pipe.rs/actions/runs/20493330400), a fork pull request run, because no run exists at HEAD [A].

### 2.8 A trap found while measuring

The first run gave `os_pipe` and `winnow` `unsafe` level 1, with 0 blocks [A].
All probes shared one target dir. The `toml` probe had built `winnow` already, and a test run had built `os_pipe`.
cargo did not print the warnings of those cached registry crates again [A]. Why cargo does not replay them is [C].
`probe.sh` now runs `cargo clean -p <crate>@<version>` before each check. Then both crates give 3, as the file does [A].
`cargo clean` refuses a target dir that has no `CACHEDIR.TAG`, so cargo must make the dir itself [A].
A forecast script that shares a target dir must do the same, or use one target dir for each crate.

## 3. The cost

Measured on one machine with 22 CPUs [A].

| step | requests | limit | token | time for 41 crates |
|---|---|---|---|---|
| crates.io API | 2 for each crate: 82 | 1 request per second, from the crates.io data access policy [B]. The script waits 1.1 s | none. A User-Agent that names the project, never a person | 126 s |
| download and build | cargo's own index and `.crate` downloads | — | none | 108 s, cold, one shared target dir (695 MB). The slowest crate is `pyo3-async-runtimes`, 10 s. Alone in a cold target dir: `lexopt` 1 s, `pyproject-toml` 9 s, `pyo3-async-runtimes` 11 s |
| GitHub | 2 GraphQL queries for each of the 38 repositories (76), and 47 REST requests: 31 runs at a HEAD, 12 job lists, 4 workflow run lists | `gh api rate_limit` gives 5,000 an hour for REST and for GraphQL | a GitHub token. GraphQL refuses a request without one, with HTTP 403. 7 repositories need no REST request, because they have no test workflow | 116 s at the first cut, 101 s at the second |
| purba's tree | none | — | none | < 1 s |

**For each crate:** about 8.5 s in sequence (350 s for 41), 2 crates.io requests, and about 3 GitHub requests.
A second run of `run.sh` from an empty directory took 333 s for all 41, and gave the same levels [A].
**No model tokens:** the script calls no model.
**The build runs code.** It runs the build scripts and proc macros of each crate and of what the crate brings. So the script runs only in a scratch directory.
**`pyo3-async-runtimes` needs Python** for `pyo3`'s build script. purba's `mise` sets `PYO3_PYTHON`, so the probe runs under `mise exec` [A].

## 4. What no script can give

| level | why |
|---|---|
| relief | a person's score, by the record |
| trust | a person's score, by the record. A script gives only the crates.io facts: owners, `published_by`, `trustpub_only` |
| blast area 3, "another part names the crate's types" | no part has a design, so nobody can say which part names which types. The parts come from the needs table, which a person wrote. So each blast level is a floor, as in the file |
| owner in purba 1 and 2 | level 2 needs a record that says *when* a person raises the crate. A script finds only the records that name it. The backtick search finds none. A word search finds 5 crates in records: `clap` (1 record), `encoding` (1), `ignore` (8), `nix` (1) and `toml` (47), most of them as common words. The one record that names `clap` says why command does not use it. Only a person can read whether a record says when someone raises the crate. Level 1 needs a bot that proposes each version, and a bot entry shows only that a bot is set up [B, research/crate-gate-tools.md § 3.8] |
| a state in the past | upstream data has no history: pull requests go away (`tokio`), owners change, and old workflow runs expire. A script gives the level on the day it runs. Run again, it cannot give the file's level from that day |

These levels need more work, but a script can give them:

| level | what the script needs |
|---|---|
| tests, for CI outside GitHub Actions or behind a shell script | a reader for each CI (Buildkite, Codeberg's Forgejo Actions) and for the scripts a workflow calls. `seccompiler` and `postcard` show both cases |
| review on Codeberg | a mapping from commits to pull requests through merge commits, as the file's `chumsky-map.sh` does [B] |
| `unsafe` on macOS | purba runs on Linux and macOS ([purba runs on Linux and macOS](https://github.com/kalonji-tools/purba/blob/af20414/docs/decisions/purba-runs-on-linux-and-macos.md)) [B]. The probe builds for `x86_64-unknown-linux-gnu` only, as the file measured. Another target needs its standard library, and a crate with C code needs that target's C toolchain [C] |

## 5. What the normalisations erase

| normalisation | what it erases |
|---|---|
| the cut at the file's commit time, `2026-11-14T10:18:47Z` | each version and commit after it: `nix` 0.32.0 and every later commit. It does not erase what has no date: today's owners, today's pull requests and reviews, and the runs GitHub keeps today. So the cut makes the time of upkeep, churn and the commit window equal to the file's, and nothing else |
| the cut by `committedDate` | a commit that reached the branch after the cut with an older committer date. No such case moved a level here [C] |
| the version of the file, not the newest | a version published after the file: `nix` 0.32.0 |
| the lock of each probe is resolved today | a newer version of a crate the crate brings. `os_pipe`'s probe resolves `libc` 0.2.191 today [A]. The file does not give the versions its probes resolved [C]. The count of each of the 41 is unchanged [A] |
| the parts from `needs.tsv` | nothing that the file has, but the parts are an input, not a measurement |

## Appendix: Commands

`$WT` is a worktree of purba at `af20414`, on the branch `research/crate-forecast-measure`. `$W` is a scratch directory. Every script is in `$WT/research/crate-forecast-measure/` (`$D`).

```sh
# inputs: the baseline levels and the crate list, from the file's Appendix B7
awk '/^pain <<.LEVELS./{f=1;next} /^LEVELS$/{f=0} f && $1!~/^#/ {print $1"\t"$2"\t"$3"\t"$4"\t"$5"\t"$6"\t"$7"\t"$8"\t"$9"\t"$10}' future-crates.md
# needs.tsv: typed from the file's § 1 and "Inferred needs"

# crates.io: 2 requests for each crate, 1.1 s apart, User-Agent 'purba-research (https://github.com/kalonji-tools/purba)'
bash $D/fetch-cratesio.sh $W/cratesio $(grep -v '^#' $D/crates.tsv | cut -f1)        # 126 s
jq -r '.versions[] | select(.created_at >= "2026-11-13") | [.crate,.num,.created_at] | @tsv' $W/cratesio/*.json

# the unsafe control, then the probes (blast area, unsafe, the published source)
cd $WT && mise exec -- bash $D/control.sh $W
cd $WT && mise exec -- bash $D/run-probes.sh $W $D/crates.tsv                          # 108 s, cold
cd $WT && mise exec -- bash $D/probe.sh $W/cold-a pyproject-toml 0.13.7                # 9 s alone; also pyo3-async-runtimes 11 s, lexopt 1 s

# GitHub: review and tests, at the file's commit time and at the time of the run
export GH_TOKEN=$(gh auth token --user <agent account>)
bash $D/run-github.sh $W 2026-11-14T10:18:47Z                                          # 116 s
bash $D/run-github.sh $W/now 2026-11-15T07:40:23Z                                      # 101 s
gh api rate_limit --jq '.resources | {core, graphql}'

# levels and the comparison
bash $D/levels.sh $W 2026-11-14T10:18:47Z $WT > $W/levels.tsv && bash $D/compare.sh $W/levels.tsv
bash $D/levels.sh $W/now 2026-11-15T07:40:23Z $WT > $W/now/levels.tsv && bash $D/compare.sh $W/now/levels.tsv

# controls and causes
bash $D/rest-control.sh $W nix-rust/nix all; bash $D/rest-control.sh $W smol-rs/polling all; bash $D/rest-control.sh $W serde-rs/json all
bash $D/rest-control.sh $W tokio-rs/tokio nopr
for n in 8651 8674 8690 8743 8747 8761; do gh api repos/tokio-rs/tokio/pulls/$n --jq '"\(.state) merged=\(.merged_at != null)"'; done
gh api repos/tokio-rs/tokio/commits/f5c5b57dde --jq '{parents: [.parents[].sha[0:10]], committer: .commit.committer.date}'
gh api "repos/tokio-rs/tokio/compare/c6a62327ff...ab0814fc8c" --jq .status            # ahead
gh api "repos/tokio-rs/tokio/commits?sha=ab0814fc8c&per_page=30"                       # the file's window; the 30th is 63fd694966
gh api repos/tokio-rs/tokio/commits/63fd694966/pulls; gh api --paginate repos/tokio-rs/tokio/pulls/8736/reviews   # 1 approval from another login
curl -s -o /dev/null -w '%{http_code}' -X POST https://api.github.com/graphql -d '{"query":"{ viewer { login } }"}'   # 403
# blast: the parts and the count of each crate, against the file's evidence table (lines 307-347)
awk -F' [|] ' 'NR > 2 {print $1, $2}' $D/out/evidence-cut-baseline.md
jq -r '.data.repository.object.entries[]? | "== \(.path)\n\(.object.text)"' $W/github/jamesmunns_postcard/workflows.json
jq -r '.data.repository.object' $W/github/rust-vmm_seccompiler/workflows.json          # null
sed -n '58,65p' <imara-diff-0.2.0>/src/sources.rs
awk -F'\t' '$1=="clippy::undocumented_unsafe_blocks"' $W/probe/rustix/warnings.tsv

# the owner reading
grep -rlw -- <crate> $WT/docs/decisions; grep -rlF -- '`<crate>`' $WT/docs/decisions

# pain of the levels that moved (the formula of the file's B7)
awk '{r=$3+$4+$5+$6+$7+$8+$9; printf "%s %.2f\n", $1, ($2+r/7)/2}'
```

`run.sh` runs the measuring steps in order. The check that the scripts as committed give the levels in `out/`:

```sh
cd $WT && bash $D/run.sh $W/final 2026-11-14T10:18:47Z && diff $D/out/levels-cut-baseline.tsv $W/final/levels.tsv
```

The pre-commit gates of purba (shellcheck with every optional check, shfmt, editorconfig-checker, typos) pass on every file here. Nothing here wrote to GitHub, except the comment on the issue.
