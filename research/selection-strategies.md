# Strategies that find the units a system acts on, and how each fits purba's test files and tests

Research for [Which strategies do systems use to find the units they act on, and which strategy fits purba's test files and tests?](https://github.com/kalonji-tools/purba/issues/1227)
Measured on 2026-11-24, against purba `main` at `3093871b`, on one Linux machine.
This file decides nothing. It names eight strategies, surveys systems that use each, and weighs each strategy against purba's scope.
It builds on [the findings of #1225](https://github.com/kalonji-tools/purba/blob/93af24dfe1cf5f84151106ee625b5b7e09a0da6e/research/selection-test-markers.md), and does not repeat its counts.

## Summary

Most systems surveyed find their units with two or more strategies. Most read a coarse marker first, such as a folder or a file name. Then they read a fine marker on each unit, such as a name, an attribute or a call. Cargo, pytest and TypeScript add a default convention that an explicit list can replace. Almost every system is silent when a unit lacks its marker. A few systems see a marker that is almost right, and report it. An explicit list is the opposite: it reports a wrong entry, and it is silent about a missing one.

Six findings matter most for purba.

- **Most systems combine strategies.** Cargo combines a folder, an attribute, a list and a default that the list can replace. Django combines a list, a folder, a file name and a call. Next.js, SvelteKit and Hugo combine a folder with a reserved file name. Section 2 lists 14 systems and their combinations. Flask's routes and Python's entry points each use one strategy.
- **A list reports a wrong entry, and is silent about a missing one.** A `[[test]]` that names a missing file stops Cargo [run]. A missing file in `tsconfig.json`'s `files` gives `TS6053` [run]. A name in `__all__` that the module does not bind raises `AttributeError` [run]. With `autotests = false`, Cargo silently skips `tests/unlisted.rs` [run]. Django leaves out the commands of an app that `INSTALLED_APPS` does not name; a call gives only `Unknown command` [run].
- **A missing marker is silent, except where a tool checks for a near miss.** `go test` fails on `Testlower` and on `TestWrong(n int)` [run]. SvelteKit prints `Missing route file prefix. Did you mean +page.svelte?` for `page.svelte` [source]. Sass refuses `@use "colors"` when `_colors.scss` and `colors.scss` both exist [run]. rustc warns that a test function with no `#[test]` is never used [run]. Jekyll drops a post whose name has no date, with no line [run]. Node's runner never runs a test file outside its globs [run].
- **A marker by place turns each helper in the place into a unit, in non-test systems too.** Node's runner runs `test/helper.js` as a test file and reports it as passed [run]. Django lists `helper.py` in `management/commands/` as a command, and fails only when it is called [run]. #1225 found the same in Cargo and Jest. Hugo is the exception: `index.md` makes each other file of its folder a resource, not a page [run].
- **An exclusion glob does not stop a unit that another unit reaches, or one named on the command line.** `tsc` compiles `src/skip/b.ts` under `exclude: ["src/skip"]`, since `src/a.ts` imports it [run]. Ruff checks an excluded file named on the command line, unless `force-exclude` is set [run]. A glob that matches nothing is an error in `tsc` [run] and in Bazel 9.3.3 [source].
- **In the 9 suites, no test is made by a loop, and few settings override pytest's defaults.** The 189 test files hold no module-level loop, `globals()` or `setattr` that makes a test. 4 `def test…` sit under `if six.PY3:` and its `else:`, which purba refuses today. 5 suites set `testpaths`, 1 adds a `python_files` pattern, and none sets `python_functions` or `python_classes`. A list of the files that the suites' own commands collect has 175 entries, in 15 folders.

judgement: purba's two stages match the pattern that most systems share. Stage one reads a coarse marker from the path. Stage two reads a fine marker from the tree. Section 3 weighs each strategy against each constraint of the issue.

## How to read this file

- **[run]** means I ran it in this session, on the version named. **[source]** means the tool's code or documentation says so, at the pinned version or commit, and I did not run it. **judgement** marks a claim that no source states.
- A **unit** is what a system acts on: a test, a test file, a page, a command, a target, a bean. A **marker** is what a system reads to decide that a thing is a unit. A **helper** is a thing beside the units that is not a unit.
- **#1225** is [the findings of #1225](https://github.com/kalonji-tools/purba/blob/93af24dfe1cf5f84151106ee625b5b7e09a0da6e/research/selection-test-markers.md). A row that cites it reuses its [run] or its count.
- A **test file** in the counts is a `.py` file whose name matches `test_*.py` or `*_test.py`, anywhere in the suite, as in #1225. There are 189.
- A count answers only its own matcher. Each table row names its matcher.

## Method

### Versions

| tool | version | how I ran it | pinned source |
|---|---|---|---|
| Cargo, rustc | cargo 1.99.0 | this machine, `--offline` | [`cargo-targets.md` at `0.100.0`](https://github.com/rust-lang/cargo/blob/0.100.0/doc/book/src/reference/cargo-targets.md#L316-L327) |
| Go | go 1.26.10 | `nix-shell -p go` | [`go_spec.html`](https://github.com/golang/go/blob/go1.26.10/doc/go_spec.html#L2294-L2308), [`alldocs.go`](https://github.com/golang/go/blob/go1.26.10/src/cmd/go/alldocs.go#L1869-L1875) |
| CPython, pytest | CPython 3.13.13, pytest 9.2.0 | a `uv` venv, `-I` | [`simple_stmts.rst`](https://github.com/python/cpython/blob/v3.13.13/Doc/reference/simple_stmts.rst#L834-L845), [`python.py`](https://github.com/pytest-dev/pytest/blob/9.2.0/src/_pytest/python.py#L374-L395) |
| Node's test runner | Node v24.22.1 | `nix-shell -p nodejs` | [`test.md`](https://github.com/nodejs/node/blob/v24.22.1/doc/api/test.md#L676-L683) |
| TypeScript | 5.9.3 | Node v24.22.1 | [`commandLineParser.ts`](https://github.com/microsoft/TypeScript/blob/v5.9.3/src/compiler/commandLineParser.ts#L3014) |
| Dart Sass | 1.100.0 | `nix-shell -p dart-sass` | [`options.dart`](https://github.com/sass/dart-sass/blob/1.100.0/lib/src/executable/options.dart#L475-L477), [`utils.dart`](https://github.com/sass/dart-sass/blob/1.100.0/lib/src/importer/utils.dart#L82-L112) |
| Hugo | v0.168.0 | `nix-shell -p hugo` | [`page-bundles.md`](https://github.com/gohugoio/hugo/blob/v0.168.0/docs/content/en/content-management/page-bundles.md#L24-L49) |
| Jekyll | 4.4.1 | `nix-shell -p jekyll` | [`entry_filter.rb`](https://github.com/jekyll/jekyll/blob/v4.4.1/lib/jekyll/entry_filter.rb#L7), [`post_reader.rb`](https://github.com/jekyll/jekyll/blob/v4.4.1/lib/jekyll/readers/post_reader.rb#L28-L54) |
| Django, Flask | Django 6.1.3, Flask 3.1.3 | a `uv` venv, CPython 3.13.13, `-I` | [`management/__init__.py`](https://github.com/django/django/blob/6.1.3/django/core/management/__init__.py#L29-L49), [`sansio/app.py`](https://github.com/pallets/flask/blob/3.1.3/src/flask/sansio/app.py#L653-L660) |
| pre-commit | 4.7.0, identify 2.6.20 | the same venv | [`clientlib.py`](https://github.com/pre-commit/pre-commit/blob/v4.7.0/pre_commit/clientlib.py#L246-L250) |
| Ruff | 0.17.6 | #1225's pytest venv | [`options.rs`](https://github.com/astral-sh/ruff/blob/0.17.6/crates/ruff_workspace/src/options.rs#L280-L300) |
| Bazel | 9.3.3 | not run | [`build-files.md`](https://github.com/bazelbuild/bazel/blob/9.3.3/site/en/concepts/build-files.md#L24-L32), [`BuildLanguageOptions.java`](https://github.com/bazelbuild/bazel/blob/9.3.3/src/main/java/com/google/devtools/build/lib/packages/semantics/BuildLanguageOptions.java#L438-L448) |
| Pants | 2.33.2 | not run | [`setup.py`](https://github.com/pantsbuild/pants/blob/release_2.33.2/src/python/pants/backend/python/subsystems/setup.py#L50-L51), [`targets-and-build-files.mdx`](https://github.com/pantsbuild/pants/blob/release_2.33.2/docs/docs/using-pants/key-concepts/targets-and-build-files.mdx#L311) |
| Spring Framework | 6.2.19 | not run | [`ComponentScan.java`](https://github.com/spring-projects/spring-framework/blob/v6.2.19/spring-context/src/main/java/org/springframework/context/annotation/ComponentScan.java#L37-L40) |
| SvelteKit | 2.70.4, commit `70a5ad21` | not run | [`create_manifest_data/index.js`](https://github.com/sveltejs/kit/blob/70a5ad21c58266d3e08863b519d9485819b1a776/packages/kit/src/core/sync/create_manifest_data/index.js#L237-L256) |
| Next.js | 16.4.6 | not run | [`02-project-structure.mdx`](https://github.com/vercel/next.js/blob/v16.4.6/docs/01-app/01-getting-started/02-project-structure.mdx#L221-L245) |
| entry points | the PyPA specification at `5b31c282` | run through CPython 3.13.13's `importlib.metadata` | [`entry-points.rst`](https://github.com/pypa/packaging.python.org/blob/5b31c28208dbcef1b089fdf9018910e611d2600f/source/specifications/entry-points.rst#L7-L8) |

The SvelteKit tag `@sveltejs/kit@2.70.4` resolves to the commit `70a5ad21c58266d3e08863b519d9485819b1a776`.

### Probes

Each probe is `research/selection-strategies/<tool>/run.sh`. It writes a small tree into an empty folder that it is given, and runs the tool there. `out.txt` beside it holds this session's output, with the folder's path replaced by `<dir>`, and the spaces at the end of a line removed.

```sh
R=research/selection-strategies
bash $R/cargo/run.sh <empty dir>
bash $R/python/run.sh <python 3.13.13 with pytest 9.2.0> <empty dir>
nix-shell -p go --run "bash $R/go/run.sh <empty dir>"
nix-shell -p nodejs --run "bash $R/node/run.sh <empty dir>"
nix-shell -p nodejs --run "bash $R/tsc/run.sh <dir with node_modules/typescript@5.9.3> <empty dir>"
nix-shell -p dart-sass --run "bash $R/sass/run.sh <empty dir>"
nix-shell -p hugo --run "bash $R/hugo/run.sh <empty dir>"
nix-shell -p jekyll --run "bash $R/jekyll/run.sh <empty dir>"
bash $R/django/run.sh <python 3.13.13 with django 6.1.3 and flask 3.1.3> <empty dir>
PRE_COMMIT_HOME=<empty dir> bash $R/precommit/run.sh <pre-commit 4.7.0> <empty dir>
bash $R/ruff/run.sh <ruff 0.17.6> <empty dir>
```

### Counts in the 9 suites

The suites are the 9 of [the cost of the rules](https://github.com/kalonji-tools/purba/blob/8e712dbf/prototypes/1015-rules-cost/projects.txt), each at its pinned commit. `measure.py` reads each file with `ast`, `tomllib` and `configparser`, and imports nothing from the suites. It counts only what #1225 leaves open. It reads #1225's `files.tsv` for the list of the files that each suite's own command collects.

```sh
git show 93af24df:research/selection-test-markers/files.tsv > <files.tsv>
mise exec python@3.14 -- python -I research/selection-strategies/measure.py <suites dir> <files.tsv> research/selection-strategies
```

It writes `config_keys.tsv` and `hooks.tsv`, and prints each count to `counts.txt`. It ran on CPython 3.14.7, free-threaded; every file parsed.

## 1. The eight strategies, and the systems that use each

Each table has one row for each system. "A unit with no marker" and "a helper with the marker" say what the system does in each case. 37 of the 52 rows are systems that are not test frameworks.

### 1.1 A name pattern marks the unit

The unit's own name carries the marker.

```text
tests/test_lend.py        def test_lend(): ...       _colors.scss  (a partial: never compiled alone)
```

| system | the marker | a unit with no marker | a helper with the marker | source |
|---|---|---|---|---|
| Dart Sass, compiling a folder | a file whose name does not start with `_` is an entry point; `_x.scss` is a partial | a partial writes no CSS file: `in/_colors.scss` gives no `out/_colors.css` [run] | `sizes.scss`, a helper with no `_`, gets its own `out/sizes.css` [run]. With `colors.scss` beside `_colors.scss`, `@use "colors"` stops: `It's not clear which file to import` [run] | [`options.dart` L475-L477](https://github.com/sass/dart-sass/blob/1.100.0/lib/src/executable/options.dart#L475-L477), [`utils.dart` L82-L112](https://github.com/sass/dart-sass/blob/1.100.0/lib/src/importer/utils.dart#L82-L112) |
| Jekyll, posts | a file in `_posts/` named `YEAR-MONTH-DAY-title.md` | `_posts/undated.md` writes no page, and no line [run] | — | [`posts.md` L20-L27](https://github.com/jekyll/jekyll/blob/v4.4.1/docs/_docs/posts.md#L20-L27), [`post_reader.rb` L52-L54](https://github.com/jekyll/jekyll/blob/v4.4.1/lib/jekyll/readers/post_reader.rb#L52-L54), [`document.rb` L15](https://github.com/jekyll/jekyll/blob/v4.4.1/lib/jekyll/document.rb#L15) |
| Jekyll, any file | a name that starts with `_`, `.`, `#` or `~` marks a file that is not a unit | `_hidden.md` writes no page [run] | — | [`entry_filter.rb` L7](https://github.com/jekyll/jekyll/blob/v4.4.1/lib/jekyll/entry_filter.rb#L7), [L31-L61](https://github.com/jekyll/jekyll/blob/v4.4.1/lib/jekyll/entry_filter.rb#L31-L61) |
| Django, management commands | a module in `management/commands/` whose name does not start with `_` | `_util.py` is not a command [run] | `helper.py` is listed as a command, and calling it fails: `has no attribute 'Command'` [run] | [`management/__init__.py` L29-L49](https://github.com/django/django/blob/6.1.3/django/core/management/__init__.py#L29-L49) |
| Go build | `_test.go` marks a test file, which `go build` leaves out; a name that starts with `_` or `.` is ignored | a file in `_old/` is ignored (#1225 [run]) | a helper in `helpers_test.go` that `use.go` calls breaks `go build` (#1225 [run]) | [`alldocs.go` L1856-L1867](https://github.com/golang/go/blob/go1.26.10/src/cmd/go/alldocs.go#L1856-L1867) |
| pytest | `test_*.py`, then the prefix `test` | `make()` is not collected (#1225 [run]) | `testable()` is collected (#1225 [run]) | [`python.py` L374-L395](https://github.com/pytest-dev/pytest/blob/9.2.0/src/_pytest/python.py#L374-L395) |
| `go test` | `TestXxx` | `checkHelper(t *testing.T)` is not a test, and no line [run] | `Testlower` fails the run: `malformed name` [run] | [`alldocs.go` L1869-L1875](https://github.com/golang/go/blob/go1.26.10/src/cmd/go/alldocs.go#L1869-L1875) |

### 1.2 A place marks the unit

The folder that holds the unit carries the marker.

```text
tests/lend.rs                         (cargo: a test target, by its folder)
src/routes/shop/+page.svelte          (SvelteKit: the route /shop)
```

| system | the marker | a unit with no marker | a helper with the marker | source |
|---|---|---|---|---|
| Cargo targets | each file in `tests/`, `src/bin/`, `examples/`, `benches/` | a file elsewhere is no target | `tests/common.rs` becomes an empty test target; `src/bin/helper.rs` stops the build (#1225 [run]) | [`cargo-targets.md` L316-L327](https://github.com/rust-lang/cargo/blob/0.100.0/doc/book/src/reference/cargo-targets.md#L316-L327) |
| Hugo | a file under `content/` is a page | `notes/n.md`, outside `content/`, writes no page [run] | `bundle/helper.md`, beside `bundle/index.md`, is a resource and not a page [run]. `_private.md` is a page: `_` marks nothing in Hugo [run] | [`page-bundles.md` L24-L30](https://github.com/gohugoio/hugo/blob/v0.168.0/docs/content/en/content-management/page-bundles.md#L24-L30), [L49](https://github.com/gohugoio/hugo/blob/v0.168.0/docs/content/en/content-management/page-bundles.md#L49) |
| Next.js, `app/` | a folder is a route segment; `page.js` or `route.js` makes it public | a folder with no `page.js` is no route | a helper beside `page.js` is "safely colocated" | [`02-project-structure.mdx` L221-L245](https://github.com/vercel/next.js/blob/v16.4.6/docs/01-app/01-getting-started/02-project-structure.mdx#L221-L245) |
| SvelteKit | a folder under `src/routes/` is a route; `+page.svelte` makes it a page | `page.svelte` prints `Missing route file prefix. Did you mean +page.svelte?`, and is skipped | `+helper.js` stops the build: `Files prefixed with + are reserved` | [`index.js` L237-L256](https://github.com/sveltejs/kit/blob/70a5ad21c58266d3e08863b519d9485819b1a776/packages/kit/src/core/sync/create_manifest_data/index.js#L237-L256), [L553](https://github.com/sveltejs/kit/blob/70a5ad21c58266d3e08863b519d9485819b1a776/packages/kit/src/core/sync/create_manifest_data/index.js#L553), [L570](https://github.com/sveltejs/kit/blob/70a5ad21c58266d3e08863b519d9485819b1a776/packages/kit/src/core/sync/create_manifest_data/index.js#L570) |
| Django, management commands | the folder `management/commands/` of an installed app | a command module elsewhere is not a command | `helper.py` is a command (row 1.1) [run] | [`management/__init__.py` L29-L39](https://github.com/django/django/blob/6.1.3/django/core/management/__init__.py#L29-L39) |
| Node's test runner | each `.js` file under a folder named `test`, and five name patterns | `lib/forgot.js`, which calls `test()`, never runs [run] | `test/helper.js` runs as a test file, and is reported as a passed test [run] | [`test.md` L676-L683](https://github.com/nodejs/node/blob/v24.22.1/doc/api/test.md#L676-L683) |
| Jest | each file in `__tests__/` | — | `__tests__/helpers.js` fails the run (#1225 [run]) | #1225 |

### 1.3 A mark on the unit itself

The unit carries an attribute, a decorator or a block of metadata.

```rust
#[test]
fn lends() { ... }
```

```markdown
---
title: post        (Jekyll: front matter marks a file to render)
---
```

| system | the marker | a unit with no marker | a helper with the marker | source |
|---|---|---|---|---|
| rustc `#[test]` | an attribute on a free function | not a test; in a test target, rustc warns: ``function `forgot` is never used`` [run] | runs as a test. `#[test]` on a function that takes an argument stops the build: `functions used as tests can not have any arguments` [run] | [the reference's `testing.md` L15-L31](https://github.com/rust-lang/reference/blob/c9cc32825b03fc206d9c6fe73e9ca6a2f3576d6c/src/attributes/testing.md#L15-L31) |
| JUnit 4 `@Test` | an annotation on a public method | not a test, and no line (#1225 [run]) | runs as a test | #1225 |
| Spring, component scan | `@Component`, or an annotation that carries it, on a concrete class | not a bean; the scan matches only its include filters | becomes a bean | [`ClassPathScanningCandidateComponentProvider.java` L225](https://github.com/spring-projects/spring-framework/blob/v6.2.19/spring-context/src/main/java/org/springframework/context/annotation/ClassPathScanningCandidateComponentProvider.java#L225), [L538-L550](https://github.com/spring-projects/spring-framework/blob/v6.2.19/spring-context/src/main/java/org/springframework/context/annotation/ClassPathScanningCandidateComponentProvider.java#L538-L550) |
| Jekyll, front matter | a YAML block at the top of the file | `plain.md` is copied as it is, as `_site/plain.md` [run] | rendered as a page | [`front-matter.md` L7-L8](https://github.com/jekyll/jekyll/blob/v4.4.1/docs/_docs/front-matter.md#L7-L8) |
| Hugo, `draft: true` | a field of front matter that marks a page that is not a unit | — | `draft.md` writes no page [run] | [run] |
| pytest, `__test__` | an attribute: `False` hides a test, `True` makes any function one | — | `lend_works.__test__ = True` is collected; `test_hidden.__test__ = False` is not [run] | [`python.py` L377-L395](https://github.com/pytest-dev/pytest/blob/9.2.0/src/_pytest/python.py#L377-L395) |

### 1.4 A call registers the unit

Code that runs at load time adds each unit to a registry.

```js
test('lends one', () => { ... });
```

```python
admin.site.register(Book)
```

| system | the marker | a unit with no marker | a helper with the marker | source |
|---|---|---|---|---|
| Node's test runner | `test(name, fn)` | — | a loop registers `lends 1` and `lends 2`; the same name registered twice runs twice, with no line [run] | [`test.md` L676-L683](https://github.com/nodejs/node/blob/v24.22.1/doc/api/test.md#L676-L683); [run] |
| Jest, Vitest | `test(name, fn)` | — | a file that registers no test fails the run (#1225 [run]) | #1225 |
| Django admin | `admin.site.register(Model)` in each app's `admin.py`, which `autodiscover()` imports | `Shelf` has no admin page, and no line [run] | a second `register(Book)` stops start-up: `AlreadyRegistered` [run] | [`sites.py` L95-L132](https://github.com/django/django/blob/6.1.3/django/contrib/admin/sites.py#L95-L132), [`admin/__init__.py` L55-L56](https://github.com/django/django/blob/6.1.3/django/contrib/admin/__init__.py#L55-L56) |
| Flask | `app.add_url_rule(rule, endpoint, view)`, or `@app.route` | — | a second view under the endpoint `page` raises `AssertionError`, after the rule `/b` is already in the map [run] | [`sansio/app.py` L653-L660](https://github.com/pallets/flask/blob/3.1.3/src/flask/sansio/app.py#L653-L660) |
| unittest, `load_tests` | a function of the module that returns its suite | `test_also` is left out of the run, with no line [run] | — | [`loader.py` L109-L113](https://github.com/python/cpython/blob/v3.13.13/Lib/unittest/loader.py#L109-L113) |

### 1.5 A list names each unit

A file apart from the units lists them.

```toml
[[test]]
name = "listed"
path = "tests/listed.rs"
```

```python
# BUILD (Pants)
python_tests(name="tests")      # sources default to test_*.py, *_test.py, tests.py
```

| system | the marker | a unit with no marker | a helper with the marker | source |
|---|---|---|---|---|
| Cargo, `[[test]]` | an entry with a `name` and a `path` | with `autotests = false`, `tests/unlisted.rs` is not built, and no line [run] | a missing path stops the build: ``can't find integration-test `gone` `` [run] | [`cargo-targets.md` L316-L327](https://github.com/rust-lang/cargo/blob/0.100.0/doc/book/src/reference/cargo-targets.md#L316-L327) |
| Bazel | a rule call in a `BUILD` file, such as `py_test(...)` | only the rules that a `BUILD` file declares make targets | a `glob()` that matches nothing is an error by default | [`build-files.md` L24-L32](https://github.com/bazelbuild/bazel/blob/9.3.3/site/en/concepts/build-files.md#L24-L32), [`BuildLanguageOptions.java` L438-L448](https://github.com/bazelbuild/bazel/blob/9.3.3/src/main/java/com/google/devtools/build/lib/packages/semantics/BuildLanguageOptions.java#L438-L448) |
| Pants | a `python_tests()` target in a `BUILD` file, whose `sources` default to globs | `pants tailor ::` adds the targets that the convention finds | — | [`setup.py` L50-L51](https://github.com/pantsbuild/pants/blob/release_2.33.2/src/python/pants/backend/python/subsystems/setup.py#L50-L51), [`targets-and-build-files.mdx` L311](https://github.com/pantsbuild/pants/blob/release_2.33.2/docs/docs/using-pants/key-concepts/targets-and-build-files.mdx#L311) |
| Python entry points | a line `name = module:attr` under a group, in `entry_points.txt` | `unlisted` is never found [run] | `gone = shop.plugins:gone` is listed; it fails only at `load()`: `AttributeError` [run] | [`entry-points.rst` L7-L8, L42-L46](https://github.com/pypa/packaging.python.org/blob/5b31c28208dbcef1b089fdf9018910e611d2600f/source/specifications/entry-points.rst#L7-L46), [`metadata/__init__.py` L173-L182](https://github.com/python/cpython/blob/v3.13.13/Lib/importlib/metadata/__init__.py#L173-L182) |
| Django, `INSTALLED_APPS` | the dotted path of each app | the command `find` of an unlisted app: `Unknown command: 'find'` [run] | — | [`applications.txt` L41](https://github.com/django/django/blob/6.1.3/docs/ref/applications.txt#L41) |
| Python, `__all__` | a list of the module's public names | a public `helper` is left out of `import *` [run] | `"gone"` in `__all__` raises `AttributeError` at `import *` [run] | [`simple_stmts.rst` L834-L845](https://github.com/python/cpython/blob/v3.13.13/Doc/reference/simple_stmts.rst#L834-L845) |
| doctest, `__test__` | a module-level dict that adds units | — | `shop.__test__.extra` is found beside `shop.lend` [run] | [`doctest.py` L1044-L1057](https://github.com/python/cpython/blob/v3.13.13/Lib/doctest.py#L1044-L1057) |
| TypeScript, `files` | a list of paths in `tsconfig.json` | — | `src/gone.ts` gives `error TS6053: File 'src/gone.ts' not found` [run] | [run] |

### 1.6 A rule of visibility or of type picks the unit

The unit's kind, signature or visibility decides. No marker is added for this purpose.

```go
func TestLend(t *testing.T) { ... }   // a test: the name, and the type of its one parameter
func helper() int { ... }             // not exported: lower case
```

| system | the rule | a unit that breaks the rule | a helper that meets the rule | source |
|---|---|---|---|---|
| Go, exported names | a name that starts with an upper-case letter is visible to other packages | `lend.helper()` from another package stops the build: `name helper not exported by package lend` [run] | visible to every package | [`go_spec.html` L2294-L2308](https://github.com/golang/go/blob/go1.26.10/doc/go_spec.html#L2294-L2308) |
| `go test`, signature | `TestXxx(t *testing.T)` | `TestWrong(n int)` stops the run: `wrong signature for TestWrong, must be: func TestWrong(t *testing.T)` [run] | runs as a test | [`alldocs.go` L1869-L1875](https://github.com/golang/go/blob/go1.26.10/src/cmd/go/alldocs.go#L1869-L1875) |
| unittest | a method `test…` of a `TestCase` subclass | a module-level function is never a test (#1225 [run]) | runs as a test | #1225 |
| Python, `import *` with no `__all__` | each name that does not start with `_` | `_helper` stays out [run] | the imported module `os` is exported too [run] | [`simple_stmts.rst` L834-L845](https://github.com/python/cpython/blob/v3.13.13/Doc/reference/simple_stmts.rst#L834-L845) |
| Spring, component scan | a concrete, independent class | an interface or an inner class is not a bean | — | [`ClassPathScanningCandidateComponentProvider.java` L570-L578](https://github.com/spring-projects/spring-framework/blob/v6.2.19/spring-context/src/main/java/org/springframework/context/annotation/ClassPathScanningCandidateComponentProvider.java#L570-L578) |
| pre-commit, `types` | identify reads a file's type from its name, mode and shebang | — | `scripts/run`, with no extension and a Python shebang, is tagged `python` and goes to a `types: [python]` hook [run] | [`clientlib.py` L248](https://github.com/pre-commit/pre-commit/blob/v4.7.0/pre_commit/clientlib.py#L248) |

### 1.7 Globs in configuration include and exclude files

A file of settings, apart from the units, names patterns of paths.

```json
{"include": ["src"], "exclude": ["src/skip"]}
```

| system | the setting | a unit that no glob matches | a helper that a glob matches | source |
|---|---|---|---|---|
| TypeScript, `tsconfig.json` | `include`, `exclude` | not compiled, unless an included file imports it: `src/skip/b.ts` is compiled under `exclude: ["src/skip"]` [run] | compiled. `include: ["nothing"]` gives `error TS18003: No inputs were found` [run] | [`commandLineParser.ts` L3014](https://github.com/microsoft/TypeScript/blob/v5.9.3/src/compiler/commandLineParser.ts#L3014); [run] |
| Ruff | `include`, `extend-exclude`, `force-exclude` | not checked, unless it is named on the command line: `tests/helper.py` is checked under `extend-exclude = ["tests"]` [run] | checked. With `force-exclude = true`, the named `tests/helper.py` is skipped too [run] | [`options.rs` L280-L300](https://github.com/astral-sh/ruff/blob/0.17.6/crates/ruff_workspace/src/options.rs#L280-L300) |
| pre-commit | `files` and `exclude`, each a regex of paths | not passed to the hook | `files: ^tests/` passes `tests/helper.py` with `tests/test_a.py`. A hook that matches no file prints `(no files to check)Skipped` [run] | [`clientlib.py` L246-L247](https://github.com/pre-commit/pre-commit/blob/v4.7.0/pre_commit/clientlib.py#L246-L247) |
| Bazel, `glob()` | include and exclude patterns, inside a rule's list | not in the target | in the target; an empty match is an error | [`BuildLanguageOptions.java` L438-L448](https://github.com/bazelbuild/bazel/blob/9.3.3/src/main/java/com/google/devtools/build/lib/packages/semantics/BuildLanguageOptions.java#L438-L448) |
| pytest, coverage.py | `testpaths`, `python_files`; coverage's `omit` | not collected; not measured | collected; measured | #1225 section 3 |

### 1.8 A default convention that an explicit list replaces

The system finds units by a convention until the project writes a list. Then the list replaces the convention, or adds to it.

```toml
[package]
autotests = false          # tests/*.rs no longer become targets; only [[test]] entries do
```

| system | the default | the list that replaces it | a unit that the list leaves out | source |
|---|---|---|---|---|
| Cargo | each file in `tests/` | `autotests = false`, then each `[[test]]` | `tests/unlisted.rs` is not built, and no line. With the default left on, a `[[test]]` adds to the folder's targets [run] | [`cargo-targets.md` L316-L327](https://github.com/rust-lang/cargo/blob/0.100.0/doc/book/src/reference/cargo-targets.md#L316-L327). In edition 2015, one `[[test]]` turned the default off; from 2018, it stays on ([L362-L364](https://github.com/rust-lang/cargo/blob/0.100.0/doc/book/src/reference/cargo-targets.md#L362-L364)) |
| TypeScript | `include` is `**/*` | `include`, or `files` | not compiled, unless imported [run] | [`commandLineParser.ts` L3014](https://github.com/microsoft/TypeScript/blob/v5.9.3/src/compiler/commandLineParser.ts#L3014) |
| Ruff | `*.py`, `*.pyi`, `*.pyw`, `*.ipynb`, `*.md` and the config files | `include` | `include = ["tests/test_*.py"]` leaves out `tests/helper.py` and `src/` [run] | [`options.rs` L294-L300](https://github.com/astral-sh/ruff/blob/0.17.6/crates/ruff_workspace/src/options.rs#L294-L300) |
| Python, `import *` | each public name | `__all__` | a public name not in `__all__` [run] | [`simple_stmts.rst` L834-L845](https://github.com/python/cpython/blob/v3.13.13/Doc/reference/simple_stmts.rst#L834-L845) |
| pytest | `python_files = test_*.py *_test.py`, `python_functions = test` | each key, in configuration | `python_functions = check_*` collects `check_lend` and leaves out `test_lend` [run] | [`python.py` L374-L375](https://github.com/pytest-dev/pytest/blob/9.2.0/src/_pytest/python.py#L374-L375) |
| Pants | `test_*.py`, `*_test.py`, `tests.py` | `sources=` on the target | — | [`setup.py` L50-L51](https://github.com/pantsbuild/pants/blob/release_2.33.2/src/python/pants/backend/python/subsystems/setup.py#L50-L51) |
| Spring | the package of the class that carries `@ComponentScan` | `basePackages` | a class outside the packages is not a bean | [`ComponentScan.java` L37-L40](https://github.com/spring-projects/spring-framework/blob/v6.2.19/spring-context/src/main/java/org/springframework/context/annotation/ComponentScan.java#L37-L40) |
| Next.js, `pages/`; setuptools | each file under `pages/`; each top-level package | `pageExtensions`; `packages` | as #1225 section 2 | #1225 |

## 2. Which strategies combine, and which systems combine them

Each system below combines two or more of the strategies of section 1.

| system | strategies | how they combine |
|---|---|---|
| Cargo | place, mark, list, default | `tests/*.rs` are targets; `#[test]` marks a test in each; `[[test]]` adds a target; `autotests = false` replaces the folder |
| Go | name, type, visibility, near-miss check | `_test.go` marks a file; `TestXxx(*testing.T)` marks a test; `go test` runs `go vet`'s `tests` check first |
| pytest | globs, name, type, mark, default | `testpaths` and `python_files`; the prefix `test`; a `Test…` class or a `TestCase`; `__test__`; each default is a key |
| unittest | name, type, call | `test*.py`; a `TestCase` subclass; `load_tests` returns the module's suite |
| Django | list, place, name, call | `INSTALLED_APPS`; `management/commands/`; `admin.py`; `admin.site.register` |
| Next.js, `app/` | place, name | a folder is a segment; `page.js` makes it a route; `_folder` is private |
| SvelteKit | place, name, near-miss check | `src/routes/`; `+page.svelte`; a warning for `page.svelte`, an error for an unknown `+` file |
| Hugo | place, name, mark | `content/`; `index.md` or `_index.md` decides what its folder's other files are; `draft: true` |
| Jekyll | name, mark, place | `_` hides; front matter marks a page; `_posts/` and a date mark a post |
| Pants | list, default, generator | a `BUILD` target whose `sources` default to globs; `pants tailor` writes the targets from the convention |
| Bazel | list, globs | each rule in a `BUILD` file; `glob()` inside it, which refuses an empty match |
| Spring | place, mark, type | the base package; `@Component`; a concrete, independent class |
| Node's test runner | name, place, call | five name globs and the folder `test`; `test()` |
| TypeScript | default, globs, list, reach | `**/*`; `include` and `exclude`; `files`; each file that an included file imports |

Four patterns recur. judgement: each pattern names what the systems share, and no source states it.

- **A coarse marker of a file, then a fine marker of the unit.** Most test runners surveyed do this, and so do Django, Next.js, SvelteKit, Hugo and Spring. JUnit and cargo's unit tests have no marker of a file (#1225). It is the shape of purba's two stages.
- **A default convention, and a list that replaces it.** Cargo, TypeScript, Ruff, pytest, Pants, Spring and `__all__` do this.
- **A marker that says "not a unit".** `_` does this in Sass, Jekyll, Django commands, Next.js's `_folder`, Go's build and Python's `import *`. `draft: true` and `__test__ = False` do it on one unit.
- **A check for a near miss.** `go vet`'s `tests` check and SvelteKit's prefix warning see a marker that is almost right. Sass's ambiguity error sees two files where one is meant. Pants's `tailor` sees a unit that the list misses. Bazel's empty glob sees a pattern that misses every file.

## 3. Each strategy against purba's scope

### The constraints

- **Stage one** picks the test files from their paths alone. **Stage two** reads the tree of each file, and imports nothing ([lazy work](https://github.com/kalonji-tools/purba/blob/3093871b/LANDSCAPE.md#L141)).
- **One identity.** Each test has one identity, `module::test::case`, and a prefix of it selects ([one identity](https://github.com/kalonji-tools/purba/blob/3093871b/LANDSCAPE.md#L144)).
- **Strict.** A missing or wrong marker is seen, not silent ([purba is always strict](https://github.com/kalonji-tools/purba/blob/3093871b/docs/decisions/purba-is-always-strict.md#decision-outcome)). Selection is not a switch, and strictness judges only the tests that a run selects ([L43](https://github.com/kalonji-tools/purba/blob/3093871b/docs/decisions/purba-is-always-strict.md#L43)).
- **Port.** A suite from pytest ports to purba. The 9 pinned suites measure the cost.
- **Other tools.** Ruff and coverage read the markers too.
- **Configuration.** purba reads no file of the project other than `pyproject.toml` ([L65](https://github.com/kalonji-tools/purba/blob/3093871b/docs/decisions/purba-reads-configuration-from-six-levels.md#L65)). No value loosens a check ([L60](https://github.com/kalonji-tools/purba/blob/3093871b/docs/decisions/purba-reads-configuration-from-six-levels.md#L60)). Selection already owns one planned dial that picks files, by platform ([L80](https://github.com/kalonji-tools/purba/blob/3093871b/docs/decisions/purba-reads-configuration-from-six-levels.md#L80)).

### 3.1 The fit table

Each cell is short; the notes in 3.2 give its source. "—" means the strategy plays no part at that point.

| strategy | stage one, the path | stage two, the tree, no import | one identity, a prefix selects | a missing marker | a helper with the marker | port of the 9 suites | Ruff and coverage read it |
|---|---|---|---|---|---|---|---|
| 1.1 name pattern | yes: `test_*.py` | yes: `def test…` | yes: `path::test_name` | silent, unless a near-miss check reads it | a rename; `testable()` is a test | today's form: 1,802 tests carry "test" three times | yes: Ruff's `PT` and `D401` read `test`; globs name `test_*` |
| 1.2 place | yes: `tests/` | — | yes: a folder is a prefix of the path | silent: 17 of 189 test files sit in no test folder | becomes a unit: 141 other files under test folders | 17 missed (2 run today), 141 added | yes, through globs such as `tests/**` |
| 1.3 mark on the unit | no: every `.py` file, 485 against 189 | yes: a decorator, resolved through the imports | yes: the name of the `def` | silent; no unused-function signal in a Python module | runs as a test | 1,899 added lines; 719 tests already decorated | no: Ruff's `PT` rules read the name |
| 1.4 call registers | no | only a call with literal arguments at module level | the name is a string: it may repeat, or hold `::` | silent | runs as a test, if called | 1,899 rewrites; 0 tests made in a loop | no |
| 1.5 list | yes, if it lists paths in `pyproject.toml` | a list of tests repeats the tree | an entry's name can differ from its path | silent | a listed helper runs; a wrong entry stops the run | 175 file entries, or 15 folder entries | no: each tool keeps its own list |
| 1.6 visibility or type | file type only | yes: `_`, and the parameters of a `def` | yes: the name of the `def` | the type rule can refuse a near miss | a public helper becomes a test: 92 in 38 files | #1225 section 5.2 | Ruff reads `_`; its `PT` rules lose the tests |
| 1.7 globs | yes | — | — | silent; an empty match can be an error | an exclude does not stop a path named on the command line, in Ruff | 5 suites set `testpaths`, 1 sets `python_files` | each tool has its own dialect |
| 1.8 default, then a list | yes | yes, for a name pattern | as the default | silent once the list replaces the default | as the default | 1 suite adds a pattern; none replaces a default | each tool has its own default |

### 3.2 Notes on each strategy

**1.1 A name pattern.** purba holds this marker today, in code only ([`facts.rs` L294](https://github.com/kalonji-tools/purba/blob/3093871b/crates/parsing/src/facts.rs#L294), [`lib.rs` L454](https://github.com/kalonji-tools/purba/blob/3093871b/crates/selection/src/lib.rs#L454)). Both stages read it without an import. Its cost and its records are #1225's section 5.1. A missing marker is silent in pytest, `go test` and Jekyll [run]. judgement: a near-miss check, as in `go vet` or SvelteKit, is the one way surveyed to make it seen. Such a check would read names close to the marker, such as `tests_…`, `Test_…` or a public `def` that states an expectation. Neither the check nor its false positives are measured.

**1.2 A place.** It serves stage one, and [`needs.toml` L9](https://github.com/kalonji-tools/purba/blob/3093871b/.config/needs.toml#L9) states it. It leaves stage two with no marker, so it combines with 1.1, 1.3 or 1.6. Each non-test system that marks by place alone also makes helpers into units: Node's `test/helper.js` and Django's `helper.py` here, Cargo and Jest in #1225. Next.js, SvelteKit and Hugo add a reserved file name. The measured cost is #1225's section 5.3.

**1.3 A mark on the unit.** Stage two reads `@test` from the tree, and resolves it through the module's imports, as it already does for `@skip` ([record L41](https://github.com/kalonji-tools/purba/blob/3093871b/docs/decisions/a-test-carries-only-purbas-own-decorators.md#L41)). It gives stage one nothing, so it needs a file marker. A missing `#[test]` in Rust gives a dead-code warning, since nothing calls the function [run]. judgement: Python has no such signal, because a module-level function is public and may be imported. The records that change are #1225's section 5.4: the public surface, the term "test", and the form.

**1.4 A call registers the unit.** Node and Jest run the file to learn its tests. A loop registers tests that only a run can see [run]. Stage two imports nothing, so it can read only a call whose arguments are literals at module level. judgement: a strict reader then refuses each other call, as selection refuses a `def` under a statement today ([record L48](https://github.com/kalonji-tools/purba/blob/3093871b/docs/decisions/a-test-carries-only-purbas-own-decorators.md#L48)). The name is a string. Node ran two tests named `lends one` with no line [run]. judgement: a free string can also hold `::` or a space, which a prefix of the identity reads as a separator. Django and Flask refuse a second registration of one key [run]. In the 9 suites, no test file makes a test in a loop, or through `globals()` or `setattr`. The one module-level loop builds the cases of a `parametrize` ([`test_lowlevel.py` L282](https://github.com/psf/requests/blob/3c7baaa05ab4aa49cc819ce3a819d260de19c157/tests/test_lowlevel.py#L282)). So the power that a call adds goes unused there. Each of the 1,899 tests would be rewritten. judgement: [the term "test"](https://github.com/kalonji-tools/purba/blob/3093871b/CONTEXT.md#L341) names a function as import binds it, and a call changes that.

**1.5 A list.** A list of test files in `[tool.purba]` serves stage one, and it is the one file that purba reads. A `BUILD` file in each folder, as in Bazel or Pants, is a second file to read. Each list surveyed reports a wrong entry, and is silent about a missing one. Pants pairs its list with `tailor`, which writes the targets that the convention finds. judgement: purba could check a list against a name pattern in the same way, and report a test file that no entry names. The suites would need 175 file entries, or 15 folder entries [run]. Ruff and coverage cannot read the list: 4 suites keep Ruff file settings and 4 keep coverage file settings [run, `config_keys.tsv`]. A Cargo entry has a `name` and a `path`. judgement: an identity built from a name would give a test a second name beside its path.

**1.6 A rule of visibility or of type.** `_` marks a helper, as in Python's `import *`, Sass and Jekyll. The cost of "each public `def` is a test" is #1225's section 5.2. A rule of type can give a near-miss check: `go test` refuses `TestWrong(n int)` [run]. judgement: purba's records already hold such rules on a test's shape. A test names the fixture of each input, and returns nothing ([CONTEXT.md L341](https://github.com/kalonji-tools/purba/blob/3093871b/CONTEXT.md#L341)). A rule of type on a class, such as `TestCase`, needs its base resolved. Stage two sees only the name of the base. purba refuses each of the 2,505 methods in classes today (#1225 section 4.4).

**1.7 Globs.** Stage one reads them from `pyproject.toml`. 5 suites set `testpaths`, and dateutil adds `*_solution.py` to `python_files` [run]. Each tool has its own dialect: Ruff globs, coverage's patterns and pre-commit's regexes. In 5 suites, pytest and Ruff or coverage each name the test files or their folder [run, `config_keys.tsv`]. judgement: a glob of purba would be one more copy. An exclude did not stop a unit that an included file reaches, in TypeScript [run]. An empty match is an error in TypeScript [run] and Bazel [source]. judgement: an exclude names a fact about the project, and selection is not a switch ([strict L43](https://github.com/kalonji-tools/purba/blob/3093871b/docs/decisions/purba-is-always-strict.md#L43)). Whether an exclude loosens a check is a question for the design of selection.

**1.8 A default, then a list.** Every system in 1.8 keeps the convention until the project writes a list. Cargo's 2015 edition turned the convention off when one entry was written. The 2018 edition stopped that [source]. Once a list replaces a default, a new unit outside it is silent [run]. In the 9 suites, dateutil adds a file pattern, and no suite replaces the function pattern [run].

## 4. New counts in the 9 suites

Each count is [run], from `counts.txt`. #1225 holds every other count that this file quotes.

### 4.1 Settings that name test files, tests or their globs

Matcher: the keys `testpaths`, `python_files`, `python_classes`, `python_functions`, `norecursedirs` and `addopts` of pytest; `source`, `include` and `omit` of coverage; `include`, `exclude`, `extend-exclude`, `extend-include`, `force-exclude` and the keys of `per-file-ignores` of Ruff. Read in `pyproject.toml`, `setup.cfg`, `tox.ini`, `pytest.ini` and `.coveragerc` at each suite's root.

| setting | loguru | attrs | click | requests | dateutil | pyflakes | toolz | six | schedule | suites |
|---|---|---|---|---|---|---|---|---|---|---|
| pytest `testpaths` | `tests` | `tests` | `tests` | `tests` | — | — | `toolz` | — | — | 5 |
| pytest `python_files` | — | — | — | — | adds `*_solution.py` | — | — | — | — | 1 |
| pytest `python_functions`, `python_classes`, `norecursedirs` | — | — | — | — | — | — | — | — | — | 0 |
| a flag of `addopts` that selects or adds tests: `-m`, `-k`, `--ignore`, `--deselect`, `--doctest-modules` and kin | — | — | `-m 'not stress'` | `--doctest-modules` | — | — | — | — | — | 2 |
| coverage `source`, `include` or `omit` | — | — | 1 | 1 | 1 | — | 2 | — | — | 4 |
| Ruff file settings or `per-file-ignores` globs | 3 | 5 | 1 | 4 | — | — | — | — | — | 4 |

pyflakes runs `unittest discover`, and six and schedule set no key of the matcher. #1225's method and section 3 hold their commands.

### 4.2 Names that change collection

Matcher: in each `conftest.py` and each test file, a module-level `def` named as a pytest collection hook or `load_tests`, an assignment to `collect_ignore`, `collect_ignore_glob`, `__test__` or `pytestmark`, and `X.__test__ = …` anywhere.

| name | rows | where |
|---|---|---|
| `collect_ignore` | 1 | `attrs/tests/conftest.py`, under a condition on the Python version |
| `pytest_collection_modifyitems` | 1 | `dateutil/tests/conftest.py`; it adds a mark, and removes no test |
| `pytestmark` | 3 | attrs 2, dateutil 1 |
| `X.__test__` | 1 | `toolz/tests/test_curried_doctests.py`: doctest's `__test__` dict, set in a test's body |
| `load_tests`, `__test__ = False` | 0 | — |

### 4.3 Module-level shapes in the 189 test files

Matcher: in each test file, a `def test…` under a module-level `if`, `for`, `while`, `with`, `try` or `match`, and not in a `def` or `class`; a module-level `for` or `while`; a module-level call to `globals`, `locals`, `vars`, `setattr` or `exec`.

| count | loguru | attrs | click | requests | dateutil | pyflakes | toolz | six | schedule | total |
|---|---|---|---|---|---|---|---|---|---|---|
| `def test…` under a module-level statement | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 4 | 0 | 4 |
| module-level `for` or `while` | 0 | 0 | 0 | 1 | 0 | 0 | 0 | 0 | 0 | 1 |
| module-level call to `globals`, `locals`, `vars`, `setattr` or `exec` | 0 | 0 | 0 | 0 | 2 | 0 | 0 | 0 | 0 | 2 |

The 4 are `test_b` and `test_u`, each defined under `if six.PY3:` and again under `else:` ([`test_six.py` L487-L510](https://github.com/benjaminp/six/blob/c8e394065cd541a16c040515dc0afb85cf22a7c3/test_six.py#L487-L510)). The loop builds `parametrize` cases ([`test_lowlevel.py` L282](https://github.com/psf/requests/blob/3c7baaa05ab4aa49cc819ce3a819d260de19c157/tests/test_lowlevel.py#L282)). The 2 calls read `locals()` to test `import *` ([`test_import_star.py` L7-L9](https://github.com/dateutil/dateutil/blob/a78cbd81d5d03b5f126eb36edb92bca78972c804/tests/test_import_star.py#L7-L9)).

### 4.4 Entries that a list of test files needs

Matcher: the files whose `own_collects` is 1 in #1225's `files.tsv`: 174 test files and dateutil's `mlk_day_rrule_solution.py`.

| count | loguru | attrs | click | requests | dateutil | pyflakes | toolz | six | schedule | total |
|---|---|---|---|---|---|---|---|---|---|---|
| files | 50 | 25 | 43 | 9 | 18 | 13 | 15 | 1 | 1 | 175 |
| folders that hold one | 1 | 1 | 4 | 1 | 3 | 1 | 2 | 1 | 1 | 15 |

A list of tests, not files, needs one entry for each of the 1,899 module-level tests (#1225 section 4.3).

## 5. What this file does not measure

- **purba's own lines for each strategy.** No probe ran a purba wheel. Each line about what selection would print is a judgement from the records and the code.
- **A tester's reading.** No form was rated blind. Strategies 1.2, 1.3, 1.4 and 1.6 change a form that a tester writes, as #1225's section 5 shows for 1.2, 1.3 and 1.6. Each such change owes a rating ([record L28](https://github.com/kalonji-tools/purba/blob/3093871b/docs/decisions/a-form-that-a-tester-meets-is-rated-blind-before-it-is-decided.md#L28)).
- **A near-miss check for purba.** Section 3 names it as a judgement. Its predicate and its false positives on the 9 suites are unmeasured.
- **Bazel, Pants, Spring, SvelteKit and Next.js** are [source] only. I did not run them.
- **The Hugo row for `draft: true`** has no pinned documentation line. It rests on the run alone.
- **The settings of pre-commit in the suites.** `measure.py` does not read `.pre-commit-config.yaml`.

## Files

| path | what it holds |
|---|---|
| `research/selection-strategies/measure.py` | the count script for the 9 suites |
| `research/selection-strategies/counts.txt` | its output, with every count of section 4 |
| `research/selection-strategies/config_keys.tsv`, `hooks.tsv` | one row for each setting of 4.1, and each name of 4.2 |
| `research/selection-strategies/{cargo,python,go,node,tsc,sass,hugo,jekyll,django,precommit,ruff}/run.sh`, `out.txt` | each probe, and its output in this session |
