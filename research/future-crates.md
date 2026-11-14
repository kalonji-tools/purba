# The needs of the parts not designed yet, and the crates that could meet each

Research for [#961](https://github.com/kalonji-tools/purba/issues/961).
Measured on 2026-11-14, against purba `main` at `2790c1e`. This file picks no crate, ranks none and gives no verdict.

## Summary

Forty-one crates could meet 23 needs of the eleven parts and concerns not designed yet.
Seventeen of those needs are stated in the landscape, a record or an open issue.
Six are inferred: no line states them, and the design of each part can confirm or drop them.
Seven more stated needs have no candidate, because std or a crate purba already depends on meets them.
Two stated needs have nothing in std or in purba's crates that meets them: decoding a module by a declared encoding other than UTF-8, and writing JUnit XML.
A third, reading a configuration file, is met by nothing once its format is chosen, and no line states the format.

Each candidate was measured against the four checks and eight of the nine inputs of [#959](https://github.com/kalonji-tools/purba/issues/959#issuecomment-6874200802).
Trust and relief are left to a person. This file gives the evidence for each.

- **Checks.** Eleven candidates fail a check. One, `encoding`, fails the advisory check: RustSec marks it unmaintained, with no patched version. All eleven fail purba's licence gate. Seven fail on their own licence, and four on a crate they bring. Every candidate passes the source check.
- **Owner in purba is 3 for all 41.** Dependabot covers GitHub Actions only, and no record names an owner for a crate purba has not adopted.
- **Upstream review is the weakest input.** 30 of 41 score 3. Three score 1: `tokio`, `pyo3-async-runtimes` and `seccompiler`. The repository of `seccompiler` is archived.
- **Tests are strong.** 33 of 41 score 1, with an observed CI run. Eight score 2, and none scores 3.
- **`unsafe` splits.** 23 score 1, 4 score 2, and 14 score 3.
- **Blast area is a floor for every candidate.** No part has a design, so nobody can tell whether another part would name a crate's types. Only `pyproject-toml` (66 crates) and `pep508_rs` (57) reach 3, by size alone.
- **Pain** runs from 1.29 to 2.57 under the primary readings. Several readings of the method move a level, and § 3 gives the pain under each.

Some facts bear on trust. The repository of `seccompiler` is archived. `textcode` has about 1/50 the downloads of the next-smallest candidate. `chumsky` 0.14.1 was published from a dirty tree. `encoding` 0.2.33 was cut from a side branch.

## How to read this file

- **Tiers.** [A] = fetched this session, with the command in [Appendix B](#appendix-b-commands). [B] = relayed and not re-verified. [C] = could not establish. A tier comes from the input file that measured it, and is never upgraded here.
- **Scope.** Target Linux x86_64 (`x86_64-unknown-linux-gnu`). Default features. The newest stable version of each crate on crates.io on 2026-11-14. purba `main` at `2790c1e`.
- **Method.** The method of the #960 audit ([audit](https://github.com/kalonji-tools/purba/issues/960#issuecomment-6874866548)), applied to the criteria of the [#959 spec](https://github.com/kalonji-tools/purba/issues/959#issuecomment-6874200802). Five slices measured the inputs in parallel: *checks* (the four checks, upkeep, maintainers, API churn, owner), *review-a* and *review-b* (upstream review, tests), *blast-a* and *blast-b* (blast area, `unsafe`). The needs and candidates come from the *needs* slice.
- **Levels.** 1 to 3, where 3 is the worst. Relief and trust are a person's score, so every table says "person" in those columns.
- **Pain** = (blast + the mean of the seven risks) / 2, where the seven risks are upkeep, maintainers, review, tests, `unsafe`, churn and owner. It is computed by the script in [Appendix B](#b7-this-file-pain), to 2 decimals. No in/out verdict is computed, because relief has no score yet.
- **Quotes** in § 1 and § 5 are copied byte for byte from the needs slice. A relative link inside a quote is kept as its source has it, so it resolves from `docs/decisions/`, not from this file.

### The primary reading of each input

Where a slice offered more than one reading, the tables use this one. § 3 gives the others.

| input | primary reading | why |
|---|---|---|
| blast area | crates counted with `cargo tree -e normal,build`, distinct `name version`, without the crate itself. Parts = the parts whose needs list the crate (a projection). | #960's audit counted `-e normal,build,dev`. A probe crate has no dev edges in its own tree, so the count is the same; the red-team checked this for `tokio`, `clap` and `pep508_rs` [A]. No part has a design, so "another part names the crate's types" is [C], and **each level is a floor**. |
| upstream review | the raw last 30 commits on the default branch. A commit is reviewed when its merged PR has an `APPROVED` review from a login other than the PR author. | the #960 audit's rule |
| tests | level 1 needs an **observed** CI test run: a check run, job or status with conclusion `success`, seen through the API. A workflow file alone is not enough. | #960 required an observed run. So `pico-args` and `supports-color` score 2: each has a workflow that runs `cargo test` on push and pull request, but no run is observed. `os_pipe` scores 1 under the same rule. Its HEAD has no run, but one run on a fork PR (2025-12-24, [run 20493330400](https://github.com/oconnor663/os_pipe.rs/actions/runs/20493330400)) was observed, and all 9 `cargo_tests` jobs passed [A]. |
| maintainers | owners by count. A team owner counts as 1. | the scale's wording. The judgement calls (`pyo3-async-runtimes`, `quick-junit`) are in § 3. |
| `unsafe` | a block "has SAFETY" when `/safety/i` is on the 3 lines above it, on its own line, or on the next line. Only blocks compiled for Linux x86_64 with default features count. | the #960 audit's rule for a block. ⚠️ #960 took "compiled" from the files that rustc's dep-info lists, which needs a build. The slices built nothing downloaded, so a hand-written `cfg` walker chose the files. It missed `tokio`'s `runtime/park.rs`, which the red-team found from rustc's own expansion. A missed block can only raise a count: no crate at 1 has an `unsafe` token anywhere in its `src/` (a `grep -rnw unsafe` over all 23, after a control that finds 16 lines in `os_pipe`) [A], each crate at 2 had its whole tarball read by the red-team [A], and a crate at 3 cannot rise. So the walker can be wrong in a count of a crate at 3, never in a level. A reading for a person is in § 3. |
| owner in purba | 3 for every candidate | `.github/dependabot.yml` at `2790c1e` has one `updates` entry, `package-ecosystem: github-actions`. A grep of `.github`, `.config` and `docs` for `package-ecosystem: *cargo\|renovate` matches nothing. No record names when a person raises a crate purba has not adopted [A]. |

**Licence: wheel or script.** Every candidate's need belongs to a part of the wheel, so purba's licence gate (`cargo-deny`, `.cargo/deny.toml`, `allow = ["MIT"]`) would check each one [A].

**One version difference between slices.** For `pyo3-async-runtimes`, the checks slice measured 0.30.0, the newest stable. The blast-b slice measured 0.29.0 first, because purba pins `pyo3` 0.29.2, and measured 0.30.0 as well. Blast and `unsafe` are the same for both versions. The tables use 0.30.0.

## 1. The needs

What each part and concern not designed yet needs, the line that states it, and what meets it today.
The table is the needs slice's own, copied byte for byte. Its legend:

Links are pinned to `2790c1e`, as `B/` = `https://github.com/kalonji-tools/purba/blob/2790c1e/`. Each quote is copied byte for byte. A quote that is a substring of its line says so with `…`.

| # | part | need | the line that states it | met today by | candidates | tier |
|---|---|---|---|---|---|---|
| S1 | strictness | read a test's code to judge its shape | "From a test's code alone, a tool decides only syntax with no false positive." ([Wayfinder: purba's strictness…](https://github.com/kalonji-tools/purba/issues/953), "Decisions so far", the line on #954) | **a purba crate**: `ruff_python_parser` + `ruff_python_ast` ([purba parses Python with ruff](https://github.com/kalonji-tools/purba/blob/2790c1e/docs/decisions/purba-parses-python-with-ruff.md#L32-L35)) | none: met | [A] |
| S2 | strictness | fork a process for each assay | "Each assay except the last runs in a process that purba forks after the act, so purba copies nothing and proves nothing." ([#953 body](https://github.com/kalonji-tools/purba/issues/953), on #964) | **a purba crate**: `libc` 0.2.189 has `fork`, `waitpid` and `pipe` | `nix`, `rustix` (the same as D1) | [A] |
| S3 | strictness | carry an assay's reading from a forked process back to the case's process | "The reading of a forked assay must reach the case's process, or be compared in the assay's process." ([#964 comment](https://github.com/kalonji-tools/purba/issues/964#issuecomment-6862174563), linked from #953) | **partly**: std `io::pipe` / `UnixStream::pair` for the channel, and `serde` derives. No purba crate writes a wire format | `serde_json`, `postcard`, `rmp-serde` (the same as D3) | [A] |
| P1 | parsing | decode a module by its encoding declaration, as CPython does | "1. How parsing decodes a module that declares an encoding other than UTF-8." ([#644](https://github.com/kalonji-tools/purba/issues/644), Decide). The risk: "Given a file that a supported CPython imports, when parsing reads it, then parsing refuses it" ([B/LANDSCAPE.md#L172](https://github.com/kalonji-tools/purba/blob/2790c1e/LANDSCAPE.md#L172), `…`) | **partly**. std decodes UTF-8 only (`String::from_utf8`, [B/crates/parsing/src/lib.rs#L228](https://github.com/kalonji-tools/purba/blob/2790c1e/crates/parsing/src/lib.rs#L228)). Parsing already finds the declaration in std ([L222](https://github.com/kalonji-tools/purba/blob/2790c1e/crates/parsing/src/lib.rs#L222)). `pyo3` could call `importlib.util.decode_source`, but #644 records that "parsing crosses the seam, and the record above changes" | `encoding_rs`, `encoding`, `textcode` | [A] |
| P2 | parsing | report more than one syntax error for a file | "Parsing decides in its design in depth how many syntax errors it reports for one file." ([B/docs/decisions/an-error-stops-only-the-work-that-depends-on-it.md#L42](https://github.com/kalonji-tools/purba/blob/2790c1e/docs/decisions/an-error-stops-only-the-work-that-depends-on-it.md#L42), `…`) | **a purba crate**: `ruff_python_parser`. "ruff lists three errors for a module where CPython names one" ([B/docs/decisions/parsing-hands-on-no-tree-that-ruff-recovered.md#L14](https://github.com/kalonji-tools/purba/blob/2790c1e/docs/decisions/parsing-hands-on-no-tree-that-ruff-recovered.md#L14), `…`) | none: met | [A] |
| C1 | configuration | read a configuration file that holds keys | "An [integrator](../../CONTEXT.md#integrator) caps the suppressed verdicts with a configuration key and a flag." ([B/docs/decisions/a-suppression-is-an-outcome-and-not-a-switch.md#L40](https://github.com/kalonji-tools/purba/blob/2790c1e/docs/decisions/a-suppression-is-an-outcome-and-not-a-switch.md#L40), `…`). The format joins the public surface: "the data purba writes, and the format of its configuration." ([B/CONTEXT.md#L181](https://github.com/kalonji-tools/purba/blob/2790c1e/CONTEXT.md#L181), `…`) | **none** parses a file. `serde` derives the keys, and std reads the bytes | ⚠️ only if the format is TOML, which no line states (see I1): `toml`, `toml_edit`, `pyproject-toml` | need [A]; format [C] |
| C2 | configuration, selection | pick a run's files by platform | "The designs in depth of configuration and selection, which [the landscape](../../LANDSCAPE.md#the-order-of-design) ranks, decide how a run picks its files by platform." ([B/docs/decisions/a-test-carries-only-purbas-own-decorators.md#L64](https://github.com/kalonji-tools/purba/blob/2790c1e/docs/decisions/a-test-carries-only-purbas-own-decorators.md#L64)) | **std**: `std::env::consts::OS`. **a purba crate**: `pyo3` can read Python's `sys.platform` | none stated. A marker syntax is inferred (I5) | [A] |
| Sel1 | selection | find the test files under a directory | "\| selection \| discovery and the query are one design, and it owns what a selected test is \|" ([B/docs/decisions/purba-divides-by-responsibility-and-not-by-language.md#L40](https://github.com/kalonji-tools/purba/blob/2790c1e/docs/decisions/purba-divides-by-responsibility-and-not-by-language.md#L40)). A lost option: "A directory and a query as the input. The query language belongs to the design in depth of selection." ([B/docs/decisions/a-tester-runs-purba-on-one-module-and-reads-one-line-for-each-test.md#L23](https://github.com/kalonji-tools/purba/blob/2790c1e/docs/decisions/a-tester-runs-purba-on-one-module-and-reads-one-line-for-each-test.md#L23), bold marks removed) | **partly**: std `fs::read_dir`, with the recursion written by hand | `walkdir`, `ignore` | [A] |
| Sel2 | selection | parse the query language that names tests | "answers which tests a query names" ([B/LANDSCAPE.md#L58](https://github.com/kalonji-tools/purba/blob/2790c1e/LANDSCAPE.md#L58), `…`). Also Sel1's two lines | **partly**: std, with a parser written by hand | `winnow`, `nom`, `chumsky`, `pest` | [A] |
| Sel3 | selection | give one module one identity, whatever the spelling of its path | "Two spellings of one path print two identities. `tests/test_a.py` and `./tests/test_a.py` name one module. The design in depth of selection decides how an identity starts." ([B/docs/decisions/a-tester-runs-purba-on-one-module-and-reads-one-line-for-each-test.md#L85](https://github.com/kalonji-tools/purba/blob/2790c1e/docs/decisions/a-tester-runs-purba-on-one-module-and-reads-one-line-for-each-test.md#L85), bold marks removed) | **std**: `std::path::absolute` (stable since 1.79.0) and `fs::canonicalize`, which resolves symlinks | `path-clean`, `normpath` | [A] |
| Seam1 | the Rust/Python seam | carry a live Python object between Rust and Python | "Carries a live Python object between Rust and Python, in execution, fixtures and assertion." ([B/LANDSCAPE.md#L153](https://github.com/kalonji-tools/purba/blob/2790c1e/LANDSCAPE.md#L153), `…`) | **a purba crate**: `pyo3` 0.29.2 | none: met | [A] |
| F1 | fixtures, execution | keep an async fixture on its event loop, and inside its test's task | "No async fixture leaves the event loop that built it, in execution and fixtures." and "No async fixture built for one test runs outside that test's task, in execution and fixtures." ([B/LANDSCAPE.md#L145-L146](https://github.com/kalonji-tools/purba/blob/2790c1e/LANDSCAPE.md#L145-L146), `…`) | **a purba crate**: `pyo3` calls `asyncio` as it calls any Python function. Assertion already calls the builtin `compile` this way ([B/crates/assertion/src/lib.rs#L287](https://github.com/kalonji-tools/purba/blob/2790c1e/crates/assertion/src/lib.rs#L287)) | `pyo3-async-runtimes` (the same as E2) | [A] |
| — | fixtures | building, tearing down, a stock's sample or checkpoint | No stated line asks a library for these. They run the tester's Python ([#965 comment](https://github.com/kalonji-tools/purba/issues/965#issuecomment-6865490125)) | — | **none: no need that a crate could meet** | [A] |
| A1 | assertion | show the values of an assert that did not hold | "so that a failure shows its values" ([B/LANDSCAPE.md#L63](https://github.com/kalonji-tools/purba/blob/2790c1e/LANDSCAPE.md#L63), `…`). "The tester reads them only after assertion shows them in its design in depth." ([B/docs/decisions/a-tester-runs-purba-on-one-module-and-reads-one-line-for-each-test.md#L79](https://github.com/kalonji-tools/purba/blob/2790c1e/docs/decisions/a-tester-runs-purba-on-one-module-and-reads-one-line-for-each-test.md#L79), `…`) | **a purba crate**: ruff rewrites at node ranges, and `pyo3` reads each value's `repr` | none: met. A diff of two values is inferred (I2) | [A] |
| E1 | execution | capture a test's output | "Output capture, warnings and stray exceptions. They belong inside execution, which wraps the test and its fixtures and owns the outcome." ([B/docs/decisions/a-concern-is-what-no-part-can-own.md#L55](https://github.com/kalonji-tools/purba/blob/2790c1e/docs/decisions/a-concern-is-what-no-part-can-own.md#L55), bold marks removed) | **partly**: `libc` (`pipe`, `dup2`) and std `io::pipe` (stable since 1.87.0) | `gag`, `os_pipe` | [A] |
| E2 | execution | await an `async def` test | "The design in depth of execution decides how purba awaits one." ([#693](https://github.com/kalonji-tools/purba/issues/693), Facts, `…`) | **a purba crate**: `pyo3`, as F1 says | `pyo3-async-runtimes` | [A] |
| D1 | dispatch | fork each test from a zygote that loaded its module, and start a test on macOS | "A zygote of purba's own loads the module once, and forks each test." ([B/docs/decisions/each-test-runs-in-a-process-of-its-own.md#L32](https://github.com/kalonji-tools/purba/blob/2790c1e/docs/decisions/each-test-runs-in-a-process-of-its-own.md#L32), `…`, bold marks removed). "How macOS starts each test." ([#694](https://github.com/kalonji-tools/purba/issues/694), Not yet decided) | **a purba crate**: `libc` 0.2.189 (`fork`, `waitpid`, `kill`, `posix_spawn`). **std**: `process::Command` for a spawn | `nix`, `rustix` | [A] |
| D2 | dispatch | end a test that hangs, on a timeout | "\| dispatch \| the worker protocol needs one owner, and the timeout of a test is designed inside it \|" ([B/docs/decisions/purba-divides-by-responsibility-and-not-by-language.md#L39](https://github.com/kalonji-tools/purba/blob/2790c1e/docs/decisions/purba-divides-by-responsibility-and-not-by-language.md#L39)). "A test that hangs has no owner until dispatch is designed." ([L60](https://github.com/kalonji-tools/purba/blob/2790c1e/docs/decisions/purba-divides-by-responsibility-and-not-by-language.md#L60), bold marks removed, `…`) | **partly**: `libc` (`kill`, `waitpid`, `poll`) and std `time::Instant` | `nix`, `rustix`, `polling`, `tokio` | [A] |
| D3 | dispatch | hand a worker's outcome to the parent, as data, over the worker protocol | D2's first line ("the worker protocol needs one owner"). "Hands each selected test to a worker and hands on its outcome, or the error it gave in place of one, whatever the worker is." ([B/LANDSCAPE.md#L60](https://github.com/kalonji-tools/purba/blob/2790c1e/LANDSCAPE.md#L60), `…`). "An outcome carries no live Python object, in execution, dispatch and reporting." ([L141](https://github.com/kalonji-tools/purba/blob/2790c1e/LANDSCAPE.md#L141), `…`) | **partly**: `serde` derives `Serialize` and `Deserialize` on `Outcome` and `Kind` ([B/crates/execution/src/lib.rs#L55](https://github.com/kalonji-tools/purba/blob/2790c1e/crates/execution/src/lib.rs#L55), [L70](https://github.com/kalonji-tools/purba/blob/2790c1e/crates/execution/src/lib.rs#L70)). std gives the channel. No purba crate writes a format | `serde_json`, `postcard`, `rmp-serde` | [A] |
| D4 | dispatch | run tests in parallel | "puts parallel runs in dispatch, and makes async a property of execution." ([B/docs/decisions/purbas-public-surface-starts-empty-and-has-no-plugin-api.md#L25](https://github.com/kalonji-tools/purba/blob/2790c1e/docs/decisions/purbas-public-surface-starts-empty-and-has-no-plugin-api.md#L25), `…`) | **std** + **a purba crate**: `thread::available_parallelism` (stable since 1.59.0) gives the CPU count, and `libc::waitpid` waits on many children | none beyond D1 and D2 | [A] |
| D5 | dispatch (no part owns it yet) | plan a run as a graph before running it | "1. What the plan holds: its nodes, and the edges between them." ([#691](https://github.com/kalonji-tools/purba/issues/691), Decide). Its third item is "which part owns the plan". | **std**: the collections, with a graph written by hand | `petgraph` | [A] |
| R1 | reporting | write outcomes as data for an agent or an integrator | "Renders each outcome as text for a person, and as data for an agent or an integrator." ([B/LANDSCAPE.md#L64](https://github.com/kalonji-tools/purba/blob/2790c1e/LANDSCAPE.md#L64), `…`). A lost option: "A data format for an [integrator](../../CONTEXT.md#integrator) now, such as JSON Lines. Reporting decides a format in its design in depth" ([B/docs/decisions/a-tester-runs-purba-on-one-module-and-reads-one-line-for-each-test.md#L24](https://github.com/kalonji-tools/purba/blob/2790c1e/docs/decisions/a-tester-runs-purba-on-one-module-and-reads-one-line-for-each-test.md#L24), bold marks removed, `…`) | **partly**: `serde` derives the shape. No purba crate writes a format | `serde_json` | [A] |
| R2 | reporting | write JUnit XML | "Built-in fixtures belong inside fixtures, and JUnit output belongs inside reporting." ([B/docs/decisions/purba-divides-by-responsibility-and-not-by-language.md#L55](https://github.com/kalonji-tools/purba/blob/2790c1e/docs/decisions/purba-divides-by-responsibility-and-not-by-language.md#L55), `…`). ⚠️ The line says *where* JUnit belongs, under "What stays off the diagram". No record admits JUnit output | **none** | `quick-junit`, `quick-xml`, `junit-report` | [A] |
| R3 | reporting | render a location as a line and a column | "Reporting renders the location in its design in depth." ([B/docs/decisions/parsing-hands-on-no-tree-that-ruff-recovered.md#L40](https://github.com/kalonji-tools/purba/blob/2790c1e/docs/decisions/parsing-hands-on-no-tree-that-ruff-recovered.md#L40), `…`) | **a purba crate**: `ruff_source_file::LineIndex`, which parsing already builds ([B/crates/parsing/src/lib.rs#L389](https://github.com/kalonji-tools/purba/blob/2790c1e/crates/parsing/src/lib.rs#L389)) | none: met | [A] |
| Cmd1 | command | read the command's flags | "the command's flags and exit code" ([B/CONTEXT.md#L181](https://github.com/kalonji-tools/purba/blob/2790c1e/CONTEXT.md#L181), `…`). C1's line names "a configuration key and a flag". Today: "A run takes exactly one path to one `.py` file, and no flag." ([B/docs/decisions/a-tester-runs-purba-on-one-module-and-reads-one-line-for-each-test.md#L41](https://github.com/kalonji-tools/purba/blob/2790c1e/docs/decisions/a-tester-runs-purba-on-one-module-and-reads-one-line-for-each-test.md#L41), bold marks removed) | **partly**: `pyo3` copies `sys.argv` into a `Vec<String>` ([B/crates/purba/src/lib.rs#L37](https://github.com/kalonji-tools/purba/blob/2790c1e/crates/purba/src/lib.rs#L37)), and command parses it by hand | `lexopt`, `pico-args`, `bpaf`, `clap`. ⚠️ A record already weighed clap: "Command reads its argument without clap. clap exits 2 on a usage error, and 2 means `unknown` here." ([B/docs/decisions/a-run-exits-0-only-when-it-checked-something-and-every-test-passed.md#L73](https://github.com/kalonji-tools/purba/blob/2790c1e/docs/decisions/a-run-exits-0-only-when-it-checked-something-and-every-test-passed.md#L73), bold marks removed) | [A] |

**Count.** 24 stated needs (S1–S3, P1–P2, C1–C2, Sel1–Sel3, Seam1, F1, A1, E1–E2, D1–D5, R1–R3, Cmd1). F1 and E2 share a candidate. S2 shares its candidates with D1, and S3 shares its candidates with D3.

**Parts and concerns with no new candidate.** Each need of **the Rust/Python seam** (Seam1) and of **assertion** (A1) is met by `pyo3` and ruff. Every need on purba's side of **fixtures**, except the async one (F1), runs the tester's Python and needs no crate. Every other part has at least one need that std or purba's crates meet only partly, or do not meet.

**Met by nothing that purba has today:** P1 (decode by a declared encoding, beyond UTF-8) and R2 (JUnit XML). C1 is met by nothing only once its format is chosen.

### Inferred needs

These are listed apart, and each one is marked inferred.

⚠️ **No line in the landscape, a record or an open issue states these needs.** They are listed so that the design of each part can confirm or drop them. They are not stated needs.

| # | part | inferred need | why it is inferred | met today by | candidates | tier |
|---|---|---|---|---|---|---|
| I1 | configuration | read purba's configuration from TOML, or from a `[tool.purba]` table in `pyproject.toml` | No line names the format. The only placement rule, [configuration takes the first location its tool reads](https://github.com/kalonji-tools/purba/blob/2790c1e/docs/decisions/configuration-takes-the-first-location-its-tool-reads.md#L48), covers the configuration of the tools purba itself runs, not purba's own | **none** | `toml`, `toml_edit`, `pyproject-toml` | [A] the crates; [C] the format |
| I2 | assertion, reporting | show a diff between two values that differ | A1 asks for the values, not for a diff | **none** | `similar`, `imara-diff`, `difflib` | [A] |
| I3 | reporting | colour the text, and find out whether stdout is a terminal | R1 says "text for a person", and no line names colour. The one line about escape codes is the cost of a lost option: "An escape code moves the cursor of a terminal, so a reason can erase the line above it." ([B/…/a-tester-runs-purba-on-one-module-and-reads-one-line-for-each-test.md#L28](https://github.com/kalonji-tools/purba/blob/2790c1e/docs/decisions/a-tester-runs-purba-on-one-module-and-reads-one-line-for-each-test.md#L28), `…`) | **std**: `io::IsTerminal` finds a terminal. Nothing colours | `anstream`, `owo-colors`, `supports-color` | [A] |
| I4 | selection | match paths against glob patterns in a query | Sel2 names a query language, and no line says that it holds globs | **none** | `globset` | [A] |
| I5 | configuration, selection | write C2's platform rule as PEP 508 markers, such as `sys_platform == "linux"` | C2 states the need, and no line states the syntax | **std** / `pyo3` give the platform | `pep508_rs` | [A] |
| I6 | strictness | block a C library's own socket, below Python | #953 lists it as a gap, not as a need: "A C library's own socket or thread. A block at run time catches only the calls that go through Python." ([#953](https://github.com/kalonji-tools/purba/issues/953), Not yet specified, bold marks removed) | **none** | `seccompiler` (Linux only; nothing found for macOS) | [A] the crate; [C] whether this is a need |

## 2. The candidates, by need

One table for each need that has a candidate: 17 stated needs, then 6 inferred ones.
A candidate under several needs repeats its row. Inside a table, rows are sorted by crate name, and the order carries no meaning.

- **Checks.** P = pass, **F** = fail, with the reason. "licence own / gate" gives the crate's own licence field, then the result of purba's real gate (`cargo-deny check licenses` with `.cargo/deny.toml`, on a probe that locks the crate alone). Source passes for all 41: each version is published on crates.io and not yanked. Trust is a person's score; its evidence is in § 4.
- **Inputs.** The levels use the primary readings in [How to read](#the-primary-reading-of-each-input). The evidence for every level is in [Evidence per crate](#evidence-per-crate).
- **Pain** = (blast + mean of the seven risks) / 2. Relief is a person's score.
- Every check cell and every level is [A], except two parts of blast area: the parts are [B, a projection from the needs], and "another part names the crate's types" is [C] for every crate.

**Stated needs with no candidate:** S1, P2, Seam1, A1 and R3 are met by a purba crate. C2 and D4 are met by std together with a purba crate. The table in § 1 gives each one.

### Stated needs

#### S2 · strictness · fork a process for each assay

| crate @ version | advisory | licence own / gate | source | trust | blast | upkeep | maint. | review | tests | unsafe | churn | owner | **pain** | relief |
|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|
| `nix` 0.31.3 | P (RUSTSEC-2021-0119 patched ≥ 0.23.0) | P MIT / P | P | person | 2 | 1 | 1 | 2 | 1 | 3 | 2 | 3 | **1.93** | person |
| `rustix` 1.1.6 | P (RUSTSEC-2026-0486 patched ≥ 1.1.6) | P `Apache-2.0 WITH LLVM-exception OR Apache-2.0 OR MIT` / P | P | person | 2 | 1 | 3 | 3 | 1 | 3 | 1 | 3 | **2.07** | person |

#### S3 · strictness · carry an assay's reading from a forked process back to the case's process

| crate @ version | advisory | licence own / gate | source | trust | blast | upkeep | maint. | review | tests | unsafe | churn | owner | **pain** | relief |
|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|
| `postcard` 1.1.3 | P (none) | P `MIT OR Apache-2.0` / P | P | person | 2 | 2 | 3 | 2 | 1 | 2 | 1 | 3 | **2.00** | person |
| `rmp-serde` 1.3.1 | P (RUSTSEC-2022-0092 patched ≥ 1.1.1) | P MIT / P | P (1.3.1 not yanked; 35 of 43 versions yanked) | person | 2 | 1 | 2 | 3 | 1 | 1 | 1 | 3 | **1.86** | person |
| `serde_json` 1.0.154 | P (none) | P `MIT OR Apache-2.0` / P | P | person | 2 | 1 | 1 | 2 | 1 | 3 | 1 | 3 | **1.86** | person |

#### P1 · parsing · decode a module by its encoding declaration, as CPython does

| crate @ version | advisory | licence own / gate | source | trust | blast | upkeep | maint. | review | tests | unsafe | churn | owner | **pain** | relief |
|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|
| `encoding` 0.2.33 | **F** (RUSTSEC-2021-0153 *unmaintained*, `patched = []`) | P MIT / **F** (6 `encoding-index-*`, CC0-1.0) | P | person | 1 | 3 | 3 | 3 | 2 | 3 | 2 | 3 | **1.86** | person |
| `encoding_rs` 0.8.44 | P (none) | **F** `(Apache-2.0 OR MIT) AND BSD-3-Clause` / **F** (itself) | P | person | 1 | 1 | 3 | 3 | 1 | 3 | 2 | 3 | **1.64** | person |
| `textcode` 0.5.1 | P (none) | P MIT / P | P | person | 1 | 1 | 3 | 3 | 2 | 3 | 3 | 3 | **1.79** | person |

#### C1 · configuration · read a configuration file that holds keys (⚠️ only if the format is TOML, which no line states)

| crate @ version | advisory | licence own / gate | source | trust | blast | upkeep | maint. | review | tests | unsafe | churn | owner | **pain** | relief |
|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|
| `pyproject-toml` 0.13.7 | P (none) | P MIT / **F** (21 crates: `pep508_rs`, `pep440_rs`, `version-ranges`, 18 ICU/zerovec) | P | person | 3 | 2 | 1 | 3 | 1 | 1 | 2 | 3 | **2.43** | person |
| `toml` 1.2.1+spec-1.1.0 | P (none) | P `MIT OR Apache-2.0` / P | P | person | 1 | 1 | 1 | 3 | 1 | 1 | 2 | 3 | **1.36** | person |
| `toml_edit` 0.25.19+spec-1.1.0 | P (none) | P `MIT OR Apache-2.0` / P | P | person | 1 | 1 | 1 | 3 | 1 | 1 | 2 | 3 | **1.36** | person |

#### Sel1 · selection · find the test files under a directory

| crate @ version | advisory | licence own / gate | source | trust | blast | upkeep | maint. | review | tests | unsafe | churn | owner | **pain** | relief |
|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|
| `ignore` 0.4.34 | P (none) | P `Unlicense OR MIT` / P | P | person | 2 | 1 | 3 | 3 | 1 | 1 | 2 | 3 | **2.00** | person |
| `walkdir` 2.5.0 | P (none) | P `Unlicense/MIT` / P | P | person | 1 | 3 | 3 | 3 | 1 | 1 | 1 | 3 | **1.57** | person |

#### Sel2 · selection · parse the query language that names tests

| crate @ version | advisory | licence own / gate | source | trust | blast | upkeep | maint. | review | tests | unsafe | churn | owner | **pain** | relief |
|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|
| `chumsky` 0.14.1 | P (none) | P MIT / **F** (`foldhash` 0.2.0, Zlib) | P | person | 2 | 1 | 1 | 3 | 1 | 2 | 3 | 3 | **2.00** | person |
| `nom` 8.0.0 | P (none) | P MIT / P | P | person | 1 | 2 | 3 | 2 | 2 | 3 | 1 | 3 | **1.64** | person |
| `pest` 2.9.6 | P (none) | P `MIT OR Apache-2.0` / P | P | person | 2 | 1 | 1 | 2 | 1 | 1 | 1 | 3 | **1.71** | person |
| `winnow` 1.0.5 | P (none) | P MIT / P | P | person | 1 | 1 | 3 | 3 | 1 | 3 | 2 | 3 | **1.64** | person |

#### Sel3 · selection · give one module one identity, whatever the spelling of its path

| crate @ version | advisory | licence own / gate | source | trust | blast | upkeep | maint. | review | tests | unsafe | churn | owner | **pain** | relief |
|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|
| `normpath` 1.5.2 | P (none) | P `MIT OR Apache-2.0` / P | P | person | 1 | 1 | 3 | 3 | 1 | 2 | 1 | 3 | **1.50** | person |
| `path-clean` 1.0.1 | P (none) | P `MIT OR Apache-2.0` / P | P | person | 1 | 3 | 3 | 3 | 2 | 1 | 1 | 3 | **1.64** | person |

#### F1 · fixtures, execution · keep an async fixture on its event loop, and inside its test's task

| crate @ version | advisory | licence own / gate | source | trust | blast | upkeep | maint. | review | tests | unsafe | churn | owner | **pain** | relief |
|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|
| `pyo3-async-runtimes` 0.30.0 | P (none) | **F** Apache-2.0 / **F** (itself) | P | person | 2 | 1 | 3 | 1 | 1 | 1 | 3 | 3 | **1.93** | person |

#### E1 · execution · capture a test's output

| crate @ version | advisory | licence own / gate | source | trust | blast | upkeep | maint. | review | tests | unsafe | churn | owner | **pain** | relief |
|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|
| `gag` 1.0.0 | P (none) | P MIT / P | P | person | 2 | 3 | 3 | 3 | 2 | 1 | 1 | 3 | **2.14** | person |
| `os_pipe` 1.2.3 | P (none) | P MIT / P | P | person | 1 | 2 | 3 | 3 | 1 | 3 | 1 | 3 | **1.64** | person |

#### E2 · execution · await an `async def` test

| crate @ version | advisory | licence own / gate | source | trust | blast | upkeep | maint. | review | tests | unsafe | churn | owner | **pain** | relief |
|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|
| `pyo3-async-runtimes` 0.30.0 | P (none) | **F** Apache-2.0 / **F** (itself) | P | person | 2 | 1 | 3 | 1 | 1 | 1 | 3 | 3 | **1.93** | person |

#### D1 · dispatch · fork each test from a zygote that loaded its module, and start a test on macOS

| crate @ version | advisory | licence own / gate | source | trust | blast | upkeep | maint. | review | tests | unsafe | churn | owner | **pain** | relief |
|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|
| `nix` 0.31.3 | P (RUSTSEC-2021-0119 patched ≥ 0.23.0) | P MIT / P | P | person | 2 | 1 | 1 | 2 | 1 | 3 | 2 | 3 | **1.93** | person |
| `rustix` 1.1.6 | P (RUSTSEC-2026-0486 patched ≥ 1.1.6) | P `Apache-2.0 WITH LLVM-exception OR Apache-2.0 OR MIT` / P | P | person | 2 | 1 | 3 | 3 | 1 | 3 | 1 | 3 | **2.07** | person |

#### D2 · dispatch · end a test that hangs, on a timeout

| crate @ version | advisory | licence own / gate | source | trust | blast | upkeep | maint. | review | tests | unsafe | churn | owner | **pain** | relief |
|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|
| `nix` 0.31.3 | P (RUSTSEC-2021-0119 patched ≥ 0.23.0) | P MIT / P | P | person | 2 | 1 | 1 | 2 | 1 | 3 | 2 | 3 | **1.93** | person |
| `polling` 3.11.0 | P (none) | P `Apache-2.0 OR MIT` / P | P | person | 1 | 2 | 1 | 2 | 1 | 3 | 1 | 3 | **1.43** | person |
| `rustix` 1.1.6 | P (RUSTSEC-2026-0486 patched ≥ 1.1.6) | P `Apache-2.0 WITH LLVM-exception OR Apache-2.0 OR MIT` / P | P | person | 2 | 1 | 3 | 3 | 1 | 3 | 1 | 3 | **2.07** | person |
| `tokio` 1.54.1 | P (7 advisories; none names 1.54.1, RUSTSEC-2026-0384 patched ≥ 1.54.1) | P MIT / P | P | person | 1 | 1 | 1 | 1 | 1 | 3 | 1 | 3 | **1.29** | person |

#### D3 · dispatch · hand a worker's outcome to the parent, as data, over the worker protocol

| crate @ version | advisory | licence own / gate | source | trust | blast | upkeep | maint. | review | tests | unsafe | churn | owner | **pain** | relief |
|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|
| `postcard` 1.1.3 | P (none) | P `MIT OR Apache-2.0` / P | P | person | 2 | 2 | 3 | 2 | 1 | 2 | 1 | 3 | **2.00** | person |
| `rmp-serde` 1.3.1 | P (RUSTSEC-2022-0092 patched ≥ 1.1.1) | P MIT / P | P (1.3.1 not yanked; 35 of 43 versions yanked) | person | 2 | 1 | 2 | 3 | 1 | 1 | 1 | 3 | **1.86** | person |
| `serde_json` 1.0.154 | P (none) | P `MIT OR Apache-2.0` / P | P | person | 2 | 1 | 1 | 2 | 1 | 3 | 1 | 3 | **1.86** | person |

#### D5 · dispatch (no part owns it yet) · plan a run as a graph before running it

| crate @ version | advisory | licence own / gate | source | trust | blast | upkeep | maint. | review | tests | unsafe | churn | owner | **pain** | relief |
|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|
| `petgraph` 0.8.3 | P (none) | P `MIT OR Apache-2.0` / **F** (`foldhash` 0.1.5, Zlib) | P | person | 1 | 2 | 1 | 3 | 1 | 3 | 2 | 3 | **1.57** | person |

#### R1 · reporting · write outcomes as data for an agent or an integrator

| crate @ version | advisory | licence own / gate | source | trust | blast | upkeep | maint. | review | tests | unsafe | churn | owner | **pain** | relief |
|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|
| `serde_json` 1.0.154 | P (none) | P `MIT OR Apache-2.0` / P | P | person | 2 | 1 | 1 | 2 | 1 | 3 | 1 | 3 | **1.86** | person |

#### R2 · reporting · write JUnit XML

| crate @ version | advisory | licence own / gate | source | trust | blast | upkeep | maint. | review | tests | unsafe | churn | owner | **pain** | relief |
|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|
| `junit-report` 0.9.1 | P (none) | P MIT / P | P | person | 2 | 1 | 3 | 3 | 1 | 1 | 2 | 3 | **2.00** | person |
| `quick-junit` 0.8.0 | P (none) | P `Apache-2.0 OR MIT` / P | P | person | 2 | 1 | 2 | 3 | 1 | 1 | 3 | 3 | **2.00** | person |
| `quick-xml` 0.42.0 | P (RUSTSEC-2026-0194 and -0195 patched ≥ 0.41.0) | P MIT / P | P | person | 1 | 1 | 1 | 2 | 1 | 1 | 3 | 3 | **1.36** | person |

#### Cmd1 · command · read the command's flags

| crate @ version | advisory | licence own / gate | source | trust | blast | upkeep | maint. | review | tests | unsafe | churn | owner | **pain** | relief |
|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|
| `bpaf` 0.9.29 | P (none) | P `MIT OR Apache-2.0` / P | P | person | 1 | 1 | 3 | 3 | 1 | 1 | 2 | 3 | **1.50** | person |
| `clap` 4.6.7 | P (none) | P `MIT OR Apache-2.0` / P | P | person | 1 | 1 | 1 | 3 | 1 | 1 | 1 | 3 | **1.29** | person |
| `lexopt` 0.3.2 | P (none) | P MIT / P | P | person | 1 | 1 | 3 | 3 | 1 | 1 | 2 | 3 | **1.50** | person |
| `pico-args` 0.5.0 | P (none) | P MIT / P | P | person | 1 | 3 | 3 | 3 | 2 | 1 | 2 | 3 | **1.71** | person |

### Inferred needs

Each of these needs is inferred: no line states it (§ 1).

#### I1 · configuration · *inferred*: read purba's configuration from TOML, or from a `[tool.purba]` table in `pyproject.toml`

| crate @ version | advisory | licence own / gate | source | trust | blast | upkeep | maint. | review | tests | unsafe | churn | owner | **pain** | relief |
|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|
| `pyproject-toml` 0.13.7 | P (none) | P MIT / **F** (21 crates: `pep508_rs`, `pep440_rs`, `version-ranges`, 18 ICU/zerovec) | P | person | 3 | 2 | 1 | 3 | 1 | 1 | 2 | 3 | **2.43** | person |
| `toml` 1.2.1+spec-1.1.0 | P (none) | P `MIT OR Apache-2.0` / P | P | person | 1 | 1 | 1 | 3 | 1 | 1 | 2 | 3 | **1.36** | person |
| `toml_edit` 0.25.19+spec-1.1.0 | P (none) | P `MIT OR Apache-2.0` / P | P | person | 1 | 1 | 1 | 3 | 1 | 1 | 2 | 3 | **1.36** | person |

#### I2 · assertion, reporting · *inferred*: show a diff between two values that differ

| crate @ version | advisory | licence own / gate | source | trust | blast | upkeep | maint. | review | tests | unsafe | churn | owner | **pain** | relief |
|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|
| `difflib` 0.4.0 | P (none) | P MIT / P | P | person | 2 | 3 | 3 | 3 | 2 | 1 | 2 | 3 | **2.21** | person |
| `imara-diff` 0.2.0 | P (none) | **F** Apache-2.0 / **F** (itself + `foldhash` 0.1.5) | P | person | 2 | 2 | 3 | 2 | 1 | 3 | 2 | 3 | **2.14** | person |
| `similar` 3.2.1 | P (none) | **F** Apache-2.0 / **F** (itself) | P | person | 2 | 1 | 3 | 3 | 1 | 1 | 2 | 3 | **2.00** | person |

#### I3 · reporting · *inferred*: colour the text, and find out whether stdout is a terminal

| crate @ version | advisory | licence own / gate | source | trust | blast | upkeep | maint. | review | tests | unsafe | churn | owner | **pain** | relief |
|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|
| `anstream` 1.0.0 | P (RUSTSEC-2024-0404 patched ≥ 0.6.8) | P `MIT OR Apache-2.0` / P | P | person | 1 | 1 | 1 | 3 | 1 | 3 | 2 | 3 | **1.50** | person |
| `owo-colors` 4.4.0 | P (none) | P MIT / P | P | person | 1 | 1 | 2 | 3 | 1 | 1 | 1 | 3 | **1.36** | person |
| `supports-color` 3.0.2 | P (none) | **F** Apache-2.0 / **F** (itself + `is_ci` 1.2.0) | P | person | 1 | 2 | 3 | 3 | 2 | 1 | 1 | 3 | **1.57** | person |

#### I4 · selection · *inferred*: match paths against glob patterns in a query

| crate @ version | advisory | licence own / gate | source | trust | blast | upkeep | maint. | review | tests | unsafe | churn | owner | **pain** | relief |
|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|
| `globset` 0.4.21 | P (none) | P `Unlicense OR MIT` / P | P | person | 1 | 1 | 3 | 3 | 1 | 1 | 2 | 3 | **1.50** | person |

#### I5 · configuration, selection · *inferred*: write C2's platform rule as PEP 508 markers

| crate @ version | advisory | licence own / gate | source | trust | blast | upkeep | maint. | review | tests | unsafe | churn | owner | **pain** | relief |
|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|
| `pep508_rs` 0.9.2 | P (none) | **F** `Apache-2.0 OR BSD-2-Clause` / **F** (21 crates, itself among them) | P | person | 3 | 2 | 3 | 3 | 1 | 1 | 2 | 3 | **2.57** | person |

#### I6 · strictness · *inferred*: block a C library's own socket, below Python

| crate @ version | advisory | licence own / gate | source | trust | blast | upkeep | maint. | review | tests | unsafe | churn | owner | **pain** | relief |
|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|
| `seccompiler` 0.5.0 | P (none) | **F** `Apache-2.0 OR BSD-3-Clause` / **F** (itself) | P | person | 1 | 2 | 1 | 1 | 1 | 2 | 2 | 3 | **1.36** | person |


### Evidence per crate

Each cell is `level: evidence`, condensed from the slices. Every cell is [A] unless it says otherwise. Every blast level is a floor: its parts are [B, projection], and crossing is [C].

#### Blast area, upkeep, maintainers, API churn

Blast: the parts, then the crates it brings with normal and build edges (normal edges alone in brackets). Upkeep: the newest stable release and its date. Churn windows: 12 months from 2025-11-14, 3 months from 2026-08-14.

| crate | blast area | upkeep | maintainers | API churn |
|---|---|---|---|---|
| `anstream` | 1: 1 part (I3 reporting); brings 6 (6) | 1: 1.0.0, 2026-02-11 | 1: team `rust-cli:maintainers` + epage | 2: ≥ 1.0; major 1.0.0 on 2026-02-11, inside 12 months |
| `bpaf` | 1: 1 part (Cmd1 command); brings 0 | 1: 0.9.29, 2026-11-12 | 3: 1 user (pacak) | 2: 0.x; 0.9.0 on 2023-07-03 |
| `chumsky` | 2: 1 part (Sel2 selection); brings 16 (10). The build edges add ar_archive_writer, cc, find-msvc-tools, memchr, object and shlex, through stacker and psm | 1: 0.14.1, 2026-11-13 | 1: team `rust-bus:maintainers` + zesterer | 3: 0.x; 0.14.0 on 2026-11-13, 1 day before |
| `clap` | 1: 1 part (Cmd1); brings 10 (10), at the ≤ 10 edge | 1: 4.6.7, 2026-09-14 | 1: teams `rust-cli:maintainers`, `clap-rs:admins` + kbknapp | 1: 4.0.0 on 2022-09-28 |
| `difflib` | 2: 2 parts (I2 assertion, reporting); brings 0 | 3: 0.4.0, 2018-07-22 | 3: 1 user (DimaKudosh) | 2: 0.x; 0.4.0 on 2018-07-22 |
| `encoding` | 1: 1 part (P1 parsing); brings 6 (6), the `encoding-index-*` crates | 3: 0.2.33, 2016-08-28 | 3: 1 user (lifthrasiir) | 2: 0.x; 0.2.0 on 2014-11-14 |
| `encoding_rs` | 1: 1 part (P1); brings 6 (5) | 1: 0.8.44, 2026-10-30 | 3: 1 user (hsivonen) | 2: 0.x; 0.8.0 on 2018-06-04 |
| `gag` | 2: 1 part (E1 execution); brings 16 (16) | 3: 1.0.0, 2021-05-30 | 3: 1 user (Stebalien) | 1: 1.0.0 on 2021-05-30 |
| `globset` | 1: 1 part (I4 selection); brings 6 (6) | 1: 0.4.21, 2026-10-27 | 3: 1 user (BurntSushi) | 2: 0.x; 0.4.0 on 2018-04-21 |
| `ignore` | 2: 1 part (Sel1 selection); brings 12 (12) | 1: 0.4.34, 2026-10-27 | 3: 1 user (BurntSushi) | 2: 0.x; 0.4.0 on 2018-02-11 |
| `imara-diff` | 2: 2 parts (I2); brings 3 (3) | 2: 0.2.0, 2025-06-14 | 3: 1 user (pascalkuthe) | 2: 0.x; 0.2.0 on 2025-06-14 |
| `junit-report` | 2: 1 part (R2 reporting); brings 13 (13) | 1: 0.9.1, 2026-07-27 | 3: 1 user (bachp) | 2: 0.x; 0.9.0 on 2026-04-15 |
| `lexopt` | 1: 1 part (Cmd1); brings 0 | 1: 0.3.2, 2026-02-28 | 3: 1 user (blyxxyz) | 2: 0.x; 0.3.0 on 2023-01-16 |
| `nix` | 2: 2 parts (S2 strictness; D1, D2 dispatch); brings 4 (3: bitflags, cfg-if, libc) | 1: 0.31.3, 2026-05-11 | 1: team `nix-maintainers` + 1 user | 2: 0.x; last breaking 0.31.0 on 2026-01-22 |
| `nom` | 1: 1 part (Sel2); brings 1 (1: memchr) | 2: 8.0.0, 2025-01-26 | 3: 1 user (Geal) | 1: 8.0.0 on 2025-01-26, before the 12-month window |
| `normpath` | 1: 1 part (Sel3 selection); brings 0 | 1: 1.5.2, 2026-09-19 | 3: 1 user (dylni) | 1: 1.0.1 on 2022-12-17 |
| `os_pipe` | 1: 1 part (E1); brings 1 (1: libc) | 2: 1.2.3, 2025-10-11 | 3: 1 user (oconnor663) | 1: 1.0.0 on 2021-11-16 |
| `owo-colors` | 1: 1 part (I3); brings 0 | 1: 4.4.0, 2026-08-27 | 2: sunshowers, jam1garner | 1: 4.0.0 on 2023-12-20 |
| `path-clean` | 1: 1 part (Sel3); brings 0 | 3: 1.0.1, 2023-02-24 | 3: 1 user (danreeves) | 1: 1.0.0 on 2023-02-24 |
| `pep508_rs` | 3: 2 parts (I5 configuration, selection); brings 57 (57), most of them the ICU and `url` stack | 2: 0.9.2, 2025-01-02 | 3: 1 user (konstin) | 2: 0.x; 0.9.0 on 2024-10-30 |
| `pest` | 2: 1 part (Sel2); brings 12 (7). The build edges add ar_archive_writer, cc, find-msvc-tools, object and shlex | 1: 2.9.6, 2026-11-10 | 1: team `pest-parser:admins` + dragostis | 1: 2.0.0 on 2018-09-30 |
| `petgraph` | 1: 1 part (D5 dispatch, whose owner is [C]: "no part owns it yet"); brings 6 (6) | 2: 0.8.3, 2025-09-30 | 1: team `petgraph:release-team` + 2 users | 2: 0.x; 0.8.0 on 2025-04-05 |
| `pico-args` | 1: 1 part (Cmd1); brings 0 | 3: 0.5.0, 2022-06-04 | 3: 1 user (RazrFalcon) | 2: 0.x; 0.5.0 on 2022-06-04 |
| `polling` | 1: 1 part (D2 dispatch); brings 4 (4) | 2: 3.11.0, 2025-09-14 | 1: team `smol-rs:admins` + 1 user | 1: 3.0.0 on 2023-09-05 |
| `postcard` | 2: 2 parts (S3 strictness, D3 dispatch); brings 19 (17) | 2: 1.1.3, 2025-07-24 | 3: 1 user (jamesmunns) | 1: 1.0.1 on 2022-07-15 (1.0.0 is yanked) |
| `pyo3-async-runtimes` | 2: 2 parts (F1 fixtures, E2 execution); 0.30.0 brings 20 (18), 0.29.0 brings 21 (19), pyo3 and its subtree included | 1: 0.30.0, 2026-10-15 | 3 by count: 1 user (davidhewitt); `trustpub_only` | 3: 0.x; 0.30.0 on 2026-10-15; 0.28.0, 0.29.0 and 0.30.0 all inside 12 months |
| `pyproject-toml` | 3: 1 part (C1 if TOML, I1 configuration); brings 66 (66), among them `toml` 0.9.12, `pep508_rs`, `url`, `idna` and the `icu_*` crates | 2: 0.13.7, 2025-10-09 | 1: team `pyo3:maintainers` + messense | 2: 0.x; 0.13.0 on 2024-10-24 |
| `quick-junit` | 2: 1 part (R2); brings 18 (17) | 1: 0.8.0, 2026-09-10 | 2 by count: sunshowers, nextest-bot; `trustpub_only` | 3: 0.x; 0.8.0 on 2026-09-10 |
| `quick-xml` | 1: 1 part (R2); brings 1 (1: memchr) | 1: 0.42.0, 2026-08-22 | 1: 3 users (tafia, Mingun, dralley) | 3: 0.x; 0.42.0 on 2026-08-22; 0.39.0 to 0.42.0 all inside 12 months |
| `rmp-serde` | 2: 2 parts (S3, D3); brings 5 (4) | 1: 1.3.1, 2025-12-23 | 2: kornelski, 3Hren | 1: 1.1.1 on 2022-09-27 (1.0.0 and 1.1.0 are yanked) |
| `rustix` | 2: 2 parts (S2; D1, D2); brings 2 (2: bitflags, linux-raw-sys) | 1: 1.1.6, 2026-10-27 | 3: 1 user (sunfishcode); the repository belongs to the `bytecodealliance` organisation | 1: ≥ 1.0; 1.0.0 on 2025-03-06 |
| `seccompiler` | 1: 1 part (I6 strictness); brings 1 (1: libc) | 2: 0.5.0, 2025-03-07 | 1: team `rust-vmm:gatekeepers` + 3 users | 2: 0.x; 0.5.0 on 2025-03-07 |
| `serde_json` | 2: 3 parts (S3, D3, R1 reporting); whether another part names its types is [C], and the scale does not say where three parts fall; brings 4 (4: itoa, memchr, serde_core, zmij) | 1: 1.0.154, 2026-10-11 | 1: team `serde-rs:publish` + dtolnay | 1: 1.0.0 on 2017-04-20 |
| `similar` | 2: 2 parts (I2); brings 0 | 1: 3.2.1, 2026-10-24 | 3: 1 user (mitsuhiko) | 2: ≥ 1.0; major 3.0.0 on 2026-04-01, inside 12 months |
| `supports-color` | 1: 1 part (I3); brings 1 (1: is_ci) | 2: 3.0.2, 2024-11-26 | 3: 1 user (zkat) | 1: 3.0.0 on 2024-02-04 |
| `textcode` | 1: 1 part (P1); brings 0 | 1: 0.5.1, 2026-10-24 | 3: 1 user (and-cesbo) | 3: 0.x; 0.5.0 on 2026-08-30 and 0.4.0 on 2026-08-22, both inside 3 months |
| `tokio` | 1: 1 part (D2); brings 1 (1: pin-project-lite), with default features `[]`. With `process,signal,time`, the features nextest-runner names, it brings 6 | 1: 1.54.1, 2026-10-21 | 1: team `tokio-rs:core` + 2 users | 1: 1.0.1 on 2020-12-25 (1.0.0 is yanked) |
| `toml` | 1: 1 part (C1 if TOML, I1); brings 6 (6) | 1: 1.2.1+spec-1.1.0, 2026-11-06 | 1: team `toml-rs:maintainers` + epage | 2: ≥ 1.0; major 1.0.0 on 2026-02-11, inside 12 months |
| `toml_edit` | 1: 1 part (C1 if TOML, I1); brings 7 (7) | 1: 0.25.19+spec-1.1.0, 2026-11-06 | 1: team `toml-rs:maintainers` + 2 users | 2: 0.x; 0.25.0 on 2026-02-11, 0.24.0 on 2025-12-18 |
| `walkdir` | 1: 1 part (Sel1); brings 1 (1: same-file) | 3: 2.5.0, 2024-03-01 | 3: 1 user (BurntSushi) | 1: 2.0.0 on 2017-10-21 |
| `winnow` | 1: 1 part (Sel2); brings 0 | 1: 1.0.5, 2026-11-05 | 3: 1 user (epage) | 2: ≥ 1.0; major 1.0.0 on 2026-03-17, inside 12 months |

Churn method: for each compatibility key (`N` for N ≥ 1, `0.N`, `0.0.N`), the first release that is not yanked and not a prerelease is a breaking release. Build metadata such as `+spec-1.1.0` is ignored.

#### Upstream review, tests, `unsafe`

Review: reviewed commits of the last 30 on the default branch, and the window's dates. Tests: the trigger, the test command, and the observed run. `unsafe`: blocks compiled for Linux x86_64 with default features, and how many lack a SAFETY comment.

| crate | upstream review | tests | `unsafe` |
|---|---|---|---|
| `anstream` | 3: 1/30 on `rust-cli/anstyle`, 2026-08-27 → 2026-11-09 | 1: `ci.yml` pull_request + push main; `cargo hack test --each-feature --workspace`; HEAD "Test (ubuntu-latest, stable)" success, [run](https://github.com/rust-cli/anstyle/actions/runs/42688163079/job/129400474794) | 3: 2 compiled, both without by hand (`src/adapter/strip.rs:136`, `:148`) |
| `bpaf` | 3: 0/30, 2026-03-13 → 2026-11-12; pacak self-merges | 1: `check-and-lint.yaml` pull_request + push master; `cargo test -p bpaf -p bpaf_derive`; HEAD step "test minimal features" success, [run](https://github.com/pacak/bpaf/actions/runs/43270973516/job/131367084523) | 1: 0 blocks |
| `chumsky` | 3: 13/30 on Codeberg, 2026-09-01 → 2026-11-13; all 13 reviewed rows are one PR (#1014) | 1: Forgejo Actions runs `.github/workflows/rust.yml` on push and pull_request to main; `cargo test --all-features --all-targets`; HEAD commit status "Rust / test (push)" success, run #125 | 2: 30 compiled, each with a `// SAFETY:` line above it, read by hand |
| `clap` | 3: 0/30, 2026-09-14 → 2026-11-02; 13 rows are renovate[bot] | 1: `ci.yml` pull_request + push; `make test-…` → `cargo test`; the `ci.yml` push run at HEAD `ec93e71255` has "Test (linux)" success, [run](https://github.com/clap-rs/clap/actions/runs/41509788542/job/125499092227); the run as a whole is `failure`, from other jobs | 1: `#![forbid(unsafe_code)]` at `src/lib.rs:85`; `clap_builder` forbids it at `src/lib.rs:9` |
| `difflib` | 3: 0/30, 2017-01-12 → 2018-07-22 | 2: 13 `#[test]`; the only CI file is `.travis.yml`; HEAD has 0 statuses and 0 check runs | 1: 0 blocks |
| `encoding` | 3: `master` 0/30, 2015-04-16 → 2017-07-11; branch `0.2` (source of 0.2.33) 0/30, head 2016-08-28 | 2: 119 `#[test]`; the only CI file is `.travis.yml`; 0 check runs, 0 statuses, 0 Actions runs | 3: 5 compiled, 5 without (e.g. `src/util.rs:16`) |
| `encoding_rs` | 3: 0/30, 2026-09-07 → 2026-10-30; 21 direct pushes, 9 contributor commits merged with no review | 1: `ci.yml` push + pull_request; `cargo test`; HEAD "unit-test (ubuntu-latest)" success, [run](https://github.com/hsivonen/encoding_rs/actions/runs/41076253995/job/124133122340) | 3: 144 compiled, 110 without (123 by the strict 3-line window) |
| `gag` | 3: 6/30, 2015-04-09 → 2021-05-30; 24 pushed with no PR | 2: `tests/test_redirect.rs`, 2 `#[test]`; the only CI file is `.travis.yml`; HEAD has 0 statuses, 0 check runs, 0 Actions runs | 1: 0 blocks |
| `globset` | 3: 8/30 on `BurntSushi/ripgrep`, 2026-07-20 → 2026-11-09; of the 6 commits under `crates/globset`, 1 is reviewed | 1: ripgrep `ci.yml`; `cargo test --verbose --workspace --features pcre2`; HEAD step "Run tests with PCRE2 (sans cross)" success, [run](https://github.com/BurntSushi/ripgrep/actions/runs/43538328288/job/132272672294) | 1: 0 blocks |
| `ignore` | 3: 8/30 on ripgrep, the same window; 16 of the 30 touch `crates/ignore/` | 1: the same ripgrep CI; HEAD "test (stable-x86, …)" success, [run](https://github.com/BurntSushi/ripgrep/actions/runs/43538328288/job/132272672502) | 1: no `unsafe` token in the tarball |
| `imara-diff` | 2: 21/30, 2025-11-07 → 2026-06-21; PR #33 is 19 of the 30 rows | 1: `ci.yml` pull_request + push master; `cargo test --workspace`, `cargo miri test`; HEAD "Test" success, [run](https://github.com/pascalkuthe/imara-diff/actions/runs/27910030258/job/82585444568) | 3: 9 compiled, 8 without (e.g. `src/myers.rs:43`) |
| `junit-report` | 3: 2/30, 2026-03-13 → 2026-07-27; 24 rows are dependabot PRs merged by bachp with no approval | 1: `ci.yml` pull_request + push master; `cargo test`; HEAD job "Build (stable)" success, [run](https://github.com/bachp/junit-report-rs/actions/runs/30308649159/job/90118858854). The API gave no step list, so the success is the job's | 1: 0 blocks |
| `lexopt` | 3: 0/30, 2023-01-15 → 2026-02-28 | 1: `ci.yaml` push + pull_request; `cargo test`; HEAD "Test (ubuntu-latest)" success, [run](https://github.com/blyxxyz/lexopt/actions/runs/22530451818/job/65268996285) | 1: forbid at `src/lib.rs:77` |
| `nix` | 2: 24/30, 2026-05-11 → 2026-11-14; 5 self-merged PRs + 1 release push | 1: `ci.yml` pull_request + push master + merge_group; HEAD job "x86_64_linux_native_builds" step `test` success, [run](https://github.com/nix-rust/nix/actions/runs/43535718741/job/132264109687) | 3: 15 compiled, 15 without (e.g. `src/unistd.rs:1354`) |
| `nom` | 2: 16/30, 2024-12-08 → 2025-08-26 | 2: `ci.yml` names `cargo test` on push and pull_request, but no test job was observed. The 3 runs the API keeps are two `failure` runs with 0 jobs and one `action_required` run; HEAD has 0 check runs | 3: 4 compiled, 4 without (`src/traits.rs:507`, `:519`, `:542`, `:556`) |
| `normpath` | 3: 0/30, 2024-02-23 → 2026-09-19; 19 direct pushes | 1: `build.yml` pull_request + push + cron; `cargo test --features localization`; HEAD "test (ubuntu-latest, stable)" success, [run](https://github.com/dylni/normpath/actions/runs/43363436008/job/131682826354) | 2: 1 compiled (`src/base.rs:55`), with `// SAFETY:` above it |
| `os_pipe` | 3: 0/30, 2022-11-25 → 2025-10-11; 24 with no PR, 6 merged by the owner with no approval | 1: `ci.yml` push + pull_request, `cargo test`. HEAD has 0 runs. One observed run, on a fork PR (2025-12-24): all 9 `cargo_tests` jobs success, [run](https://github.com/oconnor663/os_pipe.rs/actions/runs/20493330400) | 3: 4 compiled, 4 without (`src/unix.rs:16`, `:20`, `:67`, `:103`) |
| `owo-colors` | 3: 1/30, 2025-02-22 → 2026-08-27 | 1: `ci.yml` push + pull_request main; `cargo test` / `cargo hack test`; HEAD steps "Run tests…" success, [run](https://github.com/owo-colors/owo-colors/actions/runs/33100849179/job/98617823305) | 1: forbid at `src/lib.rs:76` |
| `path-clean` | 3: 0/23 (the branch has 23 commits), 2018-11-06 → 2024-04-08 | 2: 8 `#[test]`. `ci.yml` exists, but the workflow state is `disabled_inactivity`, with 0 Actions runs and 0 check runs at HEAD | 1: forbid at `src/lib.rs:27` |
| `pep508_rs` | 3: 1/30, 2024-07-30 → 2026-10-20 | 1: `test.yml` push + pull_request; `cargo test`; HEAD steps 4-5 success, [run](https://github.com/konstin/pep508_rs/actions/runs/39476604645/job/118814063991) | 1: 0 blocks |
| `pest` | 2: 16/30, 2026-07-29 → 2026-11-13; tomtau self-merges 13 of his own PRs | 1: `ci.yml` push + pull_request; `cargo test --all …`; HEAD "Unit, Style, and Lint Testing" success, [run](https://github.com/pest-parser/pest/actions/runs/43429589587/job/131899884948) | 1: no `unsafe` token in the tarball |
| `petgraph` | 3: 1/30, 2026-01-04 → 2026-10-31; 29 merged with no approval | 1: `ci.yml` pull_request + merge_group, `merge-check.yml` push master; `cargo nextest run`; HEAD "Tests petgraph (Rust stable)" success, [run](https://github.com/petgraph/petgraph/actions/runs/41209959299/job/124562680474) | 3: 8 compiled, 7 without; only `src/matrix_graph.rs:1063` has SAFETY |
| `pico-args` | 3: 0/30, 2020-10-27 → 2023-10-19 | 2: `main.yml` push + pull_request, `cargo test` ×5; 62 `#[test]`. No run observed: HEAD has 0 check runs and 0 statuses, and `actions/runs` is 0 | 1: forbid at `src/lib.rs:33` |
| `polling` | 2: 18/30, 2025-04-29 → 2026-10-12 | 1: `ci.yml` pull_request + push master + cron; `cargo test`; HEAD push run "test (ubuntu-latest, stable)" success, [run](https://github.com/smol-rs/polling/actions/runs/38265947874/job/114868017730). ⚠️ The 2026-11-08 scheduled run on the same SHA fails at `cargo build`, and the latest PR run is `failure` | 3: 4 compiled, all in `src/epoll.rs`, 4 without |
| `postcard` | 2: 18/30, 2025-08-08 → 2026-11-06; only 11 of the 30 touch `source/postcard/` | 1: `ci.yml` push + pull_request main → `ci.sh` `cargo test --all`; HEAD job "ubuntu" success, [run](https://github.com/jamesmunns/postcard/actions/runs/42214520478/job/127881082544) | 2: 8 compiled, each with a `// SAFETY:` line above it, read by hand |
| `pyo3-async-runtimes` | 1: 28/30, 2025-09-28 → 2026-10-20 | 1: `ci.yml` push + pull_request main; `cargo test --all-features`; HEAD step "Test" success, [run](https://github.com/PyO3/pyo3-async-runtimes/actions/runs/39459726118/job/118866225443) | 1: 0 blocks, in 0.29.0 and in 0.30.0 |
| `pyproject-toml` | 3: 3/30, 2024-10-28 → 2026-02-25; 13 renovate/dependabot PRs merged with no approval | 1: `CI.yml` push main + pull_request; `cargo test`; HEAD "Test Suite (ubuntu-latest)" success, [run](https://github.com/PyO3/pyproject-toml-rs/actions/runs/22401756063/job/64850086115) | 1: no `unsafe` token in the tarball |
| `quick-junit` | 3: 0/30, 2026-07-03 → 2026-09-10; 22 rows are `nextest-bot` dependency PRs | 1: `ci.yml` push + pull_request main; `just powerset nextest run`; HEAD "Build and test (ubuntu-latest, stable)" success, [run](https://github.com/nextest-rs/quick-junit/actions/runs/34526521602/job/103036655722) | 1: 0 blocks |
| `quick-xml` | 2: 20/30, 2026-10-25 → 2026-11-11 | 1: `rust.yml` push + pull_request; `cargo test` feature variants; HEAD "test (ubuntu-latest)" success, [run](https://github.com/tafia/quick-xml/actions/runs/42943760000/job/130271480676) | 1: forbid at `src/lib.rs:47` |
| `rmp-serde` | 3: 0/30, 2024-04-16 → 2025-12-23; all 30 are direct pushes by kornelski | 1: `ci.yml` push + pull_request; `cargo test --all --all-features`; HEAD "Tests (stable)" success, [run](https://github.com/3Hren/msgpack-rust/actions/runs/20474274314/job/58835744962) | 1: forbid at `src/lib.rs:2`, 0 blocks |
| `rustix` | 3: 4/30, 2026-08-25 → 2026-11-10; sunfishcode merges contributor PRs with no approval | 1: `main.yml` push main + pull_request; `cargo test --verbose --features=all-apis --release --workspace`; HEAD "Test (aarch64-linux-1.65)" and others success, [run](https://github.com/bytecodealliance/rustix/actions/runs/42851269445/job/129952274303). The run as a whole is `cancelled`, because "Test (macos-14)" was cancelled | 3: 31 compiled, 14 without, all in `src/backend/linux_raw/io/syscalls.rs`; the file has one module-level safety comment, not one per block |
| `seccompiler` | 1: 29/30, 2023-02-02 → 2026-07-27; the repository is **archived** | 1: no Actions test workflow. Buildkite through the `rust-vmm-ci` submodule runs `cargo all-features test --workspace` on pull request and push; HEAD status `buildkite/seccompiler-ci` success, [build 240](https://buildkite.com/rust-vmm/seccompiler-ci/builds/240). The contents of build 240 were not opened [C] | 2: 2 compiled (`src/lib.rs:344`, `:358`), each with `// SAFETY:` above it |
| `serde_json` | 2: 16/30, 2026-05-21 → 2026-10-16; 14 direct pushes by dtolnay | 1: `ci.yml` push + pull_request + cron; `cargo test`; HEAD "Rust stable" success, [run](https://github.com/serde-rs/json/actions/runs/43573572666/job/132382893905) | 3: 9 compiled, 3 without (`src/ser.rs:2250`, `:2269`, `src/read.rs:749`); 4 by the strict 3-line window |
| `similar` | 3: 0/30, 2026-08-09 → 2026-10-24; 27 pushed with no PR | 1, with a caveat: `tests.yml` runs on `push` only, not on pull_request; `make test` → `cargo test`; HEAD "Test on Latest" success, [run](https://github.com/mitsuhiko/similar/actions/runs/40091359155/job/120864525668) | 1: 0 blocks |
| `supports-color` | 3: 4/30, 2021-10-01 → 2024-11-26 | 2: `miri.yml` push + pull_request, `cargo miri test`; 5 `#[test]`. No run observed: HEAD has 0 check runs and 0 statuses, and `actions/runs` is 0 | 1: 0 blocks |
| `textcode` | 3: 0/30, 2022-05-02 → 2026-10-24; 26 direct pushes | 2: 50 `#[test]` across `tests/*.rs`; no CI file at HEAD; 0 check runs, 0 Actions runs | 3: 3 compiled (`src/lib.rs:83`, `src/dvb.rs:172`, `:189`), each with `// SAFE: writes valid UTF-8 …` above it, which does not match `/safety/i` |
| `tokio` | 1: 29/30, 2026-10-29 → 2026-11-10 | 1: `ci.yml` push + pull_request on master; `cargo nextest run --features full` + `cargo test --doc`; all 96 check runs at HEAD `ab0814fc8c` succeed, among them tokio's own suite, e.g. "cross-test-with-parking_lot (i686-unknown-linux-gnu, ubuntu-latest, taskdump)", [run](https://github.com/tokio-rs/tokio/actions/runs/42888588090/job/130084505071) | 3: default features `[]`; 28 compiled from tokio's source, 13 without (e.g. `src/io/read_buf.rs:34`, all 7 in `src/runtime/park.rs`), from rustc's expansion (red-team). The slice's walker missed `park.rs` and gave 23 and 6. With `process,signal,time` the slice gave 58 compiled, 21 without, which likely misses the same 7 [C] |
| `toml` | 3: 0/30 on `toml-rs/toml`, 2026-10-30 → 2026-11-06; epage and renovate[bot] self-merge | 1: `ci.yml` pull_request + push; `cargo hack test`; HEAD "Test (ubuntu-latest, stable)" success, [run](https://github.com/toml-rs/toml/actions/runs/42228973353/job/127930417170) | 1: forbid at `src/lib.rs:150`, 0 blocks |
| `toml_edit` | 3: the same repository reading as `toml`; 9 of the 30 touch `crates/toml_edit/` | 1: the same reading as `toml` (the workspace members are `crates/*`) | 1: no `unsafe` token in the tarball |
| `walkdir` | 3: 3/30, 2019-07-20 → 2024-12-31 | 1: `ci.yml` pull_request + push master + cron; `cargo test --verbose`; the nightly scheduled run at HEAD has "test (stable, ubuntu-latest, stable)" success, [run](https://github.com/BurntSushi/walkdir/actions/runs/43538568960/job/132273450803). ⚠️ In that run "test (pinned-win, …)" fails, and the latest PR run is `failure` | 1: no `unsafe` token in the tarball |
| `winnow` | 3: 0/30, 2026-08-27 → 2026-11-05; epage merges PRs with no approval | 1: `ci.yml` pull_request + push; `cargo hack test --each-feature --skip debug --workspace`; HEAD "Test (ubuntu-latest, stable)" success, [run](https://github.com/winnow-rs/winnow/actions/runs/41947652581/job/126983979993). The run is `failure` overall, from its "Coverage" and "advisories" jobs | 3: 22 compiled, 2 without (`src/stream/bstr.rs:36`, `src/stream/bytes.rs:36`) |

`unsafe` counts only the crate itself. The crates it brings were not measured, as in #960's audit. `unsafe fn`, `unsafe impl` and `unsafe trait` are counted by the slices but do not set the level. `unsafe` on macOS and other targets was not evaluated [C].

## 3. Readings that move a level

The method can be read in more than one way. Each row changes one input of one crate, and gives the pain that results. The other inputs keep their primary levels. The pain is computed by the same script ([Appendix B7](#b7-this-file-pain)).

| crate | input | primary level | alternative reading | alternative level | pain: primary → alternative | tier |
|---|---|---|---|---|---|---|
| `pyo3-async-runtimes` | review | 1 (28/30) | first-parent 30: its PRs from 2024-11 to 2025-09 were merged with no approval | 2 (20/30) | 1.93 → 2.00 | [A] |
| `serde_json` | review | 2 (16/30) | first-parent 30 | 3 (8/30) | 1.86 → 1.93 | [A] |
| `postcard` | review | 2 (18/30) | first-parent 30 | 3 (10/30) | 2.00 → 2.07 | [A] |
| `rustix` | review | 3 (4/30) | a merge by another person counts as review | 2 (25/30) | 2.07 → 2.00 | [A] |
| `ignore` | review | 3 (8/30) | a merge by another person counts as review (ripgrep) | 2 (15/30) | 2.00 → 1.93 | [A] |
| `globset` | review | 3 (8/30) | the same ripgrep reading | 2 (15/30) | 1.50 → 1.43 | [A] |
| `chumsky` | review | 3 (13/30) | a merge by another person counts as review | 1 (27/30) | 2.00 → 1.86 | [A] |
| `pyproject-toml` | review | 3 (3/30) | a merge by another person counts as review; 13 of them are renovate/dependabot PRs | 2 (18/30) | 2.43 → 2.36 | [A] |
| `encoding` | review | 3 (0/30 on `master` and on branch `0.2`) | a merge by another person counts as review, on `master` only. 0.2.33 comes from branch `0.2`, and the slice gives no such count for that branch | 2 (17/30) | 1.86 → 1.79 | [A] |
| `quick-xml` | review | 2 (20/30) | a merge by another person counts as review | 1 (28/30) | 1.36 → 1.29 | [A] |
| `imara-diff` | review | 2 (21/30) | a merge by another person counts as review | 1 (30/30) | 2.14 → 2.07 | [A] |
| `junit-report` | review | 3 (2/30) | a merge by another person counts as review; 24 of them are dependabot PRs merged by bachp | 2 (26/30) | 2.00 → 1.93 | [A] |
| `difflib` | review | 3 (0/30) | a merge by another person counts as review | 2 (16/30) | 2.21 → 2.14 | [A] |
| `anstream` | review | 3 (1/30) | a merge by another person counts as review (anstyle) | 2 (17/30) | 1.50 → 1.43 | [A] |
| `chumsky` | blast | 2 (16 crates with build edges) | normal edges only, as the brief worded it | 1 (10 crates) | 2.00 → 1.50 | [A] |
| `pest` | blast | 2 (12 crates with build edges) | normal edges only | 1 (7 crates) | 1.71 → 1.21 | [A] |
| `pico-args` | tests | 2 (no run observed) | a workflow file that runs `cargo test` on push and pull_request is enough | 1 | 1.71 → 1.64 | [A] |
| `supports-color` | tests | 2 (no run observed) | a workflow file is enough (`cargo miri test`) | 1 | 1.57 → 1.50 | [A] |
| `nom` | tests | 2 (no test job observed) | a workflow file is enough | 1 | 1.64 → 1.57 | [A] |
| `pyo3-async-runtimes` | maintainers | 3 (1 user owner) | `trustpub_only`: only a workflow in `PyO3/pyo3-async-runtimes` can publish, so the organisation counts as "an organisation account" | 1 | 1.93 → 1.79 | [A] |
| `quick-junit` | maintainers | 2 (sunshowers + nextest-bot) | the bot is not counted as a person | 3 | 2.00 → 2.07 | [A] |
| `textcode` | `unsafe` | 3 (`// SAFE:` does not match `/safety/i`) | a person accepts `// SAFE: writes valid UTF-8 …` as a SAFETY comment | 2 | 1.79 → 1.71 | [A] for the comments; the reading is [C] |
| `petgraph` | `unsafe` | 3 (7 of 8 without) | count `// safe because a, b are in bounds and distinct` (`src/graph_impl/mod.rs:489`), which has no word SAFETY | 3 (6 of 8 still without) | 1.57 → 1.57 | [A] |
| `anstream` | `unsafe` | 3 (0 of 2 with SAFETY, by hand) | the 3-line window heuristic, which matches the parameter name `safety_justification` at `src/adapter/strip.rs:147` | 3 (1 of 2 with) | 1.50 → 1.50 | [A] |

Readings that move no level:

- **First parent, no move.** `imara-diff` 21 → 15/30, exactly on the level-2 floor. `polling` 20, `tokio` 29, `nom` 22, `quick-xml` 21. Every other repository either has no merge commit in its 30, or keeps its level.
- **Blast with normal edges only, no move** for the other 39 crates.
- **More features, no move.** `tokio` with `process,signal,time`: the slice gave 58 blocks, 21 without SAFETY, likely short by the 7 of `runtime/park.rs` (§ 2): still 3. `nix` with `signal` compiles 110 blocks, all without: still 3.
- **A reason comment without the word SAFETY.** Two of `serde_json`'s 3 blocks (`src/ser.rs:2250`, `:2269`) carry "We do not emit invalid UTF-8." The third, `src/read.rs:749`, carries "The deserialization input came in as &str with a UTF-8 guarantee, and the \u-escapes are checked along the way, so don't need to check here." (three comment lines at `:746`-`:748`, joined with one space) [A] `nom`'s `src/traits.rs:507` has a reason comment below it. Neither slice gave a level for a reading that accepts such a comment.
- **`rustix`, maintainers.** It has one user owner, and its repository belongs to the `bytecodealliance` organisation. The checks slice gives the literal reading, 3, and names no other level.
- **`seccompiler`, maintainers** reads 1 (a team owner), but its repository is archived.
- **`pyo3-async-runtimes` 0.29.0 or 0.30.0**: blast 2 and `unsafe` 1 for both.

## 4. Evidence for the person's scores

### Trust

Trust is a person's score: does the name, the repository and the publisher match the project meant; does crates.io show a malicious-code notice; is the name close to a better-known crate's. These facts come from the checks and review slices.

**For every candidate** [A]:

- The `repository` field on crates.io equals the repository in the needs slice, and equals the repository's `full_name` on GitHub or Codeberg, for 39 of 41. `pyproject-toml` and `anstream` add a `.git` suffix. `ignore` and `globset` name a subdirectory of `BurntSushi/ripgrep` (`/tree/master/crates/<name>`), so their field equals the repository only up to that path. No repository redirects to another name.
- RustSec (`advisory-db` HEAD `4ef2df8`, 2026-11-13) has no `malicious` and no `withdrawn` advisory for any candidate. The crates.io crate record has no notice field. Whether crates.io shows a notice anywhere else is [C].
- Whether any name is close to a better-known crate's was not searched systematically [C]. Seen in passing: `encoding` and `encoding_rs` are two separate, real projects with different repositories. `toml` and `toml_edit` share one repository.
- Every candidate except `textcode` has at least 5.9 M downloads in total.

**Per crate.** "Listed owner" means the newest version was published by a user who is a listed owner. "Tag on the default branch" means the release tag is an ancestor of the default-branch HEAD.

| crate | who published the newest version | where the release comes from | other facts |
|---|---|---|---|
| `anstream` | listed owner | tag on the default branch | |
| `bpaf` | listed owner | tag on the default branch | |
| `chumsky` | listed owner | no tag; `.cargo_vcs_info.json` names `41be9b6fb7`, the default-branch HEAD | **published from a dirty tree**: `.cargo_vcs_info.json` has `"dirty": true` |
| `clap` | epage, **not a listed user owner**; clap has team owners, and whether epage is in them is [C] | tag on the default branch | 92 of 462 versions yanked |
| `difflib` | none recorded (`published_by: null`, 2018) | **no tag** for 0.4.0 | |
| `encoding` | none recorded (`published_by: null`, 2016) | **branch `0.2`, not the default branch.** Commit `720bc03b5d` "0.2.33: A maintenance release to fix an error in Rust 1.12+." matches by subject [A]. That this exact commit was published is [C]: the crate has no `.cargo_vcs_info.json` | RustSec marks it unmaintained (RUSTSEC-2021-0153) |
| `encoding_rs` | listed owner | tag on the default branch | |
| `gag` | listed owner | tag on the default branch | |
| `globset` | listed owner | tag on the default branch (ripgrep) | |
| `ignore` | listed owner | tag on the default branch (ripgrep) | |
| `imara-diff` | listed owner | tag on the default branch | |
| `junit-report` | listed owner | **tag `v0.9.1` points at `609acaba35`**, a child of master HEAD `b936357a9f` that is on no branch | |
| `lexopt` | listed owner | tag on the default branch | |
| `nix` | SteveLauC, **not a listed user owner**; nix has a team owner, and whether SteveLauC is in it is [C] | tag on the default branch | |
| `nom` | listed owner | tag on the default branch | |
| `normpath` | listed owner | tag on the default branch | |
| `os_pipe` | listed owner | tag on the default branch | |
| `owo-colors` | **trusted publishing** from `owo-colors/owo-colors`; `trustpub_only = true` | tag on the default branch | |
| `path-clean` | listed owner | no tag; `.cargo_vcs_info.json` names `d8948ae69d` on the default branch | |
| `pep508_rs` | listed owner | tag on the default branch | |
| `pest` | **trusted publishing** from `pest-parser/pest` | tag on the default branch | |
| `petgraph` | listed owner | tag on the default branch | |
| `pico-args` | listed owner | tag on the default branch | |
| `polling` | listed owner | tag on the default branch | |
| `postcard` | listed owner | tag on the default branch | |
| `pyo3-async-runtimes` | **trusted publishing** from `PyO3/pyo3-async-runtimes`; `trustpub_only = true` | tag on the default branch | |
| `pyproject-toml` | konstin, **not a listed user owner**; it has a team owner, and whether konstin is in it is [C] | tag on the default branch | |
| `quick-junit` | **trusted publishing** from `nextest-rs/quick-junit`; `trustpub_only = true` | tag on the default branch | |
| `quick-xml` | listed owner | tag on the default branch | |
| `rmp-serde` | listed owner | no tag; `.cargo_vcs_info.json` names `cf880019f7`, the default-branch HEAD | 35 of 43 versions yanked |
| `rustix` | listed owner | tag on the default branch | |
| `seccompiler` | listed owner | tag on the default branch | **repository archived.** Its GitHub description: "This is a public archive. The code now lives in the mono-repo: https://github.com/rust-vmm/rust-vmm/". The crates.io `repository` field still points to the archived repository |
| `serde_json` | listed owner | tag on the default branch | |
| `similar` | **trusted publishing** from `mitsuhiko/similar` | tag on the default branch | |
| `supports-color` | listed owner | tag on the default branch | |
| `textcode` | listed owner | no tag; `.cargo_vcs_info.json` names `6e0612f3a3`, the default-branch HEAD | **110,875 downloads in total and 22,563 recent.** That is about 1/50 of the next-smallest total (`gag`, 5,964,457), and 1/30 of the next-smallest recent count (`junit-report`, 692,742) |
| `tokio` | listed owner | tag on the default branch | |
| `toml` | listed owner | tag `toml-v1.2.1` on the default branch (`563cb1a5d1`) | |
| `toml_edit` | listed owner | tag `v0.25.19` on the default branch (`563cb1a5d1`) | |
| `walkdir` | listed owner | tag on the default branch | |
| `winnow` | listed owner | tag on the default branch | |

The checks slice counts 31 crates published by a listed user owner, 5 by trusted publishing, 3 by a user who is not a listed user owner, and 2 with no publisher recorded [A]. It does not name the login of each of the 31.

### Relief

Relief is a person's score. The inputs state only what already meets each need, so this section gives that and no score.

| need | what meets it today (from § 1) |
|---|---|
| S2, D1 | **a purba crate**: `libc` 0.2.189 has `fork`, `waitpid`, `pipe`, `kill` and `posix_spawn`. **std**: `process::Command` for a spawn |
| S3, D3 | **partly**: std gives the channel (`io::pipe`, `UnixStream::pair`), and `serde` derives `Serialize` and `Deserialize`. No purba crate writes a wire format |
| P1 | **partly**: std decodes UTF-8 only. `pyo3` could call `importlib.util.decode_source`, but #644 records that "parsing crosses the seam, and the record above changes" |
| C1, I1 | **none** parses a file. `serde` derives the keys, and std reads the bytes |
| Sel1 | **partly**: std `fs::read_dir`, with the recursion written by hand |
| Sel2 | **partly**: std, with a parser written by hand |
| Sel3 | **std**: `std::path::absolute` and `fs::canonicalize` |
| F1, E2 | **a purba crate**: `pyo3` calls `asyncio` as it calls any Python function |
| E1 | **partly**: `libc` (`pipe`, `dup2`) and std `io::pipe` |
| D2 | **partly**: `libc` (`kill`, `waitpid`, `poll`) and std `time::Instant` |
| D5 | **std**: the collections, with a graph written by hand |
| R1 | **partly**: `serde` derives the shape. No purba crate writes a format |
| R2 | **none** |
| Cmd1 | **partly**: `pyo3` copies `sys.argv` into a `Vec<String>`, and command parses it by hand |
| I2 | **none** |
| I3 | **std**: `io::IsTerminal` finds a terminal. Nothing colours |
| I4 | **none** |
| I5 | **std** / `pyo3` give the platform |
| I6 | **none** |

Facts that bear on relief, from the needs slice:

- `encoding_rs` reads the label `latin-1` as windows-1252. #644 records it: "the byte `0x80` is `'\x80'` in CPython and `'€'` there". No crate found implements Python's own codec table [A for the searches; C for "no such crate exists"].
- No candidate has been built, linked or probed against purba. Each "could meet" rests on the crate's crates.io description, or on a comparable project that uses the crate for the same job [A].
- `pyo3-async-runtimes` 0.30.0 needs `pyo3 ^0.30`, and purba pins 0.29.2 (§ 7).

## 5. Considered and left out

From the needs slice, copied byte for byte.

| crate | why it is not a candidate | evidence |
|---|---|---|
| `bincode` | 3.0.0 holds "only this README, as well as a lib.rs containing only a compiler error" | `/api/v1/crates/bincode/3.0.0/readme` [A] |
| `uv-pep508` | its description: "This is an internal component crate of uv" | crates.io API [A] |
| `target-spec` | it evaluates "Cargo.toml target specifications", which are Rust targets, not a Python platform | crates.io API [A]. nextest uses it ([L147](https://github.com/nextest-rs/nextest/blob/cargo-nextest-0.9.152/Cargo.toml#L147)) |
| `encoding_rs_io` | a streaming adapter over `encoding_rs`. P1 decodes one whole file | crates.io API [A]; uv uses it ([L174](https://github.com/astral-sh/uv/blob/0.13.8/Cargo.toml#L174)) |
| `rusty-fork`, `fork`, `ipc-channel` | surfaced by `q=fork process unix`. Each aims at a different job: a Rust test in a subprocess, a daemon, and a channel between processes. ⚠️ `ipc-channel` could be weighed for D3 by the design of dispatch | crates.io search [A] |

## 6. Doubts

### Where the inputs disagree

| what | one input says | another says | what this file does |
|---|---|---|---|
| whether travis-ci.org still runs these builds (`encoding`, `gag`, `difflib`) | review-a: "Travis no longer runs open-source builds from travis-ci.org" [B] | review-b: the same claim "is prior knowledge and was not checked" [C] | no level rests on it: each crate scores 2 because no run is observed. The claim stays at [C], the lower tier |
| the version of `pyo3-async-runtimes` | checks: 0.30.0, the newest stable | blast-b: 0.29.0 first, because purba pins `pyo3` 0.29.2, then 0.30.0 as well | the tables use 0.30.0. Blast (2) and `unsafe` (1) are the same for both versions |
| how many crates `blast area` counts | the shared brief and blast-a's main table: normal edges only. Under it, `chumsky` and `pest` are level 1 | #960's audit (`-e normal,build,dev`), and the second table of blast-a and blast-b (`-e normal,build`). A probe crate's tree has no dev edges, so the two agree. Under them, both are level 2 | the primary reading is normal and build edges, as #960 counted. § 3 gives the other |
| `textcode`'s downloads against the next-smallest | checks, its summary: "about 1/100" | checks, its trust table: "about 1/50" by total (`gag`, 5,964,457) and 1/30 by recent (`junit-report`, 692,742) | this file uses the raw counts: 110,875 total and 22,563 recent. 110,875 / 5,964,457 is about 1/54 |
| how many review-b crates score review 3 | review-b, its summary: "14 of the 20" | review-b, its table: 16 (`gag`, `os_pipe`, `petgraph`, `quick-junit`, `junit-report`, `lexopt`, `pico-args`, `bpaf`, `clap`, `similar`, `difflib`, `anstream`, `owo-colors`, `supports-color`, `globset`, `pep508_rs`) | this file uses the table |
| `nom`'s reason comments | blast-a, its summary: in `nom`, "those blocks carry a comment that gives a reason" | blast-a, its table: none of the 4 blocks has SAFETY, and it names one reason comment, below `src/traits.rs:507` | this file reports the table. The level is 3 either way |
| how `anstream`'s 2 blocks are counted | the #960 rule (the 3-line window): 1 of 2 has SAFETY, from the parameter name `safety_justification` | blast-b, read by hand: 0 of 2 | the level is 3 either way. § 3 has the row |
| how `serde_json`'s and `encoding_rs`'s blocks are counted | the 3-line window: 4 of 9, and 123 of 144, without SAFETY | blast-a's main figure, which also reads the unbroken run of comment lines above a block: 3 of 9, and 110 of 144 | both are level 3 either way |
| the tier of "`pyo3-async-runtimes` 0.30.0 needs `pyo3 ^0.30`" | the needs slice: [A] | checks: [B], relayed from the needs slice | [A], from the slice that fetched it |

Two slices measured `BurntSushi/ripgrep` on their own, for `ignore` and for `globset`. They agree: 8/30, review level 3, tests level 1, and 15/30 under the "merged by another person" reading.

### What could not be established [C]

| crate | what |
|---|---|
| all 41 | whether another part will name the crate's types. No part has a design, so every blast level is a floor and would become 3 |
| all 41 | whether crates.io shows a malicious-code notice anywhere outside the API's crate record |
| all 41 | whether a name is close to a better-known crate's name. No systematic search was made |
| all 41 | what classic branch protection requires on each GitHub repository, and Codeberg's `branch_protections` for `chumsky`. Both need privileged access |
| the 21 crates of blast-a | `unsafe` on macOS or any target other than Linux x86_64. The other-target counts are listed only. Blast-b read the other-OS files of `os_pipe`: on macOS 6 blocks would compile, all without SAFETY [A], but no level was given |
| `textcode` | whether `// SAFE:` counts as a SAFETY comment. The #960 rule says no |
| `petgraph` | which part owns D5: #691's third item is "which part owns the plan", so its single part is itself [C] |
| `nom` | whether CI runs the tests today. No test job was observed |
| `os_pipe`, `pico-args`, `supports-color` | why the HEAD of each has no CI run. One possible cause is that GitHub expires old runs; that is review-b's assumption, not verified |
| `pico-args`, `supports-color` | whether CI ran on any change |
| `seccompiler` | what build 240 on Buildkite contains. Only its status was read |
| `encoding` | the exact commit 0.2.33 was published from. Branch `0.2` commit `720bc03b5d` matches by subject only |
| `encoding`, `gag`, `difflib` | whether travis-ci.org still runs their builds (see the table above) |
| `rustix` | whether commit `8e6aaa6713` "Merge commit from fork" is a security-advisory merge |
| `nix`, `clap`, `pyproject-toml` | whether the publisher of the newest version belongs to the team that owns the crate |
| `toml`, `toml_edit`, `pyproject-toml` | the format of purba's configuration. C1 is a stated need, and its TOML candidates are valid only if the design of configuration picks TOML |
| `seccompiler` | whether I6 is a need at all. #953 lists it as a gap, not a need |
| `encoding_rs`, `encoding`, `textcode` | whether any crate implements Python's own codec table. The searches found none [A], but "no such crate exists" is [C] |

### Relayed and not re-verified [B]

- The parts of each crate: a projection from the needs slice, for all 41.
- `path-clean`: "a workflow disabled this way does not run on push or PR until someone re-enables it" (review-a).
- `encoding`: "GitHub PR approvals did not exist before September 2016", so most of rust-encoding's window could not carry an approval (review-a).
- `encoding`: "cargo-audit and cargo-deny report unmaintained advisories separately from vulnerabilities" (checks).
- The blast-b slice read each crate's `max_stable_version` from the needs slice's crates.io cache [B]. The tarball checksums matched the locks it resolved itself [A].

### Other doubts

- **The windows differ a lot.** `toml-rs/toml`'s 30 commits cover 7 days, and `quick-xml`'s 17. `walkdir`'s cover 5.5 years, and `difflib`'s 2017 to 2018. A review level on an old window measures how the project worked then.
- **Monorepos dilute the window.** The level is the repository's, as the method sets it. Of the 30 commits, these touch the crate's own directory: `ignore` 16, `globset` 6, `postcard` 11, `toml` 20, `toml_edit` 9, `rmp-serde` 20, `pest` 16, `tokio` 23, `anstream` 4.
- **Red jobs at HEAD** in `polling`, `walkdir` and `winnow`, and a cancelled job in `rustix`. The test jobs named in the tables passed.
- **`similar` tests on `push` only.** A contributor PR is tested only after it is merged.
- **Pending first-time PR runs.** The CI of `postcard`, `rmp-serde` and `nom` holds a first-time contributor's PR run at `action_required` until a maintainer approves it.
- **D5 comes from #691, which "belongs to no map"**, and R2 (JUnit) rests on a line that places JUnit and does not admit it. The design of each part may drop either need.
- **No candidate has been built, linked or probed against purba.**

## 7. Outside the question

- **`rustix` 1.1.5 in `scripts/check-prose/Cargo.lock`.** RUSTSEC-2026-0486 (*unsound*, xattr functions, 2026-10-27) names `>= 1.0.0, < 1.1.6`, so it names that version. The root `Cargo.lock` has no `rustix`. Whether check-prose reaches the xattr functions was not checked. #960's audit covered direct crates only, so it did not see this [A]. Now filed as [#973](https://github.com/kalonji-tools/purba/issues/973).
- **A record already weighed `clap`** for Cmd1: "Command reads its argument without clap. clap exits 2 on a usage error, and 2 means `unknown` here." ([B/docs/decisions/a-run-exits-0-only-when-it-checked-something-and-every-test-passed.md#L73](https://github.com/kalonji-tools/purba/blob/2790c1e/docs/decisions/a-run-exits-0-only-when-it-checked-something-and-every-test-passed.md#L73), bold marks removed). `clap` is measured here because the needs slice lists it.
- **`pyo3-async-runtimes` 0.30.0 requires `pyo3 ^0.30`, and purba pins `pyo3` 0.29.2.** Its 0.29.0 (published 2026-06-12) requires `pyo3 ^0.29` [A].

## Appendix A: Liveness

The default branch's last commit (committer date of its head) and the latest release (the newest stable version on crates.io, and its date). Never `pushed_at`, never stars. All [A], from the checks slice; the review slices' windows end on the same dates.

| repository | crates | default branch | last commit | latest release |
|---|---|---|---|---|
| [3Hren/msgpack-rust](https://github.com/3Hren/msgpack-rust) | `rmp-serde` | master | 2025-12-23 | 1.3.1, 2025-12-23 |
| [BurntSushi/ripgrep](https://github.com/BurntSushi/ripgrep) | `ignore`, `globset` | master | 2026-11-09 (under `crates/ignore`: 2026-11-09; under `crates/globset`: 2026-10-27) | `ignore` 0.4.34, 2026-10-27; `globset` 0.4.21, 2026-10-27 |
| [BurntSushi/walkdir](https://github.com/BurntSushi/walkdir) | `walkdir` | master | 2024-12-31 | 2.5.0, 2024-03-01 |
| [DimaKudosh/difflib](https://github.com/DimaKudosh/difflib) | `difflib` | master | 2018-07-22 | 0.4.0, 2018-07-22 |
| [PyO3/pyo3-async-runtimes](https://github.com/PyO3/pyo3-async-runtimes) | `pyo3-async-runtimes` | main | 2026-10-20 | 0.30.0, 2026-10-15 |
| [PyO3/pyproject-toml-rs](https://github.com/PyO3/pyproject-toml-rs) | `pyproject-toml` | main | 2026-02-25 | 0.13.7, 2025-10-09 |
| [RazrFalcon/pico-args](https://github.com/RazrFalcon/pico-args) | `pico-args` | master | 2023-10-19 | 0.5.0, 2022-06-04 |
| [Stebalien/gag-rs](https://github.com/Stebalien/gag-rs) | `gag` | master | 2021-05-30 | 1.0.0, 2021-05-30 |
| [bachp/junit-report-rs](https://github.com/bachp/junit-report-rs) | `junit-report` | master | 2026-07-27 | 0.9.1, 2026-07-27 |
| [blyxxyz/lexopt](https://github.com/blyxxyz/lexopt) | `lexopt` | master | 2026-02-28 | 0.3.2, 2026-02-28 |
| [bytecodealliance/rustix](https://github.com/bytecodealliance/rustix) | `rustix` | main | 2026-11-10 | 1.1.6, 2026-10-27 |
| [cesbo/textcode](https://github.com/cesbo/textcode) | `textcode` | master | 2026-10-24 | 0.5.1, 2026-10-24 |
| [clap-rs/clap](https://github.com/clap-rs/clap) | `clap` | main | 2026-11-02 | 4.6.7, 2026-09-14 |
| [danreeves/path-clean](https://github.com/danreeves/path-clean) | `path-clean` | master | 2024-04-08 | 1.0.1, 2023-02-24 |
| [dylni/normpath](https://github.com/dylni/normpath) | `normpath` | master | 2026-09-19 | 1.5.2, 2026-09-19 |
| [hsivonen/encoding_rs](https://github.com/hsivonen/encoding_rs) | `encoding_rs` | main | 2026-10-30 | 0.8.44, 2026-10-30 |
| [jamesmunns/postcard](https://github.com/jamesmunns/postcard) | `postcard` | main | 2026-11-06 | 1.1.3, 2025-07-24 |
| [konstin/pep508_rs](https://github.com/konstin/pep508_rs) | `pep508_rs` | main | 2026-10-20 | 0.9.2, 2025-01-02 |
| [lifthrasiir/rust-encoding](https://github.com/lifthrasiir/rust-encoding) | `encoding` | master (0.2.33 is from branch `0.2`, head 2016-08-28) | 2017-07-11 | 0.2.33, 2016-08-28 |
| [mitsuhiko/similar](https://github.com/mitsuhiko/similar) | `similar` | main | 2026-10-24 | 3.2.1, 2026-10-24 |
| [nextest-rs/quick-junit](https://github.com/nextest-rs/quick-junit) | `quick-junit` | main | 2026-09-10 | 0.8.0, 2026-09-10 |
| [nix-rust/nix](https://github.com/nix-rust/nix) | `nix` | master | 2026-11-14 | 0.31.3, 2026-05-11 |
| [oconnor663/os_pipe.rs](https://github.com/oconnor663/os_pipe.rs) | `os_pipe` | master | 2025-10-11 | 1.2.3, 2025-10-11 |
| [owo-colors/owo-colors](https://github.com/owo-colors/owo-colors) | `owo-colors` | main | 2026-08-27 | 4.4.0, 2026-08-27 |
| [pacak/bpaf](https://github.com/pacak/bpaf) | `bpaf` | master | 2026-11-12 | 0.9.29, 2026-11-12 |
| [pascalkuthe/imara-diff](https://github.com/pascalkuthe/imara-diff) | `imara-diff` | master | 2026-06-21 | 0.2.0, 2025-06-14 |
| [pest-parser/pest](https://github.com/pest-parser/pest) | `pest` | master | 2026-11-13 | 2.9.6, 2026-11-10 |
| [petgraph/petgraph](https://github.com/petgraph/petgraph) | `petgraph` | master | 2026-10-31 | 0.8.3, 2025-09-30 |
| [rust-bakery/nom](https://github.com/rust-bakery/nom) | `nom` | main | 2025-08-26 | 8.0.0, 2025-01-26 |
| [rust-cli/anstyle](https://github.com/rust-cli/anstyle) | `anstream` | main | 2026-11-09 | 1.0.0, 2026-02-11 |
| [rust-vmm/seccompiler](https://github.com/rust-vmm/seccompiler) (**archived**) | `seccompiler` | main | 2026-07-27 | 0.5.0, 2025-03-07 |
| [serde-rs/json](https://github.com/serde-rs/json) | `serde_json` | master | 2026-10-16 | 1.0.154, 2026-10-11 |
| [smol-rs/polling](https://github.com/smol-rs/polling) | `polling` | master | 2026-10-12 | 3.11.0, 2025-09-14 |
| [tafia/quick-xml](https://github.com/tafia/quick-xml) | `quick-xml` | master | 2026-11-11 | 0.42.0, 2026-08-22 |
| [tokio-rs/tokio](https://github.com/tokio-rs/tokio) | `tokio` | master | 2026-11-10 | 1.54.1, 2026-10-21 |
| [toml-rs/toml](https://github.com/toml-rs/toml) | `toml`, `toml_edit` | main | 2026-11-06 | `toml` 1.2.1+spec-1.1.0 and `toml_edit` 0.25.19+spec-1.1.0, both 2026-11-06 |
| [winnow-rs/winnow](https://github.com/winnow-rs/winnow) | `winnow` | main | 2026-11-05 | 1.0.5, 2026-11-05 |
| [zesterer/chumsky](https://codeberg.org/zesterer/chumsky) (Codeberg) | `chumsky` | main | 2026-11-13 | 0.14.1, 2026-11-13 |
| [zkat/supports-color](https://github.com/zkat/supports-color) | `supports-color` | main | 2024-11-26 | 3.0.2, 2024-11-26 |

## Appendix B: Commands

Every command each slice ran, copied from its own appendix and grouped by slice. In these commands, `$SP` (or `$S`) is the session's scratch directory, and `$W` is the slice's work directory inside it. Neither is on this branch, so each script is named with its logic, and a short script that decides a level is inlined. `$R` is a checkout of purba at `2790c1e`. Three scratch files are named: `r2-needs.md` is the needs slice's output (§ 1 and § 5 of this file), `r2-measure-brief.md` is the brief the five measure slices shared, and `audit-960/` holds the working files of the #960 audit. crates.io was called with the User-Agent `purba-research (https://github.com/kalonji-tools/purba)`, at least 1 s apart, and no request carried an email address. No slice made a GitHub write, a commit or a push.

### B1. Needs slice (the needs and the candidates)

`cratesio.sh` runs `curl -sS -A "$UA" "https://crates.io/api/v1/crates/$c"` for each crate named, 1 s apart.

```sh
# the issue, the checkout and the landscape
cd $R && git log -1 --oneline && ls && gh issue view 961 --repo kalonji-tools/purba
cd $R && ls docs/decisions crates scripts && wc -l LANDSCAPE.md CONTEXT.md && grep -n "order of design" -A40 LANDSCAPE.md
cd $R && cat -n LANDSCAPE.md | sed -n 1,173p
cd $R && cat -n CONTEXT.md

# the manifests
cd $R && cat Cargo.toml; for f in crates/*/Cargo.toml; do sed -n '/dependencies/,$p' $f; done; head -40 scripts/check-prose/*; cat pyproject.toml

# open issues, filtered locally (no --search)
gh issue list --repo kalonji-tools/purba --state open --limit 500 --json number,title,body,labels > $S/open-issues.json
jq -r '.[] | select(.number==957 or .number==962 or .number==971 or .number==969) | ...' $S/open-issues.json
jq -r '.[] | select([.number]|inside([953,694,693,692,691])) | ...' $S/open-issues.json
jq -r '.[] | select([.number]|inside([683,669,662,648,644,637,634,623,130,967,968])) | ...' $S/open-issues.json
# the comments that #953's body links
for c in 6837748570 6861114993 6862174563 6865490125 6865928954; do gh api repos/kalonji-tools/purba/issues/comments/$c --jq '"######## \(.html_url)\n\(.body)"'; done > $S/strict-comments.md
grep -niE "crate|library|rust|fork|pipe|hash|serial|json|toml|diff|snapshot|mmap|copy-on|libc" $S/strict-comments.md

# the records
cd $R/docs/decisions && cat -n configuration-takes-the-first-location-its-tool-reads.md a-tester-runs-purba-on-one-module-and-reads-one-line-for-each-test.md each-test-runs-in-a-process-of-its-own.md
cd $R/docs/decisions && cat -n a-concern-is-what-no-part-can-own.md a-suppression-is-an-outcome-and-not-a-switch.md purba-is-always-strict.md a-run-exits-0-only-when-it-checked-something-and-every-test-passed.md
cd $R/docs/decisions && cat -n purba-divides-by-responsibility-and-not-by-language.md purba-parses-python-with-ruff.md parsing-hands-on-no-tree-that-ruff-recovered.md
cd $R/docs/decisions && sed -n 18,40p purbas-public-surface-starts-empty-and-has-no-plugin-api.md
cd $R/docs/decisions && sed -n '/## Decision Outcome/,/## Confirmation/p' the-skeleton-crosses-the-seam-in-execution-and-assertion.md purba-runs-on-linux-and-macos.md
cd $R && grep -rniE "toml|json|junit|xml|colou?r|terminal|tty|diff\b|glob|clap|argument|flag|serde|crate|library" docs/decisions/*.md | grep -viE "lint|gate|workflow|github|cargo-?script|record"
cd $R && grep -rnE "configuration" docs/decisions/*.md LANDSCAPE.md CONTEXT.md README.md CONTRIBUTING.md AGENTS.md
cd $R && grep -rnoE "[^.]*design in depth of (strictness|parsing|configuration|selection|...)[^.]*\." docs/decisions/*.md LANDSCAPE.md CONTEXT.md
cd $R && grep -rnE "design in depth|in its design|its design|designs in depth" docs/decisions/*.md
cd $R && grep -rnwE "glob|globs|pattern|patterns|directory|directories|discovery|query|platform|timeout|hangs|hung|parallel|JSON|JUnit|colour|color|diff|repr|cache|terminal|stderr|thread|threads" docs/decisions/*.md LANDSCAPE.md | grep -viE "workflow|\.github|lint|gate|bats|prek|commit message|scripts/|mise |reach:|wiki|runner|check-" > $S/grep2.txt
# each quoted line, by number
cd $R && sed -n "<n>p" <file>   # for every LANDSCAPE / CONTEXT / record line quoted in §1
jq -r '.[]|select(.number==N)|.body' $S/open-issues.json | grep -nE "encoding_rs|How parsing decodes|How macOS starts|awaits one|nodes, and the edges|C library|reading"   # N in 644 694 693 691 953 967
jq -r '.[]|select(.number==953)|.body' $S/open-issues.json | grep -o "From a test's code alone[^.]*\.\|Each assay except the last runs[^.]*\."

# purba's code today
cd $R && grep -rn "serde\|libc::" crates/*/src
cd $R && sed -n 20,40p crates/execution/src/lib.rs && sed -n 44,90p crates/execution/src/lib.rs
cd $R && git log --oneline -S"serde" -- Cargo.toml crates/execution/Cargo.toml
cd $R && grep -n '"compile"\|read_to_string\|argv' crates/*/src/*.rs; grep -n "LineIndex" crates/*/src/*.rs
cd $R && grep -n "fs::read\|from_utf8" crates/parsing/src/lib.rs; sed -n 215,240p crates/parsing/src/lib.rs; sed -n 285,310p crates/parsing/src/lib.rs

# the dependency graph
cd $R && cargo tree --offline -e normal,dev --prefix none | sed 's/ (\*)//' | sort -u > $S/tree.txt
for c in <each candidate>; do grep -E "^$c v" $S/tree.txt; grep -A1 "^name = \"$c\"$" $R/scripts/check-prose/Cargo.lock | grep version; done

# std and libc, from local sources
S=$(rustc --print sysroot)/lib/rustlib/src/rust/library/std/src
grep -n "pub fn absolute" $S/path.rs; grep -n "pub fn canonicalize" $S/fs.rs; grep -n "pub fn available_parallelism" $S/thread/*.rs
grep -rn "pub fn pair" $S/os/unix/net/stream.rs; grep -n "pub fn read_dir" $S/fs.rs; grep -n "pub const OS" $S/env.rs; grep -rn "pub fn pipe" $S/io/pipe.rs
grep -rn "trait IsTerminal" $S; sed -n 4135,4145p $S/path.rs; sed -n 75,83p $S/io/pipe.rs
grep -n -B2 "pub fn available_parallelism" $S/thread/functions.rs; grep -n -B3 "pub fn pair() " $S/os/unix/net/stream.rs
L=~/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/libc-0.2.189; for f in fork waitpid pipe dup2 poll kill posix_spawn setrlimit; do grep -rln "pub fn $f(" $L/src/unix; done

# the ecosystem manifests, at tags
gh api repos/nextest-rs/nextest/releases --jq '.[0:5][]|.tag_name'
gh api repos/astral-sh/ruff/releases/latest --jq .tag_name
gh api repos/astral-sh/uv/releases/latest --jq .tag_name
gh api 'repos/nextest-rs/nextest/contents/Cargo.toml?ref=cargo-nextest-0.9.152' -H 'Accept: application/vnd.github.raw' > $S/eco/nextest-Cargo.toml
gh api 'repos/nextest-rs/nextest/contents/nextest-runner/Cargo.toml?ref=cargo-nextest-0.9.152' -H 'Accept: application/vnd.github.raw' > $S/eco/nextest-runner-Cargo.toml
gh api 'repos/nextest-rs/nextest/contents/nextest-filtering/Cargo.toml?ref=cargo-nextest-0.9.152' -H 'Accept: application/vnd.github.raw' > $S/eco/nextest-filtering-Cargo.toml
gh api 'repos/astral-sh/ruff/contents/Cargo.toml?ref=0.17.5' -H 'Accept: application/vnd.github.raw' > $S/eco/ruff-Cargo.toml
gh api 'repos/astral-sh/uv/contents/Cargo.toml?ref=0.13.8' -H 'Accept: application/vnd.github.raw' > $S/eco/uv-Cargo.toml
grep -nE '^(winnow|nom|...|serde)[ =]' $S/eco/*.toml
grep -nE '^(encoding[a-z_-]*|pep508[a-z_-]*|...|insta)[ =]' $S/eco/*.toml
sed -n 80,96p $S/eco/nextest-runner-Cargo.toml; sed -n 123,134p $S/eco/nextest-runner-Cargo.toml
for c in <crate>; do grep -nE "^$c =" $S/eco/ruff-Cargo.toml; grep -nE "^$c =" $S/eco/uv-Cargo.toml; done

# crates.io (User-Agent: purba-research (https://github.com/kalonji-tools/purba)); $S/cratesio.sh runs
#   curl -sS -A "$UA" "https://crates.io/api/v1/crates/$c"   for each crate named, 1 s apart
bash $S/cratesio.sh encoding_rs encoding_rs_io encoding walkdir ignore globset winnow nom chumsky pest toml toml_edit pyproject-toml serde_json postcard bincode rmp-serde ciborium nix rustix polling os_pipe gag quick-junit quick-xml junit-report petgraph lexopt pico-args clap bpaf path-clean normpath pyo3-async-runtimes target-spec pep508_rs uv-pep508 similar imara-diff anstream owo-colors supports-color seccompiler
bash $S/cratesio.sh textcode difflib tokio
for q in "iso-8859" "codepage" "charset decode" "python codecs" "pep 263"; do curl -sS -A "$UA" -G https://crates.io/api/v1/crates --data-urlencode "q=$q" --data-urlencode per_page=8 --data-urlencode sort=relevance; done
for q in "argument parser" "parser combinator" "junit" "fork process unix" "path normalize" "toml" "capture stdout fd" "diff"; do curl -sS -A "$UA" -G https://crates.io/api/v1/crates --data-urlencode "q=$q" --data-urlencode per_page=8 --data-urlencode sort=downloads; done
curl -sS -A "$UA" https://crates.io/api/v1/crates/pyo3-async-runtimes/0.30.0/dependencies
curl -sS -A "$UA" https://crates.io/api/v1/crates/pyo3-async-runtimes/versions
curl -sS -A "$UA" https://crates.io/api/v1/crates/pyo3-async-runtimes/0.29.0/dependencies
curl -sS -A "$UA" -L https://crates.io/api/v1/crates/bincode/3.0.0/readme -o $S/bincode3-readme.html
curl -sS -A "$UA" https://crates.io/api/v1/crates/bincode/versions
```

`ciborium` was fetched in the first `cratesio.sh` call and then left out as padding: no ecosystem project or search surfaced it for D3. No GitHub write, commit or push was made. No request carried an email address.

### B2. Checks slice (advisory, licence, source, upkeep, maintainers, churn, owner)

1. Candidate list: `sed -n '60,127p' r2-needs.md | grep -oE '^\| `[^`]+` \| [^|]+ \|' …`, then the first 41 lines are kept, giving the crate and version of each candidate. Repositories come the same way.
2. crates.io: `fetch-cratesio.sh`. For each crate it runs `curl -sS -f -A 'purba-research (https://github.com/kalonji-tools/purba)'` on `/api/v1/crates/<c>`, `/api/v1/crates/<c>/owners` and every page of `/api/v1/crates/<c>/versions?per_page=100&sort=date`, with a 1.1 s sleep after each request.
3. Version analysis: `jq -c --arg c <c> --arg m3 2026-08-14 --arg m12 2025-11-14 -f analyse.jq <c>.versions.json`. It gives the newest stable, its date, licence, publisher, yank status, the first release of each compatibility key, and the counts in 3 and 12 months.
4. Owners and trust: a `jq` loop over `<c>.owners.json`, `<c>.crate.json` (`repository`, `trustpub_only`) and the newest version record (`repository`, `published_by.login`, `trustpub_data`). Downloads: `jq '.crate.downloads, .crate.recent_downloads'`. The crate record's keys come from `jq '.crate|keys' nix.crate.json`.
5. RustSec: `git clone --depth 1 https://github.com/rustsec/advisory-db.git` (HEAD `4ef2df8e806a65e2f1ecc14f56f51249062b23d6`, 2026-11-13). Then `ls advisory-db/crates/<c>` for each candidate; 7 have a directory. Each file's TOML front matter is printed with `awk`, and the `[versions]` ranges are read by hand against the newest version. `grep -rlE 'withdrawn|malicious'` over those 7 directories matches nothing.
6. OSV cross-check: `osv.sh` runs `POST https://api.osv.dev/v1/query` for each crate, with and without `version`.
7. Liveness: `liveness.sh` runs `gh api repos/<r>` (`default_branch`, `archived`, `full_name`) and `gh api 'repos/<r>/commits?sha=<branch>&per_page=1'`, plus `&path=crates/<c>` for ripgrep. For Codeberg it runs `curl https://codeberg.org/api/v1/repos/zesterer/chumsky` and `…/commits?sha=main&limit=1`.
8. Licence gate: `licence-probe.sh`. For each candidate it writes a probe `Cargo.toml` (`license = "MIT"`, dependency `"<c>" = "=<version>"`, default features), runs `cargo generate-lockfile`, then runs `cargo-deny 0.20.2 --manifest-path … --config probe-deny.toml check licenses`. Rejected crates are read from the `Cargo.toml` path in each `error[rejected]` block. `probe-deny.toml` is purba's `.cargo/deny.toml` with two lines changed, because a probe uses none of purba's exceptions:

   ```diff
   -unused-allowed-license = "deny"
   -unused-license-exception = "deny"
   +unused-allowed-license = "allow"
   +unused-license-exception = "allow"
   ```

9. Owner in purba: `cat $R/.github/dependabot.yml`, then `grep -rlnE 'package-ecosystem: *cargo|renovate' $R/{.github,.config,docs}`, which found nothing.
10. Check-prose rustix: `grep -A2 '^name = "rustix"' $R/scripts/check-prose/Cargo.lock $R/Cargo.lock`.

### B3. Review-a slice (upstream review and tests, 21 crates)

`measure.sh` classifies the last 30 commits of a branch by the #960 rule. It is inlined here, because every review level rests on it:

```bash
#!/usr/bin/env bash
# measure.sh OWNER/REPO BRANCH  -> writes OWNER_REPO.tsv (method of audit-960/audit-review-tests.md)
# For each of the last 30 commits on BRANCH: commits/SHA/pulls (merged_at != null, first hit),
# pulls/N (merged_by), pulls/N/reviews (paginated). REVIEWED = >=1 APPROVED from a login != PR author.
set -uo pipefail
repo=$1; br=$2
out=$(echo "$repo" | tr / _).tsv
: > "$out"
gh api "repos/$repo/commits?sha=$br&per_page=30" \
  --jq '.[] | [.sha, .commit.committer.date[0:10], (.author.login // ("git:" + .commit.author.name)), (.commit.message|split("\n")[0])] | @tsv' > "$out.commits"
while IFS=$'\t' read -r sha date author subj; do
  pr=$(gh api "repos/$repo/commits/$sha/pulls" --jq '[.[] | select(.merged_at != null)][0].number // empty' 2>/dev/null)
  if [ -z "$pr" ]; then
    printf '%s\t%s\t%s\t-\t-\t-\t-\tUNREVIEWED(no PR)\t%s\n' "${sha:0:10}" "$date" "$author" "$subj" >> "$out"
    continue
  fi
  meta=$(gh api "repos/$repo/pulls/$pr" --jq '[.user.login, (.merged_by.login // "none"), .base.ref] | @tsv')
  prauthor=$(cut -f1 <<<"$meta"); mergedby=$(cut -f2 <<<"$meta"); base=$(cut -f3 <<<"$meta")
  appr=$(gh api --paginate "repos/$repo/pulls/$pr/reviews" --jq ".[] | select(.state==\"APPROVED\" and .user.login != \"$prauthor\") | .user.login" | sort -u | paste -sd, -)
  others=$(gh api --paginate "repos/$repo/pulls/$pr/reviews" --jq ".[] | select(.state!=\"APPROVED\") | (.user.login + \":\" + .state)" | sort -u | paste -sd, -)
  if [ -n "$appr" ]; then v=REVIEWED; else v=UNREVIEWED; appr=none; fi
  printf '%s\t%s\t%s\t#%s(base=%s)\t%s\tmergedby=%s\tapprovers=%s\t%s\t%s | other reviews: %s\n' \
    "${sha:0:10}" "$date" "$author" "$pr" "$base" "$prauthor" "$mergedby" "$appr" "$v" "$subj" "${others:-none}" >> "$out"
done < "$out.commits"
echo "$repo: $(grep -c $'\tREVIEWED\t' "$out") reviewed / $(wc -l < "$out")"
```

`fp.sh` runs the same rule on `git log --first-parent -30` of a bare blobless clone. `chumsky-map.sh` maps Codeberg commits to PRs through each merge commit, and the PR's commits on `main` through `git rev-list M^1..M^2`, because Forgejo's `commits/{sha}/pull` resolves only a merge commit. `summarize.sh` counts the "merged by a login other than the PR author" reading with `awk` over the TSVs. `touch.sh` counts the commits of a window that touch a path prefix.

```sh
# repo metadata and default branches -> repometa.tsv
while read r; do gh api repos/$r --jq '[.full_name,.default_branch,.archived,.fork,...]|@tsv'; done < repos.txt
# review classification, per repo (method of audit-960): measure.sh OWNER/REPO BRANCH -> OWNER_REPO.tsv
#   gh api "repos/$repo/commits?sha=$br&per_page=30"
#   gh api "repos/$repo/commits/$sha/pulls"  --jq '[.[]|select(.merged_at!=null)][0].number'
#   gh api "repos/$repo/pulls/$pr"           (author, merged_by, base)
#   gh api --paginate "repos/$repo/pulls/$pr/reviews"  (APPROVED by login != PR author)
cut -f1,2 repometa.tsv | xargs -P 6 -L 1 bash measure.sh
(cd branch-0.2 && bash ../measure.sh lifthrasiir/rust-encoding 0.2)   # -> rust-encoding-0.2.branch
# bare blobless clones for first-parent / tags / workflow files
git clone -q --bare --filter=blob:none --single-branch --branch $b https://github.com/$r clones/$d.git
git clone -q --bare --filter=blob:none --single-branch --branch main https://codeberg.org/zesterer/chumsky clones/zesterer_chumsky.git
git -C clones/$d.git log -30 --format='%p' | awk 'NF>1' | wc -l          # merge commits among the 30
# first parent: fp.sh OWNER/REPO -> OWNER_REPO.fpout (git log --first-parent -30, same rule)
printf '%s\n' cesbo/textcode dylni/normpath jamesmunns/postcard lifthrasiir/rust-encoding rust-bakery/nom serde-rs/json smol-rs/polling tokio-rs/tokio toml-rs/toml winnow-rs/winnow | xargs -P 5 -L 1 bash fp.sh
# chumsky (Codeberg, Forgejo API, no auth)
curl -sS https://codeberg.org/api/v1/repos/zesterer/chumsky
curl -sS https://codeberg.org/api/v1/version
curl -sS 'https://codeberg.org/api/v1/repos/zesterer/chumsky/commits?sha=main&limit=30&stat=false&verification=false&files=false' > chumsky.commits.json
curl -sS "https://codeberg.org/api/v1/repos/zesterer/chumsky/commits/<sha>/pull"      # 404 for non-merge commits
curl -sS https://codeberg.org/api/v1/repos/zesterer/chumsky/pulls/1018/commits?limit=50 # pre-rebase SHAs
bash chumsky.sh        # calls chumsky-map.sh (PR via merge commit, members via git rev-list M^1..M^2) -> zesterer_chumsky.tsv
bash chumsky-fp.sh     # -> zesterer_chumsky.fpout
curl -sS https://codeberg.org/api/v1/repos/zesterer/chumsky/commits/41be9b6fb78947393591d445e952bc9fe75d0863/status
curl -sS "https://codeberg.org/api/v1/repos/zesterer/chumsky/actions/tasks?limit=50"
curl -sS https://codeberg.org/api/v1/repos/zesterer/chumsky/branches/main
# sensitivity + bot counts
bash summarize.sh
# release tags and ancestry -> tag-ancestry.txt
git ls-remote --tags <url>; git -C clones/$d.git fetch -q <url> "+$tag:$tag"; git -C clones/$d.git merge-base --is-ancestor "$tag^{commit}" HEAD
git ls-remote --heads https://github.com/lifthrasiir/rust-encoding
# tagless crates: .cargo_vcs_info.json from the published .crate (UA 'purba-research (https://github.com/kalonji-tools/purba)', 1.2 s apart), each in its own dl/<crate>/ dir
curl -sS -L -A "$UA" -o dl/$c/$c-$v.crate "https://static.crates.io/crates/$c/$c-$v.crate"; tar -xzOf dl/$c/$c-$v.crate "$c-$v/.cargo_vcs_info.json"
curl -sS -A "$UA" https://crates.io/api/v1/crates/encoding/0.2.33
# monorepo dilution -> touch.txt
bash touch.sh <clone> <path-prefix>
# rulesets and protection -> rules.txt, protected.txt
gh api repos/$r/rules/branches/$b ; gh api repos/$r/branches/$b --jq .protected
# tests: workflow files at HEAD (saved as wf_<repo>.yml), triggers + test lines
git -C clones/$d.git ls-tree --name-only HEAD .github/workflows/
git -C clones/$d.git show HEAD:.github/workflows/<file> > wf_$d.yml ; grep -nE 'cargo (test|nextest|hack)…' wf_$d.yml
git -C clones/jamesmunns_postcard.git show HEAD:ci.sh ; git -C clones/<d>.git show HEAD:Cargo.toml   # workspace members
git -C clones/<d>.git grep -c '#\[test\]' HEAD                                                       # textcode, rust-encoding, path-clean
# observed runs -> checkruns.txt, actionruns.txt
gh api "repos/$r/commits/$HEAD/check-runs?per_page=100" ; gh api "repos/$r/commits/$HEAD/status"
gh api "repos/$r/actions/runs?per_page=100"
gh api repos/$r/actions/workflows                              # nom, path-clean, rust-encoding
gh api "repos/rust-bakery/nom/actions/workflows/ci.yml/runs?per_page=5"
gh api "repos/rust-bakery/nom/actions/runs/35617278456/jobs?per_page=50"   # total_count 0
gh api repos/smol-rs/polling/actions/jobs/128678290339 ; gh api "repos/smol-rs/polling/actions/runs/38265947874/jobs?per_page=100"
gh api "repos/winnow-rs/winnow/commits/21b3ab2535/check-runs?per_page=100"
gh api repos/nix-rust/nix/actions/jobs/132264109687
```

### B4. Review-b slice (upstream review and tests, 20 crates)

`measure.sh` here applies the same rule as B3's, with a response cache and an optional list of SHAs, which `fp.sh` passes for first parent. `run-all.sh` calls it for each repository.

```sh
# crates.io (User-Agent "purba-research (https://github.com/kalonji-tools/purba)", 1 s apart, no email)
$W/cratesio.sh pyo3-async-runtimes gag os_pipe petgraph quick-junit quick-xml junit-report lexopt pico-args bpaf clap similar imara-diff difflib anstream owo-colors supports-color globset pep508_rs seccompiler
# repo identity, default branch, archived
while read c r; do gh api repos/$r --jq '"…\(.full_name)\t\(.default_branch)\tarchived=\(.archived)\tfork=\(.fork)"'; done < $W/repos.txt   # -> repo-meta.tsv
# review, raw 30 (measure.sh implements the audit-960 rule exactly)
$W/run-all.sh        # calls $W/measure.sh OWNER/REPO for each line of repos.txt -> tsv/*.tsv
#   measure.sh: gh api repos/R/commits?sha=B&per_page=30; gh api repos/R/commits/SHA/pulls (first merged_at!=null);
#               gh api repos/R/pulls/N (user, merged_by); gh api --paginate repos/R/pulls/N/reviews
# first-parent
cd $W/clones && git clone -q --bare --filter=blob:none https://github.com/$r $k.git     # each repo
git -C $k.git log -30 --format=%P HEAD | awk 'NF>1' | wc -l                                # merges in last 30
$W/fp.sh <repo>      # git log --first-parent -30 --format=%H HEAD -> measure.sh with that SHA list -> tsv/*.fpout
# sensitivity / bot counts: awk over tsv/*.tsv (merged_by != PR author && approvers none; bot = [bot], nextest-bot, Copilot)
gh api users/nextest-bot --jq .type; gh api users/Copilot      # User; 404
git -C nextest-rs_quick-junit.git log --format='%h %an | %s' -30 HEAD
# tests: workflow files read from the clone at HEAD
git -C $k.git ls-tree --name-only HEAD .github/workflows/ ; git -C $k.git show HEAD:<file> > $W/wf/<repo>__<file>
awk '/^on:/…' + grep -nE 'cargo (test|nextest|hack|miri)|make |just ' $W/wf/*   # -> $W/wf-summary.txt
git -C $k.git show HEAD:Makefile | grep -n 'test'           # clap, similar
git -C $k.git show HEAD:Cargo.toml | grep -n members         # ripgrep, anstyle, quick-junit
git -C $k.git grep -c '#\[test\]' HEAD -- tests src          # gag, difflib, pico-args, supports-color
git -C rust-vmm_seccompiler.git ls-tree HEAD rust-vmm-ci; git -C rust-vmm_seccompiler.git show HEAD:.buildkite/custom-tests.json
gh api "repos/rust-vmm/rust-vmm-ci/git/trees/84550804a414e08983ac2c9604b2604ec1466132?recursive=1"
gh api "repos/rust-vmm/rust-vmm-ci/contents/.buildkite/test_description.json?ref=84550804…" -H 'Accept: application/vnd.github.raw'
gh api "repos/rust-vmm/rust-vmm-ci/contents/README.md?ref=84550804…" -H 'Accept: application/vnd.github.raw'
# observed runs
gh api "repos/R/commits/<HEAD>/check-runs?per_page=100"      # each repo -> $W/runs/*.checkruns.json
gh api repos/R/actions/jobs/<id> --jq '.steps[]…'            # step conclusions for the cited jobs
gh api "repos/R/commits/<HEAD>/status"                        # gag, difflib, seccompiler, os_pipe, pico-args, supports-color
gh api "repos/R/actions/runs?per_page=5"                      # os_pipe, pico-args, supports-color, gag, difflib
gh api repos/oconnor663/os_pipe.rs/actions/runs/20493330400/jobs; gh api repos/oconnor663/os_pipe.rs/actions/runs/20493330400
# release source
git -C $k.git tag -l | grep <version>; git -C $k.git merge-base --is-ancestor "<tag>^{commit}" HEAD   # -> release-tags.tsv
git -C bachp_junit-report-rs.git log -5 'v0.9.1^{commit}'; git -C … branch --contains 'v0.9.1^{commit}'
# monorepo path filter
git -C BurntSushi_ripgrep.git log 59e318f5ac^..178cc027e2 -- crates/globset
git -C rust-cli_anstyle.git diff-tree --no-commit-id -r --name-only -m --first-parent <sha>   # each window SHA, written to a file, then grep
# rulesets
gh api "repos/R/rules/branches/<default>"                     # -> rulesets.tsv
```

### B5. Blast-a slice (blast area and `unsafe`, 21 crates)

`fetch.sh` downloads each `.crate` from `static.crates.io` into its own empty directory and extracts it; nothing in it is run. `tree.sh` makes one throwaway project per crate and runs `cargo tree`, as below. `walk.pl` reads the extracted source without running it. It walks the module tree from `src/lib.rs`, following `mod x;`, `#[path]`, `#[cfg_attr(…, path = …)]` and inline modules. It evaluates each `#[cfg]`, `#![cfg]` and `cfg_if!` gate for x86_64 Linux GNU with the features cargo resolved, plus tokio's gate macros and the custom cfgs each build script would set (read by hand, not run). It then counts the blocks by the #960 rule. `run-walk.sh` runs `walk.pl` for each crate, and `sens.sh` repeats the count with more features.

```sh
# 1. crates.io metadata and the .crate files, 1 s apart, UA "purba-research (https://github.com/kalonji-tools/purba)"
#    scripts/fetch.sh: curl -sS -A "$UA" https://crates.io/api/v1/crates/$c -o api/$c.json
#                      v = .crate.max_stable_version; yanked/created/licence of that version from .versions[]
#                      curl -sS -A "$UA" -fL https://static.crates.io/crates/$c/$c-$v.crate -o dl/$c-$v/$c-$v.crate
#                      tar -xzf … -C dl/$c-$v          (each crate in its own empty directory; nothing run in it)
scripts/fetch.sh nix rustix polling tokio serde_json postcard rmp-serde encoding_rs encoding textcode \
  toml toml_edit pyproject-toml walkdir ignore winnow nom chumsky pest path-clean normpath   # log: fetch.log

# 2. crates it brings (scripts/tree.sh), one throwaway project per crate under tree/p-<crate>
cargo new --lib --vcs none --name probe_<crate> tree/p-<crate>
cargo add '<crate>@=<version>'                       # default features
cargo tree -e normal       --prefix none --target x86_64-unknown-linux-gnu   # -> members-normal.txt
cargo tree -e normal,build --prefix none --target x86_64-unknown-linux-gnu   # -> members-normal-build.txt  (the second count column, for comparison with #960)
#   members = sed 's/ (\*)$//; s/ (proc-macro)//' | awk '{print $1" "$2}' | sort -u, minus probe_* and the crate itself
cargo metadata --format-version 1 --filter-platform x86_64-unknown-linux-gnu   # -> metadata.json (resolved features)

# 3. unsafe: read-only walker over the extracted source
perl scripts/gate-macros.pl dl/tokio-1.54.1/tokio-1.54.1/src/macros/cfg.rs > tokio-gate-macros.txt
scripts/run-walk.sh < versions.txt                   # runs scripts/walk.pl per crate -> walk/<crate>.txt; summary: walk-summary.txt
#   walk.pl --root <src root> --features <cargo-resolved> [--cfg …] [--false …] [--cfgkv …] [--macro NAME=PRED …] --list
#   per crate custom cfgs: see run-walk.sh (from each build.rs, read by hand)

# 4. sensitivity (scripts/sens.sh): tokio +process,signal,time; nix +signal -> walk/<crate>-sens.txt, tree/s-<crate>/

# 5. controls
grep -rhE '\bunsafe\s*\{' <root>/src --include='*.rs' | grep -vcE '^\s*//'          # equals walker block total (gaps explained above)
grep -rE '\bunsafe\b' --include='*.rs' <tarball root>                               # 0 for toml_edit pyproject-toml walkdir ignore pest
grep -rnE 'forbid\([^)]*unsafe_code|deny\([^)]*unsafe_code' --include='*.rs' <tarball root>

# 6. overlap with purba's lock (context only)
awk '/^name = /{n=$3} /^version = /{…print n" v"v}' research-961/Cargo.lock | sort -u > purba-lock.txt   # research-961 at 2790c1e
comm -12 tree/p-<crate>/members-normal.txt purba-lock.txt

# 7. hand checks: sed -n '<line-4>,<line+1>p' on each example block named in the tables
```

### B6. Blast-b slice (blast area and `unsafe`, 20 crates)

`blast.sh` makes one throwaway project per crate and runs both `cargo tree` counts. `fetch.sh` downloads each `.crate` into its own empty directory and records its sha256. `count-unsafe.pl` blanks comments, strings and char literals (keeping line numbers), drops `#[cfg(test)]` items, counts `unsafe {` blocks, `unsafe fn`, `unsafe impl`, `unsafe trait` and `unsafe extern`, and marks a block as having SAFETY when `/safety/i` is on the 3 lines above it, on its own line, or on the next line. Its control file holds `unsafe` in each kind of comment and string, a `#[cfg(test)]` module and 3 real blocks; the counter found exactly the 3. The `cfg` filtering inside each crate with a non-zero count was done by hand. `addcol.pl` only inserted columns into the slice's table.

```sh
# inputs read
cat $SP/r2-measure-brief.md; cat $SP/audit-960/audit-blast-unsafe.md; sed -n '9,127p' $SP/r2-needs.md
jq -r '... max_stable_version, newest_version, versions ...' $SP/cratesio/<c>.json      # [B] cache from the needs phase
cargo --version                                   # cargo 1.99.0 (5f94df478 2026-08-27)
cd $SP/research-961 && mise exec -- cargo --version   # cargo 1.102.0-nightly (c07ac2c4d 2026-11-06)

# blast: $W/bin/blast.sh  (per crate: cargo new --lib --vcs none; cargo add <spec>; sleep 1;
#   cargo tree -e normal --prefix none --target x86_64-unknown-linux-gnu;
#   cargo tree -e normal,build ... ; awk '{print $1" "$2}' | sort -u | drop root and crate)
$W/bin/blast.sh gag os_pipe petgraph quick-junit quick-xml junit-report lexopt pico-args bpaf clap \
  similar imara-diff difflib anstream owo-colors supports-color globset pep508_rs seccompiler \
  pyo3-async-runtimes@0.29 pyo3-async-runtimes@0.30
# outputs: $W/trees/<tag>.tree.txt, .set.txt, .tree-nb.txt, .set-nb.txt, .add.txt

# pyo3 pinned as purba pins it
cargo new --lib --vcs none --name probe_pyo3ar_pinned $W/proj/probe-pyo3ar-pinned
cd $W/proj/probe-pyo3ar-pinned && cargo add pyo3@=0.29.2 pyo3-async-runtimes@0.29
cargo tree -e normal --prefix none --target x86_64-unknown-linux-gnu -p pyo3-async-runtimes
cargo tree -e normal,build --prefix none --target x86_64-unknown-linux-gnu -p pyo3-async-runtimes   # -> 21

# normal + build count, the #960 audit's edge set (blast.sh writes it for every probe):
#   cd $W/proj/probe-<tag> && cargo tree -e normal,build --prefix none --target x86_64-unknown-linux-gnu > $W/trees/<tag>.tree-nb.txt
#   awk '{print $1" "$2}' $W/trees/<tag>.tree-nb.txt | sort -u | grep -v '^probe_' | grep -v "^<crate> <ver>$" > $W/trees/<tag>.set-nb.txt
for f in $W/trees/*.set-nb.txt; do t=$(basename $f .set-nb.txt)
  echo "$t normal=$(wc -l < $W/trees/$t.set.txt) nb=$(wc -l < $f) extra=$(comm -13 $W/trees/$t.set.txt $f | paste -sd, -)"; done
# the summary-table columns were inserted with $W/bin/addcol.pl

# names already in purba's lock
grep '^name = ' $SP/research-961/Cargo.lock | sed 's/name = "\(.*\)"/\1/' | sort -u > $W/purba-lock-names.txt
awk '{print $1}' $W/trees/<tag>.set.txt | sort -u | comm -23 - $W/purba-lock-names.txt

# toolchain control: copy six probe manifests to $W/proj-nightly/<t>, then from $SP/research-961:
mise exec -- cargo generate-lockfile --manifest-path $W/proj-nightly/<t>/Cargo.toml -q
mise exec -- cargo tree --manifest-path $W/proj-nightly/<t>/Cargo.toml -e normal --prefix none --target x86_64-unknown-linux-gnu
git -C $SP/research-961 status --short            # empty

# unsafe: download, each into its own empty dir, never executed
$W/bin/fetch.sh < $W/bin/list.txt
#   curl -sSfL -A 'purba-research (https://github.com/kalonji-tools/purba)' \
#     https://static.crates.io/crates/<c>/<c>-<v>.crate -o $W/src/<c>-<v>/<c>-<v>.crate
#   sha256sum >> $W/src/SHA256SUMS; tar -xzf ... -C $W/src/<c>-<v>; sleep 1
# checksum control against the probe locks' `checksum = ` lines -> 22/22 match
cat $W/proj/*/Cargo.lock | awk '...name/version/checksum...' | sort -u > $W/lock-checksums.txt

# counter control
LIST=1 perl $W/bin/count-unsafe.pl $W/control/c.rs
# per crate (cwd $W/bin, files passed as arguments)
find $W/src/<c>-<v>/<c>-<v>/src -name '*.rs' -print0 | sort -z | [LIST=1] xargs -0 perl $W/bin/count-unsafe.pl
# raw control
grep -rhoE '\bunsafe\s*\{' $W/src/<c>-<v>/<c>-<v>/src | wc -l
grep -rnE '\bunsafe\s*\{' quick-xml-0.42.0/*/src clap_builder-4.6.7/*/src seccompiler-0.5.0/*/src
# lints, editions, default features, cfg gates, hand reads
grep -n -A12 '^\[lints' */*/Cargo.toml | grep -iE 'lints|unsafe'
grep -n edition */*/Cargo.toml
sed -n '/^\[features\]/,/^\[/p' <crate>/Cargo.toml
grep -nE 'cfg|mod ' <crate>/src/lib.rs
sed -n '<range>p' os_pipe .../src/unix.rs (1-110), src/lib.rs (125-133)
sed -n petgraph src/graph_impl/mod.rs (30-50, 482-494, 1308-1320), stable_graph/mod.rs (1000-1014),
       matrix_graph.rs (1055-1066), unionfind.rs (36-50, 104-146, 215-225)
sed -n imara-diff src/myers/middle_snake.rs (15-60), src/myers.rs (35-105), src/sources.rs (55-66)
sed -n anstream src/adapter/strip.rs (120-160)
sed -n seccompiler src/lib.rs (335-368), src/backend/condition.rs (299-303)
sed -n '<line>p' <crate>/src/lib.rs     # each forbid line
```

### B7. This file: pain

Every pain in § 2 and § 3 comes from this script. The first block holds the primary levels of § 2. The second holds the alternative readings of § 3, one changed input per row; the tag after the crate's name says which. Run it with `bash pain.sh`.

```bash
#!/usr/bin/env bash
# pain = (blast + mean of the seven risks) / 2, printed to 2 decimals
# columns: crate version blast upkeep maintainers review tests unsafe churn owner
pain() {
  awk 'NF && $1 !~ /^#/ {
    risks = $4 + $5 + $6 + $7 + $8 + $9 + $10
    printf "%-32s %-20s blast %d  risks %2d/7  pain %.2f\n", $1, $2, $3, risks, ($3 + risks / 7) / 2
  }'
}
echo "## primary readings"
pain <<'LEVELS'
# crate              version              bl up mt rv ts us ch ow
anstream             1.0.0                1  1  1  3  1  3  2  3
bpaf                 0.9.29               1  1  3  3  1  1  2  3
chumsky              0.14.1               2  1  1  3  1  2  3  3
clap                 4.6.7                1  1  1  3  1  1  1  3
difflib              0.4.0                2  3  3  3  2  1  2  3
encoding             0.2.33               1  3  3  3  2  3  2  3
encoding_rs          0.8.44               1  1  3  3  1  3  2  3
gag                  1.0.0                2  3  3  3  2  1  1  3
globset              0.4.21               1  1  3  3  1  1  2  3
ignore               0.4.34               2  1  3  3  1  1  2  3
imara-diff           0.2.0                2  2  3  2  1  3  2  3
junit-report         0.9.1                2  1  3  3  1  1  2  3
lexopt               0.3.2                1  1  3  3  1  1  2  3
nix                  0.31.3               2  1  1  2  1  3  2  3
nom                  8.0.0                1  2  3  2  2  3  1  3
normpath             1.5.2                1  1  3  3  1  2  1  3
os_pipe              1.2.3                1  2  3  3  1  3  1  3
owo-colors           4.4.0                1  1  2  3  1  1  1  3
path-clean           1.0.1                1  3  3  3  2  1  1  3
pep508_rs            0.9.2                3  2  3  3  1  1  2  3
pest                 2.9.6                2  1  1  2  1  1  1  3
petgraph             0.8.3                1  2  1  3  1  3  2  3
pico-args            0.5.0                1  3  3  3  2  1  2  3
polling              3.11.0               1  2  1  2  1  3  1  3
postcard             1.1.3                2  2  3  2  1  2  1  3
pyo3-async-runtimes  0.30.0               2  1  3  1  1  1  3  3
pyproject-toml       0.13.7               3  2  1  3  1  1  2  3
quick-junit          0.8.0                2  1  2  3  1  1  3  3
quick-xml            0.42.0               1  1  1  2  1  1  3  3
rmp-serde            1.3.1                2  1  2  3  1  1  1  3
rustix               1.1.6                2  1  3  3  1  3  1  3
seccompiler          0.5.0                1  2  1  1  1  2  2  3
serde_json           1.0.154              2  1  1  2  1  3  1  3
similar              3.2.1                2  1  3  3  1  1  2  3
supports-color       3.0.2                1  2  3  3  2  1  1  3
textcode             0.5.1                1  1  3  3  2  3  3  3
tokio                1.54.1               1  1  1  1  1  3  1  3
toml                 1.2.1+spec-1.1.0     1  1  1  3  1  1  2  3
toml_edit            0.25.19+spec-1.1.0   1  1  1  3  1  1  2  3
walkdir              2.5.0                1  3  3  3  1  1  1  3
winnow               1.0.5                1  1  3  3  1  3  2  3
LEVELS
echo "## alternative readings (one input changed)"
pain <<'ALT'
# alternative readings: one input changed per row; the tag after the crate names it
# crate                         version   bl up mt rv ts us ch ow
pyo3-async-runtimes:fp-review    0.30.0    2  1  3  2  1  1  3  3
serde_json:fp-review             1.0.154   2  1  1  3  1  3  1  3
postcard:fp-review               1.1.3     2  2  3  3  1  2  1  3
rustix:merge-review              1.1.6     2  1  3  2  1  3  1  3
ignore:merge-review              0.4.34    2  1  3  2  1  1  2  3
chumsky:merge-review             0.14.1    2  1  1  1  1  2  3  3
pyproject-toml:merge-review      0.13.7    3  2  1  2  1  1  2  3
encoding:merge-review-master     0.2.33    1  3  3  2  2  3  2  3
quick-xml:merge-review           0.42.0    1  1  1  1  1  1  3  3
imara-diff:merge-review          0.2.0     2  2  3  1  1  3  2  3
junit-report:merge-review        0.9.1     2  1  3  2  1  1  2  3
difflib:merge-review             0.4.0     2  3  3  2  2  1  2  3
anstream:merge-review            1.0.0     1  1  1  2  1  3  2  3
globset:merge-review             0.4.21    1  1  3  2  1  1  2  3
chumsky:normal-blast             0.14.1    1  1  1  3  1  2  3  3
pest:normal-blast                2.9.6     1  1  1  2  1  1  1  3
pico-args:workflow-tests         0.5.0     1  3  3  3  1  1  2  3
supports-color:workflow-tests    3.0.2     1  2  3  3  1  1  1  3
nom:workflow-tests               8.0.0     1  2  3  2  1  3  1  3
pyo3-async-runtimes:org-maint    0.30.0    2  1  1  1  1  1  3  3
quick-junit:bot-not-person       0.8.0     2  1  3  3  1  1  3  3
textcode:safe-comment            0.5.1     1  1  3  3  2  2  3  3
ALT
```

### B8. The red-team pass

A separate agent, which wrote none of this file, checked it before publication. It re-read 57 quotes against their sources, re-measured all eight levels of `tokio`, `clap` and `pep508_rs` and single inputs of eight more crates, recomputed every pain, and fetched every link. No level and no pain changed. The fixes it led to: `tokio`'s `unsafe` counts and test-run example, `clap`'s test-run link, the D1 quote's `…`, one `serde_json` comment, the `repository` field of `ignore` and `globset`, the edge set and file choice of the #960 method, a crossing stated as known for `serde_json`, and the Summary's count of unmet needs.

```bash
# tokio's compiled unsafe, from rustc itself (tokio and pin-project-lite have build = false and no proc-macro, so nothing downloaded runs)
cargo check -v                       # in a probe crate that depends on tokio = "=1.54.1"; copy tokio's rustc line from the output
RUSTC_BOOTSTRAP=1 <tokio's rustc line> -Zunpretty=expanded > tokio-expanded.rs
grep -c 'unsafe {' tokio-expanded.rs # 30: 28 from tokio's source, 2 from std's thread_local!
grep -n 'mod park' tokio-expanded.rs # present

# the test runs cited for tokio and clap
gh api --paginate "repos/tokio-rs/tokio/commits/ab0814fc8c/check-runs?per_page=100" --jq '.check_runs[]|[.name,.conclusion,.html_url]|@tsv'
gh api "repos/clap-rs/clap/actions/runs/41509788542" --jq '[.path,.event,.head_sha,.conclusion]|@tsv'
gh api "repos/clap-rs/clap/actions/runs/41509788542/jobs?per_page=100" --jq '.jobs[]|select(.name=="Test (linux)")|[.id,.conclusion]|@tsv'

# every crate at unsafe level 1 has no unsafe token in src/ (control first: os_pipe gives 16 lines)
for src in <each extracted <crate>-<version>/src>; do
  grep -rnw --include='*.rs' unsafe "$src" | grep -v 'unsafe_code\|unsafe_op' | grep -vE ':[0-9]+:\s*//' | wc -l
done
```
