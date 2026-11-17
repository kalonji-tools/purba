#   the decision:
# docs/decisions/a-case-of-each-strict-check-shows-its-refusal-and-its-departure.md
#
# One test for each strict check, in the modules of tests/strictness. Its first run
# sends the refused case, and its second run sends the twin, which carries a
# suppression. A refusal with no name, the private name and the helpers have one
# run each. Each run matches the exit code and the whole stdout and stderr.
#
# The environment of the wheel installs purba alone, so `pytest.py` stands in for
# pytest when the twin of foreign-framework loads.
#
# bash 3.2 on macOS fails a test at a `[[ ]]` only on its last line, so `run -N`
# checks each exit code, and `|| false` ends each other check.
# https://bats-core.readthedocs.io/en/stable/gotchas.html#or-did-not-fail-my-test

: "${BATS_TEST_DIRNAME:?set by bats}"
bats_require_minimum_version 1.5.0

@test "[foreign-decorator] is refused, and a suppression with a reason lets the test pass" {
  cd "${BATS_TEST_DIRNAME}/strictness"

  run -4 --separate-stderr purba test_foreign_decorator.py

  local refused='[foreign-decorator] the decorator keep is not purba'\''s, and purba cannot see'
  refused+=' what it does'
  diff -u - <(printf '%s\n' "${output}") <<LINES || false
test_foreign_decorator.py::test_refused project error: ${refused}
1 case: 1 blocked; 1 project error
LINES
  [[ -z "${stderr}" ]] || false

  run -0 --separate-stderr purba test_foreign_decorator_suppressed.py

  local twin='    ⚠ suppressed [foreign-decorator] the decorator keep is not purba'\''s, and'
  twin+=' purba cannot see what it does'
  diff -u - <(printf '%s\n' "${output}") <<LINES || false
test_foreign_decorator_suppressed.py::test_suppressed passed
${twin}
      reason: this case shows the departure
1 case: 1 passed; 1 suppressed
LINES
  [[ -z "${stderr}" ]]
}

@test "[foreign-framework] is refused, and a suppression with a reason lets the test pass" {
  cd "${BATS_TEST_DIRNAME}/strictness"

  run -4 --separate-stderr purba test_foreign_framework.py

  local refused='[foreign-framework] line 1 imports pytest, and a test module imports no'
  refused+=' other test framework'
  diff -u - <(printf '%s\n' "${output}") <<LINES || false
test_foreign_framework.py::test_refused project error: ${refused}
1 case: 1 blocked; 1 project error
LINES
  [[ -z "${stderr}" ]] || false

  run -0 --separate-stderr env PYTHONPATH=. purba test_foreign_framework_suppressed.py

  local twin='    ⚠ suppressed [foreign-framework] line 1 imports pytest, and a test'
  twin+=' module imports no other test framework'
  diff -u - <(printf '%s\n' "${output}") <<LINES || false
test_foreign_framework_suppressed.py::test_suppressed passed
${twin}
      reason: this case shows the departure
1 case: 1 passed; 1 suppressed
LINES
  [[ -z "${stderr}" ]]
}

@test "[name-bound-again] is refused, and a suppression with a reason lets the test pass" {
  cd "${BATS_TEST_DIRNAME}/strictness"

  run -4 --separate-stderr purba test_name_bound_again.py

  local refused='[name-bound-again] line 5 binds test_refused again, and purba cannot see'
  refused+=' what the name then holds'
  diff -u - <(printf '%s\n' "${output}") <<LINES || false
test_name_bound_again.py::test_refused project error: ${refused}
1 case: 1 blocked; 1 project error
LINES
  [[ -z "${stderr}" ]] || false

  run -0 --separate-stderr purba test_name_bound_again_suppressed.py

  local twin='    ⚠ suppressed [name-bound-again] line 6 binds test_suppressed again, and'
  twin+=' purba cannot see what the name then holds'
  diff -u - <(printf '%s\n' "${output}") <<LINES || false
test_name_bound_again_suppressed.py::test_suppressed passed
${twin}
      reason: this case shows the departure
1 case: 1 passed; 1 suppressed
LINES
  [[ -z "${stderr}" ]]
}

