#   the decision:
# docs/decisions/a-stand-in-enters-a-test-as-an-argument.md
#
# Each patch and each mock with no spec that selection refuses, and a patch that a
# suppression lets run, in tests/patch. The case matches the exit code and the
# whole stdout and stderr.
#
# bash 3.2 on macOS fails a test at a `[[ ]]` only on its last line, so `run -N`
# checks each exit code, and `|| false` ends each other check.
# https://bats-core.readthedocs.io/en/stable/gotchas.html#or-did-not-fail-my-test

: "${BATS_TEST_DIRNAME:?set by bats}"
bats_require_minimum_version 1.5.0

@test "a patch and a mock with no spec are refused, and a suppressed patch runs" {
  cd "${BATS_TEST_DIRNAME}/patch"

  run -4 --separate-stderr env PYTHONPATH=. purba test_patch.py

  local argument='and a stand-in enters a test as an argument'
  local spec='makes a mock with no spec, and a mock has the spec of what it stands for'
  diff -u - <(printf '%s\n' "${output}") <<LINES
test_patch.py::test_patch_the_code_under_test project error: \
[patched-name] line 9 patches lending.borrow, ${argument}; [mock-without-spec] line 9 ${spec}
test_patch.py::test_mock project error: [mock-without-spec] line 27 ${spec}
test_patch.py::test_borrow passed
    ⚠ suppressed [patched-name] line 21 patches lending.Library, ${argument}
      reason: the library has no seam yet
3 cases: 1 passed, 2 blocked; 2 project errors; 1 suppressed
LINES
  [[ -z "${stderr}" ]]
}
