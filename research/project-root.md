# How tools find the root of a project, in which order, and what they do when nothing marks it

Research for [Which ways do tools use to find the root of a project, and in which order do they try them?](https://github.com/kalonji-tools/purba/issues/1238)
Measured on 2026-11-25, against purba `main` at `cfc09617`, on one Linux machine.
This file decides nothing. It surveys 18 tools, merges their ways into one list, and weighs the proposed hierarchy and the toggle `implicit_root` against purba's records.
It builds on [the findings of #1212](https://github.com/kalonji-tools/purba/blob/5cdfe06d4258fdcc4bd27fa9de8c8e1da481c4b8/research/configuration-levels.md?plain=1#L105-L121), which asked where each tool finds its file of configuration. This file asks where the root is, and what a tool does when no marker matches.

## Summary

Most tools surveyed find their root by a walk up from one start: the working folder, the common folder of the arguments, or each file's own folder. coverage.py and Vitest do not walk. The walk looks for a marker. The marker is a file of the tool, a table in a shared file, a manifest of the language, or a folder of version control. The tools split on what they do when the walk finds nothing. Six take the working folder or a folder near it: pytest, Ruff, tox, coverage.py, Vitest and npm. Black takes the root of the file system. Seven refuse: cargo, Go, git, Jujutsu, pre-commit, Jest, and uv's project commands. Five of the seven have a flag that names the root. pre-commit takes git's variables. Jest refuses even with `--rootDir`, and runs with no file only when `--config` holds its settings as a JSON string.

Six findings matter most for purba.

- **No command-line tool surveyed has a toggle between a fallback and a refusal.** Each tool has one fallback, fixed in its code. The one toggle found is in an editor package: Projectile's `projectile-require-project-root` is `t` (refuse), `nil` (the working folder) or `prompt` (ask), and its default is `prompt` [source]. Projectile also tries an ordered list of five root functions, which a user can reorder [source].
- **A fallback to the working folder moves the id of a test.** With no marker, pytest names one test `a/test_a.py::test_a` from `p`, `test_a.py::test_a` from `p/a`, and `p/a/test_a.py::test_a` from a sibling folder with two arguments [run]. With `[tool.pytest]` in `p/pyproject.toml`, the id is `a/test_a.py::test_a` from `p` and from `p/a` [run].
- **A way that reads only the tree keeps one id from any folder.** nextest lists `c tests::adds` from the workspace and from `crates/c/src` [run]. `go list` prints `example.com/m/a` from `p` and from `p/a` [run]. mypy names `pkg.mod` from both folders once `pkg/__init__.py` exists, and `mod` or `pkg.mod` by the working folder under `--explicit-package-bases` [run].
- **A refusal can name the marker that it did find.** With `p/.git` and no `go.mod`, `go test` prints `cannot find main module, but found .git/config in <dir>/p`, and the command `cd .. && go mod init` [run]. jj gives a hint to run `jj git init`, but only when the working folder itself holds `.git` [run].
- **A shared file with no table for the tool is read in four ways.** Ruff and Black skip a `pyproject.toml` with no table, and walk on [run]. pytest and tox use the nearest one as the root when nothing else matches [run]. uv stops at it and refuses: ``No `project` table found`` [run]. Jest stops at a `package.json` with no `"jest"` key, and uses its folder as `rootDir` [run].
- **No level of purba's configuration holds the toggle in the case that it is for.** The toggle acts only when no `pyproject.toml` holds `[tool.purba]`, so the project's file is absent. The user's file and the system's file set only a dial that changes no outcome ([L63](https://github.com/kalonji-tools/purba/blob/cfc09617/docs/decisions/purba-reads-configuration-from-six-levels.md?plain=1#L63)). judgement: a root moves which tests `purba` runs and the id of each, so these two files cannot hold it. That leaves a flag or a variable, and no flag selects a test ([L86](https://github.com/kalonji-tools/purba/blob/cfc09617/docs/decisions/purba-picks-a-test-by-its-name-and-the-tests-of-a-run-by-one-query.md?plain=1#L86)). Section 4 weighs it.

## How to read this file

- **[run]** means I ran it in this session, on the version named. **[source]** means the tool's code or documentation says so, at the pinned tag, and I did not run it. **judgement** marks a claim that no source states.
- The **root** is the folder from which a tool names paths, or under which it looks for its units. Tools call it the rootdir, the project root, `tox_root`, the top level, the workspace root, `rootDir`, the main module or the local prefix.
- A **marker** is what the walk looks for. A **fallback** is what the tool does when no marker matches.
- `p` is the folder of the project in each probe, and `p/a` is a folder below it. `<dir>` stands for the probe's empty folder.
- **#1212** is [the findings of #1212](https://github.com/kalonji-tools/purba/blob/5cdfe06d4258fdcc4bd27fa9de8c8e1da481c4b8/research/configuration-levels.md). A row that cites it reuses its [run].
- A count answers only its own matcher. "Seven refuse" counts the tools of section 1 whose main command refuses with no marker: cargo, `go`, git, Jujutsu, pre-commit, Jest, and uv's project commands.

## Method

### Versions

| tool | version | how I ran it | pinned source |
|---|---|---|---|
| pytest | 9.2.0, on CPython 3.13.13 | a `uv` venv | [`9.2.0`](https://github.com/pytest-dev/pytest/tree/9.2.0), commit `ae1d0ff5` |
| Ruff | 0.17.6 | the same venv | [`0.17.6`](https://github.com/astral-sh/ruff/tree/0.17.6), commit `a7ba68b4919e56b6af949a42b2da38fe4eec67a9` |
| mypy | 2.4.1 | the same venv | [`v2.4.1`](https://github.com/python/mypy/tree/v2.4.1), commit `ad02a35e` |
| Black | 26.10.1 | the same venv | [`26.10.1`](https://github.com/psf/black/tree/26.10.1), commit `258e7dbf` |
| coverage.py | 7.17.5 | the same venv | [`7.17.5`](https://github.com/coveragepy/coveragepy/tree/7.17.5), commit `f1a83cfa` |
| tox | 4.67.16 | the same venv | [`4.67.16`](https://github.com/tox-dev/tox/tree/4.67.16), commit `2e2fab6b` |
| pre-commit | 4.7.0 | the same venv | [`v4.7.0`](https://github.com/pre-commit/pre-commit/tree/v4.7.0), commit `65bb884a` |
| EditorConfig | editorconfig-core-py 0.17.1 | the same venv | [`v0.17.1`](https://github.com/editorconfig/editorconfig-core-py/tree/v0.17.1), commit `5efc89badd322e31234e5987584ebc7f5a74334f` |
| uv | 0.13.7 | on `PATH` | [`0.13.7`](https://github.com/astral-sh/uv/tree/0.13.7), commit `316c827e` |
| cargo | 1.99.0 | on `PATH` | [`0.100.0`](https://github.com/rust-lang/cargo/tree/0.100.0), commit `5f94df47` |
| cargo-nextest | 0.9.149 | on `PATH` | not cited |
| git | 2.54.0 | on `PATH` | [`v2.54.0`](https://github.com/git/git/tree/v2.54.0), commit `94f05775` |
| Go | go1.26.10 | `nix-shell -p go` | [`go1.26.10`](https://github.com/golang/go/tree/go1.26.10), commit `4e8c85f5` |
| Jujutsu | 0.41.0 | `nix-shell -p jujutsu` | [`v0.41.0`](https://github.com/jj-vcs/jj/tree/v0.41.0), commit `413f5392` |
| Jest | 30.5.2 | `npm install`, Node v24.22.1 | [`v30.5.2`](https://github.com/jestjs/jest/tree/v30.5.2), commit `202dd8a1` |
| Vitest | 5.1.1 | `npm install`, Node v24.22.1 | [`v5.1.1`](https://github.com/vitest-dev/vitest/tree/v5.1.1), commit `161e25f6` |
| npm | 11.19.1 | `nix-shell -p nodejs` | [`v11.19.1`](https://github.com/npm/cli/tree/v11.19.1), commit `3acf9a78` |
| Projectile | 3.4.0 | not run | [`v3.4.0`](https://github.com/bbatsov/projectile/tree/v3.4.0), commit `b60bb099` |

### Probes

Each probe is `research/project-root/<tool>/run.sh`. It writes a small tree into an empty folder that it is given, and runs the tool there. It points `HOME` into that folder, so no file of the user reaches it. `out.txt` beside it holds this session's output, with the folder's path replaced by `<dir>`, and the spaces at the end of a line removed. No folder above the empty folder held a marker of any tool surveyed.

```sh
R=research/project-root
bash $R/pytest/run.sh <pytest 9.2.0> <empty dir>
bash $R/ruff/run.sh <ruff 0.17.6> <empty dir>
bash $R/mypy/run.sh <mypy 2.4.1> <empty dir>
bash $R/black/run.sh <black 26.10.1> <empty dir>
bash $R/coverage/run.sh <coverage 7.17.5> <empty dir>
bash $R/tox/run.sh <tox 4.67.16> <empty dir>
PRE_COMMIT_HOME=<empty dir> bash $R/precommit/run.sh <pre-commit 4.7.0> <empty dir>
bash $R/editorconfig/run.sh <editorconfig 0.17.1> <empty dir>
bash $R/uv/run.sh <python 3.13.13> <empty dir>
bash $R/cargo/run.sh <empty dir>
bash $R/git/run.sh <empty dir>
nix-shell -p go --run "bash $R/go/run.sh <empty dir>"
nix-shell -p jujutsu --run "bash $R/jj/run.sh <empty dir>"
nix-shell -p nodejs --run "bash $R/jest/run.sh <dir with node_modules/jest> <empty dir>"
nix-shell -p nodejs --run "bash $R/vitest/run.sh <dir with node_modules/vitest> <empty dir>"
nix-shell -p nodejs --run "bash $R/npm/run.sh <empty dir>"
```

## 1. Each tool

Each tool has five fields: the markers, the order of the walk, the fallback, the flags and variables, and whether a printed id or path depends on the working folder. "§n" names a section of the tool's `out.txt`.

### 1.1 pytest, `rootdir`

- **Markers.** `pytest.toml`, `.pytest.toml`, `pytest.ini`, `.pytest.ini`, `pyproject.toml` with `[tool.pytest]` or `[tool.pytest.ini_options]`, `tox.ini` with `[pytest]`, `setup.cfg` with `[tool:pytest]`. Then `setup.py`. Then a `pyproject.toml` with no table. `.git` marks nothing: with `p/.git` only, the rootdir is `p/a` [run, §2].
- **Order.** The walk starts at the common folder of the arguments, or the working folder with no argument. It tries the files of configuration up to the root of the file system. Then it tries `setup.py` upwards. Then it tries the files from each argument upwards. It does not stop at `.git` ([`customize.rst` L185-L212](https://github.com/pytest-dev/pytest/blob/9.2.0/doc/en/reference/customize.rst?plain=1#L185-L212); [`findpaths.py` L463-L481](https://github.com/pytest-dev/pytest/blob/9.2.0/src/_pytest/config/findpaths.py#L463-L481)). `p/setup.py` gives `p` [run, §3]. `p/pyproject.toml` with no table gives `p` [run, §4]. A `pytest.ini` in `$HOME` is found from below `$HOME` (#1212 [run], [L117](https://github.com/kalonji-tools/purba/blob/5cdfe06d4258fdcc4bd27fa9de8c8e1da481c4b8/research/configuration-levels.md?plain=1#L117)).
- **No marker.** The common folder of the working folder and the arguments' folder. If that is the root of the file system, the arguments' folder ([`findpaths.py` L476-L481](https://github.com/pytest-dev/pytest/blob/9.2.0/src/_pytest/config/findpaths.py#L476-L481)). From `p/a` with `. ../b`, the rootdir is `p`. From a sibling `elsewhere` with `../p/a ../p/b`, the rootdir is `<dir>`, the folder above both [run, §1]. No refusal.
- **Flags.** `--rootdir=<dir>`, on the command line only, since the rootdir finds the file ([#1212 L227](https://github.com/kalonji-tools/purba/blob/5cdfe06d4258fdcc4bd27fa9de8c8e1da481c4b8/research/configuration-levels.md?plain=1#L227)). `-c <file>` sets the rootdir to the file's folder. `-c /dev/null` reads no file, and the rootdir falls back to the arguments' folder [run, §7]; [`findpaths.py` L451-L462](https://github.com/pytest-dev/pytest/blob/9.2.0/src/_pytest/config/findpaths.py#L451-L462).
- **The id.** "each test is assigned a unique nodeid which is rooted at the rootdir" ([`customize.rst` L169-L171](https://github.com/pytest-dev/pytest/blob/9.2.0/doc/en/reference/customize.rst?plain=1#L169-L171)). With no marker, the id changes with the working folder [run, §1]. With a marker, it does not [run, §5]. With `--rootdir=b`, `b/test_b.py` becomes `test_b.py::test_b`, and `a/test_a.py`, outside the rootdir, keeps `a/test_a.py::test_a` [run, §6].

### 1.2 Ruff, `project_root`

- **Markers.** `.ruff.toml`, `ruff.toml`, or `pyproject.toml` with `[tool.ruff]`, in that order in each folder ([`pyproject.rs` L84-L116](https://github.com/astral-sh/ruff/blob/0.17.6/crates/ruff_workspace/src/pyproject.rs#L84-L116)). `.git` and a `pyproject.toml` with no `[tool.ruff]` mark nothing [run, §2].
- **Order.** `--isolated`, then `--config`, then the walk up from the working folder, then the user's file, then the defaults ([`resolve.rs` L27-L119](https://github.com/astral-sh/ruff/blob/0.17.6/crates/ruff/src/resolve.rs#L27-L119)). The walk goes to the root of the file system. Each source file then takes its own nearest file (#1212, [L118](https://github.com/kalonji-tools/purba/blob/5cdfe06d4258fdcc4bd27fa9de8c8e1da481c4b8/research/configuration-levels.md?plain=1#L118)).
- **No marker.** The working folder: `project_root` is `p` from `p`, and `p/a` from `p/a` [run, §1]. A user's file keeps the working folder as the root [run, §5]; [`resolver.rs` L450-L459](https://github.com/astral-sh/ruff/blob/0.17.6/crates/ruff_workspace/src/resolver.rs#L450-L459). No refusal.
- **Flags.** `--config <file>` reads that file, and makes the working folder the root: `project_root` is `p/a` with `--config p/pyproject.toml` [run, §4]. `--isolated` reads no file.
- **The path.** A diagnostic names its file relative to the working folder, whatever the root: `a/m.py` from `p`, `m.py` from `p/a`, with `[tool.ruff]` in `p` [run, §3].

### 1.3 mypy

mypy has two roots. One is where its walk for a settings file stops. The other is the base folder from which it names a module.

- **Markers.** For the settings file: `mypy.ini`, `.mypy.ini`, `pyproject.toml` with `[tool.mypy]`, `setup.cfg` with `[mypy]` (#1212, [L542](https://github.com/kalonji-tools/purba/blob/5cdfe06d4258fdcc4bd27fa9de8c8e1da481c4b8/research/configuration-levels.md?plain=1#L542)). For a module's name: `__init__.py` in each folder, or a base from `MYPYPATH`, `mypy_path` or the working folder under `--explicit-package-bases` ([`find_sources.py` L81-L94](https://github.com/python/mypy/blob/v2.4.1/mypy/find_sources.py#L81-L94), [L142-L224](https://github.com/python/mypy/blob/v2.4.1/mypy/find_sources.py#L142-L224)).
- **Order.** The walk for the file starts at the working folder. It stops after a folder that holds `.git` or `.hg`, or at the root of the file system ([`config_parser.py` L285-L313](https://github.com/python/mypy/blob/v2.4.1/mypy/config_parser.py#L285-L313)). `outer/mypy.ini` is read from `outer/p/pkg`, and is not read once `outer/p/.git` exists [run, §1-§2]. The walk for a module's name goes up from the file while each folder holds `__init__.py`.
- **No marker.** The user's file, then the defaults. A module with no `__init__.py` above it is named by its file alone: `mod` [run, §3]. No refusal.
- **Flags.** `--config-file`, `MYPYPATH`, `--explicit-package-bases`.
- **The id.** Without `__init__.py`, `pkg/mod.py` is `mod` from `p` and from `p/pkg` [run, §3]. Under `--explicit-package-bases`, it is `pkg.mod` from `p` and `mod` from `p/pkg` [run, §4]. With `pkg/__init__.py`, it is `pkg.mod` from both [run, §5].

### 1.4 Black

- **Markers.** In each folder, in this order: `.git` (a folder or a file), `.hg`, and `pyproject.toml` with `[tool.black]` ([`files.py` L95-L107](https://github.com/psf/black/blob/26.10.1/src/black/files.py#L95-L107)). A `pyproject.toml` with no `[tool.black]` is skipped [run, §3].
- **Order.** The walk starts at the common folder of the arguments, or the working folder with none ([`files.py` L67-L93](https://github.com/psf/black/blob/26.10.1/src/black/files.py#L67-L93)). The nearest folder with any marker wins: `p/a/pyproject.toml` with `[tool.black]` beats `p/.git` [run, §4].
- **No marker.** The root of the file system: ``Identified `/` as project root containing a file system root`` [run, §1]. Arguments in `p/a` and in `q` have the common folder `<dir>`, above `p/.git`, so the root is `/` again [run, §5]. The documentation says so ([`the_basics.md` L325-L328](https://github.com/psf/black/blob/26.10.1/docs/usage_and_configuration/the_basics.md?plain=1#L325-L328)). No refusal.
- **Flags.** `--config` names the settings file. I found no flag that names the root.
- **The path.** The root anchors the patterns of `--exclude` ([`the_basics.md` L324-L326](https://github.com/psf/black/blob/26.10.1/docs/usage_and_configuration/the_basics.md?plain=1#L324-L326)). The printed path is unmeasured.

### 1.5 uv, the project and the workspace

- **Markers.** The project: the nearest `pyproject.toml`, which must hold `[project]`. The workspace: a `pyproject.toml` above it with `[tool.uv.workspace]` whose `members` include the project ([`workspace.rs` L459-L475](https://github.com/astral-sh/uv/blob/0.13.7/crates/uv-workspace/src/workspace.rs#L459-L475), [L1906-L2023](https://github.com/astral-sh/uv/blob/0.13.7/crates/uv-workspace/src/workspace.rs#L1906-L2023)). `.git` marks nothing [run, §2].
- **Order.** The walk starts at the working folder, or at `--project`. The first `pyproject.toml` ends it, whatever it holds: `p/pyproject.toml` with only `[tool.ruff]` gives ``No `project` table found`` [run, §3]. A second walk from the project looks for a workspace; it skips the folder of uv's cache. `w/pkgs/m/src` gives the workspace root `w` [run, §5].
- **No marker.** A project command refuses: ``No `pyproject.toml` found in current directory or any parent directory``, exit 2 [run, §1]; [`workspace.rs` L308](https://github.com/astral-sh/uv/blob/0.13.7/crates/uv-workspace/src/workspace.rs#L308). `uv run` runs with no project, and so does `uv run --no-project` [run, §1].
- **Flags.** `--project <dir>` and `UV_PROJECT` start the walk elsewhere. `--directory <dir>` and `UV_WORKING_DIRECTORY` change the working folder first ([`lib.rs` L145-L153](https://github.com/astral-sh/uv/blob/0.13.7/crates/uv/src/lib.rs#L145-L153), [L207-L230](https://github.com/astral-sh/uv/blob/0.13.7/crates/uv/src/lib.rs#L207-L230)). Each gives `w` from `p/a` [run, §6].
- **The path.** uv prints no id of a test. The root does not depend on the folder below the project [run, §4-§5].

### 1.6 coverage.py

- **Markers.** None. coverage.py tries `.coveragerc`, `.coveragerc.toml`, `setup.cfg`, `tox.ini` and `pyproject.toml` as relative names, so only in the working folder ([`config.py` L645-L674](https://github.com/coveragepy/coveragepy/blob/7.17.5/coverage/config.py#L645-L674)).
- **Order.** No walk. `p/.coveragerc` is read from `p`, and not from `p/a` [run, §1].
- **No marker.** The working folder is the root of each relative name ([`files.py` L29-L62](https://github.com/coveragepy/coveragepy/blob/7.17.5/coverage/files.py#L29-L62)). No refusal.
- **Flags.** `COVERAGE_RCFILE` names the file [run, §2]. `--rcfile` names it too [run, §4]. `[run] relative_files` changes how the data file stores names.
- **The path.** A report names `a/m.py` from `p`, and `m.py` from `p/a` [run, §3].

### 1.7 tox, `tox_root`

- **Markers.** `tox.ini`, `setup.cfg`, `pyproject.toml`, `tox.toml`, each with a tox section, in that order in each folder ([`discover.py` L28-L34](https://github.com/tox-dev/tox/blob/4.67.16/src/tox/config/source/discover.py#L28-L34), [L74-L88](https://github.com/tox-dev/tox/blob/4.67.16/src/tox/config/source/discover.py#L74-L88)). `.git` marks nothing [run, §2].
- **Order.** The walk starts at the working folder and goes to the root of the file system.
- **No marker.** The folder of the nearest `pyproject.toml`, whatever it holds; else the working folder. tox warns each time: `No loadable tox.ini or setup.cfg or pyproject.toml or tox.toml found, assuming empty tox.ini at <dir>/p` [run, §1, §3]; [`discover.py` L91-L102](https://github.com/tox-dev/tox/blob/4.67.16/src/tox/config/source/discover.py#L91-L102). No refusal.
- **Flags.** `--root <dir>` sets `tox_root`; `-c <file or dir>` names the file [run, §5-§6]; [`parser.py` L567-L594](https://github.com/tox-dev/tox/blob/4.67.16/src/tox/config/cli/parser.py#L567-L594).
- **The path.** tox names an environment, not a path. With a marker, `tox_root` is `p` from `p/a` [run, §4]. Under the fallback, it moves with the working folder [run, §1].

### 1.8 pre-commit

- **Markers.** git's top level, from `git rev-parse --show-cdup` ([`git.py` L51-L73](https://github.com/pre-commit/pre-commit/blob/v4.7.0/pre_commit/git.py#L51-L73)).
- **Order.** git's walk (1.11). Then pre-commit changes to the top level, and rewrites each path relative to it ([`main.py` L175-L199](https://github.com/pre-commit/pre-commit/blob/v4.7.0/pre_commit/main.py#L175-L199)).
- **No marker.** It refuses: `git failed. Is it installed, and are you in a Git repository directory?`, exit 1 [run, §1].
- **Flags.** git's variables. `-c` names the settings file only.
- **The path.** From `p/a`, a hook runs in `p`, and gets `a/m.py` [run, §2].

### 1.9 cargo

- **Markers.** The package: the nearest `Cargo.toml`. The workspace: a `Cargo.toml` above it with `[workspace]` that lists the package, or the one that the package's `package.workspace` names ([`workspaces.md` L116-L121](https://github.com/rust-lang/cargo/blob/0.100.0/doc/book/src/reference/workspaces.md?plain=1#L116-L121)).
- **Order.** The walk for the package starts at the working folder and goes to the root of the file system ([`important_paths.rs` L6-L33](https://github.com/rust-lang/cargo/blob/0.100.0/src/util/important_paths.rs#L6-L33)). The walk for the workspace starts above the package. It stops at `target/package`, and does not cross `CARGO_HOME` ([`workspace.rs` L2290-L2310](https://github.com/rust-lang/cargo/blob/0.100.0/src/workspace/workspace.rs#L2290-L2310)). From `w/crates/c/src`, the package is `w/crates/c` and the workspace is `w` [run, §2].
- **No marker.** It refuses: ``could not find `Cargo.toml` in `<dir>/none` or any parent directory``, exit 101 [run, §1]. A `cargo.toml` in lower case adds a hint to rename it ([`important_paths.rs` L21-L26](https://github.com/rust-lang/cargo/blob/0.100.0/src/util/important_paths.rs#L21-L26)).
- **Flags.** `--manifest-path` [run, §3]. The settings files `.cargo/config.toml` are found from the working folder, not from the manifest (#1212, [L120](https://github.com/kalonji-tools/purba/blob/5cdfe06d4258fdcc4bd27fa9de8c8e1da481c4b8/research/configuration-levels.md?plain=1#L120)).
- **The id.** See 1.10.

### 1.10 cargo-nextest

- **Markers, order, fallback, flags.** cargo's (1.9). nextest reads the workspace's `.config/nextest.toml` from any folder (#1212 [run], A.2).
- **The id.** `<binary id> <path of the test in Rust>`: `c tests::adds` and `c::it lends`, from `w` and from `w/crates/c/src` [run, §4].

### 1.11 git

- **Markers.** In each folder, in this order: `.git` as a file that holds `gitdir: <path>`, `.git` as a folder, the folder itself as a bare repository ([`setup.c` L1590-L1600](https://github.com/git/git/blob/v2.54.0/setup.c#L1590-L1600)).
- **Order.** The walk starts at the working folder. It stops at a folder in `GIT_CEILING_DIRECTORIES`, and at a change of file system unless `GIT_DISCOVERY_ACROSS_FILESYSTEM` is true ([`setup.c` L1547-L1702](https://github.com/git/git/blob/v2.54.0/setup.c#L1547-L1702)). The nearest repository wins: `p/a/inner` [run, §8].
- **No marker.** It refuses: `fatal: not a git repository (or any of the parent directories): .git`, exit 128 [run, §1]. A ceiling at `p` gives the same refusal from `p/a` [run, §4].
- **Flags.** `-C <path>` [run, §7]. `--git-dir` and `GIT_DIR` skip the walk. With no `--work-tree`, `GIT_WORK_TREE` or `core.worktree`, "the current working directory is regarded as the top level" ([`core.adoc` L337-L339](https://github.com/git/git/blob/v2.54.0/Documentation/config/core.adoc?plain=1#L337-L339)). So `GIT_DIR=p/.git` from `p/a` gives the top level `p/a` [run, §5], and `GIT_WORK_TREE=p` gives `p` back [run, §6]. The variables are in [`git.adoc` L537-L575](https://github.com/git/git/blob/v2.54.0/Documentation/git.adoc?plain=1#L537-L575).
- **The path.** `git ls-files` prints `a/m.txt` from `p`, and `m.txt` from `p/a`. `--full-name` prints `a/m.txt` from both [run, §3].

### 1.12 Go

- **Markers.** The main module: the nearest `go.mod` ([`init.go` L1689-L1706](https://github.com/golang/go/blob/go1.26.10/src/cmd/go/internal/modload/init.go#L1689-L1706)). The workspace: the nearest `go.work`, which takes over the module ([`init.go` L1709-L1734](https://github.com/golang/go/blob/go1.26.10/src/cmd/go/internal/modload/init.go#L1709-L1734)).
- **Order.** `GOWORK`, or the walk for `go.work`, which does not cross `GOROOT`. Then the walk for `go.mod`. Each starts at the working folder and goes to the root of the file system. `<dir>/go.work` above `p` is found from `p/a` [run, §5].
- **No marker.** It refuses: `go: go.mod file not found in current directory or any parent directory; see 'go help modules'` [run, §1]. If a folder above holds `.git/config`, the refusal names it instead, with a command to make a module there [run, §2]; [`init.go` L725-L743](https://github.com/golang/go/blob/go1.26.10/src/cmd/go/internal/modload/init.go#L725-L743). `go env GOMOD` prints `/dev/null` and does not refuse [run, §1].
- **Flags.** `-C <dir>` [run, §6]. `GOWORK=off` or a path [run, §5]; [`init.go` L360-L377](https://github.com/golang/go/blob/go1.26.10/src/cmd/go/internal/modload/init.go#L360-L377). `-modfile` reads another file, but a `go.mod` must still mark the root ([`alldocs.go` L206-L210](https://github.com/golang/go/blob/go1.26.10/src/cmd/go/alldocs.go#L206-L210)).
- **The id.** A package is its import path, `example.com/m/a`, from `p` and from `p/a`. `go test -v` names the test `TestLend` and the package `example.com/m/a` from both [run, §3-§4].

### 1.13 Jujutsu

- **Markers.** A folder `.jj` ([`cli_util.rs` L2739-L2743](https://github.com/jj-vcs/jj/blob/v0.41.0/cli/src/cli_util.rs#L2739-L2743)).
- **Order.** The walk starts at the working folder and goes to the root of the file system. `.git` is no marker.
- **No marker.** It refuses: `There is no jj repo in "."`, exit 1 [run, §1]. When the working folder itself holds `.git`, a hint names `jj git init`. From `p/a`, below `p/.git`, there is no hint, because the walk falls back to the working folder before the check ([`cli_util.rs` L2740-L2760](https://github.com/jj-vcs/jj/blob/v0.41.0/cli/src/cli_util.rs#L2740-L2760)) [run, §2].
- **Flags.** `-R <path>` [run, §4]; [`cli_util.rs` L3580-L3585](https://github.com/jj-vcs/jj/blob/v0.41.0/cli/src/cli_util.rs#L3580-L3585).
- **The path.** `jj file list` prints `a/m.txt` from `p`, and `m.txt` from `p/a` [run, §5].

### 1.14 Jest, `rootDir`

- **Markers.** In each folder: `jest.config.{js,ts,mjs,mts,cjs,cts,json}`, or a `package.json`. A `package.json` ends the walk, with or without a `"jest"` key ([`resolveConfigPath.ts` L67-L128](https://github.com/jestjs/jest/blob/v30.5.2/packages/jest-config/src/resolveConfigPath.ts#L67-L128)).
- **Order.** The walk starts at the working folder and goes to the root of the file system. `p/package.json` with no `"jest"` key makes `rootDir` `p`, from `p` and from `p/a` [run, §2].
- **No marker.** It refuses: `Could not find a config file based on provided values`, exit 1 [run, §1]; [`resolveConfigPath.ts` L115-L119](https://github.com/jestjs/jest/blob/v30.5.2/packages/jest-config/src/resolveConfigPath.ts#L115-L119). The documentation says that the default is "the pwd if no package.json is found" ([`Configuration.md` L1590](https://github.com/jestjs/jest/blob/v30.5.2/docs/Configuration.md?plain=1#L1590)). The run and the code disagree with it. #1212 saw the same refusal ([L121](https://github.com/kalonji-tools/purba/blob/5cdfe06d4258fdcc4bd27fa9de8c8e1da481c4b8/research/configuration-levels.md?plain=1#L121)).
- **Flags.** `--rootDir` sets the root once a file is found [run, §3]. With no file, `--rootDir .` still refuses [run, §1b]. `--config '{}'`, the settings as a JSON string, skips the walk, and `rootDir` is the working folder [run, §1c]; [`index.ts` L352-L359](https://github.com/jestjs/jest/blob/v30.5.2/packages/jest-config/src/index.ts#L352-L359).
- **The path.** The `FAIL` line names the file relative to the working folder, not to `rootDir`: `a/x.test.js` from `p`, `./x.test.js` from `p/a`, with `rootDir` `p` in both [run, §2].

### 1.15 Vitest, `root`

- **Markers.** None. The root is the working folder, or `--root`. Vitest looks for `vitest.config.*`, then `vite.config.*`, in the root only ([`resolveConfig.ts` L1168-L1172](https://github.com/vitest-dev/vitest/blob/v5.1.1/packages/vitest/src/node/config/resolveConfig.ts#L1168-L1172); #1212 A.3).
- **Order.** No walk. From `p/a`, `p/vitest.config.mjs` is not read [run, §2].
- **No marker.** The working folder. It runs with no `package.json` and no settings file [run, §1]. No refusal.
- **Flags.** `--root` [run, §3].
- **The id.** `vitest list` prints `a/x.test.js > lends` from `p`, and `x.test.js > lends` from `p/a` [run, §1]. With `--root ..` from `p/a`, it prints `[fromconfig] a/x.test.js > lends` [run, §3].

### 1.16 EditorConfig

- **Markers.** Each `.editorconfig` from the file's folder up. `root = true` in one of them ends the walk ([`handler.py` L53-L85](https://github.com/editorconfig/editorconfig-core-py/blob/v0.17.1/editorconfig/handler.py#L53-L85)).
- **Order.** The walk starts at the folder of each file, not at the working folder. It merges every file it reads, the nearer first [run, §2-§3].
- **No marker.** No property, exit 0 [run, §1]. No refusal.
- **Flags.** `-f <name>` names another file name [run, §4]. The CLI takes absolute paths only ([`handler.py` L91-L93](https://github.com/editorconfig/editorconfig-core-py/blob/v0.17.1/editorconfig/handler.py#L91-L93)).
- **The path.** The globs of a section match from the folder of their own `.editorconfig`. EditorConfig prints no id.

### 1.17 npm, the local prefix

npm is added because it is the one tool here whose marker can be a folder of dependencies.

- **Markers.** A `package.json` or a folder `node_modules`. Then, above that, a `package.json` whose `workspaces` lists it ([`index.js` L696-L763](https://github.com/npm/cli/blob/v11.19.1/workspaces/config/lib/index.js#L696-L763)).
- **Order.** The walk starts at the working folder. The nearest of either marker wins: `p/a/node_modules` beats `p/package.json` [run, §3]. A workspace root above takes over [run, §4].
- **No marker.** The working folder [run, §1]. No refusal.
- **Flags.** `--prefix` [run, §5].
- **The path.** Not measured.

### 1.18 Projectile

Projectile is added because it is the one tool found with an ordered list of ways and a toggle for the fallback. It is an Emacs package, so I did not run it.

- **Markers.** `.projectile`; then the folders of version control `.git`, `.hg`, `.fslckout`, `_FOSSIL_`, `.bzr`, `_darcs`, `.pijul`, `.sl`, `.jj`; then `GTAGS`, `TAGS`, `configure.ac`, `configure.in`, `cscope.out`; then `.svn`, `CVS`, `.osc`, `Makefile` ([`projectile.el` L559-L612](https://github.com/bbatsov/projectile/blob/v3.4.0/projectile.el#L559-L612)).
- **Order.** `projectile-project-root-functions`: local, marked, bottom-up, top-down, top-down-recurring. "The functions will be run until one of them returns a project folder. Reordering the default functions will alter the project discovery algorithm." ([`projectile.el` L614-L626](https://github.com/bbatsov/projectile/blob/v3.4.0/projectile.el#L614-L626)). Its documentation says why version control comes before a manifest: "the git repository is the project" ([L589-L592](https://github.com/bbatsov/projectile/blob/v3.4.0/projectile.el#L589-L592)).
- **No marker.** `projectile-require-project-root`: `t` refuses with `Projectile cannot find a project definition in <dir>`, `nil` takes the working folder, `prompt` asks for a known project. The default is `prompt` ([`projectile.el` L406-L416](https://github.com/bbatsov/projectile/blob/v3.4.0/projectile.el#L406-L416), [L3110-L3124](https://github.com/bbatsov/projectile/blob/v3.4.0/projectile.el#L3110-L3124)).
- **Flags.** None; the user's settings set each value.
- **The path.** Not measured.

## 2. The ways, merged

Each way appears once. A tool appears under each way it uses.

### 2.1 Where a walk starts

| start | tools |
|---|---|
| the working folder | Ruff, mypy, uv, tox, cargo, git, Go, Jujutsu, Jest, npm, pre-commit (through git), purba ([L67](https://github.com/kalonji-tools/purba/blob/cfc09617/docs/decisions/purba-reads-configuration-from-six-levels.md?plain=1#L67)) |
| the common folder of the arguments | pytest, Black |
| each file's own folder | Ruff (for each file's settings), EditorConfig, mypy (for a module's name) |
| no walk: the working folder is the root | coverage.py, Vitest |

### 2.2 What marks the root

| way | example | tools |
|---|---|---|
| a file that belongs to the tool | `pytest.ini`, `.ruff.toml`, `tox.ini`, `jest.config.js`, `.projectile`, `.editorconfig` with `root = true` | pytest, Ruff, tox, Jest, Projectile, EditorConfig |
| the tool's table in a shared file | `[tool.pytest]`, `[tool.ruff]`, `[tool.black]`, a `"jest"` key | pytest, Ruff, Black, tox, Jest, purba |
| a manifest of the language, whatever it holds | `pyproject.toml`, `setup.py`, `Cargo.toml`, `go.mod`, `package.json`, `Makefile` | pytest (as a late step), tox (as a fallback), uv, cargo, Go, npm, Jest, Projectile |
| a manifest that must hold a table | `pyproject.toml` with `[project]` | uv (refuses without it) |
| a folder of version control | `.git`, `.hg`, `.jj` | git, Jujutsu, Black, pre-commit, Projectile |
| a folder of dependencies | `node_modules` | npm |
| a workspace above the first root, found by a second walk | `[workspace]`, `[tool.uv.workspace]`, `go.work`, `"workspaces"` | cargo, uv, Go, npm |
| the chain of package files of the language | `__init__.py` in each folder | mypy |
| a flag or a variable that names the root | `--rootdir`, `--root`, `--project`, `--manifest-path`, `-C`, `-R`, `GIT_WORK_TREE`, `--rootDir`, `--prefix` | pytest, tox, uv, cargo, git, Go, Jujutsu, Jest, Vitest, npm |
| a flag that names the settings file, whose folder becomes the root | `-c <file>` | pytest, tox |
| a flag that names the settings file, and leaves the working folder as the root | `--config <file>`, `GIT_DIR` with no work tree | Ruff, git |

### 2.3 What stops a walk

| stop | tools |
|---|---|
| the first match of any marker, in a fixed order per folder | pytest, Ruff, Black, tox, uv, cargo, git, Go, Jujutsu, Jest, npm |
| a folder of version control ends the walk for a file, but is not the root | mypy (`.git`, `.hg`), purba (`.git`, [L67](https://github.com/kalonji-tools/purba/blob/cfc09617/docs/decisions/purba-reads-configuration-from-six-levels.md?plain=1#L67)) |
| a shared file with no table for the tool | uv (refuses), Jest (becomes the root) |
| a value in a file: `root = true` | EditorConfig |
| a list of ceilings, or a change of file system | git (`GIT_CEILING_DIRECTORIES`, `GIT_DISCOVERY_ACROSS_FILESYSTEM`) |
| a folder of the tool's own install | cargo (`CARGO_HOME`, for the workspace), Go (`GOROOT`, for `go.work`), uv (its cache, for the workspace) |
| the root of the file system | each walk above that finds no stop |

### 2.4 What a tool does when nothing marks the root

| fallback | tools |
|---|---|
| the working folder | Ruff, tox (after a bare `pyproject.toml`), coverage.py, Vitest, npm, git (with `GIT_DIR` and no work tree), Projectile (`nil`) |
| the common folder of the working folder and the arguments | pytest |
| the folder of the nearest shared manifest, whatever it holds | pytest (`pyproject.toml`), tox (`pyproject.toml`) |
| the root of the file system | Black |
| a refusal | cargo, Go, git, Jujutsu, pre-commit, Jest, uv (project commands), Projectile (`t`) |
| a refusal that names a marker it did find | Go (`.git/config`), Jujutsu (`.git` in the working folder) |
| a prompt | Projectile (`prompt`, its default) |
| a run with no root | uv (`uv run`), mypy and EditorConfig (they read no file) |

### 2.5 Whether a printed id or path depends on the working folder

| answer | tools |
|---|---|
| no: the id comes from the tree | cargo-nextest, Go, mypy with `__init__.py`, pytest with a marker, tox with a marker |
| yes, under the fallback only | pytest, tox, mypy under `--explicit-package-bases` |
| yes, always: paths print relative to the working folder | Ruff, coverage.py, Jest, Vitest, git (`ls-files` without `--full-name`), Jujutsu |
| no: a hook gets paths from the top level | pre-commit |

## 3. The order that each tool tries

| tool | order, the first that matches wins |
|---|---|
| pytest | `-c` > `--rootdir` > the files of configuration, up from the arguments' folder > `setup.py`, up > the files, up from each argument > a bare `pyproject.toml` > the common folder of the working folder and the arguments |
| Ruff | `--isolated` > `--config` (root: the working folder) > the nearest settings file > the user's file (root: the working folder) > the working folder |
| mypy, the file | `--config-file` > the walk, which stops at `.git` or `.hg` > the user's files > the defaults |
| mypy, a module's name | an explicit base > the highest folder with `__init__.py` > the file alone |
| Black | in each folder from the arguments' folder up: `.git` > `.hg` > `pyproject.toml` with `[tool.black]`; then `/` |
| uv | `--directory` > `--project` or `UV_PROJECT` > the nearest `pyproject.toml` > a workspace above it; else a refusal |
| coverage.py | `--rcfile` or `COVERAGE_RCFILE` > the five names in the working folder |
| tox | `-c` > the walk for four files > `--root`, or the nearest `pyproject.toml`, or the working folder |
| pre-commit | git's order |
| cargo | `--manifest-path` > the nearest `Cargo.toml` > `package.workspace` > a `[workspace]` above that lists it; else a refusal |
| git | `--git-dir` or `GIT_DIR` > the walk, up to a ceiling or a change of file system; else a refusal |
| Go | `-C` > `GOWORK` > the nearest `go.work` > the nearest `go.mod`; else a refusal, which may name `.git/config` |
| Jujutsu | `-R` > the nearest `.jj`; else a refusal |
| Jest | `--rootDir` > `--config` > the nearest folder with a settings file or a `package.json`; else a refusal |
| Vitest | `--root` > the working folder |
| EditorConfig | each `.editorconfig` up from the file, until `root = true` |
| npm | `--prefix` > the nearest `package.json` or `node_modules` > a workspace above it > the working folder |
| Projectile | local > `.projectile` > version control > the topmost manifest > the recurring markers; then `projectile-require-project-root` |

## 4. Weighing the hierarchy and the toggle against purba's records

### The constraints

- [The walk goes up from the working folder to the first `pyproject.toml` that holds `[tool.purba]`, and stops at the folder that holds `.git`](https://github.com/kalonji-tools/purba/blob/cfc09617/docs/decisions/purba-reads-configuration-from-six-levels.md?plain=1#L67). [Outside git, it goes up to the root of the file system](https://github.com/kalonji-tools/purba/blob/cfc09617/docs/decisions/purba-reads-configuration-from-six-levels.md?plain=1#L92).
- [A flag sets each dial that the project's file does not keep; the environment, each dial that a flag may set; the user's and the system's file, only a dial that changes no outcome](https://github.com/kalonji-tools/purba/blob/cfc09617/docs/decisions/purba-reads-configuration-from-six-levels.md?plain=1#L54-L58).
- [No mode, flag or opt-out turns strictness off](https://github.com/kalonji-tools/purba/blob/cfc09617/docs/decisions/purba-is-always-strict.md?plain=1#L37). [No value of configuration loosens a check](https://github.com/kalonji-tools/purba/blob/cfc09617/docs/decisions/purba-is-always-strict.md?plain=1#L42).
- [`purba` with no query runs each test file under the folder of the project's file](https://github.com/kalonji-tools/purba/blob/cfc09617/docs/decisions/purba-picks-a-test-by-its-name-and-the-tests-of-a-run-by-one-query.md?plain=1#L84). [No flag selects a test](https://github.com/kalonji-tools/purba/blob/cfc09617/docs/decisions/purba-picks-a-test-by-its-name-and-the-tests-of-a-run-by-one-query.md?plain=1#L86). [The path of an id starts at that folder](https://github.com/kalonji-tools/purba/blob/cfc09617/docs/decisions/purba-picks-a-test-by-its-name-and-the-tests-of-a-run-by-one-query.md?plain=1#L99).
- [One identity: no part names a test except by the identity that selection gives it](https://github.com/kalonji-tools/purba/blob/cfc09617/LANDSCAPE.md?plain=1#L144).
- A [strict check](https://github.com/kalonji-tools/purba/blob/cfc09617/CONTEXT.md?plain=1#L335-L337) judges one of a tester's tests. A [dial](https://github.com/kalonji-tools/purba/blob/cfc09617/CONTEXT.md?plain=1#L97-L99) is a value of configuration that changes how purba behaves.

### 4.1 Which level can hold the toggle when no project's file is found

The toggle acts only when the walk finds no `[tool.purba]`. In that case, the project's level is empty.

| level | may it hold `implicit_root`? | why |
|---|---|---|
| code | no | judgement: code sits beside a test, and no test is found before the root |
| the command line | the record allows it, as a dial that the project's file does not keep | judgement: a root moves the set of files that `purba` with no query runs. A flag that moves that set selects tests, which L86 forbids |
| the environment | as the command line | the same; a variable in a shell's start-up file also drifts like the user's file ([L87](https://github.com/kalonji-tools/purba/blob/cfc09617/docs/decisions/purba-reads-configuration-from-six-levels.md?plain=1#L87)) |
| the project | never reached | judgement: when `[tool.purba]` exists, it already marks the root, so the toggle has no case to act on. pytest refuses `--rootdir` in `addopts` for the same reason: "the rootdir is used to find the configuration file" (#1212 [L227](https://github.com/kalonji-tools/purba/blob/5cdfe06d4258fdcc4bd27fa9de8c8e1da481c4b8/research/configuration-levels.md?plain=1#L227)) |
| the user's file, the system's file | no | judgement: a root changes which tests run and the id of each, so it changes an outcome. L63 makes such a key an error there |

judgement: so only a flag or a variable can hold the toggle, and each meets L86. In the survey, the tools that refuse put the escape on the command line as a flag that names the root, and not as a toggle: cargo's `--manifest-path`, git's `-C`, Go's `-C`, Jujutsu's `-R`, uv's `--project` (section 1).

### 4.2 Whether the toggle is a switch that "purba is always strict" forbids

Two readings follow from the records. Neither is decided here.

- **It is not a switch of strictness.** A strict check judges one of a tester's tests. The refusal for a missing root judges the invocation, before any test exists. So `implicit_root = true` turns no strict check off. judgement.
- **It is a mode, which L37 names.** The toggle has two states, and each state changes what purba does with the same tree. It is not a fact about the project: it says how to look for the root, not where the root is. L42 lets a value that names a fact pass, and [L60 of the configuration record](https://github.com/kalonji-tools/purba/blob/cfc09617/docs/decisions/purba-reads-configuration-from-six-levels.md?plain=1#L60) refuses a key that loosens a check. judgement: the refusal is a check on the run, so a value that turns it into a fallback loosens it.

The survey offers a form that states a fact instead of a mode. Each of these names the root, and none chooses between a fallback and a refusal:

```sh
pytest --rootdir=p       # pytest
tox --root p             # tox
cargo --manifest-path p/Cargo.toml locate-project
jj -R p root             # Jujutsu
uv --project p workspace dir
```

judgement: a flag of this form is still a flag that moves the set that `purba` with no query runs, so it meets L86 as 4.1 says.

### 4.3 Whether one test keeps one id under each way

Within one run, each way gives each test one id, so the invariant one identity holds in each run. The question is whether the id holds across runs from other folders. `.purba/failed.q` carries ids from one run to the next ([L72-L73](https://github.com/kalonji-tools/purba/blob/cfc09617/docs/decisions/purba-picks-a-test-by-its-name-and-the-tests-of-a-run-by-one-query.md?plain=1#L72-L73)), and a term that names nothing is an invocation error ([L101](https://github.com/kalonji-tools/purba/blob/cfc09617/docs/decisions/purba-picks-a-test-by-its-name-and-the-tests-of-a-run-by-one-query.md?plain=1#L101)).

| way, as a step of the hierarchy | one id from any folder of the project? | evidence |
|---|---|---|
| `[tool.purba]` in `pyproject.toml` | yes | pytest with `[tool.pytest]`: `a/test_a.py::test_a` from `p` and `p/a` [run] |
| a bare `pyproject.toml` | yes, until a nearer `pyproject.toml` appears or `[tool.purba]` is added above it | pytest with a bare `pyproject.toml`: rootdir `p` from `p/a` [run]. judgement: adding `[tool.purba]` at another folder moves the root, and each id |
| the folder that holds `.git` | yes in one checkout. judgement: an archive of the source with no `.git`, or a nested repository, gives another root | git: the nearest repository wins, so `p/a/inner/.git` moves the top level from `p` to `p/a/inner` [run] |
| the common folder of the arguments | no | pytest: `p/a/test_a.py::test_a` from `elsewhere`, `a/test_a.py::test_a` from `p/a` with `. ../b` [run] |
| the working folder | no | pytest: `test_a.py::test_a` from `p/a`, `a/test_a.py::test_a` from `p` [run]; Vitest: `x.test.js > lends` against `a/x.test.js > lends` [run] |
| the root of the file system | judgement: yes, but each id holds the path from `/` | Black's root is `/` [run]. judgement: `purba` with no query would then walk the whole file system, as [L92](https://github.com/kalonji-tools/purba/blob/cfc09617/docs/decisions/purba-reads-configuration-from-six-levels.md?plain=1#L92) already warns for the walk to the file |

judgement: a hierarchy whose last step is the working folder or the arguments gives one test several ids across runs. A hierarchy that stops at a step read from the tree keeps one id, until the tree changes which step matches first.

## 5. What this file does not measure

- **purba's own behaviour.** No probe ran a purba wheel. Each line about what purba would print is a judgement from the records.
- **Projectile.** It is [source] only. I did not run Emacs.
- **Black's and npm's printed paths.** The probes print only the root.
- **macOS and Windows.** Each probe ran on Linux. Black returns no root for sources on two drives ([`files.py` L59-L61](https://github.com/psf/black/blob/26.10.1/src/black/files.py#L59-L61)); that case is unmeasured.
- **A `.git` file of a worktree or a submodule.** git reads it [source]; no probe made one.
- **Mercurial, Bazel, Pants and Hatch** are not surveyed.

## Files

| path | what it holds |
|---|---|
| `research/project-root/{pytest,ruff,mypy,black,coverage,tox,precommit,editorconfig,uv,cargo,git,go,jj,jest,vitest,npm}/run.sh`, `out.txt` | each probe, and its output in this session |