@test "[not-one-comparison] is refused, and a suppression with a reason lets the test pass" {
  cd "${BATS_TEST_DIRNAME}/strictness"

  run -4 --separate-stderr purba test_not_one_comparison.py

  local refused='[not-one-comparison] line 3 is not one comparison, and an expectation'
  refused+=' compares a reading by one operator, or reads not x, isinstance(x, T),'
  refused+=' x.startswith(y) or x.endswith(y)'
  diff -u - <(printf '%s\n' "${output}") <<LINES || false
test_not_one_comparison.py::test_refused project error: ${refused}
1 case: 1 blocked; 1 project error
LINES
  [[ -z "${stderr}" ]] || false

  run -0 --separate-stderr purba test_not_one_comparison_suppressed.py

  local twin='    ⚠ suppressed [not-one-comparison] line 4 is not one comparison, and an'
  twin+=' expectation compares a reading by one operator, or reads not x,'
  twin+=' isinstance(x, T), x.startswith(y) or x.endswith(y)'
  diff -u - <(printf '%s\n' "${output}") <<LINES || false
test_not_one_comparison_suppressed.py::test_suppressed passed
${twin}
      reason: this case shows the departure
1 case: 1 passed; 1 suppressed
LINES
  [[ -z "${stderr}" ]]
}

@test "[computed-expected] is refused, and a suppression with a reason lets the test pass" {
  cd "${BATS_TEST_DIRNAME}/strictness"

  run -4 --separate-stderr purba test_computed_expected.py

  local refused='[computed-expected] line 6 computes N, and an expected value is a literal'
  diff -u - <(printf '%s\n' "${output}") <<LINES || false
test_computed_expected.py::test_refused project error: ${refused}
1 case: 1 blocked; 1 project error
LINES
  [[ -z "${stderr}" ]] || false

  run -0 --separate-stderr purba test_computed_expected_suppressed.py

  diff -u - <(printf '%s\n' "${output}") <<'LINES' || false
test_computed_expected_suppressed.py::test_suppressed passed
    ⚠ suppressed [computed-expected] line 7 computes N, and an expected value is a literal
      reason: this case shows the departure
1 case: 1 passed; 1 suppressed
LINES
  [[ -z "${stderr}" ]]
}

@test "[second-path] is refused, and a suppression with a reason lets the test pass" {
  cd "${BATS_TEST_DIRNAME}/strictness"

  run -4 --separate-stderr purba test_second_path.py

  local refused='[second-path] line 3 went one way only, and the lines before the first'
  refused+=' expectation take one path'
  diff -u - <(printf '%s\n' "${output}") <<LINES || false
test_second_path.py::test_refused project error: ${refused}
1 case: 1 blocked; 1 project error
LINES
  [[ -z "${stderr}" ]] || false

  run -0 --separate-stderr purba test_second_path_suppressed.py

  local twin='    ⚠ suppressed [second-path] line 4 went one way only, and the lines'
  twin+=' before the first expectation take one path'
  diff -u - <(printf '%s\n' "${output}") <<LINES || false
test_second_path_suppressed.py::test_suppressed passed
${twin}
      reason: this case shows the departure
1 case: 1 passed; 1 suppressed
LINES
  [[ -z "${stderr}" ]]
}

@test "[assert-before-expectation] is refused, and a suppression with a reason lets the test pass" {
  cd "${BATS_TEST_DIRNAME}/strictness"

  run -4 --separate-stderr purba test_assert_before_expectation.py

  local refused='[assert-before-expectation] line 3: an assert ran in the act; one test'
  refused+=' checks one state; check this state in a test of its own'
  diff -u - <(printf '%s\n' "${output}") <<LINES || false
test_assert_before_expectation.py::test_refused project error: ${refused}
1 case: 1 blocked; 1 project error
LINES
  [[ -z "${stderr}" ]] || false

  run -0 --separate-stderr purba test_assert_before_expectation_suppressed.py

  local twin='an assert ran in the act; one test checks one state; check this state in a'
  twin+=' test of its own'
  diff -u - <(printf '%s\n' "${output}") <<LINES || false
test_assert_before_expectation_suppressed.py::test_suppressed passed
    ⚠ suppressed [assert-before-expectation] line 4: ${twin}
      reason: this case shows the departure
1 case: 1 passed; 1 suppressed
LINES
  [[ -z "${stderr}" ]]
}

