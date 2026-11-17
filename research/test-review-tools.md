# Which tools measure the gaps and the redundancy of purba's tests

Research for [Which tools measure the gaps and the redundancy of a test suite in Rust, in bats and in Python, and what does each cost a run?](https://github.com/kalonji-tools/purba/issues/1058).
Measured on 2026-11-17, against purba `main` at `ee8ac272`, on one Linux machine with 22 cores.
This file picks no tool. The grilling that follows, [#1059](https://github.com/kalonji-tools/purba/issues/1059), picks.

## Summary

A tool can measure code that no test reaches in each of purba's five layers. Only the Rust crates have a tool for a change that no test notices. No layer has a tool for redundancy.

Coverage is cheap on purba. cargo-llvm-cov runs the Rust tests with coverage in about the time `test:rust` takes. It also counts the Rust code that the wheel's bats tests reach, and rustc's own flag counts the cargo script. It reads one thing wrong on purba: a child that purba forks ends with `_exit`, so it never writes its counts. Everything a forked child runs reads as unreached. kcov counts the lines of purba's bash scripts, but only when kcov starts the script itself. Started around `bats`, it counts nothing of purba's scripts.

Mutation testing is the only measure of "a change that no test notices". cargo-mutants does that for the Rust crates and nothing else. Run as it comes, it gives a false picture of purba. purba denies every warning, in two places, so most mutants fail to build. It also tests only the mutated crate, so a mutant that another crate's tests catch reads as missed. Of 43 missed mutants in one change, 34 were false for these reasons. With three settings, the 185 mutants of that change took 7.2 minutes. It cannot run the wheel's bats tests, so it reads as missed what only those tests catch. Of four missed mutants checked against the wheel by hand, the wheel caught one.

No tool measures redundancy, "two tests that check the same thing", on purba. The nearest measure is a map of the lines each test reaches. coverage.py builds it for Python; for Rust and bash, a probe script builds it here. On `check-origin.sh` six tests reach the very same lines, yet each checks a different input. In `purba-assertion`, two pairs of tests do the same. Equal lines do not mean an equal check. The other near measure, which test kills which mutant, no tool records. necessist, which deletes statements from a test, supports Rust and finds a statement that no assertion needs. That is a weak test, not a pair of tests.

Python has nothing to measure in purba today. purba holds no Python source and no pytest test. Its 30 Python files are inputs that the wheel's tests run.

## How to read this file

- **Tiers.** [A] = measured or fetched in this session, with the command in [Appendix A](#appendix-a-commands). [B] = a source says so, and this session did not run it. [C] = could not establish.
- **A gap** is code that no test reaches, or a change to the code that no test notices. **Redundancy** is two tests that check the same thing.
- **Times** are wall time of one run on the machine above. *Cold* starts from an empty `target/` with the crates already downloaded. *Warm* reruns with nothing changed.
- **"Missed" and "caught"** are cargo-mutants' words: a mutant is missed when every test still passes with it, and caught when a test fails.
- No tool here was added to purba. Each ran from the scratchpad, through `nix-shell -p` or a download.

## 1. The five layers

| layer | where | task | tests | time of the task, cold / warm | tier |
|---|---|---|---|---|---|
| Rust unit tests | `#[cfg(test)]` in `crates/*/src` | `test:rust` | 33 | 16.8 s / 5.4 s for the whole task | [A] |
| Rust integration tests | `crates/*/tests` | `test:rust` | 196, and 7 doc tests | (in the line above) | [A] |
| bats tests of the scripts | `scripts/test` | `test:shell` | 697 in 38 files | 68.4 s / 37.9 s | [A] |
| tests inside a cargo script | `scripts/check-prose/check-prose.rs` | `test:cargo-scripts` | 78 | 93.0 s / 0.7 s | [A] |
| end-to-end tests of the wheel | `tests/*.bats` | `test:wheel` | 131 in 12 files | 64.8 s / 59.9 s; the bats part alone 54.2 s | [A] |

`test:shell` and `test:cargo-scripts` build the cargo script in release mode first, which is most of their cold time. purba holds no Python source outside `tests/` (`git ls-files '*.py'` gives 30 files, all under `tests/`). Those files are modules that the wheel's tests hand to purba [A].

## 2. Tools that measure a gap

### Rust: unit tests and the tests under `crates/*/tests`

| tool | what it measures | runs on purba | cost on purba | tier |
|---|---|---|---|---|
| **cargo-llvm-cov** 0.8.7 (latest 0.9.1) | lines, regions and functions that no test reached; branches with `--branch`, on nightly only ([README#L11](https://github.com/taiki-e/cargo-llvm-cov/blob/0dc905679ae6b4740b186f6e2c2ef1ab78fb76d7/README.md#L11)) | yes. 81.1% of regions, 86.1% of lines, 87.0% of functions; 75.2% of branches | 20.0 s cold, 5.0 s warm; branches 7.7 s after a clean build; a report 0.3 s. Needs cargo-llvm-cov and the toolchain's `llvm-tools` (40 MB) | [A] |
| **rustc** `-C instrument-coverage`, with `llvm-profdata` and `llvm-cov` | the same counts, with no wrapper. `-Zcoverage-options` takes `block`, `branch` or `condition` ([coverage-options.md#L6-L17](https://github.com/rust-lang/rust/blob/5f4effd579500175af844241eb179e756190f1e7/src/doc/unstable-book/src/compiler-flags/coverage-options.md#L6-L17)) | for the crates, cargo-llvm-cov drives it. Run alone on the cargo script, below | — | [B] for the options |
| **cargo-tarpaulin** 0.37.6 | lines; "Branch coverage: NOT IMPLEMENTED" ([README#L72](https://github.com/xd009642/tarpaulin/blob/550b71ab8ac09b64c4ee32d92f7f3bb93a766e22/README.md#L72)) | not run | unmeasured | [B] |
| **cargo-mutants** 27.1.0 | a change that no test notices: it rewrites one piece of code at a time and runs the tests ([mutants.md#L20-L143](https://github.com/sourcefrog/cargo-mutants/blob/8ab1dc786a1f61a4e370416cc6c68b81a704e917/book/src/mutants.md#L20-L143)) | yes, on the lines of one change (`--in-diff`) | see [§ 4](#4-costs-measured-on-purba) | [A] |
| **necessist** 4.1.0 | a statement of a test that the test passes without ([README#L61](https://github.com/trailofbits/necessist/blob/ebe395d1195a47c34ebb570bdd52de9672708c53/README.md#L61)). It runs each test with `cargo test ... -- --exact <name>` ([mod.rs#L659-L683](https://github.com/trailofbits/necessist/blob/ebe395d1195a47c34ebb570bdd52de9672708c53/backends/src/rust/mod.rs#L659-L683)) | yes, with `--framework rust`: purba's `pyproject.toml` makes it find Python too, and it refuses to choose. On `parse.rs` and `select.rs`: 47 removals, 10 passed, 5 failed, 32 did not build | 21.8 s for the 47; installing it took about 140 s of compiling, and it needs SQLite at build and at run time | [A] |

The coverage gaps cluster in three files. `execution/src/zygote.rs` reads 32% of lines, `execution/src/reader.rs` 26%, and `purba/src/lib.rs` 0%. The first two are mostly false; [§ 5](#5-what-each-measure-misses) says why. `purba/src/lib.rs` is the entry point that Python calls, and only the wheel's tests call it [A].

necessist's 10 passed removals need a reader's judgement, as its README warns. Five remove a thread's `.name(...)`, which no test reads, and one a `drop(module);`. One removes `assert_refused(name, bytes, encoding);` in `parse.rs` line 228: the call is the check, so without it the test checks nothing. Two remove `.stack_size(SMALL_STACK)`, and one `.repeat(20_000)` in `select.rs`: each makes the test's input easier, so the test still passes. That last kind is the one necessist exists for. Nothing asserts that the expression is deep, so the test would pass on a shallow one [A].

### bats tests of the scripts

| tool | what it measures | runs on purba | cost on purba | tier |
|---|---|---|---|---|
| **kcov** 43 | bash lines that no test reached. It traces through `BASH_ENV`, `PS4` and `BASH_XTRACEFD` ([bash-engine.cc#L198-L209](https://github.com/SimonKagstrom/kcov/blob/a39874f938ce13f7a65f253120d1ec946b349ffe/src/engines/bash-engine.cc#L198-L209)) | **only when kcov starts the script.** Around `bats`, it counted bats's own files and the test file, and 0 lines of `check-origin.sh`. With a wrapper that starts each call of the script under kcov, it counted 24 of 25 lines | `check-origin.bats`, 15 tests: 1.4 s plain, 2.4 s under the wrapper, 0.2 s to merge. The whole suite: unmeasured, because each `.bats` file must hand its script through the wrapper | [A] |
| **bashcov** 4.0.3 | bash lines, through `xtrace` and `PS4` ([runner.rb#L40-L57](https://github.com/infertux/bashcov/blob/1b426a5f728665c26ead73910931d05a1ec007d8/lib/bashcov/runner.rb#L40-L57)). It names Bats as supported ([README#L12-L22](https://github.com/infertux/bashcov/blob/1b426a5f728665c26ead73910931d05a1ec007d8/README.md#L12-L22)) | not run: nixpkgs has no package, and it needs Ruby | unmeasured | [B] |
| a mutation tool for bash | — | none established. A GitHub search found four repositories, each with no star and created in 2026, such as `darettau/shmutate` | — | [B] |

### Tests inside a cargo script

| tool | what it measures | runs on purba | cost on purba | tier |
|---|---|---|---|---|
| **rustc's own coverage**: `RUSTFLAGS="-C instrument-coverage"`, then `llvm-profdata merge` and `llvm-cov report` ([instrument-coverage.md](https://github.com/rust-lang/rust/blob/5f4effd579500175af844241eb179e756190f1e7/src/doc/rustc/src/instrument-coverage.md)) | lines, regions and functions | yes. Its own 78 tests reach 93.8% of `check-prose.rs`'s lines. The 98 tests of `check-records.bats`, run against an instrumented binary, reach 94.2% of the lines the binary holds | the instrumented build and the 78 tests: 131.7 s cold, against 93.0 s for a plain `test:cargo-scripts`. `check-records.bats` with the instrumented binary: 95.3 s, against 30.1 s with the plain one. Needs the same `llvm-tools` | [A] |
| **cargo-mutants** 27.1.0 | — | **no.** Given `--manifest-path scripts/check-prose/check-prose.rs` inside the workspace, it mutated the workspace's crates instead. Given the script alone in an empty directory, it stopped: "could not find `Cargo.toml`". Its source requires a directory that holds `Cargo.toml` ([workspace.rs#L92-L99](https://github.com/sourcefrog/cargo-mutants/blob/8ab1dc786a1f61a4e370416cc6c68b81a704e917/src/workspace.rs#L92-L99)) | — | [A] |
| **necessist** 4.1.0 | — | no: its Rust backend applies only where the root holds `Cargo.toml` ([mod.rs#L47-L48](https://github.com/trailofbits/necessist/blob/ebe395d1195a47c34ebb570bdd52de9672708c53/backends/src/rust/mod.rs#L47-L48)) | — | [B] |

### End-to-end tests of the wheel

| tool | what it measures | runs on purba | cost on purba | tier |
|---|---|---|---|---|
| **cargo-llvm-cov** `show-env` | the Rust code that any program reaches, once the build carries its flags. The README names "external tests that use make, xtask, etc." ([README#L487-L507](https://github.com/taiki-e/cargo-llvm-cov/blob/0dc905679ae6b4740b186f6e2c2ef1ab78fb76d7/README.md#L487-L507)). Its maintainer says it "works with PyO3" ([#365](https://github.com/taiki-e/cargo-llvm-cov/issues/365#issuecomment-2073644869)) | yes. The 131 tests reach 52.4% of the crates' lines; `purba/src/lib.rs` 88.6%. 179 lines are reached by the wheel's tests and by no `cargo test`: 69 in `purba/src/lib.rs`, 49 in `execution/src/zygote.rs` | both wheels built with its flags: 10.3 s; the bats tests: 63.1 s, against 54.2 s plain; a report 0.2 s | [A] |
| **cargo-mutants** | — | **no.** It takes only `cargo` or `nextest` as the test tool ([options.rs#L226-L233](https://github.com/sourcefrog/cargo-mutants/blob/8ab1dc786a1f61a4e370416cc6c68b81a704e917/src/options.rs#L226-L233)). Its own docs: for "an integration test not run by `cargo test`", it "will only tell you about gaps in the in-tree tests" ([limitations.md#L9-L12](https://github.com/sourcefrog/cargo-mutants/blob/8ab1dc786a1f61a4e370416cc6c68b81a704e917/book/src/limitations.md#L9-L12)). Issue [#413](https://github.com/sourcefrog/cargo-mutants/issues/413), for another test command, is open | — | [B] |
| **kcov** | — | the tests run `purba`, a Python entry point and a Rust library, not a bash script | — | [A] |

### Python

| tool | what it measures | runs on purba | tier |
|---|---|---|---|
| **coverage.py** 7.17.3 | lines, and branches that did not take each way ([branch.rst#L14-L16](https://github.com/coveragepy/coveragepy/blob/e6580b2746f32a783b16542812f68f1562d51c56/doc/branch.rst#L14-L16)) | nothing to measure: purba has no Python source | [A] that there is none |
| **mutmut** 3.8.0 | a change no test notices, in Python functions; it runs pytest with `-x` ([harness.py#L140](https://github.com/boxed/mutmut/blob/14a7230049a5c8abd90c2bb0f7438e30da6471f5/src/mutmut/runners/harness.py#L140)) | nothing to mutate | [B] |
| **cosmic-ray** 8.9.0 | the same, with any `test-command` ([concepts.rst#L169-L170](https://github.com/sixty-north/cosmic-ray/blob/6c490ce16443d0c6c11edd6e6d36c34b153c8607/docs/source/concepts.rst#L169-L170)) | nothing to mutate | [B] |
| **necessist**, Python backend | a test statement that pytest passes without ([README#L374-L378](https://github.com/trailofbits/necessist/blob/ebe395d1195a47c34ebb570bdd52de9672708c53/README.md#L374-L378)) | purba has no pytest test. Yet `--framework python --dump-candidate-counts` finds removal candidates in `tests/rules/test_try.py` and others: those are inputs the wheel's tests hand to purba, named like pytest tests | [A] |

Each of these would apply if purba gained Python source, or pytest tests of its own.

## 3. Tools that measure redundancy

No tool on the list reports "these two tests check the same thing". Three measures come near it.

| measure | where a tool gives it | what it says | tier |
|---|---|---|---|
| **the lines each test reaches** | coverage.py's dynamic contexts: "most commonly used to answer the question 'what test ran this line?'" ([contexts.rst#L53-L57](https://github.com/coveragepy/coveragepy/blob/e6580b2746f32a783b16542812f68f1562d51c56/doc/contexts.rst#L53-L57)); pytest-cov `--cov-context=test` ([contexts.rst#L5-L17](https://github.com/pytest-dev/pytest-cov/blob/66c8a526b1246b5eb8fb1bc218878131bc628622/docs/contexts.rst#L5-L17)). For Rust and bash, no tool: cargo-llvm-cov has no option for it [B], and the probes below build it | two tests that reach the same lines, or one test whose lines another test's lines contain | [A] probes |
| **the mutants each test kills** | none. cargo-mutants stops at the first failing test and records only whether a mutant was caught ([fail-fast.md#L3](https://github.com/sourcefrog/cargo-mutants/blob/8ab1dc786a1f61a4e370416cc6c68b81a704e917/book/src/fail-fast.md#L3), [mutants-out.md#L20-L30](https://github.com/sourcefrog/cargo-mutants/blob/8ab1dc786a1f61a4e370416cc6c68b81a704e917/book/src/mutants-out.md#L20-L30)). mutmut carries the wish as a TODO: "Those that kill zero mutants are redundant!" ([__main__.py#L947](https://github.com/boxed/mutmut/blob/14a7230049a5c8abd90c2bb0f7438e30da6471f5/src/mutmut/__main__.py#L947)) | a test that kills no mutant another test does not also kill | [B] |
| **a statement a test passes without** | necessist | one test carries a statement no assertion needs. That is a weak test, not two tests that check one thing | [B] |

### The probes on purba

**bash, `check-origin.bats`.** The kcov wrapper names each run by its bats test number. Of 15 tests, 11 reach only lines that another test also reaches. Six tests reach exactly the same 20 lines: numbers 2, 3, 4, 7, 9 and 10 [A]. They are not one check six times. Test 2 refuses a commit with no sign-off, test 3 a sign-off in the body, and test 4 an address that reaches no one. The difference lies in the trailer text that the script's `grep` reads, not in which lines of the script run.

**Rust, one crate.** [`per-test-coverage.sh`](test-review-tools/per-test-coverage.sh) runs each test of one package alone under coverage, and compares the line sets. On `purba-assertion`, 45 tests ran one at a time in 31.3 s, 50 s with the build. 13 of the 45 reach only lines that another test also reaches. Two pairs reach exactly the same lines [A]:

- `each_assert_records_its_index` and `a_tuple_assert_stays_as_written_and_takes_no_index`. One checks the index each assert records; the other checks that a tuple assert keeps its form.
- `a_new_name_leaves_sys_modules_when_its_body_raises` and `a_body_that_calls_sys_exit_leaves_no_entry`. One body raises; the other calls `sys.exit`.

Each pair is two checks, not one. The lines are the same because the code is shared; the inputs and the assertions differ.

The literature says the same thing from the other side. Removing the tests whose coverage another test repeats "can severely compromise" fault detection, in Rothermel and others' study ([doi:10.1109/ICSM.1998.738487](https://doi.org/10.1109/ICSM.1998.738487)). Wong and others found "little or no reduction" under the same rule, on other programs ([doi:10.1002/(SICI)1097-024X(19980410)28:4<347::AID-SPE145>3.0.CO;2-L](https://doi.org/10.1002/(SICI)1097-024X(19980410)28:4%3C347::AID-SPE145%3E3.0.CO;2-L)). The two results disagree, so equal coverage does not settle redundancy [B].

## 4. Costs measured on purba

| tool | command, in short | wall time | what it needed | tier |
|---|---|---|---|---|
| `test:rust`, for scale | `mise run test:rust` | 16.8 s cold, 5.4 s warm | purba's own tools | [A] |
| cargo-llvm-cov, lines and regions | `cargo llvm-cov --workspace --all-targets --no-report` | 20.0 s cold, 5.0 s warm | `nix-shell -p cargo-llvm-cov` (0.8.7); `llvm-tools-nightly` of 2026-11-16, 40 MB, from static.rust-lang.org, named by `LLVM_COV` and `LLVM_PROFDATA` | [A] |
| cargo-llvm-cov, branches | the same with `--branch`, after `cargo llvm-cov clean --workspace` | 7.7 s | the same | [A] |
| cargo-llvm-cov, the wheel | `show-env`, then `test:wheel`'s own steps with the `fault` wheel built first, then `report` | 10.3 s to build both wheels, 63.1 s for the bats tests (54.2 s plain), 0.2 s to report | the same | [A] |
| cargo-llvm-cov, one test at a time | `per-test-coverage.sh purba-assertion` | 50 s for 45 tests, 31.3 s of it the runs | the same, and `jq` | [A] |
| rustc coverage, the cargo script | `RUSTFLAGS="-C instrument-coverage" cargo -Zscript test`, then `llvm-cov report` | 131.7 s cold (93.0 s plain); its bats tests 3.2 times slower | `llvm-tools` only | [A] |
| cargo-mutants, one change, as it comes | `cargo mutants --in-diff <#1037's diff> --jobs 4` | 140 s | `nix-shell -p cargo-mutants` (27.1.0) | [A] |
| cargo-mutants, one change, corrected | the same with `--cap-lints=true --test-workspace=true`, and `CARGO_BUILD_WARNINGS=allow` | 433 s, about 4.3 s for each of the 100 mutants that built | the same | [A] |
| kcov, one bats file through a wrapper | `bats` on a copy of `check-origin.bats` whose `script` is the wrapper | 2.4 s against 1.4 s plain | `nix-shell -p kcov` (43); `--bash-parser` set, because NixOS has no `/bin/bash` | [A] |
| necessist, two test files | `necessist --framework rust crates/parsing/tests/parse.rs crates/selection/tests/select.rs`, on a copy of HEAD | 21.8 s for 47 removals; 32 did not build, so ran no test | `cargo install necessist` 4.1.0 into the scratchpad, about 140 s; SQLite from nixpkgs | [A] |

The change measured is [#1037](https://github.com/kalonji-tools/purba/issues/1037), commit `6595c634`, the last commit to touch `crates/`. It changed 766 lines of `crates/*/src` and gave 185 mutants.

Mutating only the lines of one change is how Google brought mutation testing into code review. Petrović and Ivanković describe "a diff-based probabilistic approach" that also skips lines no test covers ([doi:10.1145/3183519.3183521](https://doi.org/10.1145/3183519.3183521)) [B]. `--in-diff` is cargo-mutants' form of the first half: it "tests only mutants that overlap with regions changed in the diff" ([in-diff.md#L5](https://github.com/sourcefrog/cargo-mutants/blob/8ab1dc786a1f61a4e370416cc6c68b81a704e917/book/src/in-diff.md#L5)).

### Three settings that cargo-mutants needs on purba

The same 185 mutants, three runs:

| run | settings | caught | missed | unviable | wall time |
|---|---|---|---|---|---|
| as it comes | `--in-diff` `--jobs 4` | 14 | 43 | 128 | 140 s |
| lints capped, every crate's tests | and `--cap-lints=true --test-workspace=true` | 49 | 8 | 128 | 262 s |
| and cargo's warnings allowed | and `CARGO_BUILD_WARNINGS=allow` | 88 | 12 | 85 | 433 s |

No mutant timed out in any run [A].

**A mutant runs only its own crate's tests.** "By default, each mutant runs only the tests from the package that's being mutated" ([workspaces.md#L41-L43](https://github.com/sourcefrog/cargo-mutants/blob/8ab1dc786a1f61a4e370416cc6c68b81a704e917/book/src/workspaces.md#L41-L43)). purba's crates test one another. With `--test-workspace=true`, missed mutants fell from 43 to 8, on the same set of mutants that built [A].

**purba denies every warning, twice.** The workspace's `Cargo.toml` sets the lint `warnings = "deny"`, and `.cargo/config.toml` sets cargo's own `build.warnings = "deny"`. A mutant that replaces a function's body leaves an import or a variable unused. cargo-mutants's docs name the case: such trees show "an excessive number of mutants being reported as 'unviable'", and the remedy is `--cap-lints=true` ([lints.md#L5-L10](https://github.com/sourcefrog/cargo-mutants/blob/8ab1dc786a1f61a4e370416cc6c68b81a704e917/book/src/lints.md#L5-L10)). On purba that remedy changed nothing: 128 unviable before and after. The logs say why: "warnings are denied by `build.warnings` configuration". With `CARGO_BUILD_WARNINGS=allow` beside `--cap-lints=true`, 43 more mutants built. Each setting answers one of the two denials. The wheel probe of § 5 shows the other half: with `CARGO_BUILD_WARNINGS=allow` alone, a mutant failed on "unused variable" [A]. The 85 left are type errors, mostly where a function's body became a default value of a type that has none [A].

The settings change what a missed mutant means. Of the 43 that the first run missed, 34 are caught in the third run; only 9 stay missed. The third run's other 3 missed mutants did not build before. A reviewer that reads the first run reads 34 false gaps [A].

### What #66 measured, and what this adds

[#66](https://github.com/kalonji-tools/purba/issues/66) measured oxitest's history, not purba's tree. It found that cargo-mutants yielded at most 1 of 16 mutation findings there; a hand-anchored applier yielded 15. It also found that cargo-mutants "offers no arbitrary test command", and that "a mutation verdict needs three outcomes, not two" [A, from #66's comment]. Both still hold for 27.1.0: the test tool is `cargo` or `nextest` only, and an outcome is caught, missed, timeout or unviable. This file adds the first run of cargo-mutants on purba's own tree.

## 5. What each measure misses

| measure | what it misses | source | tier |
|---|---|---|---|
| line and region coverage | that a line which ran was checked. A line runs and no assertion reads its effect | Inozemtseva and Holmes: "a low to moderate correlation between coverage and effectiveness when the number of test cases in the suite is controlled for" ([doi:10.1145/2568225.2568271](https://doi.org/10.1145/2568225.2568271)) | [B] |
| coverage of purba's Rust | **all the code a forked child runs.** LLVM writes the counts "when the program exits" ([SourceBasedCodeCoverage.rst#L75](https://github.com/llvm/llvm-project/blob/2078da43e25a4623cab2d0d60decddf709aaea28/clang/docs/SourceBasedCodeCoverage.rst#L75)), through an exit handler. `_exit` "shall not call functions registered with atexit()" ([POSIX](https://pubs.opengroup.org/onlinepubs/9799919799/functions/_exit.html)). purba's zygote and reader end each child with `libc::_exit`. cargo-llvm-cov's maintainer names "exiting in an unsupported way" as a cause of empty coverage ([#365](https://github.com/taiki-e/cargo-llvm-cov/issues/365#issuecomment-2073644869); [#235](https://github.com/taiki-e/cargo-llvm-cov/issues/235), "No coverage from killed subprocesses", is open). In `zygote.rs`, the parent reached the `fork` 25 times, and the `Forked::Child` arm reads 0, though the tests that need it pass | the coverage JSON, `zygote.rs` lines 107 to 125 | [A] |
| coverage, branches | MC/DC. rustc removed it in August 2025 ([rust#144999](https://github.com/rust-lang/rust/pull/144999)). cargo-llvm-cov still offers `--mcdc` ([main.rs#L177](https://github.com/taiki-e/cargo-llvm-cov/blob/0dc905679ae6b4740b186f6e2c2ef1ab78fb76d7/src/main.rs#L177)) | | [B] |
| coverage, doc tests | the 7 doc tests. cargo-llvm-cov: "Support for doc tests is unstable and has known issues" ([README#L914-L917](https://github.com/taiki-e/cargo-llvm-cov/blob/0dc905679ae6b4740b186f6e2c2ef1ab78fb76d7/README.md#L914-L917)). The runs here leave them out | | [A] that they were left out |
| kcov on bash | a script that `bats` starts, unless kcov starts it. Open issues name the case: "Coverage not captured for Bash functions tested by Bats" ([#476](https://github.com/SimonKagstrom/kcov/issues/476)), and "Changing current directory results in no coverage" ([#479](https://github.com/SimonKagstrom/kcov/issues/479)). It also reads a string assignment continued with `\` as unreached: `check-origin.sh` line 45 holds 0 hits, though test 2 asserts the text it assigns | | [A] |
| mutation, cargo-mutants | code that only tests outside `cargo test` reach, such as the wheel's bats tests ([limitations.md#L9-L12](https://github.com/sourcefrog/cargo-mutants/blob/8ab1dc786a1f61a4e370416cc6c68b81a704e917/book/src/limitations.md#L9-L12)). A mutant that changes nothing: a survivor "might mean we accidentally generated a no-op mutation" ([how-it-works.md#L33-L34](https://github.com/sourcefrog/cargo-mutants/blob/8ab1dc786a1f61a4e370416cc6c68b81a704e917/book/src/how-it-works.md#L33-L34)). Code made by macros, and code for another platform ([limitations.md#L20-L41](https://github.com/sourcefrog/cargo-mutants/blob/8ab1dc786a1f61a4e370416cc6c68b81a704e917/book/src/limitations.md#L20-L41)). With `--in-diff`, "a diff that only deletes or changes test code won't cause any mutants to run" ([in-diff.md#L22](https://github.com/sourcefrog/cargo-mutants/blob/8ab1dc786a1f61a4e370416cc6c68b81a704e917/book/src/in-diff.md#L22)) | | [B]; [A] for the wheel case below |
| mutation, any tool | redundant mutants inflate the score: Ammann, Delamaro and Offutt show "large numbers of redundant mutants" ([doi:10.1109/ICST.2014.13](https://doi.org/10.1109/ICST.2014.13)) | | [B] |
| the lines each test reaches | that two tests reaching the same lines check different inputs. Six tests of `check-origin.sh` show it | § 3 | [A] |
| necessist | "Slow. Modifying tests requires them to be rebuilt." And "Triage requires intimate knowledge of the source code" ([README#L636-L640](https://github.com/trailofbits/necessist/blob/ebe395d1195a47c34ebb570bdd52de9672708c53/README.md#L636-L640)). It never removes the last statement of a test ([README#L257-L262](https://github.com/trailofbits/necessist/blob/ebe395d1195a47c34ebb570bdd52de9672708c53/README.md#L257-L262)), and in Rust never an `assert!` or `assert_eq!` ([README#L392-L397](https://github.com/trailofbits/necessist/blob/ebe395d1195a47c34ebb570bdd52de9672708c53/README.md#L392-L397)). So an assertion that repeats another stays unseen | | [B] |

### A mutant missed by `cargo test`, caught by the wheel

Four of the 12 missed mutants were each applied to a copy of HEAD by hand. The probe built both wheels as `test:wheel` does, then ran the 131 bats tests [A]:

| mutant | wheel build | bats run | the wheel's verdict |
|---|---|---|---|
| `whence.rs:15:5`: replace `keep_the_folder` with `()` | 4.6 s | 63.7 s | **caught**: test 60, "a case that changes its folder names the frame of its raise from the folder of the run", fails |
| `assertion/src/lib.rs:701:9`: replace `&&` with `\|\|` in `is_iterator` | 17.6 s | 75.3 s | missed |
| `assertion/src/lib.rs:853:36`: replace `>` with `>=` in `read_each` | 7.0 s | 80.6 s | missed |
| `parsing/src/lib.rs:538:5`: replace `is_check -> bool` with `true` | 21.9 s | 64.5 s | missed |

So cargo-mutants' "missed" on purba means "missed by `cargo test`", not "missed by purba's tests". One of four was a false gap. The other three no test of any layer noticed. Whether each is a real gap or a mutant that changes nothing is a reader's call. A wheel check costs about 70 to 100 s for each mutant, against about 4 s inside cargo-mutants. The probe needed the same two settings as cargo-mutants: `CARGO_BUILD_WARNINGS=allow`, and `RUSTFLAGS=--cap-lints=warn` [A].

## 6. genotoxic, Trail of Bits' triage skill

genotoxic is a skill, not a measuring tool. It runs a mutation tool and necessist, then sorts each survivor with a call graph from Trailmark. The sorts are "False Positives", "Missing Tests" and "Fuzzing Targets". A function both tools flag is "corroborated" ([SKILL.md](https://github.com/trailofbits/skills/blob/442fc9d6c89b1e937e6f7a477e7071ea75fcbea4/plugins/trailmark/skills/genotoxic/SKILL.md), read from the local copy at that commit) [A].

For purba it would cover the two Rust layers only. Its Rust mutation tool is cargo-mutants, and necessist runs on purba's Rust tests (§ 2). It inherits what cargo-mutants misses: the cargo script, the bats layers and the wheel. It also inherits the three settings of § 4, which its cargo-mutants section does not name. That section reads survivors with `jq '.[] | select(.outcome == "survived")'` from `outcomes.json` ([mutation-frameworks.md#L172-L190](https://github.com/trailofbits/skills/blob/442fc9d6c89b1e937e6f7a477e7071ea75fcbea4/plugins/trailmark/skills/genotoxic/references/mutation-frameworks.md#L172-L190)). On the corrected run's file this filter finds 0, where the file holds 12 `MissedMutant` outcomes under `.outcomes[].summary` [A]. It names no bash tool. Its Python half would find nothing to mutate, because purba has no Python source. Its necessist step, left to choose, stops on purba: it finds both Python and Rust. Told Python, it would cut statements out of the wheel's input modules [A].

Two of its rules sort a survivor by its callers in the graph: "No callers in graph" and "Only test callers" are false positives. purba's Rust is called from Python through PyO3. Trailmark lists FFI among the "Links Trailmark cannot see", to be declared by hand in `.trailmark/links.toml` ([README#L53-L55](https://github.com/trailofbits/trailmark/blob/2cd94b6938309f3e161a20e70a0df0dd92337d1e/README.md#L53-L55)) [B]. Without that file, a function that only Python calls could be sorted as dead [C: Trailmark was not run]. Trailmark also parses bash ([README#L237](https://github.com/trailofbits/trailmark/blob/2cd94b6938309f3e161a20e70a0df0dd92337d1e/README.md#L237)), but genotoxic pairs it with no bash mutation tool. Its setup says to install with `uv tool install` and `cargo install`; under purba's rule, those would run from a scratch directory, not the repository.

## 7. Answers

1. **Which tools measure a gap.** For the Rust crates: cargo-llvm-cov for code no test reaches, and cargo-mutants for a change no test notices. For the cargo script: rustc's own coverage flag; no mutation tool takes a cargo script. For the bats tests of the scripts: kcov, only when it starts each script, and no established mutation tool. For the wheel: cargo-llvm-cov through `show-env`; no mutation tool runs bats. For Python: coverage.py and mutmut, with nothing in purba to measure.
2. **Which tools measure redundancy.** None directly. coverage.py's contexts give the lines each Python test reaches; for Rust and bash only a probe gives it. Equal lines do not mean an equal check: six bats tests of one script, and two pairs of Rust tests, show it. No tool records which test kills which mutant. necessist finds a statement a test does not need, inside one test.
3. **What each costs.** Coverage of the Rust crates costs about one `test:rust`: 20 s cold, 5 s warm. Coverage of the wheel adds about 16% to its bats run. Mutation of one change cost 7.2 minutes for 185 mutants, with three settings purba needs. kcov adds about 70% to a bats file. The rest is in [§ 4](#4-costs-measured-on-purba). Each tool came from nixpkgs or a download, none from purba's `mise.toml`.
4. **What each misses.** Coverage misses whether a reached line was checked, and on purba every line a forked child runs. Mutation misses what only the wheel's tests catch: one of four missed mutants, on purba. It cannot tell a gap from a mutant that changes nothing. necessist reports noise, such as a thread's name, beside real weak spots, and never removes an `assert!`. kcov misses a script that `bats` starts. A per-test line map misses tests that differ in their input. [§ 5](#5-what-each-measure-misses) has the sources.

## 8. Doubts

### Could not be established [C]

| what | why |
|---|---|
| bashcov on purba | nixpkgs has no package; it needs Ruby and a gem install |
| cargo-tarpaulin on purba | not run. Its LLVM engine is "Unable to handle fork and similar syscalls" ([README#L128](https://github.com/xd009642/tarpaulin/blob/550b71ab8ac09b64c4ee32d92f7f3bb93a766e22/README.md#L128)), and purba forks |
| kcov over the whole of `scripts/test` | each of the 38 files would need its script handed through the wrapper |
| whether Trailmark's graph sees purba's PyO3 entry points | Trailmark was not run |
| a per-test map for the wheel's tests | not tried |

### Other doubts

- **One change, one machine.** The mutation costs are for #1037's diff, on 22 cores with `--jobs 4`. Another change gives another count of mutants.
- **The machine was shared.** Other sessions ran builds and bats suites on it during some runs, for example during the wheel probe of [§ 5](#a-mutant-missed-by-cargo-test-caught-by-the-wheel). Each time is one run, not a median.
- **The fault wheel's runs count only in part.** The report reads the plain build's object. A profile of the `fault` build matches it only in the functions whose code the two builds share. Which functions those are was not checked [C].
- **cargo-llvm-cov 0.8.7, not 0.9.1.** nixpkgs ships 0.8.7. The docs cited are 0.9.1's.
- **Line sets come from region starts.** The per-test probe counts a line as reached when a region with a non-zero count starts on it. llvm-cov's own line count uses a different rule.

## Appendix A: Commands

Scripts lived in the session's scratchpad, `$S` below. Only `per-test-coverage.sh` is committed. `env.sh` sets `LLVM_COV` and `LLVM_PROFDATA` to the downloaded `llvm-tools`, and `LD_LIBRARY_PATH` as `test:rust` sets it.

### A1. The layers, for scale

```sh
mise run test:rust            # twice: from an empty target/, then again
mise run test:cargo-scripts ; mise run test:shell ; mise run test:wheel    # twice each
PATH="$PWD/target/wheel-venv/bin:$PATH" FAULT_PURBA="$PWD/target/fault-venv/bin/purba" bats tests   # the bats part alone
git ls-files '*.py' | grep -v '^tests/'                                    # no output
```

### A2. The tools

```sh
nix-shell -p cargo-mutants cargo-llvm-cov kcov      # cargo-mutants 27.1.0, cargo-llvm-cov 0.8.7, kcov 43
curl -sSfLO https://static.rust-lang.org/dist/2026-11-16/llvm-tools-nightly-x86_64-unknown-linux-gnu.tar.xz
# its `version` file: 1.102.0-nightly (5f4effd57 2026-11-13), the same commit as `rustc -vV`
export LLVM_COV=$B/llvm-cov LLVM_PROFDATA=$B/llvm-profdata
```

### A3. Coverage of the Rust crates

```sh
cargo llvm-cov --workspace --all-targets --no-report     # from an empty llvm-cov-target, then again
cargo llvm-cov report --summary-only
cargo llvm-cov clean --workspace                          # without it, --branch reuses the build and counts no branch
cargo llvm-cov --workspace --all-targets --branch --no-report
cargo llvm-cov report --branch --summary-only
jq '.data[0].files[] | select(.filename | endswith("execution/src/zygote.rs")) | .segments' cov.json   # lines 100 to 125
```

### A4. kcov on the bats tests

```sh
kcov --bash-parser="$(command -v bash)" --include-path="$PWD/scripts" "$out" bats scripts/test/check-origin.bats
# 0 lines of scripts/; the same with --bash-method=DEBUG, and with no --include-path
# the wrapper: a copy of check-origin.bats whose `script=` names wrap.sh, which runs
exec kcov --bash-parser="$(command -v bash)" --include-path="$W/scripts" "$runs/t${BATS_TEST_NUMBER}-$$-$RANDOM" "$W/scripts/check-origin.sh" "$@"
kcov --merge "$merged" "$runs"/*
# per test: the lines with hits > 0 in each run's cobertura.xml, grouped by test number, compared with comm
```

### A5. cargo-mutants on one change

```sh
git diff 6595c634^ 6595c634 > diff-1037.patch
cargo mutants --list --in-diff diff-1037.patch                     # 185
cargo mutants --in-diff diff-1037.patch --jobs 4                   # as it comes
cargo mutants --in-diff diff-1037.patch --jobs 4 --cap-lints=true --test-workspace=true
CARGO_BUILD_WARNINGS=allow cargo mutants --in-diff diff-1037.patch --jobs 4 --cap-lints=true --test-workspace=true
# each under `timeout 2400`, inside `mise exec --` so that PYO3_PYTHON and LD_LIBRARY_PATH hold
cargo mutants --list --manifest-path scripts/check-prose/check-prose.rs     # lists the workspace's crates
cd "$(mktemp -d)" && cp .../check-prose.rs .../Cargo.lock . && cargo mutants --list --manifest-path check-prose.rs   # could not find `Cargo.toml`
```

### A6. Sources fetched

```sh
gh api "repos/sourcefrog/cargo-mutants/contents/book/src/<page>.md?ref=8ab1dc786a1f61a4e370416cc6c68b81a704e917" -H 'Accept: application/vnd.github.raw'
gh api "repos/taiki-e/cargo-llvm-cov/contents/README.md?ref=0dc905679ae6b4740b186f6e2c2ef1ab78fb76d7" -H 'Accept: application/vnd.github.raw'
gh api "repos/trailofbits/necessist/contents/README.md?ref=ebe395d1195a47c34ebb570bdd52de9672708c53" -H 'Accept: application/vnd.github.raw'
gh api "repos/trailofbits/necessist/contents/backends/src/rust/mod.rs?ref=ebe395d1195a47c34ebb570bdd52de9672708c53" -H 'Accept: application/vnd.github.raw'
gh api "repos/trailofbits/trailmark/contents/README.md?ref=2cd94b6938309f3e161a20e70a0df0dd92337d1e" -H 'Accept: application/vnd.github.raw'
gh api "repos/boxed/mutmut/contents/src/mutmut/__main__.py?ref=14a7230049a5c8abd90c2bb0f7438e30da6471f5" -H 'Accept: application/vnd.github.raw'
gh api "repos/coveragepy/coveragepy/contents/doc/contexts.rst?ref=e6580b2746f32a783b16542812f68f1562d51c56" -H 'Accept: application/vnd.github.raw'
gh api "repos/llvm/llvm-project/contents/clang/docs/SourceBasedCodeCoverage.rst?ref=llvmorg-21.1.8" -H 'Accept: application/vnd.github.raw'
curl -sS https://pubs.opengroup.org/onlinepubs/9799919799/functions/_exit.html
```

The rest of the sources (kcov, bashcov, tarpaulin, pytest-cov, cosmic-ray, rustc's coverage docs, and the papers) were read by a helper agent of this session at the SHAs and DOIs cited. They are [B] where this file did not run what they describe.

### A7. Coverage of the wheel's tests

```sh
source <(cargo llvm-cov show-env --sh)
cargo llvm-cov clean --workspace
maturin build --quiet --features extension-module,fault --out "$M/cov-wheels-fault"   # first, so target/debug keeps the plain build
maturin build --quiet --out "$M/cov-wheels"
# two venvs, as test:wheel makes them
PATH="$M/cov-venv/bin:$PATH" FAULT_PURBA="$M/cov-fault-venv/bin/purba" bats tests      # 131 ok, 102 profiles
cargo llvm-cov report --summary-only        # maturin builds the debug profile here; `--release` finds no object
# lines reached only by the wheel: region-start lines with a count > 0, compared with comm against A3's JSON
```

### A8. One test at a time

```sh
bash research/test-review-tools/per-test-coverage.sh purba-assertion "$M/pertest-assertion"
```

### A9. Coverage of the cargo script

```sh
RUSTFLAGS="-C instrument-coverage" LLVM_PROFILE_FILE="$D/prof-test/%p-%m.profraw" \
  cargo -Zscript test --release --locked --manifest-path scripts/check-prose/check-prose.rs --target-dir "$D/target" \
  --config "resolver.lockfile-path=\"$PWD/scripts/check-prose/Cargo.lock\""
RUSTFLAGS="-C instrument-coverage" cargo -Zscript build <the same flags>
PURBA_CHECK_PROSE="$D/target/release/check-prose" LLVM_PROFILE_FILE="$D/prof-bats/%p-%m.profraw" \
  bats --jobs 44 scripts/test/check-records.bats
llvm-profdata merge -sparse "$D"/prof-test/*.profraw -o test.profdata       # and the same for prof-bats
llvm-cov report -instr-profile test.profdata "$D"/target/release/build/check-prose/*/out/check_prose-* scripts/check-prose/check-prose.rs
llvm-cov report -instr-profile bats.profdata "$D"/target/release/check-prose scripts/check-prose/check-prose.rs
PURBA_CHECK_PROSE=target/scripts/release/check-prose bats --jobs 44 scripts/test/check-records.bats   # plain, for scale
```

### A10. Missed mutants against the wheel

```sh
# for each mutant: a fresh copy of HEAD, its diff from mutants.out/diff/, both wheels, the 131 bats tests
git archive HEAD | tar -x -C "$C" && (cd "$C" && patch -p0 < "$diff")
CARGO_BUILD_WARNINGS=allow RUSTFLAGS=--cap-lints=warn maturin build --quiet --out "$C/w"   # and the fault wheel
PATH="$C/v/bin:$PATH" FAULT_PURBA="$C/vf/bin/purba" bats tests
```

### A11. necessist

```sh
nix-shell -p sqlite pkg-config --run 'cargo install --locked --version 4.1.0 --root "$S/tools/necessist" necessist'
# on a throwaway copy of HEAD; LD_LIBRARY_PATH also holds SQLite's lib directory
necessist --dump-candidate-counts                       # "found multiple applicable frameworks: python, rust"
necessist --framework python --dump-candidate-counts    # candidates in tests/*/test_*.py, the wheel's inputs
necessist --framework rust --verbose crates/parsing/tests/parse.rs crates/selection/tests/select.rs
```
