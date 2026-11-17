#   the decision:
# docs/decisions/a-test-carries-only-purbas-own-decorators.md
#
# A test that the module defines twice, in the module of tests/twice. The case
# matches the exit code and the whole stdout and stderr.
#
# bash 3.2 on macOS fails a test at a `[[ ]]` only on its last line, so `run -N`
# checks each exit code, and `|| false` ends each other check.
# https://bats-core.readthedocs.io/en/stable/gotchas.html#or-did-not-fail-my-test

: "${BATS_TEST_DIRNAME:?set by bats}"
bats_require_minimum_version 1.5.0

@test "a test defined twice is refused, and a suppression above its first def lifts it" {
  cd "${BATS_TEST_DIRNAME}/twice"

  run -4 --separate-stderr purba twice.py

  local holds='and purba cannot see what the name then holds'
  diff -u - <(printf '%s\n' "${output}") <<LINES
twice.py::test_top project error: [name-bound-again] line 5 binds test_top again, ${holds}
twice.py::test_under project error: [name-bound-again] line 14 binds test_under again, ${holds}
twice.py::Box::test_twice project error: purba cannot call a method of a class
twice.py::test_suppressed passed
    ⚠ suppressed [name-bound-again] line 23 binds test_suppressed again, ${holds}
      reason: the second def replaces a draft
twice.py::test_once passed
5 cases: 2 passed, 3 blocked; 3 project errors; 1 suppressed
LINES
  [[ -z "${stderr}" ]]
}
