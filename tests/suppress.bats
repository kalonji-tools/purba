#   the decision:
# docs/decisions/a-suppression-is-an-outcome-and-not-a-switch.md
#
# Each suppression that purba honours, and each that it refuses, in the modules of
# tests/suppress. Each case matches the exit code and the whole stdout and stderr.
#
# bash 3.2 on macOS fails a test at a `[[ ]]` only on its last line, so `run -N`
# checks each exit code, and `|| false` ends each other check.
# https://bats-core.readthedocs.io/en/stable/gotchas.html#or-did-not-fail-my-test

: "${BATS_TEST_DIRNAME:?set by bats}"
bats_require_minimum_version 1.5.0

@test "a suppressed check prints a warning under its case, and the count names it" {
  cd "${BATS_TEST_DIRNAME}/suppress"

  run -1 --separate-stderr purba suppress.py

  local fixture='the parameter tmp_path has no default'
  fixture+=', and purba cannot see which fixture builds it'
  local literal='and an expected value is a literal'
  local computed='⚠ suppressed [computed-expected]'
  local reason='reason: the payload is its own expected value'
  local digest='hashlib.sha256(b"abc").hexdigest()'
  local act='⚠ suppressed [assert-before-expectation] line 23: an assert ran in the act; one'
  act+=' test checks one state; check this state in a test of its own'
  local subset='⚠ suppressed [not-one-comparison] line 30 is not one comparison'
  subset+=', and an expectation compares a reading by one operator, or reads not x,'
  subset+=' isinstance(x, T), x.startswith(y) or x.endswith(y)'
  diff -u - <(printf '%s\n' "${output}") <<LINES
suppress.py::test_fixture project error: ${fixture}; \
[computed-expected] line 35 computes len("xy"), ${literal}
suppress.py::test_round_trip passed
    ${computed} line 10 computes obj, ${literal}
      ${reason}
suppress.py::test_round_trip_tuple failed
    assert json.loads(json.dumps(obj)) == obj
      expected  {'a': (1, 2)}
      reading   {'a': [1, 2]}
    ${computed} line 16 computes obj, ${literal}
      ${reason}
suppress.py::test_digest passed
    ${computed} line 25 computes ${digest}, ${literal}
      reason: both sides are digests of one payload
    ${act}
      reason: the act checks its own input
suppress.py::test_subset passed
    ${subset}
      reason: a subset reads best as one call
5 cases: 3 passed, 1 failed, 1 blocked; 1 project error; 5 suppressed
LINES
  [[ -z "${stderr}" ]]
}

@test "a test that selection refuses prints its suppressed warning under its line" {
  cd "${BATS_TEST_DIRNAME}/suppress"

  run -4 --separate-stderr purba refused.py

  local subset='[not-one-comparison] line 8 is not one comparison'
  subset+=', and an expectation compares a reading by one operator, or reads not x,'
  subset+=' isinstance(x, T), x.startswith(y) or x.endswith(y)'
  diff -u - <(printf '%s\n' "${output}") <<LINES
refused.py::test_refused project error: ${subset}
    ⚠ suppressed [computed-expected] line 7 computes obj, and an expected value is a literal
      reason: the payload is its own expected value
1 case: 1 blocked; 1 project error; 1 suppressed
LINES
  [[ -z "${stderr}" ]]
}

@test "a suppression that does not read, has no reason or names no check refuses its test" {
  cd "${BATS_TEST_DIRNAME}/suppress"

  run -4 --separate-stderr purba broken.py

  local help='and a suppression cannot help: purba reads no expectation of such a test'
  local returns='[returned-value] the test returns a value, and purba does not read it'
  local outcome='and an outcome cannot stand without that check'
  local unread='is a purba comment that reads as no suppression'
  unread+=', and a suppression reads # purba: suppress[<check>] <reason>'
  local computed='[computed-expected] line 20 computes obj, and an expected value is a literal'
  diff -u - <(printf '%s\n' "${output}") <<LINES
broken.py::test_returns project error: line 5 suppresses returned-value, ${help}; ${returns}
broken.py::test_logs project error: line 12 suppresses no-expectation, ${outcome}
broken.py::test_no_reason project error: line 17 suppresses computed-expected with no reason, \
and a suppression gives one; ${computed}
broken.py::test_unknown project error: line 23 suppresses slow-test, which is no strict check
broken.py::test_unread project error: line 29 ${unread}
5 cases: 5 blocked; 5 project errors
LINES
  [[ -z "${stderr}" ]]
}

@test "a suppression above a helper is the module's fault, and blocks each test" {
  cd "${BATS_TEST_DIRNAME}/suppress"

  run -4 --separate-stderr purba helper.py

  diff -u - <(printf '%s\n' "${output}") <<'LINES'
helper.py project error: line 5 suppresses above check_digest, which is no test
helper.py::test_digest blocked: helper.py did not load
1 case: 1 blocked; 1 project error
LINES
  [[ -z "${stderr}" ]]
}

@test "a suppression that purba judged and that suppresses nothing fails the run" {
  cd "${BATS_TEST_DIRNAME}/suppress"

  run -1 --separate-stderr purba unused.py

  local raised="raised: ValueError: invalid literal for int() with base 10: 'x'"
  diff -u - <(printf '%s\n' "${output}") <<LINES
unused.py::test_renamed passed
unused.py::test_failed failed
    assert value == 4
      expected  4
      reading   3
unused.py::test_raised ${raised}
unused.py project error: line 1 suppresses computed-expected, which no test of the module breaks
unused.py project error: line 4 suppresses not-one-comparison, which test_renamed does not break
unused.py project error: line 10 suppresses swallowed-raise, which test_failed does not break
3 cases: 1 passed, 1 failed, 1 raised; 3 project errors
LINES
  [[ -z "${stderr}" ]]
}

@test "a module suppression that one test uses, and one above an async test, print no line" {
  cd "${BATS_TEST_DIRNAME}/suppress"

  run -4 --separate-stderr purba judged.py

  local computed='⚠ suppressed [computed-expected] line 7 computes os.sep'
  computed+=', and an expected value is a literal'
  diff -u - <(printf '%s\n' "${output}") <<LINES
judged.py::test_async project error: purba cannot run an async test yet
judged.py::test_reads passed
    ${computed}
      reason: one test computes its expected value
judged.py::test_plain passed
3 cases: 2 passed, 1 blocked; 1 project error; 1 suppressed
LINES
  [[ -z "${stderr}" ]]
}
