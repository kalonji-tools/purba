# Prototype for #42

Six probes. Each one can falsify a decision the grilling reached.
This branch is never merged. It is evidence.

| # | probe | venue | answer |
|---|---|---|---|
| A | mise supplies a free-threaded interpreter | local | **yes, but not by a version string** |
| B | one mise config and one lock hold two interpreters | local | **yes for the versions. No for the flavor** |
| C | `mise run test:rust` passes on macOS and Windows | CI | see `proto-42.yml` |
| D | the floor `abi3` wheel installs and imports above the floor | CI | see `proto-42.yml` |
| E | a free-threaded build drops `abi3` and writes a version-specific wheel | CI | see `proto-42.yml` |
| F | a rollup that reads `needs.*.result` refuses a skipped arm | CI | see `proto-42.yml` |

## A — mise supplies a free-threaded interpreter

**The grilling assumed a version string `3.14t`. There is no such version.**

`mise ls-remote python` offers 223 versions and **none** carries a `t` suffix.
`mise ls-remote python@3.14t` returns nothing.

Free-threading is a **flavor of the asset**, not a version. It is set by
`MISE_PYTHON_PRECOMPILED_FLAVOR`, or by the `python.precompiled_flavor` option.

⚠️ **The flavor in the mise documentation does not exist.** The documented example is
`freethreaded+pgo-full`. That asset is not published. mise refuses it:

```
mise ERROR Failed to install core:python@3.14: no precompiled python found for
core:python@3.14.7 on x86_64-unknown-linux-gnu-freethreaded+pgo-full
```

The published flavors for `3.14.7` on `x86_64-unknown-linux-gnu`, read from the
`astral-sh/python-build-standalone` release `20260924`:

| flavor | note |
|---|---|
| `freethreaded-install_only_stripped` | matches the flavor `mise.lock` already uses |
| `freethreaded-install_only` | |
| `freethreaded+pgo+lto-full` | |
| `freethreaded+debug-full` | |

With the correct flavor the install succeeds and the interpreter is genuinely
free-threaded:

```
version         : 3.14.7
Py_GIL_DISABLED : 1
abiflags        : 't'
gil enabled     : False
EXT_SUFFIX      : .cpython-314t-x86_64-linux-gnu.so
```

⚠️ **`python.precompiled_flavor` is absent from `mise settings --all`.** The setting is
read: `mise settings get python.precompiled_flavor` returns the value. An audit of
the settings list concludes the setting does not exist.

⚠️ **The `EXT_SUFFIX` is `.cpython-314t-`, not `abi3`.** This is the first direct
evidence for the prediction in #63, which was read from `pyo3-build-config` source
and never run. Probe E tests the wheel itself.

## B — one mise config and one lock hold two interpreters

**The grilling said a Python matrix cannot come out of mise, because `mise.lock`
pins `3.12.14` alone. That is false.** `mise.lock` pins one version because
`mise.toml` names one. A list works, and the lock records every entry in full:
7 platform blocks, a checksum and a provenance row each. See `probe-b/mise.lock`.

**So the versions are lockable. The flavor is not.**

`probe-b/mise.toml` declares the flavor per entry:

```toml
python = [
  "3.12",
  { version = "3.14", precompiled_flavor = "freethreaded-install_only_stripped" },
]
```

⚠️ **mise ignores that option and reports success.** On a clean store it extracts
the GIL build:

```
extract cpython-3.14.7+20260924-x86_64-unknown-linux-gnu-install_only_stripped.tar.gz
```

and the interpreter reports `GIL`, `.cpython-314-x86_64-linux-gnu.so`.

The same command with the **environment variable** set extracts the right asset:

```
extract cpython-3.14.7+20260924-x86_64-unknown-linux-gnu-freethreaded-install_only_stripped.tar.gz
```

⚠️ **The generated lock records no flavor at all.** Both entries carry the GIL
`install_only_stripped` URLs. `mise install --locked` from that lock installs a
GIL interpreter. The free-threaded arm cannot be locked.

### The first reading of probe B was wrong, and the control caught it

Before the clean-store control, probe B reported that `3.14` served a
free-threaded interpreter. That reading was contamination.

