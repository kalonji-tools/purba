# How tools mark which files and which functions are tests, and what each marker costs

Research for [How do test runners and other tools mark which files and which functions are tests, and what does each marker cost?](https://github.com/kalonji-tools/purba/issues/1225)
Measured on 2026-11-24, against purba `main` at `3093871b`, on one Linux machine.
This file decides nothing. It lists the markers, measures them in the 9 pinned suites, and weighs four sets of markers against purba's records.

## Summary

Each test runner surveyed marks a test file and a test with two separate markers. No two runners use the same pair. A runner marks a test either by its name (pytest, unittest, JUnit 3, `go test`) or by an explicit mark (cargo's `#[test]`, JUnit 4's `@Test`, a `test()` call in Jest and Vitest, Ward's `@test`). It marks a file by its name (pytest, unittest, `go test`, Vitest, Maven Surefire), by its folder (cargo's `tests/`, Jest's `__tests__/`), or not at all (JUnit itself, cargo's unit tests). Where a folder alone marks the files, every helper in that folder becomes a unit. Each tool then added a way out.

Five findings matter most for purba.

- **A folder that marks its files turns each helper in it into a unit, in each tool that does it.** A helper in `tests/` becomes an empty test target in cargo [run]. A helper in `__tests__/` fails a Jest run [run]. A helper in `src/bin/` stops a Cargo build [run]. Next.js turned every file under `pages/` into a route. Its later `app/` router marks a route with the file `page.tsx`, so a helper can sit beside it [source].
- **In the 9 suites, the folder marker alone misses 17 of 189 test files, and catches 141 other files.** The suites' own commands collect 2 of the 17: `six/test_six.py` and `schedule/test_schedule.py` sit at the root. Of the 141 other files, 118 sit in deeper folders. 101 of them are loguru's sample scripts, which hold 20 functions named `test…`. 26 sit beside a test file: 10 `__init__.py`, 5 `conftest.py` and 11 modules. In six and schedule, those modules include the code under test.
- **Under "only the file carries the marker", 92 public helpers in 38 test files become tests.** 18 of them are pytest fixtures, and 5 carry another decorator. Of the other 69, 60 take a parameter with no default, and 9 take no such parameter. The latest form also keeps fixtures public: 4 of the 4 non-test functions in its ported test files are public. The test files already hold 66 private functions.
- **Other tools read the name `test`.** Ruff's `PT028` reports each test that takes a parameter with a default [run]. In purba's form, each test that takes an input has such a parameter. Ruff's `D401` skips a function whose name starts with `test` [run]. `go vet` reports `Testlower` [run]. Coverage reads no marker by default. Each suite's coverage and lint settings name their own globs [run].
- **Today only code holds the name marker, and nothing holds a file marker.** Parsing keeps only `test…` names ([`facts.rs` L294](https://github.com/kalonji-tools/purba/blob/3093871b/crates/parsing/src/facts.rs#L294)). The command takes any one `.py` path ([`lib.rs` L211](https://github.com/kalonji-tools/purba/blob/3093871b/crates/command/src/lib.rs#L211)). Four refusals of a record exist only because the name is the marker. They refuse a test name bound again, bound with no `def`, imported with `from`, and holding a value ([L46-L47 and L58-L59](https://github.com/kalonji-tools/purba/blob/3093871b/docs/decisions/a-test-carries-only-purbas-own-decorators.md#L46-L59)).

Today, 1,802 of the 1,899 module-level tests in the 9 suites carry the marker three times: in the folder, the file and the function. 60 of them, all in click, carry it four times, as in `tests/test_utils/test_echo.py::test_echo_…`.

## How to read this file

- **[run]** means I ran it in this session, on the version named. **[source]** means the tool's code or documentation says so, at the pinned version or commit, and I did not run it. **judgement** marks a claim that no source states.
- A **marker** is what a tool reads to decide that a path or a function is a test. A marker of a file serves purba's stage one. A marker of a function serves stage two.
- **A test file** in the counts is a `.py` file whose name matches `test_*.py` or `*_test.py`, anywhere in the suite. That is pytest's default `python_files`. Each of the 189 matches `test_*.py` too, which is the form's marker.
- **A test folder** in the counts is a folder named `test`, `tests` or `testing`, at any depth.
- **Beside a test file** means in the same folder as a test file.
- **A module-level function** is a `def` or `async def` in the body of the module. A `def` under an `if` at that level does not count.
- **Public** means a name that does not start with `_`. **Private** means a name that starts with `_` and is not a dunder. **`test*`** means a name that starts with `test`, as pytest's prefix match reads it.
- A count answers only its own matcher. Each table row names its matcher.

## Method

### Versions

| tool | version | how I ran it | pinned source |
|---|---|---|---|
| pytest | 9.2.0 | CPython 3.13.13, a `uv` venv, `-I` | [`python.py`](https://github.com/pytest-dev/pytest/blob/9.2.0/src/_pytest/python.py#L96-L114), [`goodpractices.rst`](https://github.com/pytest-dev/pytest/blob/9.2.0/doc/en/explanation/goodpractices.rst#L100-L117) |
| unittest | CPython 3.13.13 | the same venv, `-I` | [`loader.py` at `v3.13.13`](https://github.com/python/cpython/blob/v3.13.13/Lib/unittest/loader.py#L68) |
| cargo test | cargo 1.99.0 | this machine, `--offline` | [`cargo-targets.md` at `0.100.0`](https://github.com/rust-lang/cargo/blob/0.100.0/doc/book/src/reference/cargo-targets.md#L111-L123), [the reference's `testing.md`](https://github.com/rust-lang/reference/blob/c9cc32825b03fc206d9c6fe73e9ca6a2f3576d6c/src/attributes/testing.md#L15-L50) |
| cargo-nextest | 0.9.149 | this machine | [`how-it-works.md`](https://github.com/nextest-rs/nextest/blob/cargo-nextest-0.9.149/site/src/docs/design/how-it-works.md#L72) |
| `go test`, `go vet` | go 1.26.10 | `nix-shell -p go` | [`alldocs.go` at `go1.26.10`](https://github.com/golang/go/blob/go1.26.10/src/cmd/go/alldocs.go#L1856-L1867) |
| Jest | 30.5.2 | Node 24.22.1 | [`Defaults.ts`](https://github.com/jestjs/jest/blob/v30.5.2/packages/jest-config/src/Defaults.ts#L84-L87), [`GlobalAPI.md`](https://github.com/jestjs/jest/blob/v30.5.2/docs/GlobalAPI.md#L471-L473) |
| Vitest | 5.1.1 | Node 24.22.1 | [`defaults.ts`](https://github.com/vitest-dev/vitest/blob/v5.1.1/packages/vitest/src/defaults.ts#L9), [`include.md`](https://github.com/vitest-dev/vitest/blob/v5.1.1/docs/config/include.md#L8-L16) |
| JUnit 3 and 4 | junit 4.13.2, which ships JUnit 3's `junit.framework` | OpenJDK 21.0.12.1, `JUnitCore` | [`TestSuite.java`](https://github.com/junit-team/junit4/blob/r4.13.2/src/main/java/junit/framework/TestSuite.java#L284-L307), [`BlockJUnit4ClassRunner.java`](https://github.com/junit-team/junit4/blob/r4.13.2/src/main/java/org/junit/runners/BlockJUnit4ClassRunner.java#L144) |
| Maven Surefire | 3.6.1 | not run | [`inclusion-exclusion.md.vm`](https://github.com/apache/maven-surefire/blob/surefire-3.6.1/maven-surefire-plugin/src/site/markdown/examples/inclusion-exclusion.md.vm#L32-L38) |
| Ward | 0.68.0b0 | not run | [`_testing.py`](https://github.com/darrenburns/ward/blob/f41c6164f3923660e3b03c3797913fab23496699/ward/_testing.py#L37-L38), [`writing_tests.rst`](https://github.com/darrenburns/ward/blob/f41c6164f3923660e3b03c3797913fab23496699/docs/source/guide/writing_tests.rst#L10-L20) |
| Cargo targets | cargo 1.99.0 | this machine | [`cargo-targets.md`](https://github.com/rust-lang/cargo/blob/0.100.0/doc/book/src/reference/cargo-targets.md#L316-L360) |
| Next.js | 16.4.6 | not run | [`02-project-structure.mdx`](https://github.com/vercel/next.js/blob/v16.4.6/docs/01-app/01-getting-started/02-project-structure.mdx#L221-L282), [`pageExtensions.mdx`](https://github.com/vercel/next.js/blob/v16.4.6/docs/01-app/03-api-reference/05-config/01-next-config-js/pageExtensions.mdx#L45-L55) |
| Django | 6.1.3 | not run | [`management/__init__.py`](https://github.com/django/django/blob/6.1.3/django/core/management/__init__.py#L29-L39), [`test/runner.py`](https://github.com/django/django/blob/6.1.3/django/test/runner.py#L785-L786) |
| setuptools | 84.2.1 | not run | [`discovery.py`](https://github.com/pypa/setuptools/blob/v84.2.1/setuptools/discovery.py#L210-L290) |
| Ruff | 0.17.6 | the pytest venv | [`helpers.rs`](https://github.com/astral-sh/ruff/blob/0.17.6/crates/ruff_linter/src/rules/flake8_pytest_style/helpers.rs#L85-L95), [`visibility.rs`](https://github.com/astral-sh/ruff/blob/0.17.6/crates/ruff_python_semantic/src/analyze/visibility.rs#L142-L144) |
| coverage.py | 7.17.5 | not run | [`config.py`](https://github.com/nedbat/coveragepy/blob/7.17.5/coverage/config.py#L217) |

### Probes

Each probe is a script under `research/selection-test-markers/<tool>/run.sh`. It writes a small tree into an empty folder that it is given, and runs the tool there. `out.txt` beside it holds the output of this session's run.

```sh
R=research/selection-test-markers
bash $R/python/run.sh <python 3.13 with pytest 9.2.0> <empty dir>
bash $R/cargo/run.sh <empty dir>
nix-shell -p go --run "bash $R/go/run.sh <empty dir>"
nix-shell -p nodejs --run "bash $R/js/run.sh <dir with node_modules/jest@30.5.2 and vitest@5.1.1> <empty dir>"
nix-shell -p jdk --run "bash $R/junit/run.sh <dir with junit-4.13.2.jar and hamcrest-core-1.3.jar> <empty dir>"
bash $R/ruff/run.sh <ruff 0.17.6> <empty dir>
```

### Counts in the 9 suites

The suites are the 9 of [the cost of the rules](https://github.com/kalonji-tools/purba/blob/8e712dbf/prototypes/1015-rules-cost/projects.txt), each at its pinned commit. `measure.py` reads each `.py` file with `ast`, and imports nothing from the suites. It skips each folder whose name starts with `.`.

```sh
mise exec python@3.14 -- python -I research/selection-test-markers/measure.py <suites dir> research/selection-test-markers
```

It writes `files.tsv` (one row for each `.py` file), `functions.tsv` (each module-level function of a test file), `classes.tsv` and `bindings.tsv`. `functions.tsv` and `bindings.tsv` are stored gzipped, and `zcat` reads them. The `typos` hook of the repository would rewrite one of toolz's function names in them. It prints each count of section 4 to `counts.txt`. It ran on CPython 3.14.7, free-threaded; every file parsed.

"The suite's own command" in section 4 is the command that the suite's configuration names, read by hand into `measure.py`. For example, toolz's `testpaths` is `toolz` ([`pyproject.toml` L81](https://github.com/pytoolz/toolz/blob/451af60dec590a6010e2babdbf391ea8f815122f/pyproject.toml#L81)), and pyflakes runs `unittest discover pyflakes` ([`tox.ini` L11](https://github.com/PyCQA/pyflakes/blob/5324946c5d8602697f5df6d98ad162920b3e5d61/tox.ini#L11)). The script does not run the suites, so a `collect_ignore` that a `conftest.py` sets at import is not applied. attrs sets one under a condition ([`conftest.py` L34-L36](https://github.com/python-attrs/attrs/blob/2a1fb86d44816afcc60e86d57956ce3604820326/tests/conftest.py#L34-L36)).

A check by `grep '^def '` over the 189 test files finds 100 public non-test names, where `ast` finds 92. The 8 others are `def` lines inside string literals, in attrs, click and pyflakes [run].

### Where the earlier quick probe differs

An earlier probe, `scratchpad/naming/probe.py`, used the same file matcher and the same folder set. Its totals match mine: 189 test files, 17 under no test folder, 141 other files under test folders, 1,899 `test*` functions, 92 public helpers and 66 private ones. Its definitions hide six facts.

- **It counts the 141 other files under test folders as files beside the tests.** Only 26 sit beside a test file. 118 sit in deeper folders, such as loguru's `tests/exceptions/source/`.
- **It counts `__init__.py` and `conftest.py` with the helpers.** They are 19 of the 141.
- **It reports no file beside the test files of six and schedule.** Each of their test files sits at the root, beside the code under test and `setup.py`.
- **It does not ask which test files the suite itself runs.** 15 of the 17 files under no test folder are benchmarks, and no command of the suite collects them as tests.
- **It counts pytest fixtures as public helpers.** 18 of the 92 are fixtures. pytest itself never collects a fixture as a test ([`python.py` L389-L394](https://github.com/pytest-dev/pytest/blob/9.2.0/src/_pytest/python.py#L389-L394)).
- **It does not show that the prefix `test` also matches `tests_…`.** attrs has 2 such tests.

## 1. How each test runner marks a test file and a test

| runner | marker of a file | marker of a test | where helpers live | source |
|---|---|---|---|---|
| pytest | name: `test_*.py` or `*_test.py`, under `testpaths` or the current folder | name: `test` prefix, at module level or in a class named `Test…` with no `__init__`; also a `unittest.TestCase` method. A fixture is never a test. | a function with another name in the test file; a fixture in `conftest.py`; a helper module that the test imports | [`python.py` L96-L114](https://github.com/pytest-dev/pytest/blob/9.2.0/src/_pytest/python.py#L96-L114), [L389-L394](https://github.com/pytest-dev/pytest/blob/9.2.0/src/_pytest/python.py#L389-L394), [`goodpractices.rst` L100-L117](https://github.com/pytest-dev/pytest/blob/9.2.0/doc/en/explanation/goodpractices.rst#L100-L117); [run] |
| unittest | name: `test*.py`, in a folder that is a package (`__init__.py`) | a method named `test…` of a `TestCase` subclass. A module-level function is never a test. | another method of the class; any module whose name does not match `test*.py` | [`loader.py` L68](https://github.com/python/cpython/blob/v3.13.13/Lib/unittest/loader.py#L68), [L104](https://github.com/python/cpython/blob/v3.13.13/Lib/unittest/loader.py#L104), [L229](https://github.com/python/cpython/blob/v3.13.13/Lib/unittest/loader.py#L229), [L422](https://github.com/python/cpython/blob/v3.13.13/Lib/unittest/loader.py#L422); [run] |
| cargo test | folder: each file directly in `tests/` is a test target. A unit test sits in any source file, under `#[cfg(test)]`. | attribute: `#[test]` on a free function with no arguments. The name is free. | a function with no `#[test]`; `tests/common/mod.rs` | [`cargo-targets.md` L113-L123](https://github.com/rust-lang/cargo/blob/0.100.0/doc/book/src/reference/cargo-targets.md#L113-L123), [reference L15-L31](https://github.com/rust-lang/reference/blob/c9cc32825b03fc206d9c6fe73e9ca6a2f3576d6c/src/attributes/testing.md#L15-L31), [the book L24-L32](https://github.com/rust-lang/book/blob/5c23042f85595490ad9f7412c4c6f57fe0b330d5/src/ch11-03-test-organization.md#L24-L32); [run] |
| cargo-nextest | cargo's test targets | the tests that each test binary lists | as cargo | [`how-it-works.md` L72](https://github.com/nextest-rs/nextest/blob/cargo-nextest-0.9.149/site/src/docs/design/how-it-works.md#L72); [run] |
| `go test` | name: `*_test.go`. `go build` leaves these files out. A name that starts with `_` or `.`, and a folder named `testdata`, are ignored. | name and signature: `TestXxx(t *testing.T)`, where `Xxx` does not start with a lower-case letter | any other function of a `_test.go` file; only the test build sees it. Data goes in `testdata/`. | [`alldocs.go` L1856-L1867](https://github.com/golang/go/blob/go1.26.10/src/cmd/go/alldocs.go#L1856-L1867), [L3225-L3226](https://github.com/golang/go/blob/go1.26.10/src/cmd/go/alldocs.go#L3225-L3226), [L3568-L3574](https://github.com/golang/go/blob/go1.26.10/src/cmd/go/alldocs.go#L3568-L3574); [run] |
| Jest | folder or name: any `.js`/`.ts` file in `__tests__/`, or `*.test.*` / `*.spec.*` | a call: `test(name, fn)` or `it(name, fn)` | any file outside `testMatch`; never inside `__tests__/` | [`Defaults.ts` L84-L87](https://github.com/jestjs/jest/blob/v30.5.2/packages/jest-config/src/Defaults.ts#L84-L87), [`Configuration.md` L2261-L2265](https://github.com/jestjs/jest/blob/v30.5.2/docs/Configuration.md#L2261-L2265), [`GlobalAPI.md` L471-L473](https://github.com/jestjs/jest/blob/v30.5.2/docs/GlobalAPI.md#L471-L473); [run] |
| Vitest | name: `*.test.*` or `*.spec.*`. No folder marker. | a call: `test()` or `it()`, imported from `vitest` unless `globals` is on | any file not named `*.test.*` / `*.spec.*` | [`defaults.ts` L9](https://github.com/vitest-dev/vitest/blob/v5.1.1/packages/vitest/src/defaults.ts#L9), [`globals.md` L8-L11](https://github.com/vitest-dev/vitest/blob/v5.1.1/docs/config/globals.md#L8-L11); [run] |
| JUnit 3 | none in JUnit. Maven Surefire picks `Test*`, `*Test`, `*Tests` and `*TestCase` classes under `src/test/java`. | a public `void` method with no parameter, named `test…`, in a `TestCase` subclass | any other method; a class that matches no Surefire pattern | [`TestSuite.java` L284-L307](https://github.com/junit-team/junit4/blob/r4.13.2/src/main/java/junit/framework/TestSuite.java#L284-L307), [Surefire L32-L38](https://github.com/apache/maven-surefire/blob/surefire-3.6.1/maven-surefire-plugin/src/site/markdown/examples/inclusion-exclusion.md.vm#L32-L38); [run] |
| JUnit 4 | as JUnit 3 | annotation: `@Test` on a public `void` method with no parameter. The name is free. | any method with no `@Test` | [`BlockJUnit4ClassRunner.java` L144](https://github.com/junit-team/junit4/blob/r4.13.2/src/main/java/org/junit/runners/BlockJUnit4ClassRunner.java#L144), [L241](https://github.com/junit-team/junit4/blob/r4.13.2/src/main/java/org/junit/runners/BlockJUnit4ClassRunner.java#L241); [run] |
| Ward (Python) | name: a module named `test_…` or `…_test` | decorator: `@test("description")`. The function is often named `_`. | any function with no `@test` | [`_testing.py` L37-L38](https://github.com/darrenburns/ward/blob/f41c6164f3923660e3b03c3797913fab23496699/ward/_testing.py#L37-L38), [`writing_tests.rst` L10-L20](https://github.com/darrenburns/ward/blob/f41c6164f3923660e3b03c3797913fab23496699/docs/source/guide/writing_tests.rst#L10-L20) |

### What happens to a helper beside a test

Each row is [run] on the probe of its tool, unless it says [source].

| runner | the helper | what happens |
|---|---|---|
| pytest | `helpers.py` beside the tests, holding `def test_in_helper_module()` | not collected: the file name has no marker |
| pytest | `make()` and `_private()` in a test file | not collected |
| pytest | `testable()` in a test file | collected: the prefix `test` matches it |
| pytest | `from test_stock import test_shared` in a test file | collected twice, once in each module |
| pytest | `async def test_async()` | collected |
| pytest | a `helper.py` in `tests/a/` and in `tests/b/`, no `__init__.py` | `tests/b` reads `tests/a`'s helper, and its test fails: `assert 'a' == 'b'`. This is purba's risk [a test imports another directory's module of the same name](https://github.com/kalonji-tools/purba/blob/3093871b/LANDSCAPE.md#L170). |
| pytest | two `test_same.py`, in `same/a/` and `same/b/`, no `__init__.py` | collection stops: `import file mismatch` |
| unittest | `tests/testing_utils.py` | imported as a test module, and its test runs: its name matches `test*.py` |
| unittest | `tests/a/`, `tests/b/` with no `__init__.py` | not entered |
| unittest | the folder `tests/` with no `__init__.py`, discovered from the root | not entered: `Ran 0 tests` |
| unittest | `stock_test.py` | not imported: `*_test.py` does not match `test*.py` |
| cargo test | `tests/common.rs` | built as its own test target: `Running tests/common.rs`, `running 0 tests` |
| cargo test | `tests/support/mod.rs` | no target; `tests/lend.rs` reaches it through `mod support;` |
| cargo test | `fn helper()` in `mod tests` | not a test; `fn lends()` with `#[test]` is one |
| cargo-nextest | the same crate | lists `shop tests::lends` and `shop::lend lends_from_outside`; `common` lists nothing |
| `go test` | `func check(…)` in `shop_test.go` | not a test |
| `go test` | `func Testlower(t *testing.T)` | not a test, and `go vet` reports `Testlower has malformed name` |
| `go test` | `Two()` in `helpers_test.go`, called from `use.go` | `go build` fails: `undefined: Two` |
| `go test` | a broken `.go` file in `testdata/` and in `_old/` | ignored |
| Jest | `src/__tests__/helpers.js` | a test file: `FAIL … Your test suite must contain at least one test.` |
| Jest, Vitest | `src/fixtures.test.js` that holds no test | Jest: `Your test suite must contain at least one test.` Vitest: `No test suite found in file`. |
| Vitest | `src/__tests__/lend.js` | not a test file; Vitest has no folder marker |
| JUnit 3 | `public int make()` in a `TestCase` | not a test |
| JUnit 3 | `public void testWithInput(int n)` | not a test, and no warning |
| JUnit 3 | `void testNotPublic()` | a failed `warning` test: `Test method isn't public` |
| JUnit 4 | `public void testLooksLikeOne()` with no `@Test` | not a test: `OK (1 test)` |
| cargo, Rust book [source] | `tests/common.rs` | the book moves it to `tests/common/mod.rs` so that it shows no `running 0 tests` ([L178-L221](https://github.com/rust-lang/book/blob/5c23042f85595490ad9f7412c4c6f57fe0b330d5/src/ch11-03-test-organization.md#L178-L221)) |

## 2. Tools that find their units by a convention

| tool | marker of a unit | what breaks when a helper sits beside a unit | the way out | source |
|---|---|---|---|---|
| Cargo targets | folder and file: `src/main.rs`, each `src/bin/*.rs`, each `src/bin/<name>/main.rs`, and each file in `examples/`, `tests/`, `benches/` | `src/bin/helper.rs` becomes a binary, and the build fails: ``error[E0601]: `main` function not found in crate `helper` `` [run] | put the helper in `src/bin/<name>/`, beside a `main.rs` [run]; or set `autobins = false` | [`cargo-targets.md` L316-L360](https://github.com/rust-lang/cargo/blob/0.100.0/doc/book/src/reference/cargo-targets.md#L316-L360) |
| Next.js, `pages/` router | folder: each `.js`, `.jsx`, `.ts` or `.tsx` file under `pages/` is a route, named by its file | a helper or a test file under `pages/` becomes a route | `pageExtensions: ['page.tsx', …]`: each page is renamed `*.page.tsx`, and other files may sit beside it | [`01-pages-and-layouts.mdx` L10](https://github.com/vercel/next.js/blob/v16.4.6/docs/02-pages/03-building-your-application/01-routing/01-pages-and-layouts.mdx#L10), [`pageExtensions.mdx` L45-L55](https://github.com/vercel/next.js/blob/v16.4.6/docs/01-app/03-api-reference/05-config/01-next-config-js/pageExtensions.mdx#L45-L55) |
| Next.js, `app/` router | folder and file: a folder is a route segment, and only `page.js` or `route.js` makes it public | nothing: other files are "safely colocated" | a folder named `_folder` is private, and holds no route | [`02-project-structure.mdx` L221-L245](https://github.com/vercel/next.js/blob/v16.4.6/docs/01-app/01-getting-started/02-project-structure.mdx#L221-L245), [L257-L282](https://github.com/vercel/next.js/blob/v16.4.6/docs/01-app/01-getting-started/02-project-structure.mdx#L257-L282) |
| Django, management commands | folder: each module in `<app>/management/commands/` is a command | a helper module there becomes a command | a module whose name starts with `_` is not a command | [`management/__init__.py` L29-L39](https://github.com/django/django/blob/6.1.3/django/core/management/__init__.py#L29-L39) |
| Django, apps | an explicit list, `INSTALLED_APPS`; then a file name in each app: `admin.autodiscover()` imports each app's `admin` module | — | — | [`contrib/admin/__init__.py` L55-L56](https://github.com/django/django/blob/6.1.3/django/contrib/admin/__init__.py#L55-L56) |
| Django, test runner | unittest's marker: `test*.py` | as unittest | as unittest | [`test/runner.py` L785-L786](https://github.com/django/django/blob/6.1.3/django/test/runner.py#L785-L786) |
| setuptools, flat layout | folder: each top-level package, except reserved names; then each top-level module, except reserved names | — | the reserved names hold `test`, `tests`, `conftest`, and each name that starts with `.` or `_` | [`discovery.py` L210-L252](https://github.com/pypa/setuptools/blob/v84.2.1/setuptools/discovery.py#L210-L252), [L263-L290](https://github.com/pypa/setuptools/blob/v84.2.1/setuptools/discovery.py#L263-L290) |

Two ways out recur. A unit gets a marker of its own beside the folder: `page.tsx`, `*.page.tsx`, `src/bin/<name>/main.rs`. Or a helper gets a mark that says "not a unit": `_folder`, a `_` module in Django, `tests/common/mod.rs`. judgement: no tool in this survey keeps a folder as the only marker without one of these.

## 3. Other tools that read the same markers

| tool | marker it reads | default | source |
|---|---|---|---|
| Ruff, `PT` rules | the name: a module-level function whose name starts with `test`, or such a method in a class named `Test…` | on when `PT` is selected. `PT028` reports `def test_lend(s: int = use(shop))`, and not the same body named `check_lend` [run]. | [`helpers.rs` L85-L95](https://github.com/astral-sh/ruff/blob/0.17.6/crates/ruff_linter/src/rules/flake8_pytest_style/helpers.rs#L85-L95), [`test_functions.rs` L56](https://github.com/astral-sh/ruff/blob/0.17.6/crates/ruff_linter/src/rules/flake8_pytest_style/rules/test_functions.rs#L56) |
| Ruff, `D401` | the name: a function named `runTest` or `test…` is skipped | `D401` reports `check_lend`'s docstring and not `test_lend`'s [run] | [`visibility.rs` L142-L144](https://github.com/astral-sh/ruff/blob/0.17.6/crates/ruff_python_semantic/src/analyze/visibility.rs#L142-L144), [`non_imperative_mood.rs` L74](https://github.com/astral-sh/ruff/blob/0.17.6/crates/ruff_linter/src/rules/pydocstyle/rules/non_imperative_mood.rs#L74) |
| Ruff, `per-file-ignores` | a glob that the project writes | none | attrs: `"**/test_*"` ([L268](https://github.com/python-attrs/attrs/blob/2a1fb86d44816afcc60e86d57956ce3604820326/pyproject.toml#L268)); loguru: `"tests/**"` ([L124](https://github.com/Delgan/loguru/blob/28349be3cd740867bc947eef741c69370ca45735/pyproject.toml#L124)); requests: `"tests/compat.py"` ([L101](https://github.com/psf/requests/blob/3c7baaa05ab4aa49cc819ce3a819d260de19c157/pyproject.toml#L101)) |
| coverage.py, `omit` | a glob that the project writes | empty | [`config.py` L217](https://github.com/nedbat/coveragepy/blob/7.17.5/coverage/config.py#L217); toolz: `toolz/tests/test*` ([L73-L74](https://github.com/pytoolz/toolz/blob/451af60dec590a6010e2babdbf391ea8f815122f/pyproject.toml#L73-L74)) |
| Vitest coverage | Vitest's own file marker, `include` | test files are left out of coverage | [`include.md` L16](https://github.com/vitest-dev/vitest/blob/v5.1.1/docs/config/include.md#L16) |
| pytest's assert rewrite | `python_files`, the file marker | rewrites only test files and `conftest.py` | [`python.py` L100](https://github.com/pytest-dev/pytest/blob/9.2.0/src/_pytest/python.py#L100) |
| `go vet`, `tests` analyzer | the name `Test…` | on | [`vet/doc.go` L58](https://github.com/golang/go/blob/go1.26.10/src/cmd/vet/doc.go#L58); [run] |
| setuptools, flat layout | the folder names `test`, `tests`; the module names `test`, `tests`, `conftest` | on when the project lists no packages | [`discovery.py` L222-L223](https://github.com/pypa/setuptools/blob/v84.2.1/setuptools/discovery.py#L222-L223), [L266-L268](https://github.com/pypa/setuptools/blob/v84.2.1/setuptools/discovery.py#L266-L268) |
| pytest's `testpaths` | a folder that the project writes | the current folder | loguru, attrs, click, requests: `tests`; toolz: `toolz` ([`main.py` L244-L250](https://github.com/pytest-dev/pytest/blob/9.2.0/src/_pytest/main.py#L244-L250)) |

The suites set more settings that name a test folder or test files [run, by `grep`]:

| suite | setting | marker it names |
|---|---|---|
| click | coverage `source = ["click", "tests"]`, mypy `files`, pyright `include` of `tests/typing` ([L95](https://github.com/pallets/click/blob/e38188e9e205005d5af5f9e59df51904c984586a/pyproject.toml#L95), [L109](https://github.com/pallets/click/blob/e38188e9e205005d5af5f9e59df51904c984586a/pyproject.toml#L109), [L116](https://github.com/pallets/click/blob/e38188e9e205005d5af5f9e59df51904c984586a/pyproject.toml#L116)) | folder |
| loguru | black `force-exclude` and ruff `exclude` of `tests/exceptions/source/…` ([L87](https://github.com/Delgan/loguru/blob/28349be3cd740867bc947eef741c69370ca45735/pyproject.toml#L87), [L113](https://github.com/Delgan/loguru/blob/28349be3cd740867bc947eef741c69370ca45735/pyproject.toml#L113)) | folder |
| dateutil | `python_files` adds `*_solution.py`, and tox runs `pytest tests docs` ([L98-L102](https://github.com/dateutil/dateutil/blob/a78cbd81d5d03b5f126eb36edb92bca78972c804/pyproject.toml#L98-L102), [`tox.ini` L19](https://github.com/dateutil/dateutil/blob/a78cbd81d5d03b5f126eb36edb92bca78972c804/tox.ini#L19)) | a third file name |
| toolz, pyflakes | the package list ships `toolz.tests` and `pyflakes.test` ([toolz L62-L67](https://github.com/pytoolz/toolz/blob/451af60dec590a6010e2babdbf391ea8f815122f/pyproject.toml#L62-L67), [pyflakes `setup.py` L44](https://github.com/PyCQA/pyflakes/blob/5324946c5d8602697f5df6d98ad162920b3e5d61/setup.py#L44)) | folder |
| schedule | tox runs `py.test test_schedule.py schedule` ([`tox.ini` L23](https://github.com/dbader/schedule/blob/82a43db1b938d8fdf60103bd41f329e06c8d3651/tox.ini#L23)) | one file by name |

## 4. The markers in the 9 pinned suites

Each count is [run], from `counts.txt`. The matcher of each row is in its first column.

### 4.1 Test files and test folders

| count | loguru | attrs | click | requests | dateutil | pyflakes | toolz | six | schedule | total |
|---|---|---|---|---|---|---|---|---|---|---|
| all `.py` files | 173 | 56 | 91 | 37 | 45 | 24 | 51 | 4 | 4 | 485 |
| all `.py` files under a test folder | 153 | 31 | 59 | 15 | 23 | 15 | 17 | 0 | 0 | 313 |
| test files (`test_*.py` or `*_test.py`) | 50 | 26 | 43 | 9 | 17 | 13 | 29 | 1 | 1 | 189 |
| … under a test folder | 50 | 25 | 43 | 9 | 17 | 13 | 15 | 0 | 0 | 172 |
| … **under no test folder** | 0 | 1 | 0 | 0 | 0 | 0 | 14 | 1 | 1 | **17** |
| … collected by the suite's own command | 50 | 25 | 43 | 9 | 17 | 13 | 15 | 1 | 1 | 174 |
| … under no test folder, and collected by the suite's own command | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 1 | 1 | 2 |
| files that the suite's own command collects and that are not test files | 0 | 0 | 0 | 0 | 1 | 0 | 0 | 0 | 0 | 1 |

The 15 test files under no test folder that no command collects are benchmarks: `attrs/bench/test_benchmarks.py` and 14 files of `toolz/bench/`. The one file collected with no test name is dateutil's `docs/exercises/solutions/mlk_day_rrule_solution.py`.

### 4.2 Other `.py` files beside the test files

| count | loguru | attrs | click | requests | dateutil | pyflakes | toolz | six | schedule | total |
|---|---|---|---|---|---|---|---|---|---|---|
| **other `.py` files beside a test file** | 2 | 6 | 4 | 4 | 3 | 2 | 2 | 2 | 1 | **26** |
| … `__init__.py` | 1 | 1 | 3 | 1 | 1 | 1 | 2 | 0 | 0 | 10 |
| … `conftest.py` | 1 | 1 | 1 | 1 | 1 | 0 | 0 | 0 | 0 | 5 |
| … other modules | 0 | 4 | 0 | 2 | 1 | 1 | 0 | 2 | 1 | 11 |
| … … that a test file imports, by module name | 0 | 3 | 0 | 2 | 1 | 1 | 0 | 1 | 0 | 8 |
| other `.py` files under a test folder, at any depth | 103 | 6 | 16 | 6 | 6 | 2 | 2 | 0 | 0 | 141 |
| … in a deeper folder, beside no test file | 101 | 0 | 12 | 2 | 3 | 0 | 0 | 0 | 0 | 118 |
| … public module-level functions in these 141 files | 215 | 7 | 13 | 8 | 16 | 0 | 0 | 0 | 0 | 259 |
| … module-level `test*` functions in these 141 files | 20 | 0 | 2 | 0 | 0 | 0 | 0 | 0 | 0 | 22 |

The 11 other modules beside a test file: `attrs/tests/{attr_import_star,dataclass_transform_example,strategies,utils}.py`, `requests/tests/{compat,utils}.py`, `dateutil/tests/_common.py`, `pyflakes/pyflakes/test/harness.py`, `six/six.py`, `six/setup.py` and `schedule/setup.py`. `six.py` is the code under test. The 101 deeper files of loguru are sample scripts under `tests/exceptions/source/`. The 12 of click are `tests/typing/*.py`, which mypy and pyright check. The "imported" column matches a module's name against the last part of each import in the suite's test files; it is a heuristic.

### 4.3 Module-level functions in test files

| count | loguru | attrs | click | requests | dateutil | pyflakes | toolz | six | schedule | total |
|---|---|---|---|---|---|---|---|---|---|---|
| module-level functions | 766 | 75 | 643 | 58 | 261 | 2 | 186 | 65 | 1 | 2057 |
| `test*` | 720 | 63 | 619 | 56 | 209 | 0 | 167 | 65 | 0 | 1899 |
| … with any decorator | 306 | 4 | 233 | 29 | 137 | 0 | 1 | 9 | 0 | 719 |
| … `async` | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 |
| … named `test*` but not `test_*` | 0 | 2 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 2 |
| **public, not `test*`** | 43 | 5 | 1 | 2 | 20 | 1 | 19 | 0 | 1 | **92** |
| … test files that hold one | 13 | 4 | 1 | 2 | 7 | 1 | 9 | 0 | 1 | 38 |
| … with a fixture decorator | 9 | 0 | 0 | 0 | 9 | 0 | 0 | 0 | 0 | 18 |
| … with another decorator | 1 | 0 | 0 | 0 | 4 | 0 | 0 | 0 | 0 | 5 |
| … with no decorator, and a parameter with no default | 29 | 4 | 0 | 2 | 7 | 1 | 17 | 0 | 0 | 60 |
| … with no decorator, and no parameter without a default | 4 | 1 | 1 | 0 | 0 | 0 | 2 | 0 | 1 | 9 |
| … `async` (all 5 inside the 60) | 5 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 5 |
| **private** | 3 | 7 | 23 | 0 | 32 | 1 | 0 | 0 | 0 | **66** |
| … test files that hold one | 2 | 7 | 5 | 0 | 7 | 1 | 0 | 0 | 0 | 22 |
| … with a fixture decorator | 1 | 6 | 2 | 0 | 0 | 0 | 0 | 0 | 0 | 9 |

The 2 names `test*` but not `test_*` are `tests_weakref_does_not_add_…` in `attrs/tests/test_slots.py`. pytest collects them; so does purba's rule today.

### 4.4 Classes, and names bound with no `def`

| count | loguru | attrs | click | requests | dateutil | pyflakes | toolz | six | schedule | total |
|---|---|---|---|---|---|---|---|---|---|---|
| module-level classes in test files | 20 | 126 | 24 | 23 | 43 | 31 | 8 | 2 | 2 | 279 |
| … holding a `test*` method | 0 | 84 | 0 | 21 | 32 | 23 | 1 | 2 | 1 | 164 |
| `test*` methods in them | 0 | 620 | 0 | 291 | 924 | 564 | 15 | 10 | 81 | 2505 |
| `test*` names that a test file binds at module level with no `def` | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 |
| public names, not `test*`, that a `from … import` binds at module level | 106 | 123 | 72 | 87 | 125 | 65 | 139 | 0 | 7 | 724 |
| … test files with one | 49 | 22 | 18 | 7 | 15 | 13 | 23 | 0 | 1 | 141 |

purba refuses each of the 2,505 methods today ([record L74](https://github.com/kalonji-tools/purba/blob/3093871b/docs/decisions/a-test-returns-nothing-and-the-first-part-that-sees-a-breach-refuses-it.md#L74)). A port moves each one to the module's level, where the marker of a function applies.

### 4.5 The latest form's own ports

`form_ports.py` sorts the module-level functions of the form's ported files on [`proto/1015-form` at `107b09a6`](https://github.com/kalonji-tools/purba/tree/107b09a6/prototypes/1015-form/benchmark) [run, `form_ports.txt`].

| file | `test*` | public, not `test*` | private |
|---|---|---|---|
| `flaskr-blog/form/test_blog.py` | 8 | 3: `app`, `client`, `auth` | 0 |
| `flaskr-blog/form-2/test_blog.py` | 8 | 0 | 0 |
| `flaskr-blog/form-2/flaskr_fixtures.py`, beside it | 0 | 2: `client`, `auth` | 0 |
| `loguru-format/form/test_add_option_format.py` | 8 | 1: `reset_logger` | 0 |

Each public non-test function is a fixture or a function that `@before` takes. The second flaskr port moved its fixtures into a module beside the test file, with no marker in its name.

## 5. Four sets of markers against purba's records

### What purba holds today

- **Code holds the name marker.** Parsing keeps a `def` only when its name starts with `test` ([`facts.rs` L294](https://github.com/kalonji-tools/purba/blob/3093871b/crates/parsing/src/facts.rs#L294)). Selection refuses a `test` name bound with no `def` ([`lib.rs` L454](https://github.com/kalonji-tools/purba/blob/3093871b/crates/selection/src/lib.rs#L454)). It treats a module-level `def` with another name as a helper when it looks for a foreign framework ([L1042](https://github.com/kalonji-tools/purba/blob/3093871b/crates/selection/src/lib.rs#L1042)).
- **No code holds a file marker.** The command takes one path to any `.py` file ([`lib.rs` L211](https://github.com/kalonji-tools/purba/blob/3093871b/crates/command/src/lib.rs#L211), [L37](https://github.com/kalonji-tools/purba/blob/3093871b/crates/command/src/lib.rs#L37)).
- **No record decides a marker.** [The term "test"](https://github.com/kalonji-tools/purba/blob/3093871b/CONTEXT.md#L341) names no rule for a name. Only the form says it ([`FORM.md` L9](https://github.com/kalonji-tools/purba/blob/107b09a6/prototypes/1015-form/FORM.md#L9)).
- **One need names a folder.** [`needs.toml` L9](https://github.com/kalonji-tools/purba/blob/3093871b/.config/needs.toml#L9) states "find the test files under a directory".
- **Each set must be readable without an import.** Selection imports nothing ([lazy work](https://github.com/kalonji-tools/purba/blob/3093871b/LANDSCAPE.md#L141)), and only parsing reads source ([one parser](https://github.com/kalonji-tools/purba/blob/3093871b/LANDSCAPE.md#L145)). A folder and a file name are read from the path, in stage one. A name and a decorator are read from the tree, in stage two. Each of the four sets meets this.
- **Each set but the first changes a form that a tester writes.** Such an answer owes a blind rating before the person decides ([record L28](https://github.com/kalonji-tools/purba/blob/3093871b/docs/decisions/a-form-that-a-tester-meets-is-rated-blind-before-it-is-decided.md#L28)).

### 5.1 The file and the function both carry the marker, as today

```python
# tests/test_blog.py                     id: tests/test_blog.py::test_index
def _login(client): ...
def test_index(client: Client = use(client)) -> None: ...
```

- **Stage one** reads the file name. **Stage two** reads the function name. A folder may hold any helper module, since its name carries no marker.
- **Measured cost.** 1,802 of 1,899 module-level tests carry "test" in the folder, the file and the function. A public helper stays free: 92 in 38 files. The prefix catches `tests_…` (2 tests). 2 test files at a suite's root keep their marker.
- **Other tools.** Ruff's `PT` and `D401` and pytest read the same name. Ruff's `PT028` reports each purba test that takes an input, since [an input is a typed default](https://github.com/kalonji-tools/purba/blob/3093871b/docs/decisions/a-test-names-the-fixture-of-each-input.md#L37) [run].
- **Records.** It contradicts none. A record would state it for the first time. Four refusals exist only because the name is the marker ([L46-L47, L58-L59](https://github.com/kalonji-tools/purba/blob/3093871b/docs/decisions/a-test-carries-only-purbas-own-decorators.md#L46-L59)). In the 9 suites, no test file binds a `test*` name with no `def`.

### 5.2 Only the file carries the marker, and a helper's name starts with `_`

```python
# tests/test_blog.py                     id: tests/test_blog.py::index
def _client() -> Client: ...
def index(client: Client = use(_client)) -> None: ...
```

- **Stage one** reads the file name. **Stage two** takes each public `def` at module level as a test.
- **Measured cost.** The 92 public helpers in 38 test files become tests. 18 carry a pytest fixture decorator and 5 another decorator; selection refuses each, unless a suppression lifts it ([L40](https://github.com/kalonji-tools/purba/blob/3093871b/docs/decisions/a-test-carries-only-purbas-own-decorators.md#L40), [L45](https://github.com/kalonji-tools/purba/blob/3093871b/docs/decisions/a-test-carries-only-purbas-own-decorators.md#L45)). 60 take a parameter with no default, which selection refuses ([record L49](https://github.com/kalonji-tools/purba/blob/3093871b/docs/decisions/a-failures-kind-is-what-the-part-that-first-sees-it-observes.md#L49)). 9 take no such parameter, and would run as tests. A test that states no expectation is not reported as passed ([the driver scenario](https://github.com/kalonji-tools/purba/blob/3093871b/LANDSCAPE.md#L29)). judgement: so a missed `_` shows as a refusal or a failed run, not as a silent pass. That is unmeasured on a wheel.
- **The form's fixtures.** The form defines a fixture as a public `def` in the test file ([`FORM.md` L116-L121](https://github.com/kalonji-tools/purba/blob/107b09a6/prototypes/1015-form/FORM.md#L116-L121)). Each fixture would need `_`, or would move to a module beside the test file. All 4 such functions in the form's ports are public (section 4.5).
- **Code that changes.** The rule at [`facts.rs` L294](https://github.com/kalonji-tools/purba/blob/3093871b/crates/parsing/src/facts.rs#L294) and at [`lib.rs` L454](https://github.com/kalonji-tools/purba/blob/3093871b/crates/selection/src/lib.rs#L454) and [L1042](https://github.com/kalonji-tools/purba/blob/3093871b/crates/selection/src/lib.rs#L1042) changes from "starts with `test`" to "is not private". Selection already has a test of privacy ([`is_private`, L896-L898](https://github.com/kalonji-tools/purba/blob/3093871b/crates/selection/src/lib.rs#L896-L898)).
- **Records that change.**
  - **The private name.** purba reports each [private name](https://github.com/kalonji-tools/purba/blob/3093871b/CONTEXT.md#L229) that a test reads, when the name is an attribute or is imported ([record L101](https://github.com/kalonji-tools/purba/blob/3093871b/docs/decisions/a-test-returns-nothing-and-the-first-part-that-sees-a-breach-refuses-it.md#L101)). A `_helper` of the same file gives no notice: the code reads only attributes and imported names ([`lib.rs` L1121-L1130](https://github.com/kalonji-tools/purba/blob/3093871b/crates/selection/src/lib.rs#L1121-L1130)). A `_helper` imported from a module beside the test gives a notice on each line of each test that reads it. judgement: under this set, the notice then reports the helpers that the set asks for.
  - **The names bound with no `def`.** The records refuse a test name that the module binds with no `def`, or imports with `from` ([L47](https://github.com/kalonji-tools/purba/blob/3093871b/docs/decisions/a-test-carries-only-purbas-own-decorators.md#L47), [L58](https://github.com/kalonji-tools/purba/blob/3093871b/docs/decisions/a-test-carries-only-purbas-own-decorators.md#L58)). If every public name were a test name, these rules would refuse 724 names that `from … import` binds, in 141 of 189 test files. judgement: the rules would then hold only for a `def`, or would go.
  - **The form.** [`FORM.md` L9](https://github.com/kalonji-tools/purba/blob/107b09a6/prototypes/1015-form/FORM.md#L9) changes, and the change owes a blind rating.
- **Other tools.** Ruff's `PT` rules and `D401` no longer see a test as a test [run: `check_lend` gets no `PT028`]. pytest collects none of these tests.

### 5.3 Only the folder carries the marker

Two readings exist. The issue's set names only the folder.

```python
# tests/blog.py                          id: tests/blog.py::index        (folder alone)
def _client() -> Client: ...
def index(client: Client = use(_client)) -> None: ...

# tests/blog.py                          id: tests/blog.py::test_index   (folder, and the name)
def client() -> Client: ...
def test_index(client: Client = use(client)) -> None: ...
```

- **Stage one** takes each `.py` file under the folder. **Stage two** takes each public `def` (folder alone), or each `test…` `def` (folder and name).
- **Measured cost, files.** The folder misses 17 of 189 test files. The suites' own commands collect 2 of them, at the root of six and schedule. It adds 141 other files under test folders: 19 `__init__.py` or `conftest.py`, and 118 in deeper folders. Stage two then parses 313 files, where the file marker gives 172 under test folders.
- **Measured cost, functions.** With the folder and the name, 22 `test*` functions in the 141 files become tests: 20 in loguru's sample scripts, 2 in click's typing checks. With the folder alone, 259 public functions in those files become tests, beside the 92 of section 5.2.
- **What breaks beside a test.** Each `conftest.py` and each helper module under the folder becomes a test module. Each of the 5 `conftest.py` files beside the tests imports pytest and defines fixtures through it [run, by `grep`]; a test module may not import pytest ([record L28](https://github.com/kalonji-tools/purba/blob/3093871b/docs/decisions/a-test-module-imports-no-other-test-framework.md#L28)). judgement: a helper module that is also a test module is also exposed to [the risk of two `helper.py`](https://github.com/kalonji-tools/purba/blob/3093871b/LANDSCAPE.md#L170) as a test; that is unmeasured. Each tool in section 2 that marks by folder alone added a second marker.
- **Records.** It fits [`needs.toml` L9](https://github.com/kalonji-tools/purba/blob/3093871b/.config/needs.toml#L9). The folder-alone reading carries every change of section 5.2. Either reading changes [`FORM.md` L9](https://github.com/kalonji-tools/purba/blob/107b09a6/prototypes/1015-form/FORM.md#L9), and owes a blind rating.
- **Other tools.** Ruff's `per-file-ignores`, coverage's `omit`, pytest's `testpaths` and setuptools' flat layout can all name a folder. pytest's `python_files` and assert rewrite do not see `tests/blog.py` as a test file. judgement: a suite that pytest and purba both run during a port would need `python_files = *.py` for pytest.

### 5.4 A decorator of purba marks each test

```python
# tests/test_blog.py                     id: tests/test_blog.py::index
from purba import test

def client() -> Client: ...

@test
def index(client: Client = use(client)) -> None: ...
```

- **Stage one** still needs a marker of a file or a folder. With none, stage two parses each `.py` file that the walk meets: 485 in the 9 suites, against 189 test files. **Stage two** takes each `def` that carries `@test`.
- **Measured cost.** Each of the 1,899 module-level tests gains one line. 719 of them carry a decorator already. A helper keeps any name. The 2,505 methods in classes stay refused, as today.
- **Prior art.** Ward marks each test with `@test("…")`, and its files by name ([`writing_tests.rst` L18](https://github.com/darrenburns/ward/blob/f41c6164f3923660e3b03c3797913fab23496699/docs/source/guide/writing_tests.rst#L18), [`_testing.py` L37-L38](https://github.com/darrenburns/ward/blob/f41c6164f3923660e3b03c3797913fab23496699/ward/_testing.py#L37-L38)). cargo's `#[test]` and JUnit 4's `@Test` are the same marker in other languages.
- **Records that fit.** A test carries only purba's own decorators, named through `purba` or imported from it with no alias ([L37-L44](https://github.com/kalonji-tools/purba/blob/3093871b/docs/decisions/a-test-carries-only-purbas-own-decorators.md#L37-L44)). selection already resolves a decorator through the module's imports (L41). A decorator of purba returns the function it takes (L43).
- **Records that change.**
  - **The public surface.** A decorator of purba belongs to the [public surface](https://github.com/kalonji-tools/purba/blob/3093871b/docs/decisions/a-test-carries-only-purbas-own-decorators.md#L39), which [starts empty](https://github.com/kalonji-tools/purba/blob/3093871b/docs/decisions/purbas-public-surface-starts-empty-and-has-no-plugin-api.md#L35). `test` joins it with a milestone ([L37](https://github.com/kalonji-tools/purba/blob/3093871b/docs/decisions/purbas-public-surface-starts-empty-and-has-no-plugin-api.md#L37)). Each test module then imports purba.
  - **The term "test".** [CONTEXT.md L341](https://github.com/kalonji-tools/purba/blob/3093871b/CONTEXT.md#L341) would add that a test carries `@test`.
  - **The name refusals.** The refusals of a test name bound with no `def`, imported with `from`, or holding a value ([L47](https://github.com/kalonji-tools/purba/blob/3093871b/docs/decisions/a-test-carries-only-purbas-own-decorators.md#L47), [L58-L59](https://github.com/kalonji-tools/purba/blob/3093871b/docs/decisions/a-test-carries-only-purbas-own-decorators.md#L58-L59)) lose their subject. judgement: the refusal of a name bound again (L46) still applies to the name of a `def` that carries `@test`.
  - **The form.** The form lists `@before`, `@skip` and `@skip_if` as the only decorators ([`FORM.md` L174](https://github.com/kalonji-tools/purba/blob/107b09a6/prototypes/1015-form/FORM.md#L174)). It adds `@test`, and owes a blind rating.
- **Other tools.** With a name that does not start with `test`, Ruff's `PT028` no longer fires on a test's input [run], and pytest collects nothing.
- **A forgotten marker.** judgement: a `def` with no `@test` is a helper, and prints no line. A `def` whose name lacks the prefix (`index_works`) under the name marker prints no line either. Of the runners probed, only `go vet` reports a malformed test name [run], and only JUnit 3 reports a test method that is not public [run].

### 5.5 The four sets side by side

| set | stage one reads | stage two reads | what a helper must do | measured in the 9 suites | records and code that change |
|---|---|---|---|---|---|
| file and function, as today | `test_*.py` | `test…` | nothing | 1,802 tests carry the marker 3 times | none; a record would state it ([`facts.rs` L294](https://github.com/kalonji-tools/purba/blob/3093871b/crates/parsing/src/facts.rs#L294)) |
| file only, `_` helper | `test_*.py` | each public `def` | start with `_` | 92 public helpers in 38 files; 724 `from` imports in 141 files | name rule in code; private-name notice ([L101](https://github.com/kalonji-tools/purba/blob/3093871b/docs/decisions/a-test-returns-nothing-and-the-first-part-that-sees-a-breach-refuses-it.md#L101)); name refusals ([L47, L58](https://github.com/kalonji-tools/purba/blob/3093871b/docs/decisions/a-test-carries-only-purbas-own-decorators.md#L47-L58)); form's fixtures; `FORM.md` L9 |
| folder only | each `.py` under the folder | each public `def`, or each `test…` | sit outside the folder, or start with `_` | 17 test files missed (2 run); 141 other files added; 22 or 259 extra tests there | as the row above, or none in code for the name; foreign-framework refusal on `conftest.py` ([L28](https://github.com/kalonji-tools/purba/blob/3093871b/docs/decisions/a-test-module-imports-no-other-test-framework.md#L28)); `FORM.md` L9 |
| decorator of purba | a file or folder marker, or every `.py` (485) | each `def` with `@test` | nothing | 1,899 added lines; 719 tests already decorated | public surface ([L35-L37](https://github.com/kalonji-tools/purba/blob/3093871b/docs/decisions/purbas-public-surface-starts-empty-and-has-no-plugin-api.md#L35-L37)); term "test"; name refusals lose their subject; `FORM.md` L174 |

## 6. What this file does not measure

- **purba's own lines for each set.** No probe ran a purba wheel. Each line about what selection would print under a new set is a judgement from the records and the code.
- **A tester's reading.** No set was rated blind. Each change of the form owes one.
- **pytest's collection of the suites.** The script reads each suite's command by hand. It does not apply a `collect_ignore` set at import, nor pytest's `--doctest-modules` in requests.
- **The heuristic "imported by a test file".** It matches a module's name against the last part of each import. It can match a module of the same name elsewhere.
- **Next.js, Django and setuptools** are [source] only. I did not run them.

## Files

| path | what it holds |
|---|---|
| `research/selection-test-markers/measure.py` | the count script for the 9 suites |
| `research/selection-test-markers/counts.txt` | its output, with every count of section 4 and the lists of files |
| `research/selection-test-markers/{files,functions,classes,bindings}.tsv` | one row per file, function, class and module-level binding |
| `research/selection-test-markers/form_ports.py`, `form_ports.txt` | the count of the form's ports, and its output |
| `research/selection-test-markers/{python,cargo,go,js,junit,ruff}/run.sh`, `out.txt` | each probe, and its output in this session |
