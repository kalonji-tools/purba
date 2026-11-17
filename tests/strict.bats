#   the decision:
# docs/decisions/a-test-returns-nothing-and-the-first-part-that-sees-a-breach-refuses-it.md
#
# The rules of the experiment shape that purba keeps, and those it relaxes, in the module
# of tests/strict. The case matches the exit code and the whole stdout and stderr.
#
# bash 3.2 on macOS fails a test at a `[[ ]]` only on its last line, so `run -N`
# checks each exit code, and `|| false` ends each other check.
# https://bats-core.readthedocs.io/en/stable/gotchas.html#or-did-not-fail-my-test

: "${BATS_TEST_DIRNAME:?set by bats}"
bats_require_minimum_version 1.5.0

@test "a private name is reported, four reads check, a try takes one path, a callback is free" {
  cd "${BATS_TEST_DIRNAME}/strict"

  run -1 --separate-stderr purba strict.py

  local bare='[not-one-comparison] line 33 is not one comparison, and an expectation compares'
  bare+=' a reading by one operator, or reads not x, isinstance(x, T), x.startswith(y)'
  bare+=' or x.endswith(y)'
  local input='[computed-expected] line 66 compares to the input expected, and an expected'
  input+=' value is a literal until purba has cases'
  local filters='[hook-replaced] line 47 replaced the warning filters, and a test keeps the'
  filters+=' filters that the run began with'
  local act='[assert-before-expectation] line 59: an assert ran in the act; one test checks'
  act+=' one state; check this state in a test of its own'
  diff -u - <(printf '%s\n' "${output}") <<LINES
strict.py::test_bare project error: ${bare}
strict.py::test_input project error: ${input}
strict.py::test_private passed
    ⚠ line 12 reads the private name _default_encoder
strict.py::test_empty passed
strict.py::test_kind failed
    assert isinstance(got, str)
      expected  an instance of str
      reading   1 (int)
strict.py::test_prefix passed
strict.py::test_try failed
    the try at line 38 expects ValueError, and the statement did not raise it
    assert seen == ["handled"]
      expected  ['handled']
      reading   []
strict.py::test_filters project error: ${filters}
strict.py::test_callback passed
strict.py::test_act_assert project error: ${act}
10 cases: 4 passed, 2 failed, 4 blocked; 4 project errors
LINES
  [[ -z "${stderr}" ]]
}
