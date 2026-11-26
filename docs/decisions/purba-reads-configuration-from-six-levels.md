# purba reads configuration from six levels, and the higher level overwrites the lower

## Context and Problem Statement

[Configuration](../../LANDSCAPE.md#the-cross-cutting-concerns) is a concern, and it was first in [the order of design](https://github.com/kalonji-tools/purba/blob/9e221a74/LANDSCAPE.md#L177).
[The landscape](https://github.com/kalonji-tools/purba/blob/9e221a74/LANDSCAPE.md#L156) said only that a value states a fact about the project or tightens a check.
Only the cap on suppressed verdicts had a rule for where its value comes from and which value wins, in [a suppression is an outcome and not a switch](https://github.com/kalonji-tools/purba/blob/9e221a74/docs/decisions/a-suppression-is-an-outcome-and-not-a-switch.md#L61).
No record said where the code that reads a value lives.
The errors concern has a crate of its own, which uses no part, as [an outcome and an error live in a crate that links no Python](an-outcome-and-an-error-live-in-a-crate-that-links-no-python.md#decision-outcome) decides.

| fact | source |
|---|---|
| each of six test runners offers an option that loosens a check: 38 of 715 options | [`research/configuration-dials.md` L9-L13](https://github.com/kalonji-tools/purba/blob/5a10089323b43bb044888df086b293fdbc12949b/research/configuration-dials.md?plain=1#L9-L13) |
| in five runners, a value from outside the project changed a test's outcome | [`research/configuration-dials.md` L15](https://github.com/kalonji-tools/purba/blob/5a10089323b43bb044888df086b293fdbc12949b/research/configuration-dials.md?plain=1#L15) |
| for one plain value, each of 11 tools lets the higher level overwrite the lower: the command line, the environment, the project, the user, the system | [Which levels of configuration do test runners and Python tools read, and in which order does each tool resolve them?](https://github.com/kalonji-tools/purba/issues/1212#issuecomment-7112327125) |
| a value in code beats the command line in pytest-timeout, Jest, Vitest, mypy and ruff | [Which levels of configuration do test runners and Python tools read, and in which order does each tool resolve them?](https://github.com/kalonji-tools/purba/issues/1212#issuecomment-7112327125) |
| no tool of the 11 takes the stricter of two levels, for each value probed and each order that the documentation states | [Which levels of configuration do test runners and Python tools read, and in which order does each tool resolve them?](https://github.com/kalonji-tools/purba/issues/1212#issuecomment-7112327125) |
| the user's file or a variable of the environment made a local run differ from CI in 18 confirmed reports, in 7 of 10 trackers | [Which levels of configuration do test runners and Python tools read, and in which order does each tool resolve them?](https://github.com/kalonji-tools/purba/issues/1212#issuecomment-7112327125) |
| mypy stops its walk to the project's file at the directory that holds `.git`, and pytest does not stop | [`research/configuration-levels.md` L318](https://github.com/kalonji-tools/purba/blob/5cdfe06d4258fdcc4bd27fa9de8c8e1da481c4b8/research/configuration-levels.md?plain=1#L318) |
| cargo, Go, git, Jujutsu, pre-commit, Jest and uv's project commands refuse a run that finds no root; Go's refusal names the `.git/config` that it found | [Which ways do tools use to find the root of a project?](https://github.com/kalonji-tools/purba/issues/1238#issuecomment-7159176416) |
| a fallback to the working folder moves a test's id, and a step that reads the tree keeps one id; no surveyed command-line tool has a toggle for its fallback | [Which ways do tools use to find the root of a project?](https://github.com/kalonji-tools/purba/issues/1238#issuecomment-7159176416) |
| the survey read only the paths on Linux, and pip, uv, mypy and git name other paths on macOS | [`research/configuration-levels.md` L286](https://github.com/kalonji-tools/purba/blob/5cdfe06d4258fdcc4bd27fa9de8c8e1da481c4b8/research/configuration-levels.md?plain=1#L286) |
| on a wheel, the user's file that set a timeout of 0 turned the timeout off when the stricter value won | [How does each rule that resolves the levels of configuration look to the tester and to the integrator?](https://github.com/kalonji-tools/purba/issues/1213#issuecomment-7112950928) |
| on a wheel, the user's file that set a timeout of 1 s ended a test on one machine and not in CI, under each rule | [How does each rule that resolves the levels of configuration look to the tester and to the integrator?](https://github.com/kalonji-tools/purba/issues/1213#issuecomment-7112950928) |
| ruff and uv resolve their configuration in a crate beside their binary, and that crate depends on the crates of its types | [How do Rust tools shape the code that reads and resolves configuration from several levels, and which shape fits purba?](https://github.com/kalonji-tools/purba/issues/1217#issuecomment-7114096339) |
| no tool or crate of the eight names the nearest option of an unknown key, and no crate closes a level to a value | [How do Rust tools shape the code that reads and resolves configuration from several levels, and which shape fits purba?](https://github.com/kalonji-tools/purba/issues/1217#issuecomment-7114096339) |
| `lint:wiring` lets the crate of a concern use no `purba-*` crate | [How do Rust tools shape the code that reads and resolves configuration from several levels, and which shape fits purba?](https://github.com/kalonji-tools/purba/issues/1217#issuecomment-7114096339) |
| each of `config`, `figment`, `figment2` and `tier` replaces the order and the level of each value; `tier` also names the nearest dial, unless one struct flattens the structs of two parts | [`research/configuration-crates.md` L13-L15](https://github.com/kalonji-tools/purba/blob/fd93d16b93db22bd4a5e3799483377c51d2cceca/research/configuration-crates.md?plain=1#L13-L15) |
| no crate of those four closes a level, applies the cap's lower value, or names every error | [`research/configuration-crates.md` L13](https://github.com/kalonji-tools/purba/blob/fd93d16b93db22bd4a5e3799483377c51d2cceca/research/configuration-crates.md?plain=1#L13) |

## Considered Options

- **The stricter value wins at every level.** The user's file turned the timeout off. No surveyed tool takes the stricter value.
- **Each level may set each dial.** The user's file made the outcome on one machine differ from CI.
- **The command line beats code.** A test cannot ask for more time than the run grants.
- **A dial may loosen a check, if the report shows it.** It reopens [purba is always strict](purba-is-always-strict.md#decision-outcome).
- **A key that no dial owns gives a warning.** A misspelled key leaves its dial unset, and the run goes on.
- **purba reads `purba.toml`, or reads `purba.toml` and `pyproject.toml`.** A new file crowds the root of the project. Two files are two shapes to read.
- **The walk to the project's file does not stop.** A file above the repository reaches the run.
- **Command resolves each value, and hands each part its typed value.** Command then holds a concern. Execution held the names of errors in the same way, until [an outcome and an error live in a crate that links no Python](an-outcome-and-an-error-live-in-a-crate-that-links-no-python.md#decision-outcome) moved them out.
- **The crate of the concern declares every dial.** The meaning of each dial leaves the part that reads it.
- **Each part reads its own values.** Each part repeats the order, the closed levels and the search for the nearest dial.

## Decision Outcome

**Reach:** `crates/**` `/LANDSCAPE.md`

**Area:** configuration

**purba takes each [dial](../../CONTEXT.md#dial) from the highest [configuration level](../../CONTEXT.md#configuration-level) that may set it.**
For one plain value, each surveyed tool overwrites in the same way.

| order | level | where | may set |
|:-:|---|---|---|
| 1 | code | beside a test, in a form that a blind rating picks | each dial that its part opens to code |
| 2 | the command line | a flag | each dial that the project's file does not keep |
| 3 | the environment | a `PURBA_*` variable | each dial that the command line may set |
| 4 | the project | `pyproject.toml`, under `[tool.purba]` | each dial |
| 5 | the user | `$XDG_CONFIG_HOME/purba/purba.toml`, or `~/.config/purba/purba.toml` | each dial that changes no outcome |
| 6 | the system | `/etc/purba/purba.toml` | each dial that changes no outcome |

- **A value states a fact about the project, tightens a check, or changes behaviour that no check reads.** No value loosens a check. A key that would loosen a check is an error.
- **A timeout states how long a test may run, which is a fact about the project.** So a longer timeout in code loosens no check.
- **The cap on suppressed verdicts keeps its own rule: the lower value applies.** [A suppression is an outcome and not a switch](a-suppression-is-an-outcome-and-not-a-switch.md#decision-outcome) holds it.
- **The user's file and the system's file set only a dial that changes no outcome.** A dial in such a file that changes an outcome is an error. So a file on one machine changes no outcome.
- **Code beats the command line.** A test that needs more time says so beside itself.
- **purba reads no file of the project other than `pyproject.toml`.**
- **The project's file keeps `test-files`, `[tool.purba.platform]` and `[tool.purba.queries]`, the three dials that change which tests run.** [purba picks a test by its name, and the tests of a run by one query](purba-picks-a-test-by-its-name-and-the-tests-of-a-run-by-one-query.md#decision-outcome) says why.
- **purba walks up from the working folder to the first `pyproject.toml` that holds `[tool.purba]`.** The folder of that file is the [root](../../CONTEXT.md#root). A path of the query starts at the root, so the walk cannot start from a path. The walk stops at the folder that holds `.git`, whether `.git` is a folder or, as in a worktree of git, a file.
- **When the walk finds no such file, and `PURBA_IMPLICIT_ROOT` is off, purba refuses the run as an invocation error.** purba reads the variable as off when it is unset, empty or `0`. The error names the `pyproject.toml` with `[project]` that the walk found, as the place for `[tool.purba]`. Without one, it names the folder that holds `.git`, or else the working folder.
- **When `PURBA_IMPLICIT_ROOT` is `1`, purba tries three more steps in order: the `pyproject.toml` with `[project]`, the folder that holds `.git`, and the working folder.** Each run that takes one of these steps prints a [notice](../../CONTEXT.md#notice) above its first line. The notice names the root, the step and the variable. It changes no exit code.
- **Any other value of `PURBA_IMPLICIT_ROOT` is an invocation error, in each run.** A run whose walk finds `[tool.purba]` refuses that value too.
- **A key that no dial owns is an error.** The error names the dial whose name is nearest to the key.
- **purba refuses a `PURBA_*` variable whose name is one or two edits from the variable of a dial, and ignores every other one.** So purba refuses a misspelled variable. It ignores a variable that a test of purba sets, such as `PURBA_SENTINEL`.
- **The kind of an error of configuration says where the tester fixes it.** A flag or a `PURBA_*` variable gives an [invocation error](../../CONTEXT.md#invocation-error). A key in a file gives a [project error](../../CONTEXT.md#project-error), because purba finds the file itself. [A failure's kind is what the part that first sees it observes](a-failures-kind-is-what-the-part-that-first-sees-it-observes.md#decision-outcome) holds that rule.
- **Before purba stops on configuration, it names every error that it finds.** [An error stops only the work that depends on it](an-error-stops-only-the-work-that-depends-on-it.md#decision-outcome) holds that rule.
- **Each value keeps the level that it came from.**
- **`purba-config`, the crate of the concern, holds the mechanism.** It holds the walk to the project's file, the project's level and the level of the environment. It also holds the search for the nearest dial, and the level of each value. It uses no part, as the crate of errors uses none.
- **Each part declares its own dials to `purba-config`, as typed handles.** The part that reads a value owns what it means, as [a concern is what no part can own](a-concern-is-what-no-part-can-own.md#decision-outcome) says. Command gathers the declarations and resolves them. It turns each refusal into an error of the kind above.
- **Each level other than the project's comes with the first dial that it may set.** The order of the levels comes with the first dial that two levels may set. The cap's rule comes with the cap.

Each dial waits for the design of its part.

| what | the design that decides it |
|---|---|
| a dial's name, type and meaning | the part that declares it: the cap, `[tool.purba.platform]` and `[tool.purba.queries]` in selection, the timeout and the workers in dispatch, the form of the output in reporting, the import roots in execution |
| the form of a value in code | dispatch, with the timeout, through a blind rating |
| the line that names a value's level | reporting, through a blind rating |

**Downside:**

- **A variable of the environment can drift like the user's file.** A variable in the start-up file of a shell changes a run on one machine. The level of each value shows where it came from.
- **The cap breaks the one order.** A part that reads a value must know whether its dial is the cap.
- **A pull request that adds a dial edits its part and command.** Command gathers the declarations of each part.
- **purba writes the order, the closed levels and the search for the nearest dial itself.** A crate on the shortlist of `resolve-levels` can replace the order and the level of each value. `tier` can also replace the search for the nearest dial, unless one struct flattens the structs of two parts. No crate on the shortlist closes a level, applies the cap, or names every error.
- **purba ignores a `PURBA_*` variable whose name is far from each dial's variable.** A variable that is misspelled far from its dial goes unseen.
- **Outside git, purba walks up to the root of the file system.** A file above the project then reaches the run.
- **Under the last step that `PURBA_IMPLICIT_ROOT` turns on, the id of a test moves with the working folder.** The line of that step says so.
- **`PURBA_IMPLICIT_ROOT` drifts like any variable of the environment.** A shell that sets it makes a run on that machine differ from a run on another. The line of each step names the variable.
- **A wrong value of `PURBA_IMPLICIT_ROOT` stops a run that does not need it.** A run whose walk finds `[tool.purba]` refuses the value too.
- **An error of the environment stops the run before purba reads the project's file.** A run with a wrong variable and a wrong key names only the variable.
- **The paths on macOS are unmeasured.** pip, uv, mypy and git name other paths there.

## Confirmation

`mise run test:rust` checks what the walk to the project's file finds in `crates/config/tests/walk.rs`.
It checks the errors that command makes of what the walk finds in `crates/command/tests/root.rs`.
It checks there that each id starts at the root.
`mise run test:wheel` checks the walk and its errors in `tests/stage_one.bats`.

`mise run lint:order` refuses an order of design that still ranks configuration.
`mise run test:rust` checks `purba-config` in `crates/config/tests/project.rs`.
It checks the first dial, `test-files`, in `crates/selection/tests/test_files_dial.rs`.
`mise run test:wheel` checks the errors of the project's file in `tests/test_files.bats`.

`mise run test:rust` checks the level of the environment in `crates/config/tests/environment.rs`.
It checks the first two steps of `PURBA_IMPLICIT_ROOT` and the errors of the environment in `crates/command/tests/root.rs`.
`mise run test:wheel` checks the three steps and the errors of the environment in `tests/implicit_root.bats`.
The line of configuration in `LANDSCAPE.md` names the crate, so that `mise run lint:wiring` reads it.

[How does purba read and resolve its configuration?](https://github.com/kalonji-tools/purba/issues/1219) asks the question.
[purba finds an implicit root by a ladder when PURBA_IMPLICIT_ROOT is set, and says so](https://github.com/kalonji-tools/purba/issues/1241) built the steps that `PURBA_IMPLICIT_ROOT` turns on.
[`.config/shortlists.toml`](../../.config/shortlists.toml) holds the shortlist of `resolve-levels`: the crates that can replace part of the mechanism.
