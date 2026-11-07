# How the report of an internal error can name the ABI flags

This file answers [How does the report of an internal error name the ABI flags, with no read of sys.abiflags before the run?](https://github.com/kalonji-tools/purba/issues/845)
It is never merged.

## Answer

The two flags that remain behave differently.
**`t` (free-threaded) is fixed when the wheel is built.** pip refuses the published `cp312-abi3` wheel on every free-threaded Python, so `t` reaches the report only from a build from source.
**`d` (debug) is not fixed when the wheel is built.** pip installs the same `cp312-abi3` wheel on every debug build measured, and the debug `.so` is byte-identical to the release one.
Two values that a test cannot change from Python name both flags correctly:

- `Py_GetVersion()` holds `free-threading build` on a `t` build. purba already reads it, through `py.version_info()`.
- The exported symbol `_Py_NegativeRefcount` exists only in a debug build. `dlsym` finds it.

A scratch copy of purba built the flags from those two sources, after a test set `sys.abiflags = "x"`.
It printed the same flags as today's read at start on all 8 interpreter and wheel pairs.
A read at report time printed the test's `x`, or `?` after the test deleted `sys.abiflags`.

On the second question, other tools' crash reports name no build flag at all: 0 of the 16 surveyed.
When a bug report learns that a build is free-threaded, a template asks the tester to run a command, and the version string carries it.
No crash report or template in the sample names a debug build.

| question | short answer | tier |
|---|---|---|
| Which values can `sys.abiflags` take, CPython 3.12–3.15 on POSIX? | `''` and `d` on 3.12; `''`, `t`, `d`, `td` from 3.13 | [A] source + M1 |
| Which can a published purba wheel meet? | `''` and `d`. Never `t` | [A] M1, M5, M6 |
| Can purba know `t` without asking Python? | yes: at compile time, or from `Py_GetVersion()` | [A] M9 |
| Can purba know `d` without asking Python? | only at run time, by `dlsym` of a debug-only symbol | [A] M9 on Linux; [C] macOS |
| Does the report need the flags at all? | no surveyed tool prints a build flag in a crash line | [A] S1 |
| Does the zygote change the answer? | no record says which process writes the report | [A] for the records; the rest is reasoning |

## 1. The flags a published wheel can meet

### The values of `sys.abiflags`

| CPython | values | source |
|---|---|---|
| 3.12 | `''`, `d` | [configure.ac v3.12.0 L1674-L1687](https://github.com/python/cpython/blob/v3.12.0/configure.ac#L1674-L1687) [A] |
| 3.13 | `''`, `t`, `d`, `td` | [configure.ac v3.13.0 L1728-L1760](https://github.com/python/cpython/blob/v3.13.0/configure.ac#L1728-L1760) [A] |
| 3.14 | `''`, `t`, `d`, `td` | [configure.ac v3.14.0 L1713-L1745](https://github.com/python/cpython/blob/v3.14.0/configure.ac#L1713-L1745) [A] |
| 3.15 | `''`, `t`, `d`, `td` | [configure.ac v3.15.0 L1754-L1786](https://github.com/python/cpython/blob/v3.15.0/configure.ac#L1754-L1786) [A] |

- `m` left in 3.8: "Default flags became an empty string" ([sys.rst v3.15.2 L59-L70](https://github.com/python/cpython/blob/v3.15.2/Doc/library/sys.rst?plain=1#L59-L70)) [A].
- `u` applies only before 3.3 ([packaging tags.py 26.2 L307-L316](https://github.com/pypa/packaging/blob/26.2/src/packaging/tags.py#L307-L316)) [A].
- Since 3.8 a debug build loads extensions built in release mode and with the stable ABI ([What's New 3.8 L205-L212](https://github.com/python/cpython/blob/v3.15.2/Doc/whatsnew/3.8.rst?plain=1#L205-L212)) [A].

### What pip installs, and what runs

| interpreter | `cp312-abi3` wheel (the published shape) | a wheel built from source on it |
|---|---|---|
| 3.12, 3.13, 3.14, 3.15 | installs | — |
| 3.12d, 3.13d, 3.14d, 3.15d | **installs** | `cp312-abi3`, byte-identical `.so` |
| 3.13t | refused | PyO3 0.29.2 refuses the build |
| 3.14t, 3.15t | refused | `cp314-cp314t`, `cp315-cp315t`, after a maturin warning |
| 3.14td | refused | `cp314-cp314td`; a `cp314-cp314t` wheel installs too |

Measured in M1, M3 and M5 [A]. The reasons, in source:

- packaging gives abi3 only to a build that is not free-threaded: "The free-threaded builds do not support abi3" ([`_abi3_applies`, tags.py 26.2 L269-L276](https://github.com/pypa/packaging/blob/26.2/src/packaging/tags.py#L269-L276)) [A]. M1 read pip's vendored copy.
- A debug build also accepts the tag without `d` ([`_cpython_abis`, L292-L322](https://github.com/pypa/packaging/blob/26.2/src/packaging/tags.py#L292-L322)) [A].
- maturin hands PyO3 empty `build_flags` for a stable-ABI build, so the build cannot see `Py_DEBUG` ([compile.rs v1.15.0 L912-L949](https://github.com/PyO3/maturin/blob/v1.15.0/src/compile.rs#L912-L949); M4) [A].
- PyO3 picks no stable ABI for a free-threaded build below 3.15 ([impl_.rs v0.29.2 L102-L132](https://github.com/PyO3/pyo3/blob/v0.29.2/pyo3-build-config/src/impl_.rs#L102-L132)), and refuses 3.13t ([pyo3-ffi/build.rs v0.29.2 L107-L114](https://github.com/PyO3/pyo3/blob/v0.29.2/pyo3-ffi/build.rs#L107-L114)) [A].

A forced copy of the abi3 files into a free-threaded venv does not run either (M6) [A]:

| interpreter | result |
|---|---|
| 3.14.8t | `SystemError: init function of purba returned uninitialized object` |
| 3.15.0t | `ModuleNotFoundError: No module named 'purba.purba'`. 3.15t lists no `.abi3.so` suffix |

`publish.yml` uploads only wheels ([L47-L54](https://github.com/kalonji-tools/purba/blob/e2ccac8/.github/workflows/publish.yml#L47-L54)), so PyPI holds no sdist that pip could build on a free-threaded Python [A].
**So the record's example, "a free-threaded 3.14.0 reads `3.14.0t`" ([a-line-names-its-kind-in-the-glossarys-words.md L59](https://github.com/kalonji-tools/purba/blob/e2ccac8/docs/decisions/a-line-names-its-kind-in-the-glossarys-words.md?plain=1#L59)), comes only from a build from source**, such as the `3.14t` arm of [`build.yml` L94-L96](https://github.com/kalonji-tools/purba/blob/e2ccac8/.github/workflows/build.yml#L94-L96) [A].

## 2. Where each flag can come from

### Every source, against each build

M8 sets `sys.abiflags`, `sys.version`, `sysconfig._CONFIG_VARS` and `sys._is_gil_enabled` from Python, then reads each source again.

| source | `''` | `d` | `t` | `td` | can a test change it? |
|---|---|---|---|---|---|
| `sys.abiflags` (today, read at start) | `''` | `d` | `t` | `td` | yes |
| `PySys_GetObject("abiflags")` | same | same | same | same | yes: the same `sys` dict |
| `sysconfig.get_config_var("ABIFLAGS")` | `''` | `d` | `t` | `td` | yes |
| `sysconfig.get_config_var("Py_GIL_DISABLED")` | `0` | `0` | `1` | `1` | yes |
| `sys._is_gil_enabled()` | `True` | `True` | `False` | `False` | yes; and `PYTHON_GIL=1` on 3.14t gives `True` while the build stays `t` |
| `Py_GetVersion()` | `3.14.8 (main, …)` | no mark | `free-threading build` | `free-threading build` | **no** |
| `Py_GetBuildInfo()`, `Py_GetCompiler()` | no mark | no mark | no mark | no mark | no |
| `dlsym` of `_Py_NegativeRefcount` | absent | **present** | absent | **present** | **no** |
| `dlsym` of `_Py_MergeZeroLocalRefcount` | absent | absent | **present** | **present** | **no** |
| `cfg!(Py_GIL_DISABLED)`, at compile time | false | false | true | true | no |

All [A], M1 and M8.

- `Py_GetVersion()` is a C `static char version[300]`, set once from `"%.80s free-threading build (%.80s) %.80s"` when `Py_GIL_DISABLED` is defined ([getversion.c v3.15.2 L17-L23](https://github.com/python/cpython/blob/v3.15.2/Python/getversion.c#L17-L23)). 3.13 reads `experimental free-threading build` ([v3.13.0 L17-L23](https://github.com/python/cpython/blob/v3.13.0/Python/getversion.c#L17-L23)) [A].
- `Py_GetVersion()` is in the stable ABI ([stable_abi.toml v3.12.0 L1634-L1635](https://github.com/python/cpython/blob/v3.12.0/Misc/stable_abi.toml#L1634-L1635)) [A].
- purba's version already comes from it: `py.version_info()` parses `Python::version_str()`, which wraps `Py_GetVersion()` ([marker.rs v0.29.2 L699-L729](https://github.com/PyO3/pyo3/blob/v0.29.2/src/marker.rs#L699-L729)) [A]. **Only the flags in the report come from `sys` today.**
- The two symbols come from a set diff of `nm -D` over 12 builds (M7a) [A]. Every `d` build exports 7 symbols that no release build has; the stable-ABI manifest lists `_Py_NegativeRefcount` under `ifdef = 'Py_REF_DEBUG'` ([v3.15.0 L2267-L2274](https://github.com/python/cpython/blob/v3.15.0/Misc/stable_abi.toml#L2267-L2274)) [A]. None of the 5 `t`-only symbols is in the manifest.

### The routes, each with its code and its cost

Each route was built into a scratch copy of purba at `e2ccac8` (M9). The test sets `sys.abiflags = "x"` before the panic.

| route | code | result in M9 | cost |
|---|---|---|---|
| **read at start** (today) | `sys.getattr("abiflags")?` beside `sys.argv` in `_main` | right on all 8 | a missing `sys.abiflags` stops every test with exit 3; a Downside, two Confirmation lines, a fixture and two bats cases |
| **read at report time**, with a fallback | `sys.getattr("abiflags").unwrap_or_else(\|_\| "?".into())` | `x` on all 8; `?` after `del sys.abiflags` | prints what the test left, and nothing marks it wrong |
| **compile time**, for `t` | `build.rs`: `pyo3_build_config::use_pyo3_cfgs()`; `cfg!(Py_GIL_DISABLED)` | right on all 8 | a `build.rs` and a build dependency; **wrong under an `abi3t` wheel** (M11, below) |
| **`Py_GetVersion()` text**, for `t` | `Python::version_str().contains(" free-threading build ")` | right on all 8, and under `abi3t` | matches English text that is not an API, and changed once between 3.13 and 3.14 |
| **`dlsym`**, for `d` | `dlsym(RTLD_DEFAULT, c"_Py_NegativeRefcount")` is not null | right on all 8, including a `cp314t` wheel on 3.14td | `unsafe` FFI; macOS's `RTLD_DEFAULT` is `-2`, not `0`; a private, ABI-only symbol; a build with `Py_REF_DEBUG` and no `Py_DEBUG` would read as `d` |

M9's output, 3.12 to 3.15 [A]; I re-ran the four rows marked ✓ for this file:

```text
3.12    start=""   compile_t=false version_t=false dlsym_d=false report_time="x"   ✓
3.12d   start="d"  compile_t=false version_t=false dlsym_d=true  report_time="x"   ✓
3.14    start=""   compile_t=false version_t=false dlsym_d=false report_time="x"
3.14d   start="d"  compile_t=false version_t=false dlsym_d=true  report_time="x"
3.15d   start="d"  compile_t=false version_t=false dlsym_d=true  report_time="x"
3.14t   start="t"  compile_t=true  version_t=true  dlsym_d=false report_time="x"   ✓
3.14td  start="td" compile_t=true  version_t=true  dlsym_d=true  report_time="x"   ✓  (cp314t wheel)
3.15t   start="t"  compile_t=true  version_t=true  dlsym_d=false report_time="x"
```

`dlsym` found the symbols wherever the C API lives (M7b, M7c) [A]:

| where the C API lives | builds | result |
|---|---|---|
| in the executable (uv, python-build-standalone) | 10 | right |
| in `libpython3.13.so`, `libpython3.14t.so` (nixpkgs) | 2 | right |
| in `libpython3.14d.so`, `libpython3.14.so`, a C host linked to them | 2 | right |

Each run had two controls: `Py_GetVersion` found, and a made-up name not found.

### What compile time can and cannot know

| flag | `cp312-abi3` wheel (published) | version-specific wheel (from source) | `abi3.abi3t` wheel (not shipped) |
|---|---|---|---|
| `t` | known: always `''` | known: always `t` | **not known**: one binary loads on both 3.15 and 3.15t |
| `d` | **not known**: one `.so` for release and debug | **not known**: `cp314t` runs on 3.14td | not known |

**M11 falsifies compile time under `abi3t`** [A].
With the features `abi3-py312` and `abi3t-py315`, maturin builds `cp315-abi3.abi3t` on 3.15 and on 3.15t, and pip installs it on a GIL 3.15.
PyO3 sets `Py_GIL_DISABLED` for every `abi3t` build ([impl_.rs v0.29.2 L414-L433](https://github.com/PyO3/pyo3/blob/v0.29.2/pyo3-build-config/src/impl_.rs#L414-L433)), so the report on GIL 3.15 read `3.15.0t`.
The support record names `abi3t` as the way a free-threaded reader finds a wheel ([purba-supports-a-python-with-two-years-left.md L69](https://github.com/kalonji-tools/purba/blob/e2ccac8/docs/decisions/purba-supports-a-python-with-two-years-left.md?plain=1#L69)) [A].
`Py_GetVersion()` stayed right in M11.

### What the tester already has

| source | names `t` | names `d` |
|---|---|---|
| `python -VV` | yes, `free-threading build` | no |
| `python -V` | no | no |
| the installed wheel's tag | yes, for a build from source | no: `cp312-abi3` on a debug Python, `cp314t` on 3.14td |

M10 [A]. Only `sys.abiflags`, `sysconfig` or a debug-only symbol recovers `d`.

## 3. Does a report need the flags? What other tools print

The survey read the crash or internal-error code of 16 tools, and the bug-report template of 21, at a pinned tag.
Each count is a bound over this sample, not a claim about all tools.

| tool @ tag | crash text: what it says about the runtime | read when | template asks | names `t` / `d` / ABI flags |
|---|---|---|---|---|
| ruff @ 0.17.4 | nothing | — | `ruff version` | no / no / no |
| ty (ruff @ 0.17.4) | `Platform: {os} {arch}` · `Version:` · `Args:` | OS, arch: compile time | `ty version` | no / no / no |
| uv @ 0.13.7 | Rust's default panic text | — | `uv run python --version` | no / no / no |
| maturin @ v1.16.0 | `Platform: {OS} {ARCH}` · `Version:` · `Args:` | compile time | `python -V` | no / no / no |
| PyO3 @ v0.30.1 | nothing | — | `python --version` | no / no / no |
| pytest @ 9.2.0 | `INTERNALERROR>` + traceback; the session header says `Python {platform.python_version()}` | header: at start | OS and pytest versions | no / no / no |
| hypothesis @ v6.170.15 | no crash report | — | no template | no / no / no |
| mypy @ v2.4.1 | `version: {mypy_version}` | at import | "Python version used" | no / no / no |
| pip @ 26.3 | traceback | — | Python version, free text | no / no / no |
| CPython @ v3.15.2 | `Fatal Python error:` + state; no version | — | crash form: `python -VV` | no / no / no |
| numpy @ v2.5.4 | on a failed import: `Python {major}.{minor} from "{sys.executable}"` | at the failure | `sys.version`, `show_runtime()` | via `sys.version` / no / no |
| black @ 26.10.1 | `INTERNAL ERROR: Black {v} on Python ({impl}) {version}` | at the crash | "OS and Python version" | no / no / no |
| rustc @ 1.99.1 | `rustc {version} running on {triple}` · `compiler flags:` (this run's arguments) | compile time; arguments at the crash | `rustc --version --verbose` | no / no / no |
| cargo @ 0.100.0 | `note: cargo {version}` | compile time | `cargo version --verbose` | no / no / no |
| cargo-nextest @ 0.9.151 | Rust's default panic text | — | `cargo nextest --version` | no / no / no |
| Node.js @ v26.13.0 | `Node.js {NODE_VERSION}` | compile time | `node -v`, `uname -a` | no / no / no |

Templates only, with no crash text read: pydantic, polars, scikit-learn (each asks for `sys.version`, which names `t`), pandas, cryptography, Cython (neither names a flag).

| question | count |
|---|---|
| crash text names a build flag (`d`, `t`, ABI flags) | **0 of 16** |
| crash text names the interpreter's version | 3 of 16 (black and numpy, read at the crash; Node, its own runtime) |
| crash text names the OS or the CPU | 3 of 16, all compile-time constants |
| template has a field for `t`, `d` or ABI flags | **0 of 21** |
| template asks for output that names `t` (`sys.version`, `python -VV`) | 5 of 21 |
| template asks for anything that names `d` | **0 of 21** |
| a reporter that reads `sys.abiflags` | 0 of 16 crash texts, and 0 of 7 on-demand reporters |

Sources for every row: see [Appendix B](#appendix-b--survey-sources).

- The only reporter that names `d` is `pip debug`, on demand, through a `d` in its tag list. packaging takes it from `Py_DEBUG` and `Py_GIL_DISABLED`, not from `sys.abiflags` ([pip debug.py 26.3 L166-L193](https://github.com/pypa/pip/blob/26.3/src/pip/_internal/commands/debug.py#L166-L193)) [A].
- The free-threading guide treats the build and the GIL state as two facts. `sysconfig.get_config_var("Py_GIL_DISABLED")` tells the build "irrespective of whether the GIL was re-enabled at runtime" ([running-gil-disabled.md L22-L78](https://github.com/Quansight-Labs/free-threaded-compatibility/blob/53b7eaf3f5fa312e3deef3b944af35074854af1e/docs/running-gil-disabled.md?plain=1#L22-L78)) [A].

## 4. The zygote

What the record says, at `e2ccac8` ([each-test-runs-in-a-process-of-its-own.md](https://github.com/kalonji-tools/purba/blob/e2ccac8/docs/decisions/each-test-runs-in-a-process-of-its-own.md?plain=1#L28-L46)) [A]:

| line | text |
|---|---|
| L28-L30 | "Each test runs in a process of its own. No change that a test makes in memory reaches another test." |
| L32 | "A zygote of purba's own loads the module once, and forks each test." |
| L36 | "Until purba has its zygote, the tests of a module share one interpreter." |

Neither the record nor [#694](https://github.com/kalonji-tools/purba/issues/694) says which process writes the report, or whether the module body runs in that process [A].

Reasoning, in no record:

- A test body's change to `sys` stays in its forked child. If `_main`'s process forks every test, no test body changes the `sys` the report reads.
- The module body runs in the zygote. If the zygote is `_main`'s own process, module-level code such as `sys.abiflags = "x"` still reaches the report.
- The routes through `Py_GetVersion()` and `dlsym` do not depend on that design.

## What this research could not verify

- **macOS** [C]: no macOS host. `dlsym(RTLD_DEFAULT, …)` on a framework build, and whether the marker symbols are exported there.
- **A build with `Py_REF_DEBUG` and no `Py_DEBUG`** [C]: not built. By the manifest, `dlsym` reads it as `d` while `sys.abiflags` reads `''`.
- **Distribution Pythons** (Debian, Fedora, Homebrew) [C]: not measured. Both layouts of the C API were measured, through uv and nixpkgs.
- **An embedder that loads libpython with `RTLD_LOCAL`** [C]: not tested. purba's own `Py*` imports resolve from the same global scope that `RTLD_DEFAULT` searches, so purba would likely not load there at all. That is an inference.
- **The survey ran no tool** [C]: every crash line in section 3 is read from source. The crash text of pydantic, polars, scikit-learn, pandas, Cython and cryptography was not read. CPython's templates are from `main` at `d7684fb`, because the v3.15.2 tag has none.

## Appendix A — probes

Linux x86_64, 2026-11-07. Interpreters: uv 0.13.1 (python-build-standalone) for 3.12d, 3.13.16 (`''`, `t`, `d`), 3.14.8 (`''`, `t`, `d`, `td`) and 3.15.0 (`''`, `t`, `d`); mise 3.12.14; nixpkgs `python313` and `python314FreeThreading`. Builds: maturin 1.15.0, rust nightly-2026-11-02, PyO3 0.29.2 as `Cargo.lock` pins.

| probe | what it does | command |
|---|---|---|
| M1 | each flag source, and pip's tag verdict, per interpreter | `python -I measure.py`, with `pip._vendor.packaging.tags.sys_tags()` |
| M3 | build purba on 3.12, 3.12d, 3.13t, 3.14t, 3.14td, 3.15t | `maturin build --release --interpreter <py> --features extension-module,fault` |
| M4 | PyO3's resolved config | `PYO3_PRINT_CONFIG=1 maturin build -i <py>`; control: plain `cargo build` with `PYO3_PYTHON=<3.12d>` gives `build_flags=Py_REF_DEBUG,Py_DEBUG` |
| M5 | pip's verdict and today's report line | `pip install --no-index --find-links <wheels> purba`, then `PURBA_FAULT=assert purba test_changes_abiflags.py` in `tests/fault` |
| M6 | the abi3 files forced into a free-threaded venv | extract the wheel into site-packages, call `purba._main()`; control: 3.14.8 |
| M7 | the marker symbols | `nm -D --defined-only` diff over 12 executables; `dlsym` from a `.so` under ctypes, and from a C host linked to libpython |
| M8 | what a test can change | set each Python-level source, read every source again; control: `PYTHON_GIL=1` on 3.14t |
| M9 | each route inside purba | the scratch copy below; the test sets `sys.abiflags = "x"`, and a second test deletes it |
| M10 | what the tester has | `python -VV`, and the installed `WHEEL` tag |
| M11 | compile time under `abi3t` | M9's copy with pyo3 features `["abi3-py312", "abi3t-py315"]`, built and run on 3.15.0 and 3.15.0t |

M9's change to `crates/purba/src/lib.rs`, at the end of `ask_for_report`, and the `build.rs` it adds:

```rust
let compile_t = cfg!(Py_GIL_DISABLED);
let version_t = pyo3::Python::version_str().contains(" free-threading build ");
let dlsym_d = exported(c"_Py_NegativeRefcount");
let report_time = py
    .import("sys")
    .and_then(|sys| sys.getattr("abiflags")?.extract::<String>())
    .unwrap_or_else(|_| "?".to_owned());

fn exported(name: &std::ffi::CStr) -> bool {
    unsafe extern "C" {
        fn dlsym(handle: *mut std::ffi::c_void, symbol: *const std::ffi::c_char) -> *mut std::ffi::c_void;
    }
    // glibc's RTLD_DEFAULT is 0. macOS's is (void *)-2.
    !unsafe { dlsym(std::ptr::null_mut(), name.as_ptr()) }.is_null()
}
```

```rust
// build.rs
fn main() {
    println!("cargo::rustc-check-cfg=cfg(Py_GIL_DISABLED)");
    pyo3_build_config::use_pyo3_cfgs();
}
```

M6 and M11 output:

```text
M6   3.14.8 (control)  exit 125, "… on CPython 3.14.8, linux x86_64."
     3.14.8t           exit 1, SystemError: init function of purba returned uninitialized object
     3.15.0t           exit 1, ModuleNotFoundError: No module named 'purba.purba'
M11  cp315-abi3.abi3t on 3.15.0   compile_t=true  version_t=false   (compile time says t: wrong)
     cp315-abi3.abi3t on 3.15.0t  compile_t=true  version_t=true
```

## Appendix B — survey sources

| tool | crash code | template |
|---|---|---|
| ruff | [lib.rs L133-L153](https://github.com/astral-sh/ruff/blob/0.17.4/crates/ruff/src/lib.rs#L133-L153) | [1_bug_report.yaml L25-L31](https://github.com/astral-sh/ruff/blob/0.17.4/.github/ISSUE_TEMPLATE/1_bug_report.yaml#L25-L31) |
| ty | [ruff_db diagnostic/mod.rs L104-L137](https://github.com/astral-sh/ruff/blob/0.17.4/crates/ruff_db/src/diagnostic/mod.rs#L104-L137) | [ty 1_bug_report.yaml L31-L37](https://github.com/astral-sh/ty/blob/0.0.99/.github/ISSUE_TEMPLATE/1_bug_report.yaml#L31-L37) |
| uv | Rust's [panicking.rs L273](https://github.com/rust-lang/rust/blob/1.99.1/library/std/src/panicking.rs#L273) | [1_bug_report.yaml L21-L43](https://github.com/astral-sh/uv/blob/0.13.7/.github/ISSUE_TEMPLATE/1_bug_report.yaml#L21-L43) |
| maturin | [main.rs L290-L309](https://github.com/PyO3/maturin/blob/v1.16.0/src/main.rs#L290-L309) | [bug_report.yml L18-L38](https://github.com/PyO3/maturin/blob/v1.16.0/.github/ISSUE_TEMPLATE/bug_report.yml#L18-L38) |
| PyO3 | [err/mod.rs L300-L311](https://github.com/PyO3/pyo3/blob/v0.30.1/src/err/mod.rs#L300-L311) | [bug_report.yml L36-L76](https://github.com/PyO3/pyo3/blob/v0.30.1/.github/ISSUE_TEMPLATE/bug_report.yml#L36-L76) |
| pytest | [terminal.py L594-L597](https://github.com/pytest-dev/pytest/blob/9.2.0/src/_pytest/terminal.py#L594-L597), [L877-L899](https://github.com/pytest-dev/pytest/blob/9.2.0/src/_pytest/terminal.py#L877-L899) | [1_bug_report.md L13-L16](https://github.com/pytest-dev/pytest/blob/9.2.0/.github/ISSUE_TEMPLATE/1_bug_report.md?plain=1#L13-L16) |
| hypothesis | [engine.py L773-L780](https://github.com/HypothesisWorks/hypothesis/blob/v6.170.15/hypothesis/src/hypothesis/internal/conjecture/engine.py#L773-L780) | none |
| mypy | [errors.py L1357-L1391](https://github.com/python/mypy/blob/v2.4.1/mypy/errors.py#L1357-L1391) | [crash.md L33-L41](https://github.com/python/mypy/blob/v2.4.1/.github/ISSUE_TEMPLATE/crash.md?plain=1#L33-L41) |
| pip | [base_command.py L165-L166](https://github.com/pypa/pip/blob/26.3/src/pip/_internal/cli/base_command.py#L165-L166) | [bug-report.yml L31-L45](https://github.com/pypa/pip/blob/26.3/.github/ISSUE_TEMPLATE/bug-report.yml#L31-L45) |
| CPython | [pylifecycle.c L3661-L3722](https://github.com/python/cpython/blob/v3.15.2/Python/pylifecycle.c#L3661-L3722) | [crash.yml L26-L56](https://github.com/python/cpython/blob/d7684fbef52e563a89c88a23efc68de2de9b3503/.github/ISSUE_TEMPLATE/crash.yml#L26-L56) |
| numpy | [_core/__init__.py L58-L74](https://github.com/numpy/numpy/blob/v2.5.4/numpy/_core/__init__.py#L58-L74) | [bug-report.yml L46-L62](https://github.com/numpy/numpy/blob/v2.5.4/.github/ISSUE_TEMPLATE/bug-report.yml#L46-L62) |
| black | [__init__.py L1830-L1834](https://github.com/psf/black/blob/26.10.1/src/black/__init__.py#L1830-L1834), [L1852-L1868](https://github.com/psf/black/blob/26.10.1/src/black/__init__.py#L1852-L1868) | [bug_report.md L55-L60](https://github.com/psf/black/blob/26.10.1/.github/ISSUE_TEMPLATE/bug_report.md?plain=1#L55-L60) |
| rustc | [diagnostics.rs L73-L105](https://github.com/rust-lang/rust/blob/1.99.1/compiler/rustc_driver_impl/src/diagnostics.rs#L73-L105), [lib.rs L1524-L1612](https://github.com/rust-lang/rust/blob/1.99.1/compiler/rustc_driver_impl/src/lib.rs#L1524-L1612) | [ice.yaml L39-L55](https://github.com/rust-lang/rust/blob/1.99.1/.github/ISSUE_TEMPLATE/ice.yaml#L39-L55) |
| cargo | [src/lib.rs L194-L211](https://github.com/rust-lang/cargo/blob/0.100.0/src/lib.rs#L194-L211) | [bug_report.yml L38-L43](https://github.com/rust-lang/cargo/blob/0.100.0/.github/ISSUE_TEMPLATE/bug_report.yml#L38-L43) |
| cargo-nextest | [output.rs L251-L277](https://github.com/nextest-rs/nextest/blob/cargo-nextest-0.9.151/cargo-nextest/src/output.rs#L251-L277) | [bug-report.yml L41-L48](https://github.com/nextest-rs/nextest/blob/cargo-nextest-0.9.151/.github/ISSUE_TEMPLATE/bug-report.yml#L41-L48) |
| Node.js | [node_errors.cc L590-L592](https://github.com/nodejs/node/blob/v26.13.0/src/node_errors.cc#L590-L592) | [1-bug-report.yml L13-L25](https://github.com/nodejs/node/blob/v26.13.0/.github/ISSUE_TEMPLATE/1-bug-report.yml#L13-L25) |
| pydantic | — | [bug-v2.yml L45-L59](https://github.com/pydantic/pydantic/blob/v2.14.3/.github/ISSUE_TEMPLATE/bug-v2.yml#L45-L59) |
| polars | — | [bug_report_python.yml L63-L78](https://github.com/pola-rs/polars/blob/py-2.1.0/.github/ISSUE_TEMPLATE/bug_report_python.yml#L63-L78) |
| scikit-learn | — | [bug_report.yml L85-L95](https://github.com/scikit-learn/scikit-learn/blob/1.9.1/.github/ISSUE_TEMPLATE/bug_report.yml#L85-L95) |
| pandas | — | [bug_report.yaml L56-L71](https://github.com/pandas-dev/pandas/blob/v3.1.1/.github/ISSUE_TEMPLATE/bug_report.yaml#L56-L71) |
| cryptography | — | [ISSUE_TEMPLATE.rst L12-L18](https://github.com/pyca/cryptography/blob/50.0.2/.github/ISSUE_TEMPLATE.rst?plain=1#L12-L18) |
| Cython | — | [1_bug_report.yml L44-L58](https://github.com/cython/cython/blob/3.3.1/.github/ISSUE_TEMPLATE/1_bug_report.yml#L44-L58) |
