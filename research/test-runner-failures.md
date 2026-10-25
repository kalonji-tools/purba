# Which failures test runners' users meet, and which purba can meet

This file answers [#654](https://github.com/kalonji-tools/purba/issues/654).
It is never merged.

## Answer

Users meet a module that fails to load more often than any other failure.
It leads, or nearly leads, the counts in pytest's, Jest's and Vitest's trackers, and it holds four of the nine headings of Vitest's troubleshooting page.
In pytest, most of those reports are about the import path or the environment, not about code that raised: 30 of the 40 newest `ImportError while importing test module` issues are a module that is not found, or the wrong one.
Second comes the failed assertion and how it is shown, which leads Vitest's counts and is second in Jest's.
Then come the import path, fixture setup and teardown, and errors inside the runner (pytest's `INTERNALERROR`, 295 issues); pytest's tracker alone measures the last two.
The runners disagree on what a module that fails to load does to the rest of the run.
`unittest`, `go test`, Jest and Vitest run the other files; pytest stops the run by default; `cargo test` and nextest run nothing.
A pytest maintainer calls the stop intentional.
Both Python runners take the failed module out of `sys.modules` in their default mode, because `importlib` does; pytest's `importlib` mode leaves it in, as purba does today.
Six of the seven runners print a test that raised as a failed test; only `unittest` keeps it apart, and pytest keeps apart only a raise in setup or teardown.
oxitest's shared errors drew 13 fixes, and 9 of them were fixture errors; 6 showed the wrong kind, and 5 showed the right kind with the wrong text.
Of the 20 kinds, purba can meet 13 now and 5 only later. It never meets 1, an error raised by a plugin or a hook, and for 1 I could not establish which part would bring it.

| kind of failure | runners that name it apart | how often users meet it (estimate) | purba | inventory rows |
|---|---|---|---|---|
| a module fails to load: it raises, does not parse, or imports what is missing | all 7 | highest: pytest `ERROR collecting` 364; Jest `Test suite failed to run` 549; Vitest `Failed to load` 95 | now | 7, 7b, 7d, 8, 9, 10 |
| an error inside the runner | pytest, nextest ([#616](https://github.com/kalonji-tools/purba/blob/9b3fcc4/research/exit-codes.md#2-the-runners-own-failure)) | high in pytest: `INTERNALERROR` 295 | now | 6, 6b, 7c, 7g, 29, 29b, 31, 33, 34 |
| a failed assertion | all 7 | high: Jest `expect(received)` 460; Vitest `AssertionError` 141; pytest `topic: rewrite` 138 | now | 12c, 26 |
| a fixture or hook fails in setup or teardown | pytest, `unittest`, Vitest, nextest | high in pytest: `ERROR at setup of` 151, `ERROR at teardown of` 66 | later: fixtures | none |
| a usage or configuration error | all 7 | medium: pytest `topic: config` 202, `unrecognized arguments` 69 | now for usage; later for configuration | 1, 2, 4, 5, 5b, 5c, 34 |
| the import path finds the wrong module, or none | pytest (`import file mismatch`) | medium: pytest `import file mismatch` 32, plus 30 of 40 sampled import errors | now in part; later: execution | 31 |
| a test hangs | nextest, `go test`, Jest, Vitest | medium: Go `test timed out after` 240; pytest `hang` in a title 44 | later: dispatch | none run |
| a test names an input that nothing provides | pytest | medium: pytest `available fixtures` 101 | now as a `TypeError`; later: fixtures | 21 |
| a test raises an error that is not an assertion | `unittest` | [C]: no printed phrase sets it apart | now | 11, 12, 13, 27, 30 |
| an error outside any test (a thread, an unhandled rejection) | Vitest, pytest as a warning | medium in Vitest: `Unhandled Errors` 129; low in pytest | now | 35 |
| the process crashes | nextest, `cargo test` | low: pytest `Segmentation fault` 35; Jest 16; Vitest 17 | now | 7g, 10, 14, 14b, 20 |
| no test found | pytest, `unittest`, nextest, Jest, Vitest | low to medium: Jest 43; Vitest 26 | now | 7b, 25 |
| a test leaks a handle or a process | nextest, Jest | low: Jest 26; nextest 11 | later: dispatch | none |
| an expected failure passes | pytest, `unittest` | low: pytest `XPASS(strict)` 19; CPython 13 | later: strictness | none |
| a test shape the runner cannot run (async, generator, a value returned) | pytest, `unittest` as a warning | low: pytest 5 + 5 | now | 22, 24, 25 |
| the report cannot be written | not surveyed | low: pytest `BrokenPipeError` 7 | now | 17, 17c, 17d, 17e, 36 |
| an interrupt | not surveyed | medium: pytest `KeyboardInterrupt` 73 | now; out of this map's scope | 15, 18, 18b, 19 |
| a test declaration is invalid (parametrize, marks) | pytest | medium: pytest `topic: parametrize` 184 | later: [C] no part names it | none |
| a test that passes on a retry | nextest | low | [C]: no part or record names retries | none |
| a plugin, a hook or a `conftest.py` fails | pytest | medium in pytest: about 7 of 40 sampled `INTERNALERROR` issues; `ImportError while loading conftest` 24 | never | none |

Counts are GitHub search totals, made on 2026-10-25, and are estimates (item 2).
The "purba" column is a judgment against [LANDSCAPE.md](https://github.com/kalonji-tools/purba/blob/efb4c9c/LANDSCAPE.md) (item 4).

## 1. The kinds each runner names, and what a module that fails to load does

### 1a. What a module that fails to load does to the run

| runner | the case | do the other tests run? | what it prints (exact) | exit | the module in `sys.modules` after | tier |
|---|---|---|---|---|---|---|
| pytest 9.0.3, default (`prepend`) | `raise ValueError("x")` at the top of `test_a.py`; `test_b.py` passes | **no**: the run stops after collection | `ERROR collecting test_a.py`, `E   ValueError: x`, `!!!… Interrupted: 1 error during collection !!!…`, `1 error in 0.06s` | 2 | not observed: no test ran | [A] M1 `p1-default` |
| pytest, `--continue-on-collection-errors` | the same | yes | `1 passed, 1 error` | 1 | no | [A] M1 `p1-continue` |
| pytest, `--import-mode=append` | the same | yes, with the flag | the same | 1 | no | [A] M1 `p1-append` |
| pytest, `--import-mode=importlib` | the same | yes, with the flag | the same | 1 | **yes**: `test_b` saw `test_a` in `sys.modules`, with its `__file__` | [A] M1 `p1-importlib`, `p15-importlib` |
| pytest | a syntax error | no | `ERROR collecting test_a.py`, a traceback through `_pytest/python.py`, `_pytest/assertion/rewrite.py` and `ast.py`, then `E   SyntaxError: invalid syntax` | 2 | — | [A] M1 `p2` |
| pytest | `import no_such_module_xyz` | no | `ImportError while importing test module '<path>'.` / `Hint: make sure your test modules/packages have valid Python names.` / `E   ModuleNotFoundError: No module named 'no_such_module_xyz'` | 2 | — | [A] M1 `p3` |
| `unittest` (CPython 3.13.15) | `raise ValueError("x")` at the top of `test_a.py` | **yes** | `test_a (unittest.loader._FailedTest.test_a) ... ERROR`, `ImportError: Failed to import test module: test_a`, then the traceback; `FAILED (errors=1)` | 1 | no | [A] M1 `u1` |
| `unittest` | a syntax error; an import of a missing module | yes | the same `_FailedTest` line, with `SyntaxError: invalid syntax` or `ModuleNotFoundError` | 1 | — | [A] M1 `u2` |
| `cargo test` 1.95.0 | one of two integration-test targets does not compile | **no**: no test of any target runs, even with `--no-fail-fast` | `error[E0308]: mismatched types` … ``error: could not compile `proj` (test "broken") due to 1 previous error`` | 101 | n/a | [A] M2 |
| cargo-nextest 0.9.136 | the same | **no**: it builds first | the same, then ``error: command `…cargo test --no-run …` exited with code 101`` | 101 | n/a | [A] M2 |
| `go test` go1.26.8 | a `_test.go` does not compile | yes, the other packages | `FAIL	example.com/m/broken [build failed]`, then `ok  	example.com/m/good` | 1 | n/a | [A] M3 |
| `go test` | an import of a package that does not exist | yes | `FAIL	example.com/m/setup [setup failed]` | 1 | n/a | [A] M3 |
| `go test` | a package `init()` panics | yes, the other packages | `panic: init boom`, then `FAIL	example.com/m/initpanic	0.004s`, with no bracketed word | 1 | n/a | [A] M3 |
| Jest 30.5.2 | the top level of a test file throws; a syntax error; `require('./nope')` | **yes** | `● Test suite failed to run`, then the error; `Test Suites: 6 failed, 1 passed, 7 total` | 1 | n/a | [A] M4 |
| Vitest 5.0.5 | the same three | **yes** | `FAIL  toplevel.test.js [ toplevel.test.js ]` under `Failed Suites 1`, `❯ toplevel.test.js (0 test)`; `Test Files  5 failed \| 2 passed (7)` | 1 | n/a | [A] M4 |
| purba at `efb4c9c` | a module raises on import | the other tests of that module print `did not load`; purba runs one module | `…::test_one error: ValueError: x` and `…::test_two error: … did not load` | 4 | **yes** | [B] the inventory, row 8; [A] read in [`crates/assertion/src/lib.rs` L189-L196](https://github.com/kalonji-tools/purba/blob/efb4c9c/crates/assertion/src/lib.rs#L189-L196) |

- **Four runners keep going, and three stop.** `unittest`, `go test`, Jest and Vitest run every other file or package. pytest stops at the end of collection. `cargo test` and nextest never start a test [A].
- **pytest's stop is a design choice.** A user reported "When some test dependencies are missing, which only affect some subset of tests, all tests are not run." A maintainer answered: "That's intentional", and pointed to `--continue-on-collection-errors` ([pytest#12690](https://github.com/pytest-dev/pytest/issues/12690), RonnyPfannschmidt, 2024-08-05; the comment misspells the flag, and the spelling-check hook of this repository refuses the misspelling, so the flag is not quoted) [A].
- **The stop is in the source.** `pytest_runtestloop` raises `Interrupted` when `session.testsfailed` and the flag is not set ([`main.py` L395-L399](https://github.com/pytest-dev/pytest/blob/9.1.1/src/_pytest/main.py#L395-L399)) [A].
- **`unittest` turns the failed import into a test.** `_make_failed_import_test` builds a `_FailedTest` whose message is `Failed to import test module: %s` ([`loader.py` L38-L49](https://github.com/python/cpython/blob/v3.15.1/Lib/unittest/loader.py#L38-L49)) [A]. The docs call it a "synthetic test that will raise the original error when run" ([unittest.rst L1857-L1863](https://github.com/python/cpython/blob/v3.15.1/Doc/library/unittest.rst?plain=1#L1857-L1863)) [A].
- **`importlib` removes a module whose body raised.** `_load_unlocked` deletes `sys.modules[spec.name]` in its `except:` ([`_bootstrap.py` L908-L921](https://github.com/python/cpython/blob/v3.15.1/Lib/importlib/_bootstrap.py#L908-L921)) [A]. `unittest` and pytest's `prepend` and `append` modes import through it, so the module is gone (M1).
- **pytest's `importlib` mode does not remove it.** It sets `sys.modules[module_name] = mod`, then calls `exec_module` with no `try` ([`pathlib.py` L714-L716](https://github.com/pytest-dev/pytest/blob/9.1.1/src/_pytest/pathlib.py#L714-L716)) [A]. The docs say the mode "adds the module to" `sys.modules` ([pythonpath.rst L87-L91](https://github.com/pytest-dev/pytest/blob/9.1.1/doc/en/explanation/pythonpath.rst?plain=1#L87-L91)) [A]; they say nothing of a module that raised [A].
- **purba matches pytest's `importlib` mode.** [The loaded-module record](https://github.com/kalonji-tools/purba/blob/efb4c9c/docs/decisions/a-loaded-module-joins-sys-modules-under-its-files-name.md#decision-outcome) names it under Downside and leaves it to the design in depth of errors [A].
- **pytest shows its own frames for a syntax error.** The traceback in `p2` runs through `_pytest/python.py`, `_pytest/assertion/rewrite.py` and `ast.py`. For an import error, pytest filters its frames ([`python.py` L528-L531](https://github.com/pytest-dev/pytest/blob/9.1.1/src/_pytest/python.py#L528-L531)); for a syntax error it does not ([L514-L517](https://github.com/pytest-dev/pytest/blob/9.1.1/src/_pytest/python.py#L514-L517)) [A].

### 1b. The kinds pytest and `unittest` name

| runner | kind | printed word or line (exact, measured) | counted as | source | tier |
|---|---|---|---|---|---|
| pytest | a failed assert | `FAILED test_a.py::test_assert - assert 1 == 2` | failed | M1 `p4` | [A] |
| pytest | a test raises `ValueError` | `FAILED test_a.py::test_raises - ValueError: boom` | **failed**, the same as an assert | M1 `p4`; [`terminal.py` L334-L346](https://github.com/pytest-dev/pytest/blob/9.1.1/src/_pytest/terminal.py#L334-L346) | [A] |
| pytest | a fixture raises in setup | `ERROR at setup of test_setup` | error | M1 `p4`; [`runner.py` L220-L229](https://github.com/pytest-dev/pytest/blob/9.1.1/src/_pytest/runner.py#L220-L229) | [A] |
| pytest | a fixture raises in teardown | `ERROR at teardown of test_teardown`; the test also shows `PASSED` | error, and passed | M1 `p4` | [A] |
| pytest | a test names a fixture that does not exist | `E       fixture 'value' not found`, then `available fixtures: …` | error at setup | M1 `p4`, `p12`; [`fixtures.py` L950](https://github.com/pytest-dev/pytest/blob/9.1.1/src/_pytest/fixtures.py#L950) | [A] |
| pytest | a module fails to load | `ERROR collecting test_a.py` | error | item 1a; [`terminal.py` L1233](https://github.com/pytest-dev/pytest/blob/9.1.1/src/_pytest/terminal.py#L1233) | [A] |
| pytest | two test files with one basename, no `__init__.py` | `import file mismatch:` … `HINT: remove __pycache__ / .pyc files and/or use a unique basename for your test file modules` | error collecting | M1 `p9`; [`python.py` L518-L527](https://github.com/pytest-dev/pytest/blob/9.1.1/src/_pytest/python.py#L518-L527) | [A] |
| pytest | a hook raises | every line starts `INTERNALERROR>`; `no tests ran` | internal error | M1 `p5`; [`terminal.py` L582](https://github.com/pytest-dev/pytest/blob/9.1.1/src/_pytest/terminal.py#L582) | [A] |
| pytest | `conftest.py` raises | `ImportError while loading conftest '<path>'.` | usage error | M1 `p7` | [A] |
| pytest | a path that does not exist | `ERROR: file or directory not found: missing_test.py` | usage error | M1 `p6`; [`main.py` L1147](https://github.com/pytest-dev/pytest/blob/9.1.1/src/_pytest/main.py#L1147) | [A] |
| pytest | an unknown flag | `ERROR: usage: …` / `error: unrecognized arguments: --no-such-flag` | usage error | M1 `p6` | [A] |
| pytest | `pytest.skip()` at module level | `Using pytest.skip outside of a test will skip the entire module. …` | error collecting | M1 `p10`; [`python.py` L544-L552](https://github.com/pytest-dev/pytest/blob/9.1.1/src/_pytest/python.py#L544-L552) | [A] |
| pytest | an unknown mark, `--strict-markers` | ``'nosuchmark' not found in `markers` configuration option`` | error collecting | M1 `p10` | [A] |
| pytest | a generator test | `'yield' keyword is allowed in fixtures, but not in tests (test_gen)` | error collecting; the whole run stops | M1 `p8`; [`python.py` L247](https://github.com/pytest-dev/pytest/blob/9.1.1/src/_pytest/python.py#L247) | [A] |
| pytest | an `async def` test, no plugin | `async def functions are not natively supported.` and a list of plugins | failed | M1 `p8b`; [`python.py` L149](https://github.com/pytest-dev/pytest/blob/9.1.1/src/_pytest/python.py#L149) | [A] |
| pytest | a test returns a value | `PytestReturnNotNoneWarning: Test functions should return None, …` | **passed**, with a warning | M1 `p8b` | [A] |
| pytest | `xfail(strict=True)` passes | `FAILED test_a.py::test_x - [XPASS(strict)]` | failed | M1 `p13` | [A] |
| pytest | a thread inside a test raises | `PytestUnhandledThreadExceptionWarning: Exception in thread …` | **passed**, with a warning | M1 `p16` | [A] |
| pytest | a test segfaults after a failed test | `Fatal Python error: Segmentation fault` and a stack; no summary; the earlier `F` is the only trace of the failure | the run is lost | M1 `p11` | [A] |
| pytest | a test hangs | no core timeout; `faulthandler_timeout` only "Dumps the tracebacks of all threads" | — | [reference.rst L1586-L1592](https://github.com/pytest-dev/pytest/blob/9.1.1/doc/en/reference/reference.rst?plain=1#L1586-L1592) | [A] |
| pytest | the summary words | `failed`, `passed`, `skipped`, `deselected`, `xfailed`, `xpassed`, `warnings`, `error`, and three `subtests` words | — | [`terminal.py` L63-L75](https://github.com/pytest-dev/pytest/blob/9.1.1/src/_pytest/terminal.py#L63-L75); [output.rst L466-L474](https://github.com/pytest-dev/pytest/blob/9.1.1/doc/en/how-to/output.rst?plain=1#L466-L474) | [A] |
| `unittest` | a failed assert | `FAIL: test_fail (test_a.Kinds.test_fail)`, `AssertionError: 1 != 2` | failure | M1 `u3` | [A] |
| `unittest` | a test raises `ValueError` | `ERROR: test_error (test_a.Kinds.test_error)` | **error**, apart from a failure | M1 `u3`; [`case.py` L95-L100](https://github.com/python/cpython/blob/v3.15.1/Lib/unittest/case.py#L95-L100) | [A] |
| `unittest` | `setUp` or `tearDown` raises | `ERROR: test_x (test_a.SetUpErr.test_x)` | error, the same as a raise | M1 `u3` | [A] |
| `unittest` | `setUpClass` raises | `ERROR: setUpClass (test_a.ClassErr)` | error | M1 `u3` | [A] |
| `unittest` | an expected failure passes | `UNEXPECTED SUCCESS: test_unexpected_success (…)` | `unexpected successes=1`; the run fails | M1 `u3`; [`runner.py` L163](https://github.com/python/cpython/blob/v3.15.1/Lib/unittest/runner.py#L163) | [A] |
| `unittest` | a test returns a value | `DeprecationWarning: It is deprecated to return a value that is not None from a test case …` | **ok** | M1 `u3`; [`case.py` L636-L645](https://github.com/python/cpython/blob/v3.15.1/Lib/unittest/case.py#L636-L645) | [A] |
| `unittest` | an `async def` method on a `TestCase` | `RuntimeWarning: coroutine 'Kinds.test_async' was never awaited`, and the deprecation warning | **ok**: its `self.fail` never ran | M1 `u3` | [A] |
| `unittest` | a module name that does not exist | `ERROR: test_missing (unittest.loader._FailedTest.test_missing)` | error | M1 `u4` | [A] |
| `unittest` | a test segfaults after a failed test | `F`, then nothing | the run is lost | M1 `u5` | [A] |
| `unittest` | the summary words | `failures=`, `errors=`, `skipped=`, `expected failures=`, `unexpected successes=` | — | [`runner.py` L292-L305](https://github.com/python/cpython/blob/v3.15.1/Lib/unittest/runner.py#L292-L305); [unittest.rst L2085-L2095](https://github.com/python/cpython/blob/v3.15.1/Doc/library/unittest.rst?plain=1#L2085-L2095) | [A] |

- **pytest's word `error` means "outside the test's call".** A raise in collection, setup or teardown is an error; a raise in the call is a failure ([`terminal.py` L342-L344](https://github.com/pytest-dev/pytest/blob/9.1.1/src/_pytest/terminal.py#L342-L344)) [A].
- **`unittest`'s word `error` means "not the failure exception".** Any exception that is not a subclass of `failureException` is an error, wherever it was raised ([`case.py` L95-L100](https://github.com/python/cpython/blob/v3.15.1/Lib/unittest/case.py#L95-L100)) [A]. The docs: an error is "a test which raised an unexpected exception", and a failure is one "where a failure was explicitly signalled" by an assert method [A].
- **purba's `error` follows `unittest`.** A test that raises prints `error` (inventory row 11), and a failed assert prints `failed` [B].

### 1c. The kinds the other five runners name

All rows were measured by a forked session of this research, on 2026-10-25 [A] (M2–M4).

| runner | a test raises (not an assert) | a setup hook fails | a crash in one test | a test hangs | an error outside a test |
|---|---|---|---|---|---|
| `cargo test` | `panicked at`, `--- FAILED`: the same as an assert | — | `process didn't exit successfully: … (signal: 11, SIGSEGV: invalid memory reference)`; the later tests of that binary are lost | [C] not measured | — |
| cargo-nextest | `FAIL`: the same as an assert | setup scripts: `SETUP PASS`, `SETUP LEAK` in the source | `SIGSEGV [ … ] proj tests::segv`; every other test still runs | `TIMEOUT [ 2.003s]` | — (`LEAK` is a process that outlives its test) |
| `go test` | `--- FAIL`, `panic: …`; the rest of the package is lost | `TestMain` exits 3: `FAIL	<pkg>` | a segfault was not measured [C]; a panic loses the rest of the package | `panic: test timed out after 2s` | — |
| Jest | `● throws TypeError`: the same as an assert | `beforeAll` throws: the test is **failed** | not measured | `thrown: "Exceeded timeout of 1000 ms for a test.` | run alone, node's own `Error: unhandled boom` and **no summary**; in a full run, `Jest worker encountered 4 child process exceptions, exceeding retry limit` |
| Vitest | `FAIL  kinds.test.js > throws TypeError`: under the same banner as an assert | `beforeAll` throws: the test is **skipped**; the error is under `Failed Suites` | not measured | `Error: Test timed out in 1000ms.` | `Vitest caught 1 unhandled error during the test run.`; the test shows **passed**; exit 1 |

Sources for the words: nextest's status words are in [`displayer/imp.rs` L2587-L2636](https://github.com/nextest-rs/nextest/blob/cargo-nextest-0.9.149/nextest-runner/src/reporter/displayer/imp.rs#L2587-L2636): `SIG…`, `ABORT`, `FAIL + LEAK`, `FAIL`, `XFAIL`, `PASS`, `LEAK`, `LEAK-FAIL`, `TIMEOUT-PASS`, `TIMEOUT` [A].
libtest prints any panic as `panicked at` ([`panicking.rs` L273](https://github.com/rust-lang/rust/blob/1.99.1/library/std/src/panicking.rs#L273)) [A].
`go test` prints `[build failed]` and `[setup failed]` ([`test.go` L1524](https://github.com/golang/go/blob/go1.27.3/src/cmd/go/internal/test/test.go#L1524), [L1047](https://github.com/golang/go/blob/go1.27.3/src/cmd/go/internal/test/test.go#L1047)) [A].
A `vet` finding prints the same `[build failed]` as a compile error (M3) [A].
Jest prints `Test suite failed to run` from [`jest-message-util` L72](https://github.com/jestjs/jest/blob/v30.5.2/packages/jest-message-util/src/index.ts#L72) [A].
Vitest prints a suite failure from [`base.ts` L1285](https://github.com/vitest-dev/vitest/blob/v5.0.5/packages/vitest/src/node/reporters/base.ts#L1285), and its unhandled-error text from [`logger.ts` L267-L276](https://github.com/vitest-dev/vitest/blob/v5.0.5/packages/vitest/src/node/logger.ts#L267-L276) [A].

- **Six of seven runners print a raise as a failed test.** Only `unittest` sets it apart. pytest sets apart only a raise outside the call [A].
- **Of the runners measured, only the one with a process per test keeps a crash to one test.** nextest reports `SIGSEGV` and runs the rest. `cargo test`, pytest and `unittest` lose the rest of the binary or run, and a panic in `go test` loses the rest of the package [A]. A crash in Jest or Vitest was not measured [C].
- **Vitest prints an error outside a test beside a pass.** It says the error "might cause false positive tests", and exits 1 [A]. purba prints the same pass, and exits 0 (inventory row 35) [B].

## 2. How often users meet each kind

### 2a. pytest

| query (`repo:pytest-dev/pytest is:issue` and) | count | kind |
|---|--:|---|
| (none) | 6,428 | all issues |
| `label:"type: bug"` | 1,418 | all bugs |
| `label:"topic: fixtures"` | 386 (94 with `type: bug`) | fixtures |
| `"ERROR collecting"` | 364 | a module fails to load, or a declaration is invalid |
| `label:"topic: reporting"` | 358 (107 with `type: bug`) | how any kind is shown |
| `label:"topic: collection"` | 323 (101 with `type: bug`) | loading and the import path |
| `"during collection"` | 323 | the run stopped after collection |
| `"INTERNALERROR"` | 295 | an error inside the runner, or a plugin |
| `label:"topic: config"` | 202 (46 with `type: bug`) | configuration |
| `label:"topic: parametrize"` | 184 | a test declaration |
| `"ERROR at setup of"` | 151 | a fixture fails in setup |
| `"no tests ran"` | 149 | no test, or a run that stopped first |
| `label:"topic: rewrite"` | 138 (54 with `type: bug`) | a failed assertion, and how it is shown |
| `"assertion rewriting"` | 130 | the same |
| `label:"plugin: logging"` | 124 | — |
| `label:"plugin: warnings"` | 122 | — |
| `label:"topic: marks"` | 121 | a test declaration |
| `"available fixtures"` | 101 | a fixture that does not exist |
| `label:"plugin: capture"` | 94 | output capture |
| `label:"plugin: xdist"` | 93 | parallel runs |
| `label:"plugin: doctests"` | 85 | — |
| `"KeyboardInterrupt"` | 73 | an interrupt |
| `"unrecognized arguments"` | 69 | a usage error |
| `"ERROR at teardown of"` | 66 | a fixture fails in teardown |
| `"ImportError while importing test module"` | 61 | a module fails to import |
| `label:"plugin: unittest"` | 53 | — |
| `label:"topic: tracebacks"` | 46 (19 with `type: bug`) | how a raise is shown |
| `label:"topic: selection"` | 45 | — |
| `hang in:title` | 44 | a test hangs |
| `label:"plugin: debugging"` | 39 | — |
| `"Segmentation fault"` | 35 | a crash |
| `"import file mismatch"` | 32 | the wrong module |
| `"Fatal Python error"` | 25 | a crash |
| `"ImportError while loading conftest"` | 24 | a `conftest.py` fails |
| `flaky in:title` | 21 | a flaky test |
| `"XPASS(strict)"` | 19 | an expected failure passes |
| `"sys.modules" "import-mode"` | 17 | the module table and the import mode |
| `"SyntaxError" "ERROR collecting"` | 17 | a module does not parse |
| `"fixture" "not found" in:title` | 15 | a fixture that does not exist |
| `"file or directory not found"` | 11 | a usage error |
| `"PytestUnraisableExceptionWarning"` | 11 | an error outside a test |
| `"BrokenPipeError"` | 7 | the report cannot be written |
| `"not natively supported"` | 5 | an async test |
| `"PytestReturnNotNoneWarning"` | 5 | a test returns a value |
| `"PytestUnhandledThreadExceptionWarning"` | 3 | an error outside a test |

**Three samples, sorted by title.** The 40 newest issues of each phrase were listed (C3), and I sorted them by title. The sort is a judgment; the lists are [A].

| phrase | sorted into | count of 40 |
|---|---|--:|
| `ImportError while importing test module` | the module, or one it imports, is not found, or the wrong one is found | 30 |
| | the run stopped for one bad module ([#12690](https://github.com/pytest-dev/pytest/issues/12690)) | 1 |
| | other (doctest, order, docs, packaging) | 9 |
| `INTERNALERROR` | a defect inside pytest that a user's code or input reached | about 21 |
| | a plugin or a hook | about 7 |
| | a configuration value | about 4 |
| | other (proposals, docs, release breakage) | about 8 |
| `ERROR collecting` | the import path or the environment | about 14 |
| | a `parametrize` declaration | about 10 |
| | other | about 16 |

- **pytest's tracker shows loading first among the phrases.** `ERROR collecting` (364) leads them. Among the topic labels, fixtures (386), reporting (358) and collection (323) lead [A, estimates].
- **Most import errors are about the import path, not about code that raised.** 30 of 40 sampled [judgment].
- **Most internal errors are pytest's own defects.** About 21 of 40 sampled; about 7 come from plugins [judgment].
- **pytest has no troubleshooting page.** The docs tree at 9.1.1 has none [A]. [flaky.rst](https://github.com/pytest-dev/pytest/blob/9.1.1/doc/en/explanation/flaky.rst?plain=1#L13-L45) lists three root causes of a flaky test: system state, an overly strict assertion, and thread safety [A]. [pythonpath.rst L27-L28](https://github.com/pytest-dev/pytest/blob/9.1.1/doc/en/explanation/pythonpath.rst?plain=1#L27-L28) warns that test files with one name raise an error [A].

### 2b. `unittest`

CPython has no `unittest` label (C2), so each query asks for `unittest` in the title or body.

| query (`repo:python/cpython is:issue` and) | count | kind |
|---|--:|---|
| `unittest in:title` | 661 | all `unittest` issues, with `mock` |
| `unittest label:type-bug in:title` | 266 | its bugs |
| `"Failed to import test module"` | 30 | a module fails to import |
| `unittest discover in:title` | 20 | discovery |
| `unittest "unexpected success"` | 13 | an expected failure passes |
| `unittest import in:title` | 13 | importing |
| `unittest subTest in:title` | 10 | subtests |
| `"_FailedTest"` | 6 | a module fails to import |
| `unittest tearDown in:title` | 6 | a teardown fails |
| `unittest traceback in:title` | 6 | how a raise is shown |
| `unittest async in:title` | 6 | an async test |
| `unittest setUpClass in:title` | 3 | a setup fails |
| `unittest "return" "None" in:title` | 3 | a test returns a value |
| `unittest "exit code" in:title` | 2 | the exit code |

- **Loading leads here too.** `Failed to import test module` (30) and discovery (20) lead the kind-specific queries [A, estimates].
- **The counts are small.** CPython moved its tracker from bpo to GitHub in 2022 [B]. How that move changes these counts I could not establish [C].

### 2c. The other five runners

All from the forked session's searches on 2026-10-25 (C4) [A, estimates].

| tracker | top phrases, with counts | all issues |
|---|---|--:|
| jestjs/jest | `Test suite failed to run` 549; `expect(received)` 460; `Cannot find module` 269; `Test suite failed to run` with `SyntaxError` 155; `Jest encountered an unexpected token` 87; `Your test suite must contain at least one test` 43; `Exceeded timeout of` 30; `did not exit one second after the test run has completed` 26; `Segmentation fault` 16 | 8,243 |
| vitest-dev/vitest | `AssertionError` 141; `Unhandled Errors` 129; `Failed to load` 95; `Unhandled Rejection` 86; `Failed Suites` 77; `Cannot find module` 73; `No test suite found in file` 26; `Test timed out in` 21; `Segmentation fault` 17 | 4,451 |
| golang/go | `panic:` with `go test`, not by `gopherbot`, 3,389; `build failed` with `go test` 871; `test timed out after`, not by `gopherbot`, 240; `setup failed` with `go test` 46 | 64,263 not by `gopherbot` |
| nextest-rs/nextest | `TIMEOUT` 27; `setup script` 14; `leak` 11; `SIGSEGV` 4; `SIGABRT` 4 | 334 |
| rust-lang/cargo, rust-lang/rust | cargo `label:Command-test` 130; `to rerun pass` 49; rust `label:A-libtest` 187, of them `panicked at` 35 | — |

- **Jest's top phrase is a file that failed to load.** `Test suite failed to run` (549) leads `expect(received)` (460). A sample is mostly config, transform, TypeScript and ESM load problems [A for the sample].
- **Vitest's troubleshooting page is mostly about loading.** Four of its nine headings in [common-errors.md](https://github.com/vitest-dev/vitest/blob/v5.0.5/docs/guide/common-errors.md) are a module that does not load or resolve; the others are a worker that fails to stop, the working directory, segfaults, the time zone, and an unhandled rejection [A].
- **Jest's troubleshooting page leads with a test that fails for no clear reason.** Its headings in [Troubleshooting.md](https://github.com/jestjs/jest/blob/v30.5.2/docs/Troubleshooting.md) also cover caching, an unresolved promise (a timeout), slow CI, and a test defined asynchronously [A].
- **Go's counts include its own CI.** Seven of the first eight `test timed out after` hits are flaky-test reports that `gopherbot` filed [A]; its share of the whole count is [C].
- **Few trackers label a kind of failure.** Jest's `Area: Crash` has 1 issue; Vitest and nextest have no such label [A].
- **A phrase count is not a count of causes.** A pasted log matches its phrases, so a broad phrase such as `panic:` or `Cannot find module` overcounts. By how much I could not establish [C].

## 3. Which kinds drew fixes in oxitest

[The cost study](https://github.com/kalonji-tools/purba/issues/464#issuecomment-6100239730) counts 13 `fix` commits and 10 bug issues for oxitest's "shared errors", the files `_errors.py`, `_boundary.py`, `_paths.py` and `_exceptions.py` under `python/oxitest/_bridge/`.
A forked session reproduced both counts at `731c2b3f` with the study's rules: 44 commits, 13 `fix`, 10 issues with the `bug` label (C5) [A].
No path in the set was renamed [A].

| kind | fix commits | bug issues |
|---|--:|--:|
| a fixture error: not found, wiring, resolution, async access, context, hint text | 9 | 7 |
| a usage or configuration error with the wrong exit code | 2 | 1 |
| an error inside the runner, shown as a usage error | 1 | 0 |
| an interrupt during fixture setup | 1 | 1 |
| a false pass: a generator test | 0 | 1 |
| a module that fails to import | 0 | 0 |
| a test's own exception, shown wrong | 0 | 0 |
| **total** | **13** | **10** |

| what the defect did | fix commits | bug issues |
|---|--:|--:|
| showed the wrong kind, or the wrong exit code | 6 | 4 |
| showed the right kind, with the wrong text | 5 | 4 |
| other: an error that was missing, or a cleanup that was lost | 2 | 2 |

The kind of each commit is a judgment from its diff and message [judgment].
The 13 commits are in Appendix D.

- **A fixture wiring error exited 1.** Commit `e33d2d1f` says it was "indistinguishable in CI from a failed assertion" [A].
- **An error inside oxitest was shown as the user's.** Commit `5cca3dd9` adds `InternalError` to `_errors.py`, and its docstring says: "exit 4 tells the user that their request was invalid, which is the wrong thing to tell someone who has found a bug (#2172)" [A].
- **A test that never ran was shown as passed.** oxitest#2067: a generator test is "reported as **passed**, and **never executes a single line of its body**" [A].

**Outside the study's path set.** The study puts the loader under "Python: collection", so its import fixes are not in the 13 [A]. I listed `fix` commits whose subject names import, collection or loading (C6). Six bear on a module that fails to load [A for the commits; the selection is a judgment]:

| commit | subject | issue |
|---|---|---|
| `70fbc27a` | fix: clean up sys.modules on conftest exec_module failure (#1393) | oxitest#1393, `bug` |
| `6d50a64f` | fix: drain pending diagnostics on the collection-error exits (#2055) | oxitest#2055, `bug` |
| `bbe638a3` | fix: serialize collection errors into the CTRF report (#1682) | oxitest#1682, `bug` |
| `7bf2f70b` | fix: give test modules a real dotted identity (#1680) | oxitest#1680, `bug` |
| `46aad58d` | fix: the collection route reuses an already-imported source module (#2014) | oxitest#2014, `bug` |
| `e7f5fabe` | fix: compare file identity in already_imported (#2018) | oxitest#2018, `bug` |

- **oxitest took a failed module out of `sys.modules`.** `70fbc27a` says: "Previously a partially-initialized module remained in sys.modules, poisoning subsequent imports — especially in parallel mode", and that it "Matches the existing cleanup pattern in `_loader.py`" [A].
- **That pattern also restored the module it displaced.** [`_loader.py` L276-L283](https://github.com/kalonji-tools/oxitest/blob/731c2b3f76c708125eb4984d9725652208f5ae81/python/oxitest/_bridge/_loader.py#L276-L283) pops both keys and calls `sys.modules.update(displaced)` [A]. It catches `Exception`, so a `SystemExit` at import would stay [A from the code; not run].
- **A collection error once hid its own explanation.** `6d50a64f`: "Any collection error discarded every pending diagnostic the run had accumulated, including the one that explains the failure" [A].
- **A collection error once reported as green.** `bbe638a3`: a run that ended before any test "could only ever serialize as `tests: [], failed: 0`" [A].

## 4. Which kinds purba can meet: now, later or never

**now** is one module, run in-process, with no fixture and no plugin.
**later** names the part or concern whose design brings the kind.
**never** needs something purba rules out.
Each row is a judgment against [LANDSCAPE.md](https://github.com/kalonji-tools/purba/blob/efb4c9c/LANDSCAPE.md) and [the public-surface record](https://github.com/kalonji-tools/purba/blob/efb4c9c/docs/decisions/purbas-public-surface-starts-empty-and-has-no-plugin-api.md#decision-outcome).

| kind | purba | why | inventory rows |
|---|---|---|---|
| a failed assertion | now | the skeleton's own outcome | 12c; 26, an assert that cannot fail |
| a test raises an error that is not an assertion | now | any `BaseException` from the call | 11, 12, 13, 27, 30 |
| a module raises while it loads | now | assertion's load runs the body | 8, 9, 10 |
| a module does not parse | now | CPython's `compile()` raises at load | 7, 7b, 7d |
| a module imports what is missing | now | the same path as row 8 | 8, the same shape; not probed apart |
| an error inside the runner | now | each **P** row of the inventory | 6, 6b, 7c, 7g, 29, 29b, 31, 33, 34 |
| a usage error | now | command reads one path | 1, 2, 4, 5, 5b, 5c, 34 |
| a configuration error | later: configuration | no configuration exists | none |
| no test found | now | an empty module, or tests only in a class | 7b, 25 |
| the process crashes | now; later: dispatch could keep a crash to one worker | in-process, as pytest and `unittest` | 7g, 10, 14, 14b, 20 |
| a test shape the runner cannot run | now | async, generator, a name that is not a function | 22, 24, 25 |
| an error outside any test | now | a thread in the test | 35 |
| the report cannot be written | now | reporting writes stdout | 17, 17c, 17d, 17e, 36 |
| an interrupt | now; out of this map's scope | interrupt handling decides it | 15, 18, 18b, 19 |
| a test names an input that nothing provides | now as a `TypeError`; later: fixtures gives it its own kind | fixtures build inputs by type | 21 |
| a fixture fails in setup or teardown | later: fixtures | no fixture exists | none |
| the import path finds the wrong module, or none | now in part; later: execution | one module today; [How does a test module import a module beside it, or the package it belongs to?](https://github.com/kalonji-tools/purba/issues/637) waits for execution | 31; the loaded-module record's `test.py` |
| a test hangs | later: dispatch | no timeout; the inventory lists a hang as read, not run | none |
| a test leaks a handle or a process | later: dispatch | a worker that outlives a test [judgment] | none |
| an expected failure passes | later: strictness | a suppression's code waits for strictness, as the map's Out of scope says | none |
| a test declaration is invalid (parametrize, marks) | later: [C] | no part or concern names parametrization or marks | none |
| a test that passes on a retry | [C] | no part, concern or record names retries | none |
| a plugin, a hook or a `conftest.py` fails | **never** | "purba never loads a plugin" | none |

This table has 23 rows, where the Answer's table has 20: it splits a module that fails to load into three kinds, and a usage error from a configuration error.
Every row of the inventory appears at least once. Rows 7b, 7g, 8, 10, 25, 31 and 34 appear under two kinds [A, checked against the inventory's list].

## 5. The kinds that apply to purba, ranked by how often users meet them

The rank argues from every tracker in item 2, not from one.
A kind ranks high when it leads in more than one tracker.
"Applies" means now or later in item 4; the plugin kind is left out.

| rank | kind | purba | the evidence across the survey | confidence |
|--:|---|---|---|---|
| 1 | a module fails to load: it raises, does not parse, or imports what is missing | now | top phrase in Jest (549) and pytest (364); third in Vitest (95), with 4 of 9 troubleshooting headings; `build failed` 871 in Go; the most `unittest`-specific hits (30) | high |
| 2 | a failed assertion, and how it is shown | now | Vitest's top phrase, `AssertionError` 141; Jest's second, `expect(received)` 460; pytest `topic: rewrite` 138 | high |
| 3 | the import path finds the wrong module, or none | now in part; later: execution | 30 of 40 sampled pytest import errors; `import file mismatch` 32; Jest and Vitest `Cannot find module` 269 and 73 | medium: overlaps rank 1 |
| 4 | a fixture fails in setup or teardown | later: fixtures | pytest 151 + 66; `topic: fixtures` 386; 9 of oxitest's 13 shared-error fixes | medium: one tracker, and oxitest |
| 5 | an error inside the runner | now | pytest `INTERNALERROR` 295, about half of it pytest's own defects; oxitest's `InternalError` fix | medium: one tracker names it |
| 6 | an error outside any test | now | Vitest `Unhandled Errors` 129, `Unhandled Rejection` 86; pytest 3 + 11 | medium: high in one runner only |
| 7 | a usage or configuration error | now and later | pytest `topic: config` 202, `unrecognized arguments` 69; 2 oxitest fixes | medium |
| 8 | a test hangs | later: dispatch | Go 240; pytest 44 in titles; Jest 30; nextest 27; Vitest 21 | medium |
| 9 | a test names an input that nothing provides | now; later: fixtures | pytest `available fixtures` 101 | medium: one tracker |
| 10 | no test found | now | pytest `no tests ran` 149, a broad phrase; Jest 43; Vitest 26 | low |
| 11 | the process crashes | now | pytest 35 and 25; Vitest 17; Jest 16; nextest 4 | medium |
| 12 | a test leaks a handle or a process | later: dispatch | Jest 26; nextest 11 | low |
| 13 | an expected failure passes | later: strictness | pytest 19; CPython 13 | low |
| 14 | the report cannot be written | now | pytest `BrokenPipeError` 7 | low |
| 15 | a test shape the runner cannot run | now | pytest 5 + 5; CPython 6 + 3 | low |
| — | a test raises an error that is not an assertion | now | [C]: no runner but `unittest` prints it apart, so no phrase counts it | [C] |
| — | an interrupt | now | pytest 73; out of this map's scope | — |

## What this means for purba

| evidence | what it bears on |
|---|---|
| a module that fails to load leads the survey (item 5, rank 1) | the effort of the errors map goes first to the load path: rows 7, 7b, 8, 9 and 10 |
| four runners run the other files, and a pytest maintainer calls pytest's stop intentional (item 1a) | what purba does with a module that raised has precedent both ways; purba runs one module, so the choice binds only when command takes many |
| `unittest` and pytest's default mode (both through `importlib`) and oxitest's loader take a failed module out of `sys.modules`; pytest's `importlib` mode does not (item 1a, item 3) | the loaded-module record's Downside: three of four Python precedents remove it, and oxitest's fix names the cost of leaving it, "poisoning subsequent imports — especially in parallel mode" |
| oxitest's loader also restored a module it displaced (item 3) | the record's other Downside: `test.py` and `builtins.py` (row 31) replace a module that was there |
| six of seven runners print a raise as a failed test; `unittest` and purba keep it apart (item 1b, 1c) | purba's `error` already keeps a raise apart from `failed`; the kinds the map defines can keep that split |
| pytest's `INTERNALERROR` (295) is the third-largest phrase counted, and oxitest added `InternalError` after showing one as a usage error (item 2a, item 3) | the map's risk "purba's own error shows as a test failure" has user-side evidence beyond purba |
| a thread that raises inside a test: Vitest prints `passed` beside the error and exits 1; pytest prints `passed` with a warning and exits 0; purba prints `passed`, with Python's traceback on stderr, and exits 0 (item 1b, 1c, row 35) | the honest-outcome priority: of the three, only Vitest fails the run |
| 6 of oxitest's 13 fixes showed the wrong kind (item 3) | classifying the kind cost as much as rendering its text |
| only nextest, with a process per test, keeps a crash to one test (item 1c) | a crash is "now" for purba, but its reduction belongs to dispatch |

Facts that bear on the records:

- **The loaded-module record cites `importlib` at v3.12.14 L936-L941.** At v3.15.1 the same `del sys.modules[spec.name]` is at L918 [A].
- **purba prints `did not load` for every later test of a failed module (row 8).** pytest, Jest and Vitest print the failure once, for the file. `unittest` prints it once, as one synthetic test [A for the runners; B for purba].
- **pytest's `importlib` mode leaves a failed module in `sys.modules`.** Whether a pytest issue reports it I could not establish: the query `"sys.modules" "import-mode"` gave 17 issues, and I did not read them [C].

## Appendix A. Commands

Sources (S), searches (C) and measurements (M) ran on 2026-10-25.
`$P` is `/tmp/claude-1000/-home-snregales-Projects-kalonji-tools/16dca681-fc06-4d1d-a235-dc886367a8a3/scratchpad/research-r2`.
M2–M4, S3–S7, C4 and C5 ran in two forked sessions of this research; their raw output is under `$P/nonpy/`.

| key | command |
|---|---|
| S1 | `curl -sfL https://raw.githubusercontent.com/pytest-dev/pytest/9.1.1/<path>` for `src/_pytest/{pathlib,python,terminal,main,fixtures,runner,reports}.py` and `doc/en/{how-to/output,how-to/usage,how-to/failures,explanation/goodpractices,explanation/pythonpath,explanation/flaky,reference/reference}.rst` |
| S2 | `curl -sfL https://raw.githubusercontent.com/python/cpython/v3.15.1/<path>` for `Lib/unittest/{loader,runner,result,case}.py`, `Lib/importlib/_bootstrap.py`, `Doc/library/unittest.rst` |
| S3 | `curl -sfL https://raw.githubusercontent.com/rust-lang/rust/1.99.1/library/{test/src/formatters/pretty.rs,std/src/panicking.rs}`; `.../rust-lang/cargo/0.100.0/{src/ops/cargo_test.rs,src/doc/man/cargo-test.md}` |
| S4 | `curl -sfL https://raw.githubusercontent.com/nextest-rs/nextest/cargo-nextest-0.9.149/<path>` for `nextest-runner/src/reporter/displayer/imp.rs`, `site/src/docs/{reporting.md,features/leaky-tests.md,features/slow-tests.md}`; `gh api 'repos/nextest-rs/nextest/git/trees/cargo-nextest-0.9.149?recursive=1'` |
| S5 | `curl -sfL https://raw.githubusercontent.com/golang/go/go1.27.3/src/{cmd/go/internal/test/test.go,testing/testing.go}` |
| S6 | `curl -sfL https://raw.githubusercontent.com/jestjs/jest/v30.5.2/{packages/jest-message-util/src/index.ts,packages/jest-worker/src/workers/ChildProcessWorker.ts,docs/Troubleshooting.md}` |
| S7 | `curl -sfL https://raw.githubusercontent.com/vitest-dev/vitest/v5.0.5/{packages/vitest/src/node/reporters/base.ts,packages/vitest/src/node/logger.ts,docs/guide/common-errors.md}` |
| S8 | `gh api "repos/pytest-dev/pytest/git/trees/9.1.1?recursive=1" --jq '.tree[].path'` and the same for `python/cpython` at `v3.15.1`, filtered for troubleshooting, FAQ and failure pages |
| S9 | `gh api repos/pytest-dev/pytest/issues/12690` and `.../issues/12690/comments` |
| S10 | the purba tree at `efb4c9c`: `crates/assertion/src/lib.rs` L159-L197, `crates/execution/src/lib.rs` L189-L222 |
| C1 | `$P/search-py.sh` and `$P/search-py2.sh`: `gh api -X GET search/issues -f q='<query>' --jq .total_count`, 6 s between calls, then 20 s after a secondary rate limit; each query is the one shown in item 2a and 2b; `BrokenPipeError` re-ran alone after 90 s |
| C2 | `gh label list -R pytest-dev/pytest --limit 300`; `gh label list -R python/cpython --limit 400` |
| C3 | `$P/sample-py.sh`: `gh api -X GET search/issues -f q='repo:pytest-dev/pytest is:issue "<phrase>"' -f sort=created -f order=desc -f per_page=40` for the three phrases in item 2a |
| C4 | `$P/nonpy/search.sh`: the same search form for each query in item 2c, 6 s and then 12 s between calls; `gh label list -R <repo> --limit 1000` |
| C5 | in `/home/snregales/Projects/kalonji-tools/oxitest` at `731c2b3f`: `git log --format='%h %ad %s' --date=short -- $SET \| grep -E '^[0-9a-f]+ [0-9-]+ fix(\([^)]*\))?!?:'` (13); `git log --format='%s' -- $SET \| grep -oE '#[0-9]+' \| sort -u` (32), then `gh api repos/kalonji-tools/oxitest/issues/<n> --jq '.title,.labels[].name'` (10 with `bug`); `git show <sha>` for each commit; `$SET` is the four files of item 3 |
| C6 | the same clone: `git log --format='%h %ad %s' --date=short \| grep -E ' fix(\([^)]*\))?!?:' \| grep -i -E 'import\|collect\|load\|syntax\|sys\.modules'`; `git show -s <sha>`; `gh api repos/kalonji-tools/oxitest/issues/<n>` |
| M1 | `nix-shell -p 'python3.withPackages(ps: [ps.pytest])' --run 'bash py.sh $PWD/run/py'`, then `py2.sh` and `py3.sh`; each case is its own directory under `$P/run/py/`; pytest runs `python -I -m pytest -q -p no:cacheprovider -rA`, `unittest` runs `python -I -m unittest discover -s . -v` |
| M2 | `nix-shell -p cargo rustc cargo-nextest --run "bash $P/nonpy/rs.sh $P/nonpy"` and `rs2.sh`, with `CARGO_HOME`, `CARGO_TARGET_DIR` and `HOME` in scratch; 7 unit tests and two integration targets, one of which does not compile |
| M3 | `nix-shell -p go --run "bash $P/nonpy/go.sh $P/nonpy"` and `go2.sh`, with `GOCACHE`, `GOPATH`, `HOME` in scratch and `GOTOOLCHAIN=local`; the packages good, broken, initpanic, mainfail, kinds, setup and vet. The setup case made `go` request the invented module path from example.com; the request carried no personal data |
| M4 | `npm install --no-save jest@30.5.2 vitest@5.0.5`, then `nix-shell -p nodejs --run "bash $P/nonpy/js.sh $P/nonpy"` with `CI=true`; each test file ran alone, then all together |
| L | `gh api repos/O/R/commits/<branch> --jq .commit.committer.date`; `gh api repos/O/R/releases/latest --jq '.tag_name+" "+.published_at'`; for a repo with no releases, `git ls-remote --tags --refs`, then `gh api repos/O/R/commits/<tag> --jq .commit.committer.date` |

## Appendix B. Versions measured

Sources were read at the newest tag; measurements ran the version nixpkgs or npm gave.

| tool | source read | version measured |
|---|---|---|
| pytest | 9.1.1 (`cf470ec`) | 9.0.3, CPython 3.13.15 |
| `unittest`, `importlib` | CPython v3.15.1 (`c37e74c`) | CPython 3.13.15 |
| `cargo test` | rust 1.99.1, cargo 0.100.0 | cargo and rustc 1.95.0 |
| cargo-nextest | 0.9.149 | 0.9.136 |
| `go test` | go1.27.3 | go1.26.8 |
| Jest | v30.5.2 | 30.5.2, node 24.21.0 |
| Vitest | v5.0.5 | 5.0.5, node 24.21.0 |
| oxitest | `731c2b3f` | — (read only) |
| purba | `efb4c9c` | — (read; the inventory ran the wheel) |

## Appendix C. Liveness

| project | default branch, last commit | latest release or tag | date |
|---|---|---|---|
| pytest-dev/pytest | `main`, 2026-10-25 | release 9.1.1 | 2026-06-19 |
| python/cpython | `main`, 2026-10-25 | tag v3.15.1 (no GitHub releases) | 2026-10-21 (tag commit) |
| rust-lang/rust | `main`, 2026-10-25 | release 1.99.1 | 2026-10-16 |
| rust-lang/cargo | `master`, 2026-10-25 | tag 0.100.0 (no GitHub releases) | 2026-08-28 (tag commit) |
| nextest-rs/nextest | `main`, 2026-10-25 | release cargo-nextest-0.9.149 | 2026-10-20 |
| golang/go | `master`, 2026-10-24 | tag go1.27.3 (no GitHub releases) | 2026-10-20 (tag commit) |
| jestjs/jest | `main`, 2026-10-02 | release v30.5.2 | 2026-09-18 |
| vitest-dev/vitest | `main`, 2026-10-25 | release v5.0.5 | 2026-10-15 |
| kalonji-tools/oxitest | `main`, 2026-08-19 (archived) | release v4.0.0 | 2026-08-09 |

## Appendix D. oxitest's 13 shared-error fixes

From C5 [A for the commits and issues; the kind is a judgment].

| commit | date | subject | kind | what the defect did |
|---|---|---|---|---|
| `ef7a705a` | 2026-08-16 | fix: let an absent plugin and an absent async backend exit 4 (#2185) | usage error, wrong exit code | wrong kind |
| `c8ccb394` | 2026-08-16 | fix: decide the startup exit code by the error's class (#2172) | usage error, wrong exit code | wrong kind |
| `5cca3dd9` | 2026-08-16 | fix: mark a broken invariant as InternalError, not UsageError (#2172) | error inside the runner, shown as a usage error | wrong kind |
| `03c845f8` | 2026-08-13 | fix: resolve a yield fixture by the type it provides (#2094) | fixture resolution | wrong kind |
| `ae97e274` | 2026-08-12 | fix: show a diagnostic path against the project rootdir (#1851) | fixture diagnostic | wrong text |
| `faa1d41b` | 2026-08-11 | fix: settle ADR-0006's illegal async cell as proxy-only (#1876) | async fixture access | wrong text, mainly |
| `bddb7420` | 2026-08-11 | fix: stop recommending a tier with the same unguarded defect (#2041) | fixture hint | wrong text |
| `2f640460` | 2026-08-11 | fix: replace the retired shared vocabulary in user-facing text (#2036) | fixture hint | wrong text |
| `e33d2d1f` | 2026-08-10 | fix: vote UsageError when the bridge reports a wiring error (#1761) | fixture wiring error | wrong kind |
| `58ba4fb8506a3b8606720a1bba679ae4486a5923` | 2026-08-08 | fix: register fixture teardowns before advancing the generator (#1962) | interrupt during fixture setup | lost cleanup |
| `0923fd67` | 2026-08-03 | fix: raise when TestContext identity is read outside a test (#1874) | fixture context | missing error |
| `09f2a45b` | 2026-07-29 | fix: await and dispose async @oxi.fixture declarations (#1733) | async fixture | wrong kind |
| `07874d57` | 2026-07-13 | fix: add corrective hints to fixture and backend error messages (#1497) | fixture messages | wrong text |

The 10 bug issues are oxitest#1733, #1851, #1874, #1876, #1962, #2036, #2041, #2067, #2094 and #2185.
oxitest#2067 is cited by `38c7de14`, a `feat!:` commit, not by a fix [A].