⚠️ **The mise store is not keyed by the flavor.** `~/.local/share/mise/installs/python/`
holds one `3.14.7` directory. Probe A had already put a free-threaded build there.
mise found it, said "already installed", and served it. A GIL `3.14.7` and a
free-threaded `3.14.7` cannot coexist, and the second one requested is silently
served the first one's build.

### Two further facts this probe exposed

⚠️ **`mise lock` moves the build date.** purba's committed lock names
`python-build-standalone` release `20260901`. A regenerated lock names `20260924`
for the same `3.12.14`. The lock is not stable across regeneration.

⚠️ **One checksum is `blake3:` where every other is `sha256:`** — `3.14.7`,
`platforms.linux-x64`. One lock carries two checksum algorithms.

## What A and B change

| grilling decision | status |
|---|---|
| 7 — mise supplies an arm that builds | **holds for a GIL arm. Fails for the free-threaded arm**, whose interpreter no lock can record |
| the free-threaded arm is named `3.14t` | **false.** It is `3.14` plus a flavor |

---

# The CI probes — run 36112711839

| # | probe | verdict |
|---|---|---|
| C | `mise run test:rust` on three operating systems | ✅ **pass on all three** |
| D | the floor `abi3` wheel installs and imports above the floor | ✅ **the claim holds.** ⚠️ The first assertion was wrong |
| E | a free-threaded build drops `abi3` | ❌ **the probe measured nothing.** Rewritten and re-run |
| F | a rollup refuses an arm that did not succeed | ✅ answered, and the hazard is **not** the one that was designed for |

## C — the Rust tests pass off Linux

| runner | result |
|---|---|
| `ubuntu-latest` | success |
| `macos-14` | success |
| `windows-latest` | success |

`tasks.toml` carries a macOS `DYLD_FALLBACK_LIBRARY_PATH` workaround and a note that
Windows runs a task through cmd. Both hold. One Rust job across three operating
systems is the right shape, and the grilling's guess was correct for a reason it
did not have.

## D — the abi3 claim holds, and the first assertion was wrong

**Every one of the six arms installed the wheel.** `Successfully installed purba-0.0.0`
on 3.13 and 3.14, on Linux, macOS and Windows, from a wheel tagged
`cp312-abi3-manylinux_2_34_x86_64` built on the floor.

So `requires-python = ">=3.12"` without an upper bound is a true claim, and this is
the first evidence for it.

⚠️ **All six arms then failed, on the assertion rather than on the claim.**

```
AssertionError: the module is not the extension:
  .../site-packages/purba/__init__.py
```

**maturin writes a package wrapper into the wheel.** The wheel holds:

| path | bytes |
|---|---|
| `purba/__init__.py` | 103 |
| `purba/purba.abi3.so` | 7871608 |

and the wrapper is `from .purba import *`.

The record on the mise substrate names this trap and says a check written on the
package *"passes for a package that holds no Rust"*. Here it did the opposite: it
failed for a package that does hold Rust. Both readings are wrong the same way.

⚠️ **The grilling said this trap was "honest now and rots later", because no `.py`
file exists in the source tree. That was false.** maturin generates the wrapper at
build time, so the trap is live from the first wheel. Reading the source tree is
not reading the distribution.

## E — the probe measured nothing, and reported success

Both arms built against a **GIL 3.12**:

| arm | interpreter it actually got | should have been |
|---|---|---|
| `mise-flavor` | 3.12.3, `Py_GIL_DISABLED: None` — the runner's own python | free-threaded 3.14 |
| `setup-python` | 3.12.14, `Py_GIL_DISABLED: None` — mise's python | free-threaded 3.14 |

Two faults, and `continue-on-error` on every step hid both:

1. `jdx/mise-action` ran **after** `actions/setup-python` and won the PATH.
2. The reporting step called bare `python` instead of going through mise.

⚠️ **A probe that cannot refuse measures nothing.** The rewrite asserts the
interpreter is free-threaded before it builds, and carries `continue-on-error` only
where a refusal is a legitimate result.

## F — the hazard is `continue-on-error`, not a skipped arm

**F1. A tolerated failure is invisible to a rollup.**

