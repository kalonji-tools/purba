#   the decision:
# docs/decisions/a-test-module-imports-no-other-test-framework.md
#
# Each import of pytest that selection refuses, and an import that a suppression
# lets run, in tests/framework. Each case matches the exit code and the whole
# stdout and stderr.
#
# The environment of the wheel installs purba alone, so `pytest.py` stands in for
# pytest when `test_suppressed.py` loads.
#
# bash 3.2 on macOS fails a test at a `[[ ]]` only on its last line, so `run -N`
# checks each exit code, and `|| false` ends each other check.
# https://bats-core.readthedocs.io/en/stable/gotchas.html#or-did-not-fail-my-test

: "${BATS_TEST_DIRNAME:?set by bats}"
bats_require_minimum_version 1.5.0

@test "an import of pytest outside a test reaches every test, and one in a test reaches that test" {
  cd "${BATS_TEST_DIRNAME}/framework"

  run -4 --separate-stderr purba test_framework.py

  local rule='and a test module imports no other test framework'
  local monkeypatch="line 13 imports _pytest.monkeypatch, ${rule}"
  diff -u - <(printf '%s\n' "${output}") <<LINES
test_framework.py::test_price project error: \
[foreign-framework] line 5 imports pytest, ${rule}; [foreign-framework] ${monkeypatch}
test_framework.py::test_call project error: \
[foreign-framework] line 5 imports pytest, ${rule}; [foreign-framework] ${monkeypatch}; \
[foreign-framework] line 23 imports pytest, ${rule}
test_framework.py::test_suppressed passed
    ⚠ suppressed [foreign-framework] line 5 imports pytest, ${rule}; ${monkeypatch}
      reason: this test still runs under pytest
3 cases: 1 passed, 2 blocked; 2 project errors; 1 suppressed
LINES
  [[ -z "${stderr}" ]]
}

@test "a module that suppresses foreign-framework runs its tests" {
  cd "${BATS_TEST_DIRNAME}/framework"

  run -0 --separate-stderr env PYTHONPATH=. purba test_suppressed.py

  local rule='and a test module imports no other test framework'
  diff -u - <(printf '%s\n' "${output}") <<LINES
test_suppressed.py::test_price passed
    ⚠ suppressed [foreign-framework] line 2 imports pytest, ${rule}
      reason: the suite still runs under pytest
1 case: 1 passed; 1 suppressed
LINES
  [[ -z "${stderr}" ]]
}
