# What a test runner does when stdout refuses a write mid-run

This file answers [#686](https://github.com/kalonji-tools/purba/issues/686).
It is never merged.

## Answer

When the reader of the report quits in the middle of a run, almost every surveyed tool stops at once.
pytest, `unittest`, `cargo test`, `go test`, Jest and ripgrep end within one test (or one file, for Jest) of the refused write.
Vitest's main process also stops at once, but its worker processes outlive it and keep running tests after the shell already has the exit code.
cargo-nextest is the only one that stops on purpose in a slower way: it starts no new test, lets the running tests end, keeps trying to write, and exits with its own code, 110.
No surveyed tool ignores the refused write and runs to the end.
ripgrep did that in 2016, and its author called it a bug ([ripgrep#200](https://github.com/BurntSushi/ripgrep/issues/200)).
Only nextest gives a refused write its own exit code.
The others reuse a code that already means something: pytest, Jest and Vitest exit 1, and `cargo test` exits 101, which is their code for a failed test.
`unittest` exits 120, the interpreter's code for a failed final flush; `go test` dies of `SIGPIPE` (141).
ripgrep exits 1, its "no match" code, when it searches one file. When it searches a directory in parallel, it exits 0.
Only Go lets `SIGPIPE` end the process. Python, Rust and Node ignore the signal, so their tools see `EPIPE` as an error.
Three tools (unittest, nextest, Jest) write their report to stderr, so `| head` never refuses them; the case arises only with `2>&1`.
Where tests run in parallel, the number of tests that ran before the stop changes from run to run. Where they run one at a time, it changed by one test at most in five runs.
I found no tracker discussion, in any of the seven runners, of tests that keep running after the reader quits.

| tool (version run) | report stream | on a refused write mid-run | stderr | exit | `SIGPIPE` | each test's line written | tier |
|---|---|---|---|---|---|---|---|
| pytest 9.0.3 `-v` | stdout | stops at once: no test starts after the refused line | nothing | 1 (= tests failed) | ignored (Python) | as the test ends: flush after each report | [A] M1, S1 |
| `unittest` (CPython 3.13.15) `-v` | stderr | with `2>&1`: stops at once; with `\| head`: nothing is refused | nothing it can deliver; `lost sys.stderr` goes to the closed pipe | 120 (interpreter) | ignored (Python) | as the test starts and ends: flush after each write | [A] M2, S2 |
| `cargo test` 1.95.0 | stdout | stops at once; the process exits while other threads run tests | `error: io error when listing tests: … BrokenPipe`, `error: test failed, to rerun pass \`--lib\``, and 0–4 thread panics that change from run to run | 101 (= tests failed) | ignored (Rust) | as the test ends: flush after each write | [A] M3, S3, S4 |
| cargo-nextest 0.9.136 | stderr | with `2>&1`: starts no new test, lets running tests end, keeps writing; with `\| head`: nothing is refused | `Cancelling due to reporting error: 21 tests still running`, `warning: 250/300 tests were not run due to reporting error`, `error: error reporting results: error writing to output` (all to the closed pipe) | **110, its own** | ignored (Rust) | as the test ends | [A] M4, S5 |
| `go test` go1.26.8 `-v` | stdout | killed by `SIGPIPE` at the first refused write. Local directory mode: the test binary dies, then `go` dies writing `signal: broken pipe` to stdout. Package list mode: `go` dies first, then the test binary | nothing | 141 (signal) | ends the process | as the test runs: the test binary writes to the pipe itself, or `go` copies it as it comes | [A] M5, S6 |
| Jest 30.5.2 `--verbose` | stderr | with `2>&1`: crashes at once on Node's unhandled `EPIPE`; with `\| head`: nothing is refused | Node's `Error: write EPIPE` trace (to the closed pipe) | 1 (= tests failed) | ignored (Node) | when the whole test file ends | [A] M6, S7 |
| Vitest 5.0.5 `--reporter=verbose` | stdout | main process crashes on Node's unhandled `EPIPE`; **its workers keep running tests** | `Error: write EPIPE` trace, 27 lines | 1 (= tests failed) | ignored (Node) | when the main process gets the result; workers send at most every 100 ms | [A] M7, M8, S8 |
| ripgrep 15.1.0 (not a runner) | stdout | stops at once | nothing | 1 (= no match) for one file or `-j1`; 0 for a parallel directory search | ignored (Rust) | in blocks when stdout is not a terminal | [A] M9, S9 |

"Stops at once" means: no new unit of work starts after the refused write.
The rows argue from the whole survey in item 6, not from one row.

## 1. Stop, run on silently, or ignore: what each tool does

Each suite has 300 tests in 20 files of 15. Each test sleeps 10 ms, then appends its name to a file, so the file counts the tests that ran.
The reader is `head -n 40`, so the first refused write lands about 40 lines into the run, in the middle.
Each case ran five times. `ran` is the count when the shell had the exit code; `+3 s` is the count three seconds later.
A control run with stdout to a file ran all 300 tests and exited 0 for every runner; ripgrep's control also exited 0 (M1–M9).

| tool, command | exit (5 runs) | ran (5 runs) | ran 3 s later | stderr | tier |
|---|---|---|---|---|---|
| pytest `-v` | 1 ×5 | 36 ×5 | 36 ×5 | empty | [A] M1 |
| pytest (no `-v`; 27 lines in all) | 0 ×5 | 300 ×5 | — | empty; 27 lines never fill `head -n 40` | [A] M1 |
| `unittest -v`, `\| head` | 0 ×5 | 300 ×5 | — | the full report: stdout holds nothing | [A] M2 |
| `unittest -v`, `2>&1 \| head` | 120 ×5 | 41 ×5 | 41 ×5 | lost: it was the pipe | [A] M2 |
| `cargo test` (22 threads) | 101 ×5 | 44 ×5 | 44 ×5 | 13 to 26 lines: 4, 0, 1, 0 and 3 thread panics | [A] M3 |
| `cargo test -- --test-threads=1` | 101 ×5 | 39 ×5 | 39 ×5 | 13 lines ×5, no panic | [A] M3 |
| `cargo nextest run`, `\| head` | 0 ×5 | 300 ×5 | — | the full report: stdout holds nothing | [A] M4 |
| `cargo nextest run`, `2>&1 \| head` | 110 ×5 | 47, 51, 51, 49, 50 | the same | lost: it was the pipe | [A] M4 |
| `go test -v` (local directory mode) | 141 ×5 | 21 ×5 | 21 ×5 | empty | [A] M5 |
| `go test -v .` (package list mode) | 141 ×5 | 21, 22, 22, 21, 21 | 22 ×5 | empty | [A] M5 |
| `jest --verbose`, `\| head` | 0 ×5 | 300 ×5 | — | the full report: stdout holds nothing | [A] M6 |
| `jest --verbose`, `2>&1 \| head` | 1 ×5 | 60, 75, 75, 75, 75 | the same | lost: it was the pipe | [A] M6 |
| `vitest run` (default reporter; 29 lines in all) | 0 ×5 | 300 ×5 | — | empty | [A] M7 |
| `vitest run --reporter=verbose` | 1 ×5 | 200, 144, 232, 175, 176 | **285, 225, 300, 240, 300** | 27-line Node trace | [A] M7 |
| the same, `--no-file-parallelism` | 1 ×5 | 42 ×5 | **45 ×5** | the same trace | [A] M7 |
| `rg -n x big.txt` (2,000,000 matching lines) | 1 ×5 | — | — | empty | [A] M9 |

What each tool does after the refused write, read from strace (M1–M9) and the source:

- **pytest stops at once, and says nothing.** The refused write is the `PASSED … [ 12%]` of test 36. No later test starts. `_console_main` catches `BrokenPipeError`, points stdout at `/dev/null`, and returns 1 ([`config/__init__.py` L244-L261](https://github.com/pytest-dev/pytest/blob/9.1.1/src/_pytest/config/__init__.py#L244-L261)) [A]. It is the recipe of CPython's own docs, comment and all ([signal.rst L750-L778](https://github.com/python/cpython/blob/v3.15.2/Doc/library/signal.rst?plain=1#L750-L778)) [A]. It came from [pytest#7185](https://github.com/pytest-dev/pytest/pull/7185) (2020), which names `pytest | head -1` [A].
- **`unittest` stops at once, but the error escapes.** It has no handler for a refused write: `runner.py` and `main.py` hold no `BrokenPipeError`, `EPIPE` or `OSError` [A]. The `BrokenPipeError` leaves the run; its traceback, and Python's last-resort `lost sys.stderr`, go to the closed pipe; the final flush fails, and the interpreter exits 120 ([`main.c` L840-L844](https://github.com/python/cpython/blob/v3.15.2/Modules/main.c#L840-L844)) [A].
- **`cargo test` stops at once, and loses the tests in flight.** `run_tests_console` returns the error, and `test_main` prints it and calls `process::exit(101)` ([`lib.rs` L170-L181](https://github.com/rust-lang/rust/blob/1.99.1/library/test/src/lib.rs#L170-L181)) [A]. The text says "listing" even in a run: the same `eprintln!` serves `--list` at L133 [A]. Threads that end a test after the main thread stopped panic at `monitor_ch.send(message).unwrap()` ([1.95.0 `lib.rs` L701](https://github.com/rust-lang/rust/blob/1.95.0/library/test/src/lib.rs#L701); L771 at 1.99.1) [A]. cargo runs the test binary with stdout inherited, then adds `test failed, to rerun pass` ([`cargo_test.rs` L153](https://github.com/rust-lang/cargo/blob/0.100.0/src/ops/cargo_test.rs#L153), [L405-L411](https://github.com/rust-lang/cargo/blob/0.100.0/src/ops/cargo_test.rs#L405-L411)) [A].
- **nextest drains, by design.** The comment reads: "If the callback fails, we need to let the runner know to start shutting down. But we keep reporting results in case the callback starts working again." ([`runner/imp.rs` L522-L524](https://github.com/nextest-rs/nextest/blob/cargo-nextest-0.9.150/nextest-runner/src/runner/imp.rs#L522-L524)) [A]. The dispatcher turns it into a cancel with the reason `ReportError` ([`dispatcher.rs` L877-L879](https://github.com/nextest-rs/nextest/blob/cargo-nextest-0.9.150/nextest-runner/src/runner/dispatcher.rs#L877-L879)) [A]. In M4's strace run, it printed `Cancelling due to reporting error: 21 tests still running`. Those 21 tests ended, and nextest tried to write the line of each [A]. Its summary, also refused, read `warning: 250/300 tests were not run due to reporting error` [A].
- **`go test` is killed twice.** In local directory mode, the test binary writes to the pipe and dies at its first refused write. Then `go` prints the child's error, `signal: broken pipe`, to the same stdout ([`test.go` L1756-L1759](https://github.com/golang/go/blob/go1.27.3/src/cmd/go/internal/test/test.go#L1756-L1759)) and dies too [A].
- **In package list mode, `go` dies first.** `go` copies the test binary's output to stdout (item 3). Its 48-byte copy is the first refused write, and the test binary dies at its next write to `go` [A]. So one more test ended after the shell had the exit code, in 3 of 5 runs [A].
- **Jest stops at once, by crashing.** Its reporter writes with `process.stderr.write` ([`BaseReporter.ts` L24-L25](https://github.com/jestjs/jest/blob/v30.5.2/packages/jest-reporters/src/BaseReporter.ts#L24-L25)) [A]. A search of `packages/*/src` at v30.5.2 finds no `EPIPE` handling [A]. Node raises the stream error as an unhandled `'error'` event, and exits 1, "Uncaught Fatal Exception" ([process.md L4575-L4577](https://github.com/nodejs/node/blob/v24.21.0/doc/api/process.md?plain=1#L4575-L4577)) [A]. Its workers did not outlive it: the count three seconds later was the same [A].
- **Vitest's main process crashes; its workers run on.** The logger writes to `process.stdout` through a `Console` ([`logger.ts` L42-L45](https://github.com/vitest-dev/vitest/blob/v5.0.5/packages/vitest/src/node/logger.ts#L42-L45)) [A]. Vitest handles `EPIPE` only on the channel to a worker ([`forksWorker.ts` L125-L135](https://github.com/vitest-dev/vitest/blob/v5.0.5/packages/vitest/src/node/pools/workers/forksWorker.ts#L125-L135)), not on stdout [A]. After the crash, 12 to 16 `node` processes were alive, every one re-parented to `systemd --user`. They ran 67 to 87 more tests and were gone within 200 ms (M8) [A].
- **ripgrep stops at once; its exit code depends on the search path.** On a broken pipe, the one-thread loop `break`s before it records the match ([`main.rs` L126-L135](https://github.com/BurntSushi/ripgrep/blob/15.2.0/crates/core/main.rs#L126-L135)), so `search` returns `false` and ripgrep exits 1 [A]. The parallel path returns `WalkState::Quit` ([L204-L207](https://github.com/BurntSushi/ripgrep/blob/15.2.0/crates/core/main.rs#L204-L207)), and exits 0 [A]. Its man page says 1 "occurs only when no match was found" ([`template.rg.1` L135](https://github.com/BurntSushi/ripgrep/blob/15.2.0/crates/core/flags/doc/template.rg.1#L135)), so the one-thread exit contradicts it [A]. `main.rs` L40-L215 is the same at 15.1.0, the version run [A].

## 2. Does `SIGPIPE` end the process?

A process that `SIGPIPE` ends shows as exit 141 in the shell. Only `go test` showed 141.
Each test read the `SIGPIPE` bit of its ancestors from `/proc/<pid>/status` (M1–M7). The measuring shell did not ignore it (`SigIgn: 0000000000000004`, which is `SIGQUIT` only).

| tool | disposition measured | killed? | source | tier |
|---|---|---|---|---|
| pytest, `unittest` | `python3.13`: ignored | no | "`SIGPIPE` is ignored (so write errors on pipes and sockets can be reported as ordinary Python exceptions)" ([signal.rst L16-L21](https://github.com/python/cpython/blob/v3.15.2/Doc/library/signal.rst?plain=1#L16-L21)) | [A] |
| `cargo test` | test binary and `cargo`: ignored | no | "we set SIGPIPE to ignore when the program starts up" ([`sys/pal/unix/mod.rs` L28-L36](https://github.com/rust-lang/rust/blob/1.99.1/library/std/src/sys/pal/unix/mod.rs#L28-L36)); `SIG_IGN` before `main` unless `-Zon-broken-pipe` is used ([on-broken-pipe.md L20-L28](https://github.com/rust-lang/rust/blob/1.99.1/src/doc/unstable-book/src/compiler-flags/on-broken-pipe.md?plain=1#L20-L28)) | [A] |
| cargo-nextest | `cargo-nextest` and each test process: ignored | no | the same Rust default | [A] |
| `go test` | test binary and `go`: caught by the runtime | **yes**, both | "A write to a broken pipe on file descriptors 1 or 2 (standard output or standard error) will cause the program to exit with a SIGPIPE signal" ([os/signal doc.go L96-L102](https://github.com/golang/go/blob/go1.27.3/src/os/signal/doc.go#L96-L102)) | [A] |
| Jest, Vitest | main process and workers: ignored | no | "`'SIGPIPE'` is ignored by default" ([process.md L760](https://github.com/nodejs/node/blob/v24.21.0/doc/api/process.md?plain=1#L760)) | [A] |
| ripgrep | not measured by `/proc` | no: strace shows `EPIPE`, then `exit_group(1)` | "the Rust runtime doesn't ask for PIPE signals, and thus we get an I/O error instead" ([`main.rs` L47-L54](https://github.com/BurntSushi/ripgrep/blob/15.2.0/crates/core/main.rs#L47-L54)) | [A] |

- **Go's rule depends on the file descriptor.** On fd 1 or 2 the signal ends the program; on any other fd the write fails with `EPIPE` (doc.go L96-L102) [A].
- **Rust resets the signal for a child; Python, Go and Node were not checked.** Rust sets `SIG_DFL` before a child's `exec` (on-broken-pipe.md L20) [A]. The nextest test processes still showed "ignored", because each is a Rust program that sets it again [A].

## 3. Is each test's line written as it ends, or kept in a buffer?

The measure: a reader that quits at the first line naming a test, and the count of tests ended when that line arrived (M1–M9).
With tests in parallel, a whole first wave ends at once; one-thread runs separate buffering from parallelism.

| tool | tests ended when the first test line arrived | the buffer, and when it is written out | source | tier |
|---|---|---|---|---|
| pytest `-v` | 1 | Python block-buffers stdout to a pipe ([sys.rst L2208-L2212](https://github.com/python/cpython/blob/v3.15.2/Doc/library/sys.rst?plain=1#L2208-L2212)); the reporter calls `self.flush()` at the end of each report ([`terminal.py` L699](https://github.com/pytest-dev/pytest/blob/9.1.1/src/_pytest/terminal.py#L699), in `pytest_runtest_logreport` from L625) | [A] M1, S1 |
| pytest (no `-v`) | 15: one line per file | each `.` is its own `write(1, ".", 1)` (strace); the newline comes at the file's end | [A] M1 |
| `unittest -v` | 1 | stderr is line-buffered ([sys.rst L2209-L2210](https://github.com/python/cpython/blob/v3.15.2/Doc/library/sys.rst?plain=1#L2209-L2210)), and the result calls `flush()` after the start and after the status ([`runner.py` L58-L76](https://github.com/python/cpython/blob/v3.15.2/Lib/unittest/runner.py#L58-L76)) | [A] M2, S2 |
| `cargo test` | 22 (22 threads); 1 with `--test-threads=1` | `write_plain` flushes after every write ([`pretty.rs` L89-L93](https://github.com/rust-lang/rust/blob/1.99.1/library/test/src/formatters/pretty.rs#L89-L93)); `Stdout` is a `LineWriter` even on a pipe ([`stdio.rs` L612-L615](https://github.com/rust-lang/rust/blob/1.99.1/library/std/src/io/stdio.rs#L612-L615)) | [A] M3, S3 |
| cargo-nextest | 19 (in parallel) | strace shows one `write(2, …)` per `PASS` line | [A] M4 |
| `go test -v` | 1, in both modes | local directory mode: "Stream test output (no buffering)"; package list mode with one package and `-v`: echoed as it comes; with more packages: a `bytes.Buffer` per package, written when the package ends ([`test.go` L1467-L1509](https://github.com/golang/go/blob/go1.27.3/src/cmd/go/internal/test/test.go#L1467-L1509)). The one-package case "is intentionally undocumented" (L1476-L1478) | [A] M5, S6 |
| Jest `--verbose` | 15: one file | the verbose lines of a file print in `onTestResult`, when the file ends ([`VerboseReporter.ts` L75-L90](https://github.com/jestjs/jest/blob/v30.5.2/packages/jest-reporters/src/VerboseReporter.ts#L75-L90)) | [A] M6, S7 |
| Vitest `--reporter=verbose` | 92 (in parallel); 10 with `--no-file-parallelism` | workers send results at most every 100 ms (`throttle(sendTasksUpdate, 100)`, [`run.ts` L520](https://github.com/vitest-dev/vitest/blob/v5.0.5/packages/vitest/src/runtime/runner/run.ts#L520)); the reporter prints each test when its result arrives ([`verbose.ts` L16](https://github.com/vitest-dev/vitest/blob/v5.0.5/packages/vitest/src/node/reporters/verbose.ts#L16)) | [A] M7, S8 |
| Vitest (default reporter, not a terminal) | 227: one line per file | — | [A] M7 |
| ripgrep | — | "block buffering … is the default when ripgrep's stdout is redirected to a pipeline or a file" ([`defs.rs` L538-L543](https://github.com/BurntSushi/ripgrep/blob/15.2.0/crates/core/flags/defs.rs#L538-L543)); its first write was one 8,192-byte block, 841 lines (strace) | [A] M9, S9 |

- **Every runner writes each test's line, or file's lines, as the work ends.** None holds the whole report until the run ends. The buffering differs only in grain: a test, a file, or a 100 ms batch [A].
- **The grain sets where the stop lands.** Jest stopped at 60 or 75 tests, whole files of 15. pytest and `go test`, which write per test, stopped at the same test in all five runs [A].

## 4. Does any tracker discuss tests that keep running after the reader quits, or a stop point that changes?

Counts are GitHub search totals on 2026-10-27, and are estimates. Most hits are about sockets or worker IPC, not the report.

| query | count | what bears on the question | tier |
|---|--:|---|---|
| `repo:pytest-dev/pytest is:issue BrokenPipeError` | 7 | [#4375](https://github.com/pytest-dev/pytest/issues/4375) (2018): a broken pipe "fails two times loudly". Fixed by [#7185](https://github.com/pytest-dev/pytest/pull/7185) (2020), whose body cites `pytest \| head -1`. [#6564](https://github.com/pytest-dev/pytest/issues/6564): `pytest . \| less` raised `BrokenPipeError`; closed as a duplicate of #4375. None discusses tests that keep running | [A] C1 |
| `repo:pytest-dev/pytest is:issue "broken pipe"` | 6 | the same issues | [A] C1 |
| `repo:pytest-dev/pytest is:issue SIGPIPE` | 0 | — | [A] C1 |
| `repo:python/cpython is:issue unittest BrokenPipeError` | 16 | none about `unittest`'s report; all are other test suites or sockets (titles read) | [A] C1 |
| `repo:python/cpython is:issue SIGPIPE` | 41 | [#77731](https://github.com/python/cpython/issues/77731): docs now warn not to set `SIGPIPE` to `SIG_DFL` | [A] C1 |
| `repo:rust-lang/rust is:issue libtest "broken pipe"` | 1 | not about a closed reader | [A] C1 |
| `repo:rust-lang/rust is:issue "io error when listing tests"` | 1 | not about a closed reader | [A] C1 |
| `repo:rust-lang/rust is:issue "cargo test" "broken pipe"` | 15 | none about a closed reader (titles read) | [A] C2 |
| `repo:rust-lang/cargo is:issue "cargo test" "broken pipe"` | 5 | none about a closed reader (titles read) | [A] C1 |
| `repo:nextest-rs/nextest "broken pipe"` | 5 | [#2900](https://github.com/nextest-rs/nextest/issues/2900): quitting the pager during `cargo nextest list` printed `Broken pipe (os error 32)`. [#2901](https://github.com/nextest-rs/nextest/pull/2901), "don't fail on BrokenPipe errors from pager", fixed it for the pager | [A] C1 |
| `repo:nextest-rs/nextest EPIPE` / `SIGPIPE` | 0 / 0 | — | [A] C1 |
| `repo:golang/go is:issue "go test" "broken pipe"` | 75 | network test flakes; none about a closed reader (30 titles read) | [A] C1 |
| `repo:golang/go is:issue "signal: broken pipe"` | 21 | builder and cgo failures; none about `go test \| head` (15 titles read) | [A] C2 |
| `repo:jestjs/jest is:issue EPIPE` | 6 | the titles name workers, memory limits and `console.log`; none names the report or a closed reader (titles read) | [A] C1 |
| `repo:jestjs/jest is:issue "broken pipe"` | 0 | — | [A] C1 |
| `repo:vitest-dev/vitest is:issue EPIPE` | 6 | [#9014](https://github.com/vitest-dev/vitest/issues/9014): `write EPIPE` on the worker channel; the reporter saw "node processes running in the background which will not be cleared". The same symptom as M8, from a different pipe | [A] C1 |
| `repo:vitest-dev/vitest is:issue "write EPIPE" stdout` | 0 | — | [A] C2 |
| `repo:BurntSushi/ripgrep is:issue "broken pipe"` | 9 | [#200](https://github.com/BurntSushi/ripgrep/issues/200), "ripgrep doesn't stop when its pipe is closed" (2016) | [A] C1 |

- **The only discussion of work that goes on after the reader quits is ripgrep's.** Its author wrote: "a write to `stdout` in this case will fail with a pipe error, at which point, we should stop searching and quit" ([#200](https://github.com/BurntSushi/ripgrep/issues/200), 2016-10-30) [A].
- **The same comment frames the exit code as a choice.** "Maybe any type of error causes ripgrep to stop whatever it's doing, but a pipe error indicates normal termination where as anything else results in a non-zero exit code" [A]. The search path that M9 measured exits 1, not 0 [A].
- **A pytest maintainer refused to swallow the error in the writer.** On #4375, asottile proposed ignoring `EPIPE` in the writer. RonnyPfannschmidt answered: "wrong pattern to fix the issue (that would hide the fallout)" (2018-11-12) [A]. The fix that merged catches it at the entry point (#7185) [A].
- **No tracker discusses a stop point that changes from run to run.** I found none in any of the eight trackers with the queries above [C].

## 5. Does any tool give a refused write its own exit code or kind of error?

| tool | own exit code? | own kind of error? | source | tier |
|---|---|---|---|---|
| cargo-nextest | **yes: 110**, "Writing data to stdout or stderr produced an error" | yes: the cancel reason `ReportError`, "An error occurred while reporting results", printed as `reporting error` | [`exit_codes.rs` L43-L44](https://github.com/nextest-rs/nextest/blob/cargo-nextest-0.9.150/nextest-metadata/src/exit_codes.rs#L43-L44); [`events.rs` L2407-L2408](https://github.com/nextest-rs/nextest/blob/cargo-nextest-0.9.150/nextest-runner/src/reporter/events.rs#L2407-L2408), [L2431](https://github.com/nextest-rs/nextest/blob/cargo-nextest-0.9.150/nextest-runner/src/reporter/events.rs#L2431) | [A] |
| pytest | no: 1, the code for "some of the tests failed" | no; the comment says "Python exits with error code 1 on EPIPE" | [exit-codes.rst L8-L14](https://github.com/pytest-dev/pytest/blob/9.1.1/doc/en/reference/exit-codes.rst?plain=1#L8-L14); `config/__init__.py` L261 | [A] |
| `unittest` | no: 120, the interpreter's code when finalizing fails | no | `main.c` L840-L844 | [A] |
| `cargo test` | no: 101, libtest's `ERROR_EXIT_CODE`, also used for failed tests | no; the message calls it an "io error when listing tests" | [`lib.rs` L93](https://github.com/rust-lang/rust/blob/1.99.1/library/test/src/lib.rs#L93), L176-L179 | [A] |
| `go test` | no: the signal, 141 | no | M5 | [A] |
| Jest | no: 1, Node's "Uncaught Fatal Exception"; a failing test also exits 1 | no | process.md L4575; M10 | [A] |
| Vitest | no: 1; a failing test also exits 1 | no | M10 | [A] |
| ripgrep | no: 1 (= no match) or 0 | no; the source names it "graceful termination" | `main.rs` L47-L58, L128 | [A] |

- **nextest's code is one of several "could not write" errors.** Writing a test list, help, an event or a debug extract all map to 110 ([`errors.rs` L671-L677](https://github.com/nextest-rs/nextest/blob/cargo-nextest-0.9.150/cargo-nextest/src/errors.rs#L671-L677)) [A].
- **nextest ranks a reporting error above a test failure.** `CancelReason` is ordered by severity, and `ReportError` comes after `TestFailure` ([`events.rs` L2395-L2408](https://github.com/nextest-rs/nextest/blob/cargo-nextest-0.9.150/nextest-runner/src/reporter/events.rs#L2395-L2408)) [A].
- **nextest's text on stderr is outside its stable API.** "The public API does _not_ include human-readable output generated by nextest, or in general anything printed to standard error" ([stability/index.md L26-L27](https://github.com/nextest-rs/nextest/blob/cargo-nextest-0.9.150/site/src/docs/stability/index.md?plain=1#L26-L27)) [A].

## 6. What each behavior costs, across the survey

| behavior | tools | what it costs, measured | tier |
|---|---|---|---|
| stop at once | pytest, `unittest`, `cargo test`, `go test`, Jest, ripgrep; Vitest's main process | how many tests ran depends on where the reader quit: 21 to 75 here, and up to 232 for Vitest. Later tests never run, so their side effects never happen. Four of six runners exit with their failed-test code, so a script cannot tell a quit reader from a failed test. It is fast: pytest's stopped run took 0.7 s, its full run about 4 s | [A] M1–M7, M9 |
| stop starting tests, let running ones end, keep writing, own code | cargo-nextest | needs a cancel path. It writes about 20 more lines to a closed pipe. Its error lines and summary are lost when stderr is the pipe. The code 110 tells a quit reader apart from a failed test (100) | [A] M4, S5 |
| run the rest silently | none on purpose; Vitest's workers, by accident | the shell has exit 1 while tests still run: 67 to 87 tests ended after it. Their results reach no one | [A] M7, M8 |
| ignore the refused write | none now; ripgrep before 2016 | the full run's time for output nobody reads: 1.6 s with `\| head -n1` and with `\| wc -l` ([#200](https://github.com/BurntSushi/ripgrep/issues/200), the reporter's timing). Its author called it a bug | [B] for the timing; [A] for the quote |

- **Where tests run in parallel, the stop point moves from run to run.** nextest ran 47 to 51 tests, Jest 60 or 75, Vitest 144 to 232. One-thread runs moved by at most one test in five runs [A].
- **A parallel stop can change stderr from run to run.** `cargo test` printed 0 to 4 thread panics, with the same 44 tests run each time [A].
- **Stopping at once does not make the process tree stop at once.** `go test` let one more test end after the shell had the code (3 of 5 runs). Vitest let dozens end [A].

## What this means for purba

| evidence | what it bears on |
|---|---|
| six of seven runners, and ripgrep, stop at once; none ignores the write (item 1) | "ignore the refused write" has no current precedent; ripgrep's author called it a bug (item 4) |
| only nextest drains, and only nextest has its own code, 110 (items 1, 5) | "run the remaining tests silently and exit 3" has one precedent in part: nextest ends only the tests already running, not the rest |
| pytest, Jest, Vitest and `cargo test` reuse their failed-test code (item 5) | a code apart from the tests' code has one precedent, nextest |
| pytest, also in a Python process that ignores `SIGPIPE`, stops at once and prints nothing; it follows CPython's documented recipe (items 1, 2) | the closest precedent in process model to purba |
| parallel runners stop at a different test each run; one-at-a-time runners moved by one test at most (item 6) | how far a run gets depends on the parallelism and on when the reader quits, under every one of the three options |
| Vitest's workers outlive the exit code (item 1) | a stop must also stop whatever runs the tests, or tests run after the code is out |

## Appendix A. Commands

All measurements ran on 2026-10-27, on Linux 6.18.47 with 22 CPUs, from the scratch directory `$S` = `/home/snregales/.claude/jobs/aae7df98/tmp/research-686-scratch`, never from a purba checkout.
Every run unset the agent-session variables (`AI_AGENT`, `CLAUDECODE` and the other `CLAUDE_*`). Jest 30.5.2 and Vitest 5.0.5 switch to a short reporter when they see one: Jest's `AGENT_ENV_VARS` ([`TestScheduler.ts` L51-L69](https://github.com/jestjs/jest/blob/v30.5.2/packages/jest-core/src/TestScheduler.ts#L51-L69), [L386-L388](https://github.com/jestjs/jest/blob/v30.5.2/packages/jest-core/src/TestScheduler.ts#L386-L388)), and Vitest's `isAgent ? 'minimal' : 'default'` ([`defaults.ts` L109](https://github.com/vitest-dev/vitest/blob/v5.0.5/packages/vitest/src/defaults.ts#L109)). With them set, Jest `--verbose` printed 5 lines instead of 345.

The core of `measure.sh`, run for each case inside `nix-shell -p 'python3.withPackages(ps: [ps.pytest])' cargo rustc cargo-nextest go nodejs ripgrep strace bc`:

```bash
: > $RUNLOG; SIGLOG=$O/chain.txt "$@" > $O/ctl.out 2> $O/ctl.err        # control
for i in 1 2 3 4 5; do                                                    # five runs
  : > $RUNLOG
  "$@" 2> $O/err.$i | head -n 40 > $O/head.$i; ps=("${PIPESTATUS[@]}")    # MODE out
  # MODE both: "$@" 2>&1 | head -n 40
  n=$(wc -l < $RUNLOG); sleep 3; echo "PIPESTATUS=${ps[*]} ran=$n ran_3s_later=$(wc -l < $RUNLOG)"
done
"$@" 2>/dev/null | sh -c "grep -m1 -E '$pat' > first.txt; wc -l < $RUNLOG > atfirst.txt"   # buffering
strace -f -qq -s 200 -o $O/trace -e trace=write,writev,pwrite64,exit_group "$@" 2> err.st | head -n 40 > /dev/null
```

Each test's first call, for test `t00_00`, runs `chain.sh`, which prints the `SigIgn` and `SigCgt` bit `0x1000` (`SIGPIPE`) of six ancestors from `/proc/<pid>/status`.

| key | what | command |
|---|---|---|
| M1 | pytest | `python3 -I -m pytest -v -p no:cacheprovider` and without `-v`, in `$S/py` (20 files × 15 `unittest.TestCase` methods) |
| M2 | `unittest` | `python3 -I -m unittest discover -s . -v`, MODE out and both |
| M3 | `cargo test` | `cargo test --offline` and `cargo test --offline -- --test-threads=1`, in `$S/rs` (one lib, 300 `#[test]`), `CARGO_HOME` and `CARGO_TARGET_DIR` in `$S` |
| M4 | nextest | `cargo nextest run --offline`, MODE out and both; error text from one more run, `strace -f -qq -s 400 -e trace=write,exit_group cargo nextest run --offline 2>&1 \| head -n 40`, then `grep EPIPE` (exit 110, 50 ran) |
| M5 | `go test` | `go test -v` and `go test -v .`, in `$S/go` (one package, 300 tests), `GOFLAGS=-count=1 GOTOOLCHAIN=local GOPROXY=off` |
| M6 | Jest | `node node_modules/jest/bin/jest.js --verbose`, MODE out and both, in `$S/js` (20 files × 15 tests) |
| M7 | Vitest | `node node_modules/vitest/vitest.mjs run`, with `--reporter=verbose`, and with `--no-file-parallelism`; `globals: true` |
| M8 | Vitest's workers | `$S/orphan.sh`: after the pipeline returns, `ps -eo pid=,ppid=,comm=,args=` filtered to `node` processes under `$S/js`, polled every 100 ms; 3 runs: 13, 12, 16 alive, parent pid 2256 = `systemd --user`; ran 203→270, 224→300, 213→300 |
| M9 | ripgrep | `rg -n x big.txt` (2,000,000 lines), `rg -n x dir` (4 files), `rg -n -j1 x dir`, `rg -n zzz big.txt`, `--line-buffered`; each `\| head -n 40`, 3 runs: 1, 0, 1, 1, 1 |
| M10 | failing-test exit | one `expect(1).toBe(2)` test in `$S/jsfail`: `jest` exit 1, `vitest run` exit 1 |
| S1 | pytest 9.1.1 | `curl -sfL https://codeload.github.com/pytest-dev/pytest/tar.gz/refs/tags/9.1.1` |
| S2 | CPython v3.15.2 | `curl -sfL https://raw.githubusercontent.com/python/cpython/v3.15.2/<path>` for `Doc/library/{signal,sys,unittest}.rst`, `Lib/unittest/{runner,main}.py`, `Modules/main.c` |
| S3 | Rust 1.99.1, 1.95.0 | the same form for `library/test/src/{lib,console}.rs`, `library/test/src/formatters/pretty.rs`, `library/std/src/io/stdio.rs`, `library/std/src/sys/pal/unix/mod.rs`, `src/doc/unstable-book/src/compiler-flags/on-broken-pipe.md`; `lib.rs` also at 1.95.0 |
| S4 | cargo 0.100.0 | `src/ops/cargo_test.rs` (path from `gh api 'repos/rust-lang/cargo/git/trees/0.100.0?recursive=1'`) |
| S5 | nextest cargo-nextest-0.9.150 | the codeload tarball |
| S6 | Go go1.27.3 | raw `src/os/signal/doc.go`, `src/cmd/go/internal/test/test.go` |
| S7 | Jest v30.5.2 | the codeload tarball; `grep -rn 'EPIPE\|BrokenPipe\|broken pipe' packages/*/src` |
| S8 | Vitest v5.0.5 | the codeload tarball; `grep -rn 'EPIPE\|BrokenPipe' packages/vitest/src` |
| S9 | ripgrep 15.2.0, 15.1.0 | the 15.2.0 tarball; raw `crates/core/main.rs` at 15.1.0, `diff` of L40-L215 |
| S10 | Node v24.21.0 | raw `doc/api/process.md` |
| C1 | tracker searches | `gh api -X GET search/issues -f q='<query>' -f per_page=30`, 7 s between calls; titles from `.items[]`; issues read with `gh api repos/<o>/<r>/issues/<n>` and `/comments`; pytest#4375's timeline for the closing PR |
| C2 | more searches | the same form, 7 s apart, for the queries marked C2 |
| L | liveness | `gh api repos/O/R/commits/<branch> --jq .commit.committer.date`; `gh api repos/O/R/releases/latest`; for a repo with no releases, `git ls-remote --tags --refs`, then `gh api repos/O/R/commits/<tag>` |

## Appendix B. Versions

Sources were read at the newest tag; measurements ran what nixpkgs and npm gave.

| tool | source read | version run |
|---|---|---|
| pytest | 9.1.1 (`cf470ec0bf`) | 9.0.3, CPython 3.13.15 |
| `unittest`, CPython | v3.15.2 (`5e58eaad7c`) | CPython 3.13.15 |
| `cargo test` | rust 1.99.1 (`58cf8f9c35`), 1.95.0 (`59807616e1`); cargo 0.100.0 (`5f94df4789`) | cargo and rustc 1.95.0 |
| cargo-nextest | cargo-nextest-0.9.150 (`d225b1daf3`) | 0.9.136 |
| `go test` | go1.27.3 (`2aae1000ea`) | go1.26.8 |
| Jest | v30.5.2 (`202dd8a147`) | 30.5.2, Node 24.21.0 |
| Vitest | v5.0.5 (`7a825db43e`) | 5.0.5, Node 24.21.0 |
| Node | v24.21.0 (docs) | 24.21.0 |
| ripgrep | 15.2.0 (`e89fff89ac`), 15.1.0 (`af60c2de9d`) | 15.1.0 |

## Appendix C. Liveness

| project | default branch, last commit | latest release or tag | date |
|---|---|---|---|
| pytest-dev/pytest | `main`, 2026-10-27 | release 9.1.1 | 2026-06-19 |
| python/cpython | `main`, 2026-10-27 | tag v3.15.2 (no GitHub releases) | 2026-10-25 (tag commit) |
| rust-lang/rust | `main`, 2026-10-27 | release 1.99.1 | 2026-10-16 |
| rust-lang/cargo | `master`, 2026-10-27 | tag 0.100.0 (no GitHub releases) | 2026-08-28 (tag commit) |
| nextest-rs/nextest | `main`, 2026-10-26 | release cargo-nextest-0.9.150 | 2026-10-26 |
| golang/go | `master`, 2026-10-27 | tag go1.27.3 (no GitHub releases) | 2026-10-20 (tag commit) |
| jestjs/jest | `main`, 2026-10-02 | release v30.5.2 | 2026-09-18 |
| vitest-dev/vitest | `main`, 2026-10-26 | release v5.0.5 | 2026-10-15 |
| BurntSushi/ripgrep | `master`, 2026-10-27 | release 15.2.0 | 2026-07-15 |

## Appendix D. Claims I could not tier [A]

- **[B] ripgrep#200's timings.** 1.602 s with `\| wc -l` and 1.626 s with `\| head -n1` are the reporter's numbers, not re-run.
- **[C] A tracker discussion of a stop point that changes from run to run.** None found in any surveyed tracker (item 4).
- **[C] A tracker report of Vitest's workers outliving a stdout `EPIPE`.** vitest#9014 shows the same symptom from the worker channel; I found no report about stdout.
- **[C] A tracker report of ripgrep's exit 1 on a broken pipe.** The query `"broken pipe" "exit code"` gave 3 issues, none about it.
- **[C] macOS and Windows.** Every measurement ran on Linux.
- **[C] Whether Go and Node reset `SIGPIPE` for a child process.** Not checked.
- **[C] A test's own `print` to a closed stdout under `unittest`.** Not measured; the tests here print nothing.
