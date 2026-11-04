# How test runners handle an interrupt

This file answers [How do other test runners handle an interrupt: what stops, what prints, which code, and what a second signal does?](https://github.com/kalonji-tools/purba/issues/766).
It is never merged.
It starts from [Which parts and cross-cutting concerns do comparable test runners have?](https://github.com/kalonji-tools/purba/issues/465), and does not measure that survey's rows again.
That survey gave the mechanism of pytest (an exception), unittest (a flag) and cargo-nextest (an event with an escalation ladder) [B].
This file reads those lines again at newer tags, and adds only what it left empty: the teardown, the output, the exit code, the later signals, SIGTERM and SIGHUP.

## Answer

The runners agree on almost nothing, so purba has no common default to copy, and must choose each behaviour on purpose.
On the first Ctrl-C, twelve of the thirteen stop the test that runs, and none starts another test.
Only RSpec lets the running test finish by default; unittest does so only under its opt-in `-c`.
Only pytest, pytest-xdist and RSpec run the suite's teardown in every mode; Vitest does so only in its watch mode on a terminal.
Eight runners run no teardown of their own, because the signal kills them by its default action.
The exit codes split too. 130 is the most common, but mostly because the process died by the signal, and not because a runner chose it.
pytest chose 2, cargo-nextest gives its code for a failed test, and `go test` gives 1.
Three runners can exit 0 after an interrupt that left tests unrun: unittest with `-c`, Ward and mocha.
Where a runner has a gentle first stage, a second Ctrl-C skips it, and two runners print that promise on the first.
Only `go test` ignores the second Ctrl-C, and only cargo-nextest gives a third one a meaning.
SIGTERM is where the Python runners are weakest: pytest, xdist, unittest and Ward all die on it with no teardown.
Yet five of the seven CI cancels and container stops surveyed here send SIGTERM first.
GitHub Actions sends SIGINT, but only to the step's shell, so a runner started from `bash` may get no signal before SIGKILL.
For purba, a conclusion cannot rest on a majority. It can rest on the failures the whole survey shows: an exit 0 after an interrupt, a SIGTERM that skips every teardown, and a second Ctrl-C that cannot force an exit.

Tiers: **[A]** fetched or run in this session, with the command keyed in Appendix A. **[B]** relayed, not re-verified. **[C]** could not establish.
A row that names a probe key ran the runner and sent the signal to its process group, as a terminal's Ctrl-C does (`set -m`, then `kill -INT -- -<pgid>`).
Every probe used the same fixture: three tests of about 3 s each. Test one writes a marker when it starts. A test-level teardown and a suite-level teardown each append to a log.
Probe keys carry the cluster that ran them: `py:` Python, `rs:` Rust, `go:` Go, `js:` JavaScript, `jv:` JVM, `rb:` Ruby, `ci:` CI.

## At a glance: the first Ctrl-C, in the runner's default mode

| runner | the running test | a later test starts | teardown that runs | what prints | exit | 2nd Ctrl-C | SIGTERM like SIGINT? |
|---|---|---|---|---|---|---|---|
| pytest 9.2.0 | stops | no | `finally`, every fixture finalizer, session fixtures | banner, crash line, count of finished tests | 2 | aborts the teardown left; 130 | no: dies, 143 |
| pytest-xdist 3.8.0 | stops, in each worker | no | all of pytest's, in each worker | pytest's banner; `no tests ran` | 2 | aborts the teardown left; 130 | no: dies, 143 |
| unittest (CPython 3.15) | stops | no | `finally` only | a traceback | 130 | — | no: dies, 143 |
| unittest `-c` | **finishes** | no | all: `tearDown`, cleanups, class, module | the normal summary, `OK`; no mark of the interrupt | **0** if all ran passed | KI; aborts; 130 | no: dies, 143 |
| Ward 0.68.0b0 | stops | no | `finally` only | `Run cancelled`, results of finished tests | 0 or 3, from finished tests | 130 | no: dies, 143 |
| cargo-nextest 0.9.151 | gets SIGINT; 10 s grace, then SIGKILL | no | none in Rust; the test may handle the signal | `Cancelling due to interrupt`, a line per test, summary, `N/M tests were not run` | 100 | SIGKILL now | **yes**: forwards SIGTERM |
| `cargo test` (libtest) | dies | no | none | nothing | 130 | — | dies, 143 |
| `go test` go1.27.3 | dies | no; no later package | none: no `defer`, `t.Cleanup` or `TestMain` tail | `signal: interrupt`, `FAIL` (racy without `-v`) | 1 | ignored | no: dies, 143 |
| Jest 30.5.2 | dies | no | none | nothing | 130 | — | dies, 143 |
| Vitest 5.1.0 `run` | dies | no | none | the `RUN` banner only | 130, by `process.exit` | — | **yes**: 143 |
| Vitest watch, on a TTY | aborted | no | `afterEach`, `afterAll`; not the test's `finally` | `Cancelling test run…`, summary, `CANCELLED` | 1 | real SIGINT; 130 | — |
| mocha 12.0.3 | dies | no | none | blank lines | 130; **0** in `--parallel` or with a node flag | — | dies, 143 |
| JUnit 5 under Gradle 9.8.2 | the daemon cancels the build and sends the worker SIGTERM; it dies | no | none; JVM shutdown hooks only | nothing in the terminal; `Build cancelled` only in the daemon's log | 130, the client's | 130; the first status wins | the same: 143 |
| JUnit 5 under Surefire 3.6.1 | dies: the fork is in the group, and Maven also sends it SIGTERM | no | none; JVM shutdown hooks only | last line `[INFO] Running probe.ThreeTest`; no `Results:` | 130 | 130 | the same: 143 |
| RSpec 3.13.6 | **finishes** | no | all: `ensure`, `after`, `after(:all)`, `after(:suite)`, `at_exit` | a banner, then the summary of the examples that ran | 1; 0 when every example ran | `exit!(1)`: no hook, no summary | not trapped, but Ruby unwinds: every hook runs, the summary prints, 143 |

"dies" means the process ends by the signal's default action, or the JVM's default handler. "—" means the first signal already ended the process, or the mode does not apply.
Each cell is [A], by probe and source. Sections 1 to 5 hold the evidence.
The counts in the Answer take unittest without `-c`, and Vitest's `run` mode, which CI uses. Their other modes have rows of their own.

## 1. The first SIGINT: what stops, and what tears down

| runner | the running test | a later test starts? | test teardown | suite or session teardown | tier | source, probe |
|---|---|---|---|---|---|---|
| pytest | stops at once: `KeyboardInterrupt` is raised in the test | no | the test's `finally` and its function fixture finalizer run | session fixture teardown runs, from `pytest_sessionfinish` in `wrap_session`'s `finally` | [A] | [runner.py#L111-L112](https://github.com/pytest-dev/pytest/blob/9.2.0/src/_pytest/runner.py#L111-L112), [runner.py#L140-L144](https://github.com/pytest-dev/pytest/blob/9.2.0/src/_pytest/runner.py#L140-L144), [main.py#L339-L376](https://github.com/pytest-dev/pytest/blob/9.2.0/src/_pytest/main.py#L339-L376); py:M1, py:M3 |
| pytest-xdist | each worker gets its own SIGINT, and its test stops | no | in each worker, as pytest | in each worker, as pytest; the controller waits for them | [A] | py:M7 |
| unittest | stops at once | no | only the test's `finally`. No `tearDown`, no `addCleanup`: `testPartExecutor` re-raises `KeyboardInterrupt`, and `doCleanups()` is not in a `finally` | none | [A] | [case.py#L53-L60](https://github.com/python/cpython/blob/v3.15.2/Lib/unittest/case.py#L53-L60), [case.py#L683-L693](https://github.com/python/cpython/blob/v3.15.2/Lib/unittest/case.py#L683-L693); py:M12 |
| unittest `-c` | **finishes**: the handler only sets `shouldStop` | no | all: `tearDown` and cleanups | `tearDownClass` and `tearDownModule` run | [A] | [signals.py#L28-L39](https://github.com/python/cpython/blob/v3.15.2/Lib/unittest/signals.py#L28-L39), [suite.py#L107-L131](https://github.com/python/cpython/blob/v3.15.2/Lib/unittest/suite.py#L107-L131); py:M13 |
| Ward | stops at once: `Test.run` catches only `(Exception, SystemExit)` | no | only the test's `finally`; no fixture teardown | no global fixture teardown: the teardowns live in a generator that dies with the `KeyboardInterrupt` | [A] | [testing.py#L212](https://github.com/darrenburns/ward/blob/f41c6164f3923660e3b03c3797913fab23496699/ward/testing.py#L212), [_suite.py#L61-L96](https://github.com/darrenburns/ward/blob/f41c6164f3923660e3b03c3797913fab23496699/ward/_suite.py#L61-L96); py:M19, py:M20 |
| cargo-nextest | gets the same signal. With the default action it dies at once. A test that handles it has the grace period, 10 s by default, then SIGKILL | no: the dispatcher drops every start event once a cancel reason is set | none: no `Drop` ran under default-action SIGINT | a running setup script gets the same signal and grace; no new one starts. Teardown scripts do not exist | [A] | [dispatcher.rs#L535-L538](https://github.com/nextest-rs/nextest/blob/cargo-nextest-0.9.151/nextest-runner/src/runner/dispatcher.rs#L535-L538), [unix.rs#L214-L234](https://github.com/nextest-rs/nextest/blob/cargo-nextest-0.9.151/nextest-runner/src/runner/unix.rs#L214-L234), [slow_timeout.rs#L74-L76](https://github.com/nextest-rs/nextest/blob/cargo-nextest-0.9.151/nextest-runner/src/config/elements/slow_timeout.rs#L74-L76), [runner-loop.md#L14](https://github.com/nextest-rs/nextest/blob/cargo-nextest-0.9.151/site/src/docs/design/architecture/runner-loop.md#L14); rs:M1, rs:M5 |
| `cargo test` | dies: libtest has no signal handler, and cargo has none on Unix | no | no `Drop` | none exists | [A] | grep of all 30 files of [library/test/src](https://github.com/rust-lang/rust/tree/1.99.1/library/test/src) finds only [test_result.rs#L118-L122](https://github.com/rust-lang/rust/blob/1.99.1/library/test/src/test_result.rs#L118-L122), which classifies a child's exit; rs:L1, rs:L2 |
| `go test` | dies: the `testing` package has no SIGINT handler | no; and no later **package** starts | no `defer`, no `t.Cleanup` | no code after `m.Run()` in `TestMain` | [A] | [testing.go#L401-L424](https://github.com/golang/go/blob/go1.27.3/src/testing/testing.go#L401-L424) (no `os/signal`), [test.go#L1459-L1464](https://github.com/golang/go/blob/go1.27.3/src/cmd/go/internal/test/test.go#L1459-L1464); go:G1, go:G5 |
| Jest | dies: no SIGINT handler outside watch mode | no | no `afterEach`, no `finally` | no `afterAll` | [A] | the only `process.on` in jest-cli is [`'exit'`](https://github.com/jestjs/jest/blob/v30.5.2/packages/jest-cli/src/run.ts#L104-L115); js:J1, js:J6 |
| Vitest `run` | dies: its workers get the SIGINT too | no | none | none | [A] | js:V1, js:V6 |
| Vitest watch (TTY) | aborted, `[The test run was aborted by the user.]`; its body is abandoned, not awaited | no | `afterEach` runs; the test's `finally` does not | `afterAll` runs | [A] | [stdin.ts#L85-L96](https://github.com/vitest-dev/vitest/blob/v5.1.0/packages/vitest/src/node/stdin.ts#L85-L96), [run.ts#L1058-L1067](https://github.com/vitest-dev/vitest/blob/v5.1.0/packages/vitest/src/runtime/runner/run.ts#L1058-L1067); js:P1, js:P6 |
| mocha | dies: serial mode runs in process with no SIGINT handler | no | none | none. `Runner#abort()` exists, and no SIGINT path calls it | [A] | [bin/mocha.js#L161-L162](https://github.com/mochajs/mocha/blob/v12.0.3/bin/mocha.js#L161-L162), [runner.js#L1306-L1311](https://github.com/mochajs/mocha/blob/v12.0.3/lib/runner.js#L1306-L1311); js:K1, js:K7 |
| JUnit Platform 6.1.3 itself | stops: the platform ships no signal handler and no shutdown hook, so the JVM's default applies | no | no `@AfterEach`, no `finally` | no `@AfterAll`. The JVM runs only its shutdown hooks, which "run concurrently with any daemon or non-daemon threads" | [A] | grep of the [r6.1.3](https://github.com/junit-team/junit-framework/tree/r6.1.3) tree finds `addShutdownHook` only in a test fixture; [Runtime javadoc, JDK 21](https://docs.oracle.com/en/java/javase/21/docs/api/java.base/java/lang/Runtime.html#shutdown); jv:M14 |
| JUnit 5 under Gradle | stops. The client dies; the daemon sees the client disconnect, cancels the build, and destroys the worker. The daemon's log says `ThreeTest > one() SKIPPED` | no | none | none; the worker's shutdown hooks run | [A] | [WatchForDisconnection.java#L30-L35](https://github.com/gradle/gradle/blob/v9.8.2/platforms/core-runtime/launcher/src/main/java/org/gradle/launcher/daemon/server/exec/WatchForDisconnection.java#L30-L35), [DefaultExecHandle.java#L390-L393](https://github.com/gradle/gradle/blob/v9.8.2/platforms/core-runtime/process-services-base/src/main/java/org/gradle/process/internal/DefaultExecHandle.java#L390-L393); jv:M4 |
| JUnit 5 under Surefire | stops | no | none | none; the fork's shutdown hooks run | [A] | jv:M10, jv:M13 |
| RSpec | **finishes**: the trap sets `wants_to_quit` | no: `next if RSpec.world.wants_to_quit` | `ensure` and `after` run | `after(:all)`, `after(:suite)` and `at_exit` run | [A] | [runner.rb#L175-L191](https://github.com/rspec/rspec/blob/rspec-core-v3.13.6/rspec-core/lib/rspec/core/runner.rb#L175-L191), [example_group.rb#L648-L658](https://github.com/rspec/rspec/blob/rspec-core-v3.13.6/rspec-core/lib/rspec/core/example_group.rb#L648-L658), [configuration.rb#L2091-L2103](https://github.com/rspec/rspec/blob/rspec-core-v3.13.6/rspec-core/lib/rspec/core/configuration.rb#L2091-L2103); rb:M1 |

- **Two runners change with their mode.** unittest's `-c` is opt-in, and lets the test finish. Vitest's graceful cancel exists only in watch mode on a TTY, where raw mode turns Ctrl-C into a keypress. Watch mode is Vitest's default only off CI, on a TTY, and when no agent is detected: `watch: !isCI && process.stdin.isTTY && !isAgent` ([defaults.ts#L95](https://github.com/vitest-dev/vitest/blob/v5.1.0/packages/vitest/src/defaults.ts#L95)). The docs say a test is cancelled "when someone presses <kbd>Ctrl</kbd>+<kbd>C</kbd> in the terminal" ([cancellable.md#L7](https://github.com/vitest-dev/vitest/blob/v5.1.0/docs/guide/recipes/cancellable.md?plain=1#L7)); under `vitest run` the probe shows no cancel (js:V1). [A]
- **Watch modes of mocha and Jest.** mocha's watch mode prints `[mocha] cleaning up, please wait...`, runs the global teardown fixtures and calls `process.exit(130)` ([watch-run.cjs#L518-L533](https://github.com/mochajs/mocha/blob/v12.0.3/lib/cli/watch-run.cjs#L518-L533)); source only, not probed. Jest's `--watchAll` exits 0 on Ctrl-C with no teardown ([watch.ts#L349-L356](https://github.com/jestjs/jest/blob/v30.5.2/packages/jest-core/src/watch.ts#L349-L356); js:P9). [A]
- **A test that ignores SIGINT.** Under `cargo test`, cargo dies and the test binary goes on as an orphan, to the end of every test (rs:L5). Under `go test`, `go` waits for the binary to finish (go:G4). Under nextest, the test is SIGKILLed after the 10 s grace (rs:M5). [A]

## 2. What prints, and which exit code

| runner | the interrupted test's line | summary and count | banner | exit | by the signal, or by a code? | tier | source, probe |
|---|---|---|---|---|---|---|---|
| pytest | no outcome letter; with `-v`, the node id with nothing after it | yes: the stats line counts only finished tests, `no tests ran in 2.31s`, `1 passed in 5.32s` | `!!! KeyboardInterrupt !!!`, the crash location, and `(to show a full traceback on KeyboardInterrupt use --full-trace)` | 2, `ExitCode.INTERRUPTED` | a code: returncode 2, not −2 | [A] | [config/__init__.py#L121](https://github.com/pytest-dev/pytest/blob/9.2.0/src/_pytest/config/__init__.py#L121), [terminal.py#L1053-L1071](https://github.com/pytest-dev/pytest/blob/9.2.0/src/_pytest/terminal.py#L1053-L1071); py:M1, py:M3, py:M23 |
| pytest-xdist | started tests with no outcome | `no tests ran`: results that workers send after the controller's interrupt are not shown | pytest's, located in `threading.py:363` | 2 | a code | [A] | py:M7, py:M11, py:M24 |
| unittest | `test_1_one (…) ... ` then a traceback | none | none | 130 | **the signal**: CPython re-raises SIGINT when a `KeyboardInterrupt` goes unhandled | [A] | [main.c#L792-L815](https://github.com/python/cpython/blob/v3.15.2/Modules/main.c#L792-L815), [main.c#L848-L850](https://github.com/python/cpython/blob/v3.15.2/Modules/main.c#L848-L850); py:M12, py:M23 |
| unittest `-c` | `... ok` | `Ran 1 test in 4.502s`, `OK` | **none**: nothing says the run was interrupted | **0** after passing tests; 5 if no test ran | a code. `wasSuccessful()` reads failures, errors and unexpected successes, never `shouldStop` | [A] | [result.py#L172-L179](https://github.com/python/cpython/blob/v3.15.2/Lib/unittest/result.py#L172-L179), [main.py#L271-L277](https://github.com/python/cpython/blob/v3.15.2/Lib/unittest/main.py#L271-L277); py:M13 |
| Ward | not listed | the results of finished tests | `Run cancelled - results for tests that ran shown below.` | from finished tests only: 3 (`NO_TESTS_FOUND`) or 0 (`SUCCESS`). No code for an interrupt | a code | [A] | [_terminal.py#L707-L711](https://github.com/darrenburns/ward/blob/f41c6164f3923660e3b03c3797913fab23496699/ward/_terminal.py#L707-L711), [_terminal.py#L1234-L1244](https://github.com/darrenburns/ward/blob/f41c6164f3923660e3b03c3797913fab23496699/ward/_terminal.py#L1234-L1244), [models.py#L81-L89](https://github.com/darrenburns/ward/blob/f41c6164f3923660e3b03c3797913fab23496699/ward/models.py#L81-L89); py:M19, py:M20 |
| cargo-nextest | `SIGINT [ 0.739s] (1/3) sigfix tests::one` | `Summary [ 0.741s] 1/3 tests run: 0 passed, 1 failed, 0 skipped`, then `warning: 2/3 tests were not run due to interrupt` | `Cancelling due to interrupt: 1 test still running`; `error: test run failed` | 100, `TEST_RUN_FAILED`; 105 if the cancel lands in a setup script. No code for an interrupt | a code: `RunOutcome::Cancelled` maps to the same error as `Failed` | [A] | [exit_codes.rs#L28-L29](https://github.com/nextest-rs/nextest/blob/cargo-nextest-0.9.151/nextest-metadata/src/exit_codes.rs#L28-L29), [run.rs#L1650-L1655](https://github.com/nextest-rs/nextest/blob/cargo-nextest-0.9.151/cargo-nextest/src/dispatch/core/run.rs#L1650-L1655); rs:M1, rs:M8 |
| `cargo test` | output stops at `test tests::one ... ` | none | none | 130 | the signal: cargo and the binary both die | [A] | rs:L1, rs:L6 |
| `go test` | `=== RUN TestOne` with `-v` | none | `signal: interrupt`, then `FAIL sigfix/a 0.618s`. Without `-v` the lines are racy: 2 of 3 runs printed nothing | 1 | a code: `go` catches SIGINT, sets 1 and does not re-raise | [A] | [signal.go#L13-L31](https://github.com/golang/go/blob/go1.27.3/src/cmd/go/internal/base/signal.go#L13-L31); go:G1, go:G6 |
| Jest | none (on a TTY, `RUNS …` then `^C`) | none | none | 130 | the signal | [A] | js:J1, js:P4 |
| Vitest `run` | none | none | `RUN v5.1.0 <root>` only | 130 | **a code**: its handler sets `process.exitCode` to the signal's 128 + N and calls `process.exit()` 1 ms later | [A] | [logger.ts#L318-L333](https://github.com/vitest-dev/vitest/blob/v5.1.0/packages/vitest/src/node/logger.ts#L318-L333); js:V1 |
| Vitest watch (TTY) | `↓ one [The test run was aborted by the user.]` | `Tests 3 skipped (3)`; the cancelled file counts as `passed` | `Cancelling test run. Press CTRL+c again to exit forcefully.`, then `CANCELLED Test run cancelled.` | 1 | a code. `stdin.ts` sets 130, and `test-run.ts` then sets 1 for the `'interrupted'` state | probe [A]; the override is a reading of the source | [test-run.ts#L163-L173](https://github.com/vitest-dev/vitest/blob/v5.1.0/packages/vitest/src/node/test-run.ts#L163-L173); js:P1, js:P8 |
| mocha | none | none | none | 130 serial; **0** with `--parallel`, and **0** when `bin/mocha` spawns a child for a node flag | serial: the signal. `--parallel`: exit 0 although the source re-raises SIGINT in a `nextTick`; why it never fires is [C] | [A] | [parallel-buffered-runner.cjs#L247-L269](https://github.com/mochajs/mocha/blob/v12.0.3/lib/nodejs/parallel-buffered-runner.cjs#L247-L269), [bin/mocha.js#L110-L139](https://github.com/mochajs/mocha/blob/v12.0.3/bin/mocha.js#L110-L139); js:K1, js:K5, js:K7 |
| JUnit console launcher 6.1.3 | none | none | none: the output was empty | 130 | a code: the JVM's handler calls `Shutdown.exit(sig.getNumber() + 0200)`; strace shows `exited with 130` | [A] | [Terminator.java#L50-L51](https://github.com/openjdk/jdk21u/blob/jdk-21.0.12.1-ga/src/java.base/unix/classes/java/lang/Terminator.java#L50-L51); jv:M12, jv:M14 |
| JUnit 5 under Gradle | none in the terminal | none in the terminal. The daemon's log has `> Build cancelled while executing task ':test'` and `BUILD FAILED`. No new JUnit XML | none | 130 | a code, the client JVM's | [A] | [RunBuildAction.java#L59](https://github.com/gradle/gradle/blob/v9.8.2/platforms/core-runtime/gradle-cli/src/main/java/org/gradle/launcher/cli/RunBuildAction.java#L59) (no CLI code cancels on a signal); jv:M4 |
| JUnit 5 under Surefire | the last line is `[INFO] Running probe.ThreeTest` | none: no `Results:`, no `Tests run:`, no `BUILD` line, no `TEST-*.xml` | none | 130 (`mvn`); the fork exits 143 | a code | [A] | jv:M10, jv:M12 |
| RSpec | `.` for the example that finished | `Finished in 3 seconds`, `1 example, 0 failures`; the examples not run are not counted | `RSpec is shutting down and will print the summary report... Interrupt again to force quit (warning: at_exit hooks will be skipped if you force quit).` | 1 when an example did not run; **0** when the interrupt came during the last example, which then finished | a code | [A] | [runner.rb#L43-L46](https://github.com/rspec/rspec/blob/rspec-core-v3.13.6/rspec-core/lib/rspec/core/runner.rb#L43-L46), [runner.rb#L194-L199](https://github.com/rspec/rspec/blob/rspec-core-v3.13.6/rspec-core/lib/rspec/core/runner.rb#L194-L199); rb:M1, rb:M2 |

- **Three runners exit 0 though tests did not run.** unittest `-c` printed `Ran 1 test` and `OK` for a module of three (py:M13). Ward printed `SUCCESS` after one of three (py:M20). mocha exited 0 under `--parallel` with no output (js:K7). RSpec's 0 is not one of them: it came only when every example had run. [A]
- **mocha breaks its own documented promise.** `--posix-exit-codes` promises "`128 + <signal>` if given a signal" ([cli.mdx#L230-L243](https://github.com/mochajs/mocha/blob/v12.0.3/docs/src/content/docs/running/cli.mdx?plain=1#L230-L243)). With `--parallel` the run still exits 0 (js:K16). [A]
- **CPython's own reason for re-raising.** The comment in [main.c#L795-L798](https://github.com/python/cpython/blob/v3.15.2/Modules/main.c#L795-L798) says: "We need to exit via the SIG_DFL handler for SIGINT if KeyboardInterrupt went unhandled. If we don't, a calling process such as a shell may not know about the user's ^C." [A]

## 3. A second and a third SIGINT

| runner | 2nd SIGINT | 3rd SIGINT | tier | source, probe |
|---|---|---|---|---|
| pytest | raises `KeyboardInterrupt` in the teardown that runs. The finalizers left are skipped: `teardown_exact` catches only `(OutcomeException, Exception)`. No stats line. The banner of the first interrupt still prints, from `pytest_unconfigure`. Exit 130, by the signal | moot: the 2nd ends the process | [A] | [runner.py#L551-L581](https://github.com/pytest-dev/pytest/blob/9.2.0/src/_pytest/runner.py#L551-L581), [outcomes.py#L36](https://github.com/pytest-dev/pytest/blob/9.2.0/src/_pytest/outcomes.py#L36); py:M2 |
| pytest-xdist | hits the controller in `group.terminate` and the workers in their teardown. Both abort. Exit 130 | moot | [A] | py:M9 |
| unittest `-c` | goes to the default handler, so `KeyboardInterrupt` is raised. During the test: aborted, no teardown. During `tearDownClass`: aborted, no `tearDownModule`. Exit 130. The docs: "A second :kbd:`Control-C` raises the normal :exc:`KeyboardInterrupt` exception." | moot | [A] | [signals.py#L35-L37](https://github.com/python/cpython/blob/v3.15.2/Lib/unittest/signals.py#L35-L37), [unittest.rst#L212-L216](https://github.com/python/cpython/blob/v3.15.2/Doc/library/unittest.rst#L212-L216); py:M14, py:M15 |
| Ward | the `Run cancelled` summary still prints; then the process dies by SIGINT, exit 130. Where the 2nd `KeyboardInterrupt` lands is [C] | moot | [A] | py:M22 |
| cargo-nextest | `ShutdownRequest::Twice`: SIGKILL to every test at once; prints `Killing due to second signal: 1 test still running`. Exit 100 | `panic!("Signaled 3 times, exiting immediately")`; source only, a probe could not reach it before the run ended | [A]; 3rd from source | [unix.rs#L214-L234](https://github.com/nextest-rs/nextest/blob/cargo-nextest-0.9.151/nextest-runner/src/runner/unix.rs#L214-L234), [dispatcher.rs#L1077-L1098](https://github.com/nextest-rs/nextest/blob/cargo-nextest-0.9.151/nextest-runner/src/runner/dispatcher.rs#L1077-L1098); rs:M6, rs:M7 |
| `go test` | **ignored**: `signal.Notify` stays registered, and one goroutine reads only the first signal. If the test ignores SIGINT, `go` waits for it; the only escalation is SIGQUIT at `-timeout` + 1 min | ignored | [A] | [signal.go#L13-L31](https://github.com/golang/go/blob/go1.27.3/src/cmd/go/internal/base/signal.go#L13-L31), [test.go#L1686-L1705](https://github.com/golang/go/blob/go1.27.3/src/cmd/go/internal/test/test.go#L1686-L1705); go:G4 |
| Vitest watch (TTY) | raw mode is off by then, so it is a real SIGINT. It exits 130 at once, with no summary, and cuts the `afterEach` that runs | moot | [A] | [stdin.ts#L85-L96](https://github.com/vitest-dev/vitest/blob/v5.1.0/packages/vitest/src/node/stdin.ts#L85-L96); js:P7 |
| unittest, `cargo test`, Jest, Vitest `run`, mocha | moot: the first SIGINT ends the process | moot | [A] | py:M12, rs:L1, js:J2, js:V2, js:K2 |
| any JVM: Gradle client, Maven, the console launcher | a 2nd signal calls `Shutdown.exit` again, which blocks on the class lock. The source says this causes "any other thread that attempts to initiate shutdown to stall indefinitely". The first status wins: 130 | moot | [A]; the stall from source only | [Shutdown.java#L159-L170](https://github.com/openjdk/jdk21u/blob/jdk-21.0.12.1-ga/src/java.base/share/classes/java/lang/Shutdown.java#L159-L170); jv:M7, jv:M13 |
| RSpec | `exit!(1)`: exit 1, no summary, and no `ensure`, `after` or `at_exit` | moot | [A] | [runner.rb#L180-L182](https://github.com/rspec/rspec/blob/rspec-core-v3.13.6/rspec-core/lib/rspec/core/runner.rb#L180-L182); rb:M1 |

- **Two runners tell the tester on the first Ctrl-C what the second will do.** RSpec: "Interrupt again to force quit". Vitest watch: "Press CTRL+c again to exit forcefully." nextest prints `Killing due to second signal` only when the second arrives. [A]
- **No runner gives the second signal a gentler meaning than the first.** Where it is not moot, it aborts the teardown that runs (pytest, xdist, unittest `-c`), kills every child (nextest), or exits at once (RSpec, Vitest watch). `go test` is the only one that ignores it, and a test that ignores SIGINT then runs to its end (go:G4). [A]

## 4. SIGTERM and SIGHUP

| runner | SIGTERM | SIGHUP | tier | source, probe |
|---|---|---|---|---|
| pytest | no handler: dies, 143, no `finally`, no teardown, no output. `src/_pytest` has no `import signal` | the same, 129 | [A] | py:S9 grep; py:M5, py:M6 |
| pytest-xdist | no handler in xdist or execnet: all processes die, 143, no worker left | the same, 129 | [A] | py:M10, py:M25 |
| unittest, with or without `-c` | dies, 143. `installHandler` touches only SIGINT | dies, 129 | [A] | [signals.py#L49-L54](https://github.com/python/cpython/blob/v3.15.2/Lib/unittest/signals.py#L49-L54); py:M16, py:M17, py:M18 |
| Ward | dies, 143 | dies, 129 | [A] | py:M21, py:M26 |
| cargo-nextest | **as SIGINT**: forwards SIGTERM to each test's group, prints `Cancelling due to signal`, exit 100 | **as SIGINT**: forwards SIGHUP; also SIGQUIT | [A] | [signal.rs#L108-L116](https://github.com/nextest-rs/nextest/blob/cargo-nextest-0.9.151/nextest-runner/src/signal.rs#L108-L116), [unix.rs#L219-L232](https://github.com/nextest-rs/nextest/blob/cargo-nextest-0.9.151/nextest-runner/src/runner/unix.rs#L219-L232); rs:M2, rs:M3, rs:M4 |
| `cargo test` | dies, 143 | dies, 129 | [A] | rs:L3, rs:L4 |
| `go test` | **not** as SIGINT: `go` handles only `os.Interrupt` and SIGQUIT, so both processes die, 143, and no `FAIL` line | dies, 129 | [A] | [signal_unix.go#L14](https://github.com/golang/go/blob/go1.27.3/src/cmd/go/internal/base/signal_unix.go#L14); go:G2, go:G3 |
| Jest | dies, 143 | dies, 129 | [A] | js:J3, js:J4 |
| Vitest `run` | **as SIGINT**: `process.once('SIGTERM', onExit)`, exit 143 | no handler: dies, 129 | [A] | [logger.ts#L331-L332](https://github.com/vitest-dev/vitest/blob/v5.1.0/packages/vitest/src/node/logger.ts#L331-L332); js:V4, js:V5 |
| mocha | no handler: dies, 143, in every mode | dies, 129 (serial); `--parallel` not probed | [A]; parallel SIGHUP [C] | js:K3, js:K4, js:K9 |
| any JVM: Gradle client, Maven, the console launcher | the same handler as SIGINT: shutdown hooks, then 143. The `java` man page: "The JVM uses SIGHUP, SIGINT, and SIGTERM to initiate the running of shutdown hooks." | the same: 129 | [A] | [Terminator.java#L59-L67](https://github.com/openjdk/jdk21u/blob/jdk-21.0.12.1-ga/src/java.base/unix/classes/java/lang/Terminator.java#L59-L67), [java man page, JDK 21, `-Xrs`](https://docs.oracle.com/en/java/javase/21/docs/specs/man/java.html); jv:M5, jv:M13, jv:M14 |
| RSpec | **not trapped, yet gentler than nothing**: Ruby raises `SignalException`, which RSpec does not rescue. `ensure`, `after`, `after(:all)`, `after(:suite)` and `at_exit` ran, and the summary printed. Ruby then re-raises the signal: 143 | the same: 129 | [A] | [support.rb#L145-L153](https://github.com/rspec/rspec/blob/rspec-core-v3.13.6/rspec-support/lib/rspec/support.rb#L145-L153), [signal.c#L1068-L1097](https://github.com/ruby/ruby/blob/76cca827ab52ab1d346a728f068d5b8da3e2952b/signal.c#L1068-L1097), [eval.c#L267](https://github.com/ruby/ruby/blob/76cca827ab52ab1d346a728f068d5b8da3e2952b/eval.c#L267); rb:M1 |

- **Only three runners do anything on SIGTERM but die: nextest, Vitest `run` and RSpec.** nextest forwards it and keeps its summary. Vitest catches it only to exit 143 at once, with no teardown. RSpec runs every hook, because Ruby unwinds the stack for any signal it turns into an exception. [A]
- **Every Python runner here treats SIGTERM as a kill.** pytest, xdist, unittest and Ward run their teardown on SIGINT, because Python raises `KeyboardInterrupt`. SIGTERM's default action ends the process before any Python code runs. [A]

## 5. Tests in other processes: who gets the SIGINT, and what the runner sends

| runner | where tests run | in the terminal's process group? | what the runner sends each child | tier | source, probe |
|---|---|---|---|---|---|
| cargo-nextest | a process per test | **no**: `cmd.process_group(0)` gives each its own group, so only nextest gets the terminal's SIGINT | the signal it got, to each group; SIGKILL after the grace, or at once on the 2nd signal. `cargo nextest` exec-replaces cargo, so no cargo process is left | [A] | [unix.rs#L32-L34](https://github.com/nextest-rs/nextest/blob/cargo-nextest-0.9.151/nextest-runner/src/runner/unix.rs#L32-L34), [signal-handling.md#L46-L56](https://github.com/nextest-rs/nextest/blob/cargo-nextest-0.9.151/site/src/docs/design/architecture/signal-handling.md#L46-L56); rs:M1, rs:M8 |
| pytest-xdist | execnet popen workers | **yes**: plain `subprocess.Popen`, no new session | `GATEWAY_TERMINATE`, then up to 10 s, then SIGKILL (`Popen.kill`). The worker, on that message, waits 5 s, **sends itself SIGINT**, waits 10 s, then `os._exit(1)`. The 5 s self-SIGINT aborted a slow teardown (py:M8b); the 10 s SIGKILL was not seen | [A]; SIGKILL from source | [gateway_io.py#L34](https://github.com/pytest-dev/execnet/blob/v2.1.2/src/execnet/gateway_io.py#L34), [multi.py#L331-L349](https://github.com/pytest-dev/execnet/blob/v2.1.2/src/execnet/multi.py#L331-L349), [gateway_base.py#L1235-L1253](https://github.com/pytest-dev/execnet/blob/v2.1.2/src/execnet/gateway_base.py#L1235-L1253), [dsession.py#L146-L166](https://github.com/pytest-dev/pytest-xdist/blob/v3.8.0/src/xdist/dsession.py#L146-L166); py:M7, py:M8b |
| `cargo test` | the test binary, a child of cargo; tests are threads | **yes**: the binary's pgid is cargo's pid | nothing: cargo waits on the child | [A] | [process_builder.rs#L259-L271](https://github.com/rust-lang/cargo/blob/0.100.0/crates/cargo-util/src/process_builder.rs#L259-L271); rs:L1 |
| `go test` | a test binary per package, a child of `go` | **yes** | nothing on an interrupt; SIGQUIT only when the kill timeout fires | [A] | [signal_unix.go#L16-L18](https://github.com/golang/go/blob/go1.27.3/src/cmd/go/internal/base/signal_unix.go#L16-L18); go:G1 |
| Jest | `child_process.fork` workers | **yes**: no `detached` | nothing on a signal. On a normal end: an end message, `forceExit()` after 500 ms, which sends SIGTERM, then SIGKILL 500 ms later | [A] | [ChildProcessWorker.ts#L131-L146](https://github.com/jestjs/jest/blob/v30.5.2/packages/jest-worker/src/workers/ChildProcessWorker.ts#L131-L146), [ChildProcessWorker.ts#L473-L487](https://github.com/jestjs/jest/blob/v30.5.2/packages/jest-worker/src/workers/ChildProcessWorker.ts#L473-L487); js:J6, js:J10 |
| Vitest | `forks` pool, the default | **yes**: no `detached` | nothing under `run`. A normal `stop()` sends SIGTERM, then SIGKILL after 500 ms | [A] | [forksWorker.ts#L60-L66](https://github.com/vitest-dev/vitest/blob/v5.1.0/packages/vitest/src/node/pools/workers/forksWorker.ts#L60-L66), [forksWorker.ts#L83-L103](https://github.com/vitest-dev/vitest/blob/v5.1.0/packages/vitest/src/node/pools/workers/forksWorker.ts#L83-L103); js:V6, js:V10 |
| mocha | in process; a child when a node flag is given; workerpool processes with `--parallel` | **yes**, for both | the bin sends its child SIGINT and then SIGTERM. `--parallel` calls `pool.terminate(true)`: a terminate message, then SIGTERM after 1000 ms | [A] | [bin/mocha.js#L141-L160](https://github.com/mochajs/mocha/blob/v12.0.3/bin/mocha.js#L141-L160), [WorkerHandler.js#L517-L528](https://github.com/josdejong/workerpool/blob/v10.0.3/src/WorkerHandler.js#L517-L528), [Pool.js#L53](https://github.com/josdejong/workerpool/blob/v10.0.3/src/Pool.js#L53); js:K5, js:K7, js:K11, js:K12 |
| JUnit 5 under Gradle | a worker JVM, a child of the daemon | **no**: the daemon detaches (`processEnvironment.maybeDetachProcess()`) and has its own session; the worker is in it. Ctrl-C reaches only the client. `--no-daemon` still forked a single-use daemon in its own session | the daemon, on the client's disconnect, interrupts the task and calls `execHandle.abort()`, so `Process.destroy()`, so SIGTERM. A daemon not idle 10 s later stops itself | [A]; the worker's SIGTERM from source and its hook, not straced | [DaemonMain.java#L86-L88](https://github.com/gradle/gradle/blob/v9.8.2/platforms/core-runtime/daemon-server/src/main/java/org/gradle/launcher/daemon/bootstrap/DaemonMain.java#L86-L88), [CancelExecutionStep.java#L38-L50](https://github.com/gradle/gradle/blob/v9.8.2/platforms/core-execution/execution/src/main/java/org/gradle/internal/execution/steps/CancelExecutionStep.java#L38-L50), [ProcessHandleImpl_unix.c#L312](https://github.com/openjdk/jdk21u/blob/jdk-21.0.12.1-ga/src/java.base/unix/native/libjava/ProcessHandleImpl_unix.c#L312), [DaemonStateCoordinator.java#L270-L312](https://github.com/gradle/gradle/blob/v9.8.2/platforms/core-runtime/launcher/src/main/java/org/gradle/launcher/daemon/server/DaemonStateCoordinator.java#L270-L312); jv:M4, jv:M6 |
| JUnit 5 under Surefire | a forked JVM, `forkCount=1` | **yes**: the fork's pgid is `mvn`'s pid; the `mvn` script `exec`s java | Maven's `cli-shutdown-hook` calls `process.destroy()`: strace saw `kill(<fork>, SIGTERM)` 6 ms after the SIGINT. A second hook sends a `SHUTDOWN` command; its `shutdown` mode (`exit`, `testset`, `kill`) made no difference in the probes | [A]; one strace sample | [CommandlineExecutor.java#L75-L124](https://github.com/apache/maven-surefire/blob/surefire-3.6.1/surefire-extensions-api/src/main/java/org/apache/maven/surefire/extensions/util/CommandlineExecutor.java#L75-L124), [ForkedBooter.java#L302-L388](https://github.com/apache/maven-surefire/blob/surefire-3.6.1/surefire-booter/src/main/java/org/apache/maven/surefire/booter/ForkedBooter.java#L302-L388); jv:M11, jv:M12 |
| pytest, unittest, Ward, RSpec | one process | — | — | [A] | ps in py:M1, py:M12, py:M19, rb:M1 |

- **Surefire's own pages disagree on its default.** The mojo says `shutdown=exit` "by default (changed in version 3.0.0-M4)" ([SurefireMojo.java#L416-L433](https://github.com/apache/maven-surefire/blob/surefire-3.6.1/maven-surefire-plugin/src/main/java/org/apache/maven/plugin/surefire/SurefireMojo.java#L416-L433)). The site page says "By default (configuration parameter `shutdown=testset`)" ([shutdown.md.vm#L62](https://github.com/apache/maven-surefire/blob/surefire-3.6.1/maven-surefire-plugin/src/site/markdown/examples/shutdown.md.vm?plain=1#L62)). [A]
- **JUnit 6 has a cancellation token, and no signal reaches it.** In the shipped code, the only caller of `cancel()` is the console's `FailFastListener` ([FailFastListener.java#L34](https://github.com/junit-team/junit-framework/blob/r6.1.3/junit-platform-console/src/main/java/org/junit/platform/console/command/FailFastListener.java#L34)). [A]

- **When the children share the group, the runner's forwarding is never tested by a terminal.** Each worker gets its own SIGINT first. With a SIGINT to the parent alone, as a CI that signals one PID does, the result changes. Under Jest and Vitest `run`, the orphaned workers ran their tests to the end, with every hook (js:J10, js:V10). Under xdist, a test that was queued started **after** the interrupt (py:M11). [A]
- **nextest is the only runner here that decides what each test receives.** The earlier survey called this "an event the dispatcher forwards". The probe shows its cost: the test gets SIGINT and dies with no `Drop`, because a Rust test has no handler (rs:M1). [A]

## 6. How a CI cancel delivers its signals

| CI | first signal | to whom | then | grace before SIGKILL | configurable? | tier | source |
|---|---|---|---|---|---|---|---|
| GitHub Actions (runner v2.338.2, Unix) | SIGINT | **one PID**: the step's entry process (`bash` for `run:`). `kill(_proc.Id, sig)`, and the child gets no group of its own | SIGTERM to the same PID after 7.5 s; `Process.Kill()` after 2.5 s more | 10 s in all; the SIGKILL also reaches only that PID. Orphans are killed at job end | no: `private readonly` constants | [A] | [ProcessInvoker.cs#L32-L33](https://github.com/actions/runner/blob/v2.338.2/src/Runner.Sdk/ProcessInvoker.cs#L32-L33), [#L443-L465](https://github.com/actions/runner/blob/v2.338.2/src/Runner.Sdk/ProcessInvoker.cs#L443-L465), [#L829-L834](https://github.com/actions/runner/blob/v2.338.2/src/Runner.Sdk/ProcessInvoker.cs#L829-L834), [#L855-L863](https://github.com/actions/runner/blob/v2.338.2/src/Runner.Sdk/ProcessInvoker.cs#L855-L863) |
| GitHub docs | "`SIGINT/Ctrl-C` to the step's entry process" | as above | "`SIGTERM/Ctrl-Break`", then "kills the process tree" | 7500 ms + 2500 ms; then a 5 min server-side force | — | [A] | [workflow-cancellation.md#L21-L22](https://github.com/github/docs/blob/4a20c763a48bd4f5a34a84c002fd9858e9793f65/content/actions/reference/workflows-and-actions/workflow-cancellation.md#L21-L22) |
| Azure Pipelines agent v5.281.1 | SIGINT | one PID | SIGTERM after 7.5 s; kill after 2.5 s | 10 s | **yes**: `PROCESS_SIGINT_TIMEOUT`, `PROCESS_SIGTERM_TIMEOUT`, `USE_GRACEFUL_PROCESS_SHUTDOWN` | [A] | [ProcessInvoker.cs#L34-L35](https://github.com/microsoft/azure-pipelines-agent/blob/v5.281.1/src/Agent.Sdk/ProcessInvoker.cs#L34-L35), [AgentKnobs.cs#L335-L354](https://github.com/microsoft/azure-pipelines-agent/blob/v5.281.1/src/Agent.Sdk/Knob/AgentKnobs.cs#L335-L354) |
| GitLab Runner v19.5.2, shell executor | SIGTERM | **the process group**: `syscall.Kill(-pid, …)`; the job shell starts with `Setpgid: true` | SIGKILL to the group | **10 min** (`GracefulTimeout`); then `after_script` runs, 5 min by default | no, for the shell executor | [A] | [killer_unix.go#L21-L52](https://gitlab.com/gitlab-org/gitlab-runner/-/blob/v19.5.2/helpers/process/killer_unix.go#L21-52), [killer.go#L15-L21](https://gitlab.com/gitlab-org/gitlab-runner/-/blob/v19.5.2/helpers/process/killer.go#L15-21), [shell.md#L108-L112](https://gitlab.com/gitlab-org/gitlab-runner/-/blob/v19.5.2/docs/executors/shell.md#L108-112) |
| GitLab Runner v19.5.2, docker executor | SIGTERM | **every PID in the container but PID 1**, newest first, by a `docker exec` script | `docker stop`: the stop signal to PID 1, then SIGKILL | ≤ 10 s for the script, then the container's stop timeout | `FF_USE_NATIVE_CONTAINER_STOP` skips the script | [A] | [docker.go#L1285-L1296](https://gitlab.com/gitlab-org/gitlab-runner/-/blob/v19.5.2/executors/docker/docker.go#L1285-1296), [bash.go#L48-L58](https://gitlab.com/gitlab-org/gitlab-runner/-/blob/v19.5.2/shells/bash.go#L48-58) |
| Buildkite agent v4.9.0 | `cancel-signal`, SIGTERM by default | the process group, in two hops: agent to bootstrap, bootstrap to the command's own group | SIGKILL to the group | `cancel-signal-timeout`, **10 s** by default | **yes**, both | [A] | [signal_notwindows.go#L15-L58](https://github.com/buildkite/agent/blob/v4.9.0/internal/process/signal_notwindows.go#L15-L58), [lifecycle.md#L14-L18](https://github.com/buildkite/docs/blob/0cdde8e33a9d9e3bfdb974958cb2407b7126ae46/pages/agent/lifecycle.md#L14-L18) |
| `docker stop` (docker/cli v29.9.3) | SIGTERM, or the image's `STOPSIGNAL` | the main process, PID 1 | SIGKILL | 10 s on Linux | yes | [A] | [container_stop.md#L22-L25](https://github.com/docker/cli/blob/v29.9.3/docs/reference/commandline/container_stop.md#L22-L25) |
| Kubernetes pod deletion | TERM | the main process of each container | KILL to any process still running | 30 s, `terminationGracePeriodSeconds` | yes | [A] | [pod-lifecycle.md#L853-L863](https://github.com/kubernetes/website/blob/4e18137c4688b2a3a2f5be79867cae177da5125e/content/en/docs/concepts/workloads/pods/pod-lifecycle.md#L853-L863) |
| GNU `timeout` (coreutils v9.12) | TERM | the child **and** its own group (`kill(0, sig)`); with `--foreground`, the child only | SIGKILL only with `-k` | **none** by default | yes | [A] | [timeout.c#L89](https://github.com/coreutils/coreutils/blob/v9.12/src/timeout.c#L89), [timeout.c#L218-L251](https://github.com/coreutils/coreutils/blob/v9.12/src/timeout.c#L218-L251) |
| Jenkins | — | — | — | — | — | [C]: not researched | — |

- **GitHub's docs and code disagree.** The docs say the runner "kills the process tree" ([L21](https://github.com/github/docs/blob/4a20c763a48bd4f5a34a84c002fd9858e9793f65/content/actions/reference/workflows-and-actions/workflow-cancellation.md#L21)). On Unix, `NixKillProcessTree` calls `_proc?.Kill()` on the one PID ([ProcessInvoker.cs#L855-L861](https://github.com/actions/runner/blob/v2.338.2/src/Runner.Sdk/ProcessInvoker.cs#L855-L861)), and .NET's `Kill()` is `kill(pid, SIGKILL)` ([Process.Unix.cs#L59-L76](https://github.com/dotnet/runtime/blob/v8.0.33/src/libraries/System.Diagnostics.Process/src/System/Diagnostics/Process.Unix.cs#L59-L76)). [A]
- **Under a `run:` step, a test runner may get no signal before SIGKILL.** A mimic of GitHub's delivery signalled only the `bash` PID, with INT and then TERM (ci:M1). Bash ignored the SIGINT while it waited on its child. The SIGTERM killed only bash, exit 143. The child, standing in for the runner, logged no signal and kept running. This imitates the runner; it does not run it. [A] for the mimic, [C] for a real runner.
- **GitLab's docker executor exists because shells do not pass signals on.** Its comment: "This is necessary because most shells do not propagate signals to child processes." ([docker.go#L1288-L1290](https://gitlab.com/gitlab-org/gitlab-runner/-/blob/v19.5.2/executors/docker/docker.go#L1288-1290)). [A]

## What this means for purba

**This section is my interpretation, not a finding.** Each point stands on the whole survey, and names the rows it uses.

1. **An interrupted run needs a code that never reads as clean, and the survey offers no shared code.**
   The runners use four shapes. purba's 2 already means `unknown`, and 0, 1, 3, 4 and 125 are taken; 130 and 143 are free.

   | shape | what the shell sees | runners | example |
   |---|---|---|---|
   | die by the signal | 130 for SIGINT, 143 for SIGTERM; the parent sees a death by signal | on purpose, after unwinding: unittest (CPython), RSpec for SIGTERM (Ruby). By the default action, with no cleanup: `cargo test`, Jest, mocha serial | CPython's `exit_sigint`: `PyOS_setsig(SIGINT, SIG_DFL)`, then `kill(getpid(), SIGINT)` |
   | exit 128 + N as a code | 130 or 143; the parent sees a normal exit | Vitest `run`; every JVM | Vitest: `process.exitCode = 128 + exitCode`; the JVM: `Shutdown.exit(sig.getNumber() + 0200)` |
   | a code of its own | 2 | pytest, xdist | `ExitCode.INTERRUPTED = 2` |
   | the code of a failed run | 100, 1 | cargo-nextest, `go test`, RSpec, Vitest watch | nextest maps `RunOutcome::Cancelled` to the error of `Failed` |

   Only the first shape lets a parent tell an interrupt from an exit. CPython's comment names that reason: "a calling process such as a shell may not know about the user's ^C" (section 2).
   The integrators surveyed in [the research of exit codes](https://github.com/kalonji-tools/purba/blob/9b3fcc44897ffff09a876f00034adf6fe88a81dd/research/exit-codes.md?plain=1#L9-L13) read only zero against non-zero [B]. So every shape above fails their CI, and the choice matters to a shell or a person, and not to CI.
2. **Three runners exit 0 after an interrupt that left tests unrun. purba's exit-code record already refuses that shape.**
   unittest `-c`, Ward and mocha compute the code from the tests that finished, or lose it. RSpec counts an example that did not run as not passed, and exits 1. [The exit-code record](https://github.com/kalonji-tools/purba/blob/9051ab2/docs/decisions/a-run-exits-0-only-when-it-checked-something-and-every-test-passed.md#decision-outcome) allows 0 only when every test passed. A test that an interrupt kept from running did not pass. The design keeps that rule if it counts such a test.
3. **The running test: the survey shows three answers.**

   | answer | how | runners |
   |---|---|---|
   | stop it at once | the signal raises in the test, or kills the process | pytest, xdist, unittest, Ward, Jest, mocha, Vitest, `cargo test`, `go test`, JUnit under Gradle and Surefire; purba today |
   | let it finish, start no more | a handler sets a flag; the loop reads it between tests | RSpec (`wants_to_quit`), unittest `-c` (`shouldStop`) |
   | pass the signal on, wait a grace, then kill | the runner forwards the signal to the test's own group | cargo-nextest: 10 s by default, `slow-timeout.grace-period` |

   The second answer leaves a long test running for as long as it takes; a second Ctrl-C is then the only way out. The first is the default that CPython gives purba now.
4. **Teardown is what most runners lose.** Eight of thirteen run none of their own. pytest shows how to stop the test and still tear down: `KeyboardInterrupt` unwinds to the session, and the session's `finally` runs every finalizer (section 1). RSpec reaches the same end by letting the test finish. Both lose the teardown left on a second Ctrl-C.
5. **A second Ctrl-C forces the exit; a third is rare.** Every runner with a gentle first stage lets the second skip it. `go test` is the one counter-example: it ignores the second, and a test that ignores SIGINT then runs to its end (go:G4). Only cargo-nextest gives a third signal a meaning, a panic.
6. **SIGTERM matters more than SIGINT under CI.**
   - Five of the seven CI cancels and container stops send SIGTERM first. GitHub and Azure send SIGINT, to one PID only (section 6).
   - The grace before SIGKILL ranges from 2.5 s (GitHub, after its SIGTERM) and 10 s (Buildkite, `docker stop`) to 30 s (Kubernetes) and 10 min (GitLab's shell executor).
   - Every Python runner in the survey loses all teardown on SIGTERM.
   - RSpec shows the other way: a signal raised as an exception unwinds like SIGINT, and every hook runs.
7. **What prints: four runners count the run, and one counts what did not run.**
   - pytest counts the finished tests; its banner names the interrupt.
   - nextest counts both: `1/3 tests run` and `2/3 tests were not run due to interrupt`.
   - RSpec and Ward count only what finished.
   - unittest `-c` prints a normal `OK` with no mark of the interrupt.
   - Seven runners print no summary at all, because the signal killed them.
8. **Children and process groups belong to dispatch, out of this map's scope.**
   - Where children share the terminal's group, each gets the SIGINT itself: xdist, `cargo test`, `go test`, Jest, Vitest, mocha, Surefire.
   - Only cargo-nextest puts each test in a group of its own on purpose. Gradle's daemon does it as a side effect of detaching.
   - A CI that signals one PID, as GitHub does, leaves shared-group children unsignalled. In the probes they ran on, and under xdist a queued test started after the interrupt (py:M11, js:J10, js:V10).
   - [Each test runs in a process of its own, forked from a zygote that loaded its module once](https://github.com/kalonji-tools/purba/issues/694) meets this choice. A forked child keeps its parent's group unless it calls `setpgid`.

## What I could not establish, and the weakest claims

### The weakest [A] claims

1. **Under a GitHub `run:` step, the runner gets no signal before SIGKILL.** This rests on a bash mimic of the delivery (ci:M1), and on the runner's source. The real runner was not run. .NET's `Process.Kill()` was read at v8.0.33; the runtime that the shipped runner bundles was not checked.
2. **Vitest's watch mode exits 1.** Three runs show it. That `test-run.ts` overrides the 130 of `stdin.ts` is a reading of the source, not traced.
3. **mocha `--parallel` exits 0.** Reproduced, wrapped and unwrapped (js:K7, js:K15, js:K16). Why its `process.kill(process.pid, "SIGINT")` never fires is [C].
4. **Maven sends the fork SIGTERM before Surefire's `SHUTDOWN` command acts.** One strace sample, in `exit` mode. Two JVM shutdown hooks race in an order the JDK does not fix.
5. **Gradle's daemon sends the worker SIGTERM.** From the source chain, and the worker's shutdown hook ran. Not straced: the daemon is reparented and `ptrace_scope` is 1.
6. **unittest `-c` lets the test finish.** Probed with `time.sleep`, which CPython resumes after a handler that does not raise. A test blocked in another call may behave otherwise.
7. **Versions differ between source and probe.** `cargo test` ran 1.95.0 against 1.99.1 read. unittest ran CPython 3.15.0 against v3.15.2 read. pytest, xdist and Ward ran on CPython 3.13.15. RSpec ran on Ruby 3.4.9; v4.0.9 is the newest Ruby.
8. **Most probes ran once.** The second-signal probes of pytest, xdist and unittest `-c` aim at a 1.5 s teardown window. Their logs show a teardown start with no end, so the signal landed in it.
9. **`go test` without `-v` prints racily.** 2 of 3 runs printed nothing. The cause is inferred, not traced.
10. **RSpec exits 0 when the interrupt comes during the last example.** One sample; the source agrees.
11. **JUnit ships no signal handler and no shutdown hook.** A grep of `*.java` and `*.kt` for named APIs; reflection was not searched.
12. **The JavaScript probes ran under a wrapper** that ignored the signals and printed the runner's `(code, signal)`. Unwrapped reruns of Jest, Vitest and mocha gave the same exit codes.

### Could not establish [C]

- Jenkins: not researched.
- The exit code after cargo-nextest's third-signal panic. The second signal ended every probe first.
- A third SIGINT in any other runner: the second already ended the process.
- pytest: a second SIGINT during the terminal summary or `pytest_unconfigure`.
- Ward: where the second `KeyboardInterrupt` lands, and why no traceback prints.
- pytest-xdist: the 10 s SIGKILL path; execnet's 5 s self-SIGINT always ended the workers first. `--pdb` and looponfail.
- mocha: `--parallel` with SIGHUP; watch mode beyond its source.
- `npx` and `npm test` as wrappers. Vitest's `threads` pool and Jest's `workerThreads`.
- `go test` with several packages running at once (`-p` > 1).
- libtest under `-Zpanic-abort-tests`.
- Gradle with no forked daemon at all; Gradle's Tooling API cancel; its 10 s self-stop.
- Surefire with `forkCount` > 1 or `reuseForks=false`. The JVM's `-Xrs`.
- Windows, for every runner. nextest's Windows behaviour is from its design doc only.
- GitHub container actions and container jobs; whether `timeout-minutes` takes the same path as a cancel.
- GitLab's Kubernetes and custom executors, and its step-runner cancel path. The Buildkite race when both hops' timeouts expire together.

## Appendix A. Commands

Run 2026-11-04. `$R` is the scratch directory, `/tmp/claude-1000/-home-snregales-Projects-kalonji-tools/80ce23f8-cc7b-49a7-9068-91a4407b73c0/scratchpad/r1`.
Every line number in the tables comes from `grep -n` or `sed -n` on a fetched copy.

**The probe pattern.** Each cluster's `probe.sh` does the same steps:

```bash
set -m                                   # each background job gets its own pgid
<runner> > out.txt 2>&1 &  pid=$!        # pgid == pid
until [ -e started.one ]; do sleep 0.1; done; sleep 0.5
ps -o pid,pgid,ppid,args ...             # who is in the group
kill -<SIG> -- -$pid                     # repeated with a gap for a 2nd and 3rd signal
wait $pid; echo "exit=$?"
```

A probe marked "to one PID" sends `kill -<SIG> $pid` instead.

**Sources (S).**

| key | command |
|---|---|
| py:S | `git clone -q --depth 1 --branch <tag>` of pytest-dev/pytest `9.2.0`, pytest-dev/pytest-xdist `v3.8.0`, pytest-dev/execnet `v2.1.2`, darrenburns/ward `release/0.68.0b0`; `curl -sfL https://raw.githubusercontent.com/python/cpython/v3.15.2/<f>` for `Lib/unittest/{signals,main,suite,case,runner,result}.py`, `Lib/subprocess.py`, `Modules/main.c`, `Doc/library/unittest.rst` |
| py:S9 | `grep -rn "import signal\|from signal" src/_pytest \| wc -l` (0); `grep -rn "SIG[A-Z]" src/_pytest`; the same greps over xdist, execnet and ward; `grep -rn -i "ctrl-c\|keyboardinterrupt\|sigint\|sigterm"` over xdist's docs |
| rs:S | `curl -sfL https://raw.githubusercontent.com/nextest-rs/nextest/cargo-nextest-0.9.151/<path>` for `nextest-metadata/src/exit_codes.rs`, `nextest-runner/src/{signal.rs,runner/unix.rs,runner/dispatcher.rs,runner/executor.rs,config/elements/slow_timeout.rs}`, `site/src/docs/design/architecture/signal-handling.md`; `gh api "repos/rust-lang/rust/git/trees/1.99.1:library/test/src?recursive=1"`, each blob, then `grep -rn -i -E 'signal\|ctrlc\|ctrl_c\|sigint\|SIGTERM\|libc::'`; the cargo `0.100.0` tarball, then `grep -rn -E 'SIGINT\|ctrlc\|signal_hook\|sigaction\|process_group\|setsid' src crates/cargo-util` |
| go:S | `curl -sfL https://raw.githubusercontent.com/golang/go/go1.27.3/src/<path>` for `cmd/go/internal/base/{signal.go,signal_unix.go}`, `cmd/go/internal/test/test.go`, `cmd/go/internal/work/exec.go`, `testing/testing.go`, `testing/internal/testdeps/deps.go` |
| js:S | `curl -sfL https://codeload.github.com/<repo>/tar.gz/refs/tags/<tag> \| tar xz` for jestjs/jest `v30.5.2`, vitest-dev/vitest `v5.1.0`, mochajs/mocha `v12.0.3`, josdejong/workerpool `v10.0.3`; then `grep -rn -E "SIGINT\|SIGTERM\|SIGHUP\|SIGKILL\|process\.on\(\|process\.once\(\|detached"` over each runner's CLI, core and worker packages |
| jv:S | the junit-team/junit-framework `r6.1.3` tarball, then `grep -rn -E 'addShutdownHook\|sun\.misc\.Signal\|jdk\.internal\.misc\.Signal\|SignalHandler'`; the gradle/gradle `v9.8.2`, apache/maven-surefire `surefire-3.6.1` and apache/maven `maven-3.10.0` tarballs, then grep; `curl -sfL https://raw.githubusercontent.com/openjdk/jdk21u/jdk-21.0.12.1-ga/src/java.base/{unix/classes/java/lang/Terminator.java,share/classes/java/lang/Shutdown.java,unix/native/libjava/ProcessHandleImpl_unix.c}`; `curl -sfL https://docs.oracle.com/en/java/javase/21/docs/api/java.base/java/lang/Runtime.html` and `.../specs/man/java.html` |
| rb:S | `curl -sfL https://raw.githubusercontent.com/rspec/rspec/rspec-core-v3.13.6/rspec-core/lib/rspec/core/{runner,example_group,configuration}.rb` and `.../rspec-support/lib/rspec/support.rb`; `.../ruby/ruby/v3_4_9/{signal.c,eval.c}` |
| ci:S | `curl -sfL https://raw.githubusercontent.com/actions/runner/v2.338.2/src/Runner.Sdk/ProcessInvoker.cs`; `.../github/docs/4a20c763a48bd4f5a34a84c002fd9858e9793f65/content/actions/reference/workflows-and-actions/workflow-cancellation.md`; `.../dotnet/runtime/v8.0.33/src/libraries/System.Diagnostics.Process/src/System/Diagnostics/Process.Unix.cs`; `.../microsoft/azure-pipelines-agent/v5.281.1/src/Agent.Sdk/{ProcessInvoker.cs,ProcessInvoker.MacLinux.cs,Knob/AgentKnobs.cs}`; `curl -sfL https://gitlab.com/gitlab-org/gitlab-runner/-/raw/v19.5.2/<path>` for `helpers/process/{killer_unix,killer,job_unix}.go`, `executors/shell/shell.go`, `executors/docker/docker.go`, `shells/bash.go`, `docs/executors/shell.md`; `.../buildkite/agent/v4.9.0/internal/process/{process,signal_notwindows}.go`; `.../buildkite/docs/0cdde8e33a9d9e3bfdb974958cb2407b7126ae46/pages/agent/lifecycle.md`; `.../docker/cli/v29.9.3/docs/reference/commandline/container_stop.md`; `.../kubernetes/website/4e18137c4688b2a3a2f5be79867cae177da5125e/content/en/docs/concepts/workloads/pods/pod-lifecycle.md`; `.../coreutils/coreutils/v9.12/{src/timeout.c,doc/coreutils.texi}` |
| L | `gh api repos/O/R/commits/<branch> --jq .commit.committer.date`; `gh api repos/O/R/releases/latest --jq '.tag_name+" "+.published_at'`; where no release exists, `git ls-remote --tags --refs` and `gh api repos/O/R/commits/<tag> --jq .commit.committer.date`; for GitLab, `curl https://gitlab.com/api/v4/projects/gitlab-org%2Fgitlab-runner/repository/{tags,commits/main}` |
| V | the coordinator re-fetched these spans and compared them to the tables: `ProcessInvoker.cs` L32-33, L443-465, L829-834, L855-863; `workflow-cancellation.md` L21-22; buildkite `signal_notwindows.go` L51-57; nextest `unix.rs` L210-234, `slow_timeout.rs` L74-76, `run.rs` L1650-1655; go `signal.go` L13-31; CPython `result.py` L172-179, `main.c` L792-815 and L848-850; pytest `main.py` L339-376; execnet `gateway_base.py` L1235-1253; vitest `logger.ts` L316-334, `stdin.ts` L85-96, `test-run.ts` L160-175; rspec `runner.rb` L40-47 and L175-200; jdk21u `Terminator.java` L45-68 |

**Probes (M).** Each runs `$R/<cluster>/probe.sh KEY FIXTURE SIGNALS GAP <command>`.

| keys | runner command | signals |
|---|---|---|
| py:M1–M6 | `$R/py/venv-pt/bin/python -m pytest -p no:cacheprovider test_x.py` (pytest 9.2.0, CPython 3.13.15) | M1 INT; M2 INT,INT gap 0.7 s; M3 INT in test two, `-v`; M4 INT, `--full-trace`; M5 TERM; M6 HUP |
| py:M7–M11, M24, M25 | the same with `-n 2 -v` (pytest-xdist 3.8.0, execnet 2.1.2) | M7 INT; M8 INT, 15 s session teardown; M8b M8 with `EXECNET_DEBUG=2`; M9 INT,INT 0.7 s; M10 TERM; M11 INT to one PID; M25 HUP |
| py:M12–M18 | `python3.15 -m unittest [-c] -v test_u` (CPython 3.15.0) | M12 INT; M13 `-c` INT; M14 `-c` INT,INT 0.7 s; M15 `-c` INT,INT 3.0 s; M16 TERM; M17 HUP; M18 `-c` TERM |
| py:M19–M22, M26 | `$R/py/venv-ward/bin/ward --path test_w.py` (ward 0.68.0b0) | M19 INT; M20 INT in test two; M21 TERM; M22 INT,INT 0.01 s and 0.03 s; M26 HUP |
| py:M23, M24 | `rc.py`: starts the command with `start_new_session=True`, sends `os.killpg(pid, SIGINT)` N times, prints `Popen.returncode` | returncode −2 means death by SIGINT |
| rs:M1–M8 | `cargo nextest run -j1 --no-fail-fast` (cargo-nextest 0.9.151 release binary, checksum checked), under `nix-shell -p cargo rustc` | M1 INT; M2 TERM; M3 HUP; M4 QUIT; M5 INT, test ignores SIGINT; M6 INT,INT 1 s; M7 INT×3 0.3 s; M8 INT with default `-j` |
| rs:L1–L6 | `cargo test -- --test-threads=1` (cargo and rustc 1.95.0) | L1 INT; L2 INT, default threads; L3 TERM; L4 HUP; L5 INT, test ignores SIGINT; L6 the test binary alone |
| go:G1–G7 | `go test -count=1 -v ./a` (go1.27.3 from go.dev, checksum checked) | G1 INT; G2 TERM; G3 HUP; G4 INT×3 0.5 s, test ignores SIGINT; G5 `-p 1 ./...` INT; G6, G6b, G6c no `-v`, INT; G7 G5 with the test ignoring SIGINT |
| js:J1–J10 | `node $R/js/w/node_modules/jest/bin/jest.js [--maxWorkers=2]` (jest 30.5.2, node 24.21.0) | J1 INT; J2 INT,INT; J3 TERM; J4 HUP; J6–J9 the same with workers; J10 INT to one PID |
| js:V1–V10 | `node .../vitest/vitest.mjs run` (vitest 5.1.0) | V1 INT; V2 INT,INT; V3 INT×3; V4 TERM; V5 HUP; V6–V9 three files; V10 INT to one PID |
| js:K1–K17 | `node .../mocha/bin/mocha.js --timeout 10000 [--node-option no-warnings] [--parallel --jobs 2] [--posix-exit-codes]` (mocha 12.0.3) | K1 INT; K2 INT,INT; K3 TERM; K4 HUP; K5 INT with a node flag; K7 INT `--parallel`; K10 K7 with `DEBUG`; K11, K12 INT to one PID; K14 `--posix-exit-codes` with a node flag; K16 `--posix-exit-codes --parallel` |
| js:P1–P9 | on a pseudo-terminal: `printf '\003' \| timeout 35 script -qefc "<cmd>"` | P1 `vitest` (watch) one ^C; P3 `vitest run`; P4 jest; P5 mocha; P6, P7 watch with a slow `afterEach`, one and two ^C; P8 watch, three files; P9 `jest --watchAll` |
| jv:M4–M8 | `./gradlew [--no-daemon] test --console=plain` (Gradle 9.8.2 wrapper, JDK 21.0.12.1), then grep of the daemon's log; `./gradlew --stop` at the end | M4 INT; M5 TERM, HUP; M6 `--no-daemon` INT; M7 INT,INT 0.1 s |
| jv:M10–M13 | `mvn -B -o -Dmaven.repo.local=$R/jvmrb/m2 test` (Maven 3.10.0, Surefire 3.6.1) | M10 INT; M11 INT to one PID, with `-Dsurefire.shutdown=exit\|testset\|kill`; M12 `strace -f -tt -e trace=kill,tgkill,exit_group` and INT to the `mvn` PID; M13 INT,INT, TERM, HUP |
| jv:M14 | `java -jar junit-platform-console-standalone-6.1.3.jar execute --details=tree --select-class probe.ThreeTest` | INT, TERM, HUP |
| rb:M1, M2 | `$R/jvmrb/gem/bin/rspec` (rspec-core 3.13.6, Ruby 3.4.9) | M1 INT, INT,INT, TERM, HUP; M2 INT during example three |
| ci:M1 | `bash -e step.sh`, whose `child.sh` traps and logs INT, TERM and HUP; `kill -INT <bash pid>`, then 1.5 s later `kill -TERM <bash pid>` | a mimic of GitHub's delivery |
| ci:M2 | `timeout [--foreground] [-k 1] 1 bash -c '... & wait'` (coreutils 9.11) | the default, `--foreground` and `-k` |

## Appendix B. Versions

Sources were read at the newest release tag. Probes ran the version named.

| runner or system | source read | version probed |
|---|---|---|
| pytest | 9.2.0 | 9.2.0, CPython 3.13.15 |
| pytest-xdist, execnet | v3.8.0, v2.1.2 | 3.8.0, 2.1.2 |
| unittest | CPython v3.15.2 | CPython 3.15.0 |
| Ward | release/0.68.0b0, a beta; linked at its commit f41c6164 | 0.68.0b0 |
| cargo-nextest | cargo-nextest-0.9.151 | 0.9.151 |
| `cargo test` | rust 1.99.1, cargo 0.100.0 | cargo and rustc 1.95.0 |
| `go test` | go1.27.3 | go1.27.3 |
| Jest | v30.5.2 | 30.5.2 (jest-worker 30.5.1), node 24.21.0 |
| Vitest | v5.1.0 | 5.1.0 |
| mocha, workerpool | v12.0.3, v10.0.3 | 12.0.3, 10.0.3 |
| JUnit | r6.1.3, the newest release of the platform the issue calls JUnit 5 | 6.1.3, JDK 21.0.12.1 |
| Gradle | v9.8.2 | 9.8.2 |
| Maven, Surefire | maven-3.10.0, surefire-3.6.1 | 3.10.0, 3.6.1 |
| RSpec | rspec-core-v3.13.6 (its `runner.rb` is byte-identical at rspec-core-v4.0.0.beta1); Ruby v3_4_9, linked at its commit 76cca827 | 3.13.6, Ruby 3.4.9 |
| GitHub Actions runner | v2.338.2 | — |
| Azure Pipelines agent | v5.281.1 | — |
| GitLab Runner | v19.5.2 | — |
| Buildkite agent | v4.9.0 | — |
| coreutils `timeout` | v9.12 | 9.11 |
| bash | — | 5.3.9 |

## Appendix C. Liveness

Measured 2026-11-04 with the L commands.

| project | default branch, last commit | latest release, or tag read |
|---|---|---|
| pytest-dev/pytest | `main`, 2026-11-04 | release 9.2.0, 2026-10-27 |
| pytest-dev/pytest-xdist | `master`, 2026-11-02 | release v3.8.0, 2025-07-01 |
| pytest-dev/execnet | `main`, 2026-10-21 | no releases; tag v2.1.2, commit 2025-11-11 |
| python/cpython | `main`, 2026-11-04 | no releases; tag v3.15.2, commit 2026-10-25 |
| darrenburns/ward | `master`, 2024-07-05; **the repository is archived** | release 0.68.0b0, 2023-12-18 |
| nextest-rs/nextest | `main`, 2026-11-03 | release cargo-nextest-0.9.151, 2026-11-03 |
| rust-lang/rust | `main`, 2026-11-04 | release 1.99.1, 2026-10-16 |
| rust-lang/cargo | `master`, 2026-11-04 | no releases; tag 0.100.0, commit 2026-08-28 |
| golang/go | `master`, 2026-11-04 | no releases; tag go1.27.3, commit 2026-10-20 |
| jestjs/jest | `main`, 2026-11-02 | release v30.5.2, 2026-09-18 |
| vitest-dev/vitest | `main`, 2026-11-04 | release v5.1.0, 2026-11-02 |
| mochajs/mocha | `main`, 2026-10-27 | release v12.0.3, 2026-10-01 |
| josdejong/workerpool | `master`, 2026-07-30 | no releases; tag v10.0.3, commit 2026-07-01 |
| junit-team/junit-framework | `main`, 2026-11-03 | release r6.1.3, 2026-08-07 |
| gradle/gradle | `master`, 2026-11-03 | release v9.8.2, 2026-10-19 |
| apache/maven-surefire | `master`, 2026-11-02 | release surefire-3.6.1, 2026-10-26 |
| apache/maven | `master`, 2026-11-04 | release maven-3.10.0, 2026-10-01 |
| rspec/rspec (monorepo; rspec/rspec-core is archived) | `main`, 2026-10-20 | no releases; tag rspec-core-v3.13.6, commit 2025-10-19 |
| ruby/ruby | `master`, 2026-11-04 | release v4.0.9, 2026-10-23; v3_4_9 read, as probed |
| openjdk/jdk21u | `master`, 2026-10-27 | no releases; tag jdk-21.0.12.1-ga read, as probed |
| actions/runner | `main`, 2026-11-03 | release v2.338.2, 2026-10-26 |
| github/docs | `main`, 2026-11-03 | read at commit 4a20c763 |
| microsoft/azure-pipelines-agent | `master`, 2026-10-29 | release v5.281.1, 2026-10-23 |
| dotnet/runtime | `main`, 2026-11-04 | release v10.0.13, 2026-10-13; v8.0.33 read, which the runner's SDK 8.0.427 pins |
| gitlab-org/gitlab-runner | `main`, 2026-11-04 | tag v19.5.2, commit 2026-10-28 |
| buildkite/agent | `main`, 2026-11-04 | release v4.9.0, 2026-10-30 |
| buildkite/docs | `main`, 2026-11-04 | read at commit 0cdde8e3 |
| docker/cli | `master`, 2026-11-03 | no releases; tag v29.9.3, commit 2026-11-03 |
| kubernetes/website | `main`, 2026-11-04 | read at commit 4e18137c |
| coreutils/coreutils | `master`, 2026-11-04 | release v9.12, 2026-09-14 |

## Appendix D. Raw probe output, trimmed

Paths and timestamps are cut. `log:` is the fixture's log, in order.

```
py:M1   pytest INT            exit=2
        log: session setup | fn setup | one finally | fn teardown | session teardown start | session teardown end
        out: test_x.py
             !!!!!!!!!!!!!!!!!!!!!!!!!!!!!! KeyboardInterrupt !!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!
             .../test_x.py:7: KeyboardInterrupt
             (to show a full traceback on KeyboardInterrupt use --full-trace)
             ============================ no tests ran in 2.31s =============================
py:M2   pytest INT,INT 0.7 s  exit=130; log: ... fn teardown | session teardown start   (no end); no stats line
py:M5   pytest TERM           exit=143; log: session setup | fn setup; out ends "test_x.py "
py:M7   xdist -n 2 INT        exit=2; workers' PGID = the controller's PID
        log: one finally (worker A) | fn teardown, session teardown start (A and B) | session teardown end (A and B)
py:M8b  xdist, slow teardown  [ctl] gw0 --> sending GATEWAY_TERMINATE ...
                              [w] execution ongoing after 5 secs, trying interrupt_main
                              [w] sending ourselves a SIGINT
py:M11  xdist INT to one PID  exit=2; started: one, two, three (three started after the signal); out: "no tests ran in 7.92s"
py:M12  unittest INT          exit=130; log: one finally
py:M13  unittest -c INT       exit=0
        log: one body end | one finally | tearDown | cleanup | tearDownClass start | tearDownClass end | tearDownModule
        out: test_1_one (test_u.T.test_1_one) ... ok / Ran 1 test in 4.502s / OK
py:M19  ward INT              exit=3; log: global setup | fn setup | one finally
        out: Run cancelled - results for tests that ran shown below. / 0 Tests Encountered / NO TESTS FOUND in 0.64 seconds
py:M20  ward INT in test two  exit=0; out: PASS one / Run cancelled ... / 1 Passes (100.0%) / SUCCESS in 3.63 seconds

rs:M1   nextest INT           exit=100; the test's pgid is its own pid
          Cancelling due to interrupt: 1 test still running
              SIGINT [   0.739s] (1/3) sigfix tests::one
             Summary [   0.741s] 1/3 tests run: 0 passed, 1 failed, 0 skipped
        warning: 2/3 tests were not run due to interrupt
        error: test run failed
        log: one-start            (no one-drop)
rs:M5   nextest INT, test ignores SIGINT   exit=100 at 10.93 s; SIGKILL [  10.708s] (1/3) sigfix tests::one
rs:M6   the same, INT,INT 1 s              exit=100 at 1.94 s; Killing due to second signal: 1 test still running
rs:L1   cargo test INT        exit=130; the binary's pgid is cargo's pid; output ends "test tests::one ... "
rs:L5   cargo test INT, test ignores SIGINT   exit=130; the binary is reparented and runs one, three, two to the end
go:G1   go test -v INT        exit=1; the binary shares go's pgid
        === RUN   TestOne / signal: interrupt / FAIL	sigfix/a	0.618s
        log: testmain-before | one-start   (no defer, no cleanup, no testmain-after)
go:G4   go test, test ignores SIGINT, INT x3   exit=1 at 10.22 s; every test PASS; ok sigfix/a; defer, cleanup, testmain-after ran

js:J1   jest INT              code=null signal=SIGINT; out: (empty); log: start one
js:J10  jest, workers, INT to one PID   the workers ran on: end, finally, afterEach, afterAll for files two and three
js:V1   vitest run INT        code=130 signal=null; out: RUN v5.1.0 <root>; log: start one
js:V4   vitest run TERM       code=143 signal=null
js:K1   mocha INT             exit=130; log: start one
js:K7   mocha --parallel INT  exit=0; out: (blank); log: start two | start one
js:P1   vitest watch, one ^C  code=1
        Cancelling test run. Press CTRL+c again to exit forcefully.
        ↓ one 275ms [The test run was aborted by the user.]
        Test Files  1 passed (1) | Tests  3 skipped (3)
        CANCELLED  Test run cancelled. Watching for file changes...
        log: start one | afterEach one | afterAll
js:P7   vitest watch, slow afterEach, two ^C   code=130; log: start one | afterEach-begin one

jv:M4   gradle INT            exit=130 in 0.34 s; out ends "> Task :testClasses UP-TO-DATE"
        ps: client PGID=its PID; daemon PID=PGID=SID, reparented; worker in the daemon's session
        daemon log: ThreeTest > one() STARTED
                    thread 143: client disconnection detected, canceling the build
                    ThreeTest > one() SKIPPED
                    > Build cancelled while executing task ':test'
                    BUILD FAILED in 2s
jv:M10  mvn INT               exit=130; last line "[INFO] Running probe.ThreeTest"; log: jvm_start | shutdown_hook
jv:M12  strace, INT to mvn    --- SIGINT ... ---
                              kill(<fork>, SIGTERM) = 0            (6 ms later, from a mvn thread)
                              <fork> +++ exited with 143 +++ ; mvn +++ exited with 130 +++
jv:M14  console launcher      INT/TERM/HUP exit=130/143/129; out: (empty); log: jvm_start | shutdown_hook

rb:M1   rspec INT             exit=1
        RSpec is shutting down and will print the summary report... Interrupt again to force quit (warning: at_exit hooks will be skipped if you force quit).
        .
        Finished in 3 seconds (files took 0.09044 seconds to load)
        1 example, 0 failures
        log: body_end one | ensure one | after_each one | after_all | after_suite | at_exit
rb:M1   rspec INT,INT         exit=1; out: the banner only; log: (empty)
rb:M1   rspec TERM            exit=143; out: 1 example, 0 failures; log: ensure one | after_each one | after_all | after_suite | at_exit

ci:M1   bash -e step.sh, INT then TERM to the bash PID only
        shell alive after INT: yes
        shell alive after TERM: no; shell exit=143
        child alive after shell gone: yes   (the child logged no signal)
ci:M2   timeout, a child that ignores TERM
        default:      the grandchild got TERM; no SIGKILL; timeout exit=124 after 12.4 s
        --foreground: the grandchild got nothing
        -k 1 1:       the group got SIGKILL after 1 s; exit=137
```
