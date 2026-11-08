# The report names only the free-threaded flag

## Context and Problem Statement

The report of an internal error names the Python, as [a line names its kind in the glossary's words](a-line-names-its-kind-in-the-glossarys-words.md#decision-outcome) decides.
When purba took this decision, it read the ABI flags from `sys.abiflags` before any test.
A test shares purba's interpreter, and can set or delete `sys.abiflags`.
So purba on a Python with no `sys.abiflags` ran no test.

| fact | source |
|---|---|
| pip refuses the `cp312-abi3` wheel on each free-threaded CPython, so `t` reaches the report only from a build from source | [the research](https://github.com/kalonji-tools/purba/blob/819176b/research/abiflags.md?plain=1#L48-L73) |
| `Py_GetVersion()` holds `free-threading build` on a free-threaded build, and a test cannot change it | [the same](https://github.com/kalonji-tools/purba/blob/819176b/research/abiflags.md?plain=1#L77-L90) |
| no crash text of 16 tools names a build flag, and no bug template of 21 asks about a debug build | [the same](https://github.com/kalonji-tools/purba/blob/819176b/research/abiflags.md?plain=1#L159-L192) |
| a failed refcount check of a debug build ends in `Py_FatalError`, so the process aborts before any report | [`object.c` L273-L277](https://github.com/python/cpython/blob/v3.15.0/Objects/object.c#L273-L277), [L3278](https://github.com/python/cpython/blob/v3.15.0/Objects/object.c#L3278) |
| `Py_GetVersion()` gave the true `t` on 8 pairs of interpreter and wheel, after a test set or deleted `sys.abiflags` | [the prototype](https://github.com/kalonji-tools/purba/blob/828b4d6/proto-854/RESULTS.md?plain=1#L16-L22) |
| `Py_GetVersion()` gave `t` on the free-threaded build of python.org for macOS | [the same](https://github.com/kalonji-tools/purba/blob/828b4d6/proto-854/RESULTS.md?plain=1#L117-L127) |
| an `abi3t` wheel that takes `t` from the build of purba names a 3.15 with a GIL as `3.15.0t` | [the same](https://github.com/kalonji-tools/purba/blob/828b4d6/proto-854/RESULTS.md?plain=1#L81) |
| a read of `sys.abiflags` after a panic names the `x` that a test wrote, or a `?` when a test deleted it | [the same](https://github.com/kalonji-tools/purba/blob/828b4d6/proto-854/RESULTS.md?plain=1#L22) |

## Considered Options

- **Read `sys.abiflags` before any test.** purba on a Python with no `sys.abiflags` runs no test, for one field of the report of an internal error.
- **Read `t` from `Py_GetVersion()`, and `d` from `dlsym` of `_Py_NegativeRefcount`.** A failed refcount check of a debug build aborts before the report. So `d` marks only an internal error that the debug build did not cause. It needs an `unsafe` call. No CI job runs a debug CPython to check that call.
- **Read `t` from the build of purba, and `d` from `_Py_NegativeRefcount`.** An `abi3t` wheel loads on CPython with and without a GIL, so the build of purba cannot know `t`.
- **Read `sys.abiflags` when purba prints the report.** The report names the flags that a test left.

## Decision Outcome

**Reach:** `crates/purba/src/lib.rs` `tests/fault.bats` `tests/skeleton.bats` `tests/internal_error.sh` `tests/fault/test_changes_abiflags.py` `.github/workflows/build.yml`

**Area:** reporting

**The report of an internal error names `t` from `Py_GetVersion()`, and names no other ABI flag.**
A free-threaded build runs threads with no GIL. So purba's own code can fail there in ways that it cannot fail with a GIL.
No test can change `Py_GetVersion()`, so purba reads no flag before any test.
purba runs the tests on a Python with no `sys.abiflags`.

| flag | purba names it when | CPython source |
|---|---|---|
| `t` | `Py_GetVersion()` holds `free-threading build`, with a space on each side | [`getversion.c`](https://github.com/python/cpython/blob/v3.15.2/Python/getversion.c#L17-L23) |

**Downside:**

- **A debug build reads as a release build in the report.** A maintainer asks the tester for `sys.abiflags` when the build matters.
- **purba finds `t` in the English text of `Py_GetVersion()`.** The text is not an API. It changed from `experimental free-threading build` in 3.13 to `free-threading build` in 3.14. The match finds both.

## Confirmation

On the `fault` wheel, `tests/fault.bats` checks that a test that changes `sys.abiflags` leaves the flag of the report as `Py_GetVersion()` names it.
On the `fault` wheel, `tests/fault.bats` checks the flag of the report on a Python with no `sys.abiflags`.
Build's `wheel` job runs `tests/fault.bats` on `3.14t`, and is the only check that meets a `t`.
`mise run test:wheel` runs `tests/fault.bats` on a Python with a GIL.
`mise run test:wheel` checks that purba runs the tests on a Python with no `sys.abiflags`.

[A line names its kind in the glossary's words](a-line-names-its-kind-in-the-glossarys-words.md#decision-outcome) decides what the report names.

[How does the report of an internal error name the ABI flags, with no read of sys.abiflags before the run?](https://github.com/kalonji-tools/purba/issues/845) researched where the flags can come from.
[Where does the report of an internal error take the ABI flags from, once purba no longer reads sys.abiflags before the run?](https://github.com/kalonji-tools/purba/issues/854) built the prototype.
