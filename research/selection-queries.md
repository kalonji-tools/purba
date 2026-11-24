# Which ways of selecting tests runners offer, and what a query language answers that a tag or an id does not

Research for [Which ways of selecting tests do test runners offer, and what does a query language answer that a tag or an id does not?](https://github.com/kalonji-tools/purba/issues/1226)
Measured on 2026-11-24, against purba `main` at `3093871b`, on one Linux machine.
This file decides nothing. It lists the ways to select tests, what each can express, what each language cost, and which questions need a language.

## Summary

Every runner selects by a path or an id, and by a pattern on the test's name. Four runners also take a boolean language on the command line: pytest (`-k`, `-m`), cargo-nextest (`-E`), Vitest (`--tags-filter`) and oxitest (`-E`). JUnit has one too. cargo test, `go test`, Jest and unittest have none. But a flag pair of "include" and "skip" answers the issue's own example without a language. `cargo test slow --skip db` and `go test -run Slow -skip DB` each select the slow tests that are not database tests.

Five findings matter most for purba's design of selection.

- **Flags answer a union, and "A but not B". A language is needed for an intersection inside one kind, a union across kinds, and nesting.** "Slow and db" needs either a language or a flag that repeats with AND. Of these runners, only Vitest's `--tags-filter` repeats with AND. "Slow, or anything under `store::`" mixes a tag and an id, and only a language joins them with OR. "Slow or db, but not both" needs nesting. Section 4 lists the questions and the flags that answer each one.
- **A repeated flag combines picks in a different way in each runner.** Repeated `-k` in unittest, repeated filters in cargo test and repeated `-E` in cargo-nextest make a union. Repeated `--tags-filter` in Vitest makes an intersection. In pytest (`-k`, `-m`) and `go test` (`-run`, `-skip`), the last flag wins. Two `-t` flags in Jest selected no test and exited 0. Each of these is [run].
- **The same expression selects different tests in different languages.** `db | slow & !db` selects 3 tests in pytest, cargo-nextest and Vitest, where AND binds tighter than OR. oxitest's parser gives `&` and `|` the same rank and reads from left to right, so it selects 1 test [run]. No test in oxitest checks this case.
- **Each language drew fixes to its parser, and deep nesting breaks each of the four I probed.** pytest gave up Python's `eval()` for its own grammar in 6.0. Go changed the precedence of `|` in `-run`, and its parser for `//go:build` lines drew CVE-2024-34158 for stack exhaustion. cargo-nextest fixed `not(` with no space in 0.9.137. With deep parentheses, pytest fails with an internal error at 500 levels. Vitest throws a `RangeError`, and cargo-nextest and oxitest's parser overflow the stack [run].
- **oxitest's language is 649 production lines, and its records drifted from it.** oxitest first had a second language: `-m`, 171 production lines, which it removed for `-E`. Its how-to says `name()` matches the function name, but the run matches the whole node id, file path included [run]. Its help still lists `shared()`, which a fix removed. Its exit codes give 0 when `-E` selects nothing. The grilling agreed that purba refuses this case as an invocation error.

## How to read this file

- **[run]** means I ran it in this session, on the version named. **[source]** means the runner's code or documentation says so, at the pinned version, and I did not run it. **judgement** marks a classification that no source states.
- A **pick** is the set of tests that one predicate chooses: one tag, one id or prefix of an id, or one name pattern.
- The four **combinations** are:
  - **∪ union**: the tests in pick A or in pick B,
  - **∩ intersection**: the tests in both A and B,
  - **− exclusion**: the tests in A and not in B,
  - **( ) nesting**: a combination inside another, such as (A ∪ B) − (A ∩ B).
- A **language** is a grammar that takes at least two of the operators and, or and not, and parentheses (judgement). A regular expression is a language of characters, not of picks. Section 1 marks where a regular expression still gives a combination.
- In the tables, **yes** means the way can express the combination in one argument or one flag set. **flags** means it needs two kinds of flag together. **no** means it cannot.
- **The probe suite.** Each probe runs the same tests, in the form of its runner:

  | test | tags | file or module |
  |---|---|---|
  | add | none | math |
  | add slow | slow | math |
  | parse, cases a and b (pytest, oxitest, and subtests in `go test`) | none | math |
  | query db | db | store |
  | migrate db (slow) | slow, db | store |

  A runner with no tag carries "slow" and "db" in the test's name. So in cargo test, `go test`, Jest and unittest, a tag pick is a name pick.

## Method

### Versions

| runner | version | ran on | pinned source |
|---|---|---|---|
| pytest | 9.2.0, the newest on PyPI | CPython 3.13.13, a `uv` venv, `-I` | [`src/_pytest/mark/` at `9.2.0`](https://github.com/pytest-dev/pytest/tree/9.2.0/src/_pytest/mark) |
| unittest | CPython 3.13.13 | the same venv, `-I` | [`Lib/unittest/` at `v3.13.13`](https://github.com/python/cpython/tree/v3.13.13/Lib/unittest) |
| cargo test (libtest) | rustc 1.99.0, commit `b940084d` | this machine | [`library/test/src/lib.rs` at `b940084d`](https://github.com/rust-lang/rust/blob/b940084d7eb6a299eb4bfeb8e34901bc051e7ac4/library/test/src/lib.rs#L538-L580) |
| cargo-nextest | 0.9.149, the installed one | this machine | [`nextest-filtering/` at `cargo-nextest-0.9.149`](https://github.com/nextest-rs/nextest/tree/cargo-nextest-0.9.149/nextest-filtering) |
| `go test` | go 1.26.10, from nixpkgs | `nix-shell -p go` | [`src/cmd/go/alldocs.go`](https://github.com/golang/go/blob/go1.26.10/src/cmd/go/alldocs.go#L3400-L3430), [`src/testing/match.go`](https://github.com/golang/go/blob/go1.26.10/src/testing/match.go) at `go1.26.10` |
| Jest | 30.5.2, the newest on npm | Node 24.22.1 | [`docs/CLI.md` at `v30.5.2`](https://github.com/jestjs/jest/blob/v30.5.2/docs/CLI.md#L506-L528) |
| Vitest | 5.1.1, the newest on npm | Node 24.22.1 | [`tags.ts`](https://github.com/vitest-dev/vitest/blob/v5.1.1/packages/vitest/src/runtime/runner/utils/tags.ts), [`test-tags.md`](https://github.com/vitest-dev/vitest/blob/v5.1.1/docs/guide/test-tags.md) at `v5.1.1` |
| oxitest | `731c2b3f`, archived | its parser and evaluator, built alone (see below) | [`src/query/` at `731c2b3f`](https://github.com/kalonji-tools/oxitest/tree/731c2b3f/src/query) |
| JUnit Platform | 6.1.3, source only | not run | [`tags.adoc` at `r6.1.3`](https://github.com/junit-team/junit-framework/blob/r6.1.3/documentation/modules/ROOT/pages/running-tests/tags.adoc#L30-L47) |
| Bazel | 9.3.3, source only | not run | [`user-manual.mdx` at `9.3.3`](https://github.com/bazelbuild/bazel/blob/9.3.3/docs/docs/user-manual.mdx#L975-L988) |

JUnit and Bazel are outside the issue's list. I add them because each shows a flag design: JUnit has both repeated flags and a language, and Bazel has flags only.

### Probes

Each probe is a script under `research/selection-queries/<runner>/run.sh`, beside its tests. Each script copies its tests into an empty directory that it is given, and runs there. So no file of purba's tree reaches the run. `output.txt` beside each script is its output from this session.

```sh
R=research/selection-queries
bash $R/pytest/run.sh <python with pytest 9.2.0> <empty dir>
bash $R/unittest/run.sh <python 3.13> <empty dir>
bash $R/nextest/run.sh <empty dir>                      # cargo-nextest and cargo test
nix-shell -p go --run "bash $R/go/run.sh <empty dir>"
nix-shell -p nodejs --run "bash $R/jest/run.sh <dir with node_modules/jest@30.5.2> <empty dir>"
nix-shell -p nodejs --run "bash $R/vitest/run.sh <dir with node_modules/vitest@5.1.1> <empty dir>"
bash $R/oxitest/run.sh <oxitest checkout> <empty dir>
```

**oxitest's probe is not the oxitest binary.** The script copies `ast.rs`, `compile.rs`, `eval.rs` and `resource.rs` from `731c2b3f`, unchanged, and builds them with `main.rs`. `main.rs` builds one entry per test with the fields that oxitest's run path builds ([`transitions/mod.rs` L85-L91](https://github.com/kalonji-tools/oxitest/blob/731c2b3f/src/pipeline/transitions/mod.rs#L85-L91)). It runs oxitest's own parser and evaluator. It does not run oxitest's collection or its exit codes. A [run] mark on oxitest means this harness.

### What I counted

- **Size [run].** `research/selection-queries/sizes.sh` prints one row for each file of a language, at the pinned tag, into `sizes.tsv`. Every line counts, blank lines and comments too. In a Rust file, a production line comes before the first line `#[cfg(test)]`. A file of tests, or a file built only for tests, is all test lines.
- **History [run].** `research/selection-queries/history.sh` prints the commits on each language's files and the fixes in each changelog. `research/selection-queries/oxitest-history.sh` does the same for oxitest, with its issues. Each script prints its matcher beside its count. A count answers only that matcher.
- **Issues.** I listed issues with `gh issue list --state all`, and filtered them locally by a regular expression on the title or the body. I did not use GitHub's search.

## 1. The ways each runner selects tests

Each table lists every way to select tests that the runner's help or its selection documentation names. I leave out ways that select by the history of a run, such as `--lf` or `--changed`, and ways that split a run, such as `--shard`. They do not answer which tests a query names.

### 1.1 pytest 9.2.0

| way | grammar | example | ∪ | ∩ | − | ( ) | source |
|---|---|---|---|---|---|---|---|
| path or node id, positional | `path[::Class]::test[[case]]`, repeatable | `pytest test_math.py::test_parse[a]` | yes, repeated | no | no | no | [run]; [`usage.rst` L17-L76](https://github.com/pytest-dev/pytest/blob/9.2.0/doc/en/how-to/usage.rst#L17-L76) |
| `-k EXPR` | `or`, `and`, `not`, `( )` over case-insensitive substrings of the names of the test, its parents, its markers and its function's attributes | `-k "add and not slow"` | yes | yes | yes | yes | [run]; [`expression.py` L3-L13](https://github.com/pytest-dev/pytest/blob/9.2.0/src/_pytest/mark/expression.py#L3-L13), [`KeywordMatcher` L164-L225](https://github.com/pytest-dev/pytest/blob/9.2.0/src/_pytest/mark/__init__.py#L164-L225) |
| `-m EXPR` | the same grammar over marker names, plus `name(key=value)` | `-m "slow and not db"` | yes | yes | yes | yes | [run]; [`expression.py` L3-L13](https://github.com/pytest-dev/pytest/blob/9.2.0/src/_pytest/mark/expression.py#L3-L13) |
| `--deselect ID` | a node id or its prefix, repeatable | `--deselect test_store.py::test_migrate_db` | no | no | yes | no | [run]; [`main.py` L178](https://github.com/pytest-dev/pytest/blob/9.2.0/src/_pytest/main.py#L178) |
| `--ignore PATH`, `--ignore-glob GLOB` | a path or a glob, repeatable | `--ignore test_store.py` | no | no | yes | no | [run]; [`main.py` L166-L177](https://github.com/pytest-dev/pytest/blob/9.2.0/src/_pytest/main.py#L166-L177) |

A path, `-k`, `-m` and `--deselect` combine with AND [run]. `-m slow -k 'not migrate'` selects one test. `test_store.py -m slow` selects one test.

### 1.2 unittest, CPython 3.13.13

| way | grammar | example | ∪ | ∩ | − | ( ) | source |
|---|---|---|---|---|---|---|---|
| dotted name, positional | `module[.Class[.method]]`, repeatable | `test_math.Math.test_add` | yes, repeated | no | no | no | [run] |
| `-k PATTERN` | an `fnmatch` pattern on `module.Class.method`; a pattern with no `*` becomes `*PATTERN*`; repeatable | `-k slow -k db` | yes, repeated | no | no | no | [run]; [`main.py` L50-L56](https://github.com/python/cpython/blob/v3.13.13/Lib/unittest/main.py#L50-L56), [`loader.py` L222-L223](https://github.com/python/cpython/blob/v3.13.13/Lib/unittest/loader.py#L222-L223) |
| `discover -p PATTERN` | a glob on the file name | `-p "test_*.py"` | no | no | no | no | [source]; [`main.py`](https://github.com/python/cpython/blob/v3.13.13/Lib/unittest/main.py) |

unittest has no way to exclude a test.

### 1.3 cargo test (libtest), rustc 1.99.0

| way | grammar | example | ∪ | ∩ | − | ( ) | source |
|---|---|---|---|---|---|---|---|
| filter, positional after `--` | a substring of `module::test`, repeatable | `-- slow db` | yes, repeated | no | no | no | [run]; [`lib.rs` L554-L567](https://github.com/rust-lang/rust/blob/b940084d7eb6a299eb4bfeb8e34901bc051e7ac4/library/test/src/lib.rs#L554-L567) |
| `--skip` | a substring, repeatable | `-- slow --skip db` | no | no | yes | no | [run]; [`lib.rs` L569-L580](https://github.com/rust-lang/rust/blob/b940084d7eb6a299eb4bfeb8e34901bc051e7ac4/library/test/src/lib.rs#L569-L580) |
| `--exact` | turns every filter and skip into an exact match | `-- --exact math::add` | no | no | no | no | [run] |
| `--ignored`, `--include-ignored` | the `#[ignore]` attribute in code | `-- --ignored` | no | no | no | no | [source] |

The filters form a union, and the skips remove a union. So `slow --skip db` selects `math::add_slow` only [run].

### 1.4 cargo-nextest 0.9.149

| way | grammar | example | ∪ | ∩ | − | ( ) | source |
|---|---|---|---|---|---|---|---|
| filter, positional | a substring of the test name, repeatable | `cargo nextest run slow db` | yes, repeated | no | no | no | [run] |
| `-- --skip`, `-- --exact` | as in libtest | `-- --skip db` | no | no | yes | no | [run]; [`selecting.md` L30-L78](https://github.com/nextest-rs/nextest/blob/cargo-nextest-0.9.149/site/src/docs/selecting.md#L30-L78) |
| `-E FILTERSET` | predicates `test()`, `package()`, `deps()`, `rdeps()`, `kind()`, `binary()`, `binary_id()`, `platform()`, `group()`, `all()`, `none()`, `default()`; operators `&`, `\|`, `+`, `-`, `not`, `( )`; matchers `=`, `~`, `/regex/`, `#glob`; repeatable | `-E 'test(slow) - test(db)'` | yes | yes | yes | yes | [run]; [`reference.md` L3-L142](https://github.com/nextest-rs/nextest/blob/cargo-nextest-0.9.149/nextest-filtering/reference.md#L3-L142) |
| `default-filter` | a filterset in the project's file | `default-filter = "not test(slow)"` | yes | yes | yes | yes | [source]; [`selecting.md`](https://github.com/nextest-rs/nextest/blob/cargo-nextest-0.9.149/site/src/docs/selecting.md) |

Several `-E` flags form a union [run], as [`index.md` L25](https://github.com/nextest-rs/nextest/blob/cargo-nextest-0.9.149/site/src/docs/filtersets/index.md#L25) says. A filterset and a positional filter combine with AND [run], as [`index.md` L37-L45](https://github.com/nextest-rs/nextest/blob/cargo-nextest-0.9.149/site/src/docs/filtersets/index.md#L37-L45) says. nextest's documentation translates each `--skip` and `--exact` form into a filterset ([`selecting.md` L70-L78](https://github.com/nextest-rs/nextest/blob/cargo-nextest-0.9.149/site/src/docs/selecting.md#L70-L78)). The same filterset names tests in the project's file, for per-test overrides [source].

### 1.5 `go test`, go 1.26.10

| way | grammar | example | ∪ | ∩ | − | ( ) | source |
|---|---|---|---|---|---|---|---|
| package, positional | an import path or `./...` | `go test ./store/...` | yes, repeated | no | no | no | [source] |
| `-run REGEXP` | an RE2 expression per level of the test's name, split at `/`; not anchored; one value | `-run 'Slow\|DB'` | yes, by `\|` | only by listing both orders | no | no | [run]; [`alldocs.go` L3400-L3409](https://github.com/golang/go/blob/go1.26.10/src/cmd/go/alldocs.go#L3400-L3409) |
| `-skip REGEXP` | the same grammar; one value | `-run Slow -skip DB` | no | no | yes | no | [run]; [`alldocs.go` L3424-L3430](https://github.com/golang/go/blob/go1.26.10/src/cmd/go/alldocs.go#L3424-L3430) |
| `-tags LIST` with `//go:build EXPR` | the file carries `&&`, `\|\|`, `!`, `( )` over build tags; the command gives the tag list | `//go:build slow && !nodb`, then `-tags slow` | yes | yes | yes | yes, in the file | [run]; [`alldocs.go` L2085-L2093](https://github.com/golang/go/blob/go1.26.10/src/cmd/go/alldocs.go#L2085-L2093) |
| `-short` with `testing.Short()` | a test skips itself in code | `-short` | no | no | yes | no | [source]; [`alldocs.go`](https://github.com/golang/go/blob/go1.26.10/src/cmd/go/alldocs.go#L3410-L3414) |

RE2 has no lookahead. So `-run` gives the intersection "slow and DB" only as `DB.*Slow|Slow.*DB` [run]. Go's boolean language sits in the test file, not on the command line. It selects files, not tests.

### 1.6 Jest 30.5.2

| way | grammar | example | ∪ | ∩ | − | ( ) | source |
|---|---|---|---|---|---|---|---|
| path pattern, positional (`--testPathPatterns`) | a regular expression on the file's path | `jest math` | yes, by `\|` | no | no | no | [run]; [`CLI.md` L522-L524](https://github.com/jestjs/jest/blob/v30.5.2/docs/CLI.md#L522-L524) |
| `--testPathIgnorePatterns` | regular expressions on the path, an array | `--testPathIgnorePatterns store` | no | no | yes | no | [run]; [`CLI.md` L516-L520](https://github.com/jestjs/jest/blob/v30.5.2/docs/CLI.md#L516-L520) |
| `-t REGEX` (`--testNamePattern`) | a JavaScript regular expression on the describe names and the test name, joined by spaces | `-t '^(?!.*db).*slow'` | yes, by `\|` | yes, by lookaheads | yes, by a negative lookahead | no | [run]; [`CLI.md` L506-L514](https://github.com/jestjs/jest/blob/v30.5.2/docs/CLI.md#L506-L514), [`jest-circus/src/run.ts` L407-L413](https://github.com/jestjs/jest/blob/v30.5.2/packages/jest-circus/src/run.ts#L407-L413) |
| `--selectProjects`, `--findRelatedTests` | project names; source files | `--selectProjects api` | yes | no | no | no | [source]; [`CLI.md` L239, L441](https://github.com/jestjs/jest/blob/v30.5.2/docs/CLI.md#L441) |
| `.only`, `.skip` in code | a call in the test file | `test.only(...)` | no | no | no | no | [source] |

`-t '(?=.*slow)(?=.*db)'` selected only "store migrate db slow" [run]. A JavaScript regular expression thus expresses ∩ and − over the characters of one name. It cannot join two kinds of pick, such as a path and a name (judgement).

### 1.7 Vitest 5.1.1

| way | grammar | example | ∪ | ∩ | − | ( ) | source |
|---|---|---|---|---|---|---|---|
| file filter, positional | a substring of the file's path, repeatable | `vitest math` | yes, repeated | no | no | no | [run]; [`filtering.md` L28-L44](https://github.com/vitest-dev/vitest/blob/v5.1.1/docs/guide/filtering.md#L28-L44) |
| `file:line`, positional | a full file name and a line number | `vitest math.test.mjs:7` | yes, repeated | no | no | no | [run]; [`filtering.md` L62-L85](https://github.com/vitest-dev/vitest/blob/v5.1.1/docs/guide/filtering.md#L62-L85) |
| `-t REGEX` | a JavaScript regular expression on the full name, joined by ` > ` | `-t '^store > migrate$'` | yes, by `\|` | yes, by lookaheads | yes, by a negative lookahead | no | [run]; [`filtering.md` L46-L60](https://github.com/vitest-dev/vitest/blob/v5.1.1/docs/guide/filtering.md#L46-L60) |
| `--tags-filter EXPR` | `and`/`&&`, `or`/`\|\|`, `not`/`!`, `( )` and `*` over tags that the config defines; repeatable | `--tags-filter 'slow && !db'` | yes | yes | yes | yes | [run]; [`test-tags.md` L260-L292](https://github.com/vitest-dev/vitest/blob/v5.1.1/docs/guide/test-tags.md#L260-L292) |
| `--exclude GLOB` | a glob on the file's path, repeatable | `--exclude 'store*'` | no | no | yes | no | [source]; [`cli-config.ts`](https://github.com/vitest-dev/vitest/blob/v5.1.1/packages/vitest/src/node/cli/cli-config.ts) |
| `--project NAME` | a project name with `*`, or `!pattern` to exclude; repeatable | `--project 'pkg*' --project '!pkg-slow'` | yes | no | yes | no | [source]; [`cli-config.ts` L764-L767](https://github.com/vitest-dev/vitest/blob/v5.1.1/packages/vitest/src/node/cli/cli-config.ts#L764-L767) |
| `.only`, `.skip` in code | a call in the test file | `test.only(...)` | no | no | no | no | [source]; [`filtering.md` L107-L160](https://github.com/vitest-dev/vitest/blob/v5.1.1/docs/guide/filtering.md#L107-L160) |

The probes spell the flag `--tagsFilter`, as `vitest --help` does. The guide spells it `--tags-filter`. Vitest added tags and this flag on 2026-01-22, in [`de7c8a52`](https://github.com/vitest-dev/vitest/commit/de7c8a521daa6b79c923a7b65ec6c010d97300cf). Its guide gives a table of when to use a tag, `-t` or a file filter ([`test-tags.md` L16-L27](https://github.com/vitest-dev/vitest/blob/v5.1.1/docs/guide/test-tags.md#L16-L27)) [source]. Vitest runs each test file to learn which tests match `-t` or a tag ([`filtering.md` L12](https://github.com/vitest-dev/vitest/blob/v5.1.1/docs/guide/filtering.md#L12)) [source].

### 1.8 oxitest at `731c2b3f`

| way | grammar | example | ∪ | ∩ | − | ( ) | source |
|---|---|---|---|---|---|---|---|
| path or node id, positional | `path[::Class]::test[[case]]`; `*` and `?` make a glob; repeatable | `oxitest tests/test_math.py::test_add tests/test_strings.py` | yes, repeated | no | no | no | [source]; [`filter-tests.md` L43-L87](https://github.com/kalonji-tools/oxitest/blob/731c2b3f/docs/user/how-to/filter-tests.md#L43-L87), [`filter.rs` L98-L160](https://github.com/kalonji-tools/oxitest/blob/731c2b3f/src/filter.rs#L98-L160) |
| `-E EXPR` | predicates `name()`, `source()`, `mark()`, `async()`, `uses()`; matchers bare (contains), `~` (contains), `=` (exact), `/regex/`; operators `&`/`and`, `\|`/`or`, `!`/`not`, `( )` | `-E 'mark(slow) & !mark(db)'` | yes | yes | yes | yes | [run]; [`compile.rs` L256-L277](https://github.com/kalonji-tools/oxitest/blob/731c2b3f/src/query/compile.rs#L256-L277), [`resource.rs` L28](https://github.com/kalonji-tools/oxitest/blob/731c2b3f/src/query/resource.rs#L28) |
| `query tests --fzf` | an interactive fuzzy finder | `oxitest query tests --fzf` | yes, by multi-select | no | no | no | [source]; [`filter-tests.md` L89-L99](https://github.com/kalonji-tools/oxitest/blob/731c2b3f/docs/user/how-to/filter-tests.md#L89-L99) |

oxitest reads `-E` twice. Before it imports a file, it tests the expression on the file's AST, and on a parse error it imports the file anyway ([`filter.rs` L530-L549](https://github.com/kalonji-tools/oxitest/blob/731c2b3f/src/filter.rs#L530-L549)). After collection it tests each item again ([`transitions/mod.rs` L58-L83](https://github.com/kalonji-tools/oxitest/blob/731c2b3f/src/pipeline/transitions/mod.rs#L58-L83)) [source].

### 1.9 For comparison: JUnit Platform 6.1.3 and Bazel 9.3.3

| runner | way | grammar | example | ∪ | ∩ | − | ( ) | source |
|---|---|---|---|---|---|---|---|---|
| JUnit | `-t`/`--include-tag`, `-T`/`--exclude-tag` | a tag or a tag expression with `!`, `&`, `\|`, `( )`, `any()`, `none()`; each flag repeatable, joined with OR | `-t 'product & !end-to-end'` | yes | yes | yes | yes | [source]; [`tags.adoc` L30-L47](https://github.com/junit-team/junit-framework/blob/r6.1.3/documentation/modules/ROOT/pages/running-tests/tags.adoc#L30-L47), [`TestDiscoveryOptionsMixin.java` L171-L181](https://github.com/junit-team/junit-framework/blob/r6.1.3/junit-platform-console/src/main/java/org/junit/platform/console/options/TestDiscoveryOptionsMixin.java#L171-L181) |
| Bazel | `--test_tag_filters` | a comma list of tags; `-tag` excludes; a test runs if it has one listed tag and no excluded tag | `--test_tag_filters=slow,-db` | yes | no | yes | no | [source]; [`user-manual.mdx` L975-L988](https://github.com/bazelbuild/bazel/blob/9.3.3/docs/docs/user-manual.mdx#L975-L988) |

For tags, JUnit is the one runner here that offers both forms. A tester can repeat `--include-tag slow --include-tag db`, or write one expression. Bazel offers only the flag, so it cannot ask for the tests that are both slow and db.

## 2. What the probes show

### 2.1 Four questions on one suite

Each cell is the command that answers the question, and the tests it selected. "none" means no flag of the runner answers it. Each cell is [run].

| question | pytest | cargo test | nextest | `go test` | Jest | Vitest | oxitest |
|---|---|---|---|---|---|---|---|
| slow or db (∪) | `-m 'slow or db'`: 3 | `slow db`: 3 | `-E 'test(slow) \| test(db)'`: 3 | `-run 'Slow\|DB'`: 3 | `-t 'slow\|db'`: 3 | `--tags-filter 'slow \|\| db'`: 3 | `-E 'mark(slow) \| mark(db)'`: 3 |
| slow and db (∩) | `-m 'slow and db'`: 1 | none | `-E 'test(slow) & test(db)'`: 1 | `-run 'DB.*Slow\|Slow.*DB'`: 1 | `-t '(?=.*slow)(?=.*db)'`: 1 | `--tags-filter 'slow && db'`: 1 | `-E 'mark(slow) & mark(db)'`: 1 |
| slow, not db (−) | `-m 'slow and not db'`: 1 | `slow --skip db`: 1 | `-E 'test(slow) - test(db)'`: 1 | `-run Slow -skip DB`: 1 | `-t '^(?!.*db).*slow'`: 1 | `--tags-filter 'slow && !db'`: 1 | `-E 'mark(slow) & !mark(db)'`: 1 |
| slow or db, not both (nesting) | `-m '(slow or db) and not (slow and db)'`: 2 | none | `-E '(test(slow) \| test(db)) - (test(slow) & test(db))'`: 2 | none | none tried | `--tags-filter '(slow \|\| db) && !(slow && db)'`: 2 | `-E '(mark(slow) \| mark(db)) & !(mark(slow) & mark(db))'`: 2 |

unittest answers the first row only, with `-k slow -k db` [run]. The third row is the issue's example question. Two runners with no language answer it with two flags: cargo test and `go test`.

### 2.2 One expression, two readings

The expression "db, or slow and not db" selects different tests in different languages [run]:

| runner | expression | selected |
|---|---|---|
| pytest | `-m 'db or slow and not db'` | add slow, query db, migrate db: 3 |
| cargo-nextest | `-E 'test(db) \| test(slow) & not test(db)'` | 3 |
| Vitest | `--tags-filter 'db \|\| slow && !db'` | 3 |
| oxitest | `-E 'mark(db) \| mark(slow) & !mark(db)'` | add slow: 1 |
| oxitest, with parentheses | `-E 'mark(db) \| (mark(slow) & !mark(db))'` | 3 |

pytest, cargo-nextest and Vitest bind AND tighter than OR, and their documents say so: [`expression.py` L6-L7](https://github.com/pytest-dev/pytest/blob/9.2.0/src/_pytest/mark/expression.py#L6-L7), [`reference.md` L73-L89](https://github.com/nextest-rs/nextest/blob/cargo-nextest-0.9.149/nextest-filtering/reference.md#L73-L89), [`test-tags.md` L270](https://github.com/vitest-dev/vitest/blob/v5.1.1/docs/guide/test-tags.md#L270). JUnit does too ([`tags.adoc` L40-L47](https://github.com/junit-team/junit-framework/blob/r6.1.3/documentation/modules/ROOT/pages/running-tests/tags.adoc#L40-L47)). oxitest's grammar puts `&` and `|` in one loop, at one rank ([`compile.rs` L256-L275](https://github.com/kalonji-tools/oxitest/blob/731c2b3f/src/query/compile.rs#L256-L275)). Its tests check that `!` binds tighter than `&`, but no test mixes `&` and `|` without parentheses ([`compile.rs` L650-L667](https://github.com/kalonji-tools/oxitest/blob/731c2b3f/src/query/compile.rs#L650-L667)) [source]. oxitest's how-to does not state a precedence.

### 2.3 A repeated flag

| runner | flags | selected | how they combine | source |
|---|---|---|---|---|
| pytest | `-k add -k db` | query db, migrate db | the last wins | [run] |
| pytest | `-m slow -m db` | query db, migrate db | the last wins | [run] |
| unittest | `-k slow -k db` | 3 tests | union | [run] |
| cargo test | `slow db` | 3 tests | union | [run] |
| cargo-nextest | `-E 'test(slow)' -E 'test(db)'` | 3 tests | union | [run] |
| `go test` | `-run Slow -run DB` | query DB, migrate DB slow | the last wins | [run] |
| `go test` | `-skip DB -skip Slow` | add, query DB, parse | the last wins | [run] |
| Jest | `-t slow -t db` | none, exit 0 | no test matched; I did not trace why | [run] |
| Vitest | `--tags-filter slow --tags-filter db` | migrate | intersection | [run]; [`tags.ts` L54-L63](https://github.com/vitest-dev/vitest/blob/v5.1.1/packages/vitest/src/runtime/runner/utils/tags.ts#L54-L63) |
| JUnit | `--include-tag a --include-tag b` | | union | [source]; [`TestDiscoveryOptionsMixin.java` L171-L174](https://github.com/junit-team/junit-framework/blob/r6.1.3/junit-platform-console/src/main/java/org/junit/platform/console/options/TestDiscoveryOptionsMixin.java#L171-L174) |

So a repeated flag is itself a choice of operator. A tester who knows one runner cannot guess it in another (judgement).

### 2.4 A pick wider than the tester meant

A substring match picks more than the name a tester types [run]:

- pytest `-k math` selects all 4 tests of `test_math.py`, because the file's name is one of the names that `-k` reads.
- pytest `-k 'parse and a'` selects both cases, `[a]` and `[b]`. "a" is a substring of `test_parse`.
- oxitest `-E 'name(math)'` selects all 4 tests of `test_math.py`. The run path gives `name` the whole node id ([`transitions/mod.rs` L87](https://github.com/kalonji-tools/oxitest/blob/731c2b3f/src/pipeline/transitions/mod.rs#L87)). oxitest's how-to says that `name()` matches the function name ([`filter-tests.md` L21](https://github.com/kalonji-tools/oxitest/blob/731c2b3f/docs/user/how-to/filter-tests.md#L21)).
- oxitest `-E 'mark(slo)'` selects the two slow tests. A bare value matches a substring of a marker's name ([`eval.rs` L20-L21](https://github.com/kalonji-tools/oxitest/blob/731c2b3f/src/query/eval.rs#L20-L21)). `mark(=slo)` selects none.
- cargo test, cargo-nextest, `go test`, Jest and Vitest `-t` also match a substring or an unanchored pattern by default. `-run Add` selects `TestAdd` and `TestAddSlow`.

A tag that the grilling agreed on is a literal value. A language over literal tags avoids this case. A language over names does not (judgement).

### 2.5 A query that picks nothing, or names nothing known

| runner | case | exit | message | source |
|---|---|---|---|---|
| pytest | `test_math.py::test_nope` | 4 | `ERROR: not found: …::test_nope` | [run] |
| pytest | `test_math.py::test_ad`, a prefix of a test's name | 4 | `ERROR: not found` | [run] |
| pytest | `-m nosuchmark` | 5 | `no tests collected (6 deselected)` | [run] |
| pytest | `--strict-markers -m slwo` | 4 | `Unknown marker(s) in '-m' expression: slwo. Use 'pytest --markers' to see available markers.` | [run]; [`__init__.py` L313](https://github.com/pytest-dev/pytest/blob/9.2.0/src/_pytest/mark/__init__.py#L313) |
| pytest | `-m 'slow and'` | 4 | `Wrong expression passed to '-m': … at column 9` | [run] |
| unittest | `test_math.Math.test_nope` | 1 | an `AttributeError`, reported as an error of a test | [run] |
| unittest | `-k nosuchname` | 5 | `NO TESTS RAN` | [run] |
| cargo test | `nosuchname` | 0 | `0 passed; … 4 filtered out` | [run] |
| cargo-nextest | `run -E 'test(nosuchname)'` | 4 | `error: no tests to run` | [run] |
| cargo-nextest | `-E 'test(slow) &'` | 94 | `error: expected expression` | [run] |
| `go test` | `-run Nope` | 0 | `testing: warning: no tests to run` | [run] |
| Jest | `-t nosuchname` | 0 | none | [run] |
| Jest | `nosuchfile` | 1 | `No tests found, exiting with code 1` | [run] |
| Vitest | `run -t nosuchname` | 0 | `Tests 4 skipped (4)` | [run] |
| Vitest | `--tags-filter nosuchtag` | 1 | `The tag pattern "nosuchtag" is not defined in the configuration. Available tags are: …` | [run] |
| oxitest | `-E 'tag(slow)'` | 2 in the harness | `predicate 'tag' is not valid for resource 'tests'` | [run] |
| oxitest | `-E 'mark(slow) &'` | 2 in the harness | `empty expression` | [run] |
| oxitest | `-E` that selects nothing | 0 | none, by design | [source]; [`exit-codes.md` L33](https://github.com/kalonji-tools/oxitest/blob/731c2b3f/docs/user/reference/exit-codes.md#L33) |

In these probes, no runner named the nearest id or tag. Vitest lists every defined tag. pytest points to `pytest --markers`. The grilling agreed that purba names the nearest id. In this set, no runner does that.

### 2.6 Deep nesting

Each case wraps one predicate in N pairs of parentheses [run]:

| runner | N that worked | N that failed | failure |
|---|---|---|---|
| pytest `-m` | 200 | 500, 1000 | exit 3, `INTERNALERROR> RecursionError: maximum recursion depth exceeded` |
| cargo-nextest `-E` | 1000 | 10000 | exit 134, `thread 'main' has overflowed its stack` |
| Vitest `--tags-filter` | 1000 | 10000 | exit 1, `RangeError: Maximum call stack size exceeded` |
| oxitest's parser (harness, release build) | 10000 | 50000 | exit 134, stack overflow. 50000 `!` in a row also overflow. |

No tester writes 500 levels by hand. A script that builds an expression can (judgement). Go's parser for `//go:build` lines had the same defect, and Go fixed it as CVE-2024-34158 ([`f22d7319`](https://github.com/golang/go/commit/f22d73197635b23c13bb852e330e5339eefaf910)) [source]. I did not probe the oxitest binary, so its limit there is unmeasured.

## 3. The size of each language, and the fixes it drew

### 3.1 Size

Each row is [run] by `sizes.sh` on the pinned source. "Production" and "test" follow the rules in section "What I counted".

| runner | language | files | production lines | test lines |
|---|---|---|---|---|
| pytest 9.2.0 | `-k`, `-m` | `src/_pytest/mark/expression.py`; `testing/test_mark_expression.py` | 363 | 356 |
| cargo-nextest 0.9.149 | filtersets | `nextest-filtering/src/` (7 files); its tests and test helpers | 2643 | 2732 |
| Vitest 5.1.1 | `--tags-filter` | `runtime/runner/utils/tags.ts`, `node/tags.ts`; `test/e2e/test/test-tags.test.ts` | 366 | 1891 |
| `go test` 1.26.10 | `-run`, `-skip` (not boolean) | `src/testing/match.go`; `match_test.go` | 317 | 263 |
| Go 1.26.10 | `//go:build` | `src/go/build/constraint/expr.go`; `expr_test.go` | 596 | 386 |
| oxitest `731c2b3f` | `-E` | `src/query/ast.rs`, `compile.rs`, `eval.rs`, `resource.rs` | 649 | 578 |
| JUnit 6.1.3 | tag expressions | `junit-platform-launcher/…/tagexpression/` (15 files) | 936 in all, licence headers included, not split | not counted |

The JUnit row is [source]: I fetched the 15 files and counted their lines, and I did not split out tests. pytest's row leaves out the matchers in `mark/__init__.py`, which read the names that `-k` and `-m` match. cargo-nextest's 2643 lines also hold the evaluation against cargo's build graph, which `deps()` and `rdeps()` need.

oxitest's whole query module is larger. `src/query/` with the fuzzy finder, the output formats and the extraction holds 1694 production lines ([run], `oxitest-size.tsv`). oxitest's first language, `-m` in `src/marker.rs`, had 171 production lines and 113 test lines before its removal ([run], `git show 41ccb527^:src/marker.rs`).

### 3.2 pytest

- pytest 6.0.0rc1 replaced Python's `eval()` with its own grammar, in [`a718ad63`](https://github.com/pytest-dev/pytest/commit/a718ad636304ab594eb95c8c64ac28a96354757d) "Stop using Python's eval() for -m and -k" [run, `history.sh`]. Its changelog entry is [#7122](https://github.com/pytest-dev/pytest/blob/9.2.0/doc/en/changelog.rst#L4728) [source]. The same release stopped `-k` from matching the names of directories outside the root ([#7040](https://github.com/pytest-dev/pytest/blob/9.2.0/doc/en/changelog.rst#L4720)), outside a "Bug Fixes" section [source].
- 36 commits touch `expression.py` up to 9.2.0. Matcher: `git log 9.2.0 -- src/_pytest/mark/expression.py` [run].
- Five changelog entries under "Bug Fixes" name `-k`, `-m` or an expression, and concern the language [run, `history.sh`]. Matcher: a line under a "Bug Fixes" heading that holds `-k`, `-m`, "keyword expression", "marker expression" or "match expression". A sixth match, #12824, is about option names, not the language.
  - [#4051](https://github.com/pytest-dev/pytest/blob/9.2.0/doc/en/changelog.rst#L7767), 3.8.2: a poor error for an invalid `-m`.
  - [#4583](https://github.com/pytest-dev/pytest/blob/9.2.0/doc/en/changelog.rst#L4947), 6.0.0rc1: a crash when `eval()` of `-m` raised.
  - [#4649](https://github.com/pytest-dev/pytest/blob/9.2.0/doc/en/changelog.rst#L6983), 4.2.0: `-k` stopped matching markers, and the fix restored it.
  - [#14474](https://github.com/pytest-dev/pytest/blob/9.2.0/doc/en/changelog.rst#L730), 9.1.0: a regression with backslashes and string arguments in one expression.
  - [#4569](https://github.com/pytest-dev/pytest/blob/9.2.0/doc/en/changelog.rst#L352), 9.2.0: `-k` matched attributes that nobody meant as keywords.
- In January 2026, `-m` began to refuse an unknown marker under `--strict-markers` ([`a8cc3f12`](https://github.com/pytest-dev/pytest/commit/a8cc3f122b9b61b11930a33a62aa7a4481382100), closes [#2781](https://github.com/pytest-dev/pytest/issues/2781)) [source].

### 3.3 cargo-nextest

- Filter expressions arrived as experimental in 0.9.13 ([changelog L2246](https://github.com/nextest-rs/nextest/blob/cargo-nextest-0.9.149/site/src/changelog.md#L2246)) and became stable in 0.9.25 ([changelog L2017-L2023](https://github.com/nextest-rs/nextest/blob/cargo-nextest-0.9.149/site/src/changelog.md#L2017-L2023)) [source].
- **Filtersets replaced flags.** 0.9.25 deprecated `--platform-filter` for `-E 'platform(...)'`, and made `--skip` and `--exact` suggest a filterset ([changelog L2067-L2068](https://github.com/nextest-rs/nextest/blob/cargo-nextest-0.9.149/site/src/changelog.md#L2067-L2068)). The design notes give the reason: "rather than every command having to grow a number of options such as `--include` and `--exclude`, developers can learn the query language once and use it everywhere" ([`filtersets/reference.md` L19-L23](https://github.com/nextest-rs/nextest/blob/cargo-nextest-0.9.149/site/src/docs/filtersets/reference.md#L19-L23)) [source].
- **How a flag combines with the language had to be decided, and it changed.** [#160](https://github.com/nextest-rs/nextest/issues/160) asked for positional filters and `-E` to combine with OR. 0.9.25 made them combine with AND [source].
- 349 commits touch `nextest-filtering/`. Matcher: `git log cargo-nextest-0.9.149 -- nextest-filtering` [run]. Most are dependency updates. nextest's subjects do not use a `fix` prefix, so I count fixes in its changelogs.
- Eight lines under "### Fixed" in the site's changelog hold "filter" or "expression", in seven releases [run, `history.sh`]. Four of them fix the language or how it combines:
  - 0.9.15: filtering broke when `-E` was set and no name filter was.
  - 0.9.92: a `binary_id()` pattern that matches nothing is now refused.
  - 0.9.137: `not(test(foo))`, with no space after `not`, now parses. This answers [#1816](https://github.com/nextest-rs/nextest/issues/1816), "filterset binary operators are parsed as function applications".
  - 0.9.145: a `default-filter` in a tool's config file was ignored.

  The others fix `--exact` (0.9.81), the archive (0.9.143) and per-test overrides (0.9.145). One more fix names `-E` but neither word, so this matcher misses it: 0.9.47, "`cargo nextest run -E 'deps(foo)` queries now work again" ([changelog L1730](https://github.com/nextest-rs/nextest/blob/cargo-nextest-0.9.149/site/src/changelog.md#L1730)).
- The label `A-filter-expressions` holds 8 issues, 2 of them open [run]. Matcher: the label, on all 337 issues of the repository.

### 3.4 `go test` and Go's build constraints

- 9 commits touch `src/testing/match.go` up to go1.26.10 [run, GitHub's commit list for the path]. Two changed what a pattern means:
  - [`025308fe`](https://github.com/golang/go/commit/025308fe084264538f49924b3f52d8d6b6359658), 2021: "`A/B|C/D` is effectively `A/(B|C)/D` - with this change, it changes to effectively `(A/B)|(C/D)`". It fixes [#39904](https://github.com/golang/go/issues/39904).
  - [`e4a2c38a`](https://github.com/golang/go/commit/e4a2c38af5bdae12591004c3d35762d23da7a9bd), 2022: `-skip` was added, for proposal [#41583](https://github.com/golang/go/issues/41583). Before it, `go test` had no exclusion.
- 9 commits touch `src/go/build/constraint/expr.go` [run]. One is the fix of CVE-2024-34158, a stack exhaustion on deeply nested constraints ([`f22d7319`](https://github.com/golang/go/commit/f22d73197635b23c13bb852e330e5339eefaf910)).

### 3.5 Vitest

- 6 commits touch the tags files since tags arrived on 2026-01-22 [run, `history.sh`]. One is a fix of typos in its messages. One more fix, [`457db297`](https://github.com/vitest-dev/vitest/commit/457db297b29bac0945cef9edc0bd8e825d7448f4), changed how a test's `tags` merge with its suite's, in another file. Matcher: `git log v5.1.1` on the three paths, and `git log v5.1.1 | grep -i tag` for the second.
- The tags language is ten months old at v5.1.1. Its history is too short to compare (judgement).

### 3.6 oxitest

oxitest's history is the closest to purba's, so I list it in full. Each item is [run] by `oxitest-history.sh`, or [source] at `731c2b3f`.

- **Two languages, then one.** oxitest first had `-m`, a marker language built with `logos`, and `-k`, a substring flag. [#719](https://github.com/kalonji-tools/oxitest/issues/719) removed both for `-E` in [`41ccb527`](https://github.com/kalonji-tools/oxitest/commit/41ccb527695c1bdab665fd2213e88f097ecbe4a6), three days after `-E` arrived in [`c8093717`](https://github.com/kalonji-tools/oxitest/commit/c8093717d34c36dd316892cb1f0b0cf84b4ffc5d).
- **The records lagged the code three times.** [#793](https://github.com/kalonji-tools/oxitest/issues/793) and [#887](https://github.com/kalonji-tools/oxitest/issues/887) removed `-k` and `-m` from the documentation after the code had dropped them. [#795](https://github.com/kalonji-tools/oxitest/issues/795) completed a table of predicates that was missing some. At `731c2b3f`, the help still lists `shared()` ([`query.rs` L19](https://github.com/kalonji-tools/oxitest/blob/731c2b3f/src/config/cli/query.rs#L19)), which [`52f9d706`](https://github.com/kalonji-tools/oxitest/commit/52f9d7061ee9b327965dd187c88bca63e8b81745) retired, and it omits `uses()`. The error for an unknown predicate names four predicates, not five ([`ast.rs` L70](https://github.com/kalonji-tools/oxitest/blob/731c2b3f/src/query/ast.rs#L70)).
- **The parser grew, then split.** `dsl.rs` reached 932 lines in its first commit, and [#757](https://github.com/kalonji-tools/oxitest/issues/757) split it into syntax and semantics. Later issues added error spans ([#899](https://github.com/kalonji-tools/oxitest/issues/899)), compiled each regex once ([#1094](https://github.com/kalonji-tools/oxitest/issues/1094)) and added `uses()` ([#1107](https://github.com/kalonji-tools/oxitest/issues/1107)).
- **Fixes.** 8 commits on the query files have a subject that starts with `fix` [run]. Matcher: `git log 731c2b3f` on `src/query`, `src/config/cli/query.rs`, `query_bridge.py` and `src/marker.rs`, subject `^fix`. Two of them touch the language itself: [`c2808ab8`](https://github.com/kalonji-tools/oxitest/commit/c2808ab8d5a8f8e8d454a9884e54591e435b082e), where a parse error made oxitest skip the Python session it needed, and [`52f9d706`](https://github.com/kalonji-tools/oxitest/commit/52f9d7061ee9b327965dd187c88bca63e8b81745), which retired `shared()`.
- **Ids drew their own defects.** [#737](https://github.com/kalonji-tools/oxitest/issues/737): the query left the class out of a node id. [#738](https://github.com/kalonji-tools/oxitest/issues/738): selecting by node id gave false "unused fixture" errors. [#1797](https://github.com/kalonji-tools/oxitest/issues/1797): a node id that matched nothing exited 0, "so a typo passes CI green". [#2026](https://github.com/kalonji-tools/oxitest/issues/2026): a relative path resolved two ways. A Windows path prefix made every node id look like a glob ([`filter.rs` L103-L118](https://github.com/kalonji-tools/oxitest/blob/731c2b3f/src/filter.rs#L103-L118)).
- **Two readers disagreed.** In [#1681](https://github.com/kalonji-tools/oxitest/issues/1681), `oxitest query tests` listed 700 tests, and `oxitest run` collected 0 on the same suite. The query read the AST, and the run imported each module.
- **Counts.** Of 1317 issues, 83 match the title or body matcher that `oxitest-history.sh` prints [run]. This section names 13 of them. I did not classify the other 70.

## 4. The questions that only a language answers

A runner with flags gives one fixed shape: a union of includes, cut by each other kind of flag, less a union of excludes (judgement). For example:

```text
cargo test  slow db --skip migrate       →  (slow ∪ db) − migrate
go test     -run 'Slow|DB' -skip Migrate  →  (Slow ∪ DB) − Migrate
bazel       --test_tag_filters=slow,db,-flaky  →  (slow ∪ db) − flaky
pytest      tests/store -m slow           →  under tests/store ∩ slow
```

A question fits flags when its answer has this shape. It needs a language when it does not. Each row names the flags that answer it, where any do.

| question a tester asks | shape | flags that answer it, with no language | source |
|---|---|---|---|
| Run the slow tests. | one pick | a tag flag: JUnit `-t slow`, Bazel `slow`. In purba, a tag flag. | [source] |
| Run the slow tests and the database tests. | A ∪ B | a repeated include: unittest `-k slow -k db`, cargo test `slow db`, nextest `slow db`, JUnit `-t slow -t db`, Bazel `slow,db` | [run], [source] |
| Run the slow tests, but not the database tests. | A − B | an include and an exclude: cargo test `slow --skip db`, `go test -run Slow -skip DB`, Bazel `slow,-db`, JUnit `-t slow -T db` | [run], [source] |
| Run the slow tests of module `store`. | id ∩ tag | an id prefix and a tag flag, which combine with AND. pytest's `test_store.py -m slow` does this, with `-m` holding one tag. | [run] |
| Run the tests that are both slow and database tests. | A ∩ B, one kind | only a flag that repeats with AND: Vitest `--tags-filter slow --tags-filter db`. No other runner here. | [run] |
| Run the slow tests, and also everything under `store::migrate`. | tag ∪ id | none. Every runner here joins an id and a tag with AND. | judgement, from sections 1 and 2.3 |
| Run everything except the tests that are both slow and database tests. | − (A ∩ B) | none | judgement |
| Run the slow or database tests, but not those that are both. | (A ∪ B) − (A ∩ B) | none | [run] in 2.1 |
| Run the tests that carry no tag. | complement of all tags | none in this set; JUnit's language has `none()` | [source] |

Three patterns need a language, by this table (judgement):

1. An intersection of two picks of one kind, unless the flag of that kind repeats with AND.
2. A union of picks of two kinds, such as a tag and an id.
3. Any nesting: an exclusion of an intersection, or a union of intersections.

The issue's example, "run the slow tests, but not the tests of the database", is a single exclusion. Two flags answer it. I found no source that measures how often testers ask the questions in the last four rows. That frequency is unmeasured.

## 5. purba's facts that bear on this

Each link is pinned at `3093871b`.

- purba states the need to parse a query language that names tests ([`.config/needs.toml` L10](https://github.com/kalonji-tools/purba/blob/3093871b/.config/needs.toml#L10)). The need cites selection, which "answers which tests a query names" ([`LANDSCAPE.md` L58](https://github.com/kalonji-tools/purba/blob/3093871b/LANDSCAPE.md#L58)).
- Selection reads the AST of each test file and imports none ([`LANDSCAPE.md` L58](https://github.com/kalonji-tools/purba/blob/3093871b/LANDSCAPE.md#L58)). A literal tag is in the AST. Vitest's `-t` and tags decide only after each test file runs ([`filtering.md` L12](https://github.com/vitest-dev/vitest/blob/v5.1.1/docs/guide/filtering.md#L12)). nextest runs a test binary to list its tests, unless the filterset excludes the whole binary ([`reference.md` L41-L54](https://github.com/nextest-rs/nextest/blob/cargo-nextest-0.9.149/nextest-filtering/reference.md#L41-L54)) [source]. pytest's `-k` and `-m` act on collected items, after import (judgement, from where [`deselect_by_keyword`](https://github.com/pytest-dev/pytest/blob/9.2.0/src/_pytest/mark/__init__.py#L228) runs).
- No part names a test except by the identity that selection gives it ([`LANDSCAPE.md` L144](https://github.com/kalonji-tools/purba/blob/3093871b/LANDSCAPE.md#L144)). oxitest's [#1681](https://github.com/kalonji-tools/oxitest/issues/1681) shows two readers of one suite that disagree.
- A decorator of purba belongs to purba's public surface ([`a-test-carries-only-purbas-own-decorators.md` L39](https://github.com/kalonji-tools/purba/blob/3093871b/docs/decisions/a-test-carries-only-purbas-own-decorators.md#L39)). So does each flag ([`CONTEXT.md` L245](https://github.com/kalonji-tools/purba/blob/3093871b/CONTEXT.md#L245)). The surface starts empty, and a name joins it only through the issue that needs it ([`purbas-public-surface-starts-empty-and-has-no-plugin-api.md` L35-L37](https://github.com/kalonji-tools/purba/blob/3093871b/docs/decisions/purbas-public-surface-starts-empty-and-has-no-plugin-api.md#L35-L37)). A grammar is a larger surface than a flag. Each operator, matcher and precedence rule becomes part of it (judgement).
- An answer that picks a form a tester writes owes a blind rating before the person decides ([`a-form-that-a-tester-meets-is-rated-blind-before-it-is-decided.md` L28](https://github.com/kalonji-tools/purba/blob/3093871b/docs/decisions/a-form-that-a-tester-meets-is-rated-blind-before-it-is-decided.md#L28)). Whether a query's syntax counts as such a form is a judgement for the person.
- The grilling of #1226 agreed that a query that picks nothing is an invocation error that names the nearest id. oxitest exits 0 when `-E` selects nothing ([`exit-codes.md` L33](https://github.com/kalonji-tools/oxitest/blob/731c2b3f/docs/user/reference/exit-codes.md#L33)). No runner in section 2.5 names a nearest id.

## 6. The shapes the person can weigh

This section names three shapes by what they do, with the command each gives. It does not pick one.

**Tag and id flags, each repeated as a union, with one exclude flag.** The tester writes:

```text
purba --tag slow --tag db --exclude-tag flaky store::
```

It answers the first four rows of section 4. Its grammar is the grammar of an id and a tag. Bazel and cargo test work this way.

**The same flags, where a repeated `--tag` means AND.** The tester writes:

```text
purba --tag slow --tag db        # tests that are both
```

It adds the fifth row of section 4, and loses the second row unless a second flag gives a union. Vitest's `--tags-filter` repeats this way. Section 2.3 shows that runners disagree on what a repeat means.

**A query language over tags and ids, such as oxitest's `-E`.** The tester writes:

```text
purba -E 'tag(slow) & !tag(db) | id(store::migrate)'
```

It answers every row of section 4. It brings a precedence to state and to test (section 2.2), a depth limit (section 2.6), and a grammar on the public surface. JUnit offers both this shape and the first one.

## Appendix: the files of this research

| file | holds |
|---|---|
| `research/selection-queries/<runner>/run.sh` | the probe of one runner; `output.txt` beside it is its output in this session |
| `research/selection-queries/oxitest/main.rs` | the harness around oxitest's parser and evaluator |
| `research/selection-queries/sizes.sh`, `sizes.tsv` | the size of each language |
| `research/selection-queries/oxitest-size.sh`, `oxitest-size.tsv` | the size of each file of oxitest's query module |
| `research/selection-queries/history.sh`, `history.txt` | the commits and changelog fixes of pytest, cargo-nextest, Vitest and Go |
| `research/selection-queries/oxitest-history.sh`, `oxitest-history.txt` | the commits, fixes and issues of oxitest's query language |
