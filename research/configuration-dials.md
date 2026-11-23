# Which dials comparable test runners offer, and which dials purba already asks for

Research for [Which dials do comparable test runners offer, and which dials do purba's records and issues already ask for?](https://github.com/kalonji-tools/purba/issues/1211)
Measured on 2026-11-23, against purba `main` at `27e8f559`, on one Linux machine.
This file decides nothing. It lists the dials, sorts them, and shows which ones change an outcome.

## Summary

All six runners have many dials. unittest has the fewest, 23, and Vitest has the most, 208. In total I list 715. Most of them change how the runner behaves and touch no check: what it prints, which tests it selects, how many workers it uses, or how it builds. Each runner also has dials that do more. Some tighten a check. Some turn a check off. A large part can change the outcome of a test: 216 of the 715.

Four findings matter most for purba's design of configuration.

- **Every runner has dials that turn a check off or loosen it, and testers use them every day.** There are 38 such dials. Examples: retries in cargo-nextest, Vitest and Jest, `-vet=off` in `go test`, `-p no:<plugin>` in pytest, and `--passWithNoTests` and `--updateSnapshot` in Jest and Vitest. purba's rule for configuration refuses each of them. I probed eight of them, and each one changed an outcome: `-p no:<name>`, the warning filters, `-vet=off`, `--retries`, `--retry`, `--allowOnly`, and a test's own timeout in Vitest and in Jest.
- **A number tightens a check in one direction and loosens it in the other.** A timeout tightens when it gets lower. It turns the check off at 0, in `go test`, Vitest and pytest-timeout. I mark 32 dials as two-way. So the rule "a value tightens a check" can judge a value, but not a dial. purba's cap on suppressed verdicts already answers this case for one dial: where the integrator sets both the key and the flag, the lower value applies.
- **A value set outside the project changes outcomes in five runners.** I probed each of these:
  - `PYTEST_ADDOPTS` and `PYTEST_TIMEOUT` in pytest,
  - `NEXTEST_RETRIES` in cargo-nextest,
  - `GOFLAGS`, and the user's file that `go env -w` writes,
  - the variable `CI` in Vitest and Jest.

  With `CI` set, a `.only` fails a Vitest run, and a new snapshot fails a Jest test. A test that passes on a developer's machine fails in CI by design.
- **Where code sets a timeout, code wins.** pytest-timeout's marker, Vitest's test option and Jest's third argument each beat the timeout on the command line. `go test` has no timeout in code. cargo-nextest sets one test's timeout in the project file, through an override.

purba asks for few dials. A merged record decides one: the cap on suppressed verdicts, set by a configuration key and a flag. It is not built. The records or issues ask for nine others, and decide none of their levels: the timeout of a test, the number of workers, outcomes as data, JUnit XML, picking a run's files by platform, the import roots, the query, `skip_if` and `known_failure`. The records refuse more dials than they ask for: a mode of strictness, a silent opt-out, a suppression by a path pattern, a plugin, `conftest.py`, and a flag for how much of a value shows. Section 6 lists each with its link.

## How to read this file

- **[run]** means I ran it in this session, on the version named. **[source]** means the runner's code or documentation says so, at the pinned version, and I did not run it. **judgement** marks a classification that no source states.
- A **dial** is one value that changes how a runner behaves: a number, a switch or a trigger. One row merges every name that sets the same value: a flag, its alias, its key in a file, its environment variable and its form in code.
- **Class.** Each dial has one class. I take it from the dial's purpose, as its documentation states it.
  - **F**: it states a fact about the project, such as where the tests live or what goes on `sys.path`.
  - **T**: it tightens a check, or adds one.
  - **B**: it changes how the runner behaves, and touches no check.
  - **L**: it turns a check off, or loosens it.
  - **↕** marks a dial that also moves its check the other way. A timeout is T ↕: a value of 0 turns it off.
- **Outcome.** `test` means the dial can change the outcome of a test that runs: passed, failed, error, skipped, or expected failure. `run` means it changes only the run: its exit, or which tests get an outcome. A blank means neither.
- **Levels.** `C` is the command line, `E` an environment variable, `P` a file of the project, `K` the code of a test, and `U` a file of the user. No runner here reads a file of the system for these dials. [Which levels of configuration do test runners and Python tools read?](https://github.com/kalonji-tools/purba/issues/1212) studies the levels and their order. This file records only where each dial can be set.
- **Groups.** Each dial acts on one of these:

  | code | acts on |
  |---|---|
  | sel | which tests run |
  | ord | order and randomness |
  | par | parallel runs and workers |
  | tmo | timeouts |
  | rep | retries and repeats |
  | stp | when the run stops, and its exit |
  | chk | warnings and strictness |
  | fix | fixtures, setup and the environment of a test |
  | imp | discovery, import paths and the layout of the project |
  | ast | assertions, diffs and snapshots |
  | out | output and reports |
  | cap | capture and logging |
  | cch | caches and saved state |
  | plg | plugins and other extension |
  | dbg | debugging and inspection |
  | cov | coverage and profiling |
  | bld | the build |
  | cfg | the configuration itself |

## Method

### Versions

| runner | version | ran on | list taken from [run] | pinned source |
|---|---|---|---|---|
| pytest | 9.2.0, the newest on PyPI | CPython 3.13.13, a `uv` venv, `-I` | `pytest -p no:timeout --help`, `pytest --markers` | [`doc/en/reference/reference.rst` at `9.2.0`](https://github.com/pytest-dev/pytest/blob/9.2.0/doc/en/reference/reference.rst) |
| pytest-timeout | 2.4.0 | the same venv | probes only, not counted | [`pytest_timeout.py` at `2.4.0`](https://github.com/pytest-dev/pytest-timeout/blob/2.4.0/pytest_timeout.py) |
| unittest | CPython 3.13.13 | the same venv, `-I` | `-m unittest --help`, `-m unittest discover --help` | [`Lib/unittest/` at `v3.13.13`](https://github.com/python/cpython/tree/v3.13.13/Lib/unittest) |
| cargo-nextest | 0.9.149, with rustc 1.99.0. The newest is 0.9.152; I ran the installed one. | this machine | `cargo nextest run --help`, `cargo nextest help repo-config`, `cargo nextest help user-config` | [`repo-config-reference.md`](https://github.com/nextest-rs/nextest/blob/cargo-nextest-0.9.149/nextest-runner/repo-config-reference.md), [`user-config-reference.md`](https://github.com/nextest-rs/nextest/blob/cargo-nextest-0.9.149/nextest-runner/user-config-reference.md), [`env-vars.md`](https://github.com/nextest-rs/nextest/blob/cargo-nextest-0.9.149/site/src/docs/configuration/env-vars.md) at `cargo-nextest-0.9.149` |
| `go test` | go 1.26.10, from nixpkgs. The newest is 1.27.4. | `nix-shell -p go` | `go help testflag`, `go help test`, `go help build`, `go help environment` | [`src/cmd/go/alldocs.go` at `go1.26.10`](https://github.com/golang/go/blob/go1.26.10/src/cmd/go/alldocs.go) |
| Vitest | 5.1.1, the newest on npm | Node 24.22.1 | `vitest --help --expand-help` | [`cli-config.ts`](https://github.com/vitest-dev/vitest/blob/v5.1.1/packages/vitest/src/node/cli/cli-config.ts), [`config.ts`](https://github.com/vitest-dev/vitest/blob/v5.1.1/packages/vitest/src/node/types/config.ts), [`types.ts`](https://github.com/vitest-dev/vitest/blob/v5.1.1/packages/vitest/src/runtime/runner/types.ts#L628) at `v5.1.1` |
| Jest | 30.5.2, the newest on npm | Node 24.22.1 | `jest --help` | [`args.ts`](https://github.com/jestjs/jest/blob/v30.5.2/packages/jest-cli/src/args.ts), [`raw-types.ts`](https://github.com/jestjs/jest/blob/v30.5.2/packages/jest-schemas/src/raw-types.ts#L229), [`docs/`](https://github.com/jestjs/jest/tree/v30.5.2/docs) at `v30.5.2` |

### What I counted

The help of each runner is the measured list. I added a dial from source only where the help cannot show it: a key with no flag, or a form in code.

- **pytest [run].** Each option, each key of the file and each environment variable that `pytest --help` prints, with `-p no:timeout` so that only pytest's own dials print. Each built-in marker that `pytest --markers` prints. Plugins are not counted.
- **unittest [run] and [source].** Each option of the two helps. Each parameter of [`unittest.main()`](https://github.com/python/cpython/blob/v3.13.13/Lib/unittest/main.py#L66-L70) and [`TextTestRunner`](https://github.com/python/cpython/blob/v3.13.13/Lib/unittest/runner.py#L172-L174), each attribute of [`TestLoader`](https://github.com/python/cpython/blob/v3.13.13/Lib/unittest/loader.py#L68-L71) and [`TestCase`](https://github.com/python/cpython/blob/v3.13.13/Lib/unittest/case.py#L395-L399), and the [decorators](https://github.com/python/cpython/blob/v3.13.13/Lib/unittest/case.py#L148-L184).
- **cargo-nextest [run] and [source].** Each option of `cargo nextest run --help`, but no other subcommand. Each key of `help repo-config` and `help user-config`. Each environment variable that `env-vars.md` says nextest reads and that no flag mirrors. libtest's `#[ignore]`.
- **`go test` [run] and [source].** Each flag of `go help testflag`, `go help test` and `go help build`. `go help environment` lists many more variables; I counted 9 that change a test run, by judgement. In code: `t.Parallel`, `t.Skip`, `t.Setenv`, `TestMain` and `testing.Short`, from [`src/testing/testing.go`](https://github.com/golang/go/blob/go1.26.10/src/testing/testing.go#L687).
- **Vitest [run] and [source].** Each option line of `--help --expand-help`, less 8 lines that only group sub-options, such as `--sequence <options>`. Each config key that `cli-config.ts` maps to `null`, which means it has no flag. I left out 10 of those 40 as internal or as a copy: `middlewareMode`, `reporter`, `inspector`, `cliExclude`, `related`, `json`, `filesOnly`, `staticParse`, `staticParseConcurrency` and `benchmarkOnly`. Four callbacks of `config.ts`. The test options of `types.ts`.
- **Jest [run] and [source].** Each option line of `jest --help`. Each key of `InitialOptions` in `raw-types.ts` that `args.ts` gives no flag: 15 keys. In code: [`jest.retryTimes`](https://github.com/jestjs/jest/blob/v30.5.2/docs/JestObjectAPI.md#L1185), and `test.skip`, `test.only`, `test.todo`, `test.failing` and `test.concurrent` from [`GlobalAPI.md`](https://github.com/jestjs/jest/blob/v30.5.2/docs/GlobalAPI.md#L503). `jest.setTimeout` and the `@jest-environment` docblock join the row of their flag.

`research/configuration-dials/dials/<runner>.tsv` holds one row for each dial, with its group, levels, class, outcome and a note. `research/configuration-dials/count.sh` prints every count in this file from those rows. A count answers only these rules. A different merge of aliases gives a different number.

### Bridges

The level columns list where a runner sets a dial directly. Five runners also have a bridge that carries every dial to another level [run, from each help]:

- pytest's key `addopts` puts any flag in the project's file, and `PYTEST_ADDOPTS` puts any flag in the environment. `-o key=value` puts any key of the file on the command line.
- cargo-nextest's `[[profile.<name>.overrides]]` gives one test, or one platform, its own value of most profile keys.
- `go test` reads any flag from `GOFLAGS`. `go env -w GOFLAGS=…` writes it to a file of the user.
- Vitest gives each key of `InlineConfig` a flag, so most dials are both C and P.
- Jest's `--config` takes a JSON string as well as a path.

### Probes

Each probe is a script under `research/configuration-dials/<runner>/run.sh`, beside its test file. Each script copies its tests into an empty directory that it is given, and runs them there. So no file of purba's tree, such as its `pyproject.toml`, reaches the run. Each table below shows the dial set, the outcome, and the exit. The full output is what the script prints.

```sh
R=research/configuration-dials
bash $R/pytest/run.sh <python with pytest 9.2.0 and pytest-timeout 2.4.0> <empty dir>
bash $R/unittest/run.sh <python 3.13> <empty dir>
bash $R/nextest/run.sh <empty dir>
nix-shell -p go --run "bash $R/go/run.sh <empty dir>"
nix-shell -p nodejs --run "bash $R/vitest/run.sh <dir with node_modules/vitest@5.1.1> <empty dir>"
nix-shell -p nodejs --run "bash $R/jest/run.sh <dir with node_modules/jest@30.5.2> <empty dir>"
```

## 1. How many dials, and what they act on

All counts below are [run] on the rows, and each row is [run] or [source] as section "What I counted" says.

### Dials per runner

| | pytest | unittest | nextest | go | vitest | jest |
|---|---|---|---|---|---|---|
| dials | 141 | 23 | 138 | 78 | 208 | 127 |

unittest is small because it has no file of its own. Code sets most of its dials, through `unittest.main()` and the attributes of its classes. The others are large for different reasons. cargo-nextest and `go test` each carry about 23 dials of the build. Vitest carries 29 of coverage and 25 of a test's environment, such as a browser and its mocks.

### Dials per group

| group | pytest | unittest | nextest | go | vitest | jest |
|---|---|---|---|---|---|---|
| sel | 15 | 3 | 23 | 5 | 18 | 19 |
| ord | 2 | 1 | 1 | 1 | 6 | 3 |
| par | 0 | 0 | 4 | 5 | 7 | 7 |
| tmo | 2 | 0 | 7 | 1 | 7 | 1 |
| rep | 0 | 0 | 4 | 5 | 6 | 1 |
| stp | 6 | 2 | 3 | 1 | 5 | 6 |
| chk | 12 | 2 | 0 | 4 | 17 | 4 |
| fix | 2 | 0 | 10 | 5 | 25 | 16 |
| imp | 13 | 4 | 6 | 1 | 7 | 16 |
| ast | 5 | 3 | 0 | 0 | 23 | 7 |
| out | 35 | 3 | 32 | 6 | 23 | 16 |
| cap | 15 | 1 | 3 | 0 | 2 | 0 |
| cch | 6 | 0 | 7 | 1 | 6 | 7 |
| plg | 7 | 3 | 0 | 0 | 1 | 4 |
| dbg | 15 | 1 | 4 | 4 | 20 | 9 |
| cov | 0 | 0 | 0 | 13 | 29 | 8 |
| bld | 0 | 0 | 23 | 23 | 0 | 0 |
| cfg | 6 | 0 | 11 | 3 | 6 | 3 |

Three points stand out.

- **pytest has no dial for workers or retries, and no timeout that fails one test.** Plugins add them, such as pytest-timeout. pytest's two timeouts come from faulthandler: one dumps the stacks, and `faulthandler_exit_on_timeout` ends the whole process. Each of the other five runners has its own timeout.
- **cargo-nextest has the most timeouts, and none of them is on the command line.** Its 7 timeouts live only in the project's file, with a per-test override. The flags `--debugger` and `--tracer` turn them off.
- **Output is the largest group in pytest and cargo-nextest.** 35 and 32 dials change only what the tester or the integrator reads.

### Dials per class

| class | pytest | unittest | nextest | go | vitest | jest |
|---|---|---|---|---|---|---|
| F | 16 | 4 | 8 | 6 | 11 | 17 |
| T | 10 | 1 | 12 | 4 | 19 | 6 |
| B | 108 | 16 | 112 | 66 | 164 | 97 |
| L | 7 | 2 | 6 | 2 | 14 | 7 |
| ↕ (both ways) | 5 | 1 | 9 | 2 | 13 | 2 |
| changes a test's outcome | 31 | 8 | 40 | 26 | 62 | 49 |
| changes only the run | 31 | 7 | 31 | 7 | 39 | 29 |
| L, or ↕, that changes a test's outcome | 7 | 3 | 12 | 3 | 11 | 5 |

B is the largest class in every runner, from 70% of the dials in unittest to 85% in go. A dial of class B can still change a test's outcome. Examples are the order of tests, the number of workers, the mocks that reset between tests, and the fixtures that a file of the project adds. purba's rule counts such a dial as one that touches no check. But each one can change an outcome, so it meets purba's risk [an outcome depends on the worker](https://github.com/kalonji-tools/purba/blob/27e8f559/LANDSCAPE.md#L168).

### Dials per level

| level | pytest | unittest | nextest | go | vitest | jest |
|---|---|---|---|---|---|---|
| C | 89 | 13 | 84 | 65 | 165 | 106 |
| E | 8 | 1 | 25 | 9 | 4 | 1 |
| P | 56 | 0 | 46 | 1 | 180 | 105 |
| K | 9 | 19 | 1 | 7 | 13 | 9 |
| U | 0 | 0 | 16 | 6 | 0 | 0 |
| C only | 71 | 4 | 61 | 63 | 19 | 16 |
| two levels or more | 20 | 9 | 23 | 9 | 146 | 90 |

These counts leave out the bridges. Through `addopts`, `PYTEST_ADDOPTS` and `-o`, every pytest dial except a marker reaches C, E and P. Through `GOFLAGS`, every `go test` flag reaches E and U. Only cargo-nextest and `go test` read a file of the user. cargo-nextest's file of the user holds only how the output looks, and records of past runs. Through `GOFLAGS`, `go test`'s file can hold any flag, and section 2.4 shows it change an outcome.

## 2. Dials that change an outcome: the probes

Each row is [run]. "Outcome" is the outcome of the one probed test.

### 2.1 pytest 9.2.0, with pytest-timeout 2.4.0

The test `test_sleeps_two_seconds` sleeps 2 s. The test `test_warns_deprecation` raises a `DeprecationWarning` and passes.

| case | what is set | outcome | exit |
|---|---|---|---|
| timeout-none | nothing | PASSED | 0 |
| timeout-cli | `--timeout=1` | FAILED, `Timeout (>1.0s) from pytest-timeout` | 1 |
| timeout-env | `PYTEST_TIMEOUT=1` | FAILED | 1 |
| timeout-ini | `timeout = 1` in `pytest.ini` | FAILED | 1 |
| timeout-marker-vs-cli | `@pytest.mark.timeout(5)` in code, `--timeout=1` | PASSED: code wins | 0 |
| timeout-plugin-off | `timeout = 1` in the file, `-p no:timeout` | PASSED. The key becomes `PytestConfigWarning: Unknown config option: timeout` | 0 |
| warn-cli-error | `-W error::DeprecationWarning` | FAILED | 1 |
| warn-env-addopts | `PYTEST_ADDOPTS=-W error::DeprecationWarning` | FAILED | 1 |
| warn-ini-error | `filterwarnings = error::DeprecationWarning` in the file | FAILED | 1 |
| warn-ini-error-cli-ignore | the file says error, `-W ignore::DeprecationWarning` | PASSED: the command line wins | 0 |
| warn-ini-error-cli-override | the file says error, `-o filterwarnings=ignore::DeprecationWarning` | PASSED | 0 |
| warn-ini-error-marker-ignore | the file says error, `@pytest.mark.filterwarnings("ignore::DeprecationWarning")` | PASSED: code wins | 0 |
| warn-ini-error-plugin-off | the file says error, `-p no:warnings` | PASSED | 0 |
| warn-max-warnings | `--max-warnings=0` | PASSED, `Tests pass, but maximum allowed warnings exceeded: 2 > 0` | 6 |
| marker-strict | an unregistered marker, `--strict-markers` | ERROR at collection | 2 |
| xfail-strict | an `xfail` test that passes, `-o strict_xfail=true` | FAILED `[XPASS(strict)]`; without it, XPASS and exit 0 | 1 |
| xfail-runxfail | an `xfail` test that fails, `--runxfail` | FAILED; without it, XFAIL | 1 |
| key-misspelled | `filterwarning = error` in the file | PASSED, with `PytestConfigWarning: Unknown config option: filterwarning` | 0 |
| key-misspelled-strict | the same, `--strict-config` | `ERROR: Unknown config option: filterwarning` | 4 |

`pytest --help` says that `filterwarnings` is "Processed after -W/--pythonwarnings". The probe shows that `-W` wins. pytest's code agrees with the probe: ["Filters should have this precedence: cmdline options, config"](https://github.com/pytest-dev/pytest/blob/9.2.0/src/_pytest/config/__init__.py#L2483) [source]. A marker [applies its filters after both](https://github.com/pytest-dev/pytest/blob/9.2.0/src/_pytest/warnings.py#L34-L39) [source].

### 2.2 unittest, CPython 3.13.13

`test_b_warns_deprecation` raises a `DeprecationWarning`. `test_c_expected_failure_that_passes` carries `@expectedFailure` and passes.

| case | what is set | outcome | exit |
|---|---|---|---|
| default | nothing | `test_b` passes and prints the warning; `test_c` is an unexpected success | 1 |
| interpreter `-W` | `python -W error::DeprecationWarning` | `test_b` ERROR | 1 |
| code | `unittest.main(warnings="error")` | `test_b` ERROR | 1 |
| failfast | `-f` | `test_a` fails, and the other two get no outcome | 1 |

With no `-W`, `unittest.main()` sets the filter to `'default'`, ["print them anyway unless other warnings settings are specified"](https://github.com/python/cpython/blob/v3.13.13/Lib/unittest/main.py#L87-L91) [source].

### 2.3 cargo-nextest 0.9.149

`sleeps_three_seconds` sleeps 3 s. `fails_on_its_first_attempt` fails when `NEXTEST_ATTEMPT` is 1. `ignored_and_fails` carries `#[ignore]` and fails.

| case | what is set | outcome | exit |
|---|---|---|---|
| default | nothing | the sleeper PASS; the other FAIL | 100 |
| timeout-config | `slow-timeout = { period = "1s", terminate-after = 2 }` | TIMEOUT after 2 s | 100 |
| timeout-override | the same, and an override with `period = "10s"` for `test(sleeps)` | PASS: the per-test value wins | 0 |
| retries-cli | `--retries 1` | FLAKY 2/2, counted as passed | 0 |
| retries-env | `NEXTEST_RETRIES=1` | FLAKY, passed | 0 |
| retries-config | `retries = 1` in `.config/nextest.toml` | FLAKY, passed | 0 |
| retries-config-cli-zero | `retries = 1` in the file, `--retries 0` | FAIL: the command line wins | 100 |
| flaky-result-fail | `--retries 1 --flaky-result fail` | failed | 100 |
| run-ignored | `--run-ignored all` | the ignored test runs and FAILs | 100 |
| key-misspelled | `retires = 1` in the file | `warning: … ignoring unknown configuration key: profile.default.retires`, then FAIL | 100 |

The documentation says that ["passing in a command-line option overrides the respective environment variable"](https://github.com/nextest-rs/nextest/blob/cargo-nextest-0.9.149/site/src/docs/configuration/env-vars.md#L12) [source], and that for an unknown key ["nextest itself will only print out a warning"](https://github.com/nextest-rs/nextest/blob/cargo-nextest-0.9.149/nextest-runner/repo-config-reference.md#L29) [source].

### 2.4 `go test`, go 1.26.10

`TestSleepsTwoSeconds` sleeps 2 s. `TestLongUnlessShort` skips under `-short` and fails without it. `TestVetFinding` holds a `Sprintf` that vet reports.

| case | what is set | outcome | exit |
|---|---|---|---|
| vet-default | nothing | `[build failed]`: vet's finding stops every test of the package | 1 |
| vet-off | `-vet=off` | ok | 0 |
| timeout-cli | `-timeout=1s` | `panic: test timed out after 1s` | 1 |
| timeout-goflags | `GOFLAGS=-timeout=1s` | panic | 1 |
| timeout-goflags-cli-wins | `GOFLAGS=-timeout=1s`, `-timeout=10s` | ok: the command line wins | 0 |
| short-off | nothing | `--- FAIL: TestLongUnlessShort` | 1 |
| short-on | `-short` | ok: the test skips itself | 0 |
| goenv-file | `go env -w GOFLAGS=-timeout=1s`, then a run with no flag | panic | 1 |

The case goenv-file is a file of the user that changes a test's outcome. `go help environment` says that `go env -w` records a default ["in a Go environment configuration file stored in the per-user configuration directory"](https://github.com/golang/go/blob/go1.26.10/src/cmd/go/alldocs.go#L2374-L2377) [source]. `go help test` also says that a cached result ["will be cached and reused regardless of -timeout setting"](https://github.com/golang/go/blob/go1.26.10/src/cmd/go/alldocs.go#L1931-L1932) [run, from the help, and source].

### 2.5 Vitest 5.1.1

"waits one second" waits 1 s. "marked five" waits 1 s and sets `{ timeout: 5000 }`. `only.test.mjs` holds one `test.only`.

| case | what is set | outcome | exit |
|---|---|---|---|
| timeout-default | nothing, so 5000 ms | ✓ | 0 |
| timeout-cli | `--testTimeout=500` | × `TestTimeoutError: Test timed out in 500ms.` | 1 |
| timeout-config | `test: { testTimeout: 500 }` in `vitest.config.mjs` | × | 1 |
| timeout-code-vs-cli | `{ timeout: 5000 }` in code, `--testTimeout=500` | ✓: code wins | 0 |
| retry-none | nothing | × | 1 |
| retry-cli | `--retry=1` | ✓ `(retry x1)` | 0 |
| only-local | no `CI` | ✓ | 0 |
| only-ci | `CI=true` | `Error: [Vitest] Unexpected .only modifier.` | 1 |
| only-ci-allow | `CI=true`, `--allowOnly` | ✓ | 0 |
| key-misspelled | `test: { testTmeout: 500 }` | ✓, and no line names the key | 0 |

The help gives `--allowOnly` the default `!process.env.CI` [run]. The same default drives `--open` and `--browser.ui`, and `--browser.headless` takes `process.env.CI` ([`cli-config.ts`](https://github.com/vitest-dev/vitest/blob/v5.1.1/packages/vitest/src/node/cli/cli-config.ts#L513-L514)) [source].

### 2.6 Jest 30.5.2

| case | what is set | outcome | exit |
|---|---|---|---|
| timeout-default | nothing, so [5000 ms](https://github.com/jestjs/jest/blob/v30.5.2/docs/Configuration.md#L2459-L2461) | passed | 0 |
| timeout-cli | `--testTimeout=500` | `Exceeded timeout of 500 ms for a test.` | 1 |
| timeout-config | `testTimeout: 500` in `jest.config.js` | failed | 1 |
| timeout-code-vs-cli | `test(…, fn, 5000)`, `--testTimeout=500` | passed: code wins | 0 |
| snapshot-local | a new `toMatchSnapshot()` | passed, `1 written` | 0 |
| snapshot-ci-env | the same, `CI=true` | failed, `New snapshot was not written.` | 1 |
| snapshot-ci-flag | the same, `--ci` | failed | 1 |
| key-misspelled | `testTmeout: 500` | passed, with `Unknown option "testTmeout" with value 500 was found. Did you mean "testTimeout"?` | 0 |

The help of `--ci` says it ["is on by default in most popular CI environments"](https://github.com/jestjs/jest/blob/v30.5.2/packages/jest-cli/src/args.ts#L145-L149) [run and source]. Jest's documentation also states a case where an outcome can depend on the worker. The option `globalsCleanup` ["is set once per worker process, by the first test environment that worker creates. A per-file `@jest-environment-options` docblock therefore only takes effect if that file is the first one the worker runs"](https://github.com/jestjs/jest/blob/v30.5.2/docs/Configuration.md#L2151) [source].

### 2.7 A misspelled key, in each runner

| runner | what the tester sees | the run |
|---|---|---|
| pytest | a `PytestConfigWarning` in the summary; an error with `--strict-config` | goes on; exit 4 under `--strict-config` |
| unittest | has no file | — |
| cargo-nextest | `warning: … ignoring unknown configuration key` | goes on |
| `go test` | has no file of its own; `go.mod` is not a file of test dials | — |
| Vitest | nothing | goes on |
| Jest | `Unknown option … Did you mean "testTimeout"?` | goes on |

All [run] except the two blank rows. No runner stops by default.

## 3. Dials that the rule of configuration refuses

[The row of configuration](https://github.com/kalonji-tools/purba/blob/27e8f559/docs/decisions/a-concern-is-what-no-part-can-own.md#L42) says that a value "states a fact about the project or tightens a check, and never turns a check off". By that rule, purba refuses each dial of class L. It also refuses the loosening end of each ↕ dial. The list below holds each one. The appendix gives the note of each.

| runner | L: turns a check off or loosens it | ↕: tightens, and loosens at the other end |
|---|---|---|
| pytest | `--continue-on-collection-errors`, `--doctest-ignore-import-errors`, `-p no:<name>` (↕), `doctest_optionflags` (↕), `@pytest.mark.skip`, `@pytest.mark.skipif`, `@pytest.mark.xfail` (↕) | `-W` / `filterwarnings` / `@pytest.mark.filterwarnings`, `empty_parameter_set_mark` |
| unittest | `@skip` and its kin, `@expectedFailure` | `warnings=` / `-W` / `PYTHONWARNINGS` |
| cargo-nextest | `--retries`, `--debugger`, `--tracer`, `--ignore-rust-version`, `--override-version-check`, `#[ignore]` | `--flaky-result`, `--no-tests`, `global-timeout`, `slow-timeout`, `leak-timeout`, `bench.slow-timeout`, `bench.global-timeout`, and the two timeouts of a setup script |
| `go test` | `-vet` (↕), `t.Skip` | `-timeout` |
| Vitest | `--update`, `--coverage.thresholds.autoUpdate`, `--coverage.ignoreClassMethods`, `--browser.trackUnhandledErrors` (↕), `--passWithNoTests`, `--allowOnly`, `--dangerouslyIgnoreUnhandledErrors`, `--retry`, `--retry.condition`, `--typecheck.ignoreSourceErrors`, `onUnhandledError` (↕), the test options `retry`, `skip` and `fails` | `--testTimeout` and the test option `timeout`, `--hookTimeout`, `--teardownTimeout`, `--browser.connectTimeout`, `--expect.poll.timeout`, the four coverage thresholds, `--experimental.importDurations.thresholds.danger` |
| Jest | `--coveragePathIgnorePatterns`, `--forceExit`, `--passWithNoTests`, `--updateSnapshot`, `jest.retryTimes`, `test.skip`, `test.failing` | `--testTimeout` / `jest.setTimeout` / the third argument, `--coverageThreshold` |

Four kinds recur across runners.

- **A retry.** cargo-nextest, Vitest and Jest each let a test that failed pass on a second attempt. cargo-nextest shows it as FLAKY, and counts it as passed unless `flaky-result = "fail"`. Vitest shows `(retry x1)` and counts it as passed [run].
- **A skip or an expected failure in code.** Each runner has one. purba's record says that ["a `skip` or an `xfail` with a reason is an outcome, and not a switch"](https://github.com/kalonji-tools/purba/blob/27e8f559/docs/decisions/purba-is-always-strict.md#L44). So purba already admits this kind, as an outcome that the report shows.
- **Removal of a source of checks.** Examples are `-p no:<name>` in pytest and `-vet=off` in `go test`. In pytest, the key that the plugin owned then reads as an unknown key, and only a warning shows [run].
- **A timeout or a threshold of 0, or a high one.** Each ↕ dial is a number or a choice. The rule can accept one of its values and refuse another.

Some B dials change outcomes too, and the rule does not name them: order, workers, isolation, and mocks reset between tests. The rule judges checks. It does not judge whether a dial changes an outcome.

## 4. Which level wins, where a probe shows it

This file does not study the order of levels; #1212 does. These are only the cases that a probe above shows [run].

| runner | dial | order observed, highest first |
|---|---|---|
| pytest | warning filters | the command line (`-W` or `-o`), then the file. A marker beat the file; its filters apply after both [source] |
| pytest-timeout | timeout | code (marker), then the command line |
| cargo-nextest | retries | the command line, then the file |
| cargo-nextest | slow-timeout | a per-test override, then the profile |
| `go test` | timeout | the command line, then `GOFLAGS` |
| Vitest | testTimeout | code (test option), then the command line |
| Jest | testTimeout | code (third argument), then the command line |

In each probe where code set the value, code won over the level it was probed against. In each probe without code, the command line won over the environment and the file.

## 5. pytest-timeout, for comparison

pytest has no timeout of its own. The plugin pytest-timeout 2.4.0 adds one, and section 2.1 probes it. Its dials are not counted above. `pytest --help` with the plugin prints these [run]:

| dial | levels | class |
|---|---|---|
| `--timeout`, `timeout`, `PYTEST_TIMEOUT`, `@pytest.mark.timeout` | C P E K | T ↕: "Default is 0 which means no timeout" |
| `--timeout-method`, `timeout_method` | C P | B |
| `timeout_func_only` | P | L: the time of the fixtures no longer counts |
| `--timeout-disable-debugger-detection`, `timeout_disable_debugger_detection` | C P | T: the timeout also ends a test that sits in a debugger |
| `--session-timeout`, `session_timeout` | C P | T ↕ |

The help does not name `PYTEST_TIMEOUT`. The plugin [reads it](https://github.com/pytest-dev/pytest-timeout/blob/2.4.0/pytest_timeout.py#L360) [source], and the probe timeout-env shows it [run].

## 6. The dials that purba's tree and issues ask for

I searched `LANDSCAPE.md`, `CONTEXT.md`, the 134 records in `docs/decisions/`, and `.config/needs.toml` at `27e8f559`, with `grep -n` [run]. I filtered the 724 issues of `gh issue list --state all --limit 1300` locally, and read the bodies and comments that the filter found. Each link below is at `27e8f559`, or is an issue. The class uses this file's four classes, and is a judgement where no record states it.

### 6.1 Dials that purba asks for

| dial | what it changes | where it is set | status | link | class | outcome |
|---|---|---|---|---|---|---|
| the cap on suppressed verdicts | the most violations that a run may suppress; 0 refuses every suppression | a configuration key and a flag; the lower value applies | decided, not built | [`a-suppression-is-an-outcome-and-not-a-switch.md#L61`](https://github.com/kalonji-tools/purba/blob/27e8f559/docs/decisions/a-suppression-is-an-outcome-and-not-a-switch.md#L61), [#L89](https://github.com/kalonji-tools/purba/blob/27e8f559/docs/decisions/a-suppression-is-an-outcome-and-not-a-switch.md#L89) "the cap waits for one" | T ↕ | run |
| the timeout of a test | ends a test that hangs | not decided; #1213 sketches the project, code and the command line | asked for; its part is dispatch | [`.config/needs.toml#L12`](https://github.com/kalonji-tools/purba/blob/27e8f559/.config/needs.toml#L12), [`purba-divides-by-responsibility-and-not-by-language.md#L39`](https://github.com/kalonji-tools/purba/blob/27e8f559/docs/decisions/purba-divides-by-responsibility-and-not-by-language.md#L39), [#1213](https://github.com/kalonji-tools/purba/issues/1213) | T ↕ | test |
| a configuration file that holds keys | where the project states its values | a file; the words are `toml` and `pyproject` | asked for | [`.config/needs.toml#L7`](https://github.com/kalonji-tools/purba/blob/27e8f559/.config/needs.toml#L7) | — | — |
| the number of workers | runs in parallel | not decided | implied | [`LANDSCAPE.md#L168`](https://github.com/kalonji-tools/purba/blob/27e8f559/LANDSCAPE.md#L168), and [#1211](https://github.com/kalonji-tools/purba/issues/1211) names it as an example | B | must not: [`LANDSCAPE.md#L18`](https://github.com/kalonji-tools/purba/blob/27e8f559/LANDSCAPE.md#L18) "no outcome depends on the worker" |
| outcomes as data | the form of the output for an agent or an integrator | not decided | asked for, deferred to reporting | [`LANDSCAPE.md#L64`](https://github.com/kalonji-tools/purba/blob/27e8f559/LANDSCAPE.md#L64), [`.config/needs.toml#L15`](https://github.com/kalonji-tools/purba/blob/27e8f559/.config/needs.toml#L15) | B | — |
| JUnit XML | a report file | not decided | asked for | [`purba-divides-by-responsibility-and-not-by-language.md#L55`](https://github.com/kalonji-tools/purba/blob/27e8f559/docs/decisions/purba-divides-by-responsibility-and-not-by-language.md#L55), [`.config/needs.toml#L16`](https://github.com/kalonji-tools/purba/blob/27e8f559/.config/needs.toml#L16) | B | — |
| picking a run's files by platform | leaves out a module that cannot load on this platform | configuration and selection | asked for | [`a-test-carries-only-purbas-own-decorators.md#L74`](https://github.com/kalonji-tools/purba/blob/27e8f559/docs/decisions/a-test-carries-only-purbas-own-decorators.md#L74), [#L60](https://github.com/kalonji-tools/purba/blob/27e8f559/docs/decisions/a-test-carries-only-purbas-own-decorators.md#L60) | F | test: today such a module is a project error |
| the import roots | what goes on `sys.path` for a test module | not decided | an open question | [#637](https://github.com/kalonji-tools/purba/issues/637), risk [`LANDSCAPE.md#L170`](https://github.com/kalonji-tools/purba/blob/27e8f559/LANDSCAPE.md#L170) | F | test |
| the query | which tests a run selects | not decided | asked for, deferred to selection | [`LANDSCAPE.md#L58`](https://github.com/kalonji-tools/purba/blob/27e8f559/LANDSCAPE.md#L58), [`.config/needs.toml#L9`](https://github.com/kalonji-tools/purba/blob/27e8f559/.config/needs.toml#L9) | B | run |
| `@purba.skip_if(condition, "reason")` | switches a test off on a condition | code | decided in an issue, carried by #1063 | [#1019 comment](https://github.com/kalonji-tools/purba/issues/1019#issuecomment-6917084829) | L, and an outcome by [`purba-is-always-strict.md#L44`](https://github.com/kalonji-tools/purba/blob/27e8f559/docs/decisions/purba-is-always-strict.md#L44) | test |
| `@purba.known_failure("reason")` | marks a known gap; a case that passes fails | code | as `skip_if` | [#1019 comment](https://github.com/kalonji-tools/purba/issues/1019#issuecomment-6917084829) | L, as an outcome | test |
| "this directory is test code" | which rules apply where | the project, once | named as the honest form if one is needed | [#32 comment](https://github.com/kalonji-tools/purba/issues/32#issuecomment-5600829294) | F | test |

### 6.2 Dials and triggers that exist on `main`

| dial | what it changes | where it is set | link | class | outcome |
|---|---|---|---|---|---|
| `purba <path>` | the one module that runs | the command line, one argument, and no flag | [`a-tester-runs-purba-on-one-module-and-reads-one-line-for-each-test.md#L41`](https://github.com/kalonji-tools/purba/blob/27e8f559/docs/decisions/a-tester-runs-purba-on-one-module-and-reads-one-line-for-each-test.md#L41) | B | run |
| `# purba: suppress[<check>] <reason>`, `# purba: suppress-module[<check>] <reason>` | a violation of the practice tier becomes a warning, and the test runs | code, a comment | [`a-suppression-is-an-outcome-and-not-a-switch.md#L38`](https://github.com/kalonji-tools/purba/blob/27e8f559/docs/decisions/a-suppression-is-an-outcome-and-not-a-switch.md#L38) | L in this file's terms; the record calls it an outcome and not a switch | test |
| an interrupt, and a second interrupt | stops the run; a second one skips the cleanup left | a signal | [`an-interrupt-stops-the-run-and-a-second-one-exits-at-once.md#L57`](https://github.com/kalonji-tools/purba/blob/27e8f559/docs/decisions/an-interrupt-stops-the-run-and-a-second-one-exits-at-once.md#L57) | B | test: `interrupted` |
| `python -O`, `PYTHONOPTIMIZE` | CPython's; purba keeps the test module's asserts | the interpreter, the environment | [`a-test-module-keeps-its-asserts-under-o.md#L29`](https://github.com/kalonji-tools/purba/blob/27e8f559/docs/decisions/a-test-module-keeps-its-asserts-under-o.md#L29) | L, which purba neutralises for the test module | test, in a helper module |
| `PYTHONWARNINGS=error` | CPython's; a warning raises | the environment | [`a-test-returns-nothing-and-the-first-part-that-sees-a-breach-refuses-it.md#L119`](https://github.com/kalonji-tools/purba/blob/27e8f559/docs/decisions/a-test-returns-nothing-and-the-first-part-that-sees-a-breach-refuses-it.md#L119): execution refuses a case that changes the warning filters | T | test |
| `PURBA_FAULT` | makes a part panic | the environment, in a test build only | [`a-test-causes-an-internal-error-through-a-switch-that-only-a-test-build-carries.md#L31`](https://github.com/kalonji-tools/purba/blob/27e8f559/docs/decisions/a-test-causes-an-internal-error-through-a-switch-that-only-a-test-build-carries.md#L31) | B | test, internal error |

The last row of 6.2 has a consequence for code. purba refuses a case that changes the warning filters, so purba has no counterpart of pytest's `@pytest.mark.filterwarnings`.

### 6.3 Dials that purba's records refuse

| dial | link | the runners that offer it |
|---|---|---|
| a mode of strictness, such as oxitest's `abort`, `enforce` and `off` | [`purba-is-always-strict.md#L37`](https://github.com/kalonji-tools/purba/blob/27e8f559/docs/decisions/purba-is-always-strict.md#L37) | none of the six has one mode for all checks (judgement, from the lists in the appendix); pytest's `--strict` is the reverse, one switch that turns four checks on |
| a silent opt-out for one test or one check | [`purba-is-always-strict.md#L28`](https://github.com/kalonji-tools/purba/blob/27e8f559/docs/decisions/purba-is-always-strict.md#L28) | `-p no:<name>`, `-vet=off`, `--dangerouslyIgnoreUnhandledErrors` |
| a suppression by a path pattern in a file of configuration | [`a-suppression-is-an-outcome-and-not-a-switch.md#L38`](https://github.com/kalonji-tools/purba/blob/27e8f559/docs/decisions/a-suppression-is-an-outcome-and-not-a-switch.md#L38) | `--coverage.ignoreClassMethods`, `coveragePathIgnorePatterns` |
| a plugin | [`purbas-public-surface-starts-empty-and-has-no-plugin-api.md#L39`](https://github.com/kalonji-tools/purba/blob/27e8f559/docs/decisions/purbas-public-surface-starts-empty-and-has-no-plugin-api.md#L39) | pytest `-p`, `PYTEST_PLUGINS`, `required_plugins`; Jest `--runner`, `watchPlugins`; Vitest `runner` |
| `conftest.py` | [`purba-runs-inside-the-testers-interpreter.md#L45`](https://github.com/kalonji-tools/purba/blob/27e8f559/docs/decisions/purba-runs-inside-the-testers-interpreter.md#L45) | pytest `--confcutdir`, `--noconftest` |
| a flag for how much of a value shows | [`a-failed-expectation-shows-its-expected-value-and-its-reading.md#L12`](https://github.com/kalonji-tools/purba/blob/27e8f559/docs/decisions/a-failed-expectation-shows-its-expected-value-and-its-reading.md#L12) | pytest `truncation_limit_lines`, `truncation_limit_chars`, `verbosity_assertions`; Vitest `--diff.truncateThreshold`, `--diff.maxDepth`; Jest `--expand`; unittest `maxDiff` |
| a mark for a module; a decorator on a class | [`a-test-carries-only-purbas-own-decorators.md#L49`](https://github.com/kalonji-tools/purba/blob/27e8f559/docs/decisions/a-test-carries-only-purbas-own-decorators.md#L49) | pytest's `pytestmark`; unittest's decorators on a class |

### 6.4 The rules that frame a dial in purba

- [`LANDSCAPE.md#L156`](https://github.com/kalonji-tools/purba/blob/27e8f559/LANDSCAPE.md#L156): configuration "States facts about the project, or tightens a check".
- [`purba-is-always-strict.md#L42-L43`](https://github.com/kalonji-tools/purba/blob/27e8f559/docs/decisions/purba-is-always-strict.md#L42-L43): "A value that names a fact about the project is not a switch." "Selection is not a switch."
- [`a-suppression-is-an-outcome-and-not-a-switch.md#L72`](https://github.com/kalonji-tools/purba/blob/27e8f559/docs/decisions/a-suppression-is-an-outcome-and-not-a-switch.md#L72): the rule "admits a value that states no fact about the project", and "The cap moves that line".
- [`a-concern-is-what-no-part-can-own.md#L70`](https://github.com/kalonji-tools/purba/blob/27e8f559/docs/decisions/a-concern-is-what-no-part-can-own.md#L70): "The line between a fact about the project and a switch is thin."
- [`purbas-public-surface-starts-empty-and-has-no-plugin-api.md#L37`](https://github.com/kalonji-tools/purba/blob/27e8f559/docs/decisions/purbas-public-surface-starts-empty-and-has-no-plugin-api.md#L37): "A flag or an exit code joins with a change that purba needs to work as a command."
- [`CONTEXT.md#L237`](https://github.com/kalonji-tools/purba/blob/27e8f559/CONTEXT.md#L237): the public surface holds "the command's flags and exit code, the data purba writes, and the format of its configuration".

### 6.5 oxitest, for comparison

purba's predecessor had 31 keys in `[tool.oxitest]`, plus 3 nested, and 39 flags. An unknown key was an error. Only 4 keys had any use in the two corpora it measured. [#4, part 3](https://github.com/kalonji-tools/purba/issues/4#issuecomment-5560546415) holds the list. I did not count it again. Its part for configuration held 2,395 Rust production lines, in 124 commits, 11 of them `fix` ([#464](https://github.com/kalonji-tools/purba/issues/464#issuecomment-6100239730)).

## What #1213 can take from this

This is input for the prototype. It decides nothing. Each line names a dial, its class, and one reason to run it.

- **The cap on suppressed verdicts, T ↕.** A merged record decides it, at two levels with "the lower value applies". It is the one case where purba already has a rule for a two-way number.
- **The timeout of a test, T ↕.** `.config/needs.toml` states the need. Three runners show that code beats the command line, and `go test` shows that a file of the user can set it.
- **The number of workers, B.** It touches no check, so it can be a dial that every level may set. It is the contrast for criterion 3. purba's risk says that it must change no outcome.
- **The import roots, F.** #637 asks for it. It is a fact about the project, and it changes outcomes.
- **Picking files by platform, F.** A record asks for it. cargo-nextest's override of `default-filter` for a platform is one form that exists.
- **A retry, L.** It is the dial that turns a check off that a tester asks for most often: three runners have it. The prototype can show it against [purba is always strict](https://github.com/kalonji-tools/purba/blob/27e8f559/docs/decisions/purba-is-always-strict.md#decision-outcome).
- **`CI` as an environment variable that changes a default, a level the criteria do not name.** Vitest's `.only` and Jest's snapshots show it changing outcomes.
- **A misspelled key.** The runners give four answers: a warning (pytest, cargo-nextest), a warning with a suggestion (Jest), an error under a strict flag (pytest), and silence (Vitest).

## Appendix: every dial, by runner

Columns as in "How to read this file". `research/configuration-dials/count.sh appendix` prints these tables from the rows.

### pytest

| dial | group | levels | class | outcome | note |
|---|---|---|---|---|---|
| `file_or_dir (positional)` | sel | C | B | run |  |
| `-k EXPRESSION` | sel | C | B | run |  |
| `-m MARKEXPR` | sel | C | B | run |  |
| `--markers` | dbg | C | B |  |  |
| `-x, --exitfirst` | stp | C | B | run | later tests get no outcome |
| `--maxfail` | stp | C | B | run | later tests get no outcome |
| `--strict-config, strict_config` | cfg | C P | T | run | probe key-misspelled-strict: exit 4 |
| `--strict-markers, strict_markers` | chk | C P | T | test | probe marker-strict: collection error |
| `--strict, strict` | chk | C P | T | test | turns on strict_config, strict_markers, strict_xfail, strict_parametrization_ids |
| `--fixtures, --funcargs` | dbg | C | B |  |  |
| `--fixtures-per-test` | dbg | C | B |  |  |
| `--pdb` | dbg | C | B |  |  |
| `--pdbcls` | dbg | C | B |  |  |
| `--trace` | dbg | C | B |  |  |
| `--capture, -s` | cap | C | B | test | judgement: a test that reads sys.stdout or its isatty() sees a different stream |
| `--runxfail` | chk | C | T | test | probe xfail-runxfail: an xfail test that fails becomes FAILED |
| `--lf, --last-failed` | sel | C | B | run |  |
| `--ff, --failed-first` | ord | C | B | test | judgement: a test that depends on order can change outcome |
| `--nf, --new-first` | ord | C | B | test | judgement: as --ff |
| `--cache-show` | dbg | C | B |  |  |
| `--cache-clear` | cch | C | B |  |  |
| `--lfnf, --last-failed-no-failures` | sel | C | B | run |  |
| `--sw, --stepwise` | stp | C | B | run |  |
| `--sw-skip, --stepwise-skip` | stp | C | B | run |  |
| `--sw-reset, --stepwise-reset` | stp | C | B | run |  |
| `--durations` | out | C | B |  |  |
| `--durations-min` | out | C | B |  |  |
| `-v, -q, --verbosity` | out | C | B |  |  |
| `--no-header` | out | C | B |  |  |
| `--no-summary` | out | C | B |  |  |
| `--no-fold-skipped` | out | C | B |  |  |
| `--force-short-summary` | out | C | B |  |  |
| `-r, --report-chars` | out | C | B |  |  |
| `--disable-warnings` | out | C | B |  | hides the summary of warnings; no outcome changes |
| `-l, --showlocals, --no-showlocals` | out | C | B |  |  |
| `--tb` | out | C | B |  |  |
| `--xfail-tb` | out | C | B |  |  |
| `--show-capture` | out | C | B |  |  |
| `--full-trace` | out | C | B |  |  |
| `--color` | out | C | B |  |  |
| `--code-highlight` | out | C | B |  |  |
| `--pastebin` | out | C | B |  |  |
| `--junitxml, --junit-xml` | out | C | B |  |  |
| `--junitprefix, --junit-prefix` | out | C | B |  |  |
| `-W, filterwarnings, @pytest.mark.filterwarnings` | chk | C P K | T ↕ | test | error tightens, ignore loosens; probes warn-*: the marker and -W both beat the file |
| `--max-warnings, max_warnings` | chk | C P | T | run | probe warn-max-warnings: test passes, exit 6 |
| `--collect-only, --co` | dbg | C | B |  |  |
| `--pyargs` | sel | C | B | run |  |
| `--ignore` | sel | C | B | run |  |
| `--ignore-glob` | sel | C | B | run |  |
| `--deselect` | sel | C | B | run |  |
| `--confcutdir` | imp | C | F | test | decides which conftest.py files load |
| `--noconftest` | fix | C | B | test | fixtures and hooks of conftest.py do not load |
| `--keep-duplicates` | sel | C | B | run |  |
| `--collect-in-virtualenv` | sel | C | B | run |  |
| `--continue-on-collection-errors` | stp | C | L | run | a collection error no longer stops the run |
| `--import-mode` | imp | C | F | test | decides what goes on sys.path |
| `--doctest-modules` | sel | C | B | run | adds doctests to the run |
| `--doctest-report` | out | C | B |  |  |
| `--doctest-glob` | imp | C | F | run |  |
| `--doctest-ignore-import-errors` | chk | C | L | run |  |
| `--doctest-continue-on-failure` | out | C | B |  | reports more failures of one doctest; its outcome stays failed |
| `-c, --config-file` | cfg | C | B | test | indirect: picks the file that sets every other dial |
| `--rootdir` | imp | C | F | test | moves node ids, conftest lookup and the cache |
| `--basetemp` | cch | C | B |  |  |
| `-V, --version` | dbg | C | B |  |  |
| `-h, --help` | dbg | C | B |  |  |
| `-p name, -p no:name` | plg | C | L ↕ | test | probes timeout-plugin-off, warn-ini-error-plugin-off: no:name removes a check |
| `--disable-plugin-autoload, PYTEST_DISABLE_PLUGIN_AUTOLOAD` | plg | C E | B | test | judgement: a plugin that is not loaded runs none of its checks |
| `--trace-config` | dbg | C | B |  |  |
| `--debug` | dbg | C | B |  |  |
| `-o, --override-ini` | cfg | C | B |  | a bridge: sets any key of the file from the command line |
| `--assert` | ast | C | B |  | judgement: plain changes the message, not the raise |
| `--setup-only` | dbg | C | B |  |  |
| `--setup-show` | out | C | B |  |  |
| `--setup-plan` | dbg | C | B |  |  |
| `--log-level, log_level` | cap | C P | B | test | judgement: a test that reads caplog sees other records |
| `--log-format, log_format` | cap | C P | B |  |  |
| `--log-date-format, log_date_format` | cap | C P | B |  |  |
| `--log-cli-level, log_cli_level` | cap | C P | B |  |  |
| `--log-cli-format, log_cli_format` | cap | C P | B |  |  |
| `--log-cli-date-format, log_cli_date_format` | cap | C P | B |  |  |
| `--log-file, log_file` | cap | C P | B |  |  |
| `--log-file-mode, log_file_mode` | cap | C P | B |  |  |
| `--log-file-level, log_file_level` | cap | C P | B |  |  |
| `--log-file-format, log_file_format` | cap | C P | B |  |  |
| `--log-file-date-format, log_file_date_format` | cap | C P | B |  |  |
| `--log-auto-indent, log_auto_indent` | cap | C P | B |  |  |
| `--log-disable` | cap | C | B | test | judgement: a test that reads caplog sees no record of that logger |
| `markers` | chk | P | F | test | with strict_markers, an unregistered marker is an error |
| `empty_parameter_set_mark` | chk | P | T ↕ | test | fail_at_collect tightens; skip and xfail give other outcomes |
| `norecursedirs` | imp | P | F | run |  |
| `testpaths` | imp | P | F | run |  |
| `collect_imported_tests` | imp | P | F | run |  |
| `consider_namespace_packages` | imp | P | F | test |  |
| `usefixtures, @pytest.mark.usefixtures` | fix | P K | B | test | runs fixtures that a test does not name |
| `python_files` | imp | P | F | run |  |
| `python_classes` | imp | P | F | run |  |
| `python_functions` | imp | P | F | run |  |
| `disable_test_id_escaping_and_forfeit_all_rights_to_community_support` | out | P | B |  |  |
| `strict_parametrization_ids` | chk | P | T | test | --strict turns it on |
| `parametrize_long_str_id_strategy` | out | P | B | test | disallow raises an error |
| `console_output_style` | out | P | B |  |  |
| `verbosity_test_cases` | out | P | B |  |  |
| `strict_xfail, @pytest.mark.xfail(strict=)` | chk | P K | T | test | probe xfail-strict: XPASS becomes FAILED |
| `tmp_path_retention_count` | cch | P | B |  |  |
| `tmp_path_retention_policy` | cch | P | B |  |  |
| `enable_assertion_pass_hook` | plg | P | B |  |  |
| `truncation_limit_lines` | ast | P | B |  |  |
| `truncation_limit_chars` | ast | P | B |  |  |
| `assertion_text_diff_style` | ast | P | B |  |  |
| `verbosity_assertions` | ast | P | B |  |  |
| `junit_suite_name` | out | P | B |  |  |
| `junit_logging` | out | P | B |  |  |
| `junit_log_passing_tests` | out | P | B |  |  |
| `junit_duration_report` | out | P | B |  |  |
| `junit_family` | out | P | B |  |  |
| `doctest_optionflags` | chk | P | L ↕ | test | flags such as ELLIPSIS loosen a doctest's comparison |
| `doctest_encoding` | imp | P | F | test |  |
| `cache_dir` | cch | P | B |  |  |
| `log_cli` | cap | P | B |  |  |
| `faulthandler_timeout` | tmo | P | B |  | dumps the stacks; the test goes on |
| `faulthandler_exit_on_timeout` | tmo | P | T | test | exits the test process |
| `verbosity_subtests` | out | P | B |  |  |
| `addopts` | cfg | P | B |  | a bridge: sets any flag from the project file |
| `minversion` | cfg | P | F | run |  |
| `pythonpath` | imp | P | F | test | decides what goes on sys.path |
| `required_plugins` | plg | P | F | run |  |
| `CI, BUILD_NUMBER` | out | E | B |  | the summary is not truncated |
| `PYTEST_ADDOPTS` | cfg | E | B |  | a bridge: sets any flag from the environment; probe warn-env-addopts |
| `PYTEST_PLUGINS` | plg | E | B | test | loads plugins, which can add checks |
| `PYTEST_DEBUG` | dbg | E | B |  |  |
| `PYTEST_DEBUG_TEMPROOT` | cch | E | B |  |  |
| `PYTEST_THEME` | out | E | B |  |  |
| `PYTEST_THEME_MODE` | out | E | B |  |  |
| `@pytest.mark.skip` | sel | K | L | test | purba's record calls a skip with a reason an outcome |
| `@pytest.mark.skipif` | sel | K | L | test | as skip, on a condition |
| `@pytest.mark.xfail` | chk | K | L ↕ | test | a failure becomes XFAIL; strict=True tightens |
| `@pytest.mark.parametrize` | sel | K | B | run | makes one case per value |
| `@pytest.mark.tryfirst` | plg | K | B |  | deprecated |
| `@pytest.mark.trylast` | plg | K | B |  | deprecated |

### unittest

| dial | group | levels | class | outcome | note |
|---|---|---|---|---|---|
| `tests (positional), defaultTest=` | sel | C K | B | run |  |
| `-v, -q, verbosity=` | out | C K | B |  |  |
| `--locals, tb_locals=` | out | C K | B |  |  |
| `--durations, durations=` | out | C K | B |  |  |
| `-f, --failfast, failfast=` | stp | C K | B | run | probe: the second test gets no outcome |
| `-c, --catch, catchbreak=` | stp | C K | B | run | a first Ctrl-C ends the run and prints the results so far |
| `-b, --buffer, buffer=` | cap | C K | B |  |  |
| `-k, testNamePatterns` | sel | C K | B | run | probe: -k warns runs one test |
| `discover -s, --start-directory` | imp | C | F | run |  |
| `discover -p, --pattern` | imp | C | F | run |  |
| `discover -t, --top-level-directory` | imp | C | F | test | decides what goes on sys.path |
| `-h, --help` | dbg | C | B |  |  |
| `warnings=, python -W, PYTHONWARNINGS` | chk | C E K | T ↕ | test | with no -W, main() sets 'default'; probe: -W error and warnings='error' make the test an ERROR |
| `testRunner=, testLoader=, exit=, argv=, module=` | plg | K | B | test | code replaces the runner or the loader |
| `resultclass=` | plg | K | B | test | code replaces the result |
| `TestLoader.testMethodPrefix` | imp | K | F | run |  |
| `TestLoader.sortTestMethodsUsing` | ord | K | B | test | judgement: a test that depends on order can change outcome |
| `TestLoader.suiteClass` | plg | K | B |  |  |
| `TestCase.failureException` | ast | K | B | test | the exception that counts as a failure, not an error |
| `TestCase.longMessage` | ast | K | B |  |  |
| `TestCase.maxDiff` | ast | K | B |  |  |
| `@skip, @skipIf, @skipUnless, raise SkipTest` | sel | K | L | test |  |
| `@expectedFailure` | chk | K | L | test | probe: an expected failure that passes is an unexpected success, and the run fails |

### nextest

| dial | group | levels | class | outcome | note |
|---|---|---|---|---|---|
| `[FILTERS] (positional), --skip, --exact` | sel | C | B | run | --skip and --exact emulate libtest after -- |
| `--color, CARGO_TERM_COLOR` | out | C E | B |  |  |
| `--no-pager` | out | C | B |  |  |
| `-v, --verbose, NEXTEST_VERBOSE` | out | C E | B |  |  |
| `-h, --help` | dbg | C | B |  |  |
| `-p, --package` | sel | C | B | run |  |
| `--workspace` | sel | C | B | run |  |
| `--exclude` | sel | C | B | run |  |
| `--all` | sel | C | B | run | deprecated |
| `--lib` | sel | C | B | run |  |
| `--bin` | sel | C | B | run |  |
| `--bins` | sel | C | B | run |  |
| `--example` | sel | C | B | run |  |
| `--examples` | sel | C | B | run |  |
| `--test` | sel | C | B | run |  |
| `--tests` | sel | C | B | run |  |
| `--bench` | sel | C | B | run |  |
| `--benches` | sel | C | B | run |  |
| `--all-targets` | sel | C | B | run |  |
| `-F, --features` | bld | C | B | test | compiles other code |
| `--all-features` | bld | C | B | test | compiles other code |
| `--no-default-features` | bld | C | B | test | compiles other code |
| `--build-jobs` | bld | C | B |  |  |
| `-r, --release` | bld | C | B | test | judgement: the release profile drops debug assertions and overflow checks |
| `--cargo-profile` | bld | C | B | test | as --release |
| `--target, CARGO_BUILD_TARGET` | bld | C E | F | test |  |
| `--target-dir, CARGO_TARGET_DIR` | bld | C E | B |  |  |
| `--unit-graph` | bld | C | B |  |  |
| `--timings` | bld | C | B |  |  |
| `--manifest-path` | imp | C | F | run |  |
| `--frozen` | bld | C | T | run | refuses a stale lock file or cache |
| `--locked` | bld | C | T | run | refuses a stale lock file |
| `--offline` | bld | C | B | run |  |
| `--cargo-message-format` | out | C | B |  |  |
| `--cargo-quiet` | out | C | B |  |  |
| `--cargo-verbose` | out | C | B |  |  |
| `--ignore-rust-version` | bld | C | L | run | skips the check of rust-version |
| `--future-incompat-report` | out | C | B |  |  |
| `--config KEY=VALUE` | bld | C | B | test | a bridge to cargo's configuration |
| `-Z` | bld | C | B | test |  |
| `-R, --rerun` | sel | C | B | run |  |
| `--run-ignored, --ignored, --include-ignored` | sel | C | T | test | probe run-ignored: an ignored test runs and fails |
| `--partition` | sel | C | B | run |  |
| `--platform-filter` | sel | C | B | run | deprecated |
| `-E, --filterset` | sel | C | B | run |  |
| `--ignore-default-filter` | sel | C | B | run |  |
| `--no-run` | dbg | C | B |  |  |
| `-j, --test-threads, NEXTEST_TEST_THREADS, test-threads` | par | C E P | B |  |  |
| `--retries, NEXTEST_RETRIES, retries` | rep | C E P | L | test | probes retries-*: a test that failed once passes as flaky |
| `--flaky-result, NEXTEST_FLAKY_RESULT, flaky-result` | rep | C E P | T ↕ | test | probe flaky-result-fail: fail tightens, pass is the default |
| `--fail-fast, --no-fail-fast, fail-fast` | stp | C P | B | run |  |
| `--max-fail, fail-fast.max-fail` | stp | C P | B | run |  |
| `--debugger` | dbg | C | L | test | disables timeouts and capture |
| `--tracer` | dbg | C | L | test | disables timeouts and capture |
| `--no-tests, NEXTEST_NO_TESTS` | stp | C E | T ↕ | run | fail tightens, pass and warn loosen |
| `--no-capture, --nocapture` | cap | C | B | test | also runs the tests serially |
| `--stress-count` | rep | C | B | test | more runs can expose a flaky failure |
| `--stress-duration` | rep | C | B | test | as --stress-count |
| `--failure-output, NEXTEST_FAILURE_OUTPUT, failure-output` | out | C E P | B |  |  |
| `--success-output, NEXTEST_SUCCESS_OUTPUT, success-output` | out | C E P | B |  |  |
| `--status-level, NEXTEST_STATUS_LEVEL, status-level` | out | C E P | B |  |  |
| `--final-status-level, NEXTEST_FINAL_STATUS_LEVEL, final-status-level` | out | C E P | B |  |  |
| `--no-output-indent, NEXTEST_NO_OUTPUT_INDENT, ui.output-indent` | out | C E U | B |  |  |
| `--show-progress, NEXTEST_SHOW_PROGRESS, ui.show-progress` | out | C E U | B |  |  |
| `--hide-progress-bar, NEXTEST_HIDE_PROGRESS_BAR` | out | C E | B |  | deprecated |
| `--no-input-handler, NEXTEST_NO_INPUT_HANDLER, ui.input-handler` | out | C E U | B |  |  |
| `--max-progress-running, NEXTEST_MAX_PROGRESS_RUNNING, ui.max-progress-running` | out | C E U | B |  |  |
| `--message-format, NEXTEST_MESSAGE_FORMAT` | out | C E | B |  |  |
| `--message-format-version, NEXTEST_MESSAGE_FORMAT_VERSION` | out | C E | B |  |  |
| `--archive-file` | bld | C | B |  |  |
| `--archive-format` | bld | C | B |  |  |
| `--extract-to` | bld | C | B |  |  |
| `--extract-overwrite` | bld | C | B |  |  |
| `--persist-extract-tempdir` | bld | C | B |  |  |
| `--cargo-metadata` | imp | C | F |  |  |
| `--workspace-remap` | imp | C | F | test | moves the paths a test sees |
| `--binaries-metadata` | imp | C | F |  |  |
| `--target-dir-remap` | imp | C | F |  |  |
| `--build-dir-remap` | imp | C | F |  |  |
| `--config-file` | cfg | C | B | test | indirect: picks the file that sets every other dial |
| `--user-config-file, NEXTEST_USER_CONFIG_FILE` | cfg | C E | B |  |  |
| `--tool-config-file` | cfg | C | B | test | indirect: inserts a file below the repository's |
| `--override-version-check` | cfg | C | L | run | skips the minimum version that the repository states |
| `-P, --profile, NEXTEST_PROFILE` | cfg | C E | B | test | indirect: picks a set of values |
| `nextest-version` | cfg | P | F | run |  |
| `experimental` | cfg | P | B |  |  |
| `store.dir` | cch | P | B |  |  |
| `profile.inherits` | cfg | P | B |  |  |
| `default-filter` | sel | P | B | run | an override can replace it per platform |
| `global-timeout` | tmo | P | T ↕ | test |  |
| `threads-required` | par | P | B |  |  |
| `run-extra-args` | fix | P | B | test | passes arguments to the test binary |
| `slow-timeout (period, terminate-after, grace-period, on-timeout)` | tmo | P | T ↕ | test | probes timeout-*: terminate-after ends the test; on-timeout = pass loosens |
| `leak-timeout` | tmo | P | T ↕ | test | result = fail tightens |
| `bench.slow-timeout` | tmo | P | T ↕ | test |  |
| `bench.global-timeout` | tmo | P | T ↕ | test |  |
| `test-group` | par | P | B |  |  |
| `junit.path` | out | P | B |  |  |
| `junit.report-name` | out | P | B |  |  |
| `junit.store-success-output` | out | P | B |  |  |
| `junit.store-failure-output` | out | P | B |  |  |
| `junit.report-skipped` | out | P | B |  |  |
| `junit.flaky-fail-status` | out | P | B |  |  |
| `archive.include` | bld | P | B |  |  |
| `overrides (filter, platform)` | cfg | P | B | test | a bridge: gives a test its own value of each setting it may override; probe timeout-override |
| `overrides.priority` | ord | P | B |  |  |
| `test-groups.<name>.max-threads` | par | P | B |  |  |
| `scripts.setup.<name>.command` | fix | P | B | test |  |
| `scripts.setup.<name>.slow-timeout` | tmo | P | T ↕ | test |  |
| `scripts.setup.<name>.leak-timeout` | tmo | P | T ↕ | test |  |
| `scripts.setup.<name>.capture-stdout` | cap | P | B |  |  |
| `scripts.setup.<name>.capture-stderr` | cap | P | B |  |  |
| `scripts.setup.<name>.junit.store-success-output` | out | P | B |  |  |
| `scripts.setup.<name>.junit.store-failure-output` | out | P | B |  |  |
| `scripts.wrapper.<name>.command` | fix | P | B | test |  |
| `scripts.wrapper.<name>.target-runner` | fix | P | B | test |  |
| `profile scripts: platform` | fix | P | B | test |  |
| `profile scripts: filter` | fix | P | B | test |  |
| `profile scripts: setup` | fix | P | B | test |  |
| `profile scripts: list-wrapper` | fix | P | B | test |  |
| `profile scripts: run-wrapper` | fix | P | B | test |  |
| `experimental.record` | cch | U | B |  |  |
| `record.enabled` | cch | U | B |  |  |
| `record.max-output-size` | cch | U | B |  |  |
| `record.max-records` | cch | U | B |  |  |
| `record.max-total-size` | cch | U | B |  |  |
| `record.max-age` | cch | U | B |  |  |
| `ui.pager` | out | U | B |  |  |
| `ui.paginate` | out | U | B |  |  |
| `ui.streampager.interface` | out | U | B |  |  |
| `ui.streampager.wrapping` | out | U | B |  |  |
| `ui.streampager.show-ruler` | out | U | B |  |  |
| `overrides.platform (user)` | cfg | U | B |  | gives a platform its own ui value |
| `CARGO` | bld | E | B |  |  |
| `CARGO_TARGET_<triple>_RUNNER` | fix | E | B | test | runs each test binary under a command |
| `CARGO_TERM_PROGRESS_TERM_INTEGRATION` | out | E | B |  |  |
| `MIRI_SYSROOT` | cfg | E | B | test | activates the profile default-miri |
| `#[ignore] (libtest)` | sel | K | L | test | the test is skipped unless --run-ignored |

### go

| dial | group | levels | class | outcome | note |
|---|---|---|---|---|---|
| `[packages] (positional)` | sel | C | B | run |  |
| `-artifacts` | out | C | B |  |  |
| `-bench` | sel | C | B | run | runs benchmarks |
| `-benchtime` | rep | C | B |  |  |
| `-count` | rep | C | B | test | more runs can expose a flaky failure; -count=1 also skips the cache |
| `-cover` | cov | C | B |  |  |
| `-covermode` | cov | C | B |  |  |
| `-coverpkg` | cov | C | B |  |  |
| `-cpu` | par | C | B | test | runs each test under each GOMAXPROCS |
| `-failfast` | stp | C | B | run |  |
| `-fullpath` | out | C | B |  |  |
| `-fuzz` | rep | C | B | test | fuzzing can find a failing input |
| `-fuzztime` | rep | C | B | test |  |
| `-fuzzminimizetime` | rep | C | B |  |  |
| `-json` | out | C | B |  |  |
| `-list` | dbg | C | B |  |  |
| `-outputdir` | out | C | B |  |  |
| `-parallel` | par | C | B |  |  |
| `-run` | sel | C | B | run |  |
| `-short, testing.Short()` | fix | C K | B | test | probe short-on: the test reads it and skips |
| `-shuffle` | ord | C | B | test | judgement: a test that depends on order can change outcome |
| `-skip` | sel | C | B | run |  |
| `-timeout` | tmo | C | T ↕ | test | probes timeout-*: 0 disables it; a cached pass ignores it |
| `-v` | out | C | B |  |  |
| `-vet` | chk | C | L ↕ | test | probes vet-*: off turns the check off; all tightens |
| `-benchmem` | cov | C | B |  |  |
| `-blockprofile` | cov | C | B |  |  |
| `-blockprofilerate` | cov | C | B |  |  |
| `-coverprofile` | cov | C | B |  |  |
| `-cpuprofile` | cov | C | B |  |  |
| `-memprofile` | cov | C | B |  |  |
| `-memprofilerate` | cov | C | B |  |  |
| `-mutexprofile` | cov | C | B |  |  |
| `-mutexprofilefraction` | cov | C | B |  |  |
| `-trace` | cov | C | B |  |  |
| `-args` | cfg | C | B | test | a bridge: passes the rest to the test binary |
| `-c` | dbg | C | B |  |  |
| `-exec` | fix | C | B | test | runs the test binary under a command |
| `-o` | bld | C | B |  |  |
| `-a` | bld | C | B |  |  |
| `-n` | dbg | C | B |  |  |
| `-p` | par | C | B |  |  |
| `-race` | chk | C | T | test | a data race fails the test |
| `-msan` | chk | C | T | test |  |
| `-asan` | chk | C | T | test |  |
| `-work` | dbg | C | B |  |  |
| `-x` | out | C | B |  |  |
| `-asmflags` | bld | C | B |  |  |
| `-buildmode` | bld | C | B |  |  |
| `-buildvcs` | bld | C | B |  |  |
| `-compiler` | bld | C | B |  |  |
| `-gccgoflags` | bld | C | B |  |  |
| `-gcflags` | bld | C | B |  |  |
| `-installsuffix` | bld | C | B |  |  |
| `-ldflags` | bld | C | B | test | -X sets a variable that a test can read |
| `-linkshared` | bld | C | B |  |  |
| `-mod` | bld | C | F | run |  |
| `-modcacherw` | bld | C | B |  |  |
| `-modfile` | bld | C | F | run |  |
| `-overlay` | bld | C | B | test | replaces source files |
| `-pgo` | bld | C | B |  |  |
| `-pkgdir` | bld | C | B |  |  |
| `-tags, //go:build` | imp | C K | F | test | decides which files compile |
| `-trimpath` | bld | C | B |  |  |
| `-toolexec` | bld | C | B |  |  |
| `GOFLAGS` | cfg | E U | B |  | a bridge: sets any flag; probes timeout-goflags, goenv-file |
| `GOMAXPROCS` | par | E | B | test | as -cpu |
| `GODEBUG, go.mod godebug, //go:debug` | fix | E P K | B | test | changes the runtime's behaviour |
| `GOENV` | cfg | E | B |  | names the user's file; off disables it |
| `GOCACHE` | cch | E U | B |  |  |
| `GOOS, GOARCH` | bld | E U | F | test |  |
| `CGO_ENABLED` | bld | E U | F | test |  |
| `GOEXPERIMENT` | bld | E U | B | test |  |
| `GOTOOLCHAIN` | bld | E U | F | test |  |
| `t.Parallel()` | par | K | B | test | judgement: a test that shares state can change outcome |
| `t.Skip, t.SkipNow, t.Skipf` | sel | K | L | test |  |
| `t.Setenv` | fix | K | B | test |  |
| `TestMain` | fix | K | B | test | code wraps the whole run |

### vitest

| dial | group | levels | class | outcome | note |
|---|---|---|---|---|---|
| `[...filters] (positional)` | sel | C | B | run |  |
| `-v, --version` | dbg | C | B |  |  |
| `-h, --help` | dbg | C | B |  |  |
| `-r, --root` | imp | C P | F | run |  |
| `-c, --config` | cfg | C | B | test | indirect: picks the file that sets every other dial |
| `-u, --update` | ast | C P | L | test | a snapshot that differs is rewritten, and the test passes |
| `-w, --watch` | stp | C P | B |  |  |
| `--run` | stp | C | B |  |  |
| `-t, --testNamePattern` | sel | C P | B | run |  |
| `--dir` | imp | C P | F | run |  |
| `--ui` | dbg | C P | B |  |  |
| `--open` | dbg | C P E | B |  | its default is !process.env.CI |
| `--api, --api.port` | dbg | C P | B |  |  |
| `--api.host` | dbg | C P | B |  |  |
| `--api.strictPort` | dbg | C P | B |  |  |
| `--api.allowExec` | dbg | C P | B |  |  |
| `--api.allowWrite` | dbg | C P | B |  |  |
| `--silent` | out | C P | B |  |  |
| `--hideSkippedTests` | out | C P | B |  |  |
| `--reporter` | out | C | B |  |  |
| `--outputFile` | out | C P | B |  |  |
| `--coverage, --coverage.enabled` | cov | C P | B |  |  |
| `--coverage.provider` | cov | C P | B |  |  |
| `--coverage.include` | cov | C P | B |  |  |
| `--coverage.exclude` | cov | C P | B |  |  |
| `--coverage.clean` | cov | C P | B |  |  |
| `--coverage.cleanOnRerun` | cov | C P | B |  |  |
| `--coverage.reportsDirectory` | cov | C P | B |  |  |
| `--coverage.reporter` | cov | C P | B |  |  |
| `--coverage.reportOnFailure` | cov | C P | B |  |  |
| `--coverage.allowExternal` | cov | C P | B |  |  |
| `--coverage.skipFull` | cov | C P | B |  |  |
| `--coverage.thresholds.100` | cov | C P | T | run | fails the run under the threshold |
| `--coverage.thresholds.perFile` | cov | C P | T | run |  |
| `--coverage.thresholds.autoUpdate` | cov | C P | L | run | raises the stored thresholds; the next run checks the new value |
| `--coverage.thresholds.lines` | cov | C P | T ↕ | run |  |
| `--coverage.thresholds.functions` | cov | C P | T ↕ | run |  |
| `--coverage.thresholds.branches` | cov | C P | T ↕ | run |  |
| `--coverage.thresholds.statements` | cov | C P | T ↕ | run |  |
| `--coverage.ignoreClassMethods` | cov | C P | L | run | leaves methods out of the threshold |
| `--coverage.processingConcurrency` | cov | C P | B |  |  |
| `--coverage.customProviderModule` | cov | C P | B |  |  |
| `--coverage.watermarks.statements` | cov | C P | B |  |  |
| `--coverage.watermarks.lines` | cov | C P | B |  |  |
| `--coverage.watermarks.branches` | cov | C P | B |  |  |
| `--coverage.watermarks.functions` | cov | C P | B |  |  |
| `--coverage.changed` | cov | C P | B |  |  |
| `--coverage.excludeAfterRemap` | cov | C P | B |  |  |
| `--coverage.htmlDir` | cov | C P | B |  |  |
| `--coverage.autoAttachSubprocess` | cov | C P | B |  |  |
| `--mode` | cfg | C P | B | test | the Vite mode picks other env files |
| `--isolate` | fix | C P | B | test | judgement: without isolation, files share module state |
| `--globals` | fix | C P | B | test | injects the API as globals |
| `--injectCjsGlobals` | fix | C P | B | test |  |
| `--dom` | fix | C | B | test |  |
| `--browser, --browser.enabled` | fix | C P | B | test |  |
| `--browser.name` | fix | C P | B | test |  |
| `--browser.headless` | fix | C P E | B |  | its default is process.env.CI |
| `--browser.ui` | dbg | C P E | B |  | its default is !process.env.CI |
| `--browser.detailsPanelPosition` | dbg | C P | B |  |  |
| `--browser.connectTimeout` | tmo | C P | T ↕ | run |  |
| `--browser.dependencySourcemaps` | out | C P | B |  |  |
| `--browser.trackUnhandledErrors` | chk | C P | L ↕ | run | false stops catching uncaught errors |
| `--browser.importMapMocks` | fix | C P | B |  |  |
| `--browser.trace` | dbg | C P | B |  |  |
| `--browser.traceView, --browser.traceView.mode, --browser.traceView.enabled` | dbg | C P | B |  |  |
| `--browser.traceView.recordCanvas` | dbg | C P | B |  |  |
| `--browser.traceView.inlineImages` | dbg | C P | B |  |  |
| `--browser.traceView.ariaSnapshot` | dbg | C P | B |  |  |
| `--browser.locators.exact` | fix | C P | B | test | changes which element a locator finds |
| `--pool` | par | C P | B | test | judgement: threads and forks isolate differently |
| `--execArgv` | fix | C P | B | test |  |
| `--vmMemoryLimit` | par | C P | B |  |  |
| `--fileParallelism` | par | C P | B |  |  |
| `--maxWorkers` | par | C P | B |  |  |
| `--environment` | fix | C P K | B | test | a docblock in the file sets it too |
| `--passWithNoTests` | stp | C P | L | run | no tests found is no longer a failure |
| `--logHeapUsage` | out | C P | B |  |  |
| `--detectAsyncLeaks` | chk | C P | T | test |  |
| `--allowOnly` | chk | C P E | L | run | probe only-ci: under CI a .only is an error unless --allowOnly |
| `--dangerouslyIgnoreUnhandledErrors` | chk | C P | L | run |  |
| `--shard` | sel | C | B | run |  |
| `--changed` | sel | C P | B | run |  |
| `--sinceLastRun` | sel | C | B | run |  |
| `--lastFailed` | sel | C | B | run |  |
| `--sequence.shuffle` | ord | C P | B | test | judgement: a test that depends on order can change outcome |
| `--sequence.shuffle.files` | ord | C P | B | test | as --sequence.shuffle |
| `--sequence.shuffle.tests` | ord | C P | B | test | as --sequence.shuffle |
| `--sequence.concurrent` | par | C P | B | test | judgement: tests that share state can change outcome |
| `--sequence.seed` | ord | C P | B |  |  |
| `--sequence.hooks` | ord | C P | B | test |  |
| `--sequence.setupFiles` | ord | C P | B | test |  |
| `--inspect` | dbg | C P | B |  |  |
| `--inspectBrk` | dbg | C P | B |  |  |
| `--testTimeout` | tmo | C P K | T ↕ | test | probes timeout-*: 0 disables it; the test's own timeout beats the flag |
| `--hookTimeout` | tmo | C P | T ↕ | test | 0 disables it |
| `--bail` | stp | C P | B | run |  |
| `--retry, --retry.count` | rep | C P K | L | test | probe retry-cli: a test that failed once passes |
| `--retry.delay` | rep | C P | B |  |  |
| `--retry.condition` | rep | C P | L | test | limits which errors a retry covers |
| `--repeats` | rep | C P K | B | test | more runs can expose a flaky failure |
| `--diff (a module of options)` | ast | C P | B |  |  |
| `--diff.aAnnotation` | ast | C P | B |  |  |
| `--diff.aIndicator` | ast | C P | B |  |  |
| `--diff.bAnnotation` | ast | C P | B |  |  |
| `--diff.bIndicator` | ast | C P | B |  |  |
| `--diff.commonIndicator` | ast | C P | B |  |  |
| `--diff.contextLines` | ast | C P | B |  |  |
| `--diff.emptyFirstOrLastLinePlaceholder` | ast | C P | B |  |  |
| `--diff.expand` | ast | C P | B |  |  |
| `--diff.includeChangeCounts` | ast | C P | B |  |  |
| `--diff.omitAnnotationLines` | ast | C P | B |  |  |
| `--diff.printBasicPrototype` | ast | C P | B |  |  |
| `--diff.maxDepth` | ast | C P | B |  |  |
| `--diff.truncateThreshold` | ast | C P | B |  |  |
| `--diff.truncateAnnotation` | ast | C P | B |  |  |
| `--exclude` | sel | C P | B | run |  |
| `--expandSnapshotDiff` | ast | C P | B |  |  |
| `--disableConsoleIntercept` | cap | C P | B |  |  |
| `--typecheck, --typecheck.enabled` | chk | C P | T | test | adds type errors as failures |
| `--typecheck.only` | sel | C P | B | run |  |
| `--typecheck.checker` | chk | C P | F | test |  |
| `--typecheck.allowJs` | chk | C P | T | test |  |
| `--typecheck.ignoreSourceErrors` | chk | C P | L | test |  |
| `--typecheck.build` | chk | C P | B |  |  |
| `--typecheck.tsconfig` | chk | C P | F | test |  |
| `--typecheck.spawnTimeout` | tmo | C P | B |  |  |
| `-p, --project` | sel | C | B | run |  |
| `--slowTestThreshold` | out | C P | B |  |  |
| `--teardownTimeout` | tmo | C P | T ↕ | run |  |
| `--cache` | cch | C P | B |  |  |
| `--maxConcurrency` | par | C P | B |  |  |
| `--fsModuleCache` | cch | C P | B |  |  |
| `--fsModuleCachePath` | cch | C P | B |  |  |
| `--expect.requireAssertions` | chk | C P | T | test | a test with no assertion fails |
| `--expect.poll.interval` | ast | C P | B |  |  |
| `--expect.poll.timeout` | tmo | C P | T ↕ | test |  |
| `--printConsoleTrace` | out | C P | B |  |  |
| `--includeTaskLocation` | out | C P | B |  |  |
| `--attachmentsDir` | out | C P | B |  |  |
| `--no-color` | out | C | B |  |  |
| `--clearScreen` | out | C | B |  |  |
| `--configLoader` | cfg | C | B |  |  |
| `--standalone` | stp | C | B |  |  |
| `--mergeReports` | out | C | B |  |  |
| `--listTags` | dbg | C | B |  |  |
| `--clearCache` | cch | C | B |  |  |
| `--tagsFilter` | sel | C | B | run |  |
| `--strictTags` | chk | C P | T | test | an undefined tag is an error |
| `--sharedViteServer` | cfg | C P | B |  |  |
| `--experimental.importDurations.print` | out | C P | B |  |  |
| `--experimental.importDurations.limit` | out | C P | B |  |  |
| `--experimental.importDurations.failOnDanger` | chk | C P | T | run |  |
| `--experimental.importDurations.thresholds.warn` | out | C P | B |  |  |
| `--experimental.importDurations.thresholds.danger` | chk | C P | T ↕ | run | with failOnDanger |
| `--experimental.viteModuleRunner` | fix | C P | B | test |  |
| `--experimental.nodeLoader` | fix | C P | B | test |  |
| `--experimental.traceTimeline` | out | C P | B |  |  |
| `--experimental.vcsProvider` | sel | C P | B | run |  |
| `--experimental.recordDependencies` | sel | C P | B | run |  |
| `--experimental.preParse` | sel | C P | B | run |  |
| `--experimental.diagnostics.isolate` | out | C P | B |  |  |
| `--experimental.diagnostics.environment` | out | C P | B |  |  |
| `--experimental.diagnostics.import` | out | C P | B |  |  |
| `--experimental.diagnostics.transform` | out | C P | B |  |  |
| `include` | imp | P | F | run |  |
| `setupFiles` | fix | P | B | test |  |
| `globalSetup` | fix | P | B | test |  |
| `snapshotFormat` | ast | P | B | test | changes the text a snapshot compares |
| `snapshotSerializers` | ast | P | B | test |  |
| `includeSource` | imp | P | F | run |  |
| `alias` | imp | P | F | test |  |
| `env` | fix | P | B | test |  |
| `environmentOptions` | fix | P | B | test |  |
| `unstubEnvs` | fix | P | B | test |  |
| `restoreMocks` | fix | P | B | test |  |
| `runner` | plg | P | B | test |  |
| `mockReset` | fix | P | B | test |  |
| `forceRerunTriggers` | cch | P | B |  |  |
| `unstubGlobals` | fix | P | B | test |  |
| `uiBase` | dbg | P | B |  |  |
| `benchmark` | sel | P | B | run |  |
| `fakeTimers` | fix | P | B | test |  |
| `chaiConfig` | ast | P | B |  |  |
| `clearMocks` | fix | P | B | test |  |
| `css` | fix | P | B | test |  |
| `deps` | imp | P | F | test |  |
| `name` | cfg | P | B |  |  |
| `snapshotEnvironment` | ast | P | B |  |  |
| `provide` | fix | P | B | test |  |
| `projects` | cfg | P | B | test | indirect: each project holds its own values |
| `watchTriggerPatterns` | cch | P | B |  |  |
| `tags` | chk | P | F | test | with --strictTags, an undefined tag is an error |
| `taskTitleValueFormatTruncate` | out | P | B |  |  |
| `server` | imp | P | F | test |  |
| `onConsoleLog` | cap | P | B |  |  |
| `onStackTrace` | out | P | B |  |  |
| `onUnhandledError` | chk | P | L ↕ | run | a callback can drop an unhandled error |
| `resolveSnapshotPath` | ast | P | F | test |  |
| `test options: timeout` | tmo | K | T ↕ | test | probe timeout-code-vs-cli: it beats --testTimeout |
| `test options: retry` | rep | K | L | test |  |
| `test options: repeats` | rep | K | B | test |  |
| `test options: concurrent` | par | K | B | test |  |
| `test options: skip, test.skip, test.skipIf, test.runIf` | sel | K | L | test |  |
| `test options: only, test.only` | sel | K | B | run | under CI a .only is an error |
| `test options: todo` | sel | K | B | run |  |
| `test options: fails, test.fails` | chk | K | L | test | a failure passes; a pass fails |
| `test options: tags` | sel | K | B | run |  |

### jest

| dial | group | levels | class | outcome | note |
|---|---|---|---|---|---|
| `[TestPathPatterns] (positional)` | sel | C | B | run |  |
| `-h, --help` | dbg | C | B |  |  |
| `--version` | dbg | C | B |  |  |
| `--all` | sel | C | B | run |  |
| `--automock` | fix | C P | B | test | every import becomes a mock |
| `-b, --bail` | stp | C P | B | run |  |
| `--cache` | cch | C P | B |  |  |
| `--cacheDirectory` | cch | C P | B |  |  |
| `--changedFilesWithAncestor` | sel | C P | B | run |  |
| `--changedSince` | sel | C P | B | run |  |
| `--ci` | ast | C P E | T | test | probes snapshot-ci-*: a new snapshot fails instead of being written |
| `--clearCache` | cch | C | B |  |  |
| `--clearMocks` | fix | C P | B | test |  |
| `--coverage, --collectCoverage` | cov | C P | B |  |  |
| `--collectCoverageFrom` | cov | C P | B |  |  |
| `--collectTests` | dbg | C P | B |  |  |
| `--color, --colors` | out | C | B |  |  |
| `-c, --config` | cfg | C | B | test | indirect: picks the file that sets every other dial |
| `--coverageDirectory` | cov | C P | B |  |  |
| `--coveragePathIgnorePatterns` | cov | C P | L | run | leaves files out of the threshold |
| `--coverageProvider` | cov | C P | B |  |  |
| `--coverageReporters` | cov | C P | B |  |  |
| `--coverageThreshold` | cov | C P | T ↕ | run | fails the run under the threshold |
| `--debug` | dbg | C | B |  |  |
| `--detectLeaks` | chk | C P | T | test | a leaked global fails the test |
| `--detectOpenHandles` | dbg | C P | B |  | also runs in band |
| `--errorOnDeprecated` | chk | C P | T | test | a deprecated API throws |
| `-e, --expand` | ast | C P | B |  |  |
| `--filter` | sel | C P | B | run |  |
| `--findRelatedTests` | sel | C P | B | run |  |
| `--forceExit` | stp | C P | L | run | exits past open handles |
| `--globalSetup` | fix | C P | B | test |  |
| `--globalTeardown` | fix | C P | B | test |  |
| `--globals` | fix | C P | B | test |  |
| `--haste` | imp | C P | F | test |  |
| `--ignoreProjects` | sel | C | B | run |  |
| `--injectGlobals` | fix | C P | B | test |  |
| `--json` | out | C P | B |  |  |
| `--lastCommit` | sel | C P | B | run |  |
| `--listTests` | dbg | C P | B |  |  |
| `--logHeapUsage` | out | C P | B |  |  |
| `--maxConcurrency` | par | C P | B |  |  |
| `-w, --maxWorkers` | par | C P | B |  |  |
| `--moduleDirectories` | imp | C P | F | test |  |
| `--moduleFileExtensions` | imp | C P | F | test |  |
| `--moduleNameMapper` | imp | C P | F | test |  |
| `--modulePathIgnorePatterns` | imp | C P | F | test |  |
| `--modulePaths` | imp | C P | F | test |  |
| `--noStackTrace` | out | C P | B |  |  |
| `--notify` | out | C P | B |  |  |
| `--notifyMode` | out | C P | B |  |  |
| `-o, --onlyChanged` | sel | C P | B | run |  |
| `-f, --onlyFailures` | sel | C P | B | run |  |
| `--openHandlesTimeout` | dbg | C P | B |  |  |
| `--outputFile` | out | C P | B |  |  |
| `--passWithNoTests` | stp | C P | L | run | no tests found is no longer a failure |
| `--preset` | cfg | C P | B | test | indirect: a base set of values |
| `--prettierPath` | ast | C P | B |  |  |
| `--projects` | cfg | C P | B | test | indirect: each project holds its own values |
| `--randomize` | ord | C P | B | test | judgement: a test that depends on order can change outcome |
| `--reporters` | out | C P | B |  |  |
| `--resetMocks` | fix | C P | B | test |  |
| `--resetModules` | fix | C P | B | test |  |
| `--resolver` | imp | C P | F | test |  |
| `--restoreMocks` | fix | C P | B | test |  |
| `--rootDir` | imp | C P | F | run |  |
| `--roots` | imp | C P | F | run |  |
| `-i, --runInBand` | par | C | B |  |  |
| `--runTestsByPath` | sel | C P | B | run |  |
| `--runner` | plg | C P | B | test |  |
| `--seed` | ord | C | B |  |  |
| `--selectProjects` | sel | C | B | run |  |
| `--setupFiles` | fix | C P | B | test |  |
| `--setupFilesAfterEnv` | fix | C P | B | test |  |
| `--shard` | sel | C | B | run |  |
| `--showConfig` | dbg | C | B |  |  |
| `--showSeed` | out | C P | B |  |  |
| `--silent` | out | C P | B |  |  |
| `--skipFilter` | sel | C P | B | run |  |
| `--snapshotSerializers` | ast | C P | B | test | changes the text a snapshot compares |
| `--testEnvironment, --env, @jest-environment` | fix | C P K | B | test |  |
| `--testEnvironmentOptions, @jest-environment-options` | fix | C P K | B | test | a docblock's globalsCleanup takes effect only in a worker's first file |
| `--testFailureExitCode` | stp | C P | B | run |  |
| `--testLocationInResults` | out | C P | B |  |  |
| `--testMatch` | imp | C P | F | run |  |
| `-t, --testNamePattern` | sel | C P | B | run |  |
| `--testPathIgnorePatterns` | imp | C P | F | run |  |
| `--testPathPatterns` | sel | C | B | run |  |
| `--testRegex` | imp | C P | F | run |  |
| `--testResultsProcessor` | out | C P | B |  |  |
| `--testRunner` | plg | C P | B | test |  |
| `--testSequencer` | ord | C P | B | test |  |
| `--testTimeout, jest.setTimeout, test(name, fn, timeout)` | tmo | C P K | T ↕ | test | probes timeout-*: the test's own timeout beats the flag |
| `--transform` | imp | C P | F | test |  |
| `--transformIgnorePatterns` | imp | C P | F | test |  |
| `--unmockedModulePathPatterns` | fix | C P | B | test |  |
| `-u, --updateSnapshot` | ast | C | L | test | a snapshot that differs is rewritten, and the test passes |
| `--useStderr` | out | C P | B |  |  |
| `--verbose` | out | C P | B |  |  |
| `--waitForUnhandledRejections` | chk | C P | T | test |  |
| `--watch` | stp | C P | B |  |  |
| `--watchAll` | stp | C P | B |  |  |
| `--watchPathIgnorePatterns` | cch | C P | B |  |  |
| `--watchman` | cch | C P | B |  |  |
| `--workerGracefulExitTimeout` | par | C P | B |  |  |
| `--workerThreads` | par | C P | B | test | judgement: threads and processes isolate differently |
| `dependencyExtractor` | cch | P | B |  |  |
| `displayName` | out | P | B |  |  |
| `extensionsToTreatAsEsm` | imp | P | F | test |  |
| `fakeTimers` | fix | P | B | test |  |
| `forceCoverageMatch` | cov | P | B |  |  |
| `id` | cch | P | B |  |  |
| `replname` | dbg | P | B |  |  |
| `runtime` | plg | P | B | test |  |
| `sandboxInjectedGlobals` | fix | P | B | test |  |
| `skipNodeResolution` | imp | P | F | test |  |
| `slowTestThreshold` | out | P | B |  |  |
| `snapshotFormat` | ast | P | B | test | changes the text a snapshot compares |
| `snapshotResolver` | ast | P | F | test |  |
| `watchPlugins` | plg | P | B |  |  |
| `workerIdleMemoryLimit` | par | P | B |  |  |
| `jest.retryTimes` | rep | K | L | test | a test that failed once passes |
| `test.skip, xit` | sel | K | L | test |  |
| `test.only` | sel | K | B | run |  |
| `test.todo` | sel | K | B | run |  |
| `test.failing` | chk | K | L | test | a failure passes; a pass fails |
| `test.concurrent` | par | K | B | test |  |