@test "[swallowed-raise] is refused, and a suppression with a reason lets the test pass" {
  cd "${BATS_TEST_DIRNAME}/strictness"

  run -4 --separate-stderr purba test_swallowed_raise.py

  local refused='[swallowed-raise] the with at line 5 swallowed a raise, and line 7 did not'
  refused+=' run'
  diff -u - <(printf '%s\n' "${output}") <<LINES || false
test_swallowed_raise.py::test_refused project error: ${refused}
1 case: 1 blocked; 1 project error
LINES
  [[ -z "${stderr}" ]] || false

  run -0 --separate-stderr purba test_swallowed_raise_suppressed.py

  diff -u - <(printf '%s\n' "${output}") <<'LINES' || false
test_swallowed_raise_suppressed.py::test_suppressed passed
    ⚠ suppressed [swallowed-raise] the with at line 6 swallowed a raise, and line 8 did not run
      reason: this case shows the departure
1 case: 1 passed; 1 suppressed
LINES
  [[ -z "${stderr}" ]]
}

@test "[patched-name] is refused, and a suppression with a reason lets the test pass" {
  cd "${BATS_TEST_DIRNAME}/strictness"

  run -4 --separate-stderr purba test_patched_name.py

  local refused='[patched-name] line 5 patches os.environ["PURBA_PROBE"], and a stand-in'
  refused+=' enters a test as an argument'
  diff -u - <(printf '%s\n' "${output}") <<LINES || false
test_patched_name.py::test_refused project error: ${refused}
1 case: 1 blocked; 1 project error
LINES
  [[ -z "${stderr}" ]] || false

  run -0 --separate-stderr purba test_patched_name_suppressed.py

  local twin='    ⚠ suppressed [patched-name] line 6 patches os.environ["PURBA_PROBE"],'
  twin+=' and a stand-in enters a test as an argument'
  diff -u - <(printf '%s\n' "${output}") <<LINES || false
test_patched_name_suppressed.py::test_suppressed passed
${twin}
      reason: this case shows the departure
1 case: 1 passed; 1 suppressed
LINES
  [[ -z "${stderr}" ]]
}

@test "[mock-without-spec] is refused, and a suppression with a reason lets the test pass" {
  cd "${BATS_TEST_DIRNAME}/strictness"

  run -4 --separate-stderr purba test_mock_without_spec.py

  local refused='[mock-without-spec] line 5 makes a mock with no spec, and a mock has the'
  refused+=' spec of what it stands for'
  diff -u - <(printf '%s\n' "${output}") <<LINES || false
test_mock_without_spec.py::test_refused project error: ${refused}
1 case: 1 blocked; 1 project error
LINES
  [[ -z "${stderr}" ]] || false

  run -0 --separate-stderr purba test_mock_without_spec_suppressed.py

  local twin='    ⚠ suppressed [mock-without-spec] line 6 makes a mock with no spec, and a'
  twin+=' mock has the spec of what it stands for'
  diff -u - <(printf '%s\n' "${output}") <<LINES || false
test_mock_without_spec_suppressed.py::test_suppressed passed
${twin}
      reason: this case shows the departure
1 case: 1 passed; 1 suppressed
LINES
  [[ -z "${stderr}" ]]
}

@test "[iterator-read] is refused, and a suppression with a reason lets the test pass" {
  cd "${BATS_TEST_DIRNAME}/strictness"

  run -4 --separate-stderr purba test_iterator_read.py

  local refused='[iterator-read] line 3 reads it, an iterator that one reading empties; make'
  refused+=' it a list in the act'
  diff -u - <(printf '%s\n' "${output}") <<LINES || false
test_iterator_read.py::test_refused project error: ${refused}
1 case: 1 blocked; 1 project error
LINES
  [[ -z "${stderr}" ]] || false

  run -0 --separate-stderr purba test_iterator_read_suppressed.py

  local twin='    ⚠ suppressed [iterator-read] line 4 reads it, an iterator that one'
  twin+=' reading empties; make it a list in the act'
  diff -u - <(printf '%s\n' "${output}") <<LINES || false
test_iterator_read_suppressed.py::test_suppressed passed
${twin}
      reason: this case shows the departure
1 case: 1 passed; 1 suppressed
LINES
  [[ -z "${stderr}" ]]
}

