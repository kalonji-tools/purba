# PROTOTYPE for #535, branch `proto-535-wheel-test`. Never merges.
#
# Runs the installed `purba` on one module of a two-module project.
: "${BATS_TEST_DIRNAME:?set by bats}"

bats_require_minimum_version 1.5.0

setup() {
  : "${BATS_TEST_TMPDIR:?set by bats}"
  cd "${BATS_TEST_DIRNAME}/skeleton" || exit 1
  export PURBA_SENTINEL="${BATS_TEST_TMPDIR}/test_b-was-imported"
}

# One test, because the sentinel alone passes when purba never ran at all.
@test "purba runs test_a.py alone, prints one line for each test, and exits 1" {
  run --separate-stderr purba test_a.py

  expected='test_a.py::test_one passed
test_a.py::test_two failed: assert 1 == 2
test_a.py::test_three unknown: no assert ran
3 tests: 1 passed, 1 failed, 1 unknown'
  diff -u <(printf '%s\n' "${expected}") <(printf '%s\n' "${output}")

  [[ "${status}" -eq 1 ]]
  [[ ! -e "${PURBA_SENTINEL}" ]]
}
