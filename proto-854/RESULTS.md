# Where the report takes the ABI flags: the prototype of #854

Three ways give the true flags on all 8 interpreter and wheel pairs, whatever a test does to `sys.abiflags`.
They are W0 (today's read at start), W1 (version text and symbol) and W2 (compile time and symbol).
W0 alone stops every test, with exit 3, when `sys.abiflags` is missing before the run.
W2 alone names a GIL 3.15 as `3.15.0t` once the wheel is `abi3t`, and W1 stays right there.
W3 (version text alone) drops `d` on the 4 debug pairs, by design.
W4 (read at report time) prints the test's `x`, or `?`, on every pair once a test touches `sys.abiflags`.
Every way passes `mise run test` after the test and record changes listed below, and every planted bug either changed the report or stopped the build.
The default interpreter of `mise run test` is a release 3.12. A planted loss of `d` passed the suite's report cases there, and a 3.12 has no `t` to lose.

Tags: [A] measured this session. [B] relayed from the research (`research/abiflags.md` on `origin/research/abiflags`), not re-measured. [C] not run.

## The table

| way | 8 pairs right (control test) | survives `sys.abiflags = "x"` | survives `del sys.abiflags` | missing before the run | suite green after | added Rust (lines, `unsafe`) | new deps and files | lint |
|---|---|---|---|---|---|---|---|---|
| W0 baseline | 8/8 | 8/8 | 8/8 | exit 3, no test runs | green, 0 changes | — | — | pass |
| W1 version and symbol | 8/8 | 8/8 | 8/8 | 8/8, tests run | green; tests −14 +1, records −3 +1 | +19, 1 `unsafe` | `libc` reaches `purba`; `libc` moves to `[workspace.dependencies]` | pass |
| W2 compile time and symbol | 8/8 | 8/8 | 8/8 | 8/8, tests run | green; tests −14 +1, records −3 +1 | +18, and `build.rs` +6; 1 `unsafe` | as W1, plus build dependency `pyo3-build-config` and the new file `crates/purba/build.rs` | pass |
| W3 version alone | 4/8: the 4 debug pairs lose `d` | 4/8, same 4 | 4/8, same 4 | 4/8, tests run | green; tests −15 +2, records −4 +2 | +12, 0 `unsafe` | none | pass |
| W4 read at report time | 8/8 | 0/8: names `x` | 0/8: names `?` | 0/8: names `?`, tests run | green; tests −35, records −3 +1 | +5, 0 `unsafe` | none | pass |

All cells [A].
"Added Rust" counts the lines of the way itself.
On top of it, every way except W0 shares the removal of the read at start: `crates/purba/src/lib.rs` −13 +9.
Raw rows: `runs/pairs.txt` (8 pairs × 4 tests × 5 ways), `runs/suite-summary.txt`, `runs/suite-*.log`.

The 8 pairs are those of the research: the `cp312-abi3` wheel on 3.12 (mise, 3.12.14), 3.12d, 3.14, 3.14d and 3.15d; the `cp314-cp314t` wheel on 3.14t and on 3.14td; the `cp315-cp315t` wheel on 3.15t.
Each wheel is the `fault` wheel, and `PURBA_FAULT=assert` forces the panic, so every run exits 125 and prints the report.
The right answer is `platform.python_version() + sys.abiflags`, read in a fresh process of the same venv.
The fourth test, "missing before the run", deletes `sys.abiflags` in a `sitecustomize.py`, as `tests/skeleton.bats:50` does.

## The suite, per way

`mise run test` ran on the default mise interpreter, first with the way's code alone, then with its test and record changes.
`mise run lint` ran once the test changes were in.

| way | code alone: cases that fail | test lines changed (−/+) | record lines changed (−/+) | then `mise run test` | then `mise run lint` |
|---|---|---|---|---|---|
| W0 | — | 0 | 0 | exit 0 | exit 0, 15 of 15 lints |
| W1 | `skeleton.bats` "purba with no sys.abiflags exits 3, and says why on stderr" | −14 / +1 | −3 / +1 | exit 0 | exit 0, 15 of 15 |
| W1, `d` check in `purba-execution` | the same case | −14 / +1 | −3 / +1 | exit 0 | `lint:pins` hit GitHub's rate limit; the other 14 exit 0 |
| W2 | the same case | −14 / +1 | −3 / +1 | exit 0 | exit 0, 15 of 15 |
| W3 | the same case | −15 / +2 | −4 / +2 | exit 0 | exit 0, 15 of 15 |
| W4 | the same case, and `fault.bats` "the report of an internal error keeps the flags read before a test changed sys.abiflags" | −35 / 0 | −3 / +1 | exit 0 | exit 0, 15 of 15 |

All [A], from `runs/suite-summary.txt` and the logs beside it.

What each change is, and whether green needs it:

- **Every way but W0** deletes the case at `tests/skeleton.bats:50-61` (13 lines with its blank). Green needs it: a Python with no `sys.abiflags` now runs its tests.
- **W1, W2, W3** rename the case at `tests/fault.bats:177` to "the report of an internal error ignores a test that changed sys.abiflags". The case still passes unchanged; its old name says "read before", which is no longer true.
- **W1, W2, W3** delete the Downside at `an-error-stops-only-the-work-that-depends-on-it.md:51` and its two Confirmation lines at `:64-65`. They move the `fault.bats:177` line to the Confirmation of `a-line-names-its-kind-in-the-glossarys-words.md`, after `:85`, because it confirms the report line, not the Downside.
- **W3** also rewrites `a-line-names-its-kind-in-the-glossarys-words.md:59`, which says the Python carries its ABI flags, and `tests/internal_error.sh:13`, to drop `d` from the expected line. Green on mise's 3.12 needs neither. Without the second, the two report cases of `fault.bats` fail on 3.12d and 3.14td; with it, they pass there [A, `runs/w3-suite-patch-on-debug.txt`].
- **W4** deletes the case at `tests/fault.bats:176-191` and its fixture `tests/fault/test_changes_abiflags.py`. It rewrites the Downside at `:51` to say a test that changes `sys.abiflags` changes the report, and deletes `:64-65`. `a-line-names-its-kind-in-the-glossarys-words.md:59` still says "A tester's report then names the build that met the error"; under W4 that holds only when no test touched `sys.abiflags`. I left it, so W4's record count is a floor.
- `tests/internal_error.sh:13` reads `sys.abiflags` in a fresh process, so no test can change it. W1, W2 and W4 keep it as is.
- One lint tripped on my own first patch, not on a way: the renamed `fault.bats` title ran to 104 columns, and `lint:style` refused it. The shorter title above passes.


## Per-way notes

### W0 baseline

- Right on 8/8 for the control, the set and the delete tests [A]. The read at start happens before any test, so a test cannot reach it.
- A Python with no `sys.abiflags` before the run exits 3 with `purba: invocation error: AttributeError: module 'sys' has no attribute 'abiflags'`, on all 8 pairs [A].
- W0 has no planted bug of its own. W4 is W0 with the read moved, and its rows show the set and delete tests catch that move [A].

### W1 version and symbol

- `t` comes from `Python::version_str()`, which wraps `Py_GetVersion()`. purba already reads it for `version_info()`, so W1 adds no new source for `t` [A from PyO3 0.29.2 `marker.rs` L699-L729].
- `d` comes from `libc::dlsym(libc::RTLD_DEFAULT, c"_Py_NegativeRefcount")`. libc 0.2.189 defines `RTLD_DEFAULT` as null on Linux and as `-2` on every BSD, Apple included (`src/unix/bsd/mod.rs` L356) [A, read from the source].
- **Where the helper lives.** I put it in `purba`, beside the report. The cost: `libc` becomes a dependency of `purba`, so I moved it to `[workspace.dependencies]`, the way the repo shares `pyo3` and `serde`. That is 3 manifest lines and 1 `Cargo.lock` line [A].
- The other home, `purba-execution`, needs no manifest change: a `pub fn debug_build()` of 9 lines there, and `purba` calls it. It passes `cargo clippy --all-targets` [A]. Its cost is a public function about the interpreter in a crate whose doc says it runs tests. Patch: `ways/w1-version-and-symbol.in-execution.patch`. With W1's test changes it passes `mise run test`, and 14 of 15 lints; `lint:pins` hit GitHub's rate limit [A].
- Under an `abi3t` wheel, W1 names 3.15.0 as `3.15.0` and 3.15.0t as `3.15.0t`, from either build [A, `runs/abi3t.txt`].

### W2 compile time and symbol

- `t` comes from `cfg!(Py_GIL_DISABLED)`, set by `crates/purba/build.rs` calling `pyo3_build_config::use_pyo3_cfgs()`. That function already prints `cargo:rustc-check-cfg=cfg(Py_GIL_DISABLED)`, so `build.rs` needs no check-cfg line of its own [A, pyo3-build-config 0.29.2 `src/lib.rs` L40-L44, L154-L156].
- The build dependency `pyo3-build-config = "0.29.2"` must follow `pyo3`'s version by hand. I put it in `purba`'s manifest, since only `purba` uses it.
- **Under `abi3t` W2 is wrong.** The `cp315-abi3.abi3t` wheel, built on 3.15.0 or on 3.15.0t, names a GIL 3.15.0 as `3.15.0t` [A, `runs/abi3t.txt`]. This repeats the research's M11 inside the real way code.
- The lints guard the build script: removing `build.rs` alone fails on `unused build dependency`, and removing both fails on `unexpected cfg condition name: Py_GIL_DISABLED` [A].

### W3 version alone

- Right wherever the build is not debug. On 3.12d, 3.14d and 3.15d it names no `d`, and on 3.14td it names `t` alone [A].
- The suite stays green on mise's 3.12 without the `internal_error.sh` change. That change, `sys.abiflags.replace("d", "")`, only keeps the case true on a debug Python, which no CI arm runs. Without it, `tests/fault.bats` fails on 3.12d and 3.14td [A, `runs/planted-bats.txt` control rows].
- The record line `a-line-names-its-kind-in-the-glossarys-words.md:59` says "The Python carries its ABI flags". Under W3 that is false for `d`, so the patch rewrites it.

### W4 read at report time

- Right only when no test touched `sys.abiflags`. A test that sets it prints `x`; one that deletes it prints `?` [A].
- A `sitecustomize.py` that deletes it no longer stops the run. The tests run, and the report names `?` [A].
- The fault case at `tests/fault.bats:177` fails under W4, because it checks the true flags. The patch deletes it and its fixture. The only other choice is a case that expects the test's own `x`, which would make the defect the contract.

## Planted bugs

Each plant is a one-line edit on top of the way's patch, listed in `run.sh` under `planted=`. Each row ran the control and the set tests [A, `runs/planted.txt`].

| plant | way | pairs | the report then names | measure fails? |
|---|---|---|---|---|
| `_Py_NegativeRefCount` (wrong case) | W1 | 3.12d, 3.15d | `3.12.15`, `3.15.0`: no `d` | yes; 3.12 stays right, as it should |
| `" free-threaded build "` | W1 | 3.14t, 3.14td | `3.14.8`, `3.14.8d`: no `t` | yes |
| `build.rs` removed | W2 | 3.14t | no wheel: `error: unused build dependency pyo3-build-config` | the build fails first |
| `build.rs` and its dependency removed | W2 | 3.14t | no wheel: `error: unexpected cfg condition name: Py_GIL_DISABLED` | the build fails first |
| `build.rs` calls `print_expected_cfgs()` only | W2 | 3.14t, 3.14td | `3.14.8`, `3.14.8d`: no `t` | yes |
| `_Py_NegativeRefCount` | W2 | 3.12d | `3.12.15`: no `d` | yes |
| `" free-threaded build "` | W3 | 3.14t | `3.14.8`: no `t` | yes |
| `getattr("abiflag")` | W4 | 3.12, 3.12d | `3.12.14?`, `3.12.15?` | yes |

**Does the repo's own test see the plant?** I ran the two cases of `tests/fault.bats` that compare the report line on each planted wheel, and on each unplanted wheel as a control [A, `runs/planted-bats.txt`].

- Every plant fails both cases on the interpreter it targets. Every unplanted W1 and W2 wheel passes both, on 3.12, 3.12d, 3.14t and 3.14td.
- The `d` plant passes both cases on mise's 3.12. So `mise run test` cannot see a lost `d`; only a debug interpreter can.
- No CI arm runs a debug Python (`build.yml` L96: `3.12`, `3.13`, `3.14`, `3.14t`). The `t` plants would fail the `3.14t` arms; the `d` plants would pass every arm [A for the matrix line; the CI run itself is C].

## macOS: what a probe would need to show [C]

I cannot run macOS. These are the jobs and lines a probe pull request would read.

- **`t` for W1 and W2.** The jobs `wheel macos-latest py3.14t` (build.yml L86-L174) run `bats tests` at L171-L174. The case at `tests/fault.bats:10` and the case at `:177` compare the report's last two lines to `report_lines`, which reads `sys.abiflags` of the venv. A pass there shows `3.14.Xt` on python.org's framework build.
- **No false `d`.** The arms `py3.12`, `py3.13`, `py3.14` on macOS pass the same two cases only if `dlsym` finds nothing in a release build.
- **A pass cannot show that `d` works on macOS.** A wrong handle, such as null instead of `-2`, also finds nothing, and prints no `d`. Today's matrix cannot tell the two apart.
- **To show `d`,** the probe needs a debug Python on macOS. `actions/setup-python` offers none that I know of. One route is an extra step, `uv python install 3.14+debug`, then the same build and `bats tests/fault.bats` on it. That build is python-build-standalone, which is not a framework build, so the framework case of `d` would stay unmeasured.
- W3 needs only the first point. W4 needs none of them; its behaviour does not depend on the OS.

## What I could not verify

- macOS, for every way [C]. See above.
- A build with `Py_REF_DEBUG` and no `Py_DEBUG` [C]. W1 and W2 would name it `d`; `sys.abiflags` names it `''` [B].
- Distribution Pythons (Debian, Fedora, Homebrew) [C]. The pairs here are python-build-standalone and mise's 3.12 [A].
- The full `mise run lint` on the in-execution variant of W1: `lint:pins` stopped on GitHub's API rate limit, and I reran the others by hand. The variant changes no workflow, so the result of `lint:pins` should not depend on it [C].
- Whether the user's CI would pass: no pull request was opened, by instruction [C].
- `run.sh` itself is not lint-clean for the repo: shellcheck's optional SC2312 notes fire, and `lint:placement` would likely refuse a script under `proto-854/`. It is untracked work, not a candidate file.

## Files

- `ways/<way>.patch`: the way's code, against `e2ccac8`. W0 has none.
- `ways/<way>.suite.patch`: its test and record changes, applied on top.
- `ways/w1-version-and-symbol.in-execution.patch`: W1 with the `d` check in `purba-execution`.
- `fixtures/`: the control, set and delete tests, and the `sitecustomize.py` that deletes `sys.abiflags` before the run.
- `run.sh <interpreter dir> [pairs planted abi3t suite]`: rebuilds every tree under `work/` (ignored) from the patches, and writes `runs/`. The trees, target dirs and venvs take about 19 GB; `rm -rf proto-854/work` frees them.
