# How tools and projects find the crates that could meet a stated need

Research for [How do tools and projects find the crates that could meet a stated need?](https://github.com/kalonji-tools/purba/issues/994).
Measured on 2026-11-15, against purba `main` at `af20414`. This file picks no source and ranks none. The grilling that follows picks.

## Summary

Twelve sources can list crates. Only one returns crates for a need's words directly, with no key and no ban on scripts: the crates.io API. The crates.io database dump, and the curated lists as files, let a script bring its own matcher. GitHub's repository search also takes words, but it returns repositories, not crates. lib.rs disallows its search pages in its `robots.txt`. libraries.io refuses a request with no API key. deps.dev and ecosyste.ms take a package name, not free words.

The need's own words find few of the 41 crates. The best single mode finds 20 of the 41 in its top 50: crates.io, queried one content word at a time as a keyword. The need's whole sentence, as written, finds none. All automated modes together find 26 of the 41. Fifteen crates are in no mode's top 50.

The gap is one of words. A need speaks purba's language: "assay", "zygote", "read the command's flags". A crate speaks its own: "CLI arguments parser", "syscalls", "*nix APIs". A person bridged that gap in the hand research, with queries such as `fork process unix`.

The manifests of comparable projects hold 32 of the 41, in the Cargo.lock files of eight projects. That is more than any search finds from the need's words. They do not say which need a crate meets, and they hold 1,282 crates in all.

A source pinned to a snapshot repeats exactly: the dump of one date, a list at one commit, a lock file at one SHA. Two runs of the live crates.io API, about seven minutes apart, gave the same top 50 in the same order for all 262 queries. Only the count of matches grew, by 1 to 7 crates, in 73 queries. Its ranking breaks ties by downloads in the last 90 days, and new crates arrive each minute, so its order can move over days. Days were not measured.

## How to read this file

- **Tiers.** [A] = fetched this session, with the command in [Appendix A](#appendix-a-commands). [B] = relayed and not re-verified. [C] = could not establish.
- **The 41 crates and the 24 needs** come from [`research/future-crates.md`](https://github.com/kalonji-tools/purba/blob/8607015ff23c21b1c41fea9d17c6f693502f6679/research/future-crates.md), § 1 and § 2 [A]. 17 of the 24 stated needs have a candidate. They give 32 crates in 41 (need, candidate) pairs. The other 9 crates belong only to the 6 inferred needs (I1 to I6), so the queries cover those needs too.
- **Words of a need.** *As written* is the need's text from § 1, unchanged. *Content words* drop a fixed list of stopwords, split on every character that is not a letter or a digit, and drop `'s` ([words.sh](#a3-the-query-words)). Example: S2 "fork a process for each assay" gives `fork process assay`.
- **Recall.** A pair (need, candidate) is found at K when the candidate is in that need's results at rank K or better. *Crates of 41* counts a crate as found when any of its pairs is found, stated or inferred.
- **No person chose a word.** Every query uses the need's words only. A person who rewrites the words gets other results; this file does not measure that.

## 1. The sources

| source | takes | gives | limit and terms | a script can call it | tier |
|---|---|---|---|---|---|
| **crates.io API**, `GET /api/v1/crates` | `q` (free words), `keyword`, `all_keywords`, `category`, `sort`, `per_page` (at most 100, [pagination.rs#L22](https://github.com/rust-lang/crates.io/blob/08dbe21957e5d9f105faf5798c4d6613628d0af1/src/controllers/helpers/pagination.rs#L22)) | crates with name, description, downloads, recent downloads, newest version; `meta.total` | "A maximum of 1 request per second", and a `user-agent` that identifies the client ([data-access](https://github.com/rust-lang/crates.io/blob/08dbe21957e5d9f105faf5798c4d6613628d0af1/svelte/src/routes/data-access/+page.svelte#L112)) | yes, with no key | [A] |
| **crates.io API**, `/api/v1/keywords`, `/api/v1/categories`, `/api/v1/crates/{name}/reverse_dependencies` | a keyword, a category slug, or a crate name | a keyword's or a category's crates; the crates that depend on a crate | the same | yes. A reverse-dependency query needs a crate first, not words | [A] (paths in the [OpenAPI](https://crates.io/api/openapi.json)) |
| **crates.io database dump**, `static.crates.io/db-dump.tar.gz` | nothing: the whole database | CSV tables: crates (with description and README), keywords, categories, versions, dependencies | "No rate limits apply to static.crates.io at present." The dump is "updated every 24 hours" ([data-access](https://github.com/rust-lang/crates.io/blob/08dbe21957e5d9f105faf5798c4d6613628d0af1/svelte/src/routes/data-access/+page.svelte#L64)). 2,246,923,427 bytes on 2026-11-15 | yes. A script brings its own matcher; this file used DuckDB | [A] |
| **crates.io sparse index**, `index.crates.io` | a crate name | each version's dependencies and features | "No rate limits are required to use data from the sparse crate index." | yes, but an entry holds only `name`, `vers`, `deps`, `cksum`, `features`, `yanked` and `pubtime`: no description and no keyword. So it cannot take a need's words | [A] |
| **lib.rs** | words (`/search?q=`), a keyword (`/keywords/`), a category page, a crate page | a ranked list. Its ranking combines downloads, reverse dependencies, upkeep and more ([about](https://lib.rs/about)). Each crate page "Shows similar/related crates" | `robots.txt` has `Disallow: /search` and `Disallow: /keywords/` for `User-Agent: *` | **no** for search and keywords: robots forbid it. Category and crate pages are not disallowed. No API was found [C] | [A] |
| **blessed.rs** | nothing: one JSON file | 205 recommendations of 200 crates, grouped by purpose, each with notes ([data/crates.json](https://github.com/nicoburns/blessed-rs/blob/6e317b7e22a5eed75832619054f3f6c39162c228/data/crates.json)) | a file in a public GitHub repository; terms [C] | yes, as a file. It has no search, so a script needs a matcher | [A] |
| **awesome-rust** | nothing: one README | 1,990 list lines, most linking a repository, some a crate ([README.md](https://github.com/rust-unofficial/awesome-rust/blob/aa03ceba2c5dffa7d91bdab6602e21d8e46a4039/README.md)) | a file; terms [C] | yes, as a file. A line names a repository more often than a crate, so a script maps repository to crate through the dump | [A] |
| **Rust Cookbook** | nothing | 60 dependencies in its `Cargo.toml`, and recipes by task ([Cargo.toml](https://github.com/rust-lang-nursery/rust-cookbook/blob/555abf48f78cc8a388bf917cb2a755ecaf3109ea/Cargo.toml)) | a file; terms [C] | yes, as a file. It does not say which crate meets which task, except in prose | [A] |
| **deps.dev** (Open Source Insights) API | a package name and version, or a content hash | dependencies, dependents count, advisories, licences, similarly named packages | Use is subject to the Google API Terms of Service. "Clients are expressly permitted to cache data served by the API." ([v3 docs](https://docs.deps.dev/api/v3/)). Rate limit: not found [C] | yes, with no key. **It takes no words**: the v3alpha method list has no search ([v3alpha docs](https://docs.deps.dev/api/v3alpha/)), and `GET /v3alpha/query?q=argument+parser` answers `400 invalid query` | [A] |
| **ecosyste.ms packages** API | a package name, a keyword, a registry | package metadata; a keyword's packages across all registries; `related_packages` of one package | 5,000 requests for each window in the anonymous tier (response headers `x-ratelimit-limit: 5000`, `x-ratelimit-tier: anonymous`; the reset fell on the next full hour). The "polite pool" asks for an email address ([api](https://ecosyste.ms/api)). APIs "are available under CC-BY-SA-4.0 license" | yes, with no key. **It takes no free words**: its OpenAPI has no `q` parameter. Keyword `junit` gave 50 packages, 1 of them a crate (`rs-junit`). Related packages of `lexopt`: `ming` only | [A] |
| **libraries.io** API | `q` (free words), with `platforms`, `keywords`, `licenses`, `sort` | projects with SourceRank and dependents | "All requests are subject to a 60 request/minute rate limit based on your API key" ([api](https://libraries.io/api)) | **only with a key**. A request with no key answers `401` and "An API key is required." A key needs an account | [A]; its recall [C] |
| **GitHub repository search** | free words with qualifiers such as `language:rust` | repositories, not crates. A script maps a repository to its crates through the dump's `repository` field | 30 search requests for each window (`gh api rate_limit`), signed in | yes. All words must match | [A] |
| **manifests of comparable projects** | a list of projects, chosen by a person | each project's dependencies (direct) and its whole lock file (all) | GitHub's API limits; each project's licence [C] | yes, once the list exists. A manifest does not say which need a crate meets | [A] |

The eight projects are uv, ruff, nextest, maturin, maelstrom, karva, rtest and tryke, each at the head of its default branch on 2026-11-15. uv and nextest are the two the hand research read. ruff and maturin are Rust tools for Python. karva and rtest are Python test runners in Rust, named from prior knowledge. maelstrom and tryke came from `gh search repos 'python test runner' --language=rust`. The SHAs are in [Appendix A](#a5-manifests).

### What each source holds of the 41, ignoring the need

| source | holds | of the 41 | the 41 it does not hold |
|---|---|---|---|
| crates.io API and dump | every published crate (378,149 in the dump) | 41 | none |
| manifests, all locked packages | 1,282 crates | 32 | `bpaf` `encoding` `gag` `junit-report` `lexopt` `path-clean` `pyo3-async-runtimes` `seccompiler` `textcode` |
| manifests, direct dependencies | 624 names | 23 | the 9 above, and `chumsky` `difflib` `encoding_rs` `nom` `os_pipe` `pest` `pico-args` `polling` `quick-xml` |
| awesome-rust | 1,990 lines | 16 | 25. Two of the 16 share their repository's line with another crate: `toml_edit` (on the `toml-rs/toml` line) and `rmp-serde` (on the `3Hren/msgpack-rust` line) |
| blessed.rs | 200 crates | 12 | 29 |
| Rust Cookbook | 60 dependencies | 7 | 34 |
| GitHub search, deps.dev, ecosyste.ms, lib.rs, libraries.io | — | not measured as a whole | — |

Each project's share, direct / all: uv 15 / 22, nextest 14 / 22, ruff 11 / 21, maturin 10 / 19, maelstrom 7 / 18, tryke 7 / 16, karva 6 / 13, rtest 4 / 7 [A].

Keywords and categories limit what a keyword or category query can reach. Three of the 41 have no keyword: `nix`, `pep508_rs` and `supports-color`. Fourteen have no category [A, from the dump].

## 2. Prior art

| tool or project | how it proposes crates for a need | tier |
|---|---|---|
| `cargo search` | "This performs a textual search for crates on <https://crates.io>." It calls `/crates?q={}&per_page={}` with a limit of 10 by default and 100 at most ([cargo-search.txt](https://github.com/rust-lang/cargo/blob/930d19d31c3090019d7394e29daf3dd28bc5abe7/src/doc/man/generated_txt/cargo-search.txt), [crates-io/lib.rs#L360-L372](https://github.com/rust-lang/cargo/blob/930d19d31c3090019d7394e29daf3dd28bc5abe7/crates/crates-io/lib.rs#L360-L372)). So it gives what the crates.io API gives | [A] |
| crates.io categories and RFC 1824 | RFC 1824 aims to "create a default ranking of crates within a list of crates" that have a category or keyword. It leaves search order to relevance ([RFC 1824](https://github.com/rust-lang/rfcs/blob/f43d5789b838fb0d80962e3d4b45fb9ddfcc9a07/text/1824-crates.io-default-ranking.md)). Its appendix compares Django Packages, Libhunt, Maven Repository, PyPI, GitHub Showcases, Ruby Toolbox, npms and Package Control | [A] |
| lib.rs | ranks a search or a category by many quality signals, and "Shows similar/related crates on each crate page, which helps discovering better alternatives." ([about](https://lib.rs/about)). It "Automatically categorizes crates and adds missing keywords" | [A] |
| blessed.rs | a person curates crates by purpose. For example, the purpose `CLIs > Argument Parsing > Minimal` lists `lexopt` and `pico-args` | [A] |
| awesome-rust, Rust Cookbook | a community list by topic; recipes by task | [A] |
| cargo-crev | a package review can name "potential alternatives, similar or better" ([editing-package-review.md#L62](https://github.com/crev-dev/cargo-crev/blob/e0e71f9dc2938c061ae0e99554caa8928d2299f1/crev-lib/rc/doc/editing-package-review.md#L62)). The alternatives come from reviewers, and need a crate to start from | [A] the field; how many reviews carry it [C] |
| deps.dev | `GetSimilarlyNamedPackages`: names close to a given name. For `clap` and for `lexopt` it returned no package | [A] |
| ecosyste.ms | `related_packages` of one package | [A] |
| libraries.io | words, filtered by platform and keyword, sorted by `rank` (SourceRank) and others | [A] the docs; [C] the results |
| purba's own hand research | a person wrote queries such as `q=fork process unix`, and read the manifests of uv and nextest ([§ 5](https://github.com/kalonji-tools/purba/blob/8607015ff23c21b1c41fea9d17c6f693502f6679/research/future-crates.md#5-considered-and-left-out)) | [A] |

No tool found here takes a need's sentence and returns crates for it. Each takes words, a category, or a crate to start from. Whether a tool driven by a language model does this was not searched [C].

## 3. Recall

### The modes

| mode | source | the query for one need |
|---|---|---|
| `full` | crates.io API | `q=<the need as written>&per_page=50` |
| `word` | crates.io API | `q=<one content word>&per_page=50`, for each content word; a crate's rank is its best rank in any of the need's queries |
| `kw` | crates.io API | `keyword=<one content word>&sort=recent-downloads&per_page=50`, merged as `word` |
| `kw_or` | dump | crates that have any content word as a keyword, by total downloads |
| `score` | dump | the number of distinct content words among a crate's name parts, keywords and description words; then total downloads |
| `score_m` | dump | `score`, inside the 1,282 crates of the eight lock files |
| `blessed` | blessed.rs | `score` over the purpose names and notes |
| `awesome` | awesome-rust | `score` over each list line; a repository maps to the 41 by the dump's `repository` field |
| GitHub | GitHub search | `<content words> language:rust`, 50 results |

`word` and `kw` merge one list for each content word, so their pool is larger. At 50 they return 225 and 170 distinct crates for each need on average. The other modes return 50 or fewer. Compare them at 50 with that in mind.

### Recall at 10 and at 50

| mode | stated pairs at 10 | stated pairs at 50 | crates of 41 at 10 | crates of 41 at 50 | mean crates returned for each need, at 50 |
|---|---|---|---|---|---|
| `full` | 0/41 | 0/41 | 0/41 | 0/41 | 41.8 |
| `word` | 8/41 | 14/41 | 11/41 | 19/41 | 225.3 |
| `kw` | 8/41 | 12/41 | 16/41 | 20/41 | 169.5 |
| `kw_or` | 4/41 | 10/41 | 11/41 | 19/41 | 50.0 |
| `score` | 2/41 | 2/41 | 4/41 | 7/41 | 50.0 |
| `score_m` | 5/41 | 9/41 | 13/41 | 20/41 | 38.7 |
| `blessed` | 4/41 | 4/41 | 4/41 | 4/41 | 10.1 |
| `awesome` | 4/41 | 5/41 | 4/41 | 7/41 | 34.3 |
| GitHub | 0/41 | 0/41 | 0/41 | 0/41 | 0.4 repositories (11 in all; 27 of 30 needs return none) |
| **all modes together** | — | 18/41 | — | 26/41 | — |

All [A]; crates.io figures from run 1.

### Which candidates each mode finds, by need, at 50

`-` = none. Five needs get no hit in any mode: S2, S3, E1, D2 and I6. GitHub search is left out of the table: it found none of the 41.

| need | candidates | word | kw | full | kw_or | score | score_m | blessed | awesome |
|---|---|---|---|---|---|---|---|---|---|
| S2 | 2 | - | - | - | - | - | - | - | - |
| S3 | 3 | - | - | - | - | - | - | - | - |
| P1 | 3 | `encoding_rs`, `encoding` | `encoding_rs`, `encoding` | - | `encoding_rs`, `encoding` | - | `encoding_rs` | - | `encoding_rs`, `encoding` |
| C1 | 3 | - | - | - | - | - | - | `toml` | - |
| Sel1 | 2 | `walkdir`, `ignore` | `walkdir` | - | `walkdir` | - | `walkdir`, `ignore` | `walkdir`, `ignore` | - |
| Sel2 | 4 | `chumsky` | `chumsky` | - | `chumsky` | - | `chumsky` | - | - |
| Sel3 | 2 | `path-clean` | `path-clean`, `normpath` | - | `path-clean`, `normpath` | - | `normpath` | - | - |
| F1 | 1 | `pyo3-async-runtimes` | `pyo3-async-runtimes` | - | - | - | - | - | - |
| E1 | 2 | - | - | - | - | - | - | - | - |
| E2 | 1 | `pyo3-async-runtimes` | `pyo3-async-runtimes` | - | - | - | - | - | - |
| D1 | 2 | - | - | - | - | - | - | `nix` | - |
| D2 | 4 | - | - | - | - | - | - | - | - |
| D3 | 3 | `serde_json` | - | - | - | - | - | - | - |
| D5 | 1 | `petgraph` | `petgraph` | - | `petgraph` | - | `petgraph` | - | `petgraph` |
| R1 | 1 | `serde_json` | - | - | - | - | - | - | - |
| R2 | 3 | `quick-junit`, `quick-xml`, `junit-report` | `quick-junit`, `quick-xml`, `junit-report` | - | `quick-junit`, `quick-xml`, `junit-report` | `quick-junit`, `junit-report` | `quick-junit`, `quick-xml` | - | `quick-xml` |
| Cmd1 | 4 | - | - | - | - | - | `clap` | - | `clap` |
| I1 | 3 | `toml`, `toml_edit`, `pyproject-toml` | `toml`, `toml_edit`, `pyproject-toml` | - | `toml`, `toml_edit`, `pyproject-toml` | `pyproject-toml` | `toml`, `toml_edit`, `pyproject-toml` | `toml` | `toml`, `toml_edit` |
| I2 | 3 | `imara-diff` | `similar`, `imara-diff`, `difflib` | - | `similar`, `imara-diff`, `difflib` | - | `similar`, `imara-diff`, `difflib` | - | - |
| I3 | 3 | `supports-color` | `anstream`, `owo-colors` | - | `anstream`, `owo-colors` | `anstream`, `supports-color` | `anstream`, `owo-colors`, `supports-color` | - | - |
| I4 | 1 | `globset` | `globset` | - | `globset` | `globset` | `globset` | - | - |
| I5 | 1 | `pep508_rs` | - | - | - | `pep508_rs` | `pep508_rs` | - | - |
| I6 | 1 | - | - | - | - | - | - | - | - |

### The 15 crates no mode finds at 50

`bpaf` `gag` `lexopt` `nom` `os_pipe` `pest` `pico-args` `polling` `postcard` `rmp-serde` `rustix` `seccompiler` `textcode` `tokio` `winnow` [A].

These 15 crates make 19 (need, candidate) pairs. In 15 of the 19 pairs, the crate's name parts, keywords and description share **no** content word with the need. In the other 4, they share one common word, which ranks the crate below 50: `bpaf` and `lexopt` share "command" with Cmd1, `gag` shares "output" with E1, and `textcode` shares "encoding" with P1 [A, `shared.sql` on the dump].

Three examples show the gap:

- **A need in purba's words.** S2 says "fork a process for each assay". `rustix` describes itself as "Safe Rust bindings to POSIX/Unix/Linux/Winsock-like syscalls". D2 says "end a test that hangs, on a timeout". `polling` says "Portable interface to epoll, kqueue, event ports, and IOCP".
- **A need that names an action, not the tool.** Cmd1 says "read the command's flags". `pico-args` says "An ultra simple CLI arguments parser."
- **A need that names one of several kinds.** Sel2 says "parse the query language that names tests". `nom` and `winnow` say "parser combinators library", and `pest` says "The Elegant Parser". The dump and list matchers here do not stem, so "parse" does not match "parser". crates.io stems, but `q=parse` matched 49,623 crates in run 1, and none of the three is in its top 50 [A].

Five of the 15 sit in blessed.rs under a purpose whose words differ from the need's: `lexopt` and `pico-args` (`Argument Parsing`), `rustix` (`*nix (OSs)`), `tokio` (`General Purpose Async Executors`) and `postcard` (`Non-self-describing, no external schema file`). Ten sit in the eight lock files: `nom`, `os_pipe`, `pest`, `pico-args`, `polling`, `postcard`, `rmp-serde`, `rustix`, `tokio` and `winnow` [A].

### Why the whole sentence finds nothing on crates.io

crates.io ranks with PostgreSQL full-text search. The query is `plainto_tsquery` with the `english` configuration, and the rank is `ts_rank_cd` ([search.rs#L162-L163](https://github.com/rust-lang/crates.io/blob/08dbe21957e5d9f105faf5798c4d6613628d0af1/src/controllers/krate/search.rs#L162-L163)) [A]. In `plainto_tsquery`, "the & (AND) tsquery operator is inserted between surviving words" ([PostgreSQL docs](https://www.postgresql.org/docs/current/textsearch-controls.html)) [A]. The `english` configuration also stems: its example turns "Rats" into `rat`. The indexed text weights the name A, keywords B, the description C, and **the README D** ([the trigger, 2016](https://github.com/rust-lang/crates.io/blob/08dbe21957e5d9f105faf5798c4d6613628d0af1/migrations/20160219125609_dumped_migration_105/up.sql)) [A]. A code search found no later migration that redefines that function [A, `gh api search/code`; that the search is complete is B].

So a sentence matches only crates whose indexed text, README included, holds every word. Long READMEs win. For "read the command's flags", 12,574 crates match; the first six are `agcli`, `oneharness`, `onetaskgraph-core`, `zshrs`, `coderipper` and `asupersync` [A]. Ranking is also limited to 1,000 candidates. They are chosen by an exact name match first, then by downloads in the last 90 days ([search.rs#L537-L557](https://github.com/rust-lang/crates.io/blob/08dbe21957e5d9f105faf5798c4d6613628d0af1/src/controllers/krate/search.rs#L537-L557)) [A].

An exact name match also ranks first ([search.rs#L149](https://github.com/rust-lang/crates.io/blob/08dbe21957e5d9f105faf5798c4d6613628d0af1/src/controllers/krate/search.rs#L149)) [A]. So the word `assay` puts a crate named `assay` first, and `fork` puts `fork` first.

### Crates found that the hand research did not list

These came up in the first three ranks of `word` or `kw` for a need. They are listed, not judged or measured [A]:

| need | crates |
|---|---|
| S2 fork a process | `rusty-fork`, `fork` (both weighed and left out in [§ 5](https://github.com/kalonji-tools/purba/blob/8607015ff23c21b1c41fea9d17c6f693502f6679/research/future-crates.md#5-considered-and-left-out)), `sysinfo` |
| E1 capture output | `rusty-fork` in `full`; `capture` (an exact name) |
| D2 end a test that hangs | `ntest_timeout`, `process_control`, `timeout`, `timer-kit` |
| D4 run tests in parallel | `rayon`, `test-case`, `serial_test`. The hand research found D4 met by std and `libc` |
| R2 write JUnit XML | `merge-junit`, `xmlparser`, `roxmltree`, `facet-xml`, `cargo-nextest` |
| P2 more than one syntax error | `ariadne`, `miette`, `error-stack`. The hand research found P2 met by ruff |
| I4 globs | `camino`, `typed-path`, `globwalk`, `wax`, `glob` |

Most of the rest is noise: an exact name such as `read`, `file` or `run`, or a popular crate that carries a common keyword, such as `rstest` for "test".

## 4. Repeatability

| source | same query, two runs | what moves the result | tier |
|---|---|---|---|
| crates.io API | run 2 began 18 s after run 1 ended; each query repeated about 7.5 minutes later. All 262 queries: the same 50 crates, in the same order. `meta.total` changed in 73 queries (3 `full`, 70 `word`, 0 `kw`), each by +1 to +7 | Ties break by downloads in the last 90 days ([search.rs#L175-L180](https://github.com/rust-lang/crates.io/blob/08dbe21957e5d9f105faf5798c4d6613628d0af1/src/controllers/krate/search.rs#L175-L180)). The 1,000 candidates are chosen by the same count. That count is a materialized view over `version_downloads` "> date(CURRENT_TIMESTAMP - INTERVAL '90 days')" ([migration](https://github.com/rust-lang/crates.io/blob/08dbe21957e5d9f105faf5798c4d6613628d0af1/migrations/2019-03-22-162022_define_recent_crate_downloads_in_terms_of_version_downloads/up.sql)), refreshed by a background job; how often [C]. New crates, and edits to a README or keywords, also move it | [A] |
| crates.io dump | two runs of the same SQL on one dump: byte-identical output (`cmp`) | a new dump each 24 hours. A script that pins the dump's date repeats exactly | [A] |
| blessed.rs, awesome-rust, Rust Cookbook | a second fetch at the same commit: identical (blessed.rs, `cmp`) | a commit to the list. Pin the commit | [A] |
| manifests | a second fetch of uv's `Cargo.lock` at the same SHA: identical (`cmp`) | each project's commits, and the choice of projects. Pin the SHAs | [A] |
| GitHub search | one run only | not measured [C] | — |
| deps.dev, ecosyste.ms | not queried by words | — | — |

Recall in run 2 equals recall in run 1 for every mode [A]. A growing `meta.total` with a fixed top 50 means new matches entered below rank 50. For C1, the total went from 4,928 to 4,930; for Sel1, from 8,069 to 8,070 [A].

## 5. Doubts

### What could not be established [C]

| what | why |
|---|---|
| lib.rs recall | its `robots.txt` disallows `/search` and `/keywords/`. One search request was sent before `robots.txt` was read, in the same command. Its page was deleted unread |
| libraries.io recall | it needs an API key, which needs an account |
| deps.dev's rate limit | its docs state none that was found |
| the terms of blessed.rs, awesome-rust, the Rust Cookbook and the eight projects | not read |
| how often crates.io refreshes the 90-day download count | the job's schedule was not found |
| whether a tool driven by a language model proposes crates from a need's sentence | not searched |
| whether a person's rewording of each need closes the gap | not measured: every query here uses the need's own words |
| GitHub search repeatability | one run only |
| a lib.rs API or data dump | the about page names none, and none was found |

### Relayed and not re-verified [B]

- That GitHub's code search found every migration that names `trigger_crates_name_search`. If a later migration redefines it with other weights, the README weight could differ.
- That the search API's window is one minute. `gh api rate_limit` gives the limit (30), not the window's length.

### Other doubts

- **The matcher is a choice.** `score`, `blessed` and `awesome` use one simple matcher: count shared words. Another matcher, such as stemming or BM25, would give other numbers. The dump allows any matcher.
- **The stopword list is a choice.** It keeps purba's own terms, such as "assay" and "zygote", on purpose: they are the need's words.
- **The project list is a choice.** Eight projects were taken; another list gives another set. The list needs a person, or a rule.
- **`@10` is not comparable across modes.** `word` and `kw` merge one list for each word, so their pool at 10 is up to ten times the number of words.
- **blessed.rs files `serde_json` and `toml` under "Non-self-describing, external schema file"**, as its data has it [A]. This file reports the data as it is.
- **Run 1 and run 2 of crates.io were minutes apart, not days.** Movement over days was not measured.

## Appendix A: Commands

Scripts lived in the session's scratch directory, `$S` below; they are not committed. `UA='purba-research (https://github.com/kalonji-tools/purba)'`. Every crates.io request sent `User-Agent: purba-research (https://github.com/kalonji-tools/purba)`, at most one each second; the query runs waited 1.1 s.

### A1. The 41 and the 24 needs

```sh
git show origin/research/future-crates:research/future-crates.md   # § 1 and § 2, copied into $S/needs.tsv
# the 41 = the candidates in needs.tsv; checked equal to the crate rows of § 2:
grep -oE '^\| `[a-z0-9_-]+` [0-9]' future-crates.md | grep -oE '`[^`]+`' | tr -d '`' | sort -u | diff - the41.txt   # no output
```

### A2. Source documentation

```sh
gh api "repos/rust-lang/crates.io/contents/svelte/src/routes/data-access/+page.svelte?ref=08dbe21957e5d9f105faf5798c4d6613628d0af1" -H 'Accept: application/vnd.github.raw'
gh api "repos/rust-lang/crates.io/contents/src/controllers/krate/search.rs?ref=08dbe21957e5d9f105faf5798c4d6613628d0af1" -H 'Accept: application/vnd.github.raw'
gh api "repos/rust-lang/crates.io/contents/migrations/20160219125609_dumped_migration_105/up.sql?ref=08dbe21957e5d9f105faf5798c4d6613628d0af1" -H 'Accept: application/vnd.github.raw'
gh api -X GET search/code -f q='trigger_crates_name_search repo:rust-lang/crates.io' --jq '.items[].path'
curl -sS -A "$UA" https://crates.io/api/openapi.json | jq -r '.paths["/api/v1/crates"].get.parameters[].name'
curl -sS -A "$UA" https://index.crates.io/le/xo/lexopt | tail -1 | jq -c keys                     # cksum deps features name pubtime vers yanked
curl -sS -A "$UA" https://www.postgresql.org/docs/current/textsearch-controls.html          # plainto_tsquery
curl -sS -A "$UA" https://lib.rs/robots.txt ; curl -sS -A "$UA" https://lib.rs/about
curl -sS -A "$UA" https://docs.deps.dev/api/v3/ ; curl -sS -A "$UA" https://docs.deps.dev/api/v3alpha/
curl -sS -A "$UA" 'https://api.deps.dev/v3alpha/query?q=argument+parser'                       # 400 invalid query
curl -sS -A "$UA" 'https://api.deps.dev/v3alpha/systems/cargo/packages/clap:similarlyNamedPackages'   # "packages":[]
curl -sS -A "$UA" https://packages.ecosyste.ms/docs/api/v1/openapi.yaml ; curl -sS -A "$UA" https://ecosyste.ms/api
curl -sS -A "$UA" -D - 'https://packages.ecosyste.ms/api/v1/registries/crates.io/packages/lexopt/related_packages?per_page=20'
curl -sS -A "$UA" 'https://packages.ecosyste.ms/api/v1/keywords/junit?per_page=50'
curl -sS -A "$UA" https://libraries.io/api ; curl -sS -A "$UA" 'https://libraries.io/api/search?q=junit&platforms=Cargo'   # 401
gh api rate_limit --jq '.resources.search.limit'                                                # 30
gh api "repos/rust-lang/cargo/contents/src/doc/man/generated_txt/cargo-search.txt?ref=930d19d31c3090019d7394e29daf3dd28bc5abe7" -H 'Accept: application/vnd.github.raw'
gh api "repos/rust-lang/rfcs/contents/text/1824-crates.io-default-ranking.md?ref=f43d5789b838fb0d80962e3d4b45fb9ddfcc9a07" -H 'Accept: application/vnd.github.raw'
gh api "repos/crev-dev/cargo-crev/contents/crev-lib/rc/doc/editing-package-review.md?ref=e0e71f9dc2938c061ae0e99554caa8928d2299f1" -H 'Accept: application/vnd.github.raw'
```

### A3. The query words

```sh
# words.sh: lowercase; drop "'s"; split on [^a-z0-9]; drop these stopwords; keep the first copy of a word
STOP=" a an the of for by as to from its it each that and or on over before back whatever one more than under which inside does did not is are be with between into at in this these "
bash $S/scripts/words.sh $S/needs.tsv > $S/needwords.tsv    # 116 distinct words over 30 needs
```

### A4. crates.io API, two runs

```sh
# cio.sh RUN: 30 + 116 + 116 = 262 requests, 1.1 s apart; all answered 200
curl -sS -A "$UA" "https://crates.io/api/v1/crates?q=<need as written, URI-encoded>&per_page=50"
curl -sS -A "$UA" "https://crates.io/api/v1/crates?q=<word>&per_page=50"
curl -sS -A "$UA" "https://crates.io/api/v1/crates?keyword=<word>&sort=recent-downloads&per_page=50"
bash $S/scripts/cio2tsv.sh run1 > cio-run1.tsv ; bash $S/scripts/eval.sh cio-run1.tsv
bash $S/scripts/repeat.sh run1 run2
```

Run 1: 2026-11-15T07:30:05Z to 07:37:20Z. Run 2: 07:37:38Z to 07:45:01Z. Each: 262 answers, all `200`.

### A5. Manifests

```sh
# locks.sh: for each project, Cargo.lock at the default branch's head
gh api "repos/$r/contents/Cargo.lock?ref=$sha" -H 'Accept: application/vnd.github.raw'
# direct = dependencies of the lock's packages with no `source` line; all = packages with a `source` line
```

| project | SHA |
|---|---|
| astral-sh/uv | `d5c3f174417418c24ec3c276e01b0f62b961b13c` |
| astral-sh/ruff | `6bf61927fd328ff83b36ea6484a86dfedc53c340` |
| nextest-rs/nextest | `724cd7f904d8f2e128b62f1e5153332daf4de9c6` |
| PyO3/maturin | `fe3ae413bae518dd625479af01824e82e733cd83` |
| maelstrom-software/maelstrom | `9a29d875cba2acb2e623f6b4b7ff7385c6392d79` |
| MatthewMckee4/karva | `4a585218c4ba69cde3b680be8bd7e51370497de0` |
| hughhan1/rtest | `b9eddee626457fef3dbfd4ccbe425867cec4d20c` |
| thejchap/tryke | `39d2c592235953c92948f54355b9ff9407fd3bb0` |

### A6. The dump

```sh
curl -sSL -A "$UA" https://static.crates.io/db-dump.tar.gz | tar -xz --wildcards '*/data/crates.csv' '*/data/keywords.csv' '*/data/crates_keywords.csv' '*/data/categories.csv' '*/data/crates_categories.csv' '*/data/crate_downloads.csv' ...
# metadata.json: "timestamp": "2026-11-15T02:00:20.668843735Z", "crates_io_commit": "08dbe21957e5d9f105faf5798c4d6613628d0af1"
nix-shell -p duckdb --run 'duckdb dump.duckdb < scripts/load.sql'        # 378149 crates, 62155 keywords, 104 categories
nix-shell -p duckdb --run 'duckdb dump.duckdb < scripts/dumpsearch.sql'  # kw_or, score, score_m
nix-shell -p duckdb --run 'duckdb -csv dump.duckdb < scripts/shared.sql'    # shared content words of the 19 pairs never found
cmp dump-results.tsv dump-results.run1.tsv                                # identical
```

### A7. Lists

```sh
gh api "repos/nicoburns/blessed-rs/contents/data/crates.json?ref=6e317b7e22a5eed75832619054f3f6c39162c228" -H 'Accept: application/vnd.github.raw'
gh api "repos/rust-unofficial/awesome-rust/contents/README.md?ref=aa03ceba2c5dffa7d91bdab6602e21d8e46a4039" -H 'Accept: application/vnd.github.raw'
gh api "repos/rust-lang-nursery/rust-cookbook/contents/Cargo.toml?ref=555abf48f78cc8a388bf917cb2a755ecaf3109ea" -H 'Accept: application/vnd.github.raw'
bash $S/scripts/listscore.sh blessed blessed-entries.tsv ; bash $S/scripts/listscore.sh awesome awesome-entries.tsv
```

### A8. GitHub search

```sh
gh api -X GET search/repositories -f q="<content words> language:rust" -f per_page=50   # 2.5 s apart; 30 needs
```
