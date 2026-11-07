# How purba can start each test on macOS

This file answers [How does purba start each test on macOS, where Python calls fork unsafe?](https://github.com/kalonji-tools/purba/issues/852)
It is never merged.

## Answer

**A fork stays cheap on macOS, and it is unsafe for a reason that no thread explains.**
On a `macos-latest` runner, a fork from a parent that loaded the module cost 0.9–2.8 ms for each test.
A fresh interpreter that imports the module cost 70–113 ms, and ran the module body once for each test.

The fork crashed the child only when the parent and the child both called the same Apple framework.
On macOS 26, a DNS lookup in each crashed the child 10 times in 10, with one thread in the parent at the fork.
The crash is in `os_log`, which the Network framework and CoreFoundation call.
A child that made the call when the parent had not made it never crashed.

The build of the interpreter decides how often it crashes.
A Python linked against the macOS 26.5 SDK crashed in 25 of 27 runs of the paired cases. One linked against SDK 12.1 crashed in none of 9.
macOS 15 showed none of these crashes, but numpy's BLAS crashed a child there through `libdispatch`.

Of the 24 tools surveyed that start work in a new process, 12 fork without exec on macOS, and none of the 12 has code that avoids the crash.
The 4 tools that changed their macOS behaviour all moved from fork to spawn.

| question of the issue | short answer | tier |
|---|---|---|
| Why is `fork` unsafe on macOS? | Apple's libraries keep state that a child inherits invalid. Three guards depend on threads or on queues. The crash measured on macOS 26 does not | [A] P1–P3, sources |
| Does a zygote that starts no thread avoid it? | No. `dns-both` crashed with 1 thread at the fork | [A] P2 |
| Which modules break a fork on macOS? | A module whose import calls DNS, a proxy lookup, HTTP or CoreFoundation, and whose test calls it again. On macOS 15, also numpy's BLAS | [A] P2, P3 |
| What does a `posix_spawn` with a load of the module cost for each test? | 70–113 ms on `macos-latest`, 133–317 ms on `macos-15-intel` | [A] P1 |

## The probes

The probes are in this branch, under [`research/macos-start/`](https://github.com/kalonji-tools/purba/tree/22cbe65/research/macos-start).
A workflow of this branch alone, [`probe-852.yml`](https://github.com/kalonji-tools/purba/blob/22cbe65/.github/workflows/probe-852.yml), ran them on GitHub's runners.

| probe | what it runs | run |
|---|---|---|
| P1 cost | [`cost.py`](https://github.com/kalonji-tools/purba/blob/22cbe65/research/macos-start/cost.py), copied from [the prototype](https://github.com/kalonji-tools/purba/blob/1e0ef96/proto-689/bench/forkserver.py) with `posix_spawn` added | [42401843776](https://github.com/kalonji-tools/purba/actions/runs/42401843776), [42402325285](https://github.com/kalonji-tools/purba/actions/runs/42402325285) |
| P2 safety | [`safety.py`](https://github.com/kalonji-tools/purba/blob/22cbe65/research/macos-start/safety.py): a setup in the parent, a fork, an action in the child | the same two runs; the second ran each case 10 times and kept macOS's crash reports |
| P3 builds | [`interpreters.sh`](https://github.com/kalonji-tools/purba/blob/22cbe65/research/macos-start/interpreters.sh): P2's paired cases on 6 builds of Python, 3 times each | [42402827512](https://github.com/kalonji-tools/purba/actions/runs/42402827512) |

The runners: `macos-latest` is macOS 26.7.2 on arm64 with 3 cores. `macos-15-intel` is macOS 15.7.9 on x86_64 with 4 cores. `ubuntu-latest` is the control.
The interpreters of P1 and P2 are the python.org builds that `actions/setup-python` installs: 3.12.10 and 3.14.10 (3.14.7 on Intel).

## 1. The cost of each test

The module of [the prototype](https://github.com/kalonji-tools/purba/blob/1e0ef96/proto-689/RESULTS.md#a-process-for-each-test-with-the-module-body-run-once-per-module) imports asyncio, json and 8 more.
Each cell is the range of 6 runs, for each test after the first, in ms.

| how each test starts | macos-latest 3.12 | macos-latest 3.14 | macos-15-intel 3.12 | macos-15-intel 3.14 | ubuntu-latest 3.12 | ubuntu-latest 3.14 |
|---|---|---|---|---|---|---|
| fork from the parent | 0.86–1.76 | 1.98–2.83 | 3.08–5.38 | 2.84–6.46 | 1.30–1.74 | 1.47–1.64 |
| forkserver, the module preloaded | 2.91–4.13 | 3.95–6.29 | 10.1–11.6 | 8.2–17.8 | 4.08–5.19 | 3.08–3.17 |
| `posix_spawn` + import | 70.0–93.2 | 73.8–112.5 | 133–175 | 182–317 | 73.5–93.7 | 99.6–104.6 |
| spawn of `multiprocessing` | 71.6–87.5 | 80.6–154.0 | 145–180 | 173–341 | 78.7–101.1 | 107.3–115.3 |

- The fork and the forkserver ran the module body once in 200 tests. `posix_spawn` and spawn ran it once for each test [A] P1.
- On `macos-latest`, a `posix_spawn` cost 26 to 108 times a fork.
- The only per-task macOS cost that the survey found published: nextest says "even the simplest of tests in `cargo nextest run` taking more than 0.2 seconds" is a sign of XProtect ([macos.md L13](https://github.com/nextest-rs/nextest/blob/cargo-nextest-0.9.151/site/src/docs/installation/macos.md#L13)) [A].

## 2. What crashes a forked child

### The cases

P2's 10 runs of each case, on `macos-latest` with Python 3.14.10. Linux and Python 3.12.10 on the same runner crashed in none.

| case: parent does → child does | threads at the fork | child died by SIGSEGV |
|---|---|---|
| nothing → nothing | 1 | 0 of 10 |
| DNS (`socket.getaddrinfo`) → DNS | **1** | **10 of 10** |
| proxy lookup (`urllib.request.getproxies`) → the same | 3 | 10 of 10 |
| HTTPS `urlopen` → the same | 3–4 | 10 of 10 |
| CoreFoundation `CFPreferencesCopyAppValue` → the same | 3 | 8 of 10 |
| proxy lookup in a loop on a 2nd thread → proxy lookup | 4–5 | 10 of 10 |
| the same, with `OBJC_DISABLE_INITIALIZE_FORK_SAFETY=YES` | 4–5 | 10 of 10 |
| DNS, proxy lookup or CoreFoundation → nothing | 1–3 | 0 of 10 each |
| nothing → DNS, proxy lookup, HTTPS or CoreFoundation | 1 | 0 of 10 each |
| `ssl.create_default_context` → the same | 1 | 0 of 10 |
| a thread that waits → nothing | 2 | 0 of 10 |

On `macos-15-intel`, only numpy crashed: a matrix product in each, 10 of 10 on both Pythons, with 4–5 threads in the parent.
A parent that only imported numpy, or a child alone that multiplied, never crashed [A] P2.
numpy 2.5.4 on both macOS runners uses Accelerate as its BLAS [A] P2's `numpy.show_config()`.

### Where the child died

macOS wrote a crash report for each death. Each of the 29 kept reports names the cause `crashed on child side of fork pre-exec` [A] P2.
18 of them also carry CoreFoundation's note `*** multi-threaded process forked ***`.

| runner | reports | the three top frames of the crashing thread |
|---|---|---|
| macos-latest, 3.14 | 17 | `_os_log_preferences_refresh` < `os_log_type_enabled` < `Network` |
| macos-latest, 3.14 | 8 | `_os_log_preferences_refresh` < `os_log_type_enabled` < `-[CFPrefsSearchListSource alreadylocked_copyValueForKey:]` |
| macos-15-intel, both | 4 | `_dispatch_apply_with_attr_f` < `dispatch_apply` < `libBLAS.dylib: APL_dgemm` |

macOS kept 25 reports for the 58 deaths of the macos-latest job, and 4 for the 20 of the two Intel jobs.

[Resque #1939](https://github.com/resque/resque/issues/1939) reports the same `os_log` crash for DNS after a fork [A]:
"even a single-threaded parent with nothing else running reproduces it, as long as it initialized os_log before forking (any earlier DNS lookup does that)".
It names the range "macOS 26.0–26.5.1 and the 27 beta; not macOS 15 or earlier", and "Apple DTS confirmed it's working-as-intended and won't be fixed".
This probe saw it on 26.7.2.

### The build of the interpreter

P3, on `macos-latest`. Each paired case (DNS, proxy lookup, CoreFoundation, in parent and child) ran 3 times on each build.

| build | `minos` / `sdk` of the binary | paired cases crashed | child-only DNS crashed |
|---|---|---|---|
| python.org 3.12.10 | 11.0 / 12.1 | 0 of 9 | 0 of 3 |
| uv 3.12.16 | 11.0 / 15.0 | 3 of 9 | 0 of 3 |
| uv 3.14.10 | 11.0 / 15.0 | 3 of 9 | 0 of 3 |
| Homebrew 3.12.16 | 26.0 / 26.5 | 7 of 9 | 0 of 3 |
| python.org 3.14.10 | 11.0 / 26.5 | 9 of 9 | 0 of 3 |
| Homebrew 3.14.10 | 26.0 / 26.5 | 9 of 9 | 0 of 3 |

The same six cases crashed in none of 6 builds on `macos-15-intel` [A] P3.
The crash rate follows the SDK of the build: none at 12.1, a third at 15.0, nearly all at 26.5.
Why the SDK matters is [C]: no source this session read says what `os_log` changes by the SDK a binary links against.
purba runs inside the tester's interpreter, so the tester's build decides it, not purba's wheel.

### Why the three thread guards do not explain it

Apple's libraries guard a fork in three ways. Two of them depend on what the parent started.

| guard | it arms when | in the child | source |
|---|---|---|---|
| Objective-C `+initialize` | the process has had a thread when it forks | an uninitialized class aborts. "processes that are single-threaded when fork() is called have no restrictions on +initialize in the child" | [objc-os.mm L828-L862](https://github.com/apple-oss-distributions/objc4/blob/objc4-951.7/runtime/objc-os.mm#L828-L862), [objc-initialize.mm L632-L633](https://github.com/apple-oss-distributions/objc4/blob/objc4-951.7/runtime/objc-initialize.mm#L632-L633) [A] |
| libdispatch | GCD's queues were set up in the parent | the queue heads are set to `(void *)0x100`, and the next use crashes | [queue.c L9036-L9058](https://github.com/apple-oss-distributions/libdispatch/blob/libdispatch-1542.100.32/src/queue.c#L9036-L9058) [A] |
| CoreFoundation | CF was used in the parent; threads pick only the text of the log | "The process has forked and you cannot use this CoreFoundation functionality safely. You MUST exec()." is written, and `HALT` is commented out | [CFRuntime.c L807-L813](https://github.com/apple-oss-distributions/CF/blob/CF-1153.18/CFRuntime.c#L807-L813), the last open-source CF [A]; whether the shipped CF aborts is [C] |

The numpy crash fits the libdispatch row. The `os_log` crash fits none of them: it came with one thread and no queue that the probe started.
CPython warns of a fork in a process with more than one live thread. On macOS it counts them with `task_threads` ([posixmodule.c v3.14.0 L7975-L8072](https://github.com/python/cpython/blob/v3.14.0/Modules/posixmodule.c#L7975-L8072)) [A].
That warning was absent from every `dns-both` run, which crashed [A] P2.

### What purba itself adds

- purba's Rust starts no thread outside its tests: each `thread::spawn` under `crates/` is in a `tests/*.rs` file [A] grep at `e2ccac8`.
- Python's standard library reaches these frameworks through `socket` and `urllib.request`. A test module reaches them through any client that resolves a name or reads the proxy settings [A] P2.

## 3. What other tools do on macOS

A survey of 26 tools and 27 crash reports. 6 of its claims were re-read at their sources. The rest were fetched by the agent that wrote it, in this session [A].

| on macOS | tools | count |
|---|---|---|
| fork without exec, and no code that avoids the crash | pytest-forked, pytest-isolate, [pytest-parallel](https://github.com/browsertron/pytest-parallel/blob/0.1.1/README.md#L58) (forces fork "at the expense of safety"), nose2 mp (forces fork), gunicorn, uWSGI, Spring, Puma, Pitchfork, Unicorn, Resque, PHP-FPM | 12 |
| fork on Linux, spawn or exec on macOS | CPython `multiprocessing` ([context.py v3.14.0 L322-L328](https://github.com/python/cpython/blob/v3.14.0/Lib/multiprocessing/context.py#L322-L328)), `ProcessPoolExecutor`, [billiard](https://github.com/celery/billiard/blob/v4.3.2/billiard/context.py#L371-L376) (Celery, since 2026-02-24), Chromium ([`posix_spawnp`](https://github.com/chromium/chromium/blob/977677222700199f86fe809f4b1b7c09af4ec95e/base/process/launch_mac.cc#L405)) | 4 |
| never fork without exec | pytest-xdist, pytest-isolated, Jest, nextest, Erlang/OTP, Bazel workers, Go | 7 |
| not on macOS | Android's zygote | 1 |

- Every tool that changed its macOS behaviour moved from fork to spawn: CPython in 3.8 and billiard in 2026. None moved from spawn to fork.
- billiard moved after [Celery #9894](https://github.com/celery/celery/issues/9894): on macOS 26.0, a forked child crashed in `setproctitle`.
- Of the 27 crash reports, 12 began with a network call: a proxy lookup, an HTTP client, gRPC, libcurl, DNS or a cloud SDK. 3 began with libpq, 3 with a native library that links CoreFoundation, 1 with numpy's BLAS. 8 do not say.
- 6 of the 27 came from a test runner or a test suite.
- `OBJC_DISABLE_INITIALIZE_FORK_SAFETY=YES` fixed 6 reports and failed in 4. Two of the 4 are on macOS 26 or 27. It failed in every run of P2 too.
- [infrahub #1296](https://github.com/opsmill/infrahub-sdk-python/issues/1296) has the same control as P2: "nocall -> exit=0 # fork alone is fine", and "clean -> KILLED by signal 11".

## 4. What the answer leaves to #694

[Each test runs in a process of its own, forked from a zygote that loaded its module once](https://github.com/kalonji-tools/purba/issues/694) decides how macOS starts each test.
Two facts bound that decision.

- **A fork on macOS keeps the cost and the module body run once**, and a test crashes when its module's import and the test both reach `os_log` or GCD.
  The crash is loud: the child dies by SIGSEGV, and [the record](https://github.com/kalonji-tools/purba/blob/e2ccac8/docs/decisions/each-test-runs-in-a-process-of-its-own.md?plain=1#L30) already gives a crash to one test.
  A tester then sees a test fail that passes under pytest.
- **A fresh interpreter for each test costs 26 to 108 times a fork**, and runs the module body once for each test.
  That breaks [the zygote's line](https://github.com/kalonji-tools/purba/blob/e2ccac8/docs/decisions/each-test-runs-in-a-process-of-its-own.md?plain=1#L32) and #694's "runs its body once for all its tests".

No probe checked whether a thread count or `pthread_is_threaded_np()` at the fork predicts the `os_log` crash. P2's `dns-both` says it cannot: one thread, and a crash in 10 of 10.

## Sources

- The survey and the mechanism report stay in the session that wrote them. Every link above was fetched in this session.
- The re-read claims: Resque #1939, infrahub #1296, billiard's `context.py` and PR #443, pytest-parallel's README, Chromium's `launch_mac.cc`, nextest's `macos.md`, objc4's `objc-os.mm` and `objc-initialize.mm`, libdispatch's `queue.c`, CF's `CFRuntime.c`, CPython's `context.py` and `posixmodule.c`.
- The mechanism report claimed that only `_scproxy` reaches an Apple framework from the standard library. P2's `dns-both` falsifies it: `socket.getaddrinfo` reaches the Network framework.
- The runs' artifacts expire 90 days after 2026-11-07.