| job | conclusion |
|---|---|
| `F1 arm 1` | success |
| `F1 arm 2` | **failure**, with `continue-on-error: true` |
| `F1 arm 3` | success |

and the rollup read:

```
needs.f1-matrix.result = success
```

⚠️ **So reading `needs.*.result` does NOT catch an arm that failed under
`continue-on-error`.** The grilling decided the rollup reads the result, and that is
necessary and not sufficient. **The gated matrix must carry no `continue-on-error`
at all**, and nothing in a rollup can compensate for one.

**F2. A skipped job deadlocks a naive rollup instead of refusing.**

| job | conclusion |
|---|---|
| `F2 the job that is skipped` | skipped |
| `F2 rollup that trusts needs alone` | **skipped** |
| `F2 rollup that reads the result` | **failure** |

A rollup that only lists `needs` is itself skipped, so it **never reports**. The
`quality.yml` comment already states what that costs: a check that never reports and
a check that refuses are the same thing to the ruleset. The difference is that a
refusal names a reason and a silence does not.

⚠️ **`if: always()` is what converts the deadlock into a refusal.** It is not
optional decoration on the rollup; without it the gate blocks forever and says
nothing.

**The hazard the grilling designed against cannot arise.** actionlint refuses a
job-level `if` that reads `matrix`: the allowed contexts are `github`, `inputs`,
`needs` and `vars`. So a matrix arm cannot be skipped by a condition.

## Two measurement traps found while reading the results

- ⚠️ **`gh run view --log` refuses while a run is in progress**, even for jobs that
  have finished: *"logs will be available when it is complete"*. The per-job REST
  endpoint serves them immediately.
- ⚠️ **`gh api .../jobs/<id>/logs` returns nothing and exits 0** without
  `--allow-escape-sequences`. It prints a notice to stderr. A pipeline that reads
  stdout sees an empty log and no error.

---

# The re-run — 36113297961

| # | probe | verdict |
|---|---|---|
| D | the floor `abi3` wheel installs and imports above the floor | ✅ **6 of 6 arms pass** |
| E | a free-threaded build drops `abi3` | ✅ **answered, and the two sources disagree** |

## D — six of six

With the assertion reading `purba.purba` rather than `purba`, every arm passes:
3.13 and 3.14, on `ubuntu-latest`, `macos-14` and `windows-latest`.

The claim `requires-python = ">=3.12"` with no upper bound now has evidence.

## E — the prediction is confirmed, and only one source can reach it

`actions/setup-python` with `3.14t` supplies a genuinely free-threaded interpreter:

```
executable     : /opt/hostedtoolcache/Python/3.14.7/x64-freethreaded/bin/python
Py_GIL_DISABLED: 1
abiflags       : 't'
EXT_SUFFIX     : .cpython-314t-x86_64-linux-gnu.so
```

and maturin states what it does with it, unprompted:

```
🔗 Found pyo3 bindings with abi3-py3.12 support
⚠️ Warning: abi3 does not yet support CPython 3.14t at
   /opt/hostedtoolcache/Python/3.14.7/x64-freethreaded/bin/python
   so the build artifacts will be version-specific.
📦 Built wheel for CPython 3.14t to
   target/wheels/purba-0.0.0-cp314-cp314t-manylinux_2_34_x86_64.whl
```

**The wheel is `cp314-cp314t`.** The prediction read out of `pyo3-build-config`
source is confirmed by execution: pyo3 drops `abi3-py312` on a free-threaded
interpreter and writes a version-specific wheel. No `Cargo.toml` change is needed.

⚠️ **The mise arm refuses.** `mise x python@3.14` resolved `/usr/bin/python`, the
runner's own 3.12.3, so the probe refused before building:

```
::error::mise did not supply a free-threaded interpreter, so this arm cannot build.
AssertionError: this interpreter has the GIL, so the arm measured the wrong thing
```

The same command resolves mise's interpreter on a machine whose store already holds
it. **The warm store is why this looks like it works locally.**

**So the free-threaded arm takes its interpreter from `actions/setup-python`.** It is
the only source measured to work, and no lockfile records it either way.

---

# Two findings about the substrate, not about this gate

Both were found while reading these results. Neither belongs to this gate.

