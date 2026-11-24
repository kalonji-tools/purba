# Which existing query language, or which crate, purba can use to select tests with one query

Research for [Which existing query language, or which crate, can purba use to select tests with one query?](https://github.com/kalonji-tools/purba/issues/1228)
Measured on 2026-11-24, against purba `main` at `3093871b`, on one Linux machine.
This file decides nothing. It lists ten query languages and fifteen crates, what each costs, and how each meets purba's scope.

## Summary

No language I probed takes purba's bare id `store::migrate` as a query for that test. Jujutsu's and Mercurial's revsets read `::` as a range of commits. Six other grammars refuse the id, and evalexpr reads it as a variable name. tantivy takes it only in quotes, as `id:"store::migrate"`. So no language crate fits purba as it is, and each would at most lend its grammar. A grammar written for purba keeps the id: one small grammar, in 24 to 46 lines in each of five parser crates, parsed `store::migrate` unchanged. Every language that states its precedence binds AND tighter than OR, except Bazel's query language. But only three languages state or set a limit on depth, and most of the others crashed on a deep query.

Six findings matter most for the choice.

- **Each language crate is bound to its own domain.** `nextest-filtering` compiles a filterset against a cargo package graph, and has a fixed set of predicates. `jj-dsl` parses a revset with no repository, but the caller resolves every name. `cfg-expr`, `gherkin` and `unselector` each parse one small grammar, with no id and no path. `cel` is a general language with arithmetic, strings and types. Section 1 has a row for each [run].
- **Most grammars bind AND tighter than OR, and two have traps around it.** nextest, jj, Mercurial, CEL, Cucumber, JUnit, evalexpr and tantivy all read `db | slow & !db` as `db | (slow & !db)` [run, source]. Bazel's query language gives its three operators one rank, from left to right [source]. In tantivy, a `NOT` beside `AND` builds a group that matches nothing. So `module:store AND tags:slow AND NOT tags:flaky` selected 0 tests [run].
- **Only jj, CEL and GitHub's issue search state or set a limit on depth.** jj refuses a query nested deeper than 256, and CEL deeper than 96 [run]. GitHub's documentation allows five levels of parentheses [source]. nextest, tantivy, Cucumber's crate, evalexpr, winnow and nom overflowed the stack, and Mercurial raised a `RecursionError` at 500 levels [run]. `pest` stops at its own stack guard with an error. `chumsky` grows the stack, and `lalrpop` uses an LR table, so both parsed 100000 levels [run].
- **Three crates gave a wrong or unchecked answer with no error.** `cfg-expr` 0.20.11 counts the functions in one `all()` or `any()` in a `u8`: with 256 of them, it selected 4 tests where the right answer is 3 or 6. `nextest-filtering`'s graph-free `ParsedExpr::parse` accepted `test(slow) & (test(db)`, which `cargo-nextest`'s own parse refuses. `evalexpr` parsed `slow &&`, and failed only when it evaluated [run].
- **Most crates point at the error, and three do not.** jj, Mercurial, CEL, `pest`, `lalrpop`, `chumsky`, `cfg-expr` and `gherkin` gave a line and column or an offset. tantivy and evalexpr gave none, and `nom` gave only the rest of the input [run]. Mercurial and jj suggest a near name of a function. No crate names the nearest test id, so purba writes that in every case (judgement).
- **Saving a query is a config entry in most languages, and reusing part of one is an alias.** nextest saves a filterset as `default-filter` in a profile. jj and Mercurial save named and parameterized aliases. JUnit, Cucumber and Bazel save an expression in a build file or a config property. GitHub saves up to 25 views. nextest and pytest run the failures again from a stored record, not from a printed query [source]. The words of the need `parse-test-query` find the five parser crates and `peg`, but none of the language crates [run].

## How to read this file

- **[run]** means I ran it in this session, on the version named. **[source]** means the language's code or documentation says so, at the pinned version, and I did not run it. **judgement** marks a classification that no source states.
- **The example** is the issue's query: "the slow tests of `store`, but not the flaky ones".
- **The precedence probe** is "db, or slow and not db". A language that binds AND tighter than OR selects 5 tests. A language that reads from left to right selects 3.
- **The id probe** gives purba's id `store::migrate` and the path `tests/api` as the whole query.
- **A nesting probe** wraps one predicate in n levels: n pairs of parentheses, n negations, or n nested `x & (x & (...))`. Each probe prints how far it got: "parsed", "evaluated", an error, or a stack overflow.
- **The probe suite** has six tests:

  | id | module | tags |
  |---|---|---|
  | `math::add` | math | none |
  | `math::add_slow` | math | slow |
  | `store::query` | store | db |
  | `store::migrate` | store | slow, db |
  | `store::load` | store | slow, flaky |
  | `store::save` | store | slow |

  The example selects `store::migrate` and `store::save`. A language with no tag carries the tags in the name (nextest), in the description (Mercurial), or as a field (tantivy), as each probe's script says.
- **[The predecessor](https://github.com/kalonji-tools/purba/blob/e362ebed60afd985672bd98149bb1954a1eda0a0/research/selection-queries.md)** of this research ran the runners' own languages: pytest, cargo-nextest 0.9.149, Vitest and oxitest's `-E`. This file does not run them again. It cites the predecessor's precedence defect in oxitest, and its depth crashes in four runners.

## Method

### Versions

Each crate is the newest stable version on crates.io on 2026-11-24, and each link pins the commit that its `.cargo_vcs_info.json` names. Two crates are older on purpose: `kube-core` 4.2.0 is the version that `unselector` 0.3.0 takes, and `peg` 0.6.3 is the version that `gherkin` 0.16.0 takes.

| language or crate | version | licence | ran on | pinned source |
|---|---|---|---|---|
| `nextest-filtering` | 0.23.0 | MIT OR Apache-2.0 | `nextest/` | [`11291595`](https://github.com/nextest-rs/nextest/tree/112915957de95b630b92aad52d546a82f3cabb3e/nextest-filtering) |
| `jj-dsl` (the parser of `jj-lib`) | 0.47.1 | Apache-2.0 | `jj/` | [`dsl/` at `v0.47.1`](https://github.com/jj-vcs/jj/tree/v0.47.1/dsl) |
| Mercurial | 7.1.2, from nixpkgs | GPL-2.0-or-later | `hg/` | [`revsetlang.py` at `7.1.2`](https://foss.heptapod.net/mercurial/mercurial-devel/-/blob/7.1.2/mercurial/revsetlang.py#L23-62) |
| `gherkin` (Cucumber's tag expressions) | 0.16.0 | MIT OR Apache-2.0 | `languages/` | [`aaa7d599`](https://github.com/cucumber-rs/gherkin/tree/aaa7d5994fce9a7fe40ff2b3fb60d7be5d88c76b) |
| `unselector` with `kube-core` 4.2.0 (label selectors) | 0.3.0 | Apache-2.0 | `languages/` | [`ffd7c24d`](https://github.com/crust-gather/unselector/tree/ffd7c24d150f0f8ae1c55b0e77bbb9ca337b0a5e) |
| `cfg-expr` (Cargo's `cfg()`) | 0.20.11 | MIT OR Apache-2.0 | `languages/` | [`0.20.11`](https://github.com/EmbarkStudios/cfg-expr/tree/0.20.11) |
| `cel` (CEL) | 0.16.0 | MIT | `languages/` | [`92af54eb`](https://github.com/cel-rust/cel-rust/tree/92af54eb79f3b6cc64791fa5c3806562b904ce67/cel) |
| `evalexpr` | 14.0.0 | AGPL-3.0-only | `languages/` | [`ec943faa`](https://github.com/ISibboI/evalexpr/tree/ec943faaac7299a1a09f8946f25020a1a7de0ff0) |
| `tantivy`, `tantivy-query-grammar` | 0.27.0 | MIT | `tantivy/` | [`0.27.0`](https://github.com/quickwit-oss/tantivy/tree/0.27.0) |
| `chumsky` | 0.14.2 | MIT | `parsers/` | `a4cccf14` on Codeberg |
| `winnow` | 1.0.5 | MIT | `parsers/` | [`21b3ab25`](https://github.com/winnow-rs/winnow/tree/21b3ab2535a675768a170a851efda83253b9eb21) |
| `nom` | 8.0.0 | MIT | `parsers/` | [`2cec1b3e`](https://github.com/rust-bakery/nom/tree/2cec1b3e4c9ccac62c902d60c00de6d1549ccbe1) |
| `pest`, `pest_derive` | 2.9.7 | MIT OR Apache-2.0 | `parsers/` | [`03e6e143`](https://github.com/pest-parser/pest/tree/03e6e143181f349edae53c0d3b914de30c13a5be) |
| `lalrpop`, `lalrpop-util` | 0.23.2 | Apache-2.0 OR MIT | `parsers/` | [`8abaee08`](https://github.com/lalrpop/lalrpop/tree/8abaee08d0751c9616448cab64e2b476717c66b4) |
| JUnit Platform tag expressions | 6.1.3, source only | EPL-2.0 | not run | [`tags.adoc` at `r6.1.3`](https://github.com/junit-team/junit-framework/blob/r6.1.3/documentation/modules/ROOT/pages/running-tests/tags.adoc#L33-L46) |
| Bazel query language | 9.3.3, source only | Apache-2.0 | not run | [`language.mdx` at `9.3.3`](https://github.com/bazelbuild/bazel/blob/9.3.3/docs/query/language.mdx#L307-L318) |
| GitHub's issue search | docs at `801373e8`, source only | | not run | [`filtering-and-searching-issues-and-pull-requests.md`](https://github.com/github/docs/blob/801373e82d10ab6b9a62b01aea68edc2ecdf633a/content/issues/tracking-your-work-with-issues/using-issues/filtering-and-searching-issues-and-pull-requests.md#L75-L99) |
| Lucene's classic query syntax | 10.6.0, source only | Apache-2.0 | not run | [`package-info.java` at `4ae946e5`](https://github.com/apache/lucene/blob/4ae946e5a9b99d1e22c23865c99e561b8dcc9d71/lucene/queryparser/src/java/org/apache/lucene/queryparser/classic/package-info.java#L277-L358) |

`chumsky`'s home is Codeberg, so the hook for GitHub permalinks does not check its link. Its source is `https://codeberg.org/zesterer/chumsky/src/commit/a4cccf14cb5f93f8b940afda8af2513ab6b02040`. Mercurial's home serves a check against bots to `curl`. So I cite Heptapod's mirror at the tag `7.1.2`.

`cel` 0.16.0 is the renamed `cel-interpreter`, whose last version is 0.10.0. Both name the same repository on crates.io [run]. `git-branchless-revset` 0.11.1 is a second revset crate, for git-branchless. I did not probe it.

### Probes

Each probe is a script under `research/selection-languages/<probe>/run.sh`. It copies its cargo project into an empty directory, builds it with `--locked`, and runs there. `output.txt` beside each script is its output from this session.

```sh
R=research/selection-languages
bash $R/languages/run.sh <empty dir>   # cfg-expr, cel, gherkin, unselector, evalexpr
bash $R/nextest/run.sh <empty dir>     # nextest-filtering
bash $R/jj/run.sh <empty dir>          # jj-dsl
bash $R/tantivy/run.sh <empty dir>     # tantivy
bash $R/parsers/run.sh <empty dir>     # chumsky, winnow, nom, pest, lalrpop
nix-shell -p mercurial --run "bash $R/hg/run.sh <empty dir>"
bash $R/needs-words.sh > $R/needs-words.tsv   # asks crates.io, one request a second
bash $R/sizes.sh > $R/sizes.tsv
```

- **A language crate's probe is the crate's parser, with an evaluator that is either the crate's or mine.** `cfg-expr`, `cel`, `unselector` with `kube-core`, `evalexpr` and tantivy evaluate with their own code. For `gherkin`, `jj-dsl` and `nextest-filtering`'s `ParsedExpr`, the probe walks the parsed tree with its own short evaluator. jj's functions `tag()` and `module()` are the probe's: jj resolves names to commits.
- **The parser crates run one grammar, which I wrote five times.** It is in [`parsers/src/main.rs`](selection-languages/parsers/src/main.rs): `|` is the lowest rank, `&` binds tighter, `!` tighter still, and an atom is `tag(name)`, an id or a parenthesized query. The probe only parses. It forgets the tree after a parse, so a deep tree's recursive drop does not count against the parser.
- **Every build is a release build.** An arithmetic overflow in a release build wraps, and in a debug build it panics. I did not build a debug probe.

## 1. The languages

### 1.1 Grammar and the example

| language | grammar | the example | source |
|---|---|---|---|
| nextest filtersets | predicates `test()`, `package()`, `binary()`, `kind()`, `platform()`, `deps()`, `rdeps()`, `group()`, `all()`, `none()`, `default()` with matchers `=`, `~`, `/regex/`, `#glob`; operators `not`/`!`, `and`/`&`, `-`, `or`/`\|`/`+`, `( )` | `test(store::) & test(slow) - test(flaky)` | [run] 2; [`reference.md` L73-L82](https://github.com/nextest-rs/nextest/blob/cargo-nextest-0.9.149/nextest-filtering/reference.md#L73-L82) |
| Jujutsu revsets | functions, symbols, `pattern:value`, `~x`, `x & y`, `x ~ y`, `x \| y`, ranges `::` and `..`, `( )` | `module(store) & tag(slow) ~ tag(flaky)`, with the probe's functions | [run] 2; [`revsets.md` L67-L110](https://github.com/jj-vcs/jj/blob/v0.47.1/docs/revsets.md#L67-L110) |
| Mercurial revsets | functions, symbols, `not`/`!`, `and`/`&`, `-`, `or`/`\|`/`+`, ranges `::` and `:` | `file("glob:store/*") and desc(slow) and not desc(flaky)` | [run] 2; [`revsetlang.py` L23-62](https://foss.heptapod.net/mercurial/mercurial-devel/-/blob/7.1.2/mercurial/revsetlang.py#L23-62) |
| Cucumber tag expressions | `@tag`, `not`, `and`, `or`, `( )`; `\` escapes a reserved character | `@store and @slow and not @flaky`, with the module as a tag | [run] 2; [`README.md` L28-L40](https://github.com/cucumber/tag-expressions/blob/v11.2.0/README.md#L28-L40) |
| JUnit tag expressions | a tag, `!`, `&`, `\|`, `( )`, `any()`, `none()` | `store & slow & !flaky`, with the module as a tag | [source]; [`tags.adoc` L33-L46](https://github.com/junit-team/junit-framework/blob/r6.1.3/documentation/modules/ROOT/pages/running-tests/tags.adoc#L33-L46) |
| Bazel query language | `word`, `let`, `intersect`/`^`, `union`/`+`, `except`/`-`, `set()`, functions such as `attr()`, `tests()` | `attr(tags, slow, tests(//store/...)) except attr(tags, flaky, //store/...)` | [source]; [`language.mdx` L307-L318](https://github.com/bazelbuild/bazel/blob/9.3.3/docs/query/language.mdx#L307-L318), [L935](https://github.com/bazelbuild/bazel/blob/9.3.3/docs/query/language.mdx#L935) |
| Kubernetes label selectors | requirements joined by `,`: `key=value`, `key!=value`, `key in (a,b)`, `key notin (a,b)`, `key`, `!key` | `module=store,slow,!flaky` | [run] 2; [`labels.md` L110-L133](https://github.com/kubernetes/website/blob/a4dbdd846fafabc4335db8a251c21831fe60ffe0/content/en/docs/concepts/overview/working-with-objects/labels.md#L110-L133), [`selector.go` L883-L895](https://github.com/kubernetes/apimachinery/blob/v0.37.2/pkg/labels/selector.go#L883-L895) |
| Cargo's `cfg()` | `all(...)`, `any(...)`, `not(x)`, a name, `key = "value"` | `all(module = "store", slow, not(flaky))` | [run] 2; [`conditional-compilation.md` L6-L64](https://github.com/rust-lang/reference/blob/c9cc32825b03fc206d9c6fe73e9ca6a2f3576d6c/src/conditional-compilation.md#L6-L64) |
| CEL | a C-like expression language: literals, variables, fields, functions, macros, `!`, `in`, `==`, `&&`, `\|\|`, `?:`, arithmetic | `module == "store" && "slow" in tags && !("flaky" in tags)` | [run] 2; [`langdef.md` L71-L92](https://github.com/google/cel-spec/blob/v0.26.0/doc/langdef.md#L71-L92) |
| Lucene, in tantivy | `field:term`, `"phrase"`, `AND`, `OR`, `NOT`, `+`, `-`, `( )`, ranges | `+module:store +slow -flaky` | [run] 2; [`query_parser.rs` L152-L157](https://github.com/quickwit-oss/tantivy/blob/0.27.0/src/query/query_parser/query_parser.rs#L152-L157) |
| GitHub's issue search | `qualifier:value`, a space as `AND`, `AND`, `OR`, `( )`, `-qualifier` | `label:store label:slow -label:flaky` | [source]; [L75-L99](https://github.com/github/docs/blob/801373e82d10ab6b9a62b01aea68edc2ecdf633a/content/issues/tracking-your-work-with-issues/using-issues/filtering-and-searching-issues-and-pull-requests.md#L75-L99) |

Every probed form of the example selected `store::migrate` and `store::save` [run], except the first tantivy form in section 2.2. In a row marked only [source], the example is mine, and I did not run it.

### 1.2 Precedence, depth, and a Rust crate

| language | precedence, strongest first | limit on depth | Rust crate | take it, or borrow the grammar | source |
|---|---|---|---|---|---|
| nextest filtersets | `( )`, `not`, `and`/`-`, `or`; left to right in a rank | none stated; overflowed at 10000 [run] | `nextest-filtering` | borrow: `Filterset::parse` needs a cargo `PackageGraph`, and the predicate set is fixed | [`reference.md` L73-L82](https://github.com/nextest-rs/nextest/blob/cargo-nextest-0.9.149/nextest-filtering/reference.md#L73-L82); [`expression.rs` L560-L570](https://github.com/nextest-rs/nextest/blob/112915957de95b630b92aad52d546a82f3cabb3e/nextest-filtering/src/expression.rs#L560-L570) |
| Jujutsu revsets | functions, `x-`/`x+`, patterns, ranges, `~x`, `&`/`~`, `\|`; left to right in a rank | 256, an estimate of the tree's depth: 254 parentheses or 127 nested `&` [run] | `jj-dsl` | the parser and the aliases are separable from commits; `::` is a range | [`revsets.md` L67-L71](https://github.com/jj-vcs/jj/blob/v0.47.1/docs/revsets.md#L67-L71); [`revset_parser.rs` L63](https://github.com/jj-vcs/jj/blob/v0.47.1/dsl/src/revset_parser.rs#L63), [L516](https://github.com/jj-vcs/jj/blob/v0.47.1/dsl/src/revset_parser.rs#L516) |
| Mercurial revsets | `not` 10, `and` and `-` 5, `or` 4 | none; `RecursionError` at 500 [run] | none; Mercurial's revsets are Python | borrow | [`revsetlang.py` L23-62](https://foss.heptapod.net/mercurial/mercurial-devel/-/blob/7.1.2/mercurial/revsetlang.py#L23-62) |
| Cucumber tag expressions | `not`, `and`, `or`; `and` and `or` left, `not` right | none found; `gherkin`'s parser overflowed at 10000 nested `and` [run] | `gherkin` (its `tagexpr`, used by `cucumber`) | take it for tags; it has no id and no path | [`TagExpressionParser.java` L14-L28](https://github.com/cucumber/tag-expressions/blob/v11.2.0/java/src/main/java/io/cucumber/tagexpressions/TagExpressionParser.java#L14-L28); [`parser.rs` L565-L573](https://github.com/cucumber-rs/gherkin/blob/aaa7d5994fce9a7fe40ff2b3fb60d7be5d88c76b/src/parser.rs#L565-L573) |
| JUnit tag expressions | `!` right, `&` left, `\|` left | none stated | none found | borrow | [`tags.adoc` L40-L46](https://github.com/junit-team/junit-framework/blob/r6.1.3/documentation/modules/ROOT/pages/running-tests/tags.adoc#L40-L46) |
| Bazel query language | `^`, `+` and `-` at one rank, left to right | none stated | none found | borrow | [`language.mdx` L442-L445](https://github.com/bazelbuild/bazel/blob/9.3.3/docs/query/language.mdx#L442-L445) |
| Kubernetes label selectors | AND only; no OR, no parentheses | the grammar is flat; a key or a value is at most 63 characters | `unselector` parses; `kube-core` matches | take it for AND only | [`labels.md` L113, L126](https://github.com/kubernetes/website/blob/a4dbdd846fafabc4335db8a251c21831fe60ffe0/content/en/docs/concepts/overview/working-with-objects/labels.md#L113-L126); [`selector.go` L883-L895](https://github.com/kubernetes/apimachinery/blob/v0.37.2/pkg/labels/selector.go#L883-L895) |
| Cargo's `cfg()` | prefix functions, so no precedence | none; 100000 levels parsed; 256 functions side by side gave a wrong answer [run] | `cfg-expr` | take its parser and evaluator; its predicates are a name and `key = "value"` | [`parser.rs` L30, L282, L320](https://github.com/EmbarkStudios/cfg-expr/blob/0.20.11/src/expr/parser.rs#L30) |
| CEL | `!` 2, relations and `in` 5, `&&` 6, `\|\|` 7, `?:` 8 | the spec asks for at least 12 recursive levels and 32 terms in a row; `cel` refuses depth 96 [run] | `cel` | take it; it is a general language | [`langdef.md` L54-L92](https://github.com/google/cel-spec/blob/v0.26.0/doc/langdef.md#L54-L92); [`parser.rs` L153](https://github.com/cel-rust/cel-rust/blob/92af54eb79f3b6cc64791fa5c3806562b904ce67/cel/src/parser/parser.rs#L153) |
| Lucene, in tantivy | tantivy: `AND` over `OR`; Lucene's page states no precedence | none stated; tantivy overflowed at 10000 [run] | `tantivy`, `tantivy-query-grammar` | take it only with an index of the suite | [`query_parser.rs` L152-L153](https://github.com/quickwit-oss/tantivy/blob/0.27.0/src/query/query_parser/query_parser.rs#L152-L153) |
| GitHub's issue search | not stated | five levels of parentheses | none | borrow | [L99](https://github.com/github/docs/blob/801373e82d10ab6b9a62b01aea68edc2ecdf633a/content/issues/tracking-your-work-with-issues/using-issues/filtering-and-searching-issues-and-pull-requests.md#L99) |

"None found" for a Rust crate means `cargo search` for "junit tag", "tag expressions" and "bazel query" listed no crate that parses the language [run]. A search finds only what its words match.

## 2. What the probes show

### 2.1 The id and the path as the whole query

purba's scope keeps `module::test::case` and its prefixes as a valid query. Each cell is [run].

| language crate | `store::migrate` | `tests/api` |
|---|---|---|
| `nextest-filtering` | refused: "expected expression" | refused by `Filterset::parse`: "missing `(`". `ParsedExpr::parse` read it as `test(s\/api)` |
| `jj-dsl` | parsed as `DagRange[store, migrate]`, a range of commits. `store::migrate::case` is a syntax error | an identifier |
| Mercurial | parsed as `(dagrange (symbol 'store') (symbol 'migrate'))`, then `abort: unknown revision 'store'` | a symbol |
| `gherkin` | refused: `error at 1:1: expected one of "(", "@", "not"` | refused, the same |
| `unselector` | refused: `':' at 5..6` | refused: `'tests/api' at 0..9` |
| `cfg-expr` | refused: "expected one of `<key>`, `all`, `any`, `not` here" | refused, the same |
| `cel` | refused: `mismatched input ':'` | parsed as `tests / api`, a division, then `UndeclaredReference("tests")` |
| `evalexpr` | parsed as one variable, then "Variable identifier is not bound" | parsed as a division |
| tantivy | refused: `Syntax Error: store::migrate` | a term of the default field; `id:"store::migrate"` in quotes selected the test |
| the probe grammar, in each of five parser crates | `id store::migrate` | `id tests/api` |

So a language crate keeps purba's id only behind a quote or a function, such as `id:"store::migrate"` (judgement). jj's and Mercurial's `::` collides with the id's separator.

### 2.2 Precedence

Each row is [run]. 5 means AND binds tighter than OR.

| language crate | query | tree | selected |
|---|---|---|---|
| `nextest-filtering` | `test(db) \| test(slow) & not test(db)` | `or[test(db), and[test(slow), not[test(db)]]]` | 5 |
| `jj-dsl` | `tag(db) \| tag(slow) & ~tag(db)` | `or[tag(db), and[tag(slow), not[tag(db)]]]` | 5 |
| Mercurial | `desc(db) or desc(slow) and not desc(db)` | `(or (list (func desc db) (and (func desc slow) (not (func desc db)))))` | 5 |
| `gherkin` | `@db or @slow and not @db` | `Or(Tag("db"), And(Tag("slow"), Not(Tag("db"))))` | 5 |
| `cel` | `"db" in tags \|\| "slow" in tags && !("db" in tags)` | not printed | 5 |
| `evalexpr` | `db \|\| slow && !db` | not printed | 5 |
| `cfg-expr` | `any(db, all(slow, not(db)))` | prefix; no precedence | 5 |
| tantivy | `db OR slow AND NOT db` | `(?db ?(+slow +(-db)))` | 2 |
| tantivy | `db OR (slow -db)` | `(?db ?(*slow -db))` | 5 |
| each parser crate, the probe grammar | `tag(db) \| tag(slow) & !tag(db)` | `or[tag db, and[tag slow, not[tag db]]]` | not evaluated |

tantivy reads AND before OR, as its documentation says. But `AND NOT db` becomes a group that holds only a `MustNot` clause, and that group matches no document. So `slow AND NOT db` selects nothing, and the row selects only the 2 db tests. The example `module:store AND tags:slow AND NOT tags:flaky` selected 0 tests for the same reason. `+module:store +slow -flaky` selected the right 2. Lucene's own page warns that "The NOT operator cannot be used with just one term" ([`package-info.java` L330-L333](https://github.com/apache/lucene/blob/4ae946e5a9b99d1e22c23865c99e561b8dcc9d71/lucene/queryparser/src/java/org/apache/lucene/queryparser/classic/package-info.java#L330-L333)) [source].

The predecessor found that oxitest's `-E` gives `&` and `|` one rank, and selected 1 test where pytest, cargo-nextest and Vitest selected 3 ([section 2.2](https://github.com/kalonji-tools/purba/blob/e362ebed60afd985672bd98149bb1954a1eda0a0/research/selection-queries.md#22-one-expression-two-readings)). Bazel's query language does the same on purpose, and its page says "you might want parentheses" [source].

### 2.3 An error at a position

Each cell is the crate's own text for a missing operand and for an unclosed parenthesis [run]. The query is in each crate's syntax, such as `tag(slow) &` and `tag(slow) & (tag(db)`.

| crate | a missing operand | an unclosed parenthesis | position given |
|---|---|---|---|
| `nextest-filtering`, `Filterset::parse` | `× expected expression`, the query drawn with no mark | `× expected close parenthesis`, no mark | a span; empty at the end of the query |
| `nextest-filtering`, `ParsedExpr::parse` | the same | **no error**: it parsed `and[test(slow), test(db)]` and selected 1 test | |
| `jj-dsl` | `--> 1:12`, a caret, `= expected '::', '..', '~', or <primary>` | `--> 1:21`, `= expected '-', '+', '::', '..', '\|', '&', or '~'` | line and column |
| Mercurial | `hg: parse error at 14: not a prefix: end`, with `^ here` | `hg: parse error at 24: unexpected token: end` | an offset |
| `gherkin` | `error at 1:10: expected one of "(", "@", "not"` | `error at 1:15: expected one of ")", "and", "or", _` | line and column |
| `unselector` | `failed to parse value as expression: '!' at 5..6` | `'in' at 7..9`, the token before the set | a byte range |
| `cfg-expr` | `- unclosed parens` under `all(` | the same | a span, drawn |
| `cel` | `ERROR: <input>:1:18: Syntax error: mismatched input '<EOF>' expecting {...}`, a caret | `ERROR: <input>:1:32: Syntax error: missing ')' at '<EOF>'` | line and column |
| `evalexpr` | parsed; the evaluation failed: `An operator expected 2 arguments, but got 1.` | `Found an unmatched opening parenthesis '('.` | none |
| tantivy | `Syntax Error: slow AND` | `Syntax Error: slow AND (db` | none: the whole query |
| `chumsky` | `at 11..11: found end of input expected '!', 't', any, or '('` | `at 20..20: found end of input expected '&', '\|', or ')'` | a byte range |
| `winnow` | the query and a caret under the space before `&`, with no message | the same caret, under the same space | a column, at the end of the last good parse |
| `nom` | `Error(Error { input: "&", code: Eof })` | `Error(Error { input: "& (tag(db)", code: Eof })` | the rest of the input |
| `pest` | `--> 1:12`, a caret, `= expected not, tag, or id` | `--> 1:14`, `= expected not` | line and column; the second points inside the group |
| `lalrpop` | `Unrecognized EOF found at 11` and the expected tokens | `Unrecognized EOF found at 20`, `Expected one of ")" or "\|"` | an offset |

The text of `winnow`, `nom`, `chumsky` and `pest` is what a grammar gets with no work on its errors. Each crate lets the grammar add context, such as `winnow`'s `cut_err` and `StrContext`. I did not add it, so the table shows the floor (judgement).

**`nextest-filtering` has two parses, and only one checks.** `ParsedExpr::parse` needs no package graph. It returns the tree whenever the parser recovered, and drops the errors it collected ([`parsing.rs` L134-L141](https://github.com/nextest-rs/nextest/blob/112915957de95b630b92aad52d546a82f3cabb3e/nextest-filtering/src/parsing.rs#L134-L141)). `Filterset::parse` returns those errors ([`expression.rs` L569-L570](https://github.com/nextest-rs/nextest/blob/112915957de95b630b92aad52d546a82f3cabb3e/nextest-filtering/src/expression.rs#L569-L570)) [source]. The probe ran both [run].

**A near name.** Mercurial answered `desk(slow)` with `hg: parse error: unknown identifier: desk` and `(did you mean desc?)` [run]. jj-dsl holds the same helper, `collect_similar` ([`dsl_util.rs` L1018](https://github.com/jj-vcs/jj/blob/v0.47.1/dsl/src/dsl_util.rs#L1018)) [source]. Both suggest a function name, not a test.

### 2.4 Deep nesting

Each cell is [run]. "ok" means the probe parsed and, where it evaluates, evaluated. A crate's own refusal is an error with a message. An overflow aborts the process with exit 134.

| crate | n pairs of parentheses | n negations | n nested `x & (...)` |
|---|---|---|---|
| `nextest-filtering` | ok to 3000; overflow at 10000 | ok to 3000; overflow at 10000 | ok to 3000; overflow at 10000 |
| `jj-dsl` | ok to 254; "Expression nesting limit exceeded" from 255, and at 10000 | ok to 254 | ok to 127; "stack limit reached" at 10000 |
| Mercurial | ok to 300; `RecursionError` at 500 | not probed | not probed |
| `gherkin` | ok to 10000; overflow in the parse at 30000 | ok to 10000; overflow in the parse at 30000 | ok to 1000; overflow in the parse at 10000 |
| `cfg-expr` | `all(` ok to 100000 | `not(` ok to 100000 | not probed |
| `cel` | "Recursion limit of 96 exceeded" from 100 to 60000; from 70000, a panic while the error is printed | `!` ok to 100000 | "Recursion limit of 96 exceeded" at 100 |
| `evalexpr` | ok to 30000; overflow in the evaluation at 100000 | ok to 30000; overflow in the evaluation at 100000 | ok to 10000; overflow in the evaluation at 30000 |
| tantivy | ok to 3000; overflow at 10000 | not probed | `(slow AND ...)` ok to 3000; overflow at 10000 |
| `chumsky` | ok to 100000 | ok to 100000 | not probed |
| `winnow` | ok to 3000; overflow at 10000 | ok to 10000; overflow at 30000 | not probed |
| `nom` | ok to 10000; overflow at 30000 | ok to 30000; overflow at 100000 | not probed |
| `pest` | parsed to 10000; "stack limit reached" at 30000 | parsed to 30000; "stack limit reached" at 100000 | not probed |
| `lalrpop` | ok to 100000 | ok to 100000 | not probed |

- **jj counts depth before it parses.** It estimates the depth of the tree from pest's pairs, and refuses past 256 ([`revset_parser.rs` L516-L518](https://github.com/jj-vcs/jj/blob/v0.47.1/dsl/src/revset_parser.rs#L516-L518)). A comment in its grammar says an earlier form re-parsed an operand on each failed match, "which is exponential in the nesting depth" ([`revset.pest`](https://github.com/jj-vcs/jj/blob/v0.47.1/dsl/src/revset.pest)) [source].
- **CEL's limit is the crate's choice.** `cel` sets `max_recursion_depth: 96` and lets the caller change it ([`parser.rs` L153-L167](https://github.com/cel-rust/cel-rust/blob/92af54eb79f3b6cc64791fa5c3806562b904ce67/cel/src/parser/parser.rs#L153-L167)). The spec asks only for "12 repetitions of recursive rules" ([`langdef.md` L54-L69](https://github.com/google/cel-spec/blob/v0.26.0/doc/langdef.md#L54-L69)) [source]. At 70000 and more pairs of parentheses, printing `cel`'s error panicked with "Formatting argument out of range" [run]. Its `Display` pads the caret to the error's column ([`parser.rs` L124](https://github.com/cel-rust/cel-rust/blob/92af54eb79f3b6cc64791fa5c3806562b904ce67/cel/src/parser/parser.rs#L124)), and Rust refuses a width above 65535 (judgement, from the column and the message).
- **pest's guard returns an error.** pest 2.9.7 checks the native stack and fails the parse with "stack limit reached" ([`parser_state.rs` L107-L109](https://github.com/pest-parser/pest/blob/03e6e143181f349edae53c0d3b914de30c13a5be/pest/src/parser_state.rs#L107-L109)) [source]. The probe's own tree builder for pest is recursive. It overflowed after pest parsed 10000 parentheses and 30000 negations, so those cells name the parse only [run].
- **chumsky grows the stack, so it has no limit at all.** Its default feature `stacker` grows the stack inside a recursive parser (`src/recursive.rs` L173 at `a4cccf14`) [source]. A grammar on `chumsky` or `lalrpop` still needs its own limit, or it takes any depth that memory allows (judgement).
- **tantivy drew a defect of the same kind.** Its issue [#2498](https://github.com/quickwit-oss/tantivy/issues/2498) found the parse time doubling with each level of parentheses. A regression test now parses 60 levels ([`query_grammar.rs` L1948-L1959](https://github.com/quickwit-oss/tantivy/blob/50ac63df66084b8f77bfd067b81b5b8076b8a905/query-grammar/src/query_grammar.rs#L1948-L1959)) [source].

### 2.5 A wrong answer with no error: `cfg-expr` and 256 functions

`cfg-expr` counts the functions inside one `all()` or `any()` in a `u8` ([`parser.rs` L30](https://github.com/EmbarkStudios/cfg-expr/blob/0.20.11/src/expr/parser.rs#L30), [L282](https://github.com/EmbarkStudios/cfg-expr/blob/0.20.11/src/expr/parser.rs#L282), [L320](https://github.com/EmbarkStudios/cfg-expr/blob/0.20.11/src/expr/parser.rs#L320)) [source]. The probe put n copies of `not(flaky)` and then `slow` into one function [run]:

| query | n | selected | right answer |
|---|---|---|---|
| `any(not(flaky), …, slow)` | 255 | 6 | 6 |
| `any(not(flaky), …, slow)` | 256 | 4 | 6 |
| `all(not(flaky), …, slow)` | 255 | 3 | 3 |
| `all(not(flaky), …, slow)` | 256 | 4 | 3 |

No error came. A tester writes no such query by hand. A script that builds one from a list of ids can (judgement).

## 3. Crates that reduce the work of a language written for purba

### 3.1 Parser crates

The size is [run] by `sizes.sh`: every line of the crate's module in the probe, plus its grammar file. The probe grammar has four rules and two kinds of atom.

| crate | what it does for purba | what purba still writes | lines of the probe grammar | operator helper | its error, with no work | source |
|---|---|---|---|---|---|---|
| `chumsky` 0.14.2 | combinators in Rust; error recovery; spans | the grammar, the tree, a depth limit, the messages | 24 | `pratt`, `src/lib.rs` L2418 at `a4cccf14` | a byte range and the expected tokens | [run]; [source] |
| `winnow` 1.0.5 | combinators in Rust; `cut_err` and context for messages | the grammar, the tree, a depth limit, the messages | 40 | `combinator::expression` ([`expression.rs` L76](https://github.com/winnow-rs/winnow/blob/21b3ab2535a675768a170a851efda83253b9eb21/src/combinator/expression.rs#L76)) | the query and a caret, no message | [run]; [source] |
| `nom` 8.0.0 | combinators in Rust | the grammar, the tree, a depth limit, the position, the messages | 43 | none in `nom` itself | the rest of the input and an error kind | [run] |
| `pest` 2.9.7 | a PEG grammar file, compiled by a derive; a stack guard | the walk from pairs to a tree, the messages beyond "expected" | 46 | `PrattParser` ([`pratt_parser.rs` L207](https://github.com/pest-parser/pest/blob/03e6e143181f349edae53c0d3b914de30c13a5be/pest/src/pratt_parser.rs#L207)) | line, column, caret and expected rules | [run]; [source] |
| `lalrpop` 0.23.2 | an LR(1) grammar file, compiled by a build script; precedence by the shape of the rules | a build script, the messages | 37 | the grammar's rule levels | an offset and the expected tokens | [run] |

All five parsed the id `store::migrate` and the path `tests/api` as one atom, because the probe grammar's id takes `:` and `/` [run]. jj's revsets use `pest` with its `PrattParser` ([`revset_parser.rs` L553-L568](https://github.com/jj-vcs/jj/blob/v0.47.1/dsl/src/revset_parser.rs#L553-L568)) [source].

### 3.2 Crates that evaluate a boolean expression

| crate | version, licence | what it does for purba | what purba still writes | its error at a position | source |
|---|---|---|---|---|---|
| `cfg-expr` | 0.20.11, MIT OR Apache-2.0 | parses `all`/`any`/`not` over names and `key = "value"`, and evaluates by a closure | the meaning of each name; the id and the path outside the grammar | a drawn span | [run] |
| `evalexpr` | 14.0.0, AGPL-3.0-only | parses and evaluates `&&`, `\|\|`, `!` over typed variables, with arithmetic and assignment | a variable for each tag; the id outside the grammar | none | [run]; [`lib.rs` L118-L157, L576](https://github.com/ISibboI/evalexpr/blob/ec943faaac7299a1a09f8946f25020a1a7de0ff0/src/lib.rs#L118-L157) |
| `boolean_expression` | 0.4.4, MIT | builds and simplifies an `Expr` and BDDs | the whole parser: the crate has no parse function | not applicable | [run], `grep` for `fn parse` and `FromStr` in its source found none |
| `whereexpr` | 0.3.1, MIT | named conditions on a record's fields, joined by `&&`, `\|\|`, `!` | the attributes of a test | not probed | [source]; its `README.md` at 0.3.1 on crates.io |

purba's `.cargo/deny.toml` allows MIT, and a licence that one crate needs is an exception for that crate ([L3-L7](https://github.com/kalonji-tools/purba/blob/3093871b/.cargo/deny.toml#L3-L7)). So `jj-dsl`, `unselector` and `kube-core` (Apache-2.0) and `evalexpr` (AGPL-3.0-only) would each need an exception [source]. The other crates here offer MIT [source].

## 4. How each language saves a query and runs it again

| language | saves a query as | reuses a part of a query | runs the failures again | source |
|---|---|---|---|---|
| nextest | `default-filter` in a profile of `.config/nextest.toml`, for example `[profile.ci]` `default-filter = 'all()'` | no alias | `cargo nextest run -R latest` reruns from a recorded run; experimental | [`selecting.md` L99-L105](https://github.com/nextest-rs/nextest/blob/cargo-nextest-0.9.149/site/src/docs/selecting.md#L99-L105); [`rerun.md` L9-L13](https://github.com/nextest-rs/nextest/blob/cargo-nextest-0.9.149/site/src/docs/features/record-replay-rerun/rerun.md#L9-L13) |
| Jujutsu | `revsets.log` and other keys in the config | `[revset-aliases]`, with parameters: `'user(x)' = 'author(x) \| committer(x)'` | not applicable | [`revsets.md` L596-L602](https://github.com/jj-vcs/jj/blob/v0.47.1/docs/revsets.md#L596-L602); [`config.md` L682-L688](https://github.com/jj-vcs/jj/blob/v0.47.1/docs/config.md#L682-L688) |
| Mercurial | an alias in the config | `[revsetalias]`, with parameters: `d(s) = sort(s, date)` | not applicable | [`revisions.txt` L162-165](https://foss.heptapod.net/mercurial/mercurial-devel/-/blob/7.1.2/mercurial/helptext/revisions.txt#L162-165), [`config.txt` L2201-2204](https://foss.heptapod.net/mercurial/mercurial-devel/-/blob/7.1.2/mercurial/helptext/config.txt#L2201-2204) |
| Cucumber, on the JVM | `cucumber.filter.tags=` in the JUnit Platform's properties | no alias | not applicable | [`README.md` L476-L479](https://github.com/cucumber/cucumber-jvm/blob/v8.1.0/cucumber-junit-platform-engine/README.md#L476-L479) |
| JUnit | Maven Surefire `<groups>acceptance \| !feature-a</groups>`; Gradle `includeTags(...)` | no alias | not applicable | [`build-support.adoc` L389-L390](https://github.com/junit-team/junit-framework/blob/r6.1.3/documentation/modules/ROOT/pages/running-tests/build-support.adoc#L389-L390) |
| Bazel | a flag under a named config in `.bazelrc`, used by `--config=name` | `let name = expr in expr`, inside one query | not applicable | [`bazelrc.mdx` L236-L239](https://github.com/bazelbuild/bazel/blob/9.3.3/docs/run/bazelrc.mdx#L236-L239); [`language.mdx` L307-L318](https://github.com/bazelbuild/bazel/blob/9.3.3/docs/query/language.mdx#L307-L318) |
| GitHub's issue search | a saved view, up to 25 | no alias | not applicable | [`viewing-all-of-your-issues-and-pull-requests.md` L33-L44](https://github.com/github/docs/blob/801373e82d10ab6b9a62b01aea68edc2ecdf633a/content/issues/tracking-your-work-with-issues/using-issues/viewing-all-of-your-issues-and-pull-requests.md#L33-L44) |
| pytest | not applicable | not applicable | `--lf` reruns from `.pytest_cache` | [`cache.rst` L13-L18](https://github.com/pytest-dev/pytest/blob/9.2.0/doc/en/how-to/cache.rst#L13-L18); [`reference.rst` L1379-L1385](https://github.com/pytest-dev/pytest/blob/9.2.0/doc/en/reference/reference.rst#L1379-L1385) |

The probe ran the aliases of jj and Mercurial [run]. `stable(slow_store)`, with `slow_store` the example's first two picks and `stable(x)` the third, selected the example's 2 tests in both.

- I found no runner here that prints a query to rerun its failures. nextest and pytest each rerun from a record that they wrote (judgement, from the sources above). nextest prints a hint to continue a rerun ([changelog L128](https://github.com/nextest-rs/nextest/blob/cargo-nextest-0.9.149/site/src/changelog.md#L128)) [source].
- A rerun from a record is itself a surface. nextest fixed an advisory in 0.9.150: "`cargo nextest run --rerun` with a malicious portable recording could run arbitrary code" ([changelog L61](https://github.com/nextest-rs/nextest/blob/cargo-nextest-0.9.152/site/src/changelog.md#L61)) [source].
- A failure set as a query would be a list of ids joined by OR. Only a grammar that keeps the id, and that has a stated limit on width and depth, could hold a long one (judgement, from sections 2.1, 2.4 and 2.5).

## 5. Each language against purba's scope

Each column is one line of the issue's scope. "yes" means the language meets it as it is. "purba writes" means purba must add it in any form. "no" means the language's grammar works against it. Each cell is a judgement from sections 1 to 4, and the row's evidence is there.

| language | `store::migrate` stays a query | a path part runs in stage one | an empty pick names the nearest id | a tag is a literal | precedence stated | depth limit stated |
|---|---|---|---|---|---|---|
| nextest filtersets | no: refused | yes in kind: `matches_binary` answers yes, no or unknown before the tests are listed | purba writes | no: `test(x)` matches a substring by default | yes | no |
| Jujutsu revsets | no: `::` is a range | purba writes: the caller resolves every name | purba writes; jj suggests function names | yes, as a function argument | yes | yes, 256 |
| Mercurial revsets | no: `::` is a range | purba writes | purba writes | yes, as a function argument | in code only | no |
| Cucumber tag expressions | no: only `@tag` | no path in the grammar | purba writes | yes | in the Java parser only | no |
| JUnit tag expressions | no: only tags | no path in the grammar | purba writes | yes | yes | no |
| Bazel query language | no: a label is `//pkg:name` | yes in kind: a query reads the build graph, not the test files | purba writes | through `attr(tags, regex)`, a pattern | yes: one rank | no |
| Kubernetes label selectors | no: refused | no path in the grammar | purba writes | yes | not applicable: AND only | not applicable: flat |
| Cargo's `cfg()` | no: refused | purba writes | purba writes | yes | not applicable: prefix | no; a width defect at 256 |
| CEL | no: refused | purba writes | purba writes | yes, as a string | yes | yes, 96 in `cel` |
| Lucene, in tantivy | only quoted: `id:"store::migrate"` | purba writes | purba writes | yes, with a `STRING` field | in tantivy only | no |
| a grammar written for purba | yes, by its own id rule | purba writes | purba writes | purba writes | purba writes | purba writes |

Stage one reads paths before it parses any file ([`LANDSCAPE.md` L141](https://github.com/kalonji-tools/purba/blob/3093871b/LANDSCAPE.md#L141), [L58](https://github.com/kalonji-tools/purba/blob/3093871b/LANDSCAPE.md#L58)). nextest has the closest design in this set. A filterset first decides each test binary as yes, no or unknown ([`expression.rs` L219-L225](https://github.com/nextest-rs/nextest/blob/112915957de95b630b92aad52d546a82f3cabb3e/nextest-filtering/src/expression.rs#L219-L225)), and nextest does not run a binary that the filterset always excludes ([`reference.md` L41-L54](https://github.com/nextest-rs/nextest/blob/112915957de95b630b92aad52d546a82f3cabb3e/nextest-filtering/reference.md#L41-L54)) [source]. A query whose predicates name their kind, such as `path()` beside `tag()`, lets a parser split the two stages. A query of bare words leaves the split to a rule on the word (judgement).

A query language joins purba's public surface, which starts empty ([`purbas-public-surface-starts-empty-and-has-no-plugin-api.md`](https://github.com/kalonji-tools/purba/blob/3093871b/docs/decisions/purbas-public-surface-starts-empty-and-has-no-plugin-api.md#decision-outcome)). A borrowed grammar brings its operators onto that surface too. A crate is weighed by its relief and its pain ([`a-crate-joins-when-its-relief-exceeds-its-pain.md`](https://github.com/kalonji-tools/purba/blob/3093871b/docs/decisions/a-crate-joins-when-its-relief-exceeds-its-pain.md#decision-outcome)). This file does not score either.

## 6. Whether the need's words find these crates

The need `parse-test-query` has the words `parser-combinators`, `grammar` and `combinator` ([`.config/needs.toml` L10](https://github.com/kalonji-tools/purba/blob/3093871b/.config/needs.toml#L10)). `needs-words.sh` asks crates.io with the matcher of `scripts/forecast.sh`: for each word, `keyword=<word>&sort=recent-downloads&per_page=20`. A crate is found when one word's list holds it. The rows are [run], in `needs-words.tsv`.

| crate | found | by the word | its own keywords |
|---|---|---|---|
| `chumsky` | yes | `combinator` | combinator, language, parser, syntax, token |
| `winnow` | yes | `parser-combinators` | bit, parser, parser-combinators, parsing, streaming |
| `nom` | yes | `parser-combinators` | the same as `winnow` |
| `pest` | yes | `grammar` | grammar, parser, peg, pest |
| `lalrpop` | yes | `grammar` | generator, grammar, lr, parser, yacc |
| `peg` | yes | `grammar` | grammar, parser, parsing, peg |
| `nextest-filtering` | no | | nextest, test-runner |
| `jj-dsl`, `jj-lib` | no | | dvcs, git, mercurial, scm, vcs |
| `cfg-expr` | no | | cargo, cfg, rustc |
| `cel` | no | | none |
| `gherkin` | no | | bdd, cucumber, cucumber-gherkin, gherkin, testing |
| `unselector` | no | | k8s, kubernetes, parser, selector |
| `kube-core` | no | | apimachinery, kubernetes |
| `tantivy`, `tantivy-query-grammar` | no | | information, retrieval, search |
| `evalexpr` | no | | arithmetic, boolean, evaluate, evaluator, expression |
| `boolean_expression` | no | | none |
| `whereexpr` | no | | expression, filtering, predicate, query, rules |

So the forecast, with these words, would vet the parser crates and no language crate. A word such as `expression`, `query` or `boolean` would add `evalexpr` and `whereexpr` by their keywords. This research did not run the forecast, which builds each crate it finds.

## Appendix: the files of this research

| file | holds |
|---|---|
| `research/selection-languages/<probe>/run.sh` | the probe; `output.txt` beside it is its output in this session |
| `research/selection-languages/<probe>/Cargo.toml`, `Cargo.lock`, `src/` | the cargo project of a probe; the lock pins each crate's version |
| `research/selection-languages/parsers/src/grammar/` | the probe grammar for `pest` and for `lalrpop` |
| `research/selection-languages/hg/run.sh`, `output.txt` | the probe of Mercurial's revsets |
| `research/selection-languages/needs-words.sh`, `needs-words.tsv` | whether the need's words find each crate |
| `research/selection-languages/sizes.sh`, `sizes.tsv` | the lines of the probe grammar in each parser crate |