@test "[type-alone] is refused, and a suppression with a reason lets the test pass" {
  cd "${BATS_TEST_DIRNAME}/strictness"

  run -4 --separate-stderr purba test_type_alone.py

  local refused='[type-alone] line 13 expects Closed by its type alone; bind except Closed as'
  refused+=' e: and check str(e)'
  diff -u - <(printf '%s\n' "${output}") <<LINES || false
test_type_alone.py::test_refused project error: ${refused}
1 case: 1 blocked; 1 project error
LINES
  [[ -z "${stderr}" ]] || false

  run -0 --separate-stderr purba test_type_alone_suppressed.py

  local twin='    ⚠ suppressed [type-alone] line 14 expects Closed by its type alone; bind'
  twin+=' except Closed as e: and check str(e)'
  diff -u - <(printf '%s\n' "${output}") <<LINES || false
test_type_alone_suppressed.py::test_suppressed passed
${twin}
      reason: this case shows the departure
1 case: 1 passed; 1 suppressed
LINES
  [[ -z "${stderr}" ]]
}

@test "[no-expectation] is refused, and an outcome cannot stand without it" {
  cd "${BATS_TEST_DIRNAME}/strictness"

  run -4 --separate-stderr purba test_no_expectation.py

  diff -u - <(printf '%s\n' "${output}") <<'LINES' || false
test_no_expectation.py::test_refused project error: [no-expectation] no expectation stated
1 case: 1 blocked; 1 project error
LINES
  [[ -z "${stderr}" ]] || false

  run -4 --separate-stderr purba test_no_expectation_suppressed.py

  local twin='line 1 suppresses no-expectation, and an outcome cannot stand without that'
  twin+=' check'
  diff -u - <(printf '%s\n' "${output}") <<LINES || false
test_no_expectation_suppressed.py::test_suppressed project error: ${twin}
1 case: 1 blocked; 1 project error
LINES
  [[ -z "${stderr}" ]]
}

@test "[readings-differ] is refused, and an outcome cannot stand without it" {
  cd "${BATS_TEST_DIRNAME}/strictness"

  run -4 --separate-stderr purba test_readings_differ.py

  local refused='[readings-differ] assert seen == [1] holds after the expectations before it,'
  refused+=' and not when read alone'
  diff -u - <(printf '%s\n' "${output}") <<LINES || false
test_readings_differ.py::test_refused project error: ${refused}
1 case: 1 blocked; 1 project error
LINES
  [[ -z "${stderr}" ]] || false

  run -4 --separate-stderr purba test_readings_differ_suppressed.py

  local twin='line 4 suppresses readings-differ, and an outcome cannot stand without that'
  twin+=' check'
  diff -u - <(printf '%s\n' "${output}") <<LINES || false
test_readings_differ_suppressed.py::test_suppressed project error: ${twin}
1 case: 1 blocked; 1 project error
LINES
  [[ -z "${stderr}" ]]
}

@test "[assert-in-reading] is refused, and an outcome cannot stand without it" {
  cd "${BATS_TEST_DIRNAME}/strictness"

  run -4 --separate-stderr purba test_assert_in_reading.py

  local refused='[assert-in-reading] assert check() is True reaches an assert when read'
  refused+=' alone, and only an expectation asserts'
  diff -u - <(printf '%s\n' "${output}") <<LINES || false
test_assert_in_reading.py::test_refused project error: ${refused}
1 case: 1 blocked; 1 project error
LINES
  [[ -z "${stderr}" ]] || false

  run -4 --separate-stderr purba test_assert_in_reading_suppressed.py

  local twin='line 10 suppresses assert-in-reading, and an outcome cannot stand without'
  twin+=' that check'
  diff -u - <(printf '%s\n' "${output}") <<LINES || false
test_assert_in_reading_suppressed.py::test_suppressed project error: ${twin}
1 case: 1 blocked; 1 project error
LINES
  [[ -z "${stderr}" ]]
}

