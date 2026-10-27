#   the decision:  docs/decisions/bats-tests-the-installed-wheel-from-outside-purba.md
#
# bash 3.2 on macOS fails a test at a `[[ ]]` only on its last line, so `run -N`
# checks each exit code, and `|| false` ends each other check.
# https://bats-core.readthedocs.io/en/stable/gotchas.html#or-did-not-fail-my-test

: "${BATS_TEST_DIRNAME:?set by bats}"
bats_require_minimum_version 1.5.0

@test "purba runs test_a.py through every part, and never imports test_b.py" {
  export PURBA_SENTINEL="${BATS_TEST_TMPDIR:?set by bats}/test_b.imported"
  cd "${BATS_TEST_DIRNAME}/skeleton"

  run -1 --separate-stderr purba test_a.py

  diff -u - <(printf '%s\n' "${output}") <<'LINES'
test_a.py::test_one passed
test_a.py::test_two failed: assert 1 == 2
test_a.py::test_three unknown: no assert ran
3 tests: 1 passed, 1 failed, 0 raised, 1 unknown, 0 errors
LINES
  [[ ! -e "${PURBA_SENTINEL}" ]]
}

@test "purba with no path exits 3, and says why on stderr" {
  run -3 --separate-stderr purba

  [[ "${stderr}" == "purba: usage: purba <path to one .py file>" ]] || false
  [[ -z "${output}" ]]
}

@test "purba with stdout closed exits 3, and says why on stderr" {
  cd "${BATS_TEST_DIRNAME}/skeleton"

  run -3 --separate-stderr bash -c 'purba test_a.py >&-'

  [[ "${stderr}" == "purba: cannot write to stdout: Bad file descriptor (os error 9)" ]]
}

@test "purba with stdout open for reading exits 3, and says why on stderr" {
  cd "${BATS_TEST_DIRNAME}/skeleton"

  run -3 --separate-stderr bash -c 'purba test_a.py 1</dev/null'

  [[ "${stderr}" == "purba: cannot write to stdout: Bad file descriptor (os error 9)" ]]
}
