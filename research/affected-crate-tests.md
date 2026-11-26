# Which tests a pull request that edits one crate runs, and which tests of an unedited crate catch a break in its dependency

Research for [When a pull request edits one crate, which tests do comparable workspaces run, and which tests of an unedited crate catch a break in its dependency?](https://github.com/kalonji-tools/purba/issues/1276)
Measured on 2026-11-26, against purba `main` at `58dc3f6a`, on one Linux machine.
This file decides nothing. It lists what others do, and what a mutation probe counted on purba.

## Summary

No large Rust workspace in the survey picks tests by crate on a pull request. ruff, uv, cargo, bevy, tokio and rust-analyzer each run the tests of every crate when any code changes. Their path filters start other jobs only. The tools built to pick by change are guppy's `determinator`, the Bazel and Buck2 target determinators, Nx and Turborepo. nextest's `rdeps()` selects the same set from a crate name. By default, each one also runs the tests of every dependent of the edited unit, unit tests included. Each of them can trim the dependents only through an option or a label the user adds. Buck2 names its label `ci:dangerously_skip_upstream`.

On purba, I made every mutant that cargo-mutants generates for `errors`, `parsing` and `query`: 434 in all, of which 220 build. I ran every Rust test of the workspace on each one, and the bats tests on each one that no test of the edited crate kills.

- **3 of the 220 are killed only by the unit tests of an unedited dependent.** All three are in `errors`, in `Tally::add_suppressed`. In Rust, only unit tests in `reporting` catch them. The bats tests catch all three as well: each one fails 27 of the 295.
- 16 more are killed only by a test under `crates/<dependent>/tests`. The bats tests catch 5 of them and pass 11.
- 17 survive every Rust test. The bats tests catch 2 of them, so 15 survive every test purba has.
- **[criterion 3]** So, on this sample, the strict reading of criterion 3 lets through no mutant that today's CI catches, as long as the bats tests run. It skips the dependents' unit tests and keeps their `tests/` files. Without the bats tests, it lets through 3 more than every Rust test does: the 3 above. A loose reading also skips the dependents' `tests/` files, and lets 11 more through.
- The edited crate's own tests kill 182 of the 201 caught mutants. Every one of those kills comes from the crate's `tests/` folder. The edited crate's own unit tests kill none: `parsing` and `query` have no unit tests, and the 5 in `errors` kill nothing.
- purba has few unit tests: 72 of its 470 Rust tests. 67 of them are in dependents of `errors`. So the strict reading of criterion 3 skips 67 tests on an edit to `errors`, 35 on an edit to `parsing`, and 5 on an edit to `query`.

**[criterion 3]** The probe found the case that the grilling predicted. A unit test of `reporting` is the only Rust test that holds part of the contract of `errors`. The bats tests hold that part too. **[criterion 4]** The tools split on the lockfile. Nx (by default), Turborepo, pytest-testmon, ruff and uv run everything on a change to it. guppy's `determinator`, Nx's `"auto"` and testtrim run only the code that uses the changed package. purba's own CI runs every Rust test on a change to `Cargo.lock`.

## How to read this file

- **[run]** means I ran it in this session, at the version named. **[source]** means the tool's code or documentation says so, at the pinned version or commit, and I did not run it. **judgement** marks a reading that no source states.
- **[criterion 3]** and **[criterion 4]** mark a finding that bears on that criterion of the issue.
- The survey rows of sections 1 and 2 were gathered by a helper agent from the pinned sources. I checked six of them again against the files downloaded at those pins: nextest's `rdeps` leaf, the `determinator`'s default rules, ruff's gate on `cargo test` and its lockfile filters, bevy's test command, Turborepo's lockfile rule, and Nx's lockfile default. All six hold.
- **Buckets.** Each test target that fails on a mutant goes in one bucket:

  | bucket | the target |
  |---|---|
  | own-lib | a unit test of the edited crate (`--lib`) |
  | own-int | a test under `crates/<edited>/tests` (`--test <name>`) |
  | own-doc | a doc test of the edited crate |
  | dep-lib | a unit test of another crate |
  | dep-int | a test under `crates/<other>/tests` |
  | dep-doc | a doc test of another crate |
  | bats | a test under `tests/`, on the installed wheel |

  Every crate that fails a mutant of `errors`, `parsing` or `query` depends on the edited crate. A crate that does not depend on it cannot see the mutant.

## Method

### Versions

| tool | version | where it ran |
|---|---|---|
| rustc and cargo | `nightly-2026-11-23`: rustc 1.102.0-nightly (137d1d494 2026-11-22), cargo 1.102.0-nightly (ae8f06eb4 2026-11-21) | `mise exec --`, as `.config/mise.toml` pins it |
| Python | 3.12.14, with the GIL | `mise exec -- python`, the version `.config/mise.toml` pins |
| cargo-mutants | 27.1.0 | `cargo install --locked --root <scratch>/tools`. `.config/mise.toml` does not pin it. |
| cargo-nextest | 0.9.152 (26ebeda79 2026-11-13) | the same scratch root. `.config/mise.toml` does not pin it. |
| bats | 1.14.0 | `mise exec --`, as pinned |
| maturin | 1.15.0 | `mise exec --`, as pinned |

### The probe [run]

The scripts are in `research/affected-crate-tests/`. Each runs from the root of a purba checkout, under `mise exec --`.

1. **Baseline.** `cargo test --workspace --no-fail-fast` passes on `58dc3f6a`, with 65 test binaries and doc-test runs. A clean build of the tests took 24 s and the run took 9.5 s. `mise run test:wheel` passes 295 of 295 bats tests in 2 min, with both wheel builds.
2. **Control.** `control.sh` applies `control.patch` to `errors::catch`, runs every Rust test, and prints the targets that fail. Section 4.1 has its result.
3. **Mutants.** `mutants.sh` runs cargo-mutants on one crate with `--test-workspace=true`, so each mutant meets every test of the workspace. `--cargo-test-arg=--no-fail-fast` makes cargo go on past the first failing target, so its log names every target that kills the mutant. `--cap-lints=true` stops `.cargo/config.toml`'s `warnings = "deny"` from making a mutant unviable over an unused variable. The test timeout is 120 s.
4. **Sorting.** `classify.py` reads each log's `error: N targets failed:` list and sorts each target into a bucket. `data/rows.tsv` holds one row per mutant. `killers.sh` reads the same list from one log.
5. **bats.** `bats.sh` applies the diff of a mutant to the checkout, runs `mise run test:wheel` with `CARGO_BUILD_WARNINGS=allow`, counts the `not ok` lines, and reverts the diff. I ran it on each mutant that no test of the edited crate kills: the dep-lib, dep-int and missed classes. `data/bats.tsv` holds the result.
6. **Filtersets.** `filtersets.sh` lists, without running them, the tests that five nextest filtersets select for an edit to one crate.

**Sample.** The scope asks for a sample. The cost was low, so I ran every mutant that cargo-mutants 27.1.0 generates for the three crates: 66 for `errors`, 289 for `parsing` and 79 for `query`. The bats tests are the expensive part, so they ran on the 36 mutants that no test of the edited crate kills, and on no other.

```sh
S=<scratch dir>
mise exec -- research/affected-crate-tests/control.sh $S/logs/control.log
for c in errors query parsing; do
  mise exec -- research/affected-crate-tests/mutants.sh $S/tools/bin/cargo-mutants purba-$c $S/out/$c $S/tmp
done
R=research/affected-crate-tests
mise exec -- python -I $R/classify.py $S/out/{errors,parsing,query}/mutants.out > $R/data/rows.tsv
mise exec -- python -I $R/deponly.py $R/data/rows.tsv $S/out/{errors,parsing,query}/mutants.out > $R/data/dep-only.tsv
$R/bats.sh $S/logs/bats <diff of each dep-lib, dep-int and missed mutant> > $R/data/bats.tsv
mise exec -- python -I $R/readings.py $R/data/rows.tsv $R/data/bats.tsv $S/out/{errors,parsing,query}/mutants.out
PATH=$S/tools/bin:$PATH mise exec -- research/affected-crate-tests/filtersets.sh purba-errors
```

## 1. Which tests comparable projects and tools run

Each row is [source] at the pinned link. "Dependents' unit tests" means: when crate A changes, does the tool run the `--lib` tests of a crate B that depends on A?

| tool or project | unit it selects | runs the tests of dependents? their unit tests? | change that runs everything | merge to `main` runs everything? | mark | source |
|---|---|---|---|---|---|---|
| purba today | the whole workspace | yes, all of them | any path that `.config/gates` marks `build`: `crates/**`, `/Cargo.lock`, `/Cargo.toml`, `.config/mise.toml` and more | no run on `main`: `ci.yml` runs on `pull_request` only | **[criterion 3]** **[criterion 4]** | [gates#L19-L44](https://github.com/kalonji-tools/purba/blob/58dc3f6a9746fe2c328811bdb68bfd4132d90de2/.config/gates#L19-L44), [ci.yml#L34-L36](https://github.com/kalonji-tools/purba/blob/58dc3f6a9746fe2c328811bdb68bfd4132d90de2/.github/workflows/ci.yml#L34-L36) |
| `cargo test -p` (cargo 0.103.0) | a package, and target kinds by flag | no. `-p A` runs A only, and cargo has no option for reverse dependencies | none: it detects no change | none | **[criterion 3]**: `-p` is how to leave a dependent out | [cargo-test.md#L100-L160](https://github.com/rust-lang/cargo/blob/562527a53aa333eec76ba6207cfdb283de5c1129/doc/book/src/commands/cargo-test.md#L100-L160) |
| cargo-nextest filtersets (0.9.152) | a single test | `rdeps(A)`: yes, every binary of each dependent, the unit tests too. `- kind(lib)` removes them; section 3 runs it. | none: it detects no change | none | **[criterion 3]** | [reference.md#L17-L62](https://github.com/nextest-rs/nextest/blob/cargo-nextest-0.9.152/nextest-filtering/reference.md#L17-L62), [compile.rs#L147-L190](https://github.com/nextest-rs/nextest/blob/cargo-nextest-0.9.152/nextest-filtering/src/compile.rs#L147-L190) |
| guppy `determinator` (0.13.0) | a package | yes: its affected set is the changed packages and their transitive dependents. The caller tests each one in full. | `rust-toolchain`, the root `Cargo.toml`, `.cargo/config(.toml)`, and any file outside every package. `Cargo.lock` gets a deeper diff instead. | not built in. The README advises scheduled full runs on `main`. | **[criterion 3]** **[criterion 4]** | [default-rules.toml#L12-L26](https://github.com/guppy-rs/guppy/blob/0cbe52c4cbf4ac55bc2fa9ba7f8f5d14288fe478/tools/determinator/default-rules.toml#L12-L26), [determinator.rs#L240-L266](https://github.com/guppy-rs/guppy/blob/0cbe52c4cbf4ac55bc2fa9ba7f8f5d14288fe478/tools/determinator/src/determinator.rs#L240-L266) |
| Bazel + target-determinator | a test target | yes: a target's hash covers all its transitive dependencies | no file by name. A change that alters the configuration of all tests selects all. | not built in | **[criterion 4]** | [hash_cache.go#L82-L98](https://github.com/bazel-contrib/target-determinator/blob/21e827870e76f4bc4551b33b2294d757df9bbb09/pkg/hash_cache.go#L82-L98) |
| Bazel + bazel-diff | a target | yes, by default. `target_distance` and `package_distance` let CI skip far dependents. | a change to `MODULE.bazel`, `*bazelrc` and the like forces a full hash run. Which tests that selects: unverified. | not built in | **[criterion 3]** | [README.md#L232-L288](https://github.com/Tinder/bazel-diff/blob/6745276c6f494612b96190c852b63ca5d577db53/README.md#L232-L288) |
| Buck2 + btd | a target | yes, "including recursive dependencies". `--depth` limits the levels, and the label `ci:dangerously_skip_upstream` stops the fan-out. | a buckconfig change selects all only with `--buckconfig-select-all` | not built in | **[criterion 3]** | [lib.rs#L115-L135](https://github.com/facebookincubator/buck2-change-detector/blob/24d3a69984ec10de71c09b800726c428a81038cb/btd/src/lib.rs#L115-L135), [diff.rs#L684-L687](https://github.com/facebookincubator/buck2-change-detector/blob/24d3a69984ec10de71c09b800726c428a81038cb/btd/src/diff.rs#L684-L687) |
| Nx `affected` (24) | a task. Nx 23 and older: a project. | Nx 24: only when the dependent task's inputs reach the change. The docs' own `test` inputs, `^production`, do reach it. Nx 23: every task of each dependent. | the lockfile, by default (`"all"`). `"auto"` limits it to the projects whose resolved packages changed. Any input every task shares. | it advises comparing with the last green commit of `main` | **[criterion 3]** **[criterion 4]** | [affected.mdoc#L35-L225](https://github.com/nrwl/nx/blob/990d34125bcd040b593528bc2bce0edc2a45102b/astro-docs/src/content/docs/features/CI%20Features/affected.mdoc#L35-L225), [nx-json.mdoc#L208-L218](https://github.com/nrwl/nx/blob/990d34125bcd040b593528bc2bce0edc2a45102b/astro-docs/src/content/docs/reference/nx-json.mdoc#L208-L218) |
| Turborepo `--affected` (2.11.13) | a package. A task, with `affectedUsingTaskInputs`. | yes: `--affected` means `--filter=...[main...HEAD]`, and `...` before a name selects dependents | the root `package.json`, `turbo.json`, the lockfile and `globalDependencies`, even in task mode | not built in | **[criterion 4]** | [run.mdx#L47-L86](https://github.com/vercel/turborepo/blob/v2.11.13/apps/docs/content/docs/reference/run.mdx#L47-L86), [configuration.mdx#L372-L406](https://github.com/vercel/turborepo/blob/v2.11.13/apps/docs/content/docs/reference/configuration.mdx#L372-L406) |
| ruff | the whole workspace | yes: everything | any change outside `docs/**` and `assets/**`. Each component filter also lists `Cargo.lock` and `rust-toolchain.toml`. | yes | **[criterion 4]** | [ci.yaml#L443-L477](https://github.com/astral-sh/ruff/blob/cde61b53f6611abc526c74a341e361294c68cb92/.github/workflows/ci.yaml#L443-L477) |
| uv | the whole workspace | yes: everything | any file but docs, `*.md`, `bin/`, `assets/` and agent folders | yes | none | [plan.yml#L203-L246](https://github.com/astral-sh/uv/blob/9bf01bef71c76643290ca17635970076fab1bd0d/.github/workflows/plan.yml#L203-L246) |
| cargo | the whole workspace, in three runs | yes: everything, on every pull request | no path filter | the merge queue runs the same jobs | none | [main.yml#L2-L6](https://github.com/rust-lang/cargo/blob/562527a53aa333eec76ba6207cfdb283de5c1129/.github/workflows/main.yml#L2-L6), [main.yml#L241-L253](https://github.com/rust-lang/cargo/blob/562527a53aa333eec76ba6207cfdb283de5c1129/.github/workflows/main.yml#L241-L253) |
| bevy | the whole workspace | yes: everything | no path filter | the merge queue runs it | none | [test.rs#L22-L38](https://github.com/bevyengine/bevy/blob/71df17383f6f51f1123196368161a7d1be5ddf0f/tools/ci/src/commands/test.rs#L22-L38) |
| tokio | the whole workspace, twice: the second run drops the workspace patches | yes: everything | no path filter | yes | none | [ci.yml#L95-L133](https://github.com/tokio-rs/tokio/blob/854013a8cb7d1397e9d037315592dc1d2d55187d/.github/workflows/ci.yml#L95-L133) |
| rust-analyzer | the whole workspace | yes: everything. Only the TypeScript job has a path filter. | no path filter for Rust | the merge queue runs it | none | [ci.yaml#L26-L41](https://github.com/rust-lang/rust-analyzer/blob/9c74e78e01d7cd31663f4ecf71e65b1cc4b9ccb8/.github/workflows/ci.yaml#L26-L41) |
| rust-lang/rust | a CI job by platform, not by crate | a pull request runs 11 fixed jobs | none: it selects by job, not by change | yes: a merge runs 82 jobs | none | [jobs.yml#L108-L223](https://github.com/rust-lang/rust/blob/86e8c946f12c16ad26be9e4813cec57232290d99/src/ci/github-actions/jobs.yml#L108-L223) |

Notes on the rows:

- **nextest.** `rdeps()` compiles to the same leaf as `package()`: "all tests in crates matching `name-matcher`, and all the crates that (possibly transitively) depend on" it. So it includes the `lib` binary of each dependent. `kind(lib)` is "Unit tests for regular crates, typically in the `src/` directory under `#[cfg(test)]`". nextest only selects. It does not look at a diff.
- **determinator.** "If the file is outside a package, the determinator assumes that everything needs to be rebuilt" ([README#L61-L67](https://github.com/guppy-rs/guppy/blob/0cbe52c4cbf4ac55bc2fa9ba7f8f5d14288fe478/tools/determinator/README.md#L61-L67)). Of `Cargo.lock`, the rules say it is "ignored, since the determinator does a deeper analysis". That analysis marks a package whose "version or feature set" changed ([README#L79-L99](https://github.com/guppy-rs/guppy/blob/0cbe52c4cbf4ac55bc2fa9ba7f8f5d14288fe478/tools/determinator/README.md#L79-L99)). The README also says: "schedule full runs every few hours on the main branch in case the determinator misses something" ([README#L146-L149](https://github.com/guppy-rs/guppy/blob/0cbe52c4cbf4ac55bc2fa9ba7f8f5d14288fe478/tools/determinator/README.md#L146-L149)).
- **Buck2.** The README warns: "you may wish to avoid recompiling targets many steps away - at the risk of potentially allowing a breakage into the repo" ([README#L54-L60](https://github.com/facebookincubator/buck2-change-detector/blob/24d3a69984ec10de71c09b800726c428a81038cb/README.md#L54-L60)).
- **Nx.** "If `app:test` lists only `{projectRoot}/**/*`, a change to the `ui` source doesn't select it, even though `app` imports `ui`" ([affected.mdoc#L35-L72](https://github.com/nrwl/nx/blob/990d34125bcd040b593528bc2bce0edc2a45102b/astro-docs/src/content/docs/features/CI%20Features/affected.mdoc#L35-L72)). Nx is the one tool where criterion 3 is a setting: drop `^production` from the inputs of a test task. That drops the dependent's whole test task, not only its unit tests.
- **ruff.** `determine_changes` sets a flag for each component with `git diff --quiet`. The flags start only the ecosystem, fuzz, playground and benchmark jobs. `cargo test` runs when `code == 'true' || github.ref == 'refs/heads/main'` ([L447](https://github.com/astral-sh/ruff/blob/cde61b53f6611abc526c74a341e361294c68cb92/.github/workflows/ci.yaml#L447)).
- **uv.** `plan.yml` computes `rust_deps_changed` from `Cargo.toml` and `Cargo.lock`, but uses it only for release builds, Docker, trampolines and the cache.
- **cargo.** The test job runs `cargo test -p cargo`, then `cargo test --workspace --exclude cargo …`. The split is for disk and parallel runs, not for the change.
- **tokio.** Its second run, with the workspace patches removed, tests the crates against their published dependencies. That is a test of the chain between crates, criterion 2's "communal" kind (judgement).

## 2. Test impact analysis at the grain of one test

Each row is [source].

| tool | grain | map it keeps | what keeps the map true | a change outside the map | source |
|---|---|---|---|---|---|
| cargo-difftests (Rust) | one test | test → the files with a region it ran (LLVM coverage) | rebuilt when a test reruns. The base is the last commit, or file times. | it can ignore registry files. On a change to `Cargo.lock` alone, or to the toolchain: unverified. No release tag; last commit 2024-05-19. | [README#L94-L168](https://github.com/dnbln/cargo-difftests/blob/9f16064c7d55606b727fde0b02410dce55d51fa5/README.md#L94-L168) |
| testtrim (Rust and others, 0.14.8) | one test | test → the files it covered; test → each third-party package and version it ran | recorded on each run | a change to `Cargo.lock` selects "all tests that touched" a changed or removed package. A test that touches the network always reruns. | [docs.rs 0.14.8, rust.rs](https://docs.rs/crate/testtrim/0.14.8/source/src/platform/rust.rs) (not pinned at a commit) |
| pytest-testmon (Python, 2.4.0) | one test | test → fingerprints of the blocks it ran | recomputed on each run. It stores the installed packages, without patch versions, and the Python version. | a changed package or Python version: "All tests have to be re-executed." | [pytest_testmon.py#L350-L353](https://github.com/tarpas/pytest-testmon/blob/v2.4.0/testmon/pytest_testmon.py#L350-L353), [db.py#L665-L667](https://github.com/tarpas/pytest-testmon/blob/v2.4.0/testmon/db.py#L665-L667) |
| Ekstazi (Java, ISSTA 2015) | one test class | "dynamic dependencies of tests on files" | collected again on each run | unverified: I read the abstract only | [DOI 10.1145/2771783.2771784](https://doi.org/10.1145/2771783.2771784) |
| Meta's Predictive Test Selection | a test, per change | a model trained on "historical test outcomes", with no coverage map | retrained on history | probabilistic: it keeps "over 95% of individual test failures and over 99.9% of faulty changes" | [arXiv 1810.05286](https://arxiv.org/abs/1810.05286) |
| Google TAP study | a test target | the dependency graph and history | none: it is a study | failing tests "are generally 'closer' to the code they test" | [DOI 10.1109/ICSE-SEIP.2017.16](https://doi.org/10.1109/ICSE-SEIP.2017.16) |

- **[criterion 3]** A coverage map answers criterion 3 test by test, not crate by crate. It selects a unit test of a dependent only if that test ran code of the edited crate. On purba, the unit tests of `reporting` that call `errors::catch` and `Tally::add_suppressed` would be selected for an edit to `errors`, and the others would not (judgement, from the map's definition; not run).
- **[criterion 4]** testtrim limits a lockfile change to the tests that ran the changed package. testmon reruns everything on any change of a package's minor version.
- testtrim warns it "can be fooled when a `pub const` value, or a `lazy_static` value, is modified" ([docs.rs 0.14.8](https://docs.rs/crate/testtrim/0.14.8)). A map of what ran misses what was folded in at compile time.

## 3. purba's graph and its tests [run]

The `path` dependencies, from `cargo metadata --no-deps` on `58dc3f6a`:

| crate | depends on |
|---|---|
| `errors`, `parsing`, `query`, `config`, `fixtures` | no crate of purba |
| `reporting` | `errors` |
| `assertion` | `errors`, `parsing` |
| `selection` | `config`, `errors`, `parsing`, `query` |
| `execution` | `assertion`, `errors`, `fixtures` |
| `dispatch` | `errors`, `execution`, `selection` |
| `command` | `assertion`, `config`, `dispatch`, `errors`, `execution`, `parsing`, `reporting`, `selection` |
| `purba` | `command`, `errors`, `execution` |

The Rust tests of each crate, from `cargo nextest list --workspace` (`data/filtersets-errors.txt`, the `all()` block). A row of `lib` is a unit test; a row of `tests/` is a test under `crates/<crate>/tests`.

| crate | unit tests | `tests/` |
|---|---|---|
| `selection` | 2 | 141 |
| `assertion` | 0 | 57 |
| `parsing` | 0 | 54 |
| `execution` | 30 | 44 |
| `command` | 3 | 39 |
| `dispatch` | 0 | 23 |
| `config` | 0 | 17 |
| `query` | 0 | 15 |
| `errors` | 5 | 8 |
| `reporting` | 32 | 0 |
| `fixtures`, `purba` | 0 | 0 |
| all | 72 | 398 |

There are also 8 doc tests: 2 in `errors` and 6 in `selection`, from the baseline `cargo test --workspace`. nextest's list holds none of them: 72 and 398 make its 470.

What each nextest filterset selects for an edit to one crate, of 470 tests in all. `cargo nextest list --workspace -E <expr>`, by `filtersets.sh`; the output is in `data/filtersets-<crate>.txt`.

| edited crate | `package(A)` | `rdeps(A)` | `rdeps(A) - kind(lib)` | `package(A) \| (rdeps(A) - kind(lib))` |
|---|---|---|---|---|
| `errors` | 13 | 384 | 312 | 317 |
| `parsing` | 54 | 393 | 358 | 358 |
| `query` | 15 | 223 | 218 | 218 |

- `rdeps(A)` includes A itself. The last column is the strict reading of criteria 1 and 3: every test of A, and every test of a dependent but its unit tests.
- **[criterion 3]** The strict reading drops 67 tests on an edit to `errors`, 35 on `parsing` and 5 on `query`. Most of purba's tests are under `crates/*/tests`, so the strict reading keeps most of them.
- **[criterion 3]** A looser reading drops every test of an unedited crate, and keeps only the bats tests as communal. On `errors`, that runs 13 Rust tests instead of 384.
- Both readings are now one expression in nextest. I did not time them. nextest applies a filterset to the test binaries it built, so I expect `-E` alone saves the run and not the build (judgement, not measured). The run of every Rust test took 9.5 s on this machine; the build took 24 s.

## 4. The mutation probe [run]

### 4.1 The control

`control.patch` changes `errors::catch` to name its catcher without the first letter: `&catcher[1..]` in place of `catcher`. A unit test of `reporting` builds its error through `errors::catch` ([lib.rs#L397-L400](https://github.com/kalonji-tools/purba/blob/58dc3f6a9746fe2c328811bdb68bfd4132d90de2/crates/reporting/src/lib.rs#L397-L400)) and checks the text `internal error: assertion: …` ([lib.rs#L781-L810](https://github.com/kalonji-tools/purba/blob/58dc3f6a9746fe2c328811bdb68bfd4132d90de2/crates/reporting/src/lib.rs#L781-L810)).

**The control fired.** `cargo test --workspace --no-fail-fast` exits 101, and cargo lists 4 failed targets:

```
error: 4 targets failed:
    `-p purba-dispatch --test hand`
    `-p purba-errors --test tally`
    `-p purba-execution --test panic`
    `-p purba-reporting --lib`
```

`-p purba-reporting --lib` fails on `tests::a_test_that_an_internal_error_stops_prints_it_and_counts_as_blocked`, the test the issue names, and on one more. So the probe sees a kill by a dependent's unit tests. This control is not a dep-lib-only kill: `errors`' own `tests/tally.rs` kills it too. `data/control.txt` holds the output.

A first form of the control replaced `catcher` with a fixed string. It did not build: `catcher` became unused, and `.cargo/config.toml` denies warnings. That is why `mutants.sh` passes `--cap-lints=true`.

### 4.2 Counts

| crate | mutants | unviable | timeout | missed | own only | own + dep | dep-lib only | dep-int only |
|---|---|---|---|---|---|---|---|---|
| `errors` | 66 | 22 | 0 | 0 | 0 | 37 | **3** | 4 |
| `parsing` | 289 | 156 | 2 | 13 | 39 | 67 | **0** | 12 |
| `query` | 79 | 36 | 0 | 4 | 25 | 14 | **0** | 0 |
| all three | 434 | 214 | 2 | 17 | 64 | 118 | **3** | 16 |

- "own" means own-lib, own-int or own-doc; "dep" means dep-lib, dep-int or dep-doc. No mutant is killed by a doc test.
- **No mutant is killed by the unit tests of the edited crate.** Each own kill is from `crates/<edited>/tests`. I checked the raw logs: none names `-p purba-errors --lib`, `-p purba-parsing --lib` or `-p purba-query --lib`.
- Of the 201 caught mutants, the edited crate's tests catch 182. The other 19 need a dependent: 3 its unit tests, 16 its `tests/` folder.
- A dependent's unit tests take part in 40 of the 44 caught mutants of `errors`, and in none of `parsing` or `query`. Only `reporting`, `execution`, `command`, `errors` and `selection` have unit tests.
- A timeout is a test that ran for 120 s. Both are in `parsing`. Which target hung is not recorded, so I put them in no bucket.
- cargo-mutants marks a mutant unviable when it does not build. Most are a `Default::default()` for a type that has no `Default`.

### 4.3 The mutants that only a dependent kills

**[criterion 3]** The 3 dep-lib mutants:

| mutant | killed by |
|---|---|
| `lib.rs:269:9`: `Tally::add_suppressed` does nothing | `reporting --lib`: `a_case_writes_its_line_then_each_notice_then_selections_warnings_then_executions`, `each_suppressed_verdict_prints_under_its_case_and_changes_no_exit_code` |
| `lib.rs:269:25`: `+=` becomes `-=` | the same two, and `a_case_writes_nothing_after_a_refused_line` |
| `lib.rs:269:25`: `+=` becomes `*=` | the same two as the first |

`Tally::add_suppressed` counts a verdict that a suppression turned into a warning ([lib.rs#L267-L270](https://github.com/kalonji-tools/purba/blob/58dc3f6a9746fe2c328811bdb68bfd4132d90de2/crates/errors/src/lib.rs#L267-L270)). Its one caller is `reporting` ([lib.rs#L178](https://github.com/kalonji-tools/purba/blob/58dc3f6a9746fe2c328811bdb68bfd4132d90de2/crates/reporting/src/lib.rs#L178)). `errors`' own `tests/tally.rs` does not check it.

The 16 dep-int mutants: 4 in `errors/src/panic.rs`, killed by `execution`'s `tests/panic.rs` and `tests/set_hooks.rs`; 12 in `parsing`, killed by `selection`'s `tests/select.rs` and by `assertion`'s, `dispatch`'s and `execution`'s `tests/` files. `data/dep-only.tsv` names each failed test.

### 4.4 The bats tests

`bats.sh` ran `mise run test:wheel` on 36 mutants: each dep-lib, dep-int and missed mutant. A clean run passed 295 of 295 bats tests before the probe, and again after it. So a `not ok` here comes from the mutant, not from a flaky test.

| class | mutants | bats kills | bats passes |
|---|---|---|---|
| dep-lib | 3 | **3** | 0 |
| dep-int | 16 | 5 | 11 |
| missed | 17 | 2 | 15 |

- **[criterion 3]** **The bats tests kill all 3 dep-lib mutants.** Each fails 27 bats tests: the ones whose last line counts a suppressed verdict. One is `tests/expect_raise.bats` line 18, which expects `8 cases: … ; 1 suppressed`. With `+=` turned to `-=`, the wheel panics instead: `attempt to subtract with overflow`.
- bats kills 3 of the 4 dep-int mutants of `errors/src/panic.rs`: `61:5` (10 bats tests fail), `74:24` with `false` (8) and `74:38` (8). They fail the tests of the `fault` wheel, such as "the fault wheel panics in the assert hook when PURBA_FAULT is assert". bats passes `74:24` with `true`.
- bats kills 2 of the 12 dep-int mutants of `parsing`: `lib.rs:646:25` (51) and `lib.rs:675:13` (1). It passes the other 10: `lib.rs` `642:29`, `670:20`, `681:13`, `682:13`, `854:20`, `897:9`, `898:9`, and `facts.rs` `453:40` (both) and `498:13`. Only `selection`'s or `assertion`'s `tests/` files catch those 10.
- bats kills 2 of the 17 missed mutants: `parsing/src/lib.rs:293:9` (12) and `293:74` (8). The other 15 survive every test purba has.
- The first bats run stopped after 19 mutants, on an interrupt of this session. It left the 20th mutant applied. I reverted it with `patch -R`, checked that `git diff` was empty, and ran the last 17 again, split over two worktrees of `58dc3f6a`.

### 4.5 What each reading would have let through

The rows count the 220 mutants that build. "Let through" means that no test the pull request runs fails on it. The timeouts count as caught, since a run past 120 s fails CI's job too (judgement).

`readings.py`, on `data/rows.tsv` and `data/bats.tsv`; the output is in `data/readings.txt`.

| reading | the pull request runs | let through, of 220 | more than today |
|---|---|---|---|
| today | every Rust test, and bats | 15 | 0 |
| every Rust test, without bats | every Rust test | 17 | 2 |
| **strict criterion 3** | the edited crate's tests, the dependents' `tests/` files, and bats | **15** | **0** |
| strict criterion 3, without bats | the edited crate's tests and the dependents' `tests/` files | 20 | 5 |
| loose criterion 3 | the edited crate's tests, and bats | 26 | 11 |
| the edited crate alone | the edited crate's tests | 36 | 21 |

- **[criterion 3]** On this sample, the strict reading lets through nothing that today's CI catches, as long as the bats tests run. The 3 mutants that only a dependent's unit tests catch in Rust are caught by bats too.
- **[criterion 3]** The loose reading treats the dependents' `tests/` files as internals too, and lets 11 more through. 10 of them are in `parsing`, and only `selection`'s or `assertion`'s `tests/` files catch them.
- bats runs on each pull request that changes a path under `crates/`: the job `wheel` starts on the same `build` attribute as the job `rust` ([gates#L1-L3](https://github.com/kalonji-tools/purba/blob/58dc3f6a9746fe2c328811bdb68bfd4132d90de2/.config/gates#L1-L3)). So "as long as the bats tests run" holds for each such pull request today.

## 5. Findings by criterion

- **Criterion 1** (run every test of each edited crate). Every tool and project in section 1 does this. On purba, the edited crate's tests catch 182 of the 201 caught mutants. They all live in `crates/<edited>/tests`.
- **Criterion 2** (run the communal tests). No surveyed tool has a "communal" class. They run every test of each dependent, or none. On purba, the communal tests are the bats tests under `tests/`. Whether `crates/<dependent>/tests` counts as communal is not settled. Such a file tests the dependent through its own public interface, and that interface is built on the edited crate.
- **Criterion 3** (skip the internals of an unedited dependent). **[criterion 3]** No tool in the survey does this by default. nextest can express it as `package(A) | (rdeps(A) - kind(lib))`; Nx can, by test inputs; Buck2 and bazel-diff can trim by distance. On purba, it would skip 67 tests on an edit to `errors`. Three mutants of `errors` are caught in Rust only by those tests. The bats tests catch all three, so on this sample the strict reading loses no kill while bats runs. The loose reading, which also skips the dependents' `tests/` files, loses 11 (section 4.5).
- **Criterion 4** (purba tests no third-party crate's internals). **[criterion 4]** The tools that answer a lockfile change split in two. Nx by default, Turborepo, pytest-testmon, ruff and uv run everything. guppy's `determinator`, Nx's `"auto"` and testtrim diff the resolved packages and run only what uses a changed one. purba runs every Rust test on a change to `Cargo.lock` ([gates#L35](https://github.com/kalonji-tools/purba/blob/58dc3f6a9746fe2c328811bdb68bfd4132d90de2/.config/gates#L35)). The analogy in criterion 4 has one difference on purba, as measured: a third-party crate's tests pass at its release, before purba pins it. A crate of purba is edited in the same pull request that would skip its dependents' tests. The 3 dep-lib mutants show a dependent's unit tests holding part of an edited crate's contract (judgement on the analogy; the counts are [run]).

## 6. Not measured, or not verified

- The time each reading would save in CI. The issue's starting evidence says the `wheel` job ends `CI` later than `rust`, so skipping Rust tests alone does not move the end of `CI`. I did not measure it again.
- Mutants of crates other than `errors`, `parsing` and `query`. The counts answer only cargo-mutants 27.1.0's mutants of those three crates.
- Which target hung in the 2 timeouts.
- The bats tests on the 182 mutants that the edited crate's own tests kill. They do not change the counts above, since those mutants are caught either way.
- The survey's own list of open points: whether `rdeps()` follows an edge that exists only as a dev-dependency; how bazel-diff carries a `MODULE.bazel.lock` change to targets; what cargo-difftests does on a lockfile-only change; testtrim at a commit; Ekstazi beyond its abstract; and whether Nx release 24.0.7 holds the doc text read at `990d341`.

## Files

- `research/affected-crate-tests/control.patch`, `control.sh`: the control.
- `research/affected-crate-tests/mutants.sh`, `classify.py`, `killers.sh`, `deponly.py`: the Rust probe.
- `research/affected-crate-tests/bats.sh`: the bats probe. `readings.py`: the table of section 4.5.
- `research/affected-crate-tests/filtersets.sh`: the nextest counts.
- `research/affected-crate-tests/data/rows.tsv`: one row per mutant, with its class and killers.
- `research/affected-crate-tests/data/dep-only.tsv`: the tests that kill each dep-lib and dep-int mutant.
- `research/affected-crate-tests/data/bats.tsv`: the bats result per mutant.
- `research/affected-crate-tests/data/control.txt`, `data/filtersets-<crate>.txt`: outputs.