@test "[hook-replaced] is refused, and an outcome cannot stand without it" {
  cd "${BATS_TEST_DIRNAME}/strictness"

  run -4 --separate-stderr purba test_hook_replaced.py

  local refused='[hook-replaced] line 9 replaced sys.unraisablehook, and a test keeps purba'\''s'
  refused+=' hooks'
  diff -u - <(printf '%s\n' "${output}") <<LINES || false
test_hook_replaced.py::test_refused project error: ${refused}
1 case: 1 blocked; 1 project error
LINES
  [[ -z "${stderr}" ]] || false

  run -4 --separate-stderr purba test_hook_replaced_suppressed.py

  local twin='line 8 suppresses hook-replaced, and an outcome cannot stand without that'
  twin+=' check'
  diff -u - <(printf '%s\n' "${output}") <<LINES || false
test_hook_replaced_suppressed.py::test_suppressed project error: ${twin}
1 case: 1 blocked; 1 project error
LINES
  [[ -z "${stderr}" ]]
}

@test "[no-item] is refused, and an outcome cannot stand without it" {
  cd "${BATS_TEST_DIRNAME}/strictness"

  run -4 --separate-stderr purba test_no_item.py

  local refused='[no-item] line 3 checks each item of xs, and xs holds no item'
  diff -u - <(printf '%s\n' "${output}") <<LINES || false
test_no_item.py::test_refused project error: ${refused}
1 case: 1 blocked; 1 project error
LINES
  [[ -z "${stderr}" ]] || false

  run -4 --separate-stderr purba test_no_item_suppressed.py

  local twin='line 1 suppresses no-item, and an outcome cannot stand without that check'
  diff -u - <(printf '%s\n' "${output}") <<LINES || false
test_no_item_suppressed.py::test_suppressed project error: ${twin}
1 case: 1 blocked; 1 project error
LINES
  [[ -z "${stderr}" ]]
}

