#   the decision:
# docs/decisions/a-test-causes-an-internal-error-through-a-switch-that-only-a-test-build-carries.md
#
# `FAULT_PURBA` names the purba of the wheel built with the feature `fault`.

: "${BATS_TEST_DIRNAME:?set by bats}"
bats_require_minimum_version 1.5.0

# The last three lines of stderr after a panic that stops the run of `purba`,
# for `part` and `message`. Each value of the second line comes from the venv of
# `purba` and from the OS, and not from purba's code.
internal_error_lines() {
  local purba=$1 part=$2 message=$3
  local python="${purba%/*}/python"
  local version cpython system machine os arch
  version=$("${python}" -c 'import importlib.metadata; print(importlib.metadata.version("purba"))')
  cpython=$("${python}" -c 'import platform, sys
print(platform.python_version() + getattr(sys, "abiflags", ""))')
  system=$(uname -s)
  machine=$(uname -m)
  # Rust names each OS in lower case, macOS as macos, and Apple's arm64 as aarch64:
  # https://doc.rust-lang.org/1.99.1/std/env/consts/constant.OS.html
  # https://doc.rust-lang.org/1.99.1/std/env/consts/constant.ARCH.html
  case ${system} in
    Darwin) os=macos ;;
    Linux) os=linux ;;
    *) os=${system} ;;
  esac
  case ${machine} in
    arm64) arch=aarch64 ;;
    *) arch=${machine} ;;
  esac
  printf '%s\n' \
    "purba: internal error: ${part}: ${message}" \
    "purba: an internal error is a bug in purba ${version}, on CPython ${cpython}, ${os} ${arch}." \
    'purba: report it at https://github.com/kalonji-tools/purba/issues/new?template=defect.md'
}

@test "the fault wheel panics in the assert hook when PURBA_FAULT is assert" {
  cd "${BATS_TEST_DIRNAME}/fault"

  local fault_purba="${FAULT_PURBA:?set by mise run test:wheel and the wheel job}"
  run -125 --separate-stderr env PURBA_FAULT=assert "${fault_purba}" test_pass.py

  [[ -z "${output}" ]] || false
  grep -Fq -- 'panicked at crates/assertion/src/lib.rs:' <<<"${stderr}"
  grep -Fxq -- 'PURBA_FAULT=assert' <<<"${stderr}"
  local expected last
  expected=$(internal_error_lines "${fault_purba}" assertion PURBA_FAULT=assert)
  last=$(tail -n 3 <<<"${stderr}")
  diff -u <(printf '%s\n' "${expected}") <(printf '%s\n' "${last}")
}

@test "the fault wheel panics in command when PURBA_FAULT is command" {
  cd "${BATS_TEST_DIRNAME}/fault"

  local fault_purba="${FAULT_PURBA:?set by mise run test:wheel and the wheel job}"
  run -125 --separate-stderr env PURBA_FAULT=command "${fault_purba}" test_pass.py

  [[ -z "${output}" ]] || false
  grep -Fq -- 'panicked at crates/command/src/lib.rs:' <<<"${stderr}"
  local expected last
  expected=$(internal_error_lines "${fault_purba}" command PURBA_FAULT=command)
  last=$(tail -n 3 <<<"${stderr}")
  diff -u <(printf '%s\n' "${expected}") <(printf '%s\n' "${last}")
}

@test "the fault wheel passes the test when PURBA_FAULT is unset" {
  cd "${BATS_TEST_DIRNAME}/fault"

  local fault_purba="${FAULT_PURBA:?set by mise run test:wheel and the wheel job}"
  run -0 --separate-stderr env -u PURBA_FAULT "${fault_purba}" test_pass.py

  diff -u - <(printf '%s\n' "${output}") <<'LINES'
test_pass.py::test_one passed
1 test: 1 passed, 0 failed, 0 raised, 0 unknown, 0 blocked; 0 errors
LINES
  [[ -z "${stderr}" ]]
}

@test "the wheel without fault passes the test when PURBA_FAULT is assert" {
  cd "${BATS_TEST_DIRNAME}/fault"

  run -0 --separate-stderr env PURBA_FAULT=assert purba test_pass.py

  diff -u - <(printf '%s\n' "${output}") <<'LINES'
test_pass.py::test_one passed
1 test: 1 passed, 0 failed, 0 raised, 0 unknown, 0 blocked; 0 errors
LINES
  [[ -z "${stderr}" ]]
}

@test "the wheel without fault passes the test when PURBA_FAULT is command" {
  cd "${BATS_TEST_DIRNAME}/fault"

  run -0 --separate-stderr env PURBA_FAULT=command purba test_pass.py

  diff -u - <(printf '%s\n' "${output}") <<'LINES'
test_pass.py::test_one passed
1 test: 1 passed, 0 failed, 0 raised, 0 unknown, 0 blocked; 0 errors
LINES
  [[ -z "${stderr}" ]]
}

@test "the fault wheel names command for purba's own PanicException that a test raises" {
  cd "${BATS_TEST_DIRNAME}/fault"

  local fault_purba="${FAULT_PURBA:?set by mise run test:wheel and the wheel job}"
  run -125 --separate-stderr env -u PURBA_FAULT "${fault_purba}" test_panic_exception.py

  [[ -z "${output}" ]] || false
  local expected last
  expected=$(internal_error_lines "${fault_purba}" command 'raised by the test')
  last=$(tail -n 3 <<<"${stderr}")
  diff -u <(printf '%s\n' "${expected}") <(printf '%s\n' "${last}")
}

@test "the fault wheel names the panic that stops the run, and not one that a test caught" {
  cd "${BATS_TEST_DIRNAME}/fault"

  local fault_purba="${FAULT_PURBA:?set by mise run test:wheel and the wheel job}"
  run -125 --separate-stderr env PURBA_FAULT=assert "${fault_purba}" test_stale_note.py

  diff -u - <(printf '%s\n' "${output}") <<'LINES'
test_stale_note.py::test_catches unknown: no assert ran
LINES
  local expected last
  expected=$(internal_error_lines "${fault_purba}" command 'raised by the second test')
  last=$(tail -n 3 <<<"${stderr}")
  diff -u <(printf '%s\n' "${expected}") <(printf '%s\n' "${last}")
}
