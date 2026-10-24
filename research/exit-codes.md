# What a test runner's exit code tells the program that runs it

This file answers [#616](https://github.com/kalonji-tools/purba/issues/616).
It is never merged.

## Answer

Every integrator in this survey reads only one thing: zero or not zero.
A GitHub Actions step, a pre-commit hook, `make`, tox and nox fail on any non-zero code, and pass on 0.
None of them reads the report.
So a code that says only "the runner ran well" turns each failed test into a pass in every one of them.
Every documented case in item 6 has that shape: the runner exited 0, and the CI went green.
Only `git bisect run` gives a meaning to particular non-zero codes: 125 skips a commit, and 128 or more aborts.
In a measured bisect, a runner fault that exited 4 made bisect blame the wrong commit; 125 found the right one.
Runners keep their own failure apart from a failed test only in part.
pytest and cargo-nextest give a fault its own code.
`cargo test`, Jest, Vitest and `go test` give most faults the same code as a failed test.
Four runners exited 0 after a failed test when a test called `exit(0)`: pytest, `unittest`, `cargo test` and Jest.
POSIX checking utilities (`grep`, `diff`, `cmp`, `test`) use 0 for yes, 1 for no, and more than 1 for an error.
I found no standard written for test runners.

| runner | every test passed | a test failed | no test | usage error | path does not exist | fault apart from a failed test? |
|---|---|---|---|---|---|---|
| pytest 9.0.3 | 0 | 1 | 5 | 4 | 4 | yes, but a collection error exits 2, the code for an interrupt |
| `unittest` (CPython 3.13.15) | 0 | 1 | 5 | 2 | 1 | no: an import error or a missing module exits 1 |
| `cargo test` 1.95.0 | 0 | 101 | 0 | 1 (cargo) or 101 (harness) | 101 | no: build, harness and cargo errors exit 101 |
| cargo-nextest 0.9.136 | 0 | 100 | 4 | 2 | not measured | yes: 101 build, 96 setup, 1 unexpected |
| `go test` 1.26.8 | 0 | 1 | 0 | 2, or 1 for a flag passed to the test binary | 1 | no: a build failure and a missing package exit 1 |
| Jest 30.5.2 | 0 | 1 | 1 | 1 | 1 | no: a usage or config error exits 1 |
| Vitest 5.0.5 | 0 | 1 | 1 | 1 | 1 | no: every fault exits 1 |

All rows are [A], measured in this session (commands M1–M5).

## 1. Pass, fail and no test

| runner | case | code | meaning | tier | source |
|---|---|---|---|---|---|
| pytest | all pass | 0 | "Tests passed." | [A] | [`ExitCode`](https://github.com/pytest-dev/pytest/blob/9.1.1/src/_pytest/config/__init__.py#L101-L123) (S1); M1 |
| pytest | an assert fails | 1 | "Tests failed." | [A] | S1; M1 |
| pytest | a fixture raises | 1 | the report says `1 error`; the code is the same as a failure | [A] | M1 |
| pytest | no test collected | 5 | "pytest couldn't find tests." | [A] | S1; [`_main`](https://github.com/pytest-dev/pytest/blob/9.1.1/src/_pytest/main.py#L380-L390); M1 |
| pytest | all pass, too many warnings | 6 | "All tests pass, but maximum number of warnings exceeded." | [A] | S1, L121-L122; [exit-codes.rst](https://github.com/pytest-dev/pytest/blob/9.1.1/doc/en/reference/exit-codes.rst) |
| `unittest` | all pass | 0 | `sys.exit(0)` | [A] | [`main.py`](https://github.com/python/cpython/blob/v3.15.1/Lib/unittest/main.py#L271-L277) (S2); M1 |
| `unittest` | a test fails or raises | 1 | `not self.result.wasSuccessful()` | [A] | S2; M1 |
| `unittest` | no test ran and none skipped | 5 | `_NO_TESTS_EXITCODE = 5` | [A] | S2, [L11](https://github.com/python/cpython/blob/v3.15.1/Lib/unittest/main.py#L11); [docs](https://github.com/python/cpython/blob/v3.15.1/Doc/library/unittest.rst?plain=1#L2380-L2382); M1 |
| `cargo test` | all pass | 0 | — | [A] | M2 |
| `cargo test` | a test fails | 101 | libtest `ERROR_EXIT_CODE: i32 = 101` | [A] | [libtest](https://github.com/rust-lang/rust/blob/1.99.1/library/test/src/lib.rs#L93) and [L174-L180](https://github.com/rust-lang/rust/blob/1.99.1/library/test/src/lib.rs#L174-L180); M2 |
| `cargo test` | no test | 0 | `0 passed; 0 failed` | [A] | M2 |
| cargo-nextest | all pass | 0 | `OK` | [A] | [`exit_codes.rs`](https://github.com/nextest-rs/nextest/blob/cargo-nextest-0.9.149/nextest-metadata/src/exit_codes.rs#L13-L29) (S3); M2 |
| cargo-nextest | a test fails | 100 | `TEST_RUN_FAILED` | [A] | S3; M2 |
| cargo-nextest | no test | 4 | `NO_TESTS_RUN`, "advisory"; default `--no-tests=fail` since 0.9.85 | [A] | S3; [running.md](https://github.com/nextest-rs/nextest/blob/cargo-nextest-0.9.149/site/src/docs/running.md?plain=1#L135-L139); M2 |
| cargo-nextest | no test, `--no-tests=pass` | 0 | — | [A] | M2 |
| `go test` | all pass | 0 | — | [A] | M3 |
| `go test` | a test fails | 1 | `base.SetExitStatus(1)` | [A] | [`test.go`](https://github.com/golang/go/blob/go1.27.3/src/cmd/go/internal/test/test.go#L1748); M3 |
| `go test` | package with no test file | 0 | `[no test files]` | [A] | [`test.go`](https://github.com/golang/go/blob/go1.27.3/src/cmd/go/internal/test/test.go#L1562); M3 |
| `go test` | test file with no test | 0 | `[no tests to run]` | [A] | M3 |
| Jest | all pass | 0 | — | [A] | M4 |
| Jest | a test fails | 1 | `testFailureExitCode`, default `1` | [A] | [`run.ts`](https://github.com/jestjs/jest/blob/v30.5.2/packages/jest-cli/src/run.ts#L108); [Configuration.md](https://github.com/jestjs/jest/blob/v30.5.2/docs/Configuration.md?plain=1#L2249-L2257); M4 |
| Jest | no test | 1 | "No tests found, exiting with code 1" | [A] | [`runJest.ts`](https://github.com/jestjs/jest/blob/v30.5.2/packages/jest-core/src/runJest.ts#L369-L381); M4 |
| Jest | no test, `--passWithNoTests` | 0 | also 0 with `--lastCommit` or `--onlyChanged` | [A] | [`getNoTestsFoundMessage.ts`](https://github.com/jestjs/jest/blob/v30.5.2/packages/jest-core/src/getNoTestsFoundMessage.ts#L20-L23); M4 |
| Vitest | all pass | 0 | — | [A] | M4 |
| Vitest | a test fails | 1 | `process.exitCode = 1` | [A] | [`core.ts`](https://github.com/vitest-dev/vitest/blob/v5.0.5/packages/vitest/src/node/core.ts#L1276-L1281); M4 |
| Vitest | no test | 1 | `passWithNoTests` default `false` | [A] | [passwithnotests.md](https://github.com/vitest-dev/vitest/blob/v5.0.5/docs/config/passwithnotests.md); M4 |
| Vitest | no test, `--passWithNoTests` | 0 | `vitest related` sets it to `true` | [A] | [`cac.ts`](https://github.com/vitest-dev/vitest/blob/v5.0.5/packages/vitest/src/node/cli/cac.ts#L246-L260); M4 |

- A run with no test splits the runners: pytest, `unittest`, nextest, Jest and Vitest fail it; `cargo test` and `go test` pass it.
- Two of the five that fail it passed it before: `unittest` before 3.12, and nextest before 0.9.85 (item 6).
- Jest moved the other way for one flag: `--findRelatedTests` with no test exited 1 in 2017, and then 0 (item 6).

## 2. The runner's own failure

| runner | case | code | same as a failed test? | tier | source |
|---|---|---|---|---|---|
| pytest | unknown flag | 4 | no | [A] | `USAGE_ERROR`, S1; M1 |
| pytest | path does not exist | 4 | no | [A] | [`main.py`](https://github.com/pytest-dev/pytest/blob/9.1.1/src/_pytest/main.py#L1143-L1149) raises `UsageError`; M1 |
| pytest | hook raises in `conftest.py` | 3 | no | [A] | [`wrap_session`](https://github.com/pytest-dev/pytest/blob/9.1.1/src/_pytest/main.py#L315-L373), `except BaseException`; M1 |
| pytest | test module fails to import | 2 | no, but 2 is also "interrupted by the user" | [A] | [`pytest_runtestloop`](https://github.com/pytest-dev/pytest/blob/9.1.1/src/_pytest/main.py#L397-L401) raises `Interrupted`, a `KeyboardInterrupt`; M1 |
| pytest | `python -m pytest`, pytest not importable | 1 | **yes** | [A] | M1 (the interpreter exits 1) |
| `unittest` | unknown flag | 2 | no | [A] | [`argparse.py`](https://github.com/python/cpython/blob/v3.15.1/Lib/argparse.py#L2958); M1 |
| `unittest` | module name does not exist | 1 | **yes** | [A] | M1, `FAILED (errors=1)` |
| `unittest` | test module fails to import | 1 | **yes** | [A] | M1 |
| `unittest` | an exception escapes the runner | 1 | **yes** | [B] | relayed from the record's CPython `main.c` row; not re-run |
| `cargo test` | unknown cargo flag | 1 | no | [A] | [`CliError` from `clap::Error`](https://github.com/rust-lang/cargo/blob/0.100.0/src/util/errors.rs#L349-L354); M2 |
| `cargo test` | unknown harness flag | 101 | **yes** | [A] | [libtest](https://github.com/rust-lang/rust/blob/1.99.1/library/test/src/lib.rs#L120-L125); M2 |
| `cargo test` | `--test` target does not exist | 101 | **yes** | [A] | M2 |
| `cargo test` | build error | 101 | **yes** | [A] | M2 |
| `cargo test` | any other cargo error | 101 | **yes** | [A] | [`From<anyhow::Error>`](https://github.com/rust-lang/cargo/blob/0.100.0/src/util/errors.rs#L343-L347); [man page](https://github.com/rust-lang/cargo/blob/0.100.0/doc/man/generated_txt/cargo-test.txt#L480-L483): "101: Cargo failed to complete." |
| cargo-nextest | unknown flag | 2 | no | [A] | M2; [#1639](https://github.com/nextest-rs/nextest/issues/1639): "2 is for parsing errors" |
| cargo-nextest | build error | 101 | no (100 is a failed test) | [A] | `BUILD_FAILED`, S3; M2 |
| cargo-nextest | setup, filterset, archive, `cargo metadata` | 96, 94, 103, 102 | no | [A] | S3, L16-L60 |
| cargo-nextest | unexpected failure | 1 | no | [A] | S3, L9: "Unknown/unexpected failures will always result in exit code 1." |
| `go test` | bad `go` flag (`-count=x`) or unknown command | 2 | no | [A] | M3 |
| `go test` | unknown flag (passed on to the test binary) | 1 | **yes** | [A] | M3: the binary exits 2, `go test` prints `FAIL` and exits 1 |
| `go test` | package path does not exist | 1 | **yes** | [A] | M3, `[setup failed]` |
| `go test` | build error | 1 | **yes** | [A] | M3, `[build failed]` |
| `go test` | crash inside the `go` command | — | — | [C] | not measured |
| Jest | unknown flag (`--nosuchflag`) | 1 | **yes** (default) | [A] | M4: "Unrecognized CLI Parameter" |
| Jest | unknown flag that starts with `--no-` (`--no-such-flag`) | 0 | — (read as `such-flag` set to false; the tests ran) | [A] | M4 |
| Jest | path matches no test | 1 | **yes** (default) | [A] | M4 |
| Jest | invalid config | 1 | **yes** (default) | [A] | M4; Configuration.md: `testFailureExitCode` "does not change the exit code in the case of Jest errors" |
| Jest | error thrown inside `run` | 1 | **yes** (default) | [A] | [`run.ts`](https://github.com/jestjs/jest/blob/v30.5.2/packages/jest-cli/src/run.ts#L28-L41) |
| Vitest | unknown flag | 1 | **yes** | [A] | M4, `CACError` |
| Vitest | path filter matches nothing | 1 | **yes** | [A] | M4 |
| Vitest | start-up error | 1 | **yes** | [A] | [`cli-api.ts`](https://github.com/vitest-dev/vitest/blob/v5.0.5/packages/vitest/src/node/cli/cli-api.ts#L168-L176); [`cac.ts`](https://github.com/vitest-dev/vitest/blob/v5.0.5/packages/vitest/src/node/cli/cac.ts#L312-L320) |
| Vitest | unhandled error, every test passed | 1 | **yes** | [A] | [`core.ts`](https://github.com/vitest-dev/vitest/blob/v5.0.5/packages/vitest/src/node/core.ts#L1600-L1603); M4 |

- Jest can split a fault from a failed test: `--testFailureExitCode=3` exited 3 on a failed test (M4), and a config error still exits 1.
- pytest's maintainers named the cost of a shared code in 2017: "die to how exit codes are used in real-world deployments, we are in a bit of a bad position" ([#2950](https://github.com/pytest-dev/pytest/issues/2950), comment by RonnyPfannschmidt, 2017-11-25; the typo is in the original) [A].
- #2950 is closed as completed, but pytest 9.0.3 still exits 2 for a collection error (M1) [A].

## 3. Exit 0 when a test failed

| runner | how | code | what the documentation says it is for | tier | source |
|---|---|---|---|---|---|
| Jest | `--testFailureExitCode=0` | 0 | "The exit code Jest returns on test failure." No purpose given. | [A] | Configuration.md L2249-L2257; M4 |
| Jest | purpose of `testFailureExitCode` | — | [PR #4067](https://github.com/jestjs/jest/pull/4067) (merged 2017-07-19) has an empty description | [C] | `gh api repos/jestjs/jest/pulls/4067` |
| pytest plugin `pytest-custom_exit_code` | `--suppress-tests-failed-exit-code` | 0 | help text: 'Suppress the "some tests failed" exit code.' pytest's own exit-codes page links the plugin. | [A] | [plugin 0.3.0](https://github.com/yashtodi94/pytest-custom_exit_code/blob/0.3.0/pytest_custom_exit_code.py#L15-L43); exit-codes.rst |
| cargo-nextest | a test fails, then passes on retry | 0 | "By default, flaky tests are treated as ultimately successful." `--flaky-result fail` (0.9.131) changes it. | [A] | [retries.md](https://github.com/nextest-rs/nextest/blob/cargo-nextest-0.9.149/site/src/docs/features/retries.md?plain=1#L31) and L105-L120 |
| Vitest | `--dangerouslyIgnoreUnhandledErrors` | 0 | "Vitest will not fail the test run if there are unhandled errors." | [A] | [doc](https://github.com/vitest-dev/vitest/blob/v5.0.5/docs/config/dangerouslyignoreunhandlederrors.md); M4 |
| pytest | a test calls `os._exit(0)` after another test failed | 0 | not a feature; the process ends | [A] | M1 |
| `unittest` | the same | 0 | not a feature | [A] | M1 |
| `cargo test` | a test calls `std::process::exit(0)` after another test failed | 0 | not a feature | [A] | M2 |
| Jest | a test calls `process.exit(0)` after another test failed | 0 | Jest prints `process.exit called with "0"` and no summary | [A] | M4 |
| cargo-nextest | a test calls `std::process::exit(0)` | that test is `PASS` | nextest runs each test in its own process and reads its code; "the only interface between cargo test and the binary is the exit code" | [A] | M2; [how-it-works.md](https://github.com/nextest-rs/nextest/blob/cargo-nextest-0.9.149/site/src/docs/design/how-it-works.md?plain=1#L34) |
| `go test` | a test calls `os.Exit(0)` | 1 | since Go 1.16, such a test "will now be considered to fail" | [A] | [Go 1.16 notes](https://go.dev/doc/go1.16#go-test); `-test.paniconexit0` in [`test.go`](https://github.com/golang/go/blob/go1.27.3/src/cmd/go/internal/test/test.go#L1596); M3 |
| Vitest | a test calls `process.exit(0)` | 1 | Vitest throws `process.exit unexpectedly called` | [A] | M4 |
| pytest | a test calls `sys.exit(0)` | 1 | `SystemExit` is caught; the test fails | [A] | M1 |

- No runner in the survey exits 0 on a failed test by default.
- An `exit(0)` from inside a test ends four of them with 0, and no runner flag is needed.

## 4. What each integrator reads

| integrator | 0 | non-zero | ways to change it | tier | source |
|---|---|---|---|---|---|
| GitHub Actions step | `success` | `failure`: "Nonzero value (any integer but 0)" | `continue-on-error: true`: `outcome` is `failure`, `conclusion` is `success` | [A] | [set-exit-codes.md](https://github.com/github/docs/blob/02c25d16369fe27f9bdb28d1201f6b452f0d21dd/content/actions/how-tos/create-and-publish-actions/set-exit-codes.md?plain=1#L22-L29); [workflow-syntax.md](https://github.com/github/docs/blob/02c25d16369fe27f9bdb28d1201f6b452f0d21dd/content/actions/reference/workflows-and-actions/workflow-syntax.md?plain=1#L1066-L1068); [contexts.md](https://github.com/github/docs/blob/02c25d16369fe27f9bdb28d1201f6b452f0d21dd/content/actions/reference/workflows-and-actions/contexts.md?plain=1#L540-L541) |
| GitHub Actions `run:` with no `shell:` | `bash -e {0}` | no `pipefail`: `runner \| tee log` exits with `tee`'s 0 | `shell: bash` runs `bash --noprofile --norc -eo pipefail {0}` | [A] | [supported-shells.md](https://github.com/github/docs/blob/02c25d16369fe27f9bdb28d1201f6b452f0d21dd/data/reusables/actions/supported-shells.md?plain=1#L4-L8); workflow-syntax.md L962-L969; measured both lines locally (M6) |
| git `pre-commit` hook | commit goes on | "causes the `git commit` command to abort" | `--no-verify` | [A] | [githooks.adoc](https://github.com/git/git/blob/v2.56.0/Documentation/githooks.adoc?plain=1#L97-L104) |
| pre-commit framework | `Passed` | `Failed`, and it prints `- exit code: N` | a hook also fails when it modifies files | [A] | [`run.py`](https://github.com/pre-commit/pre-commit/blob/v4.7.0/pre_commit/commands/run.py#L193-L235); [new-hooks.md](https://github.com/pre-commit/pre-commit.com/blob/095b86c55470199081ca510386a6ff2ce364bba8/sections/new-hooks.md?plain=1#L10): "The hook must exit nonzero on failure or modify files." |
| `git bisect run` | good | 1–127 except 125: bad; 125: skip; below 0 or 128 and above: abort | the script maps codes itself, such as `make \|\| exit 125` | [A] | [git-bisect.adoc](https://github.com/git/git/blob/v2.56.0/Documentation/git-bisect.adoc?plain=1#L334-L350); [`bisect.c`](https://github.com/git/git/blob/v2.56.0/builtin/bisect.c#L1419-L1430); M6 |
| GNU `make` | next recipe line | "make gives up on the current rule, and perhaps on all rules"; make itself exits 2 | `-` before a line, `-i`, `.IGNORE`: ignore; `-k`: keep going, still non-zero | [A] | [Errors in Recipes](https://www.gnu.org/software/make/manual/html_node/Errors.html); [Running](https://www.gnu.org/software/make/manual/html_node/Running.html); M6 |
| tox | command passed | the environment stops and fails; tox exits non-zero | `-` prefix ignores a code; `!` inverts it; `ignore_errors` keeps going but still fails; `ignore_outcome`: "a failing result of this test environment will not make tox fail" | [A] | [explanation.rst](https://github.com/tox-dev/tox/blob/4.67.1/docs/explanation.rst?plain=1#L105-L118); [usage.rst](https://github.com/tox-dev/tox/blob/4.67.1/docs/how-to/usage.rst?plain=1#L1299-L1328); [config.rst](https://github.com/tox-dev/tox/blob/4.67.1/docs/reference/config.rst?plain=1#L1550-L1589) |
| nox | command passed | `CommandFailed`; the session fails; nox exits 1 | `success_codes`, default `[0]` | [A] | [`command.py`](https://github.com/wntrblm/nox/blob/2026.10.20/nox/command.py#L204-L214); [`sessions.py`](https://github.com/wntrblm/nox/blob/2026.10.20/nox/sessions.py#L441-L447); [`tasks.py`](https://github.com/wntrblm/nox/blob/2026.10.20/nox/tasks.py#L534-L546) |

Measured integrator behaviour (M6):

| probe | result | tier |
|---|---|---|
| `bash -e` script: `fail \| tee /dev/null` | exit 0; the next line ran | [A] |
| `bash --noprofile --norc -eo pipefail`, same script | exit 1 | [A] |
| `git bisect run`, bug at c6, runner fault at c3–c4 exits 1 | blames c3 | [A] |
| the same, fault exits 4 (pytest's usage error) | blames c3 | [A] |
| the same, fault exits 125 | blames c6, the right commit | [A] |
| the same, fault exits 128 | "bisect run failed: exit code 128 … is < 0 or >= 128" | [A] |
| `make test`, recipe exits 5 | make prints `Error 5`, then exits 2 | [A] |
| `make lint`, recipe `-sh -c "exit 1"` | exit 0; the next line ran | [A] |

- Every integrator fails on non-zero and passes on 0.
- None of them reads the runner's report to decide.
- Only `git bisect run` gives a meaning to particular non-zero codes.
- tox and nox let the user name codes; their docs point at pytest's codes for it.
- nox's own example accepts pytest's 5: `success_codes=[0, 5]`, to treat "tests discovered, but none selected" as success [A].
- tox's docs say: "Always check the documentation for the command that failed. For example, for pytest, see the pytest exit codes" (usage.rst L1828) [A].

## 5. Standards and conventions

| source | what it says | tier | link |
|---|---|---|---|
| POSIX.1-2024, utility defaults | "Usually, utilities return zero for successful completion and values greater than zero for various error conditions." A conforming application "tests for successful exit status values". | [A] | [V3_chap01](https://pubs.opengroup.org/onlinepubs/9799919799/utilities/V3_chap01.html), section EXIT STATUS |
| POSIX `grep` | 0: lines selected; 1: "No lines were selected."; >1: "An error occurred." | [A] | [grep](https://pubs.opengroup.org/onlinepubs/9799919799/utilities/grep.html) |
| POSIX `diff` | 0: no differences; 1: "Differences were found and all differences were successfully output."; >1: an error | [A] | [diff](https://pubs.opengroup.org/onlinepubs/9799919799/utilities/diff.html) |
| POSIX `cmp` | 0: identical; 1: different; >1: an error | [A] | [cmp](https://pubs.opengroup.org/onlinepubs/9799919799/utilities/cmp.html) |
| POSIX `test` | 0: true; 1: "expression evaluated to false or expression was missing."; >1: an error | [A] | [test](https://pubs.opengroup.org/onlinepubs/9799919799/utilities/test.html) |
| POSIX shell | 127: not found; 126: not executable; >128: killed by a signal | [A] | [2.8.2](https://pubs.opengroup.org/onlinepubs/9799919799/utilities/V3_chap02.html) |
| `sysexits.h` | 64 `EX_USAGE` … 70 `EX_SOFTWARE` … 78 `EX_CONFIG`; base 64 "to reduce the possibility of clashing" | [A] | [FreeBSD `sysexits.h`](https://github.com/freebsd/freebsd-src/blob/95719d8a895bf8170851e9844d9b4092b1d2b6fe/include/sysexits.h#L93-L113) |
| `sysexits(3)` | "This interface has been deprecated and is retained only for compatibility. Its use is discouraged." BUGS: "The choice of an appropriate exit value is often ambiguous." | [A] | [FreeBSD man page](https://github.com/freebsd/freebsd-src/blob/95719d8a895bf8170851e9844d9b4092b1d2b6fe/share/man/man3/sysexits.3#L37-L38) and L131-L135 |
| TAP 14, harness | a test program fails if its TAP shows a failure, its TAP is invalid, or "The exit code of the test program is not 0" | [A] | [spec](https://github.com/TestAnything/Specification/blob/077697618f533b7701c758b717c2b3a48a4be952/tap-version-14-specification.md?plain=1#L129-L141) |
| TAP 14, harness output | "a command-line test runner might exit with a non-zero status code" when a test program fails | [A] | same, L138-L141 |
| TAP 14, plan | "A Harness _must_ treat a TAP stream lacking a plan as a failed test." A missing test point counts as failed. `1..0` is a skip. | [A] | same, L186, L286-L300, L209-L213 |
| TAP 14, TODO | "Harnesses _must not_ treat failing `TODO` test points as a test failure." | [A] | same, L488 |
| Perl `Test::More` | 0: all passed; 255: "test died or all passed but wrong # of tests run"; other: how many failed, up to 254. "This behavior may go away in future versions." | [A] | [More.pm](https://github.com/Test-More/test-more/blob/v1.302226/lib/Test/More.pm#L1833-L1851) |
| a standard written for test-runner exit codes | none found | [C] | searched POSIX, `sysexits`, TAP 13 and 14 |

- POSIX checking utilities keep "the answer is no" (1) apart from "I failed" (>1).
- nextest's `DOUBLE_SPAWN_ERROR` is 70, the value of `EX_SOFTWARE` (S3) [A]; I could not establish whether that is on purpose [C].
- TAP reads two channels, the exit code and the stream, and fails the program on either one.

## 6. Documented cases

| case | what happened | integrator | tier | source |
|---|---|---|---|---|
| `go test` and `os.Exit(0)`, 2018 | a test that called code with `os.Exit(0)` stopped the run; the next test, which fails, never ran; `go test` exited 0. "CI also cannot detect it." Fixed in Go 1.16. | CI | [A] | [golang/go#29062](https://github.com/golang/go/issues/29062); Go 1.16 notes |
| nextest, 2023 | "our CI has actually not been running certain tests suites"; a typo `-profile ci` was read as a test filter; 0 tests ran and the run passed | CI | [A] the report; [B] the CI details | [nextest#1107](https://github.com/nextest-rs/nextest/issues/1107), [#1108](https://github.com/nextest-rs/nextest/issues/1108) |
| nextest's answer, 2024 | exit 4 when no test runs: "Running zero tests likely indicates some kind of configuration issue." | — | [A] | [nextest#1639](https://github.com/nextest-rs/nextest/issues/1639) |
| CPython regrtest, 2022 | a typo in a test name gave "NO TEST RUN" with exit 0, "and so the mistake can be easily missed"; led to exit 5 in `unittest` 3.12 | — | [A] | [cpython#98903](https://github.com/python/cpython/issues/98903) |
| pytest-xdist on GitHub Actions, 2024 | pytest "will abruptly exit with code 0", "which causes the CI run to be marked as success even though not all the tests ran" | GitHub Actions, self-hosted on Kubernetes | [A] the report; [B] the cause | [pytest-xdist#1106](https://github.com/pytest-dev/pytest-xdist/issues/1106); the reporter blames [runner-container-hooks#165](https://github.com/actions/runner-container-hooks/issues/165) ("Job Steps Incorrectly Marked as Successful") |
| pytest-xdist and an internal error, 2024 | with `-n 2`, a hook that raises gives `INTERNALERROR>` lines but exit 0, not 3 | pytester | [A] the report, still open; [C] not reproduced | [pytest-xdist#1017](https://github.com/pytest-dev/pytest-xdist/issues/1017) |
| Jest 27.3.1, 2021 | "It will be failure but exit with 0"; "Github Action CI has this bug too." Closed by the stale bot; no cause found. | GitHub Actions | [B] | [jest#12094](https://github.com/jestjs/jest/issues/12094) |
| Vitest 0.24.3, 2022 | a failed test, then `echo $?` printed 0; closed as not planned without an accepted reproduction | shell | [B] | [vitest#2363](https://github.com/vitest-dev/vitest/issues/2363) |
| pytest keeps 5, 2017 | maintainers refused to suppress exit 5: "the very reason for having and not suppressing this error code is to learn of broken/gone bad ci setups" | CI | [A] | [pytest#2393](https://github.com/pytest-dev/pytest/issues/2393) |
| `unittest` 3.12 and skipped tests, 2024 | all tests skipped exited 5, and `make` stopped: `make[1]: *** [Makefile:645: py3-test] Error 5`; apache/thrift added a dummy test; CPython then exited 0 when a test was skipped | `make` | [A] | [cpython#113661](https://github.com/python/cpython/issues/113661); current rule in S2 L274 |
| Jest and a pre-commit hook, 2017 | `--findRelatedTests` with no related test exited 1 and failed a lint-staged pre-commit hook; Jest changed it to exit 0 | pre-commit | [A] | [jest#5112](https://github.com/jestjs/jest/issues/5112) |
| Jest 30.5.2 and `--findRelatedTests` | the exit-0 list in the source names `passWithNoTests`, `lastCommit` and `onlyChanged`, not `findRelatedTests` | — | [A] source; [C] not measured | `getNoTestsFoundMessage.ts` L20-L23 |
| broken code shipped to users because a runner exited 0 on a failed test | no postmortem found | — | [C] | searches in Appendix A, C1 |

- Every documented case is one shape: the runner exited 0, and the integrator read 0 as clean.
- Two cases go the other way: a "no test" code broke `make` (cpython#113661) and a pre-commit hook (jest#5112), and the runner eased its rule.

## What this means for purba

| evidence | for (a), a verdict on the tests | for (b), only whether purba ran well |
|---|---|---|
| every integrator fails on non-zero and passes on 0 (item 4) | a failed test fails the CI step, the hook, `make`, tox and nox | a failed test exits 0, so each integrator passes it, unless it parses the report or a wrapper maps the result |
| every documented case is a runner that exited 0 (item 6) | — | exit 0 on a failed test is the designed behaviour, not a defect |
| `git bisect run` reads 1–127 as bad and 125 as skip (item 4, M6) | purba's 3 ("the run did not finish") reads as bad; bisect can blame the wrong commit | a fault exits non-zero and reads as bad too; a failed test reads as good, so bisect cannot find the bug |
| POSIX `grep`, `diff`, `cmp`, `test` use 0 yes, 1 no, >1 error (item 5) | the same shape: 1 is the answer, a higher code is a fault | no POSIX checking utility found with this shape |
| TAP fails a program on a non-zero code or a failing stream (item 5) | the code and the report agree | the code and the report would disagree on purpose |
| integrators accept chosen codes: nox `success_codes=[0, 5]`, the pytest#2393 wrapper `[ $ret = 5 ] && exit 0` | each kind of result needs its own code for this to work | — |
| runners that share one code for a fault and a failed test: `cargo test`, Jest, Vitest, `go test` (item 2) | the record keeps a fault (3) apart from a failed test (1); its Downside says 4 carries both an error inside purba and an error of the tester's code | a fault is the only non-zero code, so a fault and a failed test cannot share one |

Facts that bear on the record itself:

- **The record's pytest row lists 0–5.** pytest 9.1.1 also has 6, `MAX_WARNINGS_ERROR`, at L121-L122 of the file it links [A].
- **pytest's 2 is not only an interrupt.** A test module that fails to import exits 2 (M1). An integrator that reads pytest's 2 reads both [A].
- **An `exit(0)` inside a test ended four runners with 0 after a failed test** (item 3). The record's rule "exits 0 only when it checked something" holds only if purba's process survives a test that calls exit. I could not establish how purba runs a test's code [C].
- **Go and nextest detect this.** Go turns `os.Exit(0)` in a test into a failure; nextest still reads that test as a pass (M2, M3) [A].
- **No surveyed runner exits 125.** An integrator that bisects must map a fault to 125 itself, under either design [A].

## Appendix A. Commands

Sources (S) and searches (C) were fetched this session, 2026-10-24.
`$P` is the scratch directory; each fixture lived in its own directory under `$P/run/`.

| key | command |
|---|---|
| S1 | `curl -sfL https://raw.githubusercontent.com/pytest-dev/pytest/9.1.1/src/_pytest/{config/__init__.py,main.py}` and `.../doc/en/reference/exit-codes.rst` |
| S2 | `curl -sfL https://raw.githubusercontent.com/python/cpython/v3.15.1/{Lib/unittest/main.py,Doc/library/unittest.rst,Lib/argparse.py}` |
| S3 | `curl -sfL https://raw.githubusercontent.com/nextest-rs/nextest/cargo-nextest-0.9.149/{nextest-metadata/src/exit_codes.rs,site/src/docs/running.md,site/src/docs/features/retries.md,site/src/docs/design/how-it-works.md}` |
| S4 | `curl -sfL https://raw.githubusercontent.com/rust-lang/rust/1.99.1/library/test/src/lib.rs`; `.../rust-lang/cargo/0.100.0/{src/util/errors.rs,doc/man/generated_txt/cargo-test.txt}` |
| S5 | `curl -sfL https://raw.githubusercontent.com/golang/go/go1.27.3/src/cmd/go/internal/{test/test.go,base/base.go}`; `curl -sfL https://go.dev/doc/go1.16`; `gh api repos/golang/go/issues/29062` |
| S6 | `curl -sfL https://raw.githubusercontent.com/jestjs/jest/v30.5.2/{packages/jest-cli/src/run.ts,packages/jest-core/src/runJest.ts,packages/jest-core/src/getNoTestsFoundMessage.ts,docs/Configuration.md,docs/CLI.md}`; `gh api repos/jestjs/jest/pulls/4067` |
| S7 | `curl -sfL https://raw.githubusercontent.com/vitest-dev/vitest/v5.0.5/{packages/vitest/src/node/core.ts,packages/vitest/src/node/cli/cli-api.ts,packages/vitest/src/node/cli/cac.ts,docs/config/passwithnotests.md,docs/config/dangerouslyignoreunhandlederrors.md}` |
| S8 | `curl -sfL https://raw.githubusercontent.com/github/docs/02c25d16369fe27f9bdb28d1201f6b452f0d21dd/{content/actions/how-tos/create-and-publish-actions/set-exit-codes.md,content/actions/reference/workflows-and-actions/workflow-syntax.md,content/actions/reference/workflows-and-actions/contexts.md,data/reusables/actions/supported-shells.md}` |
| S9 | `curl -sfL https://raw.githubusercontent.com/pre-commit/pre-commit/v4.7.0/pre_commit/commands/run.py`; `.../pre-commit/pre-commit.com/095b86c55470199081ca510386a6ff2ce364bba8/sections/new-hooks.md`; `.../git/git/v2.56.0/{Documentation/githooks.adoc,Documentation/git-bisect.adoc,builtin/bisect.c}` |
| S10 | `curl -sfL https://www.gnu.org/software/make/manual/html_node/{Errors,Running}.html` |
| S11 | `curl -sfL https://raw.githubusercontent.com/tox-dev/tox/4.67.1/docs/{explanation.rst,how-to/usage.rst,reference/config.rst}`; `.../wntrblm/nox/2026.10.20/nox/{command.py,sessions.py,tasks.py}` |
| S12 | `curl -sfL https://pubs.opengroup.org/onlinepubs/9799919799/utilities/{V3_chap01,V3_chap02,grep,diff,cmp,test}.html` |
| S13 | `curl -sfL https://raw.githubusercontent.com/freebsd/freebsd-src/release/15.1.0-p6/{include/sysexits.h,share/man/man3/sysexits.3}` |
| S14 | `curl -sfL https://raw.githubusercontent.com/TestAnything/Specification/077697618f533b7701c758b717c2b3a48a4be952/tap-version-14-specification.md`; `.../Test-More/test-more/v1.302226/lib/Test/More.pm` |
| S15 | `curl -sfL https://raw.githubusercontent.com/yashtodi94/pytest-custom_exit_code/0.3.0/pytest_custom_exit_code.py` |
| C1 | `gh search issues "<query>" --repo <repo>` for: jest "exit code 0 failed", "exits with 0", "testFailureExitCode"; vitest "exit code 0", "exit code 0 CI"; pytest "exit code 0 failed", "os._exit", "exit code 5", "CI green failed"; pytest-xdist "exit code 0"; rust "process::exit test exit code 0"; nextest "no tests"; cpython "NO TESTS RAN"; actions/runner "pipefail", "pipe exit code"; pre-commit "exit code 0 hook failed" |
| C2 | `gh api repos/<repo>/issues/<n>` and `.../issues/<n>/comments` for each issue in item 6 |
| M1 | `nix-shell -p 'python3.withPackages(ps: [ps.pytest])' --run 'bash py.sh $P/run/py'`; each case runs `python -I -m pytest -q -p no:cacheprovider` or `python -I -m unittest discover -s .` in its own fixture directory |
| M2 | `nix-shell -p cargo rustc cargo-nextest --run 'bash rs.sh $P/run/rs'`; each case runs `cargo test --lib -q` and `cargo nextest run --lib` with `CARGO_HOME` and `CARGO_TARGET_DIR` in scratch |
| M3 | `nix-shell -p go --run 'bash go.sh $P/run/go'` with `GOCACHE`, `GOPATH` in scratch, `GOTOOLCHAIN=local`; each case runs `go test ./...`; plus `go test -count=x ./...`, `go test -covermode=nope ./...`, `go nosuchcmd` |
| M4 | `nix-shell -p nodejs --run 'bash js.sh $P/run/js'`, plus `jest --ci --nosuchflag` and `jest --ci --no-such-flag`; `npm install --no-save jest@30.5.2 vitest@5.0.5` in `$P/run/js/pkgs`; each case runs `jest --ci` or `vitest run` in its own fixture directory with `CI=true` |
| M5 | the `osexit`, `exit0`, `jexit` and `vexit` fixtures of M1–M4: test A fails, then test B calls the language's `exit(0)` |
| M6 | `nix-shell -p git gnumake bash --run 'bash integrators.sh $P/run/integ'`, with `GIT_CONFIG_GLOBAL=/dev/null`, `GIT_CONFIG_NOSYSTEM=1` and git's repo variables unset; 8 commits, `check.sh` exits 1 from c6 and exits N at c3–c4; `git bisect start c8 c1; git bisect run check.sh` for N in 1, 4, 125, 128 |
| L | `gh api repos/O/R/commits/<branch> --jq .commit.committer.date`; `gh api repos/O/R/releases/latest --jq '.tag_name+" "+.published_at'`; for repos with no releases, `git ls-remote --tags --refs` then `gh api repos/O/R/commits/<tag> --jq .commit.committer.date` |

## Appendix B. Versions measured

Sources were read at the newest tag; measurements ran the version nixpkgs or npm gave.

| tool | source read | version measured |
|---|---|---|
| pytest | 9.1.1 | 9.0.3, CPython 3.13.15 |
| `unittest` | CPython v3.15.1 | CPython 3.13.15 (exit 5 exists since 3.12) |
| `cargo test` | rust 1.99.1, cargo 0.100.0 | cargo and rustc 1.95.0 |
| cargo-nextest | 0.9.149 | 0.9.136 |
| `go test` | go1.27.3 | go1.26.8 |
| Jest | v30.5.2 | 30.5.2, node 24.21.0 |
| Vitest | v5.0.5 | 5.0.5, node 24.21.0 |
| git | v2.56.0 | 2.54.0 |
| GNU make | manual for 4.4.1 | 4.4.1 |
| bash | — | 5.3.9 |

## Appendix C. Liveness

| project | default branch, last commit | latest release or tag | date |
|---|---|---|---|
| pytest-dev/pytest | `main`, 2026-10-23 | release 9.1.1 | 2026-06-19 |
| python/cpython | `main`, 2026-10-24 | tag v3.15.1 (no GitHub releases) | 2026-10-21 (tag commit) |
| rust-lang/rust | `main`, 2026-10-24 | release 1.99.1 | 2026-10-16 |
| rust-lang/cargo | `master`, 2026-10-23 | tag 0.100.0 (no GitHub releases) | 2026-08-28 (tag commit) |
| nextest-rs/nextest | `main`, 2026-10-24 | release cargo-nextest-0.9.149 | 2026-10-20 |
| golang/go | `master`, 2026-10-23 | tag go1.27.3 (no GitHub releases) | 2026-10-20 (tag commit) |
| jestjs/jest | `main`, 2026-10-02 | release v30.5.2 | 2026-09-18 |
| vitest-dev/vitest | `main`, 2026-10-24 | release v5.0.5 | 2026-10-15 |
| git/git | `master`, 2026-10-22 | tag v2.56.0 (no GitHub releases) | 2026-09-28 (tag commit) |
| GNU make (savannah) | `b380278`, 2026-04-29 | 4.4.1 tarball | 2023-02-26 |
| tox-dev/tox | `main`, 2026-10-20 | release 4.67.1 | 2026-10-18 |
| wntrblm/nox | `main`, 2026-10-20 | release 2026.10.20 | 2026-10-20 |
| pre-commit/pre-commit | `main`, 2026-10-20 | release v4.7.0 | 2026-10-13 |
| pre-commit/pre-commit.com | `main`, 2026-10-20 | — (docs site) | — |
| github/docs | `main`, 2026-10-23 | — (docs site) | — |
| TestAnything/Specification | `master`, 2022-12-15 | — (no tag) | — |
| Test-More/test-more | `master`, 2026-10-19 | tag v1.302226 | 2026-10-19 (tag commit) |
| freebsd/freebsd-src | `main`, 2026-10-24 | tag release/15.1.0-p6 | 2026-10-20 (tag commit) |
| yashtodi94/pytest-custom_exit_code | `master`, 2020-02-08 | release 0.3.0 | 2019-08-24 |
| POSIX.1-2024 | — (The Open Group, Issue 8) | — | — |