## 1 — `[tool_config] locked = true` is dead configuration

`mise.toml` carries:

```toml
[tool_config]
locked = true
```

and a comment describing what it enforces. **mise does not read it.** Two arms, one
difference, on mise 2026.8.6:

| `mise.toml` says | `mise settings --all` reports |
|---|---|
| `[settings]`<br>`locked = true` | `locked true`, sourced from that file |
| `[tool_config]`<br>`locked = true` | `locked false` |

purba's own worktree reports `locked false`. The section is silently ignored, and
`mise settings get tool_config.locked` answers `Unknown setting`.

⚠️ **The enforcement does exist in CI, and it comes from somewhere else.**
`jdx/mise-action@v4` runs `mise install --locked` — the flag, not the file. So CI is
locked and a contributor's machine is not, and no location says so.

## 2 — `mise.lock` does not pin the Rust toolchain

`mise.toml` asks for `rust = "nightly"`. `mise.lock` records
`version = "nightly-2026-09-22"`.

Two runs of this workflow, on this branch, with `mise.lock` **byte-identical**
between the two commits:

| run | started | `mise install --locked` installed |
|---|---|---|
| 36112711839 | 08:24 | `rust@nightly-2026-09-22` |
| 36113297961 | 08:30 | **`rust@nightly-2026-09-25`** |

Six minutes apart, same tree, different compiler.

⚠️ **The record states the opposite.** `mise-names-every-tool-version.md` says the
lockfile pins the toolchain by date and not by checksum, *"so the version is
reproducible and the download is not verified."* The version is **not** reproducible.

That record chose mise over devenv because the two disagreed on `nightly` by four
days. This measurement shows mise disagrees with **itself** across six minutes.

### ⚠️ The first explanation of this was wrong, and the logs name the real one

This file first said the mise-action cache was the likely reason the drift is rarely
visible. **That is backwards. The cache is what causes the drift.**

Both runs ran `mise install --locked`. They differ in one line before it.

**Run 36112711839, no cache restored:**

```
Detected a mise lock file, running `mise install --locked`
mise rust@nightly-2026-09-22 info: syncing channel updates for nightly-2026-09-22-...
mise rust@nightly-2026-09-22   installed - rustc 1.100.0-nightly (1303417c4 2026-09-21)
```

It goes straight to the locked date. There is no resolution step.

**Run 36113297961, cache restored:**

```
mise cache restored from key: mise-v1-linux-x64-ubuntu24-abf677e131...
Detected a mise lock file, running `mise install --locked`
  rust@nightly  resolving  3.0s
  rust@nightly  resolving  6.0s
mise rust@nightly-2026-09-25 info: syncing channel updates for nightly-2026-09-25-...
```

⚠️ **`rust@nightly resolving`.** With the cache restored, mise resolves the floating
name against the live channel instead of reading the locked date.

| run | cache | what mise did | toolchain |
|---|---|---|---|
| 36112711839 | none | read the lock | `nightly-2026-09-22`, rustc `1303417c4` |
| 36113297961 | restored | **resolved `rust@nightly`** | `nightly-2026-09-25` |

**Why the cache changes the code path is not explained here.** The observation is
recorded and the mechanism is not, because one explanation has already been wrong.
The other four tools read `already installed` in both runs, so rust is the only entry
that takes this path, and its lock entry is also the only one with no checksum and no
platform rows.

### What this falsifies

Two records, not one.

| record | the sentence |
|---|---|
| `mise-names-every-tool-version.md` | *"the version is reproducible and the download is not verified"* |
| `purba-meets-the-next-trait-solver-before-it-stabilizes.md` | *"A person reads and bumps the floating name, and a machine installs the date"* |

The machine installs the date on a cold runner. On a warm one it installs today.

⚠️ **The second record rejected the obvious repair in advance.** It says naming a date
in both files *"would state one fact twice and let the two copies disagree"*. So
pinning the date in `mise.toml` is not available without changing that decision.

⚠️ **This reaches the gates that already run.** `quality.yml` runs `cargo fmt --check`
and `clippy` and then `git diff --exit-code`, on whichever nightly the cache yields.
A formatting change between two nightlies refuses a pull request for a reason that is
in nobody's diff.