@test "[def-under-block] is refused, and a suppression of it cannot help" {
  cd "${BATS_TEST_DIRNAME}/strictness"

  run -4 --separate-stderr purba test_def_under_block.py

  local under='[def-under-block] the def sits under'
  local unseen='and purba cannot see whether import binds it'
  diff -u - <(printf '%s\n' "${output}") <<LINES || false
test_def_under_block.py::test_refused project error: ${under} \`if\`, ${unseen}
1 case: 1 blocked; 1 project error
LINES
  [[ -z "${stderr}" ]] || false

  run -4 --separate-stderr purba test_def_under_block_suppressed.py

  local fault='line 4 suppresses def-under-block, and a suppression cannot help: purba'
  fault+=' reads no expectation of such a test;'
  diff -u - <(printf '%s\n' "${output}") <<LINES || false
test_def_under_block_suppressed.py::test_suppressed project error: \
${fault} ${under} \`if\`, ${unseen}
1 case: 1 blocked; 1 project error
LINES
  [[ -z "${stderr}" ]]
}

@test "[name-without-def] is refused, and a suppression of it cannot help" {
  cd "${BATS_TEST_DIRNAME}/strictness"

  run -4 --separate-stderr purba test_name_without_def.py

  local refused='[name-without-def] line 5 binds test_refused with no def, and purba cannot'
  refused+=' see what the name holds'
  diff -u - <(printf '%s\n' "${output}") <<LINES || false
test_name_without_def.py::test_refused project error: ${refused}
1 case: 1 blocked; 1 project error
LINES
  [[ -z "${stderr}" ]] || false

  run -4 --separate-stderr purba test_name_without_def_suppressed.py

  local twin='[name-without-def] line 6 binds test_suppressed with no def, and purba'
  twin+=' cannot see what the name holds'
  local fault='line 1 suppresses name-without-def, and a suppression cannot help: purba'
  fault+=' reads no expectation of such a test'
  diff -u - <(printf '%s\n' "${output}") <<LINES || false
test_name_without_def_suppressed.py::test_suppressed project error: ${twin}
test_name_without_def_suppressed.py project error: ${fault}
1 case: 1 blocked; 2 project errors
LINES
  [[ -z "${stderr}" ]]
}

@test "[returned-value] is refused, and a suppression of it cannot help" {
  cd "${BATS_TEST_DIRNAME}/strictness"

  run -4 --separate-stderr purba test_returned_value.py

  local refused='[returned-value] the test returns a value, and purba does not read it'
  diff -u - <(printf '%s\n' "${output}") <<LINES || false
test_returned_value.py::test_refused project error: ${refused}
1 case: 1 blocked; 1 project error
LINES
  [[ -z "${stderr}" ]] || false

  run -4 --separate-stderr purba test_returned_value_suppressed.py

  local twin='line 1 suppresses returned-value, and a suppression cannot help: purba reads'
  twin+=' no expectation of such a test; [returned-value] the test returns a value,'
  twin+=' and purba does not read it'
  diff -u - <(printf '%s\n' "${output}") <<LINES || false
test_returned_value_suppressed.py::test_suppressed project error: ${twin}
1 case: 1 blocked; 1 project error
LINES
  [[ -z "${stderr}" ]]
}

@test "an async test is refused with no name" {
  cd "${BATS_TEST_DIRNAME}/strictness"

  run -4 --separate-stderr purba test_async.py

  diff -u - <(printf '%s\n' "${output}") <<'LINES' || false
test_async.py::test_async project error: purba cannot run an async test yet
1 case: 1 blocked; 1 project error
LINES
  [[ -z "${stderr}" ]]
}

@test "a test that is a generator is refused with no name" {
  cd "${BATS_TEST_DIRNAME}/strictness"

  run -4 --separate-stderr purba test_generator.py

  local line='the test is a generator, and purba does not iterate it'
  diff -u - <(printf '%s\n' "${output}") <<LINES || false
test_generator.py::test_generator project error: ${line}
1 case: 1 blocked; 1 project error
LINES
  [[ -z "${stderr}" ]]
}

@test "a parameter with no default is refused with no name" {
  cd "${BATS_TEST_DIRNAME}/strictness"

  run -4 --separate-stderr purba test_param.py

  local line='the parameter x has no default, and purba cannot see which fixture builds it'
  diff -u - <(printf '%s\n' "${output}") <<LINES || false
test_param.py::test_param project error: ${line}
1 case: 1 blocked; 1 project error
LINES
  [[ -z "${stderr}" ]]
}

@test "a test in a class is refused with no name" {
  cd "${BATS_TEST_DIRNAME}/strictness"

  run -4 --separate-stderr purba test_class.py

  diff -u - <(printf '%s\n' "${output}") <<'LINES' || false
test_class.py::TestBox::test_method project error: purba cannot call a method of a class
1 case: 1 blocked; 1 project error
LINES
  [[ -z "${stderr}" ]]
}

@test "a read of a private name passes, with a line under the test" {
  cd "${BATS_TEST_DIRNAME}/strictness"

  run -0 --separate-stderr purba test_private.py

  diff -u - <(printf '%s\n' "${output}") <<'LINES' || false
test_private.py::test_private passed
    ⚠ line 5 reads the private name _default_encoder
1 case: 1 passed
LINES
  [[ -z "${stderr}" ]]
}

@test "a test that checks through a helper is refused" {
  cd "${BATS_TEST_DIRNAME}/strictness"

  run -4 --separate-stderr purba test_helper.py

  local line='[assert-before-expectation] line 17 reaches line 9 of check, which ran an'
  line+=' assert in the act; one test checks one state; check this state in a test of'
  line+=' its own'
  diff -u - <(printf '%s\n' "${output}") <<LINES || false
test_helper.py::test_unittest_helper project error: [no-expectation] no expectation stated
test_helper.py::test_own_helper project error: ${line}
2 cases: 2 blocked; 2 project errors
LINES
  [[ -z "${stderr}" ]]
}
