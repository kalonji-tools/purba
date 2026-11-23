# The levels of configuration that test runners and Python tools read

Research for [Which levels of configuration do test runners and Python tools read, and in which order does each tool resolve them?](https://github.com/kalonji-tools/purba/issues/1212)
Measured on 2026-11-23, against purba `main` at `27e8f559`, on one Linux machine.
This file decides nothing. It lists the levels each tool reads, how it finds each file, and which level wins.

## Summary

I surveyed 11 tools: the test runners pytest, cargo-nextest, Vitest, Jest and `go test`, and the tools ruff, uv, mypy, pip, cargo and git. I ran a probe on each of them. Every tool reads the command line and environment variables. Ten read a file of the project; pip reads none. Eight read a file of the user, and four read a file of the system. Seven read a setting from code: a marker, a call, or a comment in the source.

For one plain value, every tool resolves the levels in the same order. The command line wins over an environment variable, and an environment variable wins over a file. Among the files, a file of the project wins over a file of the user, and a file of the user wins over a file of the system. So the rule `higher-overwrites` describes all 11 tools for a plain value.

That rule does not describe every value. Four other rules appear:

- **Code wins over the command line** for a value that a test sets for itself. In the scenario of #1212 (project 10 s, code 60 s, `--timeout 5`), pytest with pytest-timeout, Jest and Vitest all give the test 60 s. I ran this. `go test` and cargo-nextest have no level in code for a timeout, so the command line wins there. mypy and ruff also let a comment in code win over the command line. ruff alone has a flag, `--ignore-noqa`, that wins back over the comment.
- **A list is extended at each level** in pytest, uv, cargo, git and pip. In other tools, or for other lists, a higher level replaces the whole list. Examples are ruff's `select`, Go's `GOFLAGS`, and pip between two files.
- **A level can be closed to a value.** git refuses `safe.directory` from the repository's own file. The reason is that an untrusted repository could set it. cargo-nextest ignores, with a warning, any setting in a file of the user that changes how tests run. uv refuses 12 project fields in a `uv.toml`.
- **The environment changes a default that decides an outcome.** With `CI=true`, Vitest fails a test marked `test.only`, and both Jest and Vitest fail a test whose snapshot is new. Without `CI`, both tests pass. I ran this.

I found no tool that resolves two levels by **`stricter-wins`**. That holds for each value I probed and for each order of levels the documentation states. purba's cap on suppressed verdicts would be the only such rule in this sample.

A file of the user can change an outcome on one machine only. ruff and mypy drop the file of the user when they find a file of the project. uv, cargo, git, pip and Go keep it, below the file of the project. In 7 of the 10 issue trackers searched, a search found at least one confirmed report of a different outcome on a developer's machine and in CI. A file of the user or an environment variable caused each one. git has no issue tracker on GitHub.

One probe found a fault in ruff 0.17.6 (section 6). A file below the working directory can take its parent directory's configuration in place of its own. This happens when another argument points above the working directory.

## How to read this file

- **[run]** means I ran it in this session, on the version named. **[source]** means documentation or code says so, and I did not run it. Each source is pinned to a tag. The Method section gives each tag's commit.
- A **level** is one place a value comes from: the command line, an environment variable, code, a file of the project, a file of the user, or a file of the system. git calls a level a "scope".
- A value **wins** when the tool uses it and ignores the same value from another level.
- **First match** means the tool reads one file and stops. **Merge** means the tool reads several files and combines them.
- A test runner's outcomes are pass and fail. For ruff and mypy, the outcome is the list of diagnostics.
- Every probe ran with `HOME`, `XDG_CONFIG_HOME` and the tool's own variables pointed into a scratch directory. No probe wrote to the real `~`, `~/.config` or `/etc`.

## Method

### Versions and sources

| tool | version run | how | source tag | commit |
|---|---|---|---|---|
| pytest | 9.2.0, on CPython 3.13.13 | `uv venv`, `uv pip install` | [`9.2.0`](https://github.com/pytest-dev/pytest/tree/9.2.0) | `ae1d0ff5` |
| pytest-timeout | 2.4.0 | same venv | [`2.4.0`](https://github.com/pytest-dev/pytest-timeout/tree/2.4.0) | `ddabc934` |
| cargo-nextest | 0.9.149 | on `PATH` | [`cargo-nextest-0.9.149`](https://github.com/nextest-rs/nextest/tree/cargo-nextest-0.9.149) | `3fe4ef31` |
| Vitest | 5.1.1 | `npm install`, node 24.21.0 | [`v5.1.1`](https://github.com/vitest-dev/vitest/tree/v5.1.1) | `161e25f6` |
| Jest | 30.5.2 | `npm install`, node 24.21.0 | [`v30.5.2`](https://github.com/jestjs/jest/tree/v30.5.2) | `202dd8a1` |
| `go test` | go1.26.10 | `nix-shell -p go` | [`go1.26.10`](https://github.com/golang/go/tree/go1.26.10) | `4e8c85f5` |
| ruff | 0.17.6 | same venv | [`0.17.6`](https://github.com/astral-sh/ruff/tree/0.17.6) | `a7ba68b4919e56b6af949a42b2da38fe4eec67a9` |
| uv | 0.13.7 | on `PATH` | [`0.13.7`](https://github.com/astral-sh/uv/tree/0.13.7) | `316c827e` |
| mypy | 2.4.1 | same venv | [`v2.4.1`](https://github.com/python/mypy/tree/v2.4.1) | `ad02a35e` |
| pip | 26.3 | same venv | [`26.3`](https://github.com/pypa/pip/tree/26.3) | `191854fb` |
| cargo | 1.99.0 | rustup stable | [`0.100.0`](https://github.com/rust-lang/cargo/tree/0.100.0) | `5f94df47` |
| git | 2.54.0 | on `PATH` | [`v2.54.0`](https://github.com/git/git/tree/v2.54.0) | `94f05775` |

For pytest, pytest-timeout, mypy and pip, I compared each cited file in the venv with the same file at its tag, with `cmp`. Each was the same, so a line number in the venv is a line number at the tag.

### Probes

Each probe is a shell script in [`configuration-levels/`](configuration-levels/). It takes an empty work directory, builds the case there, and prints each command with its output. The appendix of each tool quotes the output. The scripts are `pytest.sh`, `nextest.sh`, `vitest.sh`, `jest.sh`, `go.sh`, `ruff.sh`, `uv.sh`, `mypy.sh`, `pip.sh`, `cargo.sh` and `git.sh`.

A file of the system is faked where the tool lets a variable move it:

- uv and pip: `XDG_CONFIG_DIRS`.
- git: `GIT_CONFIG_SYSTEM`.
- Go: `$GOROOT/go.env` sits in the read-only Nix store, so the probe reads it and does not change it.

Faking a level moves the file and keeps the rule that reads it. The real paths `/etc/uv/uv.toml`, `/etc/pip.conf` and `/etc/gitconfig` are [source] only.

### Reports from issue trackers

A sub-agent ran the search of the issue trackers in this session, under my brief. It ran 71 queries over the 10 trackers through the GitHub search API. It read the body and first comments of 61 candidates. It kept a candidate only when both of these were true: (a) the outcome differed between machines, or between a machine and CI, and (b) the cause was a file of the user, a file of the system, or an environment variable. I re-read 4 of its confirmed reports myself: uv #19196, ruff #2304, cargo #15130 and Go #37390. Each count answers only its own query. Appendix B gives the queries.

## 1. The levels each tool reads

| tool | command line | environment | code | project file | user file | system file |
|---|---|---|---|---|---|---|
| pytest | ✅ | ✅ `PYTEST_ADDOPTS`, and a plugin's own, such as `PYTEST_TIMEOUT` | ✅ marks, `pytestmark`, `conftest.py` | ✅ | — | — |
| cargo-nextest | ✅ | ✅ `NEXTEST_*` | — | ✅ `.config/nextest.toml` | ✅ display only | — |
| Vitest | ✅ | ✅ `CI`, which changes defaults | ✅ a test's options, `vi.setConfig` | ✅ | — | — |
| Jest | ✅ | ✅ `CI`, which changes defaults | ✅ `jest.setTimeout`, a test's timeout | ✅ | — | — |
| `go test` | ✅ | ✅ `GOFLAGS`, `GODEBUG`, every `GO*` | ✅ `//go:debug`, `t.Parallel()` | ✅ `go.mod`, with no test flags | ✅ go env file | ✅ `$GOROOT/go.env` |
| ruff | ✅ | ✅ 4 `RUFF_*`, for output and cache only | ✅ `# noqa`, `# ruff: noqa` | ✅ | ✅ only with no project file | — |
| uv | ✅ | ✅ `UV_*` | ✅ `[tool.uv]` in a script's metadata [source] | ✅ | ✅ | ✅ |
| mypy | ✅ | ✅ 6 `MYPY*`, for paths, cache, display and workers only | ✅ `# mypy:`, `# type: ignore` | ✅ | ✅ only with no project file | — |
| pip | ✅ | ✅ `PIP_*`, `PIP_CONFIG_FILE` | — | — (the venv's `pip.conf` instead) | ✅ | ✅ |
| cargo | ✅ `--config`, flags | ✅ `CARGO_*` | — | ✅ every `.cargo/config.toml` up to `/` | ✅ `$CARGO_HOME/config.toml` | — |
| git | ✅ `-c` | ✅ `GIT_CONFIG_COUNT`, `GIT_*` | — | ✅ `.git/config`, `config.worktree` | ✅ two global files | ✅ |

Counts over these 11 rows:

| level | tools that read it |
|---|---|
| command line | 11 |
| environment | 11 |
| project file | 10 (not pip) |
| user file | 8: nextest, `go test`, ruff, uv, mypy, pip, cargo, git |
| code | 7: pytest, Vitest, Jest, `go test`, ruff, uv, mypy |
| system file | 4: `go test`, uv, pip, git |

No test runner but `go test` reads a file of the user that changes how tests run. cargo-nextest reads one, but [ignores any key outside display, with a warning](#a2-cargo-nextest-09149) [run].

## 2. How each tool finds its files

Four patterns appear.

| pattern | tools | where the walk starts | a file with no section for the tool |
|---|---|---|---|
| **Walk up, first match wins** | pytest, ruff, uv, mypy, Jest | pytest: the common ancestor of the arguments. ruff: each file's own directory. uv: the project's root, or else the cwd. mypy: the cwd. Jest: the cwd. | pytest, ruff, uv and mypy skip it and walk on. **Jest stops at a `package.json` with no `"jest"` key.** |
| **Walk up and merge every file** | cargo | the cwd, not the manifest | — |
| **One fixed place** | cargo-nextest, Vitest, git | nextest: the workspace root. Vitest: the root (cwd or `--root`), with no walk. git: the repository found from the cwd. | — |
| **No file of the project** | pip | the venv's `pip.conf` takes its place | — |

All of these are [run]. The rows of the appendices give each command.

Some details change the outcome:

- **mypy stops its walk at a `.git` or `.hg` directory** [run]. A file above the repository's root is not read. Then the file of the user applies.
- **pytest has no file of the user. But a `pytest.ini` in `$HOME` is found from any directory below `$HOME`** [run]. The walk treats `$HOME` as one more parent directory.
- **ruff picks a file for each source file.** The other walk-up tools pick one file for the whole run.
- **uv starts at the project root**, not at the cwd. A `uv.toml` in a subdirectory of a project is not read [run]. Its documentation says that uv "will begin its search at the workspace root, ignoring any configuration defined in workspace members" ([`configuration-files.md` L14-L16](https://github.com/astral-sh/uv/blob/0.13.7/docs/concepts/configuration-files.md#L14-L16)).
- **cargo reads from the cwd, not from the manifest.** `cargo build --manifest-path p/Cargo.toml`, run elsewhere, ignores `p/.cargo/config.toml` [run].
- **Jest needs a file.** With no `package.json` and no Jest file anywhere up to `/`, Jest stops with "Could not find a config file" [run].
- **In one directory, a dedicated file wins over a shared one.** These are all [run]:
  - pytest: `pytest.toml` beats `pytest.ini`, which beats `pyproject.toml`, `tox.ini` and `setup.cfg`.
  - ruff: `.ruff.toml` beats `ruff.toml`, which beats `pyproject.toml`.
  - uv: `uv.toml` beats `pyproject.toml`, with a warning.
  - Vitest: `vitest.config.*` beats `vite.config.*`.
  - Jest is the exception: two files in one directory are refused.

## 3. The order of the levels

For one plain value, the order is the same in every tool. Each row lists the levels from the one that wins down to the lowest.

| tool | order, the winner first | example probed |
|---|---|---|
| pytest | command line > `PYTEST_ADDOPTS` > `addopts` in the file > the file's value | `--maxfail` [run] |
| pytest-timeout | marker > `--timeout` > `PYTEST_TIMEOUT` > `timeout` in the file | `timeout` [run] |
| cargo-nextest | command line > `NEXTEST_*` > a per-test override > the selected profile > an inherited profile > default | `retries` [run] |
| Vitest | code > command line > file | `testTimeout` [run] |
| Jest | code > command line > file | `testTimeout` [run] |
| `go test` | command line > `GOFLAGS` in the environment > `GOFLAGS` in the go env file > `$GOROOT/go.env` | `-timeout` [run]; `go.env` [source] |
| `go test`, `GODEBUG` | `GODEBUG` > `//go:debug` in code > `godebug` in `go.mod` > the `go` version | `panicnil` [run] |
| ruff | code (`noqa`) > command line > nearest project file, *or else* user file | `select` [run] |
| uv | command line > `UV_*` > project > user > system | `resolution` [run] |
| mypy | inline comment > per-module section > command line > `[mypy]` section, *from one file*: project, or else user | `disallow_untyped_defs` [run] |
| pip | command line > `PIP_*` > `PIP_CONFIG_FILE` > site (venv) > user (XDG) > user (legacy) > system; in a file, `[<command>]` > `[global]` | `index-url` [run] |
| cargo | `--config` > `CARGO_*` > the deepest `.cargo/config.toml` > … > `/.cargo/config.toml` > `$CARGO_HOME/config.toml` | `build.jobs` [run] |
| git | `-c` > `GIT_CONFIG_COUNT` > worktree > local > global (`~/.gitconfig` > XDG) > system | `probe.key` [run] |

Two exceptions to "the environment is below the command line" are worth naming:

- **git: `GIT_EDITOR` wins over `-c core.editor=…`** [run]. The variable and the key are two names of one value, and the variable wins. I probed only this one variable.
- **cargo: `[env]` with `force = true` makes a file win over the process environment**, for the variables cargo passes to rustc, build scripts and `cargo run` ([config.md L727-L748](https://github.com/rust-lang/cargo/blob/0.100.0/doc/book/src/reference/config.md#L727-L748)) [source]. purba's own `.cargo/config.toml` uses this for clippy ([record](https://github.com/kalonji-tools/purba/blob/27e8f559/docs/decisions/configuration-takes-the-first-location-its-tool-reads.md#decision-outcome)).

### The scenario of #1212

The project sets 10 s, the test is marked with 60 s in code, and the tester runs `--timeout 5`. I ran each case scaled: project 3 s or 1 s, code 6 s or 5 s, command line 1 s or 3 s, and the test sleeps 1.5 s to 2 s.

| tool | the timeout the test gets | why |
|---|---|---|
| pytest + pytest-timeout | **60 s, code** [run] | `_get_item_settings` reads the marker first, and falls back to the command line only with no marker ([`pytest_timeout.py` L388-L410](https://github.com/pytest-dev/pytest-timeout/blob/2.4.0/pytest_timeout.py#L388-L410)) |
| Jest | **60 s, code** [run] | `jest.setTimeout` and a test's own timeout both beat `--testTimeout` |
| Vitest | **60 s, code** [run] | `vi.setConfig` and a test's own timeout both beat `--testTimeout` |
| `go test` | **5 s, command line** [run] | no level in code or in `go.mod` sets a timeout. `-timeout` is one deadline for the whole test binary ([alldocs.go L3432-L3435](https://github.com/golang/go/blob/go1.26.10/src/cmd/go/alldocs.go#L3432-L3435)) |
| cargo-nextest | no level in code | a per-test override in the file beats the profile, and the command line and `NEXTEST_*` beat the override [run, for `retries`] |

So every runner that has a level in code lets code win over the command line for the timeout. None of them takes the lower value.

## 4. Values with a rule of their own

### 4.1 Lists that each level extends

| tool | value | rule | evidence |
|---|---|---|---|
| pytest | `-W`, `-p` and every flag that appends | the file's `addopts`, then `PYTEST_ADDOPTS`, then the command line, are joined into one argument list. A flag that appends collects from all three. | three `-W error::…`, one from each level, all applied: 3 failed [run] |
| pytest | `addopts` itself | `-o addopts=` on the command line clears the file's `addopts` | `--maxfail=1` in the file is gone: 3 failed [run] |
| uv | every array, such as `no-build-package` | the files are joined in this order: project, user, system. The environment and the command line are put **in front of** the files' array. | `fromenv, fromproject, fromuser, fromsystem` [run] |
| uv | the same array, set in the environment and on the command line | the command line replaces the environment, and still joins the files | `fromcli, fromproject, fromuser, fromsystem` [run] |
| cargo | every array, such as `build.rustflags` | joined from `$CARGO_HOME` to the deepest file; then `CARGO_*`, then `--config` | `home, top, mid, cli` [run] |
| cargo | the flags that rustc gets | **its own rule: the first source wins**: `CARGO_ENCODED_RUSTFLAGS`, then `RUSTFLAGS`, then `target.*.rustflags`, then `build.rustflags` ([config.md L580-L593](https://github.com/rust-lang/cargo/blob/0.100.0/doc/book/src/reference/config.md#L580-L593)) | [source] |
| git | a multi-valued key, such as `credential.helper` | every scope adds a value. An empty value resets the list ([gitcredentials.adoc L199](https://github.com/git/git/blob/v2.54.0/Documentation/gitcredentials.adoc#L199)) | `--get-all` lists system, global, `""`, local [run]; the reset is [source] |
| pip | an option that appends, such as `extra-index-url` | between files: **replace**, and the higher file wins. The command line **appends** to the files' list. `PIP_*` replaces the files' value. | `base, x-site, x-cli` and `base, x-env, x-cli` [run] |
| ruff | `select` | **replace**. The command line's `select` replaces each file's `select` and `ignore`. `--extend-select` adds to each file's list. | `--select E741`: E741 only; `--extend-select`: adds [run] |
| `go test` | `GOFLAGS` | **replace**. The variable in the environment replaces the one in the go env file as a whole. The command line then overrides flag by flag. | [run] |
| mypy | — | no setting is joined across levels | [source] |

### 4.2 Code against the command line

| tool | code-level setting | does it beat the command line? | can the command line win back? |
|---|---|---|---|
| pytest-timeout | `@pytest.mark.timeout` | yes [run] | no flag found |
| pytest | `@pytest.mark.filterwarnings` | yes. The order is mark > `-W` > the file's `filterwarnings` [run], as documented ([capture-warnings.rst L182-L183](https://github.com/pytest-dev/pytest/blob/9.2.0/doc/en/how-to/capture-warnings.rst#L182-L183)) | no |
| pytest | `@pytest.mark.xfail(strict=False)` | yes, over the file's `xfail_strict = true` [run] | `-o xfail_strict=…` sets only the file's value |
| pytest | a marker on the module and one on the test | the closest marker wins: `get_closest_marker` [run] | — |
| Jest | `jest.setTimeout`, a test's timeout | yes [run] | no flag found |
| Vitest | `vi.setConfig`, a test's options | yes [run] | no flag found |
| `go test` | `//go:debug` | no. `GODEBUG` wins over it [run] | yes, through `GODEBUG` |
| `go test` | `t.Parallel()` | the test opts in, and `-parallel N` caps how many run at once | both apply [source] |
| ruff | `# noqa`, `# ruff: noqa` | yes [run] | yes: `--ignore-noqa` [run] |
| mypy | `# mypy: allow-untyped-defs` | yes, over the per-module section and `--disallow-untyped-defs` [run]; "Inline configuration comments take precedence over all other configuration mechanisms" ([inline_config.rst L13-L14](https://github.com/python/mypy/blob/v2.4.1/docs/source/inline_config.rst#L13-L14)) | no |
| mypy | a `[mypy-m]` section of the file | **yes: a per-module section beats the command line** in both directions [run]. `--strict` on the command line lost to `disallow_untyped_defs = False` in `[mypy-m]` [run]. | no |

mypy's order is the clearest statement of the pattern. A value that names a narrower scope wins, whatever its level ([config_file.rst L74-L84](https://github.com/python/mypy/blob/v2.4.1/docs/source/config_file.rst#L74-L84)). ruff states the same for rule selectors: "ALL < category < linter group < linter prefix < rule" ([linter.md L215-L224](https://github.com/astral-sh/ruff/blob/0.17.6/docs/linter.md#L215-L224)). cargo-nextest's per-test override beats its profile for the same reason, but loses to the command line.

### 4.3 Stricter wins

I found no value whose resolution takes the stricter of two levels. I checked each value probed above. I also read the precedence section of each tool's documentation, cited in the appendices. Neither source states such a rule. This is a bound, not a proof: I did not read the rule of every value of every tool.

Three things come close:

- **A list that each level extends** (4.1). Where each entry adds a check, as with pytest's `-W error::…`, a higher level cannot remove a lower level's entry without a reset. So such a list behaves like stricter-wins.
- **A level closed to a value** (4.4). A lower or less trusted level cannot loosen the value, because the tool ignores it there.
- **An explicit tightening flag.** Examples are `--strict` in mypy and `--ci` in Jest. Each is one value that a higher level overwrites. Neither takes the minimum of two.

purba's own record already holds a stricter-wins rule: "Where the integrator sets both, the lower value applies" ([a suppression is an outcome](https://github.com/kalonji-tools/purba/blob/27e8f559/docs/decisions/a-suppression-is-an-outcome-and-not-a-switch.md#L61)). No surveyed tool resolves a value that way.

### 4.4 Values that a tool takes only at some levels, and why

| tool | value | levels that take it | reason the tool gives |
|---|---|---|---|
| git | `safe.directory`, `safe.bareRepository`, `uploadpack.packObjectsHook` | system, global, command. Not local, not worktree. | "This prevents untrusted repositories from tampering with this value" ([safe.adoc L34-L36](https://github.com/git/git/blob/v2.54.0/Documentation/config/safe.adoc#L34-L36)); protected configuration exists "For security reasons" ([git-config.adoc L455-L465](https://github.com/git/git/blob/v2.54.0/Documentation/git-config.adoc#L455-L465)). I counted 3 keys by grepping `Documentation/config/*.adoc` for "only respected … in protected configuration". [run]: with git's test switch `GIT_TEST_ASSUME_DIFFERENT_OWNER=1`, `safe.directory=*` in the local file still fails with "dubious ownership"; in the global file or through `-c`, git runs. |
| cargo-nextest | every setting that runs tests: `retries`, timeouts, test groups | project file, environment, command line. Not the user file. | User config "does not change test execution behavior" ([user-config/index.md L61-L63](https://github.com/nextest-rs/nextest/blob/cargo-nextest-0.9.149/site/src/docs/user-config/index.md#L61-L63)). [run]: `[profile.default] retries = 0` in the user file prints "ignoring unknown configuration key: profile", and the retry still runs. |
| uv | 12 fields: `build-backend`, `conflicts`, `default-groups`, `dependency-groups`, `dev-dependencies`, `environments`, `managed`, `minimum-libc-version`, `package`, `required-environments`, `sources`, `workspace` | only `[tool.uv]` in `pyproject.toml`. A `uv.toml` is refused, at any level. | "`managed` is only applicable in the context of a project" [run], from [`lib.rs` L704-L707](https://github.com/astral-sh/uv/blob/0.13.7/crates/uv-settings/src/lib.rs#L704-L707). The list is at [L328-L393](https://github.com/astral-sh/uv/blob/0.13.7/crates/uv-settings/src/lib.rs#L328-L393). A user or system file can only be a `uv.toml`, so these fields are project-only. |
| uv | every project file | `uv tool` and `uv self` read only the user and system files | "For commands that operate at the user-level, ignore local configuration" ([`lib.rs` L325-L329](https://github.com/astral-sh/uv/blob/0.13.7/crates/uv/src/lib.rs#L325-L329)) [source] |
| cargo | 12 of the 78 keys that have an "Environment:" line, for example `paths`, `include`, `source.<name>.*` | files and `--config` only; no `CARGO_*` | "not supported due to technical issues" ([config.md L222-L226](https://github.com/rust-lang/cargo/blob/0.100.0/doc/book/src/reference/config.md#L222-L226)). I counted the lines `* Environment: not supported`. [source] |
| mypy | the options marked "may only be set in the global section" | `[mypy]` and the command line; not a per-module section, nor an inline comment | not given. I counted 19 such notes in [config_file.rst](https://github.com/python/mypy/blob/v2.4.1/docs/source/config_file.rst#L111-L112); some notes cover a group of options. [source] |
| mypy, ruff | every check option | no environment variable sets a check. mypy's 6 `MYPY*` variables set paths, cache, display and workers. ruff's 4 `RUFF_*` variables set output and cache. | not given. Found by grepping mypy's source for `getenv`, and by reading `ruff check --help` [run]. |
| pytest | `--rootdir` | command line only, not `addopts` in a file | "the rootdir is used to *find* the configuration file" ([customize.rst L180-L183](https://github.com/pytest-dev/pytest/blob/9.2.0/doc/en/reference/customize.rst#L180-L183)) [source] |
| `go test` | `-timeout`, `-count`, `-parallel` | command line and `GOFLAGS` only; not `go.mod` | `go.mod` has no directive for them [source] |

### 4.5 The environment changes a default that decides an outcome

| tool | variable | what changes | evidence |
|---|---|---|---|
| Vitest | `CI` (read through `std-env`) | `allowOnly` defaults to `!isCI`, so a test marked `test.only` fails | without `CI`: 1 passed. With `CI=true`: "Unexpected .only modifier". With `CI=true --allowOnly`: passes [run]; [defaults.ts L93](https://github.com/vitest-dev/vitest/blob/v5.1.1/packages/vitest/src/defaults.ts#L93) |
| Vitest | `CI` | a new snapshot is not written, and the test fails | with `CI=true`: "Snapshot … mismatched", 1 failed. Without: "1 written", passed [run]; [resolveConfig.ts L679-L690](https://github.com/vitest-dev/vitest/blob/v5.1.1/packages/vitest/src/node/config/resolveConfig.ts#L679-L690) |
| Jest | `CI` (read through `ci-info`) | `ci` defaults to `isCI`, so a new snapshot is not written and the test fails | with `CI=true`: "New snapshot was not written", 1 failed. Without: written, passed. `--ci=false` wins over `CI=true` [run]; [Defaults.ts L23](https://github.com/jestjs/jest/blob/v30.5.2/packages/jest-config/src/Defaults.ts#L23) |
| pytest | `PYTEST_ADDOPTS` | any flag | 4.1 [run] |
| `go test` | `GOFLAGS`, `GODEBUG` | any flag; any GODEBUG setting | section 3 [run] |

## 5. A file of the user or of the system, and runs that differ from CI

The sub-agent read 61 candidates. Its rows below sum to 18 confirmed reports and 4 partial ones. Its own total said 17 and 5, and I did not resolve the gap, so the rows are the count to trust. git has no tracker on GitHub (`has_issues: false`), so it is not searched. Each count answers only its own queries, in Appendix B.

| tracker | read | confirmed | the confirmed reports |
|---|---|---|---|
| astral-sh/uv | 6 | 3 | [#19196](https://github.com/astral-sh/uv/issues/19196): `uv lock` on a laptop takes `exclude-newer-package` from `~/.config/uv/uv.toml`, so `uv sync --locked` fails in CI. [#18968](https://github.com/astral-sh/uv/issues/18968): `UV_CONFIG_FILE` in a container drops the project's `[tool.uv.audit]`. [#6349](https://github.com/astral-sh/uv/issues/6349): `UV_INDEX_URL` differs per machine, and the lock records it. |
| rust-lang/cargo | 7 | 2 (+1 partial) | [#15130](https://github.com/rust-lang/cargo/issues/15130): `[patch]` in `~/.cargo/config.toml` writes `[[patch.unused]]` into each project's `Cargo.lock`. [#9854](https://github.com/rust-lang/cargo/issues/9854): an unparsable `~/.config/git/config` breaks cargo for one user. |
| pypa/pip | 7 | 4 (+1 partial) | [#11058](https://github.com/pypa/pip/issues/11058), [#10350](https://github.com/pypa/pip/issues/10350): a global `init.defaultBranch` fails pip's tests locally. [#5957](https://github.com/pypa/pip/issues/5957): a test reads the developer's real `pip.conf`. [#5085](https://github.com/pypa/pip/issues/5085): `install.user = true` in a user file breaks `pip wheel`. |
| pytest-dev/pytest | 7 | 3 | [#2542](https://github.com/pytest-dev/pytest/issues/2542), [#4359](https://github.com/pytest-dev/pytest/issues/4359): `PYTEST_ADDOPTS` set in CI leaks into nested pytest runs. [#8256](https://github.com/pytest-dev/pytest/issues/8256): `~/.inputrc` or `INPUTRC` fails the pdb tests on some machines. |
| golang/go | 7 | 3 | [#37390](https://github.com/golang/go/issues/37390): `GOBIN` set with `go env -w` fails `all.bash`. [#80525](https://github.com/golang/go/issues/80525): `GOAMD64=v3` in the environment fails a test. [#64665](https://github.com/golang/go/issues/64665): `GOTOOLCHAIN` set with `go env -w` makes `gotip` run another release. |
| astral-sh/ruff | 8 | 2 (+1 partial) | [#2304](https://github.com/astral-sh/ruff/issues/2304), [#25046](https://github.com/astral-sh/ruff/issues/25046): the developer's `~/.config/ruff/ruff.toml` changes ruff's own test results. |
| vitest-dev/vitest | 5 | 1 | [#2732](https://github.com/vitest-dev/vitest/issues/2732): `CI` changes colour output, so snapshots fail in CI only. |
| jestjs/jest | 3 | 0 (+1 partial) | — |
| python/mypy | 4 | 0 | — |
| nextest-rs/nextest | 7 | 0 | — |

Two patterns stand out. First, a tool's own test suite reads the developer's real files. This happened in pip, cargo, ruff, pytest and Go. Second, an artifact made on one machine breaks in CI: uv's lock, and cargo's lock.

Maintainers decided some of these. The sub-agent's quotes, which I did not all re-read:

- ruff #2304: "We should run the tests under `--isolated` mode".
- pytest #2542: pytester clears `PYTEST_ADDOPTS`.
- pip #11058: the tests pass `--initial-branch` and stop relying on the user's gitconfig.
- cargo #7621: an option that ignores the parent files is open, as "S-needs-design".
- nextest [#1419](https://github.com/nextest-rs/nextest/issues/1419): the scope of a user file was an open question. The shipped answer, at 0.9.149, is display only (4.4).

## 6. A fault found in ruff 0.17.6

ruff picks the closest file for each source file. One case breaks this [run]:

```text
proj/ruff.toml        select = ["F401"]
proj/sub/ruff.toml    select = ["E711"]

(cd proj)     ruff check m.py sub/m.py   ->  m.py: F401   sub/m.py: E711   (as documented)
(cd proj/sub) ruff check m.py            ->  m.py: E711                    (as documented)
(cd proj/sub) ruff check m.py ../m.py    ->  m.py: F401   ../m.py: F401    (sub/m.py takes proj's file)
```

**Why [source].** The file found from the cwd becomes the root settings. Its directory goes into `seen` ([resolver.rs L475-L478](https://github.com/astral-sh/ruff/blob/0.17.6/crates/ruff_workspace/src/resolver.rs#L475-L478)), but not into the router that maps a path to its settings. The walk up from `../m.py` then adds `proj` to the router ([L480-L500](https://github.com/astral-sh/ruff/blob/0.17.6/crates/ruff_workspace/src/resolver.rs#L480-L500)). A lookup for `sub/m.py` matches `proj`, the longest prefix in the router. It never falls back to the root settings ([L136-L145](https://github.com/astral-sh/ruff/blob/0.17.6/crates/ruff_workspace/src/resolver.rs#L136-L145)).

Calling this a fault is my judgement. I found no ruff issue for it with five searches, among them "nested ruff.toml", "hierarchical configuration" and "subdirectory config not respected". It matters to purba only as a warning. One file's configuration can depend on the other arguments of the run.

## What was not checked

- **Windows and macOS.** Every path above is the Linux path. pip, uv, mypy and git name other paths there [source].
- **The real `/etc`.** The system files of uv, pip and git were faked through a variable (Method). `$GOROOT/go.env` was read and not changed. Its order below the go env file is [source] ([cfg.go L361-L370](https://github.com/golang/go/blob/go1.26.10/src/cmd/go/internal/cfg/cfg.go#L361-L370)).
- **uv's `[tool.uv]` in a script's metadata** against the files. I read only that it exists ([scripts.md L294-L310](https://github.com/astral-sh/uv/blob/0.13.7/docs/guides/scripts.md#L294-L310)).
- **The installed plugins as a level.** pytest loads every installed plugin through entry points. So the environment's packages change a run, outside every file. `PYTEST_DISABLE_PLUGIN_AUTOLOAD` and `-p no:` close this. I did not probe it.
- **Every variable of Jest and Vitest.** I probed only `CI`.
- **The issue search.** A sub-agent did it, and I re-read 4 of the 18 confirmed reports that its rows hold. GitHub's search ranks and caps results, so a count is a floor for its query, not for the tracker.

## What #1213 can take from this

This section is input for the prototype in [#1213](https://github.com/kalonji-tools/purba/issues/1213). It is not a decision. Each line gives one reason and the tools that do it.

**Candidate levels**

- **Command line, environment variable, code, project file.** All 11 tools read the first two. 10 read a project file, and 7 read code.
- **A file of the user: optional, and narrow when present.** 8 tools read one. It is the level behind each report in section 5. cargo-nextest limits it to display. ruff and mypy drop it when a project file exists.
- **A file of the system: rare.** 4 tools read one: uv, pip, git and Go. Each of them also manages installs or machines.

**Candidate rules**

- **`higher-overwrites`, for a plain value.** All 11 tools follow it: command line > environment > project > user > system.
- **`stricter-wins`.** No surveyed tool follows it between levels. purba's own cap on suppressed verdicts does. A prototype that keeps it is ahead of every tool here, and has no prior art to copy.
- **The narrowest scope wins, even over the command line.** pytest-timeout, pytest's `filterwarnings` and `xfail`, Jest, Vitest, mypy and ruff follow it. It answers #1212's scenario as 60 s.
- **The command line wins over code.** `go test` follows it, because it has no level in code for the timeout. cargo-nextest follows it, because its command line beats its per-test overrides. ruff's `--ignore-noqa` and Go's `GODEBUG` are a flag and a variable that win back over code. This rule answers the scenario as 5 s.
- **A list that each level extends, with a reset.** pytest, uv, cargo, git and pip follow it. The resets are git's empty value and pytest's `-o addopts=`. The other rule is "replace", followed by ruff's `select`, Go's `GOFLAGS`, and pip between files.
- **A file of the user is dropped when a project file exists**, as in ruff and mypy. The other rule is **merged below the project file**, as in uv, cargo, git, pip and Go. Dropping it keeps a project's outcome the same on every machine that has the project file.
- **A level is closed to a value.** git closes its repository file to `safe.directory`, for security. cargo-nextest closes its user file to every setting that runs tests. uv closes a `uv.toml` to project fields.
- **An explicit switch that ignores outside files.** Examples are ruff's and pip's `--isolated`, uv's `--no-config`, Go's `GOENV=off`, git's `GIT_CONFIG_NOSYSTEM` and `GIT_CONFIG_GLOBAL`, nextest's `--user-config-file none` and pip's `PIP_CONFIG_FILE=/dev/null`. Every tool with a file of the user has one.
- **No default that reads `CI`.** Jest and Vitest change an outcome on `CI` alone (4.5). [The driver scenario of an honest outcome](https://github.com/kalonji-tools/purba/blob/27e8f559/LANDSCAPE.md#L28) asks for the same outcome in two runners. A default that reads `CI` gives two outcomes in one runner.

**Candidate rules for finding the project file**

- **Walk up from the cwd or the arguments, and take the first file that has a section for the tool.** pytest, uv and mypy do this; ruff does it for each file. The walk skips a file with no section.
- **Stop the walk at the repository's root**, as mypy does at `.git`. This keeps a file above the project, such as one in `$HOME`, out of the run. pytest has no such stop [run].
- **One fixed place**, as cargo-nextest (the workspace root) and Vitest (the root, no walk) do.

## Appendix A: each tool

### A.1 pytest 9.2.0 (with pytest-timeout 2.4.0)

**Levels.**

- Command line, including `-c <file>`, `-o key=value` and `--rootdir`.
- `PYTEST_ADDOPTS`, plus a plugin's own variables.
- Code: marks, `pytestmark` on a module or class, and `conftest.py`.
- One project file.
- No file of the user or of the system.

**Finding the file.** pytest walks up from each argument. At each directory it checks `pytest.toml`, `.pytest.toml`, `pytest.ini`, `.pytest.ini`, `pyproject.toml`, `tox.ini` and `setup.cfg`, in that order. The first file with a pytest section wins. `pytest.toml` and `pytest.ini` win even when empty. A `pyproject.toml` with no table is kept as the fallback ([findpaths.py L245-L253](https://github.com/pytest-dev/pytest/blob/9.2.0/src/_pytest/config/findpaths.py#L245-L253), [L289-L327](https://github.com/pytest-dev/pytest/blob/9.2.0/src/_pytest/config/findpaths.py#L289-L327); [customize.rst L185-L227](https://github.com/pytest-dev/pytest/blob/9.2.0/doc/en/reference/customize.rst#L185-L227)). "Options from multiple configfiles candidates are never merged - the first match wins." (L226-L227).

**Order.** The file's `addopts` comes first, then `PYTEST_ADDOPTS`, then the command line. The last value of a flag wins ([`__init__.py` L1694-L1734](https://github.com/pytest-dev/pytest/blob/9.2.0/src/_pytest/config/__init__.py#L1694-L1734)). pytest-timeout: marker > `--timeout` > `PYTEST_TIMEOUT` > file ([L352-L386](https://github.com/pytest-dev/pytest-timeout/blob/2.4.0/pytest_timeout.py#L352-L386); [README L80-L104](https://github.com/pytest-dev/pytest-timeout/blob/2.4.0/README.rst#L80-L104)).

**Probe [run]** (`pytest.sh`, abridged):

```text
walk/pytest.ini, walk/a/pyproject.toml [tool.pytest.ini_options], walk/a/b/setup.cfg [tool:pytest]
  pytest walk/a/b/test_w.py                -> configfile: setup.cfg       (closest)
  setup.cfg without a section              -> configfile: pyproject.toml  (walk goes on)
  a/pyproject.toml without a table too     -> configfile: pytest.ini      (walk goes on)
--maxfail: file 1, PYTEST_ADDOPTS 2, CLI 3 -> 1 failed / 2 failed / 3 failed
-o addopts=                                -> 3 failed (the file's --maxfail=1 is gone)
-W from the file + PYTEST_ADDOPTS + CLI    -> 3 failed (all three applied)
timeout: file 3, PYTEST_TIMEOUT=1          -> plain: Timeout (>1.0s); marked 6: passed
PYTEST_TIMEOUT=1 --timeout 5               -> 2 passed
file 3, --timeout 1, marker 6              -> plain fails at 1 s; marked passes
pytestmark 1 s, function marker 6 s        -> module-marked fails; function-marked passes
filterwarnings: file error, mark ignore    -> 1 failed, 1 passed
file ignore, -W error, mark ignore         -> plain fails, marked passes
xfail_strict=true in file, xfail(strict=False) -> 1 xpassed
$HOME/pytest.ini, run in $HOME/proj        -> configfile: pytest.ini (rootdir: home)
```

### A.2 cargo-nextest 0.9.149

**Levels.**

- Command line.
- `NEXTEST_*` variables.
- The project file `.config/nextest.toml` at the workspace root, or `--config-file`.
- Tool files, through `--tool-config-file`.
- The user file `$XDG_CONFIG_HOME/nextest/config.toml`, for display only.
- No level in code. No file of the system.

**Order.** Command line > environment > per-test override > the selected profile > tool files > each inherited profile > default ([configuration/index.md L68-L90](https://github.com/nextest-rs/nextest/blob/cargo-nextest-0.9.149/site/src/docs/configuration/index.md#L68-L90)). Among overrides, the first one that matches wins, for each setting on its own ([per-test-overrides.md L95-L107](https://github.com/nextest-rs/nextest/blob/cargo-nextest-0.9.149/site/src/docs/configuration/per-test-overrides.md#L95-L107)). "`--retries` … and the `NEXTEST_RETRIES` environment variable both disable per-test overrides" ([retries.md L101-L103](https://github.com/nextest-rs/nextest/blob/cargo-nextest-0.9.149/site/src/docs/features/retries.md#L101-L103)). The user file is not merged: "the first matching path is always used" ([user-config/index.md L23](https://github.com/nextest-rs/nextest/blob/cargo-nextest-0.9.149/site/src/docs/user-config/index.md#L23)).

**Probe [run]** (`nextest.sh`). The test `flaky` fails on its first try only.

```text
profile.default retries=0, override test(flaky) retries=1   -> FLAKY 2/2 (the override wins)
NEXTEST_RETRIES=0                                           -> FAIL
NEXTEST_RETRIES=0 --retries 1                               -> FLAKY 2/2
--profile ci (retries=0), override only in default          -> FLAKY 2/2 (the inherited override wins over ci's profile value)
run from src/                                               -> FLAKY 2/2 (the root's file)
src/.config/nextest.toml with retries=0                     -> FLAKY 2/2 (not read)
user file [profile.default] retries=0                       -> "ignoring unknown configuration key: profile"; FLAKY 2/2
```

### A.3 Vitest 5.1.1

**Levels.**

- Command line.
- The environment: `CI` and others through `std-env`.
- Code: a test's options, and `vi.setConfig`.
- One project file: `vitest.config.*`, then `vite.config.*`, in the root only ([resolveConfig.ts L59-L65](https://github.com/vitest-dev/vitest/blob/v5.1.1/packages/vitest/src/node/config/resolveConfig.ts#L59-L65), [L1147-L1157](https://github.com/vitest-dev/vitest/blob/v5.1.1/packages/vitest/src/node/config/resolveConfig.ts#L1147-L1157)).
- No file of the user or of the system.

**Probe [run]** (`vitest.sh`). Each test sleeps 1500 ms.

```text
file testTimeout 1000                         -> plain × 1004ms, own 5000 ✓
--testTimeout 3000                            -> 2 passed
vi.setConfig({testTimeout:1000}) + CLI 3000   -> plain × (code wins)
from proj/sub, file in proj                   -> 2 passed (not read: no walk)
--root ..                                     -> plain × (read)
vitest.config 1000 + vite.config 3000         -> plain × (vitest.config wins)
test.only: no CI / CI=true / CI=true --allowOnly -> passed / "Unexpected .only modifier" / passed
new snapshot: CI=true / no CI                 -> "Snapshot `snap 1` mismatched" / "1 written"
```

### A.4 Jest 30.5.2

**Levels.**

- Command line.
- The environment: `CI` through `ci-info`.
- Code: `jest.setTimeout`, and a test's timeout argument.
- One project file.
- No file of the user or of the system.

**Finding the file.** Jest walks up from the cwd. At each directory it collects `jest.config.{js,ts,mjs,mts,cjs,cts,json}` and a `package.json` that has a `"jest"` key. Two files in one directory are refused. **A `package.json` with no `"jest"` key ends the walk**, and Jest uses the defaults ([resolveConfigPath.ts L67-L128](https://github.com/jestjs/jest/blob/v30.5.2/packages/jest-config/src/resolveConfigPath.ts#L67-L128)).

**Probe [run]** (`jest.sh`):

```text
file testTimeout 1000                     -> 1 failed, 1 passed (own 5000 wins)
--testTimeout 3000                        -> 2 passed
jest.setTimeout(1000) + CLI 3000          -> 1 failed, 1 passed (code wins)
sub/package.json without "jest"           -> 2 passed (the walk stops; proj's file not read)
no package.json in sub                    -> fails at 1000 ms (proj's file read)
jest.config.js + package.json "jest"      -> "Multiple configurations found"
no file up to /                           -> "Could not find a config file"
new snapshot: CI=true / no CI / CI=true --ci=false -> "New snapshot was not written" / written / written
```

### A.5 `go test` (go1.26.10)

**Levels.**

- Command line.
- The environment: `GOFLAGS`, `GODEBUG`, and every `GO*`.
- The go env file `$GOENV`, by default `$XDG_CONFIG_HOME/go/env`, written by `go env -w`.
- `$GOROOT/go.env`.
- `go.mod` or `go.work`, found by walking up from the cwd.
- Code: `//go:debug` in a `_test.go` file, `t.Parallel()`, and build tags.

**Order.** For a variable: the process environment, then the go env file, then `$GOROOT/go.env` ([cfg.go L361-L370](https://github.com/golang/go/blob/go1.26.10/src/cmd/go/internal/cfg/cfg.go#L361-L370), [L428-L442](https://github.com/golang/go/blob/go1.26.10/src/cmd/go/internal/cfg/cfg.go#L428-L442)). For a flag: "Flags listed on the command line are applied after this list and therefore override it" ([alldocs.go L2414-L2420](https://github.com/golang/go/blob/go1.26.10/src/cmd/go/alldocs.go#L2414-L2420)). For `GODEBUG`: the variable, then `//go:debug`, then `go.mod`, then the toolchain's default ([godebug.md L66-L72](https://github.com/golang/go/blob/go1.26.10/doc/godebug.md#L66-L72), [L140-L142](https://github.com/golang/go/blob/go1.26.10/doc/godebug.md#L140-L142)).

**Probe [run]** (`go.sh`):

```text
$GOROOT/go.env: GOPROXY=https://proxy.golang.org,direct  GOSUMDB=sum.golang.org  GOTOOLCHAIN=auto
test sleeps 2 s:
  no setting                                  -> ok
  go env -w GOFLAGS=-timeout=1s               -> panic: test timed out after 1s
  + GOFLAGS=-timeout=5s in the environment    -> ok
  + -timeout=1s on the command line           -> panic: test timed out after 1s
  GOENV=off                                   -> ok (the user file is dropped)
panic(nil), recovered value:
  go 1.26 only                                -> *runtime.PanicNilError
  go.mod: godebug panicnil=1                  -> <nil>
  + //go:debug panicnil=0 in the test file    -> *runtime.PanicNilError
  + GODEBUG=panicnil=1                        -> <nil>
  sub/ with no //go:debug                     -> <nil> (the parent go.mod applies)
```

### A.6 ruff 0.17.6

**Levels.**

- Command line: flags, `--config key=value`, `--config <file>` and `--isolated`.
- 4 `RUFF_*` variables, for output and cache only.
- Code: `# noqa` and `# ruff: noqa`.
- A project file: the closest one for each source file.
- A file of the user, `$XDG_CONFIG_HOME/ruff/{.ruff.toml,ruff.toml,pyproject.toml}`, but only when no project file is found from the cwd.

**Order** ([resolve.rs L27-L117](https://github.com/astral-sh/ruff/blob/0.17.6/crates/ruff/src/resolve.rs#L27-L117)):

1. `--isolated` gives the defaults.
2. `--config <file>` gives one file for every source file.
3. Otherwise, the closest project file to the cwd, with a lookup per source file.
4. Otherwise, the file of the user.
5. Otherwise, the defaults.

The command line's values go on top of each of these. "Ruff does not merge settings across configuration files" ([configuration.md L254-L278](https://github.com/astral-sh/ruff/blob/0.17.6/docs/configuration.md#L254-L278)). A `pyproject.toml` with no `[tool.ruff]` is skipped ([pyproject.rs L84-L115](https://github.com/astral-sh/ruff/blob/0.17.6/crates/ruff_workspace/src/pyproject.rs#L84-L115)).

**Probe [run]** (`ruff.sh`). The module trips F401, E741 and E711.

```text
user file select E741 only                 -> E741
+ proj/pyproject.toml without [tool.ruff]  -> E741 (still the user file)
+ proj/ruff.toml select F401               -> F401 (user file dropped, not merged)
+ proj/sub/ruff.toml select E711           -> m.py F401, sub/m.py E711
sub/ruff.toml extend="../ruff.toml" + extend-select E711 -> sub/m.py F401 + E711
--select E741                              -> E741 in both files
--extend-select E741                       -> each file's list + E741
--config ruff.toml                         -> F401 in both files
.ruff.toml / ruff.toml / pyproject.toml in one dir -> .ruff.toml wins, then ruff.toml
noqa on F401 and E741, --select all three  -> E711 only
+ --ignore-noqa                            -> all three
file ignore E741, --select F401,E741,E711  -> all three (the CLI's select resets the file's ignore)
```

Section 6 gives the fault.

### A.7 uv 0.13.7

**Levels.**

- Command line.
- `UV_*` variables.
- Code: `[tool.uv]` in a script's metadata [source].
- A project file: `uv.toml` or `[tool.uv]` in `pyproject.toml`, the first found from the project root, or else from the cwd.
- A file of the user: `$XDG_CONFIG_HOME/uv/uv.toml`.
- A file of the system: the first `uv/uv.toml` in `$XDG_CONFIG_DIRS`, or else `/etc/uv/uv.toml` ([uv-dirs lib.rs L163-L188](https://github.com/astral-sh/uv/blob/0.13.7/crates/uv-dirs/src/lib.rs#L163-L188)).

**Order.** Command line > environment > project > user > system. Scalars take the highest level. Arrays are joined, project first ([configuration-files.md L55-L74](https://github.com/astral-sh/uv/blob/0.13.7/docs/concepts/configuration-files.md#L55-L74); [crates/uv/src/lib.rs L300-L347](https://github.com/astral-sh/uv/blob/0.13.7/crates/uv/src/lib.rs#L300-L347)). `--config-file` replaces every file. `--no-config` and `UV_NO_CONFIG` drop every file. `UV_NO_SYSTEM_CONFIG` drops the file of the system.

**Probe [run]** (`uv.sh`, `uv pip compile --show-settings` and `uv lock --show-settings`):

```text
system resolution=lowest, no-build-package [fromsystem]      -> Lowest; [fromsystem]
+ user lowest-direct, [fromuser]                             -> LowestDirect; [fromuser, fromsystem]
+ pyproject.toml without [tool.uv]                           -> unchanged (skipped)
+ project uv.toml highest, [fromproject]                     -> Highest; [fromproject, fromuser, fromsystem]
+ [tool.uv] beside uv.toml                                   -> warning: "... will be ignored in favor of the uv.toml file"
sub/uv.toml inside a project, cwd sub                        -> proj's file (starts at the project root)
sub/uv.toml with no project, cwd sub                         -> [fromsub, fromuser, fromsystem]; LowestDirect
UV_RESOLUTION=lowest                                         -> Lowest
+ --resolution lowest-direct                                 -> LowestDirect
UV_NO_BUILD_PACKAGE=fromenv (uv lock)                        -> [fromenv, fromproject, fromuser, fromsystem]
+ --no-build-package fromcli                                 -> [fromcli, fromproject, fromuser, fromsystem]
--no-config / UV_NO_CONFIG=1                                 -> Highest, no array
--config-file named.toml                                     -> Lowest (only that file)
UV_NO_SYSTEM_CONFIG=1                                        -> [fromproject, fromuser]
managed = true in the user file / in the project uv.toml     -> refused: "not allowed in a uv.toml file"
```

### A.8 mypy 2.4.1

**Levels.**

- Command line, including `--config-file`.
- 6 `MYPY*` variables: `MYPYPATH`, `MYPY_CACHE_DIR`, `MYPY_FORCE_COLOR`, `MYPY_FORCE_TERMINAL_WIDTH`, `MYPY_NUM_WORKERS` and `MYPY_TEST_PREFIX`.
- Code: `# mypy:` and `# type: ignore`.
- One file: a project file, or else a file of the user.

**Finding the file.** mypy walks up from the cwd and stops at a `.git` or `.hg` directory. At each directory it checks `mypy.ini`, `.mypy.ini`, `pyproject.toml` with `[tool.mypy]`, and `setup.cfg` with `[mypy]`. If it finds none, it tries `$XDG_CONFIG_HOME/mypy/config`, `~/.config/mypy/config` and `~/.mypy.ini`. The first file wins, with no merge ([config_parser.py L285-L314](https://github.com/python/mypy/blob/v2.4.1/mypy/config_parser.py#L285-L314); [defaults.py L20-L26](https://github.com/python/mypy/blob/v2.4.1/mypy/defaults.py#L20-L26); [config_file.rst L10-L32](https://github.com/python/mypy/blob/v2.4.1/docs/source/config_file.rst#L10-L32)).

**Order.** Inline comment > a section that names the module > a pattern section > command line > `[mypy]` ([config_file.rst L74-L84](https://github.com/python/mypy/blob/v2.4.1/docs/source/config_file.rst#L74-L84)).

**Probe [run]** (`mypy.sh`):

```text
user file only, cwd repo/a/b                 -> Config File: home/.config/mypy/config; error
+ repo/mypy.ini above                        -> Config File: repo/mypy.ini; Success (user file dropped)
+ repo/a/.git                                -> Config File: home/.config/mypy/config (walk stopped)
run from another dir on repo/a/b/m.py        -> Config File: home/.config/mypy/config (walk starts at the cwd)
repo/a/pyproject.toml without [tool.mypy]    -> Config File: repo/mypy.ini (walk goes on)
[mypy] True + --allow-untyped-defs           -> Success (CLI beats [mypy])
[mypy] False, [mypy-m] True + --allow-untyped-defs -> error ([mypy-m] beats CLI)
[mypy] True, [mypy-m] False + --disallow-untyped-defs -> Success ([mypy-m] beats CLI)
# mypy: allow-untyped-defs + [mypy-m] True + CLI    -> Success (inline beats both)
[mypy-m] False + --strict                    -> Success
```

From `/`, mypy 2.4.1 stopped with "INTERNAL ERROR", so the probe runs from an empty directory instead.

### A.9 pip 26.3

**Levels.**

- Command line.
- `PIP_*` variables.
- `PIP_CONFIG_FILE`.
- The site file `$VIRTUAL_ENV/pip.conf`.
- Files of the user: `~/.pip/pip.conf`, then `$XDG_CONFIG_HOME/pip/pip.conf`.
- Files of the system: `$XDG_CONFIG_DIRS/pip/pip.conf`, then `/etc/pip.conf`.
- No project file. No level in code.

**Order.** `OVERRIDE_ORDER = GLOBAL, USER, SITE, ENV, ENV_VAR`, and each later one wins ([configuration.py L53](https://github.com/pypa/pip/blob/26.3/src/pip/_internal/configuration.py#L53)). An existing `PIP_CONFIG_FILE` drops the user files ([L360-L390](https://github.com/pypa/pip/blob/26.3/src/pip/_internal/configuration.py#L360-L390)). In a file, `[<command>]` beats `[global]`. The command line beats `PIP_*` ([configuration.md L118-L132](https://github.com/pypa/pip/blob/26.3/docs/html/topics/configuration.md#L118-L132), [L249-L258](https://github.com/pypa/pip/blob/26.3/docs/html/topics/configuration.md#L249-L258)). A file's value is the default of an option that appends, so the command line adds to it ([parser.py L307-L309](https://github.com/pypa/pip/blob/26.3/src/pip/_internal/cli/parser.py#L307-L309)).

**Probe [run]** (`pip.sh`, "Looking in indexes:"):

```text
system / + user legacy / + user XDG / + site     -> system / user-legacy / user-xdg / site
[install] beside [global] in the site file        -> site-install-section
PIP_CONFIG_FILE=envfile.conf                      -> envfile
PIP_INDEX_URL / + --index-url                     -> envvar / cli
PIP_INDEX_URL + --isolated                        -> system (user files and PIP_* dropped)
extra-index-url: user x-user, site x-site         -> base, x-site (files replace)
+ --extra-index-url x-cli                         -> base, x-site, x-cli (CLI appends)
+ PIP_EXTRA_INDEX_URL=x-env                       -> base, x-env, x-cli
```

### A.10 cargo 1.99.0

**Levels.**

- `--config key=value` or `--config <file>`.
- `CARGO_*` variables.
- Every `.cargo/config.toml` from the cwd up to `/`, merged.
- `$CARGO_HOME/config.toml`.
- No level in code. No file of the system.

**Order.** `--config` > environment > the deepest file > … > `$CARGO_HOME`. Arrays are joined, "with higher precedence items being placed later" ([config.md L7-L45](https://github.com/rust-lang/cargo/blob/0.100.0/doc/book/src/reference/config.md#L7-L45), [L212-L249](https://github.com/rust-lang/cargo/blob/0.100.0/doc/book/src/reference/config.md#L212-L249)).

**Probe [run]** (`cargo.sh`, `RUSTC_BOOTSTRAP=1 cargo -Zunstable-options config get --show-origin`):

```text
jobs: home 1, top 2, leaf 4; cwd leaf / cwd mid  -> 4 (leaf) / 2 (top)
rustflags: home, top, mid                        -> [home, top, mid]
term.verbose only in $CARGO_HOME                 -> true (cargo-home/config.toml)
CARGO_BUILD_JOBS=8 / + --config build.jobs=16    -> 8 / 16
CARGO_BUILD_RUSTFLAGS=--cfg=env                  -> [home, top, mid, env]
--config 'build.rustflags=["--cfg=cli"]'         -> [home, top, mid, cli]
--config named.toml                              -> 32 (named.toml)
cargo build -v --manifest-path …/leaf from elsewhere -> --cfg=home only
cargo build -v from leaf                          -> --cfg=home, --cfg=mid, --cfg=top
```

### A.11 git 2.54.0

**Levels.**

- `-c`, and `GIT_CONFIG_COUNT`/`KEY`/`VALUE` (the "command" scope).
- `$GIT_DIR/config.worktree` (needs `extensions.worktreeConfig`).
- `$GIT_DIR/config`.
- `~/.gitconfig` and `$XDG_CONFIG_HOME/git/config`.
- `$(prefix)/etc/gitconfig`, or `GIT_CONFIG_SYSTEM`.

`GIT_CONFIG_NOSYSTEM` and `GIT_CONFIG_GLOBAL` drop or replace a scope. No level in code.

**Order.** "The files are read in the order given above, with last value found taking precedence over values read earlier. When multiple values are taken then all values of a key from all files will be used" ([git-config.adoc L368-L405](https://github.com/git/git/blob/v2.54.0/Documentation/git-config.adoc#L368-L405)). `GIT_CONFIG_COUNT` pairs "override values in configuration files, but will be overridden by any explicit options passed via `git -c`" (L480-L493).

**Probe [run]** (`git.sh`):

```text
probe.key at every scope, --show-scope --get-all -> system, global (xdg), global (home), local
git config --get (repo) / (worktree)              -> local / worktree
-c probe.key=command                              -> command
GIT_CONFIG_COUNT=1 …=env-count                    -> env-count
GIT_CONFIG_COUNT + -c, --get-all                  -> … command env-count, command command (-c last)
GIT_CONFIG_NOSYSTEM=1 / GIT_CONFIG_GLOBAL=/dev/null -> system dropped / global dropped
credential.helper: system, global, "", local      -> all four listed
GIT_TEST_ASSUME_DIFFERENT_OWNER=1:
  no exception                                    -> fatal: detected dubious ownership
  safe.directory=* in the local file              -> fatal: detected dubious ownership
  -c safe.directory=*                             -> runs
  safe.directory=* in ~/.gitconfig                -> runs
core.editor=from-config; GIT_EDITOR; + -c core.editor -> from-config; from-env; from-env
```

## Appendix B: the queries of the issue search

The search API took `repo:<repo> is:issue <query>`, and the hit count is its `total_count`. A row marked "phrase" used `gh search issues`. That command wraps the words as one phrase and caps the count at 100. The raw log is not committed.

| tracker | query | hits |
|---|---|---|
| astral-sh/ruff | `"user config"` (phrase) | 3 |
| | `"global config"` (phrase) | 19 |
| | `"~/.config/ruff"` (phrase) | 85 |
| | `"user-level" configuration ignored project` | 5 |
| | `merge user configuration with project configuration` | 41 |
| astral-sh/uv | `"user config" CI` (phrase) | 0 |
| | `"~/.config/uv"` (phrase) | 100 |
| | `"uv.toml" "works locally"` (phrase) | 0 |
| | `"user-level" uv.toml lockfile different` (phrase) | 0 |
| | `user uv.toml lockfile different machine` | 14 |
| | `"--no-config" CI user configuration` | 11 |
| | `user-level configuration not allowed OR ignored OR refuse` | 118 |
| pytest-dev/pytest | `PYTEST_ADDOPTS` (phrase) | 33 |
| | `"home directory" ini rootdir` (phrase) | 0 |
| | `"works locally" CI environment variable` (phrase) | 0 |
| | `rootdir ini "parent directory" picked` | 3 |
| | `tests fail environment variable set locally` | 12 |
| python/mypy | `"~/.config/mypy"` (phrase) | 46 |
| | `"user config" mypy` (phrase) | 0 |
| | `"~/.mypy.ini"` (phrase) | 100 |
| | `"config/mypy/config"` | 170 |
| | `"user config file" OR "user-level config" OR "global config"` | 56 |
| | `different results locally CI environment variable` | 2 |
| pypa/pip | six phrase queries on `pip.conf`, `PIP_`, user config and isolation | 0 each |
| | `pip.conf tests fail` | 26 |
| | `PIP_ environment variable test suite fails` | 1 |
| | `config file "works locally" CI` | 0 |
| | `gitconfig tests fail` | 1 |
| | `"user configuration" tests` | 25 |
| rust-lang/cargo | six phrase queries on `~/.cargo/config`, `CARGO_HOME` and the test suite | 0, 0, 0, 1, 0, 0 |
| | `"cargo/config" tests fail home` | 59 |
| | `testsuite fails "config.toml" home directory` | 2 |
| | `CARGO_ environment variable "works locally" CI` | 0 |
| | `gitconfig tests fail` | 6 |
| | `"global config" rustflags unexpected` | 3 |
| | `testsuite fails when environment variable set` | 4 |
| | `tests fail RUSTC_WRAPPER OR CARGO_TARGET_DIR OR CARGO_BUILD_TARGET set` | 251 |
| | `"$CARGO_HOME/config" OR "~/.cargo/config.toml" unexpected` | 326 |
| golang/go | `GOFLAGS tests fail set environment` | 184 |
| | `"go env -w" unexpected` | 44 |
| | `GOENV config file tests` | 815 |
| | `GOFLAGS in:title test` | 2 |
| | `"go env -w" in:title` | 12 |
| | `cmd/go tests fail "go env -w" OR GOENV OR "go/env"` | 1877 |
| | `all.bash fails GOFLAGS OR "go env -w" OR GOENV OR "CGO_CFLAGS"` | 383 |
| nextest-rs/nextest | `user config` | 34 |
| | `environment variable CI locally different` | 3 |
| | `"cargo/config" OR "config.toml" home` | 38 |
| | `locally CI different OR "works locally" OR "on my machine"` | 13 |
| jestjs/jest | `config "parent directory"` | 17 |
| | `"home directory" config` | 9 |
| | `"works locally" CI environment variable` | 5 |
| | `CI environment variable snapshot "not written"` | 3 |
| | `snapshot CI "--ci" environment variable written` | 8 |
| | `babelrc OR "babel.config" home directory picked up` | 0 |
| vitest-dev/vitest | `config "parent directory"` | 8 |
| | `"works locally" CI env` | 3 |
| | `"home directory" config` | 4 |
| | `"CI" environment variable behaves differently locally` | 0 |
| | `config picked up outside project OR "home directory"` | 3 |
| | `CI env snapshot written OR "process.env.CI"` | 15 |
