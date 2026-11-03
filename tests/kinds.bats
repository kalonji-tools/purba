#   the decision:
# docs/decisions/a-case-of-each-kind-checks-that-no-failure-prints-as-another-kind.md
#
# One case of each kind, in one run for each case. Each case matches the exit
# code and the whole stdout. Each case but the panic matches the whole stderr.
#
# bash 3.2 on macOS fails a test at a `[[ ]]` only on its last line, so `run -N`
# checks each exit code, and `|| false` ends each other check.
# https://bats-core.readthedocs.io/en/stable/gotchas.html#or-did-not-fail-my-test

: "${BATS_TEST_DIRNAME:?set by bats}"
bats_require_minimum_version 1.5.0
load internal_error.sh

@test "a test whose assert fails prints failed, and the run exits 1" {
  cd "${BATS_TEST_TMPDIR:?set by bats}"
  printf '%s\n' 'def test_one():' '    assert 1 == 2' >test_fail.py

  run -1 --separate-stderr purba test_fail.py

  diff -u - <(printf '%s\n' "${output}") <<'LINES'
test_fail.py::test_one failed: assert 1 == 2
1 test: 1 failed
LINES
  [[ -z "${stderr}" ]]
}

@test "a test that raises prints raised, and the run exits 4" {
  cd "${BATS_TEST_TMPDIR:?set by bats}"
  printf '%s\n' 'def test_one():' '    raise KeyError("k")' >test_raise.py

  run -4 --separate-stderr purba test_raise.py

  diff -u - <(printf '%s\n' "${output}") <<'LINES'
test_raise.py::test_one raised: KeyError: 'k'
1 test: 1 raised
LINES
  [[ -z "${stderr}" ]]
}

@test "a module that does not parse prints a project error, and the run exits 4" {
  cd "${BATS_TEST_TMPDIR:?set by bats}"
  printf '%s\n' 'def test_one(:' '    pass' >test_paren.py

  run -4 --separate-stderr purba test_paren.py

  diff -u - <(printf '%s\n' "${output}") <<'LINES'
test_paren.py project error: SyntaxError: Expected a parameter or the end of the parameter list
0 tests; 1 project error
LINES
  [[ -z "${stderr}" ]]
}

@test "an argument that purba cannot take prints an invocation error, and the run exits 3" {
  cd "${BATS_TEST_TMPDIR:?set by bats}"

  run -3 --separate-stderr purba test_a.txt

  [[ -z "${output}" ]] || false
  [[ "${stderr}" == "purba: invocation error: usage: purba <path to one .py file>" ]]
}

@test "a panic stops only its test, each other test keeps its line, and the run exits 125" {
  cd "${BATS_TEST_TMPDIR:?set by bats}"
  printf '%s\n' \
    'import os' '' '' \
    'def test_passes():' '    assert True' '' '' \
    'def test_panics():' \
    '    os.environ["PURBA_FAULT"] = "assert"' \
    '    try:' '        assert True' \
    '    finally:' '        del os.environ["PURBA_FAULT"]' '' '' \
    'def test_fails():' '    assert 1 == 2' '' '' \
    'def test_raises():' '    raise KeyError("k")' >test_panic.py

  local fault_purba="${FAULT_PURBA:?set by mise run test:wheel and the wheel job}"
  run -125 --separate-stderr env -u PURBA_FAULT "${fault_purba}" test_panic.py

  diff -u - <(printf '%s\n' "${output}") <<'LINES'
test_panic.py::test_passes passed
test_panic.py::test_panics internal error: assertion: PURBA_FAULT=assert
test_panic.py::test_fails failed: assert 1 == 2
test_panic.py::test_raises raised: KeyError: 'k'
4 tests: 1 passed, 1 failed, 1 raised, 1 blocked; 1 internal error
LINES
  local expected last
  expected=$(report_lines "${fault_purba}")
  last=$(tail -n 2 <<<"${stderr}")
  diff -u <(printf '%s\n' "${expected}") <(printf '%s\n' "${last}")
}
