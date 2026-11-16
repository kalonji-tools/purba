#   the decision:  docs/decisions/a-test-module-keeps-its-asserts-under-o.md
#
# Each case matches the exit code and the whole stdout and stderr.
#
# bash 3.2 on macOS fails a test at a `[[ ]]` only on its last line, so `run -N`
# checks each exit code, and `|| false` ends each other check.
# https://bats-core.readthedocs.io/en/stable/gotchas.html#or-did-not-fail-my-test

: "${BATS_TEST_DIRNAME:?set by bats}"
bats_require_minimum_version 1.5.0

warning='purba: warning: Python runs with -O, so an assert outside a test module does not run'

@test "under -O, a test whose helper in the test module asserts is refused, and the run warns" {
  cd "${BATS_TEST_DIRNAME}/optimize"

  run -4 --separate-stderr env PYTHONOPTIMIZE=1 purba test_helper.py

  local refusal='[assert-before-expectation] line 6 reaches line 2 of check, '
  refusal+='which asserts before the first expectation, and only an expectation asserts'
  diff -u - <(printf '%s\n' "${output}") <<LINES || false
test_helper.py::test_helper project error: ${refusal}
1 case: 1 blocked; 1 project error
LINES
  [[ "${stderr}" == "${warning}" ]]
}

@test "under -O at level 256, a test whose helper asserts is refused, and the run warns" {
  cd "${BATS_TEST_DIRNAME}/optimize"

  run -4 --separate-stderr env PYTHONOPTIMIZE=256 purba test_helper.py

  local refusal='[assert-before-expectation] line 6 reaches line 2 of check, '
  refusal+='which asserts before the first expectation, and only an expectation asserts'
  diff -u - <(printf '%s\n' "${output}") <<LINES || false
test_helper.py::test_helper project error: ${refusal}
1 case: 1 blocked; 1 project error
LINES
  [[ "${stderr}" == "${warning}" ]]
}

@test "under -O, the warning comes before the error of a module that purba cannot read" {
  cd "${BATS_TEST_DIRNAME}/optimize"

  run -3 --separate-stderr env PYTHONOPTIMIZE=1 purba missing.py

  local refused='purba: invocation error: cannot read missing.py: '
  refused+='No such file or directory (os error 2)'
  [[ "${stderr}" == "${warning}"$'\n'"${refused}" ]] || false
  [[ -z "${output}" ]]
}

@test "under -O, the warning comes before the error of an argument that is not UTF-8" {
  cd "${BATS_TEST_DIRNAME}/optimize"

  run -3 --separate-stderr env PYTHONOPTIMIZE=1 purba $'x\xff.py'

  local refused="purba: invocation error: UnicodeEncodeError: 'utf-8' codec can't encode "
  refused+="character '\\udcff' in position 1: surrogates not allowed"
  [[ "${stderr}" == "${warning}"$'\n'"${refused}" ]] || false
  [[ -z "${output}" ]]
}

@test "under -O, the test module reads __debug__ as True" {
  cd "${BATS_TEST_DIRNAME}/optimize"

  run -0 --separate-stderr env PYTHONOPTIMIZE=1 purba test_debug.py

  [[ "${output}" == $'test_debug.py::test_debug passed\n1 case: 1 passed' ]] || false
  [[ "${stderr}" == "${warning}" ]]
}

@test "under -OO, the test module keeps its docstrings" {
  cd "${BATS_TEST_DIRNAME}/optimize"

  run -0 --separate-stderr env PYTHONOPTIMIZE=2 purba test_doc.py

  [[ "${output}" == $'test_doc.py::test_doc passed\n1 case: 1 passed' ]] || false
  [[ "${stderr}" == "${warning}" ]]
}

@test "under -O, a helper of another module loses its asserts, as the warning says" {
  cd "${BATS_TEST_DIRNAME}/optimize"

  run -0 --separate-stderr env PYTHONOPTIMIZE=1 PYTHONPATH=. purba test_sibling.py

  [[ "${output}" == $'test_sibling.py::test_sibling passed\n1 case: 1 passed' ]] || false
  [[ "${stderr}" == "${warning}" ]]
}

@test "a plain run prints no warning" {
  cd "${BATS_TEST_DIRNAME}/optimize"

  run -0 --separate-stderr env -u PYTHONOPTIMIZE purba test_debug.py

  [[ "${output}" == $'test_debug.py::test_debug passed\n1 case: 1 passed' ]] || false
  [[ -z "${stderr}" ]]
}
