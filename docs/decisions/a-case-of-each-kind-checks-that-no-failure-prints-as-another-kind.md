# A case of each kind checks that no failure prints as another kind

## Context and Problem Statement

[A line names its kind in the glossary's words](a-line-names-its-kind-in-the-glossarys-words.md#decision-outcome) gives each kind its line and its stream.
[A run exits 0 only when it checked something and every test passed](a-run-exits-0-only-when-it-checked-something-and-every-test-passed.md#decision-outcome) gives each kind its exit code.
On `main` at `77ed22a`, purba prints no line that names a project error, an internal error or an invocation error.
A failure can change into another kind where a part sees it, or in the error that carries it.
It can also change into another kind in the line that reporting writes, or in the exit code.

| fact | source |
|---|---|
| `tests/skeleton.bats` checks the exit code of each run and the whole of each stream it reads, and holds no case of a test that raised and no case of an internal error | [`tests/skeleton.bats`](https://github.com/kalonji-tools/purba/blob/77ed22a/tests/skeleton.bats#L14) |
| reporting prints the error of every test through one arm | [`reporting`](https://github.com/kalonji-tools/purba/blob/77ed22a/crates/reporting/src/lib.rs#L90) |
| the error that execution returns is text | [`execution`](https://github.com/kalonji-tools/purba/blob/77ed22a/crates/execution/src/lib.rs#L80) |
| clippy's `wildcard_enum_match_arm` refuses a `_` arm over three variants, and passes an `Err(error)` arm that takes every error | [the probe](https://github.com/kalonji-tools/purba/issues/716#issuecomment-6528571812) |
| outside reporting, the word `failed` stands on 25 lines of 5 files in `crates/`, and one of those lines names a variable that prints nothing | [the count](https://github.com/kalonji-tools/purba/issues/716#issuecomment-6528571812) |
| the ruleset requires Build, and Build runs bats on Linux and macOS only | [`sign.yml`](https://github.com/kalonji-tools/purba/blob/77ed22a/.github/workflows/sign.yml#L26-L27), [`build.yml`](https://github.com/kalonji-tools/purba/blob/77ed22a/.github/workflows/build.yml#L160-L163) |
| a panic that crosses a test's call prints Rust's hook, PyO3's banner and a Python traceback on stderr | [a panic inside a test gives that test an internal error](a-panic-inside-a-test-gives-that-test-an-internal-error.md#decision-outcome) |

## Considered Options

- **A type that reporting must match in full.** The compiler refuses a match that misses an arm, and passes an arm that prints the wrong word. Clippy passes an `Err(error)` arm that takes every error.
- **A check over the source.** The word that names a kind also stands in code that prints nothing.

## Decision Outcome

**Reach:** `tests/**` `.github/workflows/build.yml`

**Area:** errors

**A case of each kind checks that no failure prints as another kind.**

- **No case of a kind prints a line, uses a stream or sets an exit code that the records do not give its kind.** A project error and a test that raised still share one exit code.
- **`tests/kinds.bats` runs one case of each kind on an installed wheel, in one run for each case.** It reads what a tester reads. It matches the exit code and the whole stdout. It matches the whole stderr of each case except the panic case.
- **`tests/kinds.bats` matches only the last two lines of the panic case's stderr.** It builds them from purba's version, the Python's version and ABI flags, the OS and the architecture of the run. Rust's hook and the Python traceback above them change with each edit and each Python version.
- **The module of the panic case also holds a test that passes, one that fails and one that raises.** purba prints the line of each of them. The run exits 125.
- **The panic case's test sets `PURBA_FAULT=assert` around its own assert, and takes it away after.** The switch needs no new code. The test runs second, so two tests run after it.
- **The `wheel` job of Build builds the `fault` wheel and runs `tests/kinds.bats`, on each OS where bats runs and each Python version.** The ruleset requires Build.
- **The panic case runs on the `fault` wheel, and every other case runs on the wheel without `fault`.** [A test causes an internal error through a switch that only a test build carries](a-test-causes-an-internal-error-through-a-switch-that-only-a-test-build-carries.md#decision-outcome) decides each wheel.

**Downside:**

- **`tests/kinds.bats` misses a shape of a kind that it holds no case of.** A thread's exception, a module named after a loaded module and a panic while a module loads are some of them.
- **`tests/kinds.bats` misses a defect of purba that a part sees as a failed assert.** The `fault` switch causes a panic or an error that purba never expects, and never a wrong answer.
- **`tests/kinds.bats` misses whether exit code 1 comes before exit code 4.** The panic case meets both, and exits 125. `mise run test:rust` checks that order through command.
- **`tests/kinds.bats` misses Windows.** bats does not run there.
- **`tests/kinds.bats` misses a defect in the code that `fault` adds.** That code runs only in the `fault` wheel.
- **`tests/kinds.bats` misses a line above the last two on the panic case's stderr.** It does not read that line.

## Confirmation

`mise run test:wheel` and the `wheel` job of Build run `tests/kinds.bats` on Linux and macOS.
A mutant files purba's own `PanicException` as `raised`.
`tests/kinds.bats` runs red under it.

```
sed -i 's/Ok(Err(payload)) => return Err(Error::caught(payload.as_ref(), "execution")),/Ok(Err(payload)) => return Ok(Outcome { kind: Kind::Raised { exception: Error::caught(payload.as_ref(), "execution").to_string() } }),/' crates/execution/src/lib.rs
mise run test:wheel
```

```
not ok 14 a panic stops only its test, each other test keeps its line, and the run exits 125
# (in test file tests/kinds.bats, line 76)
#   `run -125 --separate-stderr env -u PURBA_FAULT "${fault_purba}" test_panic.py' failed, expected exit code 125, got 1
# test_panic.py::test_passes passed
# test_panic.py::test_panics raised: PURBA_FAULT=assert
# test_panic.py::test_fails failed: assert 1 == 2
# test_panic.py::test_raises raised: KeyError: 'k'
# 4 tests: 1 passed, 1 failed, 2 raised
```

[A failure's kind is what the part that first sees it observes](a-failures-kind-is-what-the-part-that-first-sees-it-observes.md#decision-outcome) holds the kinds.
[A test causes an internal error through a switch that only a test build carries](a-test-causes-an-internal-error-through-a-switch-that-only-a-test-build-carries.md#decision-outcome) holds the `fault` wheel.
