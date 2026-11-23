# A test of the wheel makes each fork fail through a library it preloads

## Context and Problem Statement

purba prints a line at each place where it meets a refused fork.
No test of the wheel reached any of those places.

| fact | source |
|---|---|
| purba meets a refused fork at four places: main forks the zygote, the zygote forks a test's process or a replay, and a test's process forks to read an expectation alone | [`zygote.rs` L590](https://github.com/kalonji-tools/purba/blob/50a85ffe/crates/execution/src/zygote.rs#L590), [L1131-L1136](https://github.com/kalonji-tools/purba/blob/50a85ffe/crates/execution/src/zygote.rs#L1131-L1136), [`load.rs` L663-L667](https://github.com/kalonji-tools/purba/blob/50a85ffe/crates/assertion/src/load.rs#L663-L667) |
| no file under `tests/` held "cannot fork" | [the probe](https://github.com/kalonji-tools/purba/pull/1210#issuecomment-7109790442) |
| a library that refused `fork` once a file existed, preloaded into purba, reached all four places on Linux | [the resolution](https://github.com/kalonji-tools/purba/issues/1154#issuecomment-7108457009) |
| on macOS, `DYLD_INSERT_LIBRARIES` loaded the library, `__DATA,__interpose` replaced `fork`, and each case passed on 3.12, 3.13, 3.14 and 3.14t | [job 137850506981](https://github.com/kalonji-tools/purba/actions/runs/45219710905/job/137850506981), [job 137861225476](https://github.com/kalonji-tools/purba/actions/runs/45219710905/job/137861225476) |
| each of five mutants turned at least one case red, and no case that does not reach its line | [the probe](https://github.com/kalonji-tools/purba/pull/1210#issuecomment-7109790442) |
| a replay's refusal sent as a project error printed the same line, so only the case with a raise turned red | [the probe](https://github.com/kalonji-tools/purba/pull/1210#issuecomment-7109790442) |
| in the order of a case, a raise comes before a refused fork, and a refused fork comes before a failure and a pass | [`order.rs` L227-L244 at `50a85ffe`](https://github.com/kalonji-tools/purba/blob/50a85ffe/crates/execution/src/order.rs#L227-L244) |
| on Linux, a refused replay printed its line when each expectation held, and when the first did not | [the critique](https://github.com/kalonji-tools/purba/pull/1210#issuecomment-7109295100) |
| with no `Ended(0)` after a refusal, purba did not end, and the test killed it after its time limit | [the probe](https://github.com/kalonji-tools/purba/pull/1210#issuecomment-7109790442) |
| `timeout` that carried the library could not fork once the file existed | [the resolution](https://github.com/kalonji-tools/purba/issues/1154#issuecomment-7108457009) |
| the compile of the library took 57 ms on Linux | [the resolution](https://github.com/kalonji-tools/purba/issues/1154#issuecomment-7108457009) |

## Considered Options

- **No test, and the fakes of `run_on` only.** The four places stay unchecked in the wheel that a tester installs.
- **A test of a refused replay only.** The other three places stay unchecked. The order of a raise and a refused replay also stays unchecked.
- **A test on Linux only.** [The record of platforms](purba-runs-on-linux-and-macos.md#confirmation) says that CI checks on macOS each test that a [zygote](../../CONTEXT.md#zygote) forks. The library works on macOS.
- **A switch in the `fault` wheel that refuses a fork.** That switch causes an error that purba never expects, and a refused fork is one that purba expects. The switch adds code to a crate. The `fault` wheel also differs from the wheel that a tester installs.
- **A test of an interrupt while a fork fails.** A refused fork and the end of the module's process reach [one check of an interrupt](https://github.com/kalonji-tools/purba/blob/50a85ffe/crates/execution/src/zygote.rs#L880). [A test of an interrupt already reaches that check](https://github.com/kalonji-tools/purba/issues/1154#issuecomment-7108457009).

## Decision Outcome

**Reach:** `tests/fork.bats` `tests/fork/**` `crates/execution/src/fork.rs` `crates/execution/src/zygote.rs` `crates/assertion/src/load.rs`

**Area:** errors, tooling

**A test of the wheel makes each fork of purba fail, through a library that the test preloads into purba.**

- **`tests/fork/refuse-fork.c` makes `fork` fail with `EAGAIN` once the file that `REFUSE_FORK_FILE` names exists.** Linux loads it with `LD_PRELOAD`. macOS loads it with `DYLD_INSERT_LIBRARIES`.
- **A case makes the file where it wants the next fork to fail.**

  | place | the file appears |
  |---|---|
  | purba's main process forks the zygote | before the run |
  | the zygote forks a test's process | in the module's body |
  | the zygote forks a [replay](../../CONTEXT.md#replay) | in the test, while a thread is alive |
  | a test's process forks to read an expectation alone | in the test, with no thread |

  In the cases of the replay and of the read alone, the first expectation does not hold, so the refused fork beats a failure.
  A second case of the replay adds a raise, and the raise decides the case.
- **`tests/fork.bats` compiles the library with `cc` once, before its first test.** purba already needs a C compiler on the host, as [purba carries no tool manager beside mise, and no compiler of its own](purba-carries-no-tool-manager-beside-mise-and-no-compiler-of-its-own.md#decision-outcome) says.
- **Only purba gets the library.** Python starts purba in a session of its own. It kills that session when purba ends, or when a time limit runs out.

**Downside:**

- **No case reaches a refused pipe of the zygote.** That pipe prints the line of a refused fork ([`zygote.rs` L590-L592](https://github.com/kalonji-tools/purba/blob/50a85ffe/crates/execution/src/zygote.rs#L590-L592)), and the library refuses `fork` alone.
- **The number of the error at the end of each line differs by platform.** The test reads it from Python's `errno`.
- **No gate compiles or formats `tests/fork/refuse-fork.c`.** Only the run of `tests/fork.bats` compiles it.

## Confirmation

`mise run test:wheel`, and the `wheel` job of `ci.yml` on each Linux and macOS cell, run `tests/fork.bats`.
It checks the line of each place, and the case of a raise before a refused replay.
Each case checks that stderr is empty.

A run with no file reaches all four places, and refuses no fork.
[`load.rs` L649-L654](https://github.com/kalonji-tools/purba/blob/50a85ffe/crates/assertion/src/load.rs#L649-L654) decides that reach.
A test with a thread alive at its first expectation replays each later one.
A test with no thread forks to read each later one.

[Should a test of the installed wheel make the zygote's fork fail?](https://github.com/kalonji-tools/purba/issues/1154) asks the question.
[A test causes an internal error through a switch that only a test build carries](a-test-causes-an-internal-error-through-a-switch-that-only-a-test-build-carries.md#decision-outcome) causes the error that purba never expects.
