# The criteria of a crate that a tool can decide, and the tools that decide each

Research for [#975](https://github.com/kalonji-tools/purba/issues/975).
Measured on 2026-11-14, against purba `main` at `2790c1e`.
The criteria are the four checks and nine inputs of [the #959 spec](https://github.com/kalonji-tools/purba/issues/959#issuecomment-6874200802), and the waiver of [the #969 spec](https://github.com/kalonji-tools/purba/issues/969#issuecomment-6877037346).
This file picks no tool and recommends none. It gives the facts a person needs to decide.
It does not decide when a crate is judged again. That is [#977](https://github.com/kalonji-tools/purba/issues/977).

## Summary

Three of the four checks have a tool that gives the verdict: known advisory, licence and source.
For known advisory, a person must first fix which advisory categories count, because the tools differ on an informational advisory [C].
`cargo-deny` gives all three, and purba already pins it, but `.config/tasks.toml` runs only `cargo-deny check licenses`.
`cargo-deny --manifest-path` refuses a cargo script, but `cargo-deny --metadata-path` reads the output of `cargo -Zscript metadata` and checks the script's graph [A]. `cargo-audit` and `osv-scanner` read any `Cargo.lock`.
So the three crates of `scripts/check-prose` reach no gate because no task runs a check on them, not because the tools cannot see them.
**Today the tools find two check failures in that script.** RustSec names `pulldown-cmark` 0.13.4 in two advisories. `harper-core` and `harper-pos-utils` come from a git repository.
No input of the nine has a tool that gives its level with no mapping.
For six inputs, a tool gives a number that a script must map to the level: blast area, upkeep, maintainers, upstream review, `unsafe` and API churn. Each number differs from the spec's wording somewhere, and § 3 says where.
For `unsafe`, rustc's `unsafe_code` lint and clippy's `undocumented_unsafe_blocks` lint can be forced onto dependencies, and then they read exactly what purba compiles. Two probes show cases where their warnings map to a level the spec does not give (§ 3.6).
Tests and owner in purba get part of their level from a tool. Trust gets its crates.io half from a tool. Relief, and the "project meant" half of trust, get nothing from any tool surveyed.
Each of the three waiver refusals needs only data that `.config/crates.toml` and the check tools already give, so a script could decide them [C]. `cargo-deny` has a form of the first refusal for advisories and sources, but its lists cover the whole graph, not one crate. It has no form of it for `"score"`.

### Grades

- **[A]** primary source, and probed this session with a control, by this file's author or by its red team. Each probe of the author is in [Appendix B](#appendix-b-commands); a red-team probe is named as one.
- **[B]** primary source, not probed.
- **[C]** inferred. [§ 6](#6-what-is-inferred-c) lists every [C].

### The summary table

"Decides" means the tool's output is the verdict or the level, with no mapping.
"Number" means the tool gives a count, a date, a field or a set of warnings, and a script must map it to the level.
"Part" means the tool gives only some of what the level needs.

| criterion | decides | number | part | what no tool surveyed sees |
|---|---|---|---|---|
| known advisory | `cargo-deny check advisories` [A], `cargo-audit` [A], each once a person fixes which advisory categories count [C] | — | `osv-scanner` [A], deps.dev `advisoryKeys` [A] | an advisory on a git-sourced crate, in `cargo-deny` and `cargo-audit` [A] |
| licence | `cargo-deny check licenses` (gated today) [A] | — | `osv-scanner --licenses` [A], deps.dev `licenses` [A] | which crate reaches the wheel: `include-dev = true` also checks `proptest` [B] |
| source | `cargo-deny check sources`, also on a cargo script through `--metadata-path` [A] | `cargo metadata` `source` field [A] | — | — |
| trust | the malicious-code notice: RustSec `categories = ["malicious"]`, through `cargo-audit` [A] or `cargo-deny` [B] | crates.io owners, `published_by`, `trustpub_data` [A] | `cargo-vet` trusted entries [B], `cargo-crev` [B], deps.dev `SOURCE_REPO` [A] | whether the name, repository and publisher are the project meant [C] |
| blast area | — | `cargo metadata` / `cargo tree`: the parts that declare it, the crates it brings [A] | "another part names its types": `grep` [A], `cargo-check-external-types` (public API only) [B]; deps.dev `:dependencies` [A] | — |
| upkeep | — | crates.io `versions[].created_at` [A], deps.dev `publishedAt` [A] | Scorecard `Maintained` [A], `cargo-unmaintained` [B], RustSec `unmaintained` [B] | — |
| maintainers | — | crates.io `/owners`, with `kind` user or team [A]; `cargo-supply-chain` [A] | crates.io Trusted Publishing configurations need a token [B] | whether a user owner is an organisation account [C] |
| upstream review | — | Scorecard `Code-Review`: "Found X/Y approved changesets" [A] | its unit is a changeset, and it counts an implicit review [B] | — |
| tests | — | — | Scorecard `CI-Tests`: "N out of M merged PRs checked by a CI test" [A] | level 2 (tests, and no CI) apart from level 3 (no tests) [C] |
| `unsafe` | — | rustc `unsafe_code` + clippy `undocumented_unsafe_blocks`, forced onto dependencies: warnings that a script maps to the level [A]; the mapping [C] | `cargo-geiger` [A]; a source `grep` [A] | — |
| API churn | — | crates.io version list with dates [A]; deps.dev versions [B] | `cargo-semver-checks` [B] | — |
| owner in purba | — | — | level 1: a script that reads `.github/dependabot.yml` can find a cargo entry, which does not show that a bot proposes each version [C]; Scorecard `Dependency-Update-Tool` [B]; Dependabot and Renovate read no cargo script [B] | level 2: whether a record names when a person raises the crate [C] |
| relief | — | — | crates.io `linecounts` counts the crate's lines, not the lines saved [B] | relief itself |
| waiver names a failure the crate does not have | — | — | `cargo-deny` `unused-ignored-advisory = "deny"` for an advisory id [A]; `unused-allowed-source = "deny"` for a source [B]. Both lists cover the whole graph, not one crate, and live in `deny.toml`, not `.config/crates.toml` [B] | `"score"`: no tool has the formula [C] |
| entry fails something its waiver does not name | — | — | the check tools' own refusal, for an advisory or a source [A], under the same two limits [B] | `"score"` [C] |
| waiver names anything but an id, `"source"` or `"score"` | — | — | `cargo-deny` `unknown-advisory` refuses an id that no database holds [B] | the field's shape: no tool reads `.config/crates.toml` [C] |
| a stored level gone stale | — | each number tool above, run again, against the stored level [C] | `cargo-vet` makes a trusted entry expire [B]; `cargo-deny` `maximum-db-staleness` [B] | — |

## 1. Found on the tree

These are the tools' outputs on `2790c1e`. They are facts for the person, and this file proposes no change.

| finding | tools that show it | what misses it |
|---|---|---|
| RustSec advisories `RUSTSEC-2026-0414` and `RUSTSEC-2026-0433` name `pulldown-cmark` 0.13.4, locked in `scripts/check-prose/Cargo.lock`. Each is `categories = ["denial-of-service"]`, and the patched versions are `>= 0.13.5` and `>= 0.13.6` ([0414](https://github.com/rustsec/advisory-db/blob/4ef2df8e806a65e2f1ecc14f56f51249062b23d6/crates/pulldown-cmark/RUSTSEC-2026-0414.md), [0433](https://github.com/rustsec/advisory-db/blob/4ef2df8e806a65e2f1ecc14f56f51249062b23d6/crates/pulldown-cmark/RUSTSEC-2026-0433.md)). Both titles name `ENABLE_WIKILINKS`. | `cargo-deny --metadata-path`, `cargo-audit`, `osv-scanner`, deps.dev [A] | every gate: `lint:licences` is the only `cargo-deny` task, and no task runs a check on the script [A] |
| `harper-core` and `harper-pos-utils` 2.3.0 come from `git+https://github.com/Automattic/harper?tag=v2.3.0`. crates.io has both crates, at 3.2.0. Two crates they bring, `harper-brill` and `harper-thesaurus`, come from the same repository. | `cargo metadata` on the script, `cargo-deny --metadata-path … check sources` [A] | every gate: no task runs a check on the script [A] |
| The workspace lock has no advisory, and every crate in it comes from crates.io. | `cargo-deny check advisories sources` with a stricter scratch configuration, `cargo-audit`, `osv-scanner` [A] | — |

The script's lock holds 538 packages. Beyond the three it names, `cargo-audit` reports advisories on 14 more packages there. The #959 spec judges such a crate "by the gates alone" [B].

## 2. The four checks

### 2.1 Known advisory

The check passes when no RustSec advisory names the locked version.

| tool | what it reports | what it needs | what it cannot see |
|---|---|---|---|
| `cargo-deny` 0.20.2, `check advisories` | each advisory that names a locked version, by id, with exit 1 | network to fetch the advisory database, or `--offline` within `maximum-db-staleness`. No `[advisories]` section is needed: "If the `[advisories]` section is omitted entirely, the advisory check still runs using the default value documented for each field" ([cfg.md L19](https://github.com/EmbarkStudios/cargo-deny/blob/bca0dde53651ee946720e4540b5ce2610bec8f06/docs/src/checks/advisories/cfg.md#L19)) [B] | a git-sourced crate (below) [A]. A cargo script needs two steps: `--manifest-path` refuses it with "--manifest-path must point to a Cargo.toml file", exit 1, and `--metadata-path` reads the output of `cargo -Zscript metadata`. That run exits 9 with `RUSTSEC-2026-0414` and `RUSTSEC-2026-0433`; on the workspace it gives `advisories ok` [A] |
| `cargo-audit` 0.23.0 | the same database, from any `Cargo.lock` given with `-f` | network for the database | a git-sourced crate [A] |
| `osv-scanner` 2.7.1 | OSV records, which include RustSec ids, from any `Cargo.lock`. On the script's lock it gave the same 15 package versions and 32 RustSec ids as `cargo-audit` [A] | network to osv.dev | the source of a package (below) [A] |
| deps.dev API v3 | `advisoryKeys` of one version: `RUSTSEC-2026-0414` and `RUSTSEC-2026-0433` for `pulldown-cmark` 0.13.4 [A] | network | one request per crate and version; it reads no lock |
| Scorecard `Vulnerabilities` | open vulnerabilities in the upstream project's own tree, "using the [OSV (Open Source Vulnerabilities)](https://osv.dev/) service" ([checks.md L697-L700](https://github.com/ossf/scorecard/blob/61e84743356275f6ffca23b1b2f2cb945e19eb96/docs/checks.md#L697-L700)) [B] | a GitHub token | the version purba locks |

**A git-sourced crate.** `rustsec` 0.34.0, which `cargo-audit` uses, matches an advisory only when the package's source equals the advisory's. An advisory with no source takes the default, crates.io:

```rust
// TODO(tarcieri): better source comparison?
if advisory_source.kind() != package_source.kind()
    || advisory_source.url() != package_source.url()
{
    return false;
}
```

([query.rs L188-L193](https://github.com/rustsec/rustsec/blob/e3cf3703ed640854af152d019c976ae78211aa22/rustsec/src/database/query.rs#L188-L193)) [A].
`cargo-deny` does not use the `rustsec` crate. It makes the same comparison in its own code: "// Validate the crate's source is the same as the advisory" ([db.rs L218-L221](https://github.com/EmbarkStudios/cargo-deny/blob/bca0dde53651ee946720e4540b5ce2610bec8f06/src/advisories/helpers/db.rs#L218-L221)) [A].
The probe: a git repository holding a crate named `smallvec` at version 1.6.0. `cargo-audit` and `cargo-deny` both passed it. Both refuse `smallvec` 1.6.0 from crates.io with `RUSTSEC-2021-0003`.
`osv-scanner` matches by name and version alone. It reported `RUSTSEC-2021-0003` on the git crate, with the ecosystem "crates.io" [A].
So on a git-sourced crate, `cargo-deny` and `cargo-audit` can miss a real advisory, and `osv-scanner` can report one that is not real.

**Which advisories count.** RustSec has informational advisories: `"unsound"`, `"unmaintained"` and `"notice"` ([README.md L73-L77](https://github.com/rustsec/advisory-db/blob/4ef2df8e806a65e2f1ecc14f56f51249062b23d6/README.md#L73-L77)) [B].
By default `cargo-deny` fails on `unmaintained = "all"` and on `unsound = "workspace"` ([cfg.md L12-L13](https://github.com/EmbarkStudios/cargo-deny/blob/bca0dde53651ee946720e4540b5ce2610bec8f06/docs/src/checks/advisories/cfg.md#L12-L13)) [B]. `"workspace"` means "a direct dependency of one or more workspace crates" ([cfg.md L105](https://github.com/EmbarkStudios/cargo-deny/blob/bca0dde53651ee946720e4540b5ce2610bec8f06/docs/src/checks/advisories/cfg.md#L105), [L118](https://github.com/EmbarkStudios/cargo-deny/blob/bca0dde53651ee946720e4540b5ce2610bec8f06/docs/src/checks/advisories/cfg.md#L118)) [B]. Whether each crate the spec judges counts as one is not established: the three script crates are not in purba's workspace [C].
`cargo-audit` lists unmaintained and unsound advisories as warnings and exits 0 on them alone. On the script's lock it exited 1 for vulnerabilities and listed 15 warnings [A].
So the two tools give different verdicts on one lock. Through `--metadata-path`, `cargo-deny` gave `error[unmaintained]` for `bincode` and `paste`, and `cargo-audit` listed the same two only as warnings [A]. Both tools also judge the whole graph, not only the named crate [A]. Each gives the check's verdict only once a person fixes which categories count [C].
`yanked` defaults to `"warn"` in `cargo-deny` ([cfg.md L64-L70](https://github.com/EmbarkStudios/cargo-deny/blob/bca0dde53651ee946720e4540b5ce2610bec8f06/docs/src/checks/advisories/cfg.md#L64-L70)) [B]. The spec's check names an advisory and not a yank, so a yank does not fail it [C].

### 2.2 Licence

The check passes when the licence is on the allow list of `.cargo/deny.toml`, or that file has an exception for the crate. Only a crate that reaches the wheel is checked.

| tool | what it reports | what it cannot see |
|---|---|---|
| `cargo-deny check licenses` | the verdict, against `allow` and `exceptions`. `lint:licences` runs it today. A crate licensed `GPL-3.0-only` drew `error[rejected]` under purba's `allow` list [A] | a cargo script through `--manifest-path`; it needs `--metadata-path`, as in § 2.1. The spec checks no script crate's licence. Dev dependencies: purba sets `include-dev = true`, and the docs say "By default this is false as `dev-dependencies` are not used by downstream crates, nor part of binary artifacts" ([licenses/cfg.md L46](https://github.com/EmbarkStudios/cargo-deny/blob/bca0dde53651ee946720e4540b5ce2610bec8f06/docs/src/checks/licenses/cfg.md#L46)) [B]. So `proptest`, which never reaches the wheel, is checked |
| `osv-scanner --licenses="MIT"` | 16 packages of the workspace lock as violations. They include the seven crates of the `exceptions` list and purba's own crates as `UNKNOWN` [A] | the per-crate exceptions of `.cargo/deny.toml` |
| deps.dev API v3 | the `licenses` of one version: `["MIT"]` for `pulldown-cmark` 0.13.4 [A] | no allow list |

### 2.3 Source

The check passes when the crate comes from crates.io, not from a git repository.

| tool | what it reports | what it needs | what it cannot see |
|---|---|---|---|
| `cargo-deny check sources` | `error[source-not-allowed]: detected 'git' source not explicitly allowed` [A] | a `[sources]` section: `unknown-git` defaults to `warn`, and `deny` "Will emit an error with the URL of the repository, and fail the check" ([sources/cfg.md L19-L25](https://github.com/EmbarkStudios/cargo-deny/blob/bca0dde53651ee946720e4540b5ce2610bec8f06/docs/src/checks/sources/cfg.md#L19-L25)) [B]. `unknown-registry` also defaults to `warn`. For a cargo script, `--metadata-path` with the output of `cargo -Zscript metadata`: it gave four `error[source-not-allowed]`, for `harper-core`, `harper-pos-utils`, `harper-brill` and `harper-thesaurus`, and `sources ok` on the workspace [A] | a cargo script through `--manifest-path` [A] |
| `cargo metadata` | each package's `source`, such as `git+https://github.com/Automattic/harper?tag=v2.3.0#d0ae67ed…` [A] | for a cargo script: `-Zscript` and `--config 'resolver.lockfile-path="…/Cargo.lock"'`, as `.config/tasks.toml` already passes. Without the setting, `--locked` fails [A] | — |
| the `Cargo.lock` itself | the same `source` line [A] | — | which crate a manifest names, and which arrive through others |

### 2.4 Trust

The check passes when a person confirms that the name, the repository and the publisher are the project meant, and crates.io shows no notice of malicious code.

**The malicious-code notice.** crates.io's security tab fetches its advisories from RustSec: ``fetch(`https://rustsec.org/packages/${crateId}.json`)`` ([rustsec.ts L107-L108](https://github.com/rust-lang/crates.io/blob/3203d67d3d53575b31438352a447898ffbf72298/svelte/src/lib/utils/rustsec.ts#L107-L108)) [B].
RustSec records a malicious crate as an advisory. Its newest example is `categories = ["malicious"]` with `expect-deleted = true`, titled "`shinchina-test` was removed from crates.io for malicious code" ([RUSTSEC-2026-0472](https://github.com/rustsec/advisory-db/blob/4ef2df8e806a65e2f1ecc14f56f51249062b23d6/crates/shinchina-test/RUSTSEC-2026-0472.md)) [B].
The database at `4ef2df8e` holds 183 files under `crates/` that contain the word "malicious" [A].
So `cargo-deny check advisories` and `cargo-audit` read this half of the check, as they read § 2.1. For `cargo-audit` a red-team probe showed it: a hand-written lock that holds `shinchina-test` 0.1.0 gave `RUSTSEC-2026-0472` and exit 1, and a control name gave exit 0 [A]. `cargo-deny` was not probed for this [B]. A report goes to crates.io, not RustSec: "please see the [Crates.io Security Policy](https://crates.io/policies/security) to get content violating crates.io's policies taken down" ([CONTRIBUTING.md L91-L93](https://github.com/rustsec/advisory-db/blob/4ef2df8e806a65e2f1ecc14f56f51249062b23d6/CONTRIBUTING.md#L91-L93)) [B].

**Name, repository and publisher.** Tools give facts. None says which project the person meant [C].

| tool | what it gives | how far it can be trusted |
|---|---|---|
| crates.io `/api/v1/crates/{name}` | `repository`; each version's `published_by`, and `trustpub_data` when "published via trusted publishing (e.g., GitHub Actions) rather than a regular API token" ([OpenAPI snapshot L1440](https://github.com/rust-lang/crates.io/blob/3203d67d3d53575b31438352a447898ffbf72298/src/tests/snapshots/integration__openapi__openapi_snapshot-2.snap#L1440)) [A]. `trustpub_only`: "Whether this crate can only be published via Trusted Publishing" ([L418](https://github.com/rust-lang/crates.io/blob/3203d67d3d53575b31438352a447898ffbf72298/src/tests/snapshots/integration__openapi__openapi_snapshot-2.snap#L418)) [B] | `trustpub_data` names a repository, a run and a commit, as the probe shows for `pyo3` 0.30.1: `{"provider":"github","repository":"PyO3/pyo3","run_id":"38639289091","sha":"f1685386…"}` [A]. The field's status is "**Unstable**" [B] |
| deps.dev `relatedProjects` | `SOURCE_REPO` `github.com/raphlinus/pulldown-cmark`, with `relationProvenance` `UNVERIFIED_METADATA` [A] | deps.dev itself marks it unverified |
| `cargo-vet` 0.10.3 | records a person's trust in a publisher: "Entries require a trust expiration date, which ensures that the judgment is revisited periodically" ([trusting-publishers.md](https://github.com/mozilla/cargo-vet/blob/cef4dd43a0b9d2d37febf0e5ad42abc6d9ea5472/book/src/trusting-publishers.md)) [B] | "Trust entries are fundamentally a heuristic. The trusted publisher is not consulted and may or may not have personally authored or reviewed all the code." (same page) [B]. It records a decision and does not make one |
| `cargo-crev` 0.27.1 | reviews by people in a web of trust: it can "use reviews produced by other users" and "build a web of trust of other reputable users" ([cargo-crev/README.md L45-L47](https://github.com/crev-dev/cargo-crev/blob/54d198b02359a612c0c5f4f8eee6ceb8b9a4169b/cargo-crev/README.md#L45-L47)) [B] | it reads reviews that other people wrote |
| Socket | "Socket supports Rust", with `Cargo.lock` "Fully supported" ([Ecosystem Support](https://docs.socket.dev/docs/language-support), updated 2026-11-05, a live page with no pinned form) [B] | which alerts it raises on a crate could not be established: its Rust page returned 404 |

## 3. The nine inputs

### 3.1 Blast area

Level 1: one part, and ≤ 10 crates it brings. Level 2: two parts, or ≤ 30. Level 3: another part names the crate's types, or > 30.

| tool | what it gives | difference from the spec |
|---|---|---|
| `cargo tree -i <crate> --depth 1` | the parts that declare the crate [A] | it counts a dev dependency when `-e dev` is passed. `pyo3` reaches `purba-command` only as a dev dependency |
| `cargo tree -p <crate> -e normal,build --prefix none`, distinct lines | the crates it brings [A] | the count follows the features that the whole workspace turns on. `ruff_text_size` brings 28 because other crates turn on its `get-size2` features [A]. For a dev-only crate, `-e normal,build` prints "nothing to print"; `proptest` brings 14 under `-e normal,build,dev` [A] |
| deps.dev `:dependencies` | 20 nodes for `pulldown-cmark` 0.13.4 [A] | its direct nodes include `bincode`, `serde` and `serde_json`, which purba's script does not compile. The script's own tree gives 7 [A] |
| `grep` for `\b<crate>::` or `use <crate>` in `crates/*/src` and `tests` | the parts whose code names a crate's paths. On today's tree it agrees with the manifests for all eight workspace crates. A control name (`smallvec::`) matched no file [A] | it finds a path, not a type. A glob import or a re-export hides the crate's name [C] |
| `cargo-check-external-types` | "set and verify which types from other libraries are allowed to be exposed in their public API" ([README](https://github.com/awslabs/cargo-check-external-types/blob/705a0941997ebeb3bfb1a7f14070ba342f79d879/README.md)) [B] | the public API only. Whether level 3 means "in the public API" is a reading of the spec [C] |

### 3.2 Upkeep

Level 1: a release within 12 months. Level 2: within 2 years. Level 3: older.

| tool | what it gives | difference from the spec |
|---|---|---|
| crates.io `/api/v1/crates/{name}` | every version with `created_at` and `yanked`. On 2026-11-14 the newest non-yanked release of `serde` was 118 days old, and of `harper-core` 0 days [A] | none for a crate on crates.io. For a git-sourced crate, the crates.io releases are not the code purba compiles [C] |
| deps.dev versions | `publishedAt` of each version [A] | the same |
| Scorecard `Maintained` | "If there is at least one commit per week during the previous 90 days, the project receives the highest score" ([checks.md L400-L410](https://github.com/ossf/scorecard/blob/61e84743356275f6ffca23b1b2f2cb945e19eb96/docs/checks.md#L400-L410)) [B]. `pulldown-cmark`: "30 commit(s) and 3 issue activity found in the last 90 days" [A] | it counts commits and issues in 90 days, not releases |
| `cargo-unmaintained` 2.1.0 | a package X is unmaintained when its repository is archived, or X "is not a member of its named repository", or both of two things hold. First, X depends on a package Y whose latest version is incompatible with the version X depends on and "was released over a year ago". Second, X has no repository, or its repository's last commit "was over a year ago" ([README L13-L19](https://github.com/trailofbits/cargo-unmaintained/blob/10d735c2b1bc3cd3001e921268eca865f8e7d845/README.md#L13-L19)) [B] | a heuristic on commits and dependencies. The archive check needs `GITHUB_TOKEN_PATH` [B] |
| RustSec `unmaintained` advisories | a person's report that a crate is unmaintained [B] | no window of time |
| `cargo-outdated` 0.19.0 | "displaying when dependencies have newer versions available" ([README L10](https://github.com/kbknapp/cargo-outdated/blob/9a25ecbfcd0a1d7394bb521e89022950bca9d7b0/README.md#L10)) [B] | no dates |

### 3.3 Maintainers

Level 1: ≥ 3 people can publish, or a team or an organisation account owns the crate. Level 2: two. Level 3: one.

| tool | what it gives | difference from the spec |
|---|---|---|
| crates.io `/api/v1/crates/{name}/owners` | each owner with `kind` `"user"` or `"team"` [A]. `serde`: `dtolnay` and `github:serde-rs:publish`. `libc`: `rust-lang-owner`. `ruff_python_parser`: `charliermarsh` and `astral-crates-bot` [A] | a user owner can be an account that a project, not a person, holds. Whether `rust-lang-owner` or `astral-crates-bot` counts as an organisation account is a reading [C] |
| `cargo-supply-chain` 0.3.8, `json` | the same owners for every crate in the graph, with `kind` [A] | it uses the crates.io API at one request per second, with the User-Agent `cargo supply-chain (https://github.com/rust-secure-code/cargo-supply-chain)` ([api_client.rs L22-L27](https://github.com/rust-secure-code/cargo-supply-chain/blob/ae1c240ace44b55ba01ac732f056eb632118e655/src/api_client.rs#L22-L27)) [B] |
| crates.io `/api/v1/trusted_publishing/github_configs` | the workflows that can publish a crate | it requires `cookie` or `api_token` security ([OpenAPI snapshot L5329-L5337](https://github.com/rust-lang/crates.io/blob/3203d67d3d53575b31438352a447898ffbf72298/src/tests/snapshots/integration__openapi__openapi_snapshot-2.snap#L5329-L5337)) [B]. A team's members are a GitHub question, which crates.io does not answer |

### 3.4 Upstream review

Level 1: ≥ 27 of the last 30 commits reviewed. Level 2: ≥ 15. Level 3: < 15.

| tool | what it gives | difference from the spec |
|---|---|---|
| Scorecard 5.6.0 CLI, `Code-Review` | `pulldown-cmark/pulldown-cmark`: "Found 9/11 approved changesets -- score normalized to 8". `Automattic/harper`: "Found 8/30 approved changesets -- score normalized to 2" [A] | its unit is a changeset, which groups commits by pull request. It looks at "the most recent changes (over the last ~30 commits)", and a change counts as reviewed when it has "an approval on GitHub/GitLab or if the merger is different from the committer (implicit review)" ([checks.md L207-L209](https://github.com/ossf/scorecard/blob/61e84743356275f6ffca23b1b2f2cb945e19eb96/docs/checks.md#L207-L209)) [B]. The #960 audit counted an `APPROVED` review from a login other than the author ([research/future-crates.md](https://github.com/kalonji-tools/purba/blob/8607015ff23c21b1c41fea9d17c6f693502f6679/research/future-crates.md)) [B]. It needs a GitHub token |
| the same, its score | — | the docs say "Scoring is leveled instead of proportional" ([checks.md L216](https://github.com/ossf/scorecard/blob/61e84743356275f6ffca23b1b2f2cb945e19eb96/docs/checks.md#L216)). The code calls `CreateProportionalScoreResult(name, f.Message, approved, total)` ([code_review.go L59](https://github.com/ossf/scorecard/blob/61e84743356275f6ffca23b1b2f2cb945e19eb96/checks/evaluation/code_review.go#L59)), and the probe's scores are proportional [A]. A script must read the `X/Y` of the reason, not the score |
| Scorecard REST API (`api.securityscorecards.dev`) | `serde-rs/serde`: "Found 5/21 approved changesets", on 2026-11-02 [A] | it holds only the repositories in its weekly crawl: `pulldown-cmark/pulldown-cmark` gave HTTP 404 [A] |
| deps.dev projects | `github.com/raphlinus/pulldown-cmark`: score 10, "all changesets reviewed", dated 2026-08-24 [A] | the key is the repository's old name, and the result is 82 days old. The CLI on the current name gave 9/11 today [A] |

### 3.5 Tests

Level 1: CI runs the tests on each change. Level 2: tests, and no CI. Level 3: no tests.

| tool | what it gives | difference from the spec |
|---|---|---|
| Scorecard `CI-Tests` | "28 out of 28 merged PRs checked by a CI test" for `Automattic/harper`, "9 out of 9" for `pulldown-cmark` [A]. It looks "for a set of CI-system names in GitHub `CheckRuns` and `Statuses` among the recent commits (~30)", and a name counts when it contains one of "appveyor, buildkite, circleci, e2e, github-actions, jenkins, mergeable, test, travis-ci, woodpecker" ([checks.md L150-L153](https://github.com/ossf/scorecard/blob/61e84743356275f6ffca23b1b2f2cb945e19eb96/docs/checks.md#L150-L153)) [B] | it reads the name of a check, not whether the check ran tests. It cannot tell level 2 from level 3, because it reads only CI [C]. "It is currently limited to repositories hosted on GitHub" (same, L142-L143) [B] |
| GitHub check-runs API | the run, job and conclusion that the #960 audit observed [B] | the job's name or log, read by a person or a script, says whether it ran tests [C] |

### 3.6 `unsafe`, in what purba compiles

Level 1: `#![forbid(unsafe_code)]`, or no `unsafe`. Level 2: each `unsafe` block has a `SAFETY` comment. Level 3: an `unsafe` block without one.

**Plain `cargo clippy` sees no dependency.** "Cargo … will pass `--cap-lints allow` when compiling your dependencies" ([levels.md L325-L327](https://github.com/rust-lang/rust/blob/5b7b37aefcb5b2fba3c2392bc9065b9e4602d494/src/doc/rustc/src/lints/levels.md#L325-L327)) [B]. With `-W clippy::undocumented_unsafe_blocks`, it reported 0 warnings on a git dependency that holds an undocumented block [A].

**A forced lint sees each one.** clippy's driver runs on a capped crate when a `--force-warn` names a clippy lint:

```rust
let cap_lints_allow = arg_value(&orig_args, "--cap-lints", |val| val == "allow")
    && !arg_value(&orig_args, "--force-warn", |val| val.contains("clippy::"));
```

([driver.rs L293-L294](https://github.com/rust-lang/rust/blob/5b7b37aefcb5b2fba3c2392bc9065b9e4602d494/src/tools/clippy/src/driver.rs#L293-L294)) [B].
So `RUSTC_WRAPPER=<clippy-driver>` with `RUSTFLAGS="--force-warn clippy::undocumented_unsafe_blocks"` lints every crate that cargo compiles.
The control: two git crates, one with a `// SAFETY:` comment above its block and one without. The run warned once, at the one without [A].
The lint "Checks for `unsafe` blocks and impls without a `// SAFETY: ` comment" ([undocumented_unsafe_blocks.rs L19-L21](https://github.com/rust-lang/rust/blob/5b7b37aefcb5b2fba3c2392bc9065b9e4602d494/src/tools/clippy/clippy_lints/src/undocumented_unsafe_blocks.rs#L19-L21)) [B]. It does not report a block that has no `unsafe` at all, so level 1 needs a second lint.
rustc's `unsafe_code` lint, forced the same way, reports each `unsafe` block, `unsafe fn` and `unsafe impl` [A].

The two runs give warnings, and a script must map them to a level. One mapping: no `unsafe_code` warning is level 1, `unsafe_code` warnings with no clippy warning is level 2, and a clippy warning is level 3 [C].
Two red-team probes show where that mapping gives a level the spec does not give. Each probe is a git dependency with no other code [A].

| the dependency holds | forced `unsafe_code` | forced clippy lint | the mapping gives | the spec gives |
|---|---|---|---|---|
| only `#[no_mangle] pub extern "C" fn` | 1: "usage of the unsafe `no_mangle` attribute" | 0 | 2 | 1: no `unsafe` |
| only `unsafe impl Send for S {}`, with no comment | 1: "implementation of an `unsafe` trait" | 1: "unsafe impl missing a safety comment" | 3 | level 3 names "an `unsafe` block without one", and the crate has no block |

Each run reads only the code compiled for the host target and purba's features.
`pulldown-cmark` 0.13.4 shows the difference. Its source has `unsafe` blocks, and its `lib.rs` says `#![cfg_attr(not(feature = "simd"), forbid(unsafe_code))]`. purba's script does not turn on `simd`, and the forced lints reported nothing in it [A].
One cost: purba's `build.warnings` denies a warning, so the workspace run exits 101 on purba's own `purba-execution`. A script must read the warnings, not the exit status [A].

| crate | `unsafe_code` warnings | `undocumented_unsafe_blocks` warnings |
|---|---:|---:|
| `pyo3` 0.29.2 | 822 (597 blocks) | 386 (360 blocks, 26 impls) |
| `libc` 0.2.189 | 15 (4 blocks) | 4 (2 blocks, 2 impls) |
| `ruff_python_ast` 0.0.17 | 2 (1 block) | 0 |
| `serde_core` 1.0.229, which `serde` brings | 2 (2 blocks) | 1 |
| `pyo3-ffi`, which `pyo3` brings and re-exports as `pyo3::ffi` | 98 | 0 |
| `pyo3-macros-backend`, which `pyo3` brings | 23 | 0 |
| `serde`, `ruff_python_parser`, `ruff_source_file`, `ruff_text_size`, `proptest`, `pulldown-cmark`, `harper-core`, `harper-pos-utils` | 0 | 0 |
`serde` re-exports `serde_core`, and `pyo3` re-exports `pyo3-ffi`. Whether a crate's `unsafe` counts as the crate that re-exports it is a reading of the spec [C].

| other tool | what it gives | difference from the spec |
|---|---|---|
| `cargo-geiger` 0.14.0 | per crate, `:)` "No `unsafe` usage found, declares #![forbid(unsafe_code)]", `?` "No `unsafe` usage found, missing #![forbid(unsafe_code)]", `!` "`unsafe` usage found", and counts "x = unsafe code used by the build" of "y = total unsafe code found in the crate" [A] | it tells level 1 from the rest. It counts expressions, not blocks, and reads no comment: `ruff_python_ast` shows 130 expressions where rustc saw 1 block. It refuses a virtual manifest, and it exited 1 on 19 warnings about files it never scanned [A]. Its README says it "is not meant to advise directly whether the code ultimately is truly insecure or not" ([README](https://docs.rs/crate/cargo-geiger/0.14.0/source/README.md)) [B] |
| a `grep` of the source | every `unsafe` token in the tarball [A] | it counts code that purba does not compile, as `pulldown-cmark` shows [A] |

### 3.7 API churn

Level 1: ≥ 1.0, and no major release in 12 months. Level 2: 0.x, or a major release in 12 months. Level 3: 0.x with a breaking release within 3 months (a 0.0.x bump counts as breaking), or ≥ 1.0 with a major release within 3 months.

| tool | what it gives | difference from the spec |
|---|---|---|
| crates.io `/api/v1/crates/{name}` | each version's `num`, `created_at` and `yanked` [A]. `ruff_python_parser` had 13 non-yanked releases in the last 90 days, each a 0.0.x bump [A] | none: the spec judges by version number and date, and the list has both |
| deps.dev versions | the same, per version [B] | the same |
| `cargo-semver-checks` 0.51.0 | "Lint your crate API changes for semver violations" ([README](https://github.com/obi1kenobi/cargo-semver-checks/blob/260fad71517e8b320c5c1ad1f367900ab37362a1/README.md)) [B] | it checks whether an API change broke semver, which the version number does not say. It "will not" catch every violation (same, L256-L260) [B]. The spec counts a bump, not a break [C] |

### 3.8 Owner in purba

Level 1: a bot proposes each new version. Level 2: a record names when a person raises it. Level 3: nobody.

| tool | what it gives | difference from the spec |
|---|---|---|
| Dependabot | `cargo` is supported for version updates ([supported-package-managers.md L13](https://github.com/github/docs/blob/d59b2631959db46d71ca5f7748ec04311c6d0870/data/reusables/dependabot/supported-package-managers.md#L13)) [B]. purba's `.github/dependabot.yml` has one entry, `package-ecosystem: github-actions` [A] | its cargo fetcher needs a `Cargo.toml`: `filenames.include?("Cargo.toml")` and "Repo must contain a Cargo.toml." ([file_fetcher.rb L22-L28](https://github.com/dependabot/dependabot-core/blob/13987a86518585c1b2fd84124ab23391dd5a318b/cargo/lib/dependabot/cargo/file_fetcher.rb#L22-L28)) [B]. A cargo script has none |
| Renovate 44.239.1 | the cargo manager's default `managerFilePatterns: ['/(^|/)Cargo\\.toml$/']` ([index.ts L37](https://github.com/renovatebot/renovate/blob/9bd78fd68dc4133dea88154ffb4aac6dc66be75a/lib/modules/manager/cargo/index.ts#L37)) [B] | the same: no cargo script |
| Scorecard `Dependency-Update-Tool` | "This check can determine only whether the dependency update tool is enabled; it does not ensure that the tool is run or that the tool's pull requests are merged" ([checks.md L320-L322](https://github.com/ossf/scorecard/blob/61e84743356275f6ffca23b1b2f2cb945e19eb96/docs/checks.md#L320-L322)) [B] | it reads the upstream project, not purba |
| a script over `.github/dependabot.yml` | whether an entry has `package-ecosystem: cargo` and a `directory` that holds the manifest [C] | an entry does not show that a bot proposes each new version, as the Scorecard row says of its own check [C]. Level 2 is a record. A script can find a record that names the crate. Whether the record names *when* a person raises it is a person's reading [C] |

### 3.9 Relief

A person scores relief. No tool surveyed measures the lines that purba would write without a crate, or the quality a crate serves.
crates.io gives `linecounts` for each version, such as 12,097 code lines for `pulldown-cmark` 0.13.6 [A]. Its status is "**Unstable**", and it "may be `null` until the version has been analyzed" ([OpenAPI snapshot L1386](https://github.com/rust-lang/crates.io/blob/3203d67d3d53575b31438352a447898ffbf72298/src/tests/snapshots/integration__openapi__openapi_snapshot-2.snap#L1386)) [B]. It counts the crate's lines, not the lines it saves [C].

## 4. The waiver

`lint:crates` refuses three more cases ([#969 spec § 4](https://github.com/kalonji-tools/purba/issues/969#issuecomment-6877037346)). Each needs the failures of the crate and the waiver's `fails` list, and nothing more.

| refusal | the failures it needs, and where they come from | what a tool has of its own |
|---|---|---|
| a waiver names a failure the crate does not have | advisory ids: the JSON of `cargo-deny` or `cargo-audit`. `"source"`: `cargo metadata`. `"score"`: the formula over `.config/crates.toml`. A script could compare the two lists [C] | `cargo-deny` `unused-ignored-advisory = "deny"`: "An unused advisory in the `advisories.ignore` list triggers an error, and cause the advisory check to fail" ([cfg.md L166](https://github.com/EmbarkStudios/cargo-deny/blob/bca0dde53651ee946720e4540b5ce2610bec8f06/docs/src/checks/advisories/cfg.md#L166)). The probe: `ignore = ["RUSTSEC-2021-0003"]` on purba's workspace drew `error[advisory-not-detected]`, and on the control that has it, `advisories ok` [A]. `unused-allowed-source = "deny"` is the same for `[sources]` ([sources/cfg.md L132](https://github.com/EmbarkStudios/cargo-deny/blob/bca0dde53651ee946720e4540b5ce2610bec8f06/docs/src/checks/sources/cfg.md#L132)) [B]. Both are lists for the whole graph, not for one crate: an ignored id "will cause the advisory to be treated as a note" ([cfg.md L90](https://github.com/EmbarkStudios/cargo-deny/blob/bca0dde53651ee946720e4540b5ce2610bec8f06/docs/src/checks/advisories/cfg.md#L90)) wherever it applies [B]. So a waiver on crate X that names an advisory on crate Y passes `cargo-deny`, and #969's first refusal must refuse it [C]. `cargo-deny` also reads `deny.toml`, not `.config/crates.toml`, so the ids must be copied across first [B]. Nothing has `"score"` |
| an entry fails something its waiver does not name | the same lists | each check tool refuses a failure that is not ignored or allowed [A], under the same two limits [B] |
| a waiver names anything but an advisory id, `"source"` or `"score"` | the waiver's own text | `cargo-deny` refuses an id in `ignore` that no database holds: `unknown-advisory` ([diags.md L42-L44](https://github.com/EmbarkStudios/cargo-deny/blob/bca0dde53651ee946720e4540b5ce2610bec8f06/docs/src/checks/advisories/diags.md#L42-L44)) [B]. No tool reads `.config/crates.toml` [C] |

## 5. A level that goes stale

`.config/crates.toml` holds the date each crate was judged. A tool can tell that a stored level no longer holds only by measuring again.

| input or check | what moves it | can a tool measure it again |
|---|---|---|
| upkeep, API churn | time alone: a window of 3 months, 12 months or 2 years passes | yes, from crates.io dates (§ 3.2, § 3.7) [C] |
| known advisory, trust's notice | a new advisory | yes: § 2.1. `cargo-deny`'s `maximum-db-staleness` refuses a database older than "`P90D`, or 90 days", but "This is only checked when advisory database fetching has been disabled" ([cfg.md L129-L133](https://github.com/EmbarkStudios/cargo-deny/blob/bca0dde53651ee946720e4540b5ce2610bec8f06/docs/src/checks/advisories/cfg.md#L129-L133)) [B]. The docs name `check --disable-fetch`; a red-team probe found that cargo-deny 0.20.2 rejects that flag, and only the global `--offline` works [A] |
| maintainers, upstream review | an owner or a commit upstream | yes, with the number tools of § 3 [C] |
| tests | a commit or a CI change upstream | in part: Scorecard `CI-Tests` gives level 1 against the rest (§ 3.5) [C] |
| `unsafe` | a new version in purba's lock | yes, with the forced lints of § 3.6 [C] |
| blast area | a new version, or a change in purba's parts | in part: the counts, not level 3's "names the crate's types" (§ 3.1) [C] |
| owner in purba, relief, trust's "project meant" | purba, or a person | no tool [C] |

No probe measured a stored level a second time, so this column is inferred.

`cargo-vet` sets an end on a person's judgment: a trusted entry's `end` "may be at most 1 year in the future" ([trusted-entries.md](https://github.com/mozilla/cargo-vet/blob/cef4dd43a0b9d2d37febf0e5ad42abc6d9ea5472/book/src/trusted-entries.md)) [B]. When a crate is judged again is [#977](https://github.com/kalonji-tools/purba/issues/977).

## 6. What is inferred [C]

1. `cargo-deny` and `cargo-audit` each give the known-advisory verdict only once a person fixes which advisory categories count (§ 2.1).
2. Whether each crate the spec judges is "a direct dependency of one or more workspace crates", as `cargo-deny`'s `"workspace"` setting reads it. The three script crates are not in purba's workspace (§ 2.1).
3. The spec's check names an advisory and not a yank, so a yank does not fail it (§ 2.1).
4. No tool surveyed says whether a crate's name, repository and publisher are the project meant (§ 2.4).
5. For a git-sourced crate, its crates.io releases are not the code purba compiles. `harper-core` is on crates.io at 3.2.0, and purba compiles git tag v2.3.0 (§ 3.2).
6. A `grep` finds a crate's path, not its type. A glob import or a re-export hides the crate's name (§ 3.1).
7. Whether level 3 of blast area means "in the public API", which `cargo-check-external-types` reads (§ 3.1).
8. Whether a user owner such as `rust-lang-owner` or `astral-crates-bot` counts as an organisation account (§ 3.3).
9. Scorecard `CI-Tests` cannot tell level 2 of tests from level 3, because it reads only CI (§ 3.5).
10. A check's name or log says whether a CI job ran tests (§ 3.5).
11. The mapping of the two forced lints' warnings to the three `unsafe` levels. Two probes show it gives a level the spec does not give (§ 3.6).
12. Whether a crate's `unsafe` counts as the crate that re-exports it: `serde_core` for `serde`, `pyo3-ffi` for `pyo3` (§ 3.6).
13. The spec counts a version bump, not an API break, so `cargo-semver-checks` answers another question (§ 3.7).
14. A script over `.github/dependabot.yml` can find a cargo entry, and an entry does not show that a bot proposes each new version. Whether a record names *when* a person raises a crate is a person's reading (§ 3.8).
15. crates.io's `linecounts` counts a crate's lines, not the lines it saves (§ 3.9).
16. A script could decide each of the three waiver refusals from the check tools' output and `.config/crates.toml`. No tool reads that file today (§ 4).
17. A waiver on one crate that names an advisory on another passes `cargo-deny`, so #969's first refusal must refuse it itself (§ 4).
18. Each "can a tool measure it again" cell of § 5. No probe measured a stored level a second time.

## 7. What is left to a person

- **Trust:** whether the name, the repository and the publisher are the project meant. `trustpub_data`, `published_by` and the owners are evidence, and `cargo-vet` can record the decision with an end date.
- **Relief:** the lines saved, or the quality served.
- **Owner in purba, level 2:** whether a record names when a person raises the crate.
- **Tests, level 2 against level 3:** whether a crate has tests that no CI runs.
- **Blast area, level 3:** whether another part names the crate's types, where a `grep` or the public API does not show it.
- **Maintainers:** whether a user owner is an organisation account.
- **`unsafe`:** whether a crate that a named crate re-exports counts as the named crate, and how a `#[no_mangle]` item or an `unsafe impl` with no block maps to a level.
- **Known advisory:** which advisory categories fail the check.
- **Upstream review:** whether Scorecard's changeset and implicit review stand for the spec's "reviewed commit".
- **A reworded check:** "A reworded check can need a person to judge it" ([#969 spec § 5](https://github.com/kalonji-tools/purba/issues/969#issuecomment-6877037346)).

## Sources

| source | version or commit |
|---|---|
| [cargo-deny docs](https://github.com/EmbarkStudios/cargo-deny/tree/bca0dde53651ee946720e4540b5ce2610bec8f06/docs/src/checks) and [`db.rs`](https://github.com/EmbarkStudios/cargo-deny/blob/bca0dde53651ee946720e4540b5ce2610bec8f06/src/advisories/helpers/db.rs) | 0.20.2, `bca0dde5`; the binary is purba's pin |
| [rustsec `query.rs`](https://github.com/rustsec/rustsec/blob/e3cf3703ed640854af152d019c976ae78211aa22/rustsec/src/database/query.rs) | rustsec 0.34.0, `e3cf3703`, used by cargo-audit 0.23.0 and not by cargo-deny |
| [RustSec advisory-db](https://github.com/rustsec/advisory-db/tree/4ef2df8e806a65e2f1ecc14f56f51249062b23d6) | `4ef2df8e`, 2026-11-13 |
| [osv-scanner](https://github.com/google/osv-scanner/releases/tag/v2.7.1) | 2.7.1, commit `02f96a40` |
| [OpenSSF Scorecard checks.md](https://github.com/ossf/scorecard/blob/61e84743356275f6ffca23b1b2f2cb945e19eb96/docs/checks.md) and [code_review.go](https://github.com/ossf/scorecard/blob/61e84743356275f6ffca23b1b2f2cb945e19eb96/checks/evaluation/code_review.go) | 5.6.0, `61e84743` |
| [clippy `undocumented_unsafe_blocks`](https://github.com/rust-lang/rust/blob/5b7b37aefcb5b2fba3c2392bc9065b9e4602d494/src/tools/clippy/clippy_lints/src/undocumented_unsafe_blocks.rs), [clippy driver](https://github.com/rust-lang/rust/blob/5b7b37aefcb5b2fba3c2392bc9065b9e4602d494/src/tools/clippy/src/driver.rs), [rustc lint levels](https://github.com/rust-lang/rust/blob/5b7b37aefcb5b2fba3c2392bc9065b9e4602d494/src/doc/rustc/src/lints/levels.md) | rust `5b7b37ae`, purba's pinned `nightly-2026-11-09` |
| [cargo-geiger README](https://docs.rs/crate/cargo-geiger/0.14.0/source/README.md) | 0.14.0 |
| [cargo-vet book](https://github.com/mozilla/cargo-vet/tree/cef4dd43a0b9d2d37febf0e5ad42abc6d9ea5472/book/src) | 0.10.3, `cef4dd43` |
| [cargo-crev README](https://github.com/crev-dev/cargo-crev/blob/54d198b02359a612c0c5f4f8eee6ceb8b9a4169b/cargo-crev/README.md) | 0.27.1, `54d198b0` |
| [cargo-supply-chain](https://github.com/rust-secure-code/cargo-supply-chain/tree/ae1c240ace44b55ba01ac732f056eb632118e655) | 0.3.8, `ae1c240a` |
| [cargo-unmaintained README](https://github.com/trailofbits/cargo-unmaintained/blob/10d735c2b1bc3cd3001e921268eca865f8e7d845/README.md) | 2.1.0, `10d735c2` |
| [cargo-outdated README](https://github.com/kbknapp/cargo-outdated/blob/9a25ecbfcd0a1d7394bb521e89022950bca9d7b0/README.md) | 0.19.0, `9a25ecbf` |
| [cargo-semver-checks README](https://github.com/obi1kenobi/cargo-semver-checks/blob/260fad71517e8b320c5c1ad1f367900ab37362a1/README.md) | 0.51.0, `260fad71` |
| [cargo-check-external-types README](https://github.com/awslabs/cargo-check-external-types/blob/705a0941997ebeb3bfb1a7f14070ba342f79d879/README.md) | `705a0941` |
| [Dependabot supported package managers](https://github.com/github/docs/blob/d59b2631959db46d71ca5f7748ec04311c6d0870/data/reusables/dependabot/supported-package-managers.md) | github/docs `d59b2631` |
| [dependabot-core cargo file fetcher](https://github.com/dependabot/dependabot-core/blob/13987a86518585c1b2fd84124ab23391dd5a318b/cargo/lib/dependabot/cargo/file_fetcher.rb) | `13987a86` |
| [Renovate cargo manager](https://github.com/renovatebot/renovate/tree/9bd78fd68dc4133dea88154ffb4aac6dc66be75a/lib/modules/manager/cargo) | 44.239.1, `9bd78fd6` |
| [crates.io `rustsec.ts`](https://github.com/rust-lang/crates.io/blob/3203d67d3d53575b31438352a447898ffbf72298/svelte/src/lib/utils/rustsec.ts) and [policies page](https://github.com/rust-lang/crates.io/blob/3203d67d3d53575b31438352a447898ffbf72298/svelte/src/routes/policies/+page.svelte) | `3203d67d` |
| [crates.io OpenAPI snapshot](https://github.com/rust-lang/crates.io/blob/3203d67d3d53575b31438352a447898ffbf72298/src/tests/snapshots/integration__openapi__openapi_snapshot-2.snap) | `3203d67d` |
| crates.io API v1, deps.dev API v3, Scorecard REST API | live, fetched 2026-11-14 |
| [Socket ecosystem support](https://docs.socket.dev/docs/language-support) | unpinned: a live page, updated 2026-11-05 |

## Appendix A: What the tools returned on the 11 crates

Numbers, not levels. Measured on 2026-11-14. "Brings" is `cargo tree -p <crate> -e normal,build --prefix none`, distinct lines, without the crate itself; `proptest` uses `-e normal,build,dev`.

| crate | source | advisories on the locked version | owners (`kind`) | brings | parts that declare it | newest release |
|---|---|---|---|---:|---|---|
| `pyo3` 0.29.2 | crates.io | none | 2 users, 1 team | 12 | purba, assertion, command (dev), execution | 0.30.1, 2026-10-14 |
| `ruff_python_ast` 0.0.17 | crates.io | none | 2 users | 47 | assertion, parsing, selection | 0.0.22, 2026-11-12 |
| `ruff_python_parser` 0.0.17 | crates.io | none | 2 users | 71 | parsing | 0.0.22, 2026-11-12 |
| `ruff_source_file` 0.0.17 | crates.io | none | 2 users | 30 | parsing | 0.0.22, 2026-11-12 |
| `ruff_text_size` 0.0.17 | crates.io | none | 2 users | 28 | assertion, parsing, selection | 0.0.22, 2026-11-12 |
| `serde` 1.0.229 | crates.io | none | 1 user, 1 team | 6 | execution | 1.0.229, 2026-07-18 |
| `libc` 0.2.189 | crates.io | none | 1 user | 0 | execution | 0.2.191, 2026-10-16 |
| `proptest` 1.12.0 | crates.io | none | 2 users, 1 team | 14 | assertion (dev) | 1.12.0, 2026-10-10 |
| `pulldown-cmark` 0.13.4 | crates.io | `RUSTSEC-2026-0414`, `RUSTSEC-2026-0433` | 3 users | 7 | `scripts/check-prose` | 0.13.6, 2026-10-30 |
| `harper-core` 2.3.0 | git, tag v2.3.0 | none matched: `cargo-deny` and `cargo-audit` skip a git source | 1 user (on crates.io) | 224 | `scripts/check-prose` | 3.2.0 on crates.io, 2026-11-13 |
| `harper-pos-utils` 2.3.0 | git, tag v2.3.0 | the same | 1 user (on crates.io) | 128 | `scripts/check-prose` | 3.2.0 on crates.io, 2026-11-13 |

## Appendix B: Commands

`W` is the worktree. `S` is the scratchpad. Every tool not pinned in mise ran from `$S/tools` or `$S/dl`. Every crates.io and deps.dev request sent the User-Agent `purba-research (https://github.com/kalonji-tools/purba)`.

```bash
# B1. cargo-deny advisories and sources: tree green, control red (control = smallvec 1.6.0 + a file:// git crate)
cat > $S/probe/deny/deny.toml <<EOF
[advisories]
db-path = "$S/probe/advisory-dbs"
yanked = "deny"
unused-ignored-advisory = "deny"
[sources]
unknown-registry = "deny"
unknown-git = "deny"
unused-allowed-source = "deny"
EOF
mise exec -- cargo-deny --manifest-path $W/Cargo.toml --config $S/probe/deny/deny.toml check advisories sources       # advisories ok, sources ok
mise exec -- cargo-deny --manifest-path $S/probe/ctl/app/Cargo.toml --config $S/probe/deny/deny.toml check advisories sources
#   error[source-not-allowed]; error[vulnerability] RUSTSEC-2021-0003; exit 9
mise exec -- cargo-deny --manifest-path $W/scripts/check-prose/check-prose.rs --config … check advisories sources
#   [ERROR] --manifest-path must point to a Cargo.toml file, exit 1
L="resolver.lockfile-path=\"$W/scripts/check-prose/Cargo.lock\""
mise exec -- cargo -Zscript metadata --locked --offline --format-version 1 \
  --manifest-path scripts/check-prose/check-prose.rs --config "$L" > $S/probe/meta-script.json
mise exec -- cargo-deny --offline --metadata-path $S/probe/meta-script.json --config $S/probe/deny/deny.toml check advisories sources
#   exit 9: RUSTSEC-2026-0414, RUSTSEC-2026-0433; error[unmaintained] bincode, paste;
#   four error[source-not-allowed]: harper-core, harper-pos-utils, harper-brill, harper-thesaurus
mise exec -- cargo metadata --locked --offline --format-version 1 --manifest-path $W/Cargo.toml > $S/probe/meta-ws.json
mise exec -- cargo-deny --offline --metadata-path $S/probe/meta-ws.json --config $S/probe/deny/deny.toml check advisories sources
#   control: advisories ok, sources ok, exit 0

# B2. the first waiver refusal, in cargo-deny's own form
#   the same config plus ignore = ["RUSTSEC-2021-0003"]
#   workspace: error[advisory-not-detected], exit 1. control: advisories ok, exit 0

# B3. cargo-audit and osv-scanner on both locks and the control (a re-run of osv-scanner on the script lock: 15 package versions, 32 RustSec ids, as cargo-audit)
$S/tools/bin/cargo-audit audit --db $S/probe/rustsec-db -f <lock> --json
$S/dl/osv/osv-scanner_linux_amd64 scan source --lockfile <lock> --format json

# B4. a git-sourced smallvec 1.6.0 (control for the source comparison in rustsec)
#   cargo-audit: exit 0. cargo-deny advisories: ok. osv-scanner: RUSTSEC-2021-0003, ecosystem crates.io, exit 1
#   (cargo-deny makes the comparison in its own src/advisories/helpers/db.rs, not through rustsec)

# B5. licences: tree green, control red (a path crate licensed GPL-3.0-only, purba's allow list without exceptions)
mise exec -- cargo-deny check licenses                                         # licenses ok
mise exec -- cargo-deny --manifest-path $S/probe/lic/app/Cargo.toml --config $S/probe/lic/deny.toml check licenses   # error[rejected]
$S/dl/osv/osv-scanner_linux_amd64 scan source --licenses="MIT" --lockfile $W/Cargo.lock --format json   # 16 violations

# B6. the cargo script's graph
L="resolver.lockfile-path=\"$W/scripts/check-prose/Cargo.lock\""
mise exec -- cargo -Zscript metadata --locked --format-version 1 --manifest-path scripts/check-prose/check-prose.rs --config "$L"
mise exec -- cargo -Zscript tree --locked --manifest-path scripts/check-prose/check-prose.rs --config "$L" -p <crate> -e normal,build --prefix none

# B7. blast area on the workspace
cargo tree --locked --offline -p <crate> -e normal,build --prefix none | sed 's/ (\*)//; s/ (proc-macro)//' | grep -v '^$' | sort -u | grep -v "^<crate> v" | wc -l
cargo tree --locked --offline -i <crate> -e normal,build,dev --depth 1 --prefix none | grep '^purba'
grep -rlE "\b<crate>::|use <crate>\b" crates/*/src crates/*/tests | cut -d/ -f1 | sort -u      # control: smallvec:: matches 0 files

# B8. unsafe: control first (two file:// git crates, one with // SAFETY:, one without)
cargo +stable clippy -- -W clippy::undocumented_unsafe_blocks                                   # 0 warnings
RUSTC_WRAPPER=$(rustup which --toolchain stable clippy-driver) \
  RUSTFLAGS="--force-warn clippy::undocumented_unsafe_blocks" cargo +stable check               # 1 warning, at the undocumented crate
# then purba, with the pinned toolchain and a scratch target dir
CD=$(mise exec -- rustup which clippy-driver)
RUSTC_WRAPPER=$CD RUSTFLAGS="--force-warn clippy::undocumented_unsafe_blocks" \
  mise exec -- cargo check --locked --workspace --all-targets --target-dir $S/build/uub-ws --message-format short
RUSTC_WRAPPER=$CD RUSTFLAGS="--force-warn unsafe_code" \
  mise exec -- cargo check --locked --workspace --all-targets --target-dir $S/build/uc-ws --message-format short   # exit 101 on purba-execution
#   the same two runs with `cargo -Zscript check --release --locked --manifest-path scripts/check-prose/check-prose.rs --config "$L"`
LD_LIBRARY_PATH=<openssl>/lib CARGO_TARGET_DIR=$S/build/geiger \
  mise exec -- $S/tools/bin/cargo-geiger geiger --manifest-path $W/crates/purba/Cargo.toml --output-format Ascii

# B9. upstream review, tests, upkeep
GITHUB_AUTH_TOKEN=<agent account> $S/dl/sc/scorecard --repo=github.com/<owner>/<repo> \
  --checks=Code-Review,CI-Tests,Maintained --show-details --format json
SC_API=api.securityscorecards.dev DEPS=api.deps.dev CRATES=crates.io
curl -A "$UA" "https://${SC_API}/projects/github.com/${OWNER}/${REPO}"
curl -A "$UA" "https://${DEPS}/v3/projects/github.com%2F${OWNER}%2F${REPO}"

# B10. crates.io and deps.dev, one request per second
curl -sS --rate 1/s -A "$UA" -o "${C}.json" "https://${CRATES}/api/v1/crates/${C}" -o "${C}.owners.json" "https://${CRATES}/api/v1/crates/${C}/owners" …
curl -A "$UA" https://api.deps.dev/v3/systems/cargo/packages/pulldown-cmark/versions/0.13.4
curl -A "$UA" https://api.deps.dev/v3/systems/cargo/packages/pulldown-cmark/versions/0.13.4:dependencies
$S/tools/bin/cargo-supply-chain json --cache-max-age=0s --manifest-path=$W/Cargo.toml
```
