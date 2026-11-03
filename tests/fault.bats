#   the decision:
# docs/decisions/a-test-causes-an-internal-error-through-a-switch-that-only-a-test-build-carries.md
#
# `FAULT_PURBA` names the purba of the wheel built with the feature `fault`.

: "${BATS_TEST_DIRNAME:?set by bats}"
bats_require_minimum_version 1.5.0
load internal_error.sh

@test "the fault wheel panics in the assert hook when PURBA_FAULT is assert" {
  cd "${BATS_TEST_DIRNAME}/fault"

  local fault_purba="${FAULT_PURBA:?set by mise run test:wheel and the wheel job}"
  run -125 --separate-stderr env PURBA_FAULT=assert "${fault_purba}" test_pass.py

  diff -u - <(printf '%s\n' "${output}") <<'LINES'
test_pass.py::test_one internal error: assertion: PURBA_FAULT=assert
1 test: 1 blocked; 1 internal error
LINES
  grep -Fq -- 'panicked at crates/assertion/src/lib.rs:' <<<"${stderr}"
  grep -Fxq -- 'PURBA_FAULT=assert' <<<"${stderr}"
  local expected last
  expected=$(report_lines "${fault_purba}")
  last=$(tail -n 2 <<<"${stderr}")
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
1 test: 1 passed
LINES
  [[ -z "${stderr}" ]]
}

@test "the wheel without fault passes the test when PURBA_FAULT is assert" {
  cd "${BATS_TEST_DIRNAME}/fault"

  run -0 --separate-stderr env PURBA_FAULT=assert purba test_pass.py

  diff -u - <(printf '%s\n' "${output}") <<'LINES'
test_pass.py::test_one passed
1 test: 1 passed
LINES
  [[ -z "${stderr}" ]]
}

@test "the wheel without fault passes the test when PURBA_FAULT is command" {
  cd "${BATS_TEST_DIRNAME}/fault"

  run -0 --separate-stderr env PURBA_FAULT=command purba test_pass.py

  diff -u - <(printf '%s\n' "${output}") <<'LINES'
test_pass.py::test_one passed
1 test: 1 passed
LINES
  [[ -z "${stderr}" ]]
}

@test "the fault wheel names execution for purba's own PanicException that a test raises" {
  cd "${BATS_TEST_DIRNAME}/fault"

  local fault_purba="${FAULT_PURBA:?set by mise run test:wheel and the wheel job}"
  run -125 --separate-stderr env -u PURBA_FAULT "${fault_purba}" test_panic_exception.py

  diff -u - <(printf '%s\n' "${output}") <<'LINES'
test_panic_exception.py::test_raises_it internal error: execution: raised by the test
1 test: 1 blocked; 1 internal error
LINES
  local expected last
  expected=$(report_lines "${fault_purba}")
  last=$(tail -n 2 <<<"${stderr}")
  diff -u <(printf '%s\n' "${expected}") <(printf '%s\n' "${last}")
}

@test "a test that catches the panic still gets its internal error, and the next test its own" {
  cd "${BATS_TEST_DIRNAME}/fault"

  local fault_purba="${FAULT_PURBA:?set by mise run test:wheel and the wheel job}"
  run -125 --separate-stderr env PURBA_FAULT=assert "${fault_purba}" test_stale_note.py

  diff -u - <(printf '%s\n' "${output}") <<'LINES'
test_stale_note.py::test_catches internal error: assertion: PURBA_FAULT=assert
test_stale_note.py::test_raises internal error: execution: raised by the second test
2 tests: 2 blocked; 2 internal errors
LINES
  local expected last
  expected=$(report_lines "${fault_purba}")
  last=$(tail -n 2 <<<"${stderr}")
  diff -u <(printf '%s\n' "${expected}") <(printf '%s\n' "${last}")
}

@test "the fault wheel gives the module an internal error when PURBA_FAULT is selection" {
  cd "${BATS_TEST_DIRNAME}/fault"

  local fault_purba="${FAULT_PURBA:?set by mise run test:wheel and the wheel job}"
  run -125 --separate-stderr env PURBA_FAULT=selection "${fault_purba}" test_pass.py

  diff -u - <(printf '%s\n' "${output}") <<'LINES'
test_pass.py internal error: selection: PURBA_FAULT=selection
0 tests; 1 internal error
LINES
  grep -Fq -- 'panicked at crates/selection/src/lib.rs:' <<<"${stderr}"
  local expected last
  expected=$(report_lines "${fault_purba}")
  last=$(tail -n 2 <<<"${stderr}")
  diff -u <(printf '%s\n' "${expected}") <(printf '%s\n' "${last}")
}

@test "the wheel without fault passes the test when PURBA_FAULT is selection" {
  cd "${BATS_TEST_DIRNAME}/fault"

  run -0 --separate-stderr env PURBA_FAULT=selection purba test_pass.py

  diff -u - <(printf '%s\n' "${output}") <<'LINES'
test_pass.py::test_one passed
1 test: 1 passed
LINES
  [[ -z "${stderr}" ]]
}
