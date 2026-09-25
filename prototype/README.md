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
