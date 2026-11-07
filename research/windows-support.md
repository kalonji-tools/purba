# What Windows support costs purba, and what to do about it

This file feeds [#840](https://github.com/kalonji-tools/purba/issues/840). It answers a question the owner asked on 2026-11-07: is it better to keep Windows as it is, to move the Windows code into a crate of its own that each build picks by target, or to drop Windows? And how have other projects solved this?
It is never merged.

Three parts follow the synthesis.
Each part was written by a separate research pass, and keeps its own evidence tags.

- [Part 1: what Windows costs purba](#part-1-what-windows-support-costs-purba-today-and-what-it-blocks-tomorrow) — measured in the tree, CI, records and issues.
- [Part 2: the crate split, probed](#part-2-a-windows-crate-picked-by-target-and-the-other-layouts) — Cargo's rules, and a throwaway workspace that tests them.
- [Part 3: how other projects handle Windows](#part-3-how-other-projects-handle-windows) — 29 projects and 5 tier policies.

Tags: **[A]** read in a primary source at a pinned version. **[P]** measured or probed (the command is in the part). **[I]** inference, with its basis. **[B]** a secondary source, named.

Pins. Parts 1 and 2 read purba `main` at `b59f42f`. While they ran, `main` moved to `291f6ec`: PR #839 (#809) merged, with 15 more lines that split by platform, all Unix-side. Part 1 calls #839 "open". The synthesis uses `291f6ec`. [P] The Windows-target lint of Part 2 was run again at `291f6ec`, and its result holds. [P]

## Answer

Windows costs purba little code and little CI time. It costs design attention, and that cost is growing.
The Windows-only Rust is about 52 lines. [A] Windows adds a median 4 s to a green Build. macOS, not Windows, is the slow arm. [P]
But each process and signal decision now gets a second, weaker answer for Windows. The `cfg` lines that split by platform went from 20 to 35 with #809 alone, merged on 2026-11-07. [P]
No end-to-end test has ever run on Windows, so each Windows behaviour that purba claims is unprobed. [A][P]
The zygote of #694 needs `fork`, and Windows has none. So the largest planned feature needs a second design there. [A]
No user and no stated goal asks for Windows. The platform set came from "we build it, so we test it" in #42 and #48. No record decides it. [P]

A Windows crate picked by target moves the code. It does not lower any of these costs.
Its main expected benefit is a check of the Windows code from Linux. That check already works in any layout: `cargo clippy --target x86_64-pc-windows-gnu` passes on purba today, and it fails on an error planted only in a Windows arm. The normal Linux lint misses that error. [P]
CI has never linted the Windows code, because clippy runs only on ubuntu. [A]
A crate also makes each platform function `pub`, so Rust stops warning about code that one OS no longer uses. [P]
Among the projects read, only uv splits Windows into its own crate, and it does so because a second crate shares that code. [A] purba has no second consumer. [I]

Other projects split into two groups. [A, 29 projects in Part 3]
Tools that support Windows treat `fork` as a Unix speed-up, and give Windows the same contract through a slower spawn: `multiprocessing`, Chromium, Ray, nextest. purba's plan for #694 already has this shape.
Tools whose contract *is* `fork` or POSIX signals mostly never shipped native Windows: gunicorn, uvloop, pytest-forked, uWSGI, the Ansible control node.
For signals, every project read documents the Windows gap and does not emulate it.
Each drop in the survey gave a reason about people, not a technical wall: Celery was "unable to diagnose issues on this platform", and Rust's demotion named "no dedicated target maintainers". [A]
No project in the survey dropped Windows and later restored it. [A, for the searches made; not proof of absence]

So the choice is about the **promise** that purba makes to Windows, not about where the code lives. [I: rests on the three parts together]
The layout question has a cheap answer: lint the Windows target now, and gather each OS into one private module when #694 lands. [P][I]
The promise question is the owner's decision. The options are below.

## The options, named for what they do

```
                     cost today          at #694 (zygote)        Windows users get        undo later
                     ──────────          ────────────────        ─────────────────        ──────────
keep as is           test gap, records   build spawn path too    every feature, unprobed  —
+ lint Windows       one CI step         same                    same, type- + lint-safe  remove the step
move to a crate      +manifests, pub fns same                    same                     merge back
one os module        none                arms collected          same                     inline again
promise less (T2)    one record          spawn path optional     "builds, may fail"       raise the tier
drop, WSL only       one record + 8      no Windows design       WSL2 only                hard: none came back
```

### Keep Windows as it is

The tree stays as it is today. [A]

```rust
#[cfg(not(windows))] const NAMES: [&str; 3] = ["SIGINT", "SIGTERM", "SIGHUP"];
#[cfg(windows)]      const NAMES: [&str; 3] = ["SIGINT", "SIGTERM", "SIGBREAK"];
```

- Windows CI runs `cargo test` and loads the wheel. It runs no bats and no clippy. [A]
- Each new process or signal decision gets a Windows sentence. Today 8 of 98 records carry one. [P]
- #694 needs a spawn-per-test path, and its record already says outcomes "can then differ by platform". [A]

### Keep Windows, and lint its code

This option adds one step to the existing Windows `rust` job, as nextest and uv do. [A]

```yaml
- name: Lint the Windows arm
  if: runner.os == 'Windows'
  run: mise exec -- cargo clippy --all-targets --locked
```

The other form runs on Linux and needs a MinGW C compiler for `stacker`, from ruff's parser. `zig cc` is enough. [P]

```sh
CC_x86_64_pc_windows_gnu=<zig-cc wrapper> \
  cargo clippy --all-targets --locked --target x86_64-pc-windows-gnu
```

- It finds a type error, a lint, or a signature drift that lives only in the Windows arm. [P]
- It does not run the Windows code. Ctrl-Break, `STATUS_CONTROL_C_EXIT` and the null-handle stdout check stay unprobed. [A]
- It is useful under every option except a drop. [I]

### Move the Windows code into a crate picked by target

This is uv's layout. [A]

```toml
# crates/execution/Cargo.toml
[target.'cfg(windows)'.dependencies]
purba-windows = { path = "../windows" }
```
```rust
// crates/windows/src/lib.rs
#![cfg(windows)]          // required: lint:rust compiles every member for Linux
pub fn die_by(signum: i32) -> ! { … }
```

- Cargo keeps the crate out of the Linux build of `purba-execution`. [P]
- `Cargo.lock` and the licence gate still read the Windows crates. They already do: `windows-sys` comes in through ruff's parser. [P]
- Without `#![cfg(windows)]` at its root, `cargo clippy --all-targets` on Linux fails with `E0433`. [P]
- Its functions are `pub`, so `dead_code` never fires on a function that one OS stopped using. [P]
- It checks nothing more than the lint step above. [P]
- It pays only when a second crate needs the Windows code alone. That is uv's stated reason. [A]

### Gather each OS into one private module

This is the layout of nextest, std, mio and ctrlc. [A]

```rust
// crates/execution/src/lib.rs
#[cfg_attr(unix, path = "os/unix.rs")]
#[cfg_attr(windows, path = "os/windows.rs")]
mod os;   // os::die_by(..), os::writable_stdout(..), later os::start_test(..)
```

- It adds no manifest. It keeps the `dead_code` signal. [P]
- It collects the arms. Today they are spread through `interrupt.rs`: 30 platform lines in one file. [P]
- nextest keeps its Unix-only "double spawn" with a no-op Windows stub in this layout. That is the shape the #694 zygote would take. [A][I]
- It has the most value when #694 lands. Before that, two files hold all the Windows code. [I]

### Promise Windows less: build it, test it on Linux only

This is a written "Tier 2", the wording of Rust and uv. [A]

```markdown
## Decision Outcome   (a new record: which platforms purba supports)
Linux and macOS are tested on each pull request, and a failure blocks the merge.
Windows is built on each pull request. A Windows test failure does not block it.
```

- Rust: a Tier 2 target "will always build, but [it] may or may not pass tests". [A]
- Rust also warns that such a target "will likely accumulate bugs faster". [A]
- Rust asks 3 named maintainers for Tier 1 and 2 for Tier 2. A single maintainer meets only Tier 3 in Rust and CPython. [A]
- #694 could ship spawn-per-test later, or never. [I]

### Drop Windows, and point to WSL

This is the policy of Ansible, pyenv and Pants. [A]

```markdown
purba runs on Linux and macOS. On Windows, run it under WSL2.
```

- It removes about 52 Rust lines, 5 of 15 Build jobs, 1 of 3 publish jobs, and a few config lines. [P]
- It needs one new record, which decides the platforms, and rewrites of the 8 records that name Windows. [P]
- #694 and the interrupt work lose their second design. [A]
- It loses the `win_amd64` wheel, which never shipped, and native Windows development. [P]
- It does not reopen #94: devenv also fails on Intel macOS and is slower. [A]
- It is hard to undo. Unix-only projects that later wanted Windows needed a funded team (Bun) or a beta label (Ray). [A] uvloop, gunicorn and Pants have open Windows issues 6 to 13 years old. [A]

## What the three parts agree on

| finding | part 1 (purba) | part 2 (mechanics) | part 3 (others) |
|---|---|---|---|
| the pain is behaviour and decisions, not code size | 52 lines; 8 records; #694 second design | a split changes none of these | process tools pay "two implementations of the process layer" |
| a crate split is not the lever | — | checks no more than `--target`; loses `dead_code` | modules in one crate everywhere but uv |
| fork on Unix, spawn on Windows is the normal shape | #694 record already says so | nextest's double spawn, Unix-only with a stub | multiprocessing, Chromium, Ray, nextest |
| document a signal gap, do not emulate it | #839 already leaves Windows weaker | — | nextest, uv, coverage.py, tokio, asyncio |
| no record decides the platform set | the matrices hold it | — | every tier policy writes it down |

## Gaps

- Nothing in this research ran purba on Windows. The Windows arms were compiled and linted, never run. [P]
- Part 3 did not read pytest-subprocess, crossterm or rustix. Four of its claims rest on secondary sources only. They are listed at its end.
- No part asked whether a Windows user would come. purba has no release to measure that. [P]
- A `<scratch>` path names a file on the machine that ran the research. Those files are not kept.
- The synthesis spot-checked 4 quotes in Part 3 against their pinned sources (Celery 4.0, pytest-forked, nextest, uv). All 4 matched. [P]


## Part 1: What Windows support costs purba today, and what it blocks tomorrow

Read at purba `main` = `b59f42f9ec59b1796631818e6eabf4e07c94a796` (the checkout is on `main`, `git rev-parse HEAD main` gives the same SHA).
Measured on 2026-11-07.
GitHub data read as `snregales-agent`, reads only.

Tags: **[A]** read in the source itself. **[P]** measured with a command (the command is named). **[I]** inference, with what it rests on.

### Answer

Windows costs purba little code today, and a lot of attention. [I: rests on §1, §3 and §4 together]
The Windows-only Rust is about 52 lines in two files. [A]
The bigger cost is in decisions: 8 of 98 records carry a Windows sentence. 7 of those 8 state a Windows-only behaviour, a Windows exception or a Windows downside; 1 only claims Windows in its Confirmation. [P][A]
In CI, Windows does not usually make Build slower: macOS finishes last in most green runs, and its queue is the slow one. [P]
But a Windows arm does the least work and still uses as many runner-seconds as macOS. [P]
Windows alone broke Build in 4 runs, all while CI was being built, and never after 2026-10-13. [P]
The real cost is a test gap: no end-to-end test has ever run on Windows. So every Windows behaviour purba claims is either untested or tested only by one unit test. [A][P]
Tomorrow is different. The zygote of #694 needs fork, and Windows has none. Windows gets a second design, "spawn each test", with different semantics that the record already names as a downside. [A]
The open interrupt fix (#809, PR #839) already builds a native handler for Unix alone and leaves Windows with weaker behaviour. [A]
No user, issue author or stated goal asks for Windows. purba has no release and no tag. [P]
Dropping Windows deletes a small amount of code. It needs one new record and rewrites of about 8 records. It loses the `win_amd64` wheel and native Windows development, which nobody uses today. [I: rests on §6 and §7]

| cost category | count / measure | evidence |
|---|---|---|
| Windows-only Rust lines in `crates/**` | ~52 lines (38 in `interrupt.rs`, 14 in `purba/src/lib.rs`) | §1 table, [A] |
| `cfg` sites on `main` that split by platform | 20 attributes in 5 files | `grep -rnE 'cfg\(' crates` [P] |
| more `cfg(unix)` sites in the open PR #839 | 15 added lines with a `cfg` attribute, all Unix-side | `gh pr diff 839` [P] |
| Windows arms tested by an end-to-end test | 0 of 53 bats cases | `build.yml:163,175` [A]; `grep -h '@test' tests/*.bats \| wc -l` = 53 [P] |
| Windows-only unit tests | 1 (`interrupt.rs:388-400`) | [A] |
| Unix-only Rust tests (no Windows twin) | 3 | `run.rs:223`, `interrupt_term.rs:3`, `interrupt_load_waiting.rs:2` [A] |
| Build matrix jobs that are Windows | 5 of 15 per run | `build.yml:68,93,96` [A] |
| Windows share of runner-seconds, green runs | 549 s of 1,443 s per run (38%), while it skips bats and the fault wheel | §2 [P] |
| green runs where Windows finished last | 12 of 54 (macOS: 41 of 54) | §2 [P] |
| wall time Windows adds to a green run | median wall 186 s with Windows, 182 s without (54 runs); in the 11 runs where Windows extends it, by a median 20 s | §2 [P] |
| failed Build runs where only Windows failed | 4 of 69 runs with a failed matrix job (since 2026-09-25); none after 2026-10-13 | §2 [P] |
| decision records that name Windows | 8 of 98 | §3 [P] |
| issues whose body names Windows (true hits) | 11 of 486 (4 false hits: "deprecation windows", "windows-1252", oxitest's `windows-spike`) | §4 [P] |
| issue/PR comments naming Windows | 104 issue comments + 8 review comments | §4 [P] |
| planned work that needs a second Windows design | #694 (zygote), #809/PR #839 (native handler), #837 (interrupt cases), any future timeout or parallel worker | §5 [A][I] |
| evidence of a Windows user | none: 0 issues by anyone but the owner, the agent account and Actions | §6 [P] |

### 1. Code

#### Rust `cfg` sites on `main`

Command: `grep -rnE 'cfg\(|target_os|target_family' --include='*.rs' crates` [P].
It finds 20 attributes. 15 are in `crates/execution/src/interrupt.rs`.

| site (`b59f42f`) | what it does | Unix arm | Windows arm | Windows arm tested? |
|---|---|---|---|---|
| `execution/src/interrupt.rs:41-42`, `:46-47` | statics `UNMASKED`, `MASKED`: the thread's signal mask at `install` | 2 + 2 lines | none (Windows has no signal mask) | n/a |
| `interrupt.rs:62-65` | `NAMES`: the three interrupt signals | `SIGINT SIGTERM SIGHUP` (1 line) | `SIGINT SIGTERM SIGBREAK` (1 line) | no test sends SIGBREAK [A: #794 comment](https://github.com/kalonji-tools/purba/issues/794#issuecomment-6670895504) |
| `interrupt.rs:122-132` | `masked()`: reads the mask with `pthread_sigmask` | 11 lines | none | n/a |
| `interrupt.rs:137-147` | `put_back_mask()` | 11 lines | none | n/a |
| `interrupt.rs:150-162` | `change_mask()` | 13 lines | none | n/a |
| `interrupt.rs:166-180` | `shown()`: the name on the interrupt line | 4 lines (Python's name) | 8 lines (`Ctrl-C`, `Ctrl-Break`) | **yes**, one unit test `interrupt.rs:388-400` |
| `interrupt.rs:213-214` | `install` records the mask | 1 line | none | n/a |
| `interrupt.rs:318-319` | `put_back` restores the mask | 1 line | none | n/a |
| `interrupt.rs:358-385` | `die_by()`: how purba dies after an interrupt | 15 lines: `SIG_DFL`, unmask, `raise`, then `exit(128+n)` | 11 lines: `exit(STATUS_CONTROL_C_EXIT)` | **no**. Rust job compiles it; nothing runs it [A: #794 comment: "no probe checks its exit code"] |
| `interrupt.rs:388-400` | `#[cfg(all(test, windows))]` test module | — | 13 lines | runs in `rust windows-latest` [A] |
| `purba/src/lib.rs:71-80` vs `:82-95` | `writable_stdout()`: does stdout take a write? | 6 lines: `write(&[])` on a dup of fd 1 | 10 lines: null handle check, `ERROR_INVALID_HANDLE` | **no**. [#651 comment](https://github.com/kalonji-tools/purba/issues/651#issuecomment-6428856144): "No test measures the Windows half" |
| `execution/tests/run.rs:223` | `a_forked_child_exits_with_its_own_code` | 1 test | — (Windows has no `os.fork`) | n/a |
| `execution/tests/interrupt_term.rs:3` | whole file `#![cfg(unix)]`, 73 lines, 1 test | SIGTERM and dispositions via `libc::sigaction` | — | Windows has no twin |
| `execution/tests/interrupt_load_waiting.rs:2` | whole file `#![cfg(unix)]`, 41 lines, 1 test | a signal that waits as a load starts | — | Windows has no twin |

Other Windows-shaped Rust that has no `cfg`:

- `execution/src/panic.rs:104,108` splits a path on both `/` and `\\`, and a test checks "as Windows separates it" (`panic.rs:124-128`). [A] It runs on every platform.
- `crates/execution/Cargo.toml` takes `libc` on every target. #794's spec says why: "`libc` becomes a dependency on every target, because Windows needs its `signal` too." ([comment](https://github.com/kalonji-tools/purba/issues/794#issuecomment-6670895504)) [A] Without Windows, `libc` would still be needed on Unix. [I]
- `selection/tests/select.rs:251` uses `sys.platform == "win32"` only as sample decorator text. [A] It is not a platform split.

Windows-only Rust lines, counted from the line ranges above: `interrupt.rs` 64-65, 164-173, 373-385, 388-400 = 38 lines; `lib.rs` 82-95 = 14 lines. Total about 52. [A]

#### Python, scripts, tasks, lock

| location | Windows content | effect |
|---|---|---|
| `python/` | none: `grep -rniE 'sys\.platform\|os\.name'` finds no Python source hit outside the Rust test text above [P] | — |
| `.config/mise.toml:90-92` | `shell = "bash -c -o errexit"`, with the reason "Windows otherwise runs a task through `cmd`, which cannot exec a shebang." [A] | every task runs through bash; the line also gives `errexit` [A: #343 comment](https://github.com/kalonji-tools/purba/issues/343#issuecomment-6057183858) |
| `.config/tasks.toml:171-180` | `test:rust` has a Tera guard `{% if os() != "windows" %}` and the comment "Windows needs no assignment" [A] | the only `os()` guard in the tasks [P: `grep -n 'os(' .config/tasks.toml`] |
| `.config/mise.lock` | 12 `platforms.windows-x64` entries; 13 for each of the 6 other platforms [P] | the missing one is `bats`, so `mise install` on Windows skips bats silently [A: mise record L91-92] |
| `scripts/test/require-abi3.bats:26-96` | `win_amd64` file names as test fixtures [A] | data only; the check is platform-neutral [I] |
| `prek.toml:27` | hook `check-illegal-windows-names` [A] | refuses a path Windows cannot check out; cheap |
| `README.md:35` | "Windows: the Visual Studio Build Tools, with the C++ workload" [A] | the README's C-toolchain table |
| `CONTEXT.md:99` | interrupt = "SIGINT, SIGTERM or SIGHUP, or on Windows a Ctrl-C or Ctrl-Break" [A] | glossary term carries a Windows clause |
| `Cargo.lock` | `windows-sys`, `windows-link` [P] | transitive; dependencies declare them, so they stay whatever purba decides [I] |

### 2. CI

#### Where Windows runs

| workflow | Windows arms | what the Windows arm skips |
|---|---|---|
| `build.yml` `rust` | 1 of 3 (`:68`) | nothing: it runs `mise run test:rust` [A] |
| `build.yml` `wheel` | 4 of 12 (`:93`, `:96`) | the fault wheel (`:163`) and all bats end-to-end tests (`:175`) [A] |
| `publish.yml` `wheel` | 1 of 3 (`:37`) | — ; builds `win_amd64` only, not `win_arm64` [A] |
| all other workflows | 0 (every other `runs-on` is `ubuntu-latest`) [P: `grep -nE 'runs-on' .github/workflows/*.yml`] | — |

Windows also forces `shell: bash` on the wheel steps. `build.yml:139-141` says why: "`run:` defaults to PowerShell on Windows, where the expansion below is empty". [A]

#### Durations

`Build` runs on `pull_request` only (`build.yml:28-30`), so `gh run list --workflow build.yml --branch main` returns `[]`. [P]
I sampled the latest 200 Build runs (2026-11-03 to 2026-11-07) with `gh run list --limit 200`, then `gh api .../runs/<id>/jobs?filter=latest` for each. [P]
81 of them ran the matrix. 54 had all 15 matrix jobs green. [P]
Script: `<scratch>/analyse.sh`.

| job | OS | n | median s | p90 s |
|---|---|---|---|---|
| rust | ubuntu | 80 | 50 | 60 |
| rust | macos | 73 | 84 | 103 |
| rust | **windows** | 76 | **113** | 128 |
| wheel | ubuntu | 315 | 73 | 92 |
| wheel | macos | 285 | 112 | 183 |
| wheel | **windows** | 309 | **104** | 146 |

Step medians over 20 green runs (`fetch_steps.sh`) [P]:

| step | ubuntu | macos | windows |
|---|---|---|---|
| `jdx/mise-action` setup | 15 s | 18 s | **37 s** |
| `mise run test:rust` | 31 s | 43 s | **63 s** |
| `mise run build` (wheel) | 24 s | 47 s | 47 s |
| fault wheel build | 5 s | 5 s | skipped |
| bats end-to-end | 5 s | 6 s | skipped |

So the Windows wheel arm does the least work and is still slower than Linux. [P] Skipping bats saves Windows only about 10 s. [P]

#### Critical path

| measure (54 fully green runs) | value |
|---|---|
| OS of the last matrix job to finish | macOS 41, **Windows 12**, ubuntu 1 [P] |
| median wall time, first start to last finish | 186 s [P] |
| the same, with Windows jobs removed | 182 s [P] |
| runs where Windows extends the wall time | 11 of 54; median extension 20 s [P] |
| median queue wait after `changed` (405 jobs each) | ubuntu 2 s, windows 2 s, macOS 3 s; **p90: macOS 304 s**, windows 5 s [P] |
| runner-seconds per green run | ubuntu 341, macOS 553, **windows 549** [P] |

The repository is public (`gh repo view --json visibility` = `PUBLIC`) [P], so these runner-seconds cost no money. [I: GitHub bills standard runners only on private repositories]
Windows is rarely the critical path. macOS is, mostly through its queue. [P]

#### Failures

I read the jobs of every failed Build run: `gh run list --workflow build.yml --limit 2000` returns 1,930 runs since 2026-09-25, and 328 concluded `failure`. [P]
In 69 of them a `rust` or `wheel` job failed. The rest failed in `changed`, in the rollup, or by cancellation. [P]

| OS set whose matrix jobs failed | runs |
|---|---|
| ubuntu + macOS + Windows | 48 |
| ubuntu + macOS (Windows passed) | 12 |
| macOS alone | 3 |
| ubuntu + Windows | 1 |
| ubuntu alone | 1 |
| **Windows alone** | **4** |

The 4 Windows-only failures [P]:

| run | date | branch | failed step | cause |
|---|---|---|---|---|
| 36195079298, 36196058299, 36196406136 | 2026-09-25 | `ci/42-build-gate` (PR #185) | `Build against <py>` on all 4 Windows wheel arms | `run:` defaults to PowerShell [I: same branch and day as the [PR #185 review comment](https://github.com/kalonji-tools/purba/pull/185#discussion_r4109148442)] |
| 38445852805 | 2026-10-13 | `proto-535-wheel-test` | `Run the end-to-end test` | `bats: command not found`, a deliberate probe [A: [PR #544 comment](https://github.com/kalonji-tools/purba/pull/544#issuecomment-6152302976)] |

So Windows broke Build on its own twice in six weeks, both while CI was being built, and never since 2026-10-13. [P]
The opposite case is more frequent. In 16 runs another OS failed and Windows passed. [P]
At least two of those were TDD red pushes whose failing test was a bats case, which Windows skips (PR #712, PR #749). [A] So the Windows arm cannot show a red step that lives in bats. [I]

### 3. Records

`grep -liE windows docs/decisions/*.md` finds 8 of 98 records. [P]

| record (`docs/decisions/`) | Windows sentence, quoted | class |
|---|---|---|
| `each-test-runs-in-a-process-of-its-own.md` | L34 "**On Windows, purba spawns each test.**" L41 "**A tester pays a fork for each test, and a spawn on Windows.**" L42 "**On Windows, a module body with an effect outside the process runs once for each test.** An outcome can then differ by platform." | Windows-only behaviour + Windows downside (outcomes differ by platform) |
| `an-interrupted-run-prints-its-count-and-dies-by-the-signal.md` | L16 fact "on Windows, CPython exits `STATUS_CONTROL_C_EXIT` instead"; L57 "**On Windows, purba exits `STATUS_CONTROL_C_EXIT` after a Ctrl-C or a Ctrl-Break.** The line names `Ctrl-C` or `Ctrl-Break`." | Windows-only behaviour; exception to "dies by the signal" |
| `an-interrupt-stops-the-run-and-a-second-one-exits-at-once.md` | L42 "**SIGTERM, SIGHUP and a Ctrl-Break on Windows do what SIGINT does.**" L45 "On Unix, purba then masks again each interrupt signal that the parent masks" | Windows-only behaviour; Windows exception (no mask) |
| `bats-tests-the-installed-wheel-from-outside-purba.md` | L12 fact "all four Windows arms found no `bats`"; L49 "Every arm runs them except the Windows arms."; L55 "**Windows never runs the console script.**" | Windows exception to the test rule + Windows downside (test gap) |
| `a-case-of-each-kind-checks-that-no-failure-prints-as-another-kind.md` | L18 fact "Build runs bats on Linux and macOS only"; L47 "**`tests/kinds.bats` misses Windows.** bats does not run there." | Windows downside (test gap) |
| `a-line-names-its-kind-in-the-glossarys-words.md` | L59 "On Unix, the Python carries its ABI flags"; L74 "**On Windows, a free-threaded build reads as the build with a GIL.**" | Windows downside (users see a wrong word) |
| `purba-carries-no-tool-manager-beside-mise-and-no-compiler-of-its-own.md` | L44 "**devenv alone.** Windows is reachable only through WSL2"; L91-92 "The lockfile holds no Windows entry for `bats`. The install on Windows skips that tool and reports nothing." | option rejected partly because of Windows + Windows downside |
| `the-crate-carries-an-rlib.md` | L54 "The required `Build` check runs `cargo test --doc -p purba` on Linux, macOS and Windows." | Windows in a Confirmation (claim only) |

No record decides which platforms purba supports. [P: `grep -rnE 'three operating systems|operating system|TESTED|windows-latest' docs` finds only the mise record and `the-nightly-is-named-by-a-date.md`]
The platform set lives in `build.yml:68,93` and `publish.yml:32-37` ("The platforms `build.yml` builds and loads on, and no others"). [A]
The #42 grilling chose it: "TESTED covers three runners: `ubuntu-latest`, `macos-14`, `windows-latest`" ([comment](https://github.com/kalonji-tools/purba/issues/42#issuecomment-5829089063)). [A]

### 4. History

`gh issue list --state all --limit 900` returns 486 issues. 15 bodies match "windows". [P]
4 are false hits: #23 and #24 ("deprecation windows"), #644 ("windows-1252"), #52 (oxitest's `windows-spike`). [A]
`gh api .../issues/comments --paginate` returns 1,864 comments; 104 match. `.../pulls/comments` returns 818; 8 match. [P]

| issue / PR | what Windows did | cost type |
|---|---|---|
| [#9](https://github.com/kalonji-tools/purba/issues/9#issuecomment-5560688166), #11, #41 | oxitest's `check_subprocess_encoding.py`: "292 of 1774 tests red on Windows". Carried as "a dated stopgap, once a Windows job exists" | predicted future cost (encoding of spawned subprocesses) |
| [#46](https://github.com/kalonji-tools/purba/issues/46#issuecomment-5664518007), #78 | "A symlink is REJECTED … needs Administrator rights or Developer Mode on Windows. purba's wheel matrix targets Windows." `CLAUDE.md` is a one-line `@AGENTS.md` file | blocked option |
| [#94](https://github.com/kalonji-tools/purba/issues/94#issuecomment-5680064201) | "native Windows: mise. devenv reaches Windows only through WSL2"; zig cannot drive the msvc linker ([comment](https://github.com/kalonji-tools/purba/issues/94#issuecomment-5682241419)) | extra decision; one reason among several to reject devenv |
| [#40](https://github.com/kalonji-tools/purba/issues/40#issuecomment-5778039195), [PR #138](https://github.com/kalonji-tools/purba/pull/138#discussion_r4073553920) | "mise runs a task through `cmd` on Windows … That trap already cost this branch one red run." | extra code (`env` instead of an assignment, a Tera guard), one red run |
| [#343](https://github.com/kalonji-tools/purba/issues/343), [PR #361](https://github.com/kalonji-tools/purba/pull/361#issuecomment-6057149702), [PR #355](https://github.com/kalonji-tools/purba/pull/355#issuecomment-6045826886) | `shell = "bash -c"` repeated 13 times "because Windows runs a command through `cmd`"; one PR to unify it | extra decision + one PR |
| [PR #185](https://github.com/kalonji-tools/purba/pull/185#discussion_r4109148442) | "`Build` went red on all four Windows arms … `run:` defaults to PowerShell on Windows" | one red run, `shell: bash` lines |
| [PR #191](https://github.com/kalonji-tools/purba/pull/191#issuecomment-5845272859) | "The Windows arm's cache key ends in a different hash" | investigation time |
| [#89](https://github.com/kalonji-tools/purba/issues/89#issuecomment-5978167909) | Operating System classifiers: Windows is in the tested set, so the old reason was false | extra decision |
| [#481](https://github.com/kalonji-tools/purba/issues/481#issuecomment-6107174085), [PR #483](https://github.com/kalonji-tools/purba/pull/483#issuecomment-6107813410) | PyO3 links `python3.dll` for abi3 on Windows, unlike Unix; a record row had to cite both spans | extra review round |
| [#535](https://github.com/kalonji-tools/purba/issues/535#issuecomment-6152849037), [PR #544](https://github.com/kalonji-tools/purba/pull/544#issuecomment-6152302976), PR #546 | "All four Windows arms fail with `bats: command not found` (exit 127)"; option "a second installer of bats" refused by the mise record | **permanent test gap**; Windows exception in a record |
| [#570](https://github.com/kalonji-tools/purba/issues/570#issuecomment-6312976694), #571, PR #606 | `test:rust` library path: "Windows loads it with none"; a record line rewritten | extra code (Tera guard) + record edit |
| [#651](https://github.com/kalonji-tools/purba/issues/651#issuecomment-6428856144), PR #678 | closed-stdout detection: a second implementation via a null handle; "No test measures the Windows half" | extra code + test gap + behaviour difference (a non-null handle that refuses writes is not caught) |
| [#653](https://github.com/kalonji-tools/purba/issues/653#issuecomment-6403667214) | exit-code limits: "Windows was not measured" | open unknown |
| [#689](https://github.com/kalonji-tools/purba/issues/689#issuecomment-6463880787) → #694 | "on Windows one process holds a module's tests — rejected"; "On Windows, purba spawns each test" | extra decision; behaviour difference users see |
| [PR #712](https://github.com/kalonji-tools/purba/pull/712#issuecomment-6501008886), [PR #749](https://github.com/kalonji-tools/purba/pull/749#issuecomment-6613748222) | TDD red step: "The four Windows wheel jobs passed, because they skip bats" | a plan claim had to be corrected each time |
| [#728](https://github.com/kalonji-tools/purba/issues/728#issuecomment-6541727977), [#729](https://github.com/kalonji-tools/purba/issues/729#issuecomment-6603163594), [#716](https://github.com/kalonji-tools/purba/issues/716#issuecomment-6528571812) | "No cell checks Windows, because bats does not run there" — now a standard "what no check sees" line | recurring test-gap disclosure |
| [PR #735](https://github.com/kalonji-tools/purba/pull/735#issuecomment-6541975947) | "`3.14.0t` is false on Windows"; `sys.abiflags` exists on Unix only | behaviour difference users see + record downside |
| [#765](https://github.com/kalonji-tools/purba/issues/765), [#767](https://github.com/kalonji-tools/purba/issues/767#issuecomment-6626769221), [#769](https://github.com/kalonji-tools/purba/issues/769#issuecomment-6631401274), #774, [#794](https://github.com/kalonji-tools/purba/issues/794#issuecomment-6670895504) | Ctrl-Break, `STATUS_CONTROL_C_EXIT`, "every Windows row … not run"; "**This slice builds Ctrl-Break … No test sends them a signal.** ('Build now, unprobed')" | extra research question, extra code, unprobed behaviour |
| [#810](https://github.com/kalonji-tools/purba/issues/810#issuecomment-6704680023) | "The mask is put back on Unix only. Windows has no signal mask." | Windows exception |
| [#809](https://github.com/kalonji-tools/purba/issues/809#issuecomment-6707041118), PR #839 (open) | "**The change is for Unix only.** On Windows, purba keeps today's behaviour"; new Downside "a second interrupt can wait for C code" | second (weaker) behaviour on Windows |

`git log --oneline -G'cfg\(…windows…|unix\)' main -- crates` lists 6 commits that changed a platform `cfg` line: #617, #651, #774, #794, #806, #810. [P]
All 6 are in the last 3 weeks of a 482-commit history. [P: `git log --oneline main | wc -l`] So the Windows split grows with the interrupt and process work, not with time. [I]

### 5. Future

| planned work | why Windows needs its own design | second implementation? |
|---|---|---|
| [#694](https://github.com/kalonji-tools/purba/issues/694) zygote + fork per test (open) | "Windows has no fork: each test is spawned, and the module body runs once for each test" [A: issue body]. Prototype on Linux: fork 0.9 ms, spawn 53 ms per test [A: record L13]. "An outcome can then differ by platform" [A: record L42] | **yes**: a spawn path with different isolation semantics, different cost, and its own tests |
| [#809](https://github.com/kalonji-tools/purba/issues/809) / PR #839 (open) | native signal handler via `sigaction`; PR adds 15 Unix-side `cfg` lines incl. `target_os = "linux"`/`target_vendor = "apple"` splits [P] | **yes, deferred**: Windows keeps the Python-level handler; record gets a Windows downside [A] |
| [#837](https://github.com/kalonji-tools/purba/issues/837) (open) | cases use process groups, `pthread_sigmask`, `faulthandler.register(SIGINT)` [A] | probably Unix-only again [I: Windows has no signal mask (#810) and no process groups in this sense] |
| [#810](https://github.com/kalonji-tools/purba/issues/810) (merged) | "The mask is put back on Unix only" [A] | Windows exception already shipped |
| [#692](https://github.com/kalonji-tools/purba/issues/692), [#693](https://github.com/kalonji-tools/purba/issues/693) | rest on the "process of its own" model [A] | inherits #694's split [I] |
| a timeout per test (no issue yet) | killing a test's process tree uses process groups and signals on Unix and job objects on Windows; nextest keeps `runner/unix.rs` and `runner/windows.rs` [A: #465 comment](https://github.com/kalonji-tools/purba/issues/465#issuecomment-6100161159) | yes, by prior art [I] |
| parallel workers (no issue yet) | follows the #694 start method; "process creation is … slow on Windows" [A: #481 comment, citing nextest] | yes [I] |
| subprocess encoding (no issue yet) | oxitest: 292 of 1774 tests red on Windows from encoding [A: #9 comment] | a stopgap check was planned "once a Windows job exists" [A] |

### 6. Demand

- Authors: all 486 issues are by `snregales` (71), `snregales-agent` (413) or Actions (2). [P: `gh issue list --json author`] Comments: the same accounts plus bots. [P]
- No tag and no GitHub release exist. [P: `git tag` and `gh release list` both empty]
- #1 (wayfinder) names Windows only to say the studied competitor has "**no Windows**". [A] #10 and #12 say the same. [A]
- #42, titled "Prove the wheel builds, installs and imports everywhere purba claims", fixed "TESTED covers three runners: … `windows-latest`". [A]
- #48 spec: "**The platforms are the three that `Build` already builds and loads on**" ([comment](https://github.com/kalonji-tools/purba/issues/48#issuecomment-5984474479)). [A]
- `pyproject.toml:25-27` has no `Operating System ::` classifier: "Each wheel's platform tag already states where it installs". [A]

Where purba **claims** Windows:

| location | claim |
|---|---|
| `README.md:35` | Windows build prerequisite row |
| `build.yml:68,93`, `publish.yml:37` | Windows in every wheel matrix |
| `the-crate-carries-an-rlib.md:54` | "runs `cargo test --doc -p purba` on Linux, macOS and Windows" |
| mise record L23, L112 | "every architecture and operating system the wheel matrix covers"; "on three operating systems" |
| `CONTEXT.md:99` | interrupt includes Ctrl-C/Ctrl-Break on Windows |

No location states a Windows goal or a Windows user. [P: the greps above] The claim is "we build it, so we test it", from #42 and #48. [I]

### 7. Drop inventory

#### What changes

| area | change | size |
|---|---|---|
| `interrupt.rs` | delete `:64-65`, `:164-173`, `:373-385`, `:388-400`; `cfg(not(windows))` → plain or `cfg(unix)` | ~38 lines, 4 attributes |
| `purba/src/lib.rs` | delete `:82-95` | 14 lines |
| `.config/tasks.toml:171,176` | drop the comment and the `os() != "windows"` guard | 2 lines |
| `.config/mise.toml:90` | the reason for `shell = "bash …"` changes; the line itself may stay for `errexit` [A: #343] | 1 comment line |
| `.config/mise.lock` | 12 `windows-x64` entries; the mise record says "mise chooses the platform list itself", so removing them needs a platform setting [A: L65] | generated; one config line [I] |
| `build.yml` | `windows-latest` from 2 matrices; 2 `if: runner.os != 'Windows'`; the PowerShell comment `:139-140` | 5 of 15 matrix jobs gone |
| `publish.yml:37` | drop `windows-latest` | 1 of 3 wheel jobs |
| `README.md:35` | drop the Windows row | 1 line |
| `CONTEXT.md:99` | drop the Windows clause | 1 sentence |
| `prek.toml:27` | `check-illegal-windows-names` may stay (cheap, protects a checkout) | optional |
| records | a **new** record "which platforms purba supports" (no title answers it today [P]), and rewrites of the 8 records in §3 | 1 new + 8 rewritten |
| #694 | "Done when" drops the Windows line; the spawn path is never built | removes the second design |
| PR #839 / #809 | the Windows downside goes; no Windows follow-up | removes a deferred gap |

#### What is lost

- The `win_amd64` wheel from `publish.yml`. No release has shipped one. [P] `win_arm64` was never built by `publish.yml`. [A]
- Native Windows development. #94 measured it as a reason to pick mise over devenv. [A] mise still wins without Windows: devenv also fails on Intel macOS and is slower everywhere (record L44). [A] So dropping Windows does not reopen #94. [I]
- Ctrl-Break handling and `STATUS_CONTROL_C_EXIT`. Nobody tests them today. [A]
- A Windows tester could still run purba under WSL2, which is Linux. [I]
- The one competitor named in #1 has no Windows support, so Windows was a differentiator. [A] Whether that matters is the sibling "other projects" research. [I]

## Part 2: A Windows crate picked by target, and the other layouts

Tags: **[A]** read in a pinned primary source (link given). **[P]** I ran it (command in the Probe log). **[I]** inference (basis given).

Pins: purba `b59f42f9ec59b1796631818e6eabf4e07c94a796` (main, 2026-11-07). Toolchain `nightly-2026-11-02` (rustc 1.101.0-nightly 3e92e1a2d, cargo 31d3246e0). Cargo Book read at the same cargo commit `31d3246e0e8d3a80d4a90b54aa48ff180adb408b`. PyO3 guide at `v0.29.2`.

### Answer

A Windows crate selected by a target table works as described, and it changes less than it seems to. Cargo keeps it out of the Linux build graph of the crate that depends on it. Cargo still lists its dependencies in `Cargo.lock`, and still compiles it on Linux as a workspace member, so its root needs `#![cfg(windows)]`. The main benefit people expect from it is a Linux check of the Windows code. That benefit does not come from the split. It comes from `cargo clippy --target x86_64-pc-windows-gnu`, which checks `#[cfg(windows)]` code in any layout. I ran that command on purba's real tree. It passes today with no Python installed. It caught a type error and a clippy lint planted only in `die_by`'s Windows arm. purba's CI never lints that arm today, because clippy runs only on ubuntu. The one obstacle is a C build script (`stacker`, via ruff's parser), which needs a MinGW C compiler. `zig cc` is enough. A split does not change what Windows does, and it does not remove the need for a Windows runner to *run* the tests. It earns its cost only when a second consumer needs the Windows code, which is uv's reason. The smallest change that gives the main benefit is a Windows-target clippy step. After that, when #694 lands, put one `os` module per OS inside `purba-execution`.

| Layout | Example code shape | Who uses it | Checks the Windows arm from Linux? | Cost |
|---|---|---|---|---|
| **Inline cfg items** (purba today) | `#[cfg(unix)] fn writable_stdout() {…}`<br>`#[cfg(windows)] fn writable_stdout() {…}` | purba `interrupt.rs`, `lib.rs`; nextest `signal.rs` (`#[cfg(unix)] mod imp` / `#[cfg(windows)] mod imp`) | Yes, with `--target x86_64-pc-windows-gnu` [P] | The arms are mixed in with shared code. Each new divergence adds cfg sites. |
| **One `os` module per OS, in the crate** | `#[cfg_attr(unix, path = "sys/unix.rs")]`<br>`#[cfg_attr(windows, path = "sys/windows.rs")]`<br>`mod sys;`<br>`pub use sys::stop_code;` | std (`cfg_select!` in `sys/pal`, `sys/process`), mio `sys/`, ctrlc `platform/`, nextest `runner/{unix,windows}.rs`, tokio `signal/windows.rs` | Yes [P] | The shared signature is a convention. mio writes it down as a "Required types" list. A private helper unused on one OS fails `dead_code` [P]. |
| **A public API per OS, no fake common API** | `tokio::signal::unix::signal(SignalKind::hangup())`<br>`tokio::signal::windows::ctrl_break()`<br>`std::os::unix::process::CommandExt::pre_exec` vs `std::os::windows::process::CommandExt::creation_flags` | tokio, std | Yes [I: it is cfg code like the others] | Each caller writes its own cfg. In return, the API does not pretend that the OSes behave the same. |
| **A trait, one impl per OS, a cfg type alias** | `pub trait Watcher {…}`<br>`impl Watcher for ReadDirectoryChangesWatcher`<br>`#[cfg(windows)] pub type RecommendedWatcher = ReadDirectoryChangesWatcher;` | notify | Yes [I: same basis] | The trait states the signature once. Each impl still compiles only on its own target. |
| **One crate per OS, picked by a target table** | `# crates/uv/Cargo.toml`<br>`[target.'cfg(target_os = "windows")'.dependencies]`<br>`uv-windows = { workspace = true }`<br>`// uv-windows/src/lib.rs`<br>`#![cfg(windows)]` | uv (`uv-windows`, `uv-unix`) | Yes, the same command [P]. No better than the module layouts. | Two more manifests. The root cfg is required [P]. A `pub fn` is never dead code [P]. `cargo doc` and `cargo test` see an empty crate on Linux [P]. The lockfile is unchanged [P]. |

### 1. Cargo mechanics

#### Target-specific dependencies

- A dependency under `[target.'cfg(windows)'.dependencies]` applies only when the target matches the cfg. The table takes the same keys as `[dependencies]`. [A] [specifying-dependencies.md L424-L451](https://github.com/rust-lang/cargo/blob/31d3246e0e8d3a80d4a90b54aa48ff180adb408b/doc/book/src/reference/specifying-dependencies.md#L424-L451)
- `cfg(feature = …)`, `cfg(test)` and `cfg(debug_assertions)` do not work in a target table. [A] same file, L453-L469.
- A target table can inherit from `[workspace.dependencies]` with `workspace = true`. [A] [L658-L660](https://github.com/rust-lang/cargo/blob/31d3246e0e8d3a80d4a90b54aa48ff180adb408b/doc/book/src/reference/specifying-dependencies.md#L658-L660). [P] the probe uses `windows-sys.workspace = true` under `[target.'cfg(windows)'.dependencies]`.
- `[workspace.dependencies]` has no target form. `[workspace.target.'cfg(windows)'.dependencies]` makes Cargo print `warning: Cargo.toml: unused manifest key: workspace.target`. [P]
- That warning does not fail the build, even with purba's `[build] warnings = "deny"`. `cargo check` exited 0. [P]

#### The lockfile and the resolver

- The resolver ignores the platform. It resolves target tables "as-if all platforms are enabled". [A] [resolver.md L503-L505](https://github.com/rust-lang/cargo/blob/31d3246e0e8d3a80d4a90b54aa48ff180adb408b/doc/book/src/reference/resolver.md#L503-L505)
- So `Cargo.lock` lists Windows-only crates on Linux. In the probe, `windows-sys 0.61.2` and `windows-link 0.2.1` are in the lockfile after a Linux build. [P]
- purba's lockfile already lists `windows-sys` and `windows-link`. They come in through `stacker` ← `ruff_python_parser` ← `purba-parsing`. [P] `cargo tree --target all -i windows-sys`.
- Feature resolver 2 does not enable features of a target-specific dependency when that target is not being built. [A] [resolver.md L374-L394](https://github.com/rust-lang/cargo/blob/31d3246e0e8d3a80d4a90b54aa48ff180adb408b/doc/book/src/reference/resolver.md#L374-L394)
- Probe: `libc` with `features = ["extra_traits"]` only under `cfg(windows)`. `cargo tree -e features -i libc` shows `extra_traits` for `x86_64-pc-windows-gnu` and not for `x86_64-unknown-linux-gnu`. [P]
- Resolver 3 keeps resolver 2's feature rules. It changes only the default of `resolver.incompatible-rust-versions` to `fallback`. [A] [resolver.md L549](https://github.com/rust-lang/cargo/blob/31d3246e0e8d3a80d4a90b54aa48ff180adb408b/doc/book/src/reference/resolver.md#L549)
- `cargo tree` filters by the host by default. `--target all` shows every target. [A] [cargo-tree.md L144-L147](https://github.com/rust-lang/cargo/blob/31d3246e0e8d3a80d4a90b54aa48ff180adb408b/doc/book/src/commands/cargo-tree.md#L144-L147). [P] the host tree omits `probe-platform-windows` under `probe-core`, and `--target all` shows both arms.
- cargo-deny checks every resolved crate, target-specific ones included, unless `[graph] targets` narrows the set. [A] [cargo-deny 0.20.2 cfg.md L29-L43](https://github.com/EmbarkStudios/cargo-deny/blob/0.20.2/docs/src/checks/cfg.md#L29-L43). purba's `.cargo/deny.toml` sets no `targets`. [P] `grep`. So the licence gate already reads Windows crates. A split changes nothing there. [I]

#### A Windows-only crate as a workspace member

- With `members = ["crates/*"]` and a virtual manifest, a command with no package flag runs on every member. [A] [workspaces.md L133-L151](https://github.com/rust-lang/cargo/blob/31d3246e0e8d3a80d4a90b54aa48ff180adb408b/doc/book/src/reference/workspaces.md#L133-L151)
- So `cargo build --workspace` on Linux compiles the Windows crate *for Linux*, even though no Linux crate depends on it. With no root cfg, it failed with `error[E0433]: cannot find 'windows' in 'os'`. [P]
- `cargo build -p probe-core` still passed, because the crate is not in `probe-core`'s Linux graph. [P]
- `#![cfg(windows)]` at the crate root makes the crate empty on Linux. Build, clippy, doc and test then pass. [P] uv does exactly this. [A] [uv-windows/src/lib.rs L1-L6](https://github.com/astral-sh/uv/blob/0.13.7/crates/uv-windows/src/lib.rs#L1-L6)
- The crate's own OS dependencies must also go in a target table. Otherwise the empty Linux crate would carry an unused dependency. [I] Basis: the control below, where an unused non-transitive dependency is refused.
- `default-members` lets plain `cargo build` and `cargo clippy` skip a member. `--workspace` still includes it. [A] workspaces.md L133-L151. [P] with `default-members` set, `cargo build` passed and `cargo build --workspace` failed.
- purba's `lint:rust` runs `cargo clippy --all-targets` with no `--workspace`. On a virtual manifest that already means every member. [A] [tasks.toml L120-L122](https://github.com/kalonji-tools/purba/blob/b59f42f9ec59b1796631818e6eabf4e07c94a796/.config/tasks.toml#L120-L122) with workspaces.md L149-L151.

#### purba's strict lints against an empty-on-Linux crate

- clippy `pedantic`, `nursery` and `cargo` at `deny`, rustdoc `all = "deny"`, and `warnings = "deny"` all pass on a `#![cfg(windows)]` crate on Linux. [P] `cargo clippy --workspace --all-targets`, `cargo doc --no-deps --workspace`.
- `cargo doc` on Linux documents nothing in that crate. A broken intra-doc link in it passes on Linux. It fails with `cargo doc --target x86_64-pc-windows-gnu` (`error: unresolved link to 'NoSuchItem'`). [P] The same holds for a `#[cfg(windows)]` module in any layout. [I] Basis: rustdoc documents only cfg-active items.
- The `cargo::unused_dependencies` lint is live in this cargo. An unused `windows-sys` in `probe-core`'s `[dependencies]` gave `error: unused dependency 'windows-sys'`. [P]
- The lint skips a dependency that is also a transitive one, "may be activating features". [A] [unused_dependencies.rs L290](https://github.com/rust-lang/cargo/blob/31d3246e0e8d3a80d4a90b54aa48ff180adb408b/src/diagnostics/rules/unused_dependencies.rs#L290). [P] the same unused `windows-sys` passed for `--target x86_64-pc-windows-gnu`, where `probe-platform-windows` brings it in transitively.
- It also checks only the selected packages. [A] same file, [L43-L60](https://github.com/rust-lang/cargo/blob/31d3246e0e8d3a80d4a90b54aa48ff180adb408b/src/diagnostics/rules/unused_dependencies.rs#L43-L60).
- purba-execution's unconditional `libc` is unused on Windows: every `libc::` call sits under `cfg(unix)` or `cfg(not(windows))`. The Windows clippy run did not refuse it. [P] [I] Basis: the lint skips a dependency that another crate also brings in, and pyo3 brings in `libc`. Moving `libc` to `[target.'cfg(unix)'.dependencies]` states the truth. It needs no split.

#### Checking the Windows arm from Linux, with pyo3 in the graph

- `rustup target add` installs the Windows std. I added `x86_64-pc-windows-gnu` and `x86_64-pc-windows-msvc` to `nightly-2026-11-02`. Undo: `rustup target remove --toolchain nightly-2026-11-02 x86_64-pc-windows-gnu x86_64-pc-windows-msvc`. [P]
- `cargo check` and `cargo clippy` do not link, so no Windows linker is needed for Rust code. [P] the probe passed for gnu and msvc with no MinGW or MSVC installed.
- PyO3 0.29.2 says a target interpreter and a host interpreter are both "optional when building abi3 extension modules". It says an `abi3-py3*` feature can replace `PYO3_CROSS_PYTHON_VERSION`, and that `PYO3_CROSS_LIB_DIR` "can often be omitted" for Windows. [A] [building-and-distribution.md L394-L438](https://github.com/PyO3/pyo3/blob/v0.29.2/guide/src/building-and-distribution.md#L394-L438)
- On Windows, PyO3 links with `raw-dylib`, so it needs no import library. [A] [L242-L248](https://github.com/PyO3/pyo3/blob/v0.29.2/guide/src/building-and-distribution.md#L242-L248)
- Probe: `cargo clippy --workspace --all-targets --target x86_64-pc-windows-gnu` passed with no `python` on PATH and no `PYO3_*` variable. [P] Also for `-msvc`. [P]
- The workspace build works because `crates/purba` enables `abi3-py312`, and the features of one build unify. [I] Basis: the next line.
- A pyo3 crate built alone without abi3 fails: `PYO3_CROSS_PYTHON_VERSION or either an abi3-py3* or abi3t-py3* feature must be specified when cross-compiling`. [P] (`-p probe-py-noabi`, and on purba `-p purba-execution`.) `PYO3_CROSS_PYTHON_VERSION=3.12` fixes it. [P]
- A host build of the same probe needs an interpreter: `error: no Python 3.x interpreter found`. `PYO3_PYTHON` set to mise's 3.12 fixes it. [P]
- **purba's real tree** passes `cargo clippy --all-targets --locked --target x86_64-pc-windows-gnu` from Linux. [P] It needs one extra input: a C compiler for the target. `stacker`'s build script compiles `windows.c`, which includes `<windows.h>`. With no compiler: `failed to find tool "x86_64-w64-mingw32-gcc"`. [P] With `CC_x86_64_pc_windows_gnu` set to a wrapper around `zig cc -target x86_64-windows-gnu`: `Finished`, exit 0. [P]
- `-msvc` from Linux fails on the same kind of build script (`psm`, then `stacker`), because there is no MSVC-capable C compiler. [P] purba's code uses no `target_env` cfg, so `-gnu` checks the same Rust arms that `-msvc` would. [P] `grep -E 'target_env|target_os'` over `crates/` found nothing outside the two files.

### 2. Probe results that bear on the design

These are the findings. Commands and output are in the Probe log.

| What was planted | Host `clippy` | `--target …windows-gnu` | Layout |
|---|---|---|---|
| Type error only in the Windows crate | passes | `error[E0308]` [P] | separate crate |
| Same type error in `sys/windows.rs` | passes | `error[E0308]` [P] | one crate, `cfg_attr(path)` |
| Windows arm takes `i64`, Unix arm takes `i32` | passes | `error[E0308]` at the shared caller, in both layouts [P] | both |
| purba `die_by`: `const STATUS_CONTROL_C_EXIT: u32 = "…"` | passes | `error[E0308] … interrupt.rs:377` [P] | purba today |
| purba `die_by`: `#[expect(clippy::cast_possible_wrap)]` removed | passes | `error: casting 'u32' to 'i32' may wrap` [P]; `cargo check` for the same target passes [P] | purba today |
| Windows `stop_code` is `const fn`, Unix is not | passes | `clippy::missing_const_for_fn` on the shared `#[pyfunction]` wrapper [P] | one crate |
| A private helper used only by Unix callers | `dead_code` error in `sys/unix.rs` | `dead_code` error in `sys/windows.rs` [P] | one crate |
| The same helper as a `pub fn` in a separate crate | passes | passes [P] | separate crate |

What the table says:

- The off-target check catches the same defects in every layout. The split adds no check. [P]
- A defect can live only on one target: a type error, a lint, a signature drift, or a constness mismatch. Only a check for that target finds it. [P]
- purba's `#[expect(clippy::cast_possible_wrap)]` in `die_by` was never checked by CI. Clippy runs only in `quality.yml`, on `ubuntu-latest` [A] [quality.yml L37](https://github.com/kalonji-tools/purba/blob/b59f42f9ec59b1796631818e6eabf4e07c94a796/.github/workflows/quality.yml#L37). The Windows `rust` job runs only `mise run test:rust`, which is `cargo test` [A] [build.yml L61-L79](https://github.com/kalonji-tools/purba/blob/b59f42f9ec59b1796631818e6eabf4e07c94a796/.github/workflows/build.yml#L61-L79). The probe shows the expectation is met today. [P]
- A crate boundary makes a platform function `pub`, and `dead_code` never fires on `pub`. A split therefore loses the "unused on this OS" signal that a private module keeps. [P]

### 3. Code-layout patterns in other projects

Every project below keeps its platform code as **modules in one crate**, except uv. Of the two whose CI I read, nextest and uv, both run clippy on a Windows runner. Neither relies on a cross check.

**Rust std: one facade, cfg-selected modules.**
- `sys/pal/mod.rs` picks one platform module with `cfg_select! { unix => { mod unix; pub use self::unix::*; } windows => { … } … }`. [A] [pal/mod.rs L6-L13](https://github.com/rust-lang/rust/blob/1.99.1/library/std/src/sys/pal/mod.rs#L6-L13)
- `sys/process/mod.rs` does the same for each subsystem, with `use unix as imp;` and a `_ => unsupported` arm. Then `pub use imp::{Command, ExitStatus, Process, …}` names the shared surface. [A] [sys/process/mod.rs L1-L39](https://github.com/rust-lang/rust/blob/1.99.1/library/std/src/sys/process/mod.rs#L1-L39)
- The public `std::process` holds an `imp::Process` and never names an OS. [A] [process.rs L171](https://github.com/rust-lang/rust/blob/1.99.1/library/std/src/process.rs#L171)
- What differs is exposed per OS, through extension traits: `std::os::unix::process::CommandExt::pre_exec` and `process_group` [A] [L110](https://github.com/rust-lang/rust/blob/1.99.1/library/std/src/os/unix/process.rs#L110), and `std::os::windows::process::CommandExt::creation_flags` [A] [L175](https://github.com/rust-lang/rust/blob/1.99.1/library/std/src/os/windows/process.rs#L175).
- `cfg_select!` is stable since 1.95.0. [A] [core/src/macros/mod.rs L233](https://github.com/rust-lang/rust/blob/1.99.1/library/core/src/macros/mod.rs#L233). purba's nightly has it. [I] 1.101 > 1.95.

**mio: `sys/` modules with a written contract.**
- `src/sys/mod.rs` opens with a list of "Required types" each platform module must define (`Event`, `Events`, `Selector`, `Waker`, …). [A] [L1-L14](https://github.com/tokio-rs/mio/blob/v1.2.4/src/sys/mod.rs#L1-L14)
- It then selects `mod unix; pub use self::unix::*;` or `mod windows; pub use self::windows::*;`. [A] [L53-L67](https://github.com/tokio-rs/mio/blob/v1.2.4/src/sys/mod.rs#L53-L67)
- OS crates come from target tables: `libc` for unix, `windows-sys` for windows. [A] [Cargo.toml L49-L53](https://github.com/tokio-rs/mio/blob/v1.2.4/Cargo.toml#L49-L53)
- No trait enforces the contract. Each target's compile does. [I] Basis: the comment is the only statement of the contract.

**tokio: a public API per OS, with one small common function.**
- `pub mod unix; pub mod windows;`, plus a common `ctrl_c`. [A] [signal/mod.rs L48-L57](https://github.com/tokio-rs/tokio/blob/tokio-1.54.1/tokio/src/signal/mod.rs#L48-L57)
- `signal/unix.rs` is `#![cfg(unix)]`. [A] [L6](https://github.com/tokio-rs/tokio/blob/tokio-1.54.1/tokio/src/signal/unix.rs#L6)
- `signal/windows.rs` is `#![cfg(any(windows, docsrs))]`. It uses `#[cfg(windows)] #[path = "windows/sys.rs"] mod imp;` and a `windows/stub.rs` so docs build on Unix. [A] [L8-L22](https://github.com/tokio-rs/tokio/blob/tokio-1.54.1/tokio/src/signal/windows.rs#L8-L22)
- The common `ctrl_c` is a thin cfg alias: `#[cfg(unix)] use super::unix::{self as os_impl};`. [A] [ctrl_c.rs L1-L4](https://github.com/tokio-rs/tokio/blob/tokio-1.54.1/tokio/src/signal/ctrl_c.rs#L1-L4)
- This is the honest shape for divergent behaviour: SIGHUP has no Windows twin, and `ctrl_break` has no Unix twin. [I] Basis: the two modules export different functions.

**ctrlc: one crate, a `platform/` module per OS, external OS crates by target.**
- `nix` under `cfg(unix)`, `windows-sys` under `cfg(windows)`. [A] [Cargo.toml L17-L24](https://github.com/Detegr/rust-ctrlc/blob/3.5.1/Cargo.toml#L17-L24)
- `#[cfg(unix)] mod unix; … #[cfg(unix)] pub use self::unix::*;`. [A] [platform/mod.rs L10-L20](https://github.com/Detegr/rust-ctrlc/blob/3.5.1/src/platform/mod.rs#L10-L20)
- It does **not** use a separate crate per OS. The brief assumed otherwise.

**notify: a trait, one backend module per OS, and a cfg type alias.**
- Backend crates (`inotify`, `kqueue`, `objc2-core-foundation`, `windows-sys`) are target dependencies. The backends themselves are modules. [A] [notify/Cargo.toml L37-L71](https://github.com/notify-rs/notify/blob/notify-8.2.1/notify/Cargo.toml#L37-L71)
- `pub trait Watcher` [A] [lib.rs L321](https://github.com/notify-rs/notify/blob/notify-8.2.1/notify/src/lib.rs#L321) and `impl Watcher for ReadDirectoryChangesWatcher` [A] [windows.rs L562](https://github.com/notify-rs/notify/blob/notify-8.2.1/notify/src/windows.rs#L562).
- `#[cfg(target_os = "windows")] pub type RecommendedWatcher = ReadDirectoryChangesWatcher;`. [A] [lib.rs L407-L437](https://github.com/notify-rs/notify/blob/notify-8.2.1/notify/src/lib.rs#L407-L437)

**nextest-runner: a test runner, the closest match to purba.**
- `#[cfg(unix)] #[path = "unix.rs"] mod os;` and `#[cfg(windows)] #[path = "windows.rs"] mod os;`. [A] [runner/mod.rs L19-L25](https://github.com/nextest-rs/nextest/blob/cargo-nextest-0.9.151/nextest-runner/src/runner/mod.rs#L19-L25)
- Signal handling is one public `SignalHandler` over an inline `#[cfg(unix)] mod imp` and `#[cfg(windows)] mod imp`. [A] [signal.rs L75-L76, L233-L234](https://github.com/nextest-rs/nextest/blob/cargo-nextest-0.9.151/nextest-runner/src/signal.rs#L75-L234)
- "Double spawn" is a Unix-only process mechanism that uses `pthread_sigmask`. [A] [double_spawn.rs L1-L28](https://github.com/nextest-rs/nextest/blob/cargo-nextest-0.9.151/nextest-runner/src/double_spawn.rs#L1-L28) Windows gets a `#[cfg(not(unix))] mod imp` whose functions return `None` or do nothing. [A] [L202-L232](https://github.com/nextest-rs/nextest/blob/cargo-nextest-0.9.151/nextest-runner/src/double_spawn.rs#L202-L232). This is the shape #694's zygote would take inside one crate. [I]
- OS crates come from target tables (`libc`, `nix`; `windows-sys`, `dunce`). [A] [Cargo.toml L123-L140](https://github.com/nextest-rs/nextest/blob/cargo-nextest-0.9.151/nextest-runner/Cargo.toml#L123-L140)
- nextest runs `cargo clippy --all-features --all-targets` on a matrix that includes `windows-latest`. [A] [ci.yml L23-L48](https://github.com/nextest-rs/nextest/blob/cargo-nextest-0.9.151/.github/workflows/ci.yml#L23-L48)

**uv: separate crates per OS, picked by target table.**
- `uv-windows` is `#![cfg(windows)]`. Its own `windows` dependency is under `[target.'cfg(windows)'.dependencies]`. [A] [lib.rs L1-L6](https://github.com/astral-sh/uv/blob/0.13.7/crates/uv-windows/src/lib.rs#L1-L6), [Cargo.toml](https://github.com/astral-sh/uv/blob/0.13.7/crates/uv-windows/Cargo.toml)
- `uv-unix` is `#![cfg(unix)]` with `nix` under `cfg(unix)`. [A] [lib.rs L1-L5](https://github.com/astral-sh/uv/blob/0.13.7/crates/uv-unix/src/lib.rs#L1-L5)
- The `uv` crate picks them: `[target.'cfg(target_os = "windows")'.dependencies] uv-windows`, `[target.'cfg(unix)'.dependencies] uv-unix`. [A] [crates/uv/Cargo.toml L104-L106, L150-L151](https://github.com/astral-sh/uv/blob/0.13.7/crates/uv/Cargo.toml#L104-L151)
- Its stated reason is sharing: it "provides shared Windows functionality used by both the main `uv` crate and `uv-trampoline`", with a `std` feature for `no_std` use. [A] lib.rs L1-L4. `uv-trampoline` uses it with `default-features = false` [A] [uv-trampoline/Cargo.toml L40](https://github.com/astral-sh/uv/blob/0.13.7/crates/uv-trampoline/Cargo.toml#L40), and is excluded from the workspace because it "Needs nightly". [A] [Cargo.toml L1-L7](https://github.com/astral-sh/uv/blob/0.13.7/Cargo.toml#L1-L7)
- uv still has a `clippy-windows` job on a Windows runner beside `clippy-ubuntu`. [A] [check-lint.yml L104-L160](https://github.com/astral-sh/uv/blob/0.13.7/.github/workflows/check-lint.yml#L104-L160). So the split did not replace a Windows lint.
- uv has 91 crates [P] (`git/trees` listing). One more crate costs it little. purba has 9. [P] `ls crates`.

**Not read:** crossterm and rustix. rustix has no Windows backend, so it is not relevant here. [I]

#### What each pattern costs and buys

| Question | Inline cfg | `os` module per OS | API per OS | Trait + alias | Crate per OS |
|---|---|---|---|---|---|
| Linux check of the Windows arm | `--target` [P] | `--target` [P] | `--target` [I] | `--target` [I] | `--target` [P] |
| One place to read the Windows code | no | yes | yes | yes | yes |
| Shared signature enforced by | caller compile per target [P] | caller compile per target [P] | nothing; signatures differ by design | the trait, per target | caller compile per target [P] |
| Unused-on-this-OS code flagged | yes (`dead_code`) | yes [P] | no if `pub` | no if `pub` | no: always `pub` [P] |
| Where the Windows tests live | `#[cfg(all(test, windows))]` beside the code | `sys/windows.rs` | per module | per impl | the crate's own tests. These run only on Windows. Its `lib test` is empty on Linux [P]. |
| Extra manifests | 0 | 0 | 0 | 0 | 1 per OS |
| A second consumer can depend on only the Windows part | no | no | no | no | yes (uv's reason) [A] |

### 4. What this means for purba

#### Today

- There are 15 `cfg(` lines in `crates/execution/src/interrupt.rs` and 2 in `crates/purba/src/lib.rs`. [P] `grep -c`. Integration tests add three `cfg(unix)` gates: `interrupt_load_waiting.rs`, `interrupt_term.rs`, `run.rs:223`. [P]
- The Windows arms already type-check and lint from Linux, with one extra C compiler. [P]
- CI never lints them with clippy. [A] (quality.yml on ubuntu only; build.yml's Windows job runs tests.)

#### What a split CHANGES

- **Where the code lives.** It moves into a crate with its own manifest, `lib.rs` and test target.
- **The visibility of the code.** Platform functions become `pub`, so `dead_code` stops flagging the ones an OS no longer uses. [P]
- **The dependency graph per target.** `windows-sys` or `libc` features would sit in the platform crate's target table. The same effect is available in one crate: move `libc` in `crates/execution/Cargo.toml` to `[target.'cfg(unix)'.dependencies]`. [A] specifying-dependencies.md L424-L451.
- **Manifest rules.** Each crate needs its root `#![cfg(...)]` and its own OS dependencies in a target table. Without the root cfg, `cargo clippy --all-targets` on Linux breaks, and that is purba's `lint:rust`. [P]
- **The record that settles crate placement.** purba's root `Cargo.toml` names `docs/decisions/every-crate-lives-under-crates.md`. A new crate goes under `crates/`, and the `crates/*` glob makes it a member automatically. [A] [Cargo.toml L1-L3](https://github.com/kalonji-tools/purba/blob/b59f42f9ec59b1796631818e6eabf4e07c94a796/Cargo.toml#L1-L3) [I] I did not read that record.

#### What a split does NOT change

- **Whether a Linux developer can type-check the Windows arm.** They can today, in any layout, with `--target x86_64-pc-windows-gnu`. [P]
- **The lockfile and the licence gate.** Both already cover every target. [A] [P]
- **The behaviour divergence.** On Unix, `die_by` raises the signal. On Windows, it exits with `STATUS_CONTROL_C_EXIT`. Under #694, Unix forks from a zygote and Windows spawns once per test, so the module body runs once per test there. [A] #694 body. No layout makes the two arms alike.
- **The need for a Windows runner to run tests.** A cross check compiles and does not run. The `#[cfg(all(test, windows))]` tests in `interrupt.rs` only run on `windows-latest`. [I] Basis: no Windows execution environment was involved in any probe.
- **The decisions.** A record still has to say what Windows does: which signals exist, which exit code, and how many times the module body runs. A crate cannot state that.

#### When does a split pay?

- A split pays when a second consumer needs only the Windows part. That is uv's documented reason. [A] purba has no such consumer. [I] Basis: the 9 crates under `crates/` are one product.
- A split also pays when the platform code is big enough to need its own `[lints]` or its own `[features]`. #694 does not need either. [I] Basis: the issue's Done-when lines are all behaviour.
- #694 makes the arms *larger*, not *shared*. The Unix arm gains fork, pipes and a wait loop. The Windows arm gains spawn per test. nextest has the same split, a Unix-only double spawn with a no-op Windows stub, and keeps it inside one crate. [A] [I]

#### The smallest change that gives the main benefit

The main benefit is a check of the Windows arm. Three ways to get it, by what each does:

**Lint the Windows target on the existing Windows runner.** This adds clippy to the `windows-latest` job, as nextest and uv do. [A]
```yaml
# .github/workflows/build.yml, rust job, after "Run the Rust tests"
- name: Lint the Windows arm
  if: runner.os == 'Windows'
  run: mise exec -- cargo clippy --all-targets --locked
```
Cost: none for a developer. It reuses a job that already compiles. [I]

**Lint the Windows target from Linux.** This runs on any Linux machine and in `quality.yml`. [P]
```sh
rustup target add x86_64-pc-windows-gnu
CC_x86_64_pc_windows_gnu=<mingw or zig-cc wrapper> \
  cargo clippy --all-targets --locked --target x86_64-pc-windows-gnu
```
Cost: a MinGW C compiler for `stacker`, and the target std. `--target` builds use a separate `target/x86_64-pc-windows-gnu/` directory, so the host cache is untouched. [P] (directory seen in the probe). It also checks `cargo doc --target …` links. [P]

**Keep one `os` module per OS inside `purba-execution` when #694 lands.** This is a layout change. It adds no check, and it collects the arms.
```rust
// crates/execution/src/lib.rs
#[cfg_attr(unix, path = "os/unix.rs")]
#[cfg_attr(windows, path = "os/windows.rs")]
mod os;   // os::run_isolated(..), os::die_by(..), os::writable_stdout(..)
```
or the std form, which also gives a compile error on any other OS:
```rust
cfg_select! {
    unix => { mod unix; use unix as os; }
    windows => { mod windows; use windows as os; }
}
```
Cost: none in manifests. A private `os` keeps the `dead_code` signal. [P]

A `purba-platform` crate, or one crate per OS, gives none of these three benefits more strongly. It adds manifests and loses `dead_code`. [P] [I]

### Probe log

Environment: `RUSTUP_TOOLCHAIN=nightly-2026-11-02`, cargo `1.101.0-nightly (31d3246e0 2026-10-27)`. No `python` or `python3` on PATH. The probe workspace is `<scratch>/probe-split/`. Its root `Cargo.toml` copies purba's `[workspace.lints.*]` tables verbatim, and `.cargo/config.toml` copies `[build] warnings = "deny"`. `run.sh` wraps cargo with the pinned toolchain.

Crates: `probe-core` (target tables → `probe-platform-unix` | `probe-platform-windows`), `probe-platform-unix` (`#![cfg(unix)]`, `libc`), `probe-platform-windows` (`#![cfg(windows)]`, `windows-sys`, `std::os::windows`), `probe-inline` (one crate, `#[cfg_attr(…, path = "sys/….rs")] mod sys;`), `probe-py` (pyo3 `=0.29.2` + `abi3-py312`, `cdylib`+`rlib`, as `crates/purba`), and `probe-py-noabi` (pyo3 without abi3, as `crates/execution`).

#### Setup
```
$ rustup target add --toolchain nightly-2026-11-02 x86_64-pc-windows-gnu x86_64-pc-windows-msvc
info: downloading 2 components
```

#### Split workspace, host
```
$ cargo build --workspace
     Locking 3 packages ...
   Compiling probe-platform-windows v0.0.0 (…)        # compiled for Linux, empty
   Compiling probe-platform-unix v0.0.0 (…)
   Compiling probe-core v0.0.0 (…)
    Finished `dev` profile … exit=0
$ cargo clippy --workspace --all-targets                → Finished, exit=0
$ grep '^name = ' Cargo.lock
libc, probe-core, probe-platform-unix, probe-platform-windows, windows-link, windows-sys
$ cargo tree -e normal
probe-core → probe-platform-unix → libc
probe-platform-windows                                  # member root, no deps on Linux
$ cargo tree -e normal --target all
probe-core ├── probe-platform-unix → libc
           └── probe-platform-windows → windows-sys → windows-link
$ cargo tree -e normal --target x86_64-pc-windows-msvc
probe-core → probe-platform-windows → windows-sys → windows-link
```

#### Windows target from Linux
```
$ cargo check  --workspace --target x86_64-pc-windows-gnu               → exit=0
$ cargo clippy --workspace --all-targets --target x86_64-pc-windows-msvc → exit=0
```

#### Control: does `cargo::unused_dependencies` fire?
```
# libc added unused to probe-core [dependencies] (libc is also transitive via platform-unix)
$ cargo check --workspace                              → exit=0   (skipped: transitive)
# windows-sys added unused instead (not transitive on Linux)
$ cargo check --workspace
error: unused dependency `windows-sys`
  --> crates/core/Cargo.toml:11:1
   = note: `cargo::unused_dependencies` is set to `deny` in `[lints]`
exit=101
$ cargo check --workspace --target x86_64-pc-windows-gnu  → exit=0  (transitive there)
$ cargo -Zcargo-lints check --workspace
warning: flag `-Z cargo-lints` has been stabilized in the 1.100 release
```

#### E2: type error only in the Windows crate (`0xC000_013A` → `"0xC000_013A"`)
```
$ cargo check  --workspace --all-targets               → exit=0   (control: host)
$ cargo clippy --workspace --all-targets               → exit=0   (control: host)
$ cargo check  --workspace --target x86_64-pc-windows-gnu
error[E0308]: mismatched types
 --> crates/platform-windows/src/lib.rs:9:5
8 | pub const fn stop_code(_signum: i32) -> u32 {
9 |     "0xC000_013A"  expected `u32`, found `&str`
exit=101
```

#### One-crate `sys` module variant: dead code per target
```
$ cargo clippy --workspace --all-targets
error: function `interrupt` is never used   --> crates/inline/src/sys/unix.rs:11:14
exit=101
$ cargo clippy --workspace --all-targets --target x86_64-pc-windows-gnu
error: function `stdout_is_null` is never used  --> crates/inline/src/sys/windows.rs:13:8
exit=101
# the same two functions as `pub fn` in probe-platform-{unix,windows}: no error on either target
# after `#[cfg(unix)] pub use sys::interrupt; #[cfg(windows)] pub use sys::stdout_is_null;` → both exit=0
```

#### E3: the same type error in `sys/windows.rs`
```
$ cargo clippy --workspace --all-targets               → exit=0
$ cargo check --workspace --target x86_64-pc-windows-gnu
error[E0308]: mismatched types  --> crates/inline/src/sys/windows.rs:8:5
exit=101
```

#### E5: signature drift (Windows arm takes `i64`)
```
$ cargo clippy --workspace --all-targets               → exit=0
$ cargo check --workspace --target x86_64-pc-windows-gnu
error[E0308]: mismatched types --> crates/inline/src/lib.rs:16:34   (caller)
error[E0308]: mismatched types --> crates/core/src/lib.rs:11:34     (caller)
exit=101
```

#### E1: Windows crate without `#![cfg(windows)]`
```
$ cargo build --workspace
error[E0433]: cannot find `windows` in `os`               --> crates/platform-windows/src/lib.rs:3:14
error[E0433]: cannot find module or crate `windows_sys`   --> …:15:29
error[E0599]: no method named `as_raw_handle` found for struct `Stdout`
exit=101
$ cargo build -p probe-core                             → exit=0
# with default-members = ["crates/core","crates/inline","crates/platform-unix"]
$ cargo build                                           → exit=0
$ cargo clippy --all-targets                            → exit=0
$ cargo build --workspace                               → error[E0433] …, exit=101
$ cargo check --target x86_64-pc-windows-gnu            → exit=0
```

#### No `[workspace.target]` table
```
$ cargo check --workspace      # with [workspace.target.'cfg(windows)'.dependencies]
warning: Cargo.toml: unused manifest key: workspace.target
    Finished … exit=0
```

#### Feature unification per target
```
# platform-unix: [dependencies] libc ; [target.'cfg(windows)'.dependencies] libc features=["extra_traits"]
$ cargo tree -e features -i libc --target x86_64-unknown-linux-gnu | grep extra_traits   → (no match)
$ cargo tree -e features -i libc --target x86_64-pc-windows-gnu    | grep extra_traits
├── libc feature "extra_traits"
│   └── probe-platform-unix …
```

#### pyo3 in the graph
```
$ cargo clippy --workspace --all-targets               # host, no python
error: failed to run custom build command for `pyo3-ffi v0.29.2`
  error: no Python 3.x interpreter found
$ PYO3_PYTHON=~/.local/share/mise/installs/python/3.12.14/bin/python cargo clippy --workspace --all-targets
    Finished … exit=0
$ cargo clippy --workspace --all-targets --target x86_64-pc-windows-gnu   # no python, no PYO3_*
error: this could be a `const fn`  --> crates/py/src/lib.rs:12:5
   = note: `-D clippy::missing-const-for-fn` implied by `-D clippy::nursery`
exit=101            # a Windows-only lint: the Windows `stop_code` was `const fn`, the Unix one not
$ cargo clippy --workspace --all-targets --target x86_64-pc-windows-msvc  → same lint, exit=101
$ cargo check -p probe-py-noabi --target x86_64-pc-windows-gnu
  error: PYO3_CROSS_PYTHON_VERSION or either an abi3-py3* or abi3t-py3* feature must be specified
  when cross-compiling and PYO3_CROSS_LIB_DIR is not set.
$ PYO3_CROSS_PYTHON_VERSION=3.12 cargo check -p probe-py-noabi --target x86_64-pc-windows-gnu → exit=0
# after matching constness (#[expect(clippy::missing_const_for_fn)] on the Windows arm):
$ cargo clippy --workspace --all-targets [--target x86_64-pc-windows-gnu | -msvc]  → exit=0 on all three
```

#### rustdoc and tests with an empty-on-Linux crate
```
$ cargo doc --no-deps --workspace                       → exit=0; probe_platform_windows/all.html lists 0 fns
$ cargo test --workspace                                → every target "0 passed", exit=0
# broken link `[`NoSuchItem`]` planted in the Windows crate
$ cargo doc --no-deps --workspace                       → exit=0
$ cargo doc --no-deps --workspace --target x86_64-pc-windows-gnu
error: unresolved link to `NoSuchItem`  --> crates/platform-windows/src/lib.rs:12:47
exit=101
```

#### purba's real tree, from Linux
Clone: `git clone --shared` of purba at `b59f42f` into `<scratch>/purba-wincheck`. The purba checkout was not modified. `git status` there was clean afterwards.
```
$ cargo clippy --all-targets --locked --target x86_64-pc-windows-gnu
error: failed to run custom build command for `stacker v0.1.25`
  error occurred in cc-rs: failed to find tool "x86_64-w64-mingw32-gcc"
exit=101
$ cargo tree --target x86_64-pc-windows-gnu -i stacker -e normal
stacker v0.1.25 └── ruff_python_parser v0.0.17 └── purba-parsing …
# stacker/src/arch/windows.c: #include <windows.h>
$ nix-shell -p zig        # zig 0.16.0 into the nix store
# wrapper $W/cc: drops cc-rs's --target=<rust triple>, then `zig cc -target x86_64-windows-gnu "$@"`
$ CC_x86_64_pc_windows_gnu=$W/cc AR_x86_64_pc_windows_gnu=$W/ar \
  cargo clippy --all-targets --locked --target x86_64-pc-windows-gnu
    Finished `dev` profile … exit=0          # no python, no PYO3_* set
```
Controls on the clone, each reverted with `git checkout`:
```
# die_by: const STATUS_CONTROL_C_EXIT: u32 = "0xC000_013A";
$ PYO3_PYTHON=… cargo clippy --all-targets --locked                → exit=0   (host control)
$ cargo clippy --all-targets --locked --target x86_64-pc-windows-gnu
error[E0308]: mismatched types  --> crates/execution/src/interrupt.rs:377:44
exit=101
# die_by: #[expect(clippy::cast_possible_wrap, …)] removed
$ cargo clippy --all-targets --locked --target x86_64-pc-windows-gnu
error: casting `u32` to `i32` may wrap around the value  --> crates/execution/src/interrupt.rs:380:28
   = note: `-D clippy::cast-possible-wrap` implied by `-D clippy::pedantic`
exit=101
$ cargo check  --all-targets --locked --target x86_64-pc-windows-gnu   → exit=0   (rustc alone misses it)
# one crate alone
$ cargo clippy -p purba-execution --all-targets --locked --target x86_64-pc-windows-gnu
error: failed to run custom build command for `pyo3-ffi v0.29.2`   (PYO3_CROSS… needed)
$ PYO3_CROSS_PYTHON_VERSION=3.12 cargo clippy -p purba-execution … --target x86_64-pc-windows-gnu → exit=0
# msvc
$ cargo clippy --all-targets --locked --target x86_64-pc-windows-msvc
error: failed to run custom build command for `psm v0.1.32`   (cc-rs: "cc" … x86_64_windows_gnu.s)
error: failed to run custom build command for `stacker v0.1.25`
```

#### Cleanup notes
- Undo the targets: `rustup target remove --toolchain nightly-2026-11-02 x86_64-pc-windows-gnu x86_64-pc-windows-msvc`.
- zig sits in the nix store as `/nix/store/p3jg4qds0w03309wkbp19yylmr3pmd5i-zig-0.16.0`. It is not in any profile, and `nix-collect-garbage` removes it.
- Scratch paths to delete when done: `<scratch>/{probe-split,purba-wincheck,wincc,wincheck-gnu.log,host.log}`.

## Part 3: How other projects handle Windows

Survey date: 2026-11-07. Scope: support policy, the stated reason, the cost, and what followed.
Evidence tags: **[A]** read in the primary source. **[B]** secondary source (named). **[I]** my inference (basis given).
I did not touch any repo or any GitHub write API. Reads used `WebFetch` and `gh api` (read-only).

### Answer

Most process-heavy tools that support Windows did not port `fork` to it. They made `fork` an optimization on Unix and used a slower spawn or exec path on Windows: `multiprocessing`, Chromium's zygote, Ray, and cargo-nextest all do this. Each one also documents the signal features Windows lacks. The tools whose central design *is* `fork` or POSIX signals mostly never shipped native Windows: gunicorn, uvloop, pytest-forked, uWSGI, the Ansible control node, and Celery's prefork pool. Their issues stay open for years, and third parties fill the gap with forks (winloop, pyenv-win, nvm-windows, Memurai). Every formal tier policy surveyed (Rust, CPython, Go, Node, uv) puts x86_64 Windows in its top tier. Rust, CPython and Go also tie each tier to a minimum number of named maintainers. One project, Celery, dropped Windows for the stated reason "unable to diagnose issues on this platform", and it still runs a small Windows CI job today. I found no project that dropped Windows and later restored full support. Bun is the only project here that went from "WSL only" to native support, and it took a funded team plus an "experimental" release first.

| Strategy (what it does) | Example projects | What Windows users get | What maintainers pay |
|---|---|---|---|
| **Full parity, platform code split by `cfg` inside one crate or package** | cargo-nextest, uv, ruff, rustup, pytest, pytest-xdist/execnet, tox, nox, coverage.py | Every feature. Often slower process start. | Windows CI on every PR. Two implementations of the process/signal layer. Windows expertise ("could benefit from Windows expertise" — nextest). |
| **Windows gets a slower fallback path** (fork on Unix, spawn/exec on Windows) | `multiprocessing` (spawn only), Chromium (zygote on Linux, exec on Windows/Mac), Ray ("more memory"), cargo-nextest (process per test, job objects) | Same results, slower start, more memory. | A second start path that must give the same results. Tests on both. |
| **Supported, with features documented as missing on Windows** | pytest-timeout (no `signal` method), coverage.py (`sigterm` "has no effect"), asyncio (no `add_signal_handler`), Python `signal`, tokio `signal::windows`, signal-hook, uv (ignores Ctrl-C), nextest (no SIGTSTP, no graceful kill) | Core works. Named features are absent or weaker. | One doc line or "Availability" marker per feature. A fallback such as a thread timer. |
| **Formal tiers** (promise per platform) | Rust, CPython PEP 11, Go, Node.js, uv | A known promise: tested, or only built, or best effort. | Named maintainers per tier (Rust: 3 for Tier 1, 2 for Tier 2). CI per tier. |
| **Build on Windows, test only on Linux** (a "Tier 2") | Rust Tier 2 targets, uv on Windows arm64, Rust's `i686-pc-windows-gnu` after 1.88 | Binaries that compile. Bugs found by users. | A build job only. Rust warns such a target "will likely accumulate bugs faster". |
| **Move the Unix-only feature into its own package** | pytest-xdist moved `--boxed` into pytest-forked (2017), removed it in 3.0.2 (2022) | The main tool works. The fork feature is absent. | The split-off package was later orphaned: pytest-forked is in "maintenance mode". |
| **Unsupported, "may work", patches welcome** | Celery ≥ 4.0, uvloop, gunicorn | Workarounds (`-P solo`, an env var), or nothing. | Recurring issues. Issue threads locked or open 10+ years. |
| **WSL or a POSIX layer only** | Ansible control node, pyenv, nvm, Pants, fish (WSL/Cygwin/MSYS2), Redis (WSL), uWSGI (Cygwin) | A Linux build inside a VM. No native Windows behaviour. | Nothing, or one docs paragraph. Some even disclaim WSL ("not supported by Ansible"). |
| **A third party ships the Windows port** | pyenv-win, nvm-windows, winloop (uvloop), Microsoft's Redis port → archived → Memurai | A separate tool with its own bugs and its own release cadence. | Nothing upstream. Upstream loses control of the Windows story. |

### Survey

Policy codes: **parity**, **fallback** (slower Windows path), **degraded** (features documented missing), **tiered**, **unsupported**, **WSL/POSIX-layer**, **dropped**, **late-added**.

#### Python test and process tools

| Project + version | Policy | Reason given (quote) | Cost / what followed | Source |
|---|---|---|---|---|
| pytest 9.2.0 | parity | none stated | [A] CI tests `windows-latest` on Python 3.10–3.15 beside Ubuntu and macOS. | [test.yml@9.2.0](https://github.com/pytest-dev/pytest/blob/9.2.0/.github/workflows/test.yml) |
| pytest-xdist 3.8.0 / execnet 2.1.2 | parity | execnet: "interoperable between Windows and Unix-ish systems" [A] | [A] Both CI on `ubuntu-latest` + `windows-latest`. [A] xdist moved the fork-based `--boxed` out: "`--boxed` functionality has been moved to a separate plugin" (1.19.0, 2017-08-09); deprecated 2.5.0 (2021-12-10); "has been removed" 3.0.2 (2022-10-25). [I] Workers are spawned interpreters, not forks, so Windows needs no second path. | [CHANGELOG@v3.8.0](https://github.com/pytest-dev/pytest-xdist/blob/v3.8.0/CHANGELOG.rst), [execnet README@v2.1.2](https://github.com/pytest-dev/execnet/blob/v2.1.2/README.rst) |
| pytest-forked 1.6.0 | unsupported | "This plugin does not work on Windows because there's no fork support." [A] | [A] CI is Ubuntu only. [A] "future maintenance beyond the bare minimum is not planned until a new maintainer is found." | [README@v1.6.0](https://github.com/pytest-dev/pytest-forked/blob/v1.6.0/README.rst) |
| pytest-timeout 2.5.0 | degraded | "If the system supports the SIGALRM signal the *signal* method will be used by default." Thread is "the default on systems not supporting the *signal* method." [A] | [A] The thread method kills via `os._exit()`: "no teardown, JUnit XML output etc." [A] README never names Windows; it names the capability (SIGALRM). [A] CI runs on `windows-latest`. | [README@2.5.0](https://github.com/pytest-dev/pytest-timeout/blob/2.5.0/README.rst) |
| CPython `multiprocessing` 3.14 | fallback | fork: "safely forking a multithreaded process is problematic." 3.14 default changed "from *fork* to *forkserver* to retain the performance but avoid common multithreaded process incompatibilities." [A] | [A] spawn is "Available on POSIX and Windows platforms. The default on Windows and macOS." fork and forkserver are POSIX only. [A] macOS moved to spawn in 3.8 because fork "can lead to crashes". [I] Even Unix moved away from plain fork as the default. | [docs 3.14](https://docs.python.org/3.14/library/multiprocessing.html#contexts-and-start-methods) |
| CPython `signal` / asyncio 3.14 | degraded | asyncio "is designed to be portable, but some platforms have subtle differences and limitations" [A] | [A] On Windows `signal()` accepts only SIGABRT, SIGFPE, SIGILL, SIGINT, SIGSEGV, SIGTERM, SIGBREAK; others raise `ValueError`. [A] `alarm`, `setitimer`, `pthread_kill`, `sigwait` carry "Availability: Unix." [A] asyncio: `add_signal_handler()` "not supported" on all Windows loops. | [signal](https://docs.python.org/3.14/library/signal.html), [asyncio-platforms](https://docs.python.org/3.14/library/asyncio-platforms.html) |
| coverage.py 7.17.2 | parity + one degraded feature | none stated | [A] `[run] sigterm`: "The signal handler is only registered on Linux and Mac. On Windows, this setting has no effect." [A] Test matrix: ubuntu, macos, windows. | [config.rst@7.17.2](https://github.com/nedbat/coveragepy/blob/7.17.2/doc/config.rst), [testsuite.yml@7.17.2](https://github.com/nedbat/coveragepy/blob/7.17.2/.github/workflows/testsuite.yml) |
| tox 4.67.13 / nox 2026.10.20 | parity | none stated | [A] CI includes `windows-2025` (tox) and `windows-latest` (nox). | [tox check.yaml](https://github.com/tox-dev/tox/blob/4.67.13/.github/workflows/check.yaml), [nox ci.yml](https://github.com/wntrblm/nox/blob/2026.10.20/.github/workflows/ci.yml) |
| pytest-subprocess | not checked | — | I did not open this source. | — |

#### Rust tools

| Project + version | Policy | Reason given (quote) | Cost / what followed | Source |
|---|---|---|---|---|
| cargo-nextest 0.9.151 | parity + fallback + degraded | "Windows has a much simpler signal model than Unix." "Job objects don't support graceful termination like Unix signals do." [A] | [A] One crate, cfg-split: `runner/unix.rs` 246 lines, `runner/windows.rs` 224 lines, shared `runner/imp.rs` 942 lines. [A] Not on Windows: per-test process groups, `WM_CLOSE`, SIGTSTP pause. Code has `// TODO: set process group on Windows for better ctrl-C handling.` [A] Docs: "process creation is significantly slower on Windows … Because nextest creates a process for every test, this can result in a significant performance penalty on Windows." [A] "an area where nextest could benefit from Windows expertise." [A] CI tests `windows-latest`. | [windows.md](https://github.com/nextest-rs/nextest/blob/cargo-nextest-0.9.151/site/src/docs/installation/windows.md), [signal-handling.md](https://github.com/nextest-rs/nextest/blob/cargo-nextest-0.9.151/site/src/docs/design/architecture/signal-handling.md), [runner/windows.rs](https://github.com/nextest-rs/nextest/blob/cargo-nextest-0.9.151/nextest-runner/src/runner/windows.rs) |
| uv 0.13.7 | tiered (Windows x86_64 Tier 1) + degraded signals | Tier 1 "can be thought of as 'guaranteed to work'" [A] | [A] `uv run`: on Unix uv forwards most signals; "On Windows, these concepts do not apply and uv ignores Ctrl-C events, deferring handling to the child process so it can exit cleanly." [A] Many Windows CI jobs (x86, aarch64, nanoserver, registry). | [platforms.md@0.13.7](https://github.com/astral-sh/uv/blob/0.13.7/docs/reference/policies/platforms.md), [run.md@0.13.7](https://github.com/astral-sh/uv/blob/0.13.7/docs/concepts/projects/run.md) |
| ruff 0.17.4 | parity | no tier doc found | [A] `cargo-test-other` job runs `cargo test` on Windows 2022 and macOS. [I] Ruff spawns few processes, so it is a weak comparison. | [ci.yaml@0.17.4](https://github.com/astral-sh/ruff/blob/0.17.4/.github/workflows/ci.yaml) |
| rustup 1.29.3 | parity | none stated | [A] CI covers windows msvc, gnu, gnullvm. [A] Platform split by file: `src/cli/self_update/unix.rs` and `windows.rs`. | [ci.yaml@1.29.3](https://github.com/rust-lang/rustup/blob/1.29.3/.github/workflows/ci.yaml) |
| tokio 1.48.0 | degraded (module per OS) | "This module is only defined on Windows" [A] | [A] `tokio::signal::windows` offers `ctrl_c`, `ctrl_break`, `ctrl_close`, `ctrl_logoff`, `ctrl_shutdown`; docs mark it "Available on Windows". [I] Same crate, a separate module per OS. | [docs.rs 1.48.0](https://docs.rs/tokio/1.48.0/tokio/signal/windows/index.html) |
| signal-hook 0.3.18 | degraded | "It should work on any POSIX.1-2001 system, which are all the major big OSes with the notable exception of Windows." [A] | [A] "limited support for Windows", "Many parts of the library are not available there", "SIGTERM isn't actually used and not all Ctrl-Cs are turned into SIGINT." | [docs.rs 0.3.18](https://docs.rs/signal-hook/0.3.18/signal_hook/) |
| process-wrap 10.0.1 | parity via per-OS wrappers | — | [A] `ProcessGroup` wrapper is "(Unix-only)"; `JobObject` wrapper is Windows-only and "always sets the `CREATE_SUSPENDED` flag." Successor to `command-group`. | [docs.rs 10.0.1](https://docs.rs/process-wrap/10.0.1/process_wrap/) |

#### Projects that never supported, or dropped, Windows

| Project + version | Policy | Reason given (quote) | Cost / what followed | Source |
|---|---|---|---|---|
| Celery 4.0 (2016) → 5.7.0 | dropped; still "No" | 4.0: "Microsoft Windows is no longer supported. The test suite is passing, and Celery seems to be working with Windows, but we make no guarantees as we are unable to diagnose issues on this platform." FAQ: "due to lack of resources. But it may still work and we are happy to accept patches." [A] | [A] #4178 (2017): "We do not have an active maintainer that is interested in providing support for Windows." "best-effort basis." [A] #4081: workaround `FORKED_BY_MULTIPROCESSING=1` (18 👍); maintainer in 2022: "in practice I will suggest to move to any unix-like system". [A] Windows CI job added in 2021 (PR #6271, commit `bb80305627`); at v5.7.0 it runs only Python 3.14 on Windows. [B] simplethread.com: prefork pool fails on Windows; users run `-P solo`/threads/eventlet. | [whatsnew-4.0.rst@v5.7.0](https://github.com/celery/celery/blob/v5.7.0/docs/history/whatsnew-4.0.rst), [faq.rst@v5.7.0](https://github.com/celery/celery/blob/v5.7.0/docs/faq.rst), [#4178](https://github.com/celery/celery/issues/4178), [#4081](https://github.com/celery/celery/issues/4081) |
| gunicorn 23.0.0 | never supported | README: "a Python WSGI HTTP Server for UNIX." benoitc (2021): "windows is a really low priority for me." [A] | [A] Issue #524 open since 2013-05-01. Windows support was promised for R18, then R20; both slipped. A 2021 PR (#2643) is not merged. Last comment 2024: "Any news here?" [A] CI has no Windows job. | [README@23.0.0](https://github.com/benoitc/gunicorn/blob/23.0.0/README.rst), [#524](https://github.com/benoitc/gunicorn/issues/524) |
| uvloop 0.21.0 | never supported | `setup.py`: `raise RuntimeError('uvloop does not support Windows at the moment')`. 1st1 (2020): "It doesn't make sense to work on this during my work time." [A] | [A] Issue #14 open since 2016-05-06; locked 2020-01-17 "as too heated". [B] (search results, PyPI page) A third party ships **winloop**, a Windows port of uvloop, plus **winuvloop** to pick one per OS. | [setup.py@v0.21.0](https://github.com/MagicStack/uvloop/blob/v0.21.0/setup.py), [#14](https://github.com/MagicStack/uvloop/issues/14) |
| uWSGI 2.0.28 | POSIX layer only | none found | [A] "Supported Platforms" lists "Windows Cygwin" only. [A] README: "The project is in maintenance mode". | [SupportedPlatforms](https://uwsgi-docs.readthedocs.io/en/latest/SupportedPlatforms.html) |
| Redis 7.4.0 | never supported upstream | README: "Redis can be compiled and used on Linux, OSX, OpenBSD, NetBSD, FreeBSD." [A] | [A] Microsoft's port: "This project is no longer being actively maintained", archived 2021-02-20; it recommends Memurai. [A] redis.io today points to Memurai ("official Redis partner for Windows compatibility") or WSL2. | [README@7.4.0](https://github.com/redis/redis/blob/7.4.0/README.md), [microsoftarchive/redis](https://github.com/microsoftarchive/redis), [redis.io Windows page](https://redis.io/docs/latest/operate/oss_and_stack/install/archive/install-redis/install-redis-on-windows/) |
| fish 4.9.3 | WSL or POSIX layer | none stated | [A] README: install "under the WSL", or "using Cygwin or MSYS2". [A] CI does test on `windows-latest`, but inside an MSYS2 shell. | [README@4.9.3](https://github.com/fish-shell/fish-shell/blob/4.9.3/README.rst), [test.yml@4.9.3](https://github.com/fish-shell/fish-shell/blob/4.9.3/.github/workflows/test.yml) |
| Ray 2.60.0 | late-added, beta, fallback | "Windows does not have a copy-on-write forking model, so spinning up new processes can require more memory." [A] | [A] "Ray on Windows is currently in beta." Multi-node clusters "experimental and untested". "Performance on Windows is known to be slower". | [Ray install docs (2.60.0 shown)](https://docs.ray.io/en/latest/ray-overview/installation.html) |
| Ansible (ansible-core 2.16 FAQ) | WSL only, and WSL also unsupported | "Ansible cannot run on Windows as the control node due to API limitations on the platform." [A] | [A] "The Windows Subsystem for Linux is not supported by Ansible and should not be used for production systems." [B] Ansible mailing list (search summary): the worker model uses `fork`. | [windows_faq 2.16](https://docs.ansible.com/projects/ansible-core/2.16/os_guide/windows_faq.html) |
| pyenv 2.6.0 | WSL only → third-party fork | "Pyenv does not officially support Windows and does not work in Windows outside the Windows Subsystem for Linux." [A] | [A] Points users to pyenv-win, "which does install native Windows Python versions." | [README@v2.6.0](https://github.com/pyenv/pyenv/blob/v2.6.0/README.md) |
| nvm 0.40.3 | WSL / POSIX layer → third parties | "`nvm` also supports Windows in some cases." "It should work through WSL … It should also work with GitBash (MSYS) or Cygwin." [A] | [A] Lists nvm-windows, nodist, nvs, "which are neither supported nor developed by us". | [README@v0.40.3](https://github.com/nvm-sh/nvm/blob/v0.40.3/README.md) |
| Pants 2.33 | WSL only | "Pants runs on Windows 10 under the Windows Subsystem for Linux (WSL)" [A] | [A] Issue #11170 "First class Windows support for PEX and Pants" open since 2020-11-12. jsirois (2023): resuming "the Pex side of this but definitely not the Pants side." | [prerequisites](https://www.pantsbuild.org/stable/docs/getting-started/prerequisites), [#11170](https://github.com/pantsbuild/pants/issues/11170) |
| Bun 1.0 → 1.1 | WSL → experimental → late-added | 1.0 (2023-09): "the Windows build is highly experimental. At the moment, only the JavaScript runtime is supported; the package manager, test runner, and bundler have been disabled" [A] | [A] 1.1 (2024-04): "Bun on Windows passes 98% of our own test suite for Bun on macOS and Linux." It needed a new `.bunx` file format because "Shebangs … are not read on Windows". | [bun v1.0](https://bun.sh/blog/bun-v1.0), [bun v1.1](https://bun.sh/blog/bun-v1.1) |
| Chromium zygote | fallback | "A zygote process is one that listens for spawn requests from a main process and forks itself in response." "In the model used on Windows and Mac, renderers are exec'ed as needed from the chrome binary" [A] | [A] Zygote saves "~6 MB and 60 ms/GHz per process" on Linux. [I] The zygote is a Linux optimization; Windows uses exec and still ships. | [zygote.md@130.0.6723.1](https://chromium.googlesource.com/chromium/src/+/refs/tags/130.0.6723.1/docs/linux/zygote.md) |

### Tier policies

Every policy below puts x86_64 Windows in its top tier. Four of the five (all but uv) name people per tier.
[A] Rust: `x86_64-pc-windows-msvc`, `x86_64-pc-windows-gnu` and `aarch64-pc-windows-msvc` are "Tier 1 with Host Tools"; `i686-pc-windows-gnu` is Tier 2 with host tools ([platform-support.md@aad80a2](https://github.com/rust-lang/rust/blob/aad80a2d5ac1d6d08db6cc26e3e6dfc00c02f3e5/src/doc/rustc/src/platform-support.md)).

| Policy | Top tier promise | Middle tier promise | Bottom tier promise | Who must exist | Source |
|---|---|---|---|---|---|
| **Rust target tiers** | Tier 1: "will always build and pass tests"; patches that fail are rejected. | Tier 2: "will always build, but they may or may not pass tests"; build failures block, test failures do not. | Tier 3: "no guarantees … may or may not build"; "must not impose burden on the authors of pull requests". | Tier 1: ≥ 3 target maintainers. Tier 2: ≥ 2. Tier 3: a designated developer to CC. [A] | [target-tier-policy.md@aad80a2](https://github.com/rust-lang/rust/blob/aad80a2d5ac1d6d08db6cc26e3e6dfc00c02f3e5/src/doc/rustc/src/target-tier-policy.md) |
| **CPython PEP 11** | Tier 1: "CI failures block releases." "All core developers are responsible". | Tier 2: needs a reliable buildbot; failures block a release; fix or revert "within 24 hours". | Tier 3: buildbot; failures do not block; "No response SLA to failures." Unsupported: "does not develop and test on such platforms". | Tier 2: ≥ 2 core devs. Tier 3: ≥ 1 core dev. [A] Windows x86 and x86_64 MSVC are Tier 1; arm64 is Tier 2. | [pep-0011.rst@e931180](https://github.com/python/peps/blob/e931180fc33ecad5a82e68ee86928f486c9ec3bb/peps/pep-0011.rst) (page modified 2026-10-31) |
| **Go ports** | First-class: "Broken builds block releases"; "You break it, you fix it, or find someone who can." Google team owns the builder. | Secondary: "A change that breaks a secondary port will not necessarily be rolled back"; port maintainers are responsible. | Broken: "If a port is broken in release 1.N, then the core Go team can choose to remove the port from release 1.N+1." | Any new port: ≥ 2 named maintainers + a builder maintainer. [A] windows/386 and windows/amd64 are first-class. | [go.dev/wiki/PortingPolicy](https://go.dev/wiki/PortingPolicy) (fetched 2026-11-07) |
| **Node.js** | Tier 1: "Test failures on tier 1 platforms will block releases." | Tier 2: also blocks releases; binaries may be delayed. | Experimental: "May not compile or test suite may not pass"; no releases; does not block. | Build Working Group maintains infrastructure. [A] Windows x64 Tier 1, arm64 Tier 2, 8.1 Experimental. | [BUILDING.md@v22.11.0](https://github.com/nodejs/node/blob/v22.11.0/BUILDING.md) |
| **uv** | Tier 1: "continuously built, tested, and developed against". | Tier 2: "continuously built, but the uv test suite is not run on them and stability may vary". | Tier 3: "may not be built or tested, but uv will accept patches". | Not stated. [A] Windows x86_64 Tier 1, arm64 Tier 2, i686 Tier 3. Binaries for Tier 1 and 2 only. | [platforms.md@0.13.7](https://github.com/astral-sh/uv/blob/0.13.7/docs/reference/policies/platforms.md) |

Pattern across the five [I]:
- A tier promises three things: does it **build** in CI, are **tests** run, and does a failure **block** a merge or release.
- In Rust, CPython and Go, every tier above "best effort" requires **named people**. A single maintainer meets only the bottom tier in Rust (Tier 3) and CPython (Tier 3).
- uv copied Rust's wording. That shows the Rust scheme is the reference model for Rust-based Python tools.

### Drops and what followed

| Event | Stated trigger | What followed |
|---|---|---|
| Celery 4.0 drops Windows (2016) | "unable to diagnose issues on this platform"; later "lack of resources" [A] | Years of crash reports with one workaround env var. A maintainer advises moving "to any unix-like system". A small Windows CI job was added back in 2021, but the FAQ still says "No." [A] No full restoration. |
| pytest-xdist moves `--boxed` (fork) out (2017), removes it (2022) | none stated in the changelog [A] | The fork feature lives on in pytest-forked, which says it is in maintenance mode until "a new maintainer is found." [A] |
| Microsoft archives its Redis port (2021-02-20) | none stated beyond "no longer being actively maintained" [A] | Users moved to Memurai (commercial) or WSL. Redis now calls Memurai its "official Redis partner for Windows compatibility." [A] |
| Rust demotes `i686-pc-windows-gnu` to Tier 2 (1.88, 2025) | "do not have any dedicated target maintainers"; CI problems "we have a hard time to debug"; "significantly less usage" [A] | Builds still ship. The blog warns it "will likely accumulate bugs faster in the future because of the reduced testing." [A] ([blog, 2025-05-26](https://blog.rust-lang.org/2025/05/26/demoting-i686-pc-windows-gnu/)) |
| Go marks `windows/arm` broken (1.24) | release note gives none; points to issue #70705 [A] | [B] A Go-source mirror shows a commit "go 1.26 remove support for windows/arm" (#71671). I did not confirm this in the official 1.26 notes. ([go1.24 notes](https://go.dev/doc/go1.24)) |
| Node.js stops 32-bit Windows binaries (v23.0.0, 2024) | [B] hackernoon / dev.to: about 0.27 % of Windows users on 32-bit | I did not open the primary GitHub issue. |
| uvloop (2016→), gunicorn (2013→), Pants (2020→) never add Windows | maintainer time and priority [A] | Issues open 6–13 years. Third parties build Windows ports (winloop) or users use WSL. |
| Bun adds Windows (2023→2024) | demand; company-funded | Seven months from "highly experimental, runtime only" to 98 % of the test suite. It needed new Windows-only machinery (`.bunx`). [A] |

Restorations: I searched for a project that dropped Windows and restored full support. I found none [A for the absence in my searches; not proof of absence]. The nearest cases are Celery's partial CI return and Bun's late add.

### What this means for a project like purba

Everything in this section is **[I] inference** from the survey above.

1. **Tools like purba split into two groups.** Tools that run units in separate processes and still support Windows (nextest, xdist, multiprocessing, Chromium, Ray) treat `fork` as an optimization. Windows gets the same contract via spawn or exec, and runs slower. Tools whose public contract *is* `fork` (pytest-forked, gunicorn, Celery prefork, Ansible) mostly dropped or never shipped Windows. Purba's plan (zygote on Unix, fresh process per test on Windows) is the first group's pattern. It is the same shape as `multiprocessing` (forkserver vs spawn) and Chromium (zygote vs exec).

2. **The common signal answer is "document the gap, don't emulate it."** nextest, uv, coverage.py, pytest-timeout, asyncio and tokio all accept a weaker Windows signal story. Each one names the missing feature. None of them builds a fake SIGTERM on Windows. nextest kills via job objects; uv lets the child handle Ctrl-C.

3. **Platform code usually stays in the same crate, split by file.** nextest (`runner/unix.rs` + `runner/windows.rs`), rustup, tokio (`signal::unix` / `signal::windows`) and process-wrap all keep both OSes in one crate behind `cfg`. The one "split into its own package" case I found (xdist → pytest-forked) split off the *Unix-only* feature, and that package was later orphaned. The sibling agent on Cargo mechanics owns the crate question; this survey only says the in-crate split is the common choice.

4. **Dropping is cheap now and costly later.** With zero users, a drop costs nothing today. The cost shows up if Windows is wanted later. No project in this survey that started Unix-only added native Windows without a funded team (Bun) or a beta label (Ray). uvloop, gunicorn and Pants have open Windows issues of 6–13 years.

5. **Tier policies scale the promise to the people.** A single maintainer meets only the bottom tier of Rust and CPython. A tier policy lets purba make an honest middle promise: a "Tier 2" Windows that is built and maybe smoke-tested, but whose test failures do not block a merge. Rust and uv both say such a tier "may or may not pass tests". Rust says it will "accumulate bugs faster".

6. **Fit for zero users and one maintainer:**

| Strategy | Fit | Why (inference) |
|---|---|---|
| Full parity, Windows tested on every PR | Poor | Rust asks 3 named maintainers for this promise, CPython asks all core devs, Go asks the core team. |
| Build on Windows, test on Linux (a written "Tier 2") | Good | Keeps the spawn path compiling. A failing Windows test does not block. Promise is written down, like Rust and uv. |
| Windows fallback path + documented signal gaps | Good, if the fallback stays | This is how purba's own plan already looks. It matches nextest, multiprocessing and Chromium. |
| Unsupported, "may work, patches welcome" | Acceptable | Celery's wording. The cost is issue traffic, and purba has no users yet. |
| WSL only | Acceptable | Ansible, pyenv, Pants. Zero code cost. Windows-native users are lost, and a later port starts from Unix assumptions. |
| Drop, then restore later | Risky | No restoration found in this survey. |

7. **What triggered drops was always people, never a technical wall.** Celery: "unable to diagnose". Rust: "no dedicated target maintainers". uvloop: maintainer time. Pex: "no active Windows contributors" ([B] Pants chat). The technical gaps (no fork, few signals) were solved by every project that had a Windows person.

### Sources I could not open or did not check

- pytest-subprocess: not checked.
- Node.js 32-bit Windows drop: only secondary posts read.
- Go 1.26 `windows/arm` removal: only a source mirror commit seen in search results.
- Ansible's "fork in the worker model" reason: only a search summary of the mailing list.
- winloop: only search results and a PyPI summary.
