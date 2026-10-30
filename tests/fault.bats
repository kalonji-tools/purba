#   the decision:
# docs/decisions/a-test-causes-an-internal-error-through-a-switch-that-only-a-test-build-carries.md
#
# `FAULT_PURBA` names the purba of the wheel built with the feature `fault`.

: "${BATS_TEST_DIRNAME:?set by bats}"
bats_require_minimum_version 1.5.0

@test "the fault wheel panics in the assert hook when PURBA_FAULT is assert" {
  cd "${BATS_TEST_DIRNAME}/fault"

  local fault_purba="${FAULT_PURBA:?set by mise run test:wheel and the wheel job}"
  run -3 --separate-stderr env PURBA_FAULT=assert "${fault_purba}" test_pass.py

  [[ -z "${output}" ]] || false
  grep -Fq -- 'panicked at crates/assertion/src/lib.rs:' <<<"${stderr}"
  grep -Fxq -- 'PURBA_FAULT=assert' <<<"${stderr}"
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
