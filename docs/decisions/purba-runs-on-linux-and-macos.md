# purba runs on Linux and macOS, and a Windows user runs it under WSL

## Context and Problem Statement

Build and publish ran each job on Linux, macOS and Windows, as [Prove the wheel builds, installs and imports everywhere purba claims](https://github.com/kalonji-tools/purba/issues/42) chose.
No record decided which platforms purba supports.
Windows has no `fork` and fewer signals than Linux and macOS.
purba's lockfile holds no `bats` for Windows.
So each process and signal decision got a second answer for Windows.
No end-to-end test ran that answer.

| fact | source |
|---|---|
| the Windows-only Rust in `crates/` was about 52 lines | [the research, Part 1](https://github.com/kalonji-tools/purba/blob/99fce9cc8d2798e65f81345db3188ef0069905f8/research/windows-support.md?plain=1#L188) |
| 7 of 98 records stated a Windows-only behaviour, a Windows exception or a Windows downside | [the same](https://github.com/kalonji-tools/purba/blob/99fce9cc8d2798e65f81345db3188ef0069905f8/research/windows-support.md?plain=1#L189) |
| none of the 53 end-to-end cases ran on Windows | [the same](https://github.com/kalonji-tools/purba/blob/99fce9cc8d2798e65f81345db3188ef0069905f8/research/windows-support.md?plain=1#L204) |
| one change to the interrupt code took the `cfg` lines that split by platform from 20 to 35 | [the research](https://github.com/kalonji-tools/purba/blob/99fce9cc8d2798e65f81345db3188ef0069905f8/research/windows-support.md?plain=1#L21) |
| a zygote needs `fork`, so Windows needed a spawn for each test, and an outcome could differ by platform | [Each test runs in a process of its own](https://github.com/kalonji-tools/purba/blob/291f6ec35836dcba385c04b91c2cfa8760f0a2b8/docs/decisions/each-test-runs-in-a-process-of-its-own.md?plain=1#L34-L42) |
| no issue asked for Windows, and purba had no release | [the research, Part 1](https://github.com/kalonji-tools/purba/blob/99fce9cc8d2798e65f81345db3188ef0069905f8/research/windows-support.md?plain=1#L196) |
| `cargo clippy --target x86_64-pc-windows-gnu` checked the Windows code in every layout, so a Windows crate that each build picks by target added no check | [the research, Part 2](https://github.com/kalonji-tools/purba/blob/99fce9cc8d2798e65f81345db3188ef0069905f8/research/windows-support.md?plain=1#L465) |
| gunicorn, uvloop and pytest-forked, which each rest on `fork`, never shipped native Windows | [the research, Part 3](https://github.com/kalonji-tools/purba/blob/99fce9cc8d2798e65f81345db3188ef0069905f8/research/windows-support.md#survey) |
| no surveyed project dropped Windows and restored it later | [the same](https://github.com/kalonji-tools/purba/blob/99fce9cc8d2798e65f81345db3188ef0069905f8/research/windows-support.md#drops-and-what-followed) |
| WSL 2 uses "an actual Linux kernel inside a managed VM" with "full system call compatibility" | [Microsoft's WSL documentation](https://learn.microsoft.com/en-us/windows/wsl/compare-versions) |

## Considered Options

- **Keep native Windows as it is.** Each process and signal decision keeps a second answer for Windows that no end-to-end test runs. The zygote needs a second design.
- **Move the Windows code into a crate of its own, which each build picks by target.** The move keeps the Windows answer of each decision. A lint of the Windows target already checks all that the crate checks.

## Decision Outcome

**Reach:** `crates/**` `.github/workflows/build.yml` `.github/workflows/publish.yml` `.config/tasks.toml` `.config/mise.toml` `/README.md` `/CONTRIBUTING.md` `/CONTEXT.md`

**Area:** design, tooling

**purba builds, tests and publishes for Linux and macOS alone, and a Windows user runs purba under WSL2.**
No process or signal decision then needs a second answer for Windows.
WSL2 runs a Linux kernel, so purba runs there as it does on Linux.

- **A build for Windows stops at once, and its message names WSL2.**
- **No record and no code carries a Windows exception.**
- **An issue that asks for native Windows, filed by somebody who does not maintain purba, opens the question again.** The person who decides judges whether the demand is enough. A new record then decides it, and names who maintains the Windows arm.

**Downside:**

- **A tester who cannot run WSL2 cannot run purba on Windows.**
- **A later Windows port starts from a design that assumes Unix.** Each process and signal decision then needs its Windows answer at once.
- **A contributor on Windows develops purba inside WSL2.**

## Confirmation

The matrices of `build.yml` and `publish.yml` name no Windows runner. No command checks that.
[Remove the Windows arms from purba's code](https://github.com/kalonji-tools/purba/issues/841) makes a build for Windows stop with a message that names WSL2.
It also removes each Windows arm of the code.
