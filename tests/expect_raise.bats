#   the decision:
# docs/decisions/a-test-that-expects-a-raise-names-more-than-its-type.md
#
# Each except that only passes, which selection refuses as type-alone, a try of one path
# whose except binds the raise, and a suppression of type-alone, in tests/expect_raise.
# Each case matches the exit code and the whole stdout and stderr.
#
# bash 3.2 on macOS fails a test at a `[[ ]]` only on its last line, so `run -N`
# checks each exit code.
# https://bats-core.readthedocs.io/en/stable/gotchas.html#or-did-not-fail-my-test

: "${BATS_TEST_DIRNAME:?set by bats}"
bats_require_minimum_version 1.5.0

@test "an except that only passes is refused, and an except that binds the raise is one path" {
  cd "${BATS_TEST_DIRNAME}/expect_raise"

  run -1 --separate-stderr purba test_expect_raise.py

  local fix='as e: and check str(e)'
  local closed="expects Closed by its type alone; bind except Closed ${fix}"
  diff -u - <(printf '%s\n' "${output}") <<LINES
test_expect_raise.py::test_pass project error: [type-alone] line 17 ${closed}
test_expect_raise.py::test_ellipsis project error: [type-alone] line 26 ${closed}
test_expect_raise.py::test_tuple project error: [type-alone] line 35 \
expects (Closed, KeyError) by its type alone; bind except (Closed, KeyError) ${fix}
test_expect_raise.py::test_bare project error: [type-alone] line 44 \
expects any raise; bind except SomeError ${fix}
test_expect_raise.py::test_two_statements project error: [type-alone] line 54 ${closed}
test_expect_raise.py::test_message passed
test_expect_raise.py::test_quiet failed
    the try at line 70 expects Closed, and the statement did not raise it
    assert got == "the library is closed"
      expected  'the library is closed'
      reading   None
test_expect_raise.py::test_suppressed passed
    ⚠ suppressed [type-alone] line 82 ${closed}
      reason: the type is the whole contract of lend
8 cases: 2 passed, 1 failed, 5 blocked; 5 project errors; 1 suppressed
LINES
  [[ -z "${stderr}" ]]
}
