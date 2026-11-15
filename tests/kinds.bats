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
test_fail.py::test_one failed
    assert 1 == 2
1 case: 1 failed
LINES
  [[ -z "${stderr}" ]]
}

@test "a test that fails before a cleanup assert that holds prints failed, and the run exits 1" {
  cd "${BATS_TEST_TMPDIR:?set by bats}"
  printf '%s\n' \
    'import contextlib' '' 'opened = []' '' '' \
    '@contextlib.contextmanager' 'def no_leaks():' '    try:' '        yield' \
    '    finally:' '        assert not opened, "a handle leaked"' '' '' \
    'def test_inside_a_checking_context():' '    with no_leaks():' \
    '        assert 1 + 1 == 3' '' '' \
    'def test_with_a_finally_check():' '    try:' '        assert 1 + 1 == 3' \
    '    finally:' '        assert not opened' >test_cleanup_assert.py

  run -1 --separate-stderr purba test_cleanup_assert.py

  diff -u - <(printf '%s\n' "${output}") <<'LINES'
test_cleanup_assert.py::test_inside_a_checking_context failed
    assert 1 + 1 == 3
test_cleanup_assert.py::test_with_a_finally_check failed
    assert 1 + 1 == 3
2 cases: 2 failed
LINES
  [[ -z "${stderr}" ]]
}

@test "a test that raises prints raised, and the run exits 4" {
  cd "${BATS_TEST_TMPDIR:?set by bats}"
  printf '%s\n' 'def test_one():' '    raise KeyError("k")' >test_raise.py

  run -4 --separate-stderr purba test_raise.py

  diff -u - <(printf '%s\n' "${output}") <<'LINES'
test_raise.py::test_one raised: KeyError: 'k'
1 case: 1 raised
LINES
  [[ -z "${stderr}" ]]
}

@test "a test that states no expectation prints a project error, and the run exits 4" {
  cd "${BATS_TEST_TMPDIR:?set by bats}"
  printf '%s\n' 'def test_one():' '    total = 1 + 1' >test_none.py

  run -4 --separate-stderr purba test_none.py

  diff -u - <(printf '%s\n' "${output}") <<'LINES'
test_none.py::test_one project error: no expectation stated
1 case: 1 blocked; 1 project error
LINES
  [[ -z "${stderr}" ]]
}

@test "an assert inside a with is no expectation, and the run exits 4" {
  cd "${BATS_TEST_TMPDIR:?set by bats}"
  printf '%s\n' 'import contextlib' '' '' \
    'def test_one():' '    with contextlib.nullcontext():' '        assert 1 == 1' >test_with.py

  run -4 --separate-stderr purba test_with.py

  diff -u - <(printf '%s\n' "${output}") <<'LINES'
test_with.py::test_one project error: no expectation stated
1 case: 1 blocked; 1 project error
LINES
  [[ -z "${stderr}" ]]
}

@test "a test whose expectation never runs prints a project error, and the run exits 4" {
  cd "${BATS_TEST_TMPDIR:?set by bats}"
  printf '%s\n' 'def test_one():' '    return' '    assert 1 == 1' >test_early.py

  run -4 --separate-stderr purba test_early.py

  diff -u - <(printf '%s\n' "${output}") <<'LINES'
test_early.py::test_one project error: no expectation ran
1 case: 1 blocked; 1 project error
LINES
  [[ -z "${stderr}" ]]
}

@test "a test whose two expectations fail prints both under its line, and the run exits 1" {
  cd "${BATS_TEST_TMPDIR:?set by bats}"
  printf '%s\n' 'def test_one():' '    a, b = 2, 3' '    assert a == 1' '    assert b == 2' \
    >test_two_fail.py

  run -1 --separate-stderr purba test_two_fail.py

  diff -u - <(printf '%s\n' "${output}") <<'LINES'
test_two_fail.py::test_one failed
    assert a == 1
    assert b == 2
1 case: 1 failed
LINES
  [[ -z "${stderr}" ]]
}

@test "an expectation that a reading before it changed is refused, and the run exits 4" {
  cd "${BATS_TEST_TMPDIR:?set by bats}"
  printf '%s\n' 'seen = []' '' '' \
    'def test_holds_alone():' '    assert seen.append(1) is None' '    assert seen == []' '' '' \
    'def test_fails_alone():' '    assert seen.append(1) is None' '    assert seen == [1]' \
    >test_reading.py

  run -4 --separate-stderr purba test_reading.py

  local alone='holds when read alone, and not after the expectations before it'
  local after='holds after the expectations before it, and not when read alone'
  diff -u - <(printf '%s\n' "${output}") <<LINES
test_reading.py::test_holds_alone project error: assert seen == [] ${alone}
test_reading.py::test_fails_alone project error: assert seen == [1] ${after}
2 cases: 2 blocked; 2 project errors
LINES
  [[ -z "${stderr}" ]]
}

@test "a thread alive at the first expectation leaves each expectation read in order only" {
  cd "${BATS_TEST_TMPDIR:?set by bats}"
  printf '%s\n' 'import threading' '' 'seen = []' '' '' 'def test_one():' \
    '    threading.Thread(target=threading.Event().wait, daemon=True).start()' \
    '    assert seen.append(1) is None' '    assert seen == [1]' >test_thread.py

  run -0 --separate-stderr purba test_thread.py

  diff -u - <(printf '%s\n' "${output}") <<'LINES'
test_thread.py::test_one passed
1 case: 1 passed
LINES
  [[ -z "${stderr}" ]]
}

@test "an expectation that raises gives raised, and no later expectation prints" {
  cd "${BATS_TEST_TMPDIR:?set by bats}"
  printf '%s\n' 'def test_one():' '    assert {}["k"] == 1' '    assert 1 == 2' >test_raise_first.py

  run -4 --separate-stderr purba test_raise_first.py

  diff -u - <(printf '%s\n' "${output}") <<'LINES'
test_raise_first.py::test_one raised: KeyError: 'k'
1 case: 1 raised
LINES
  [[ -z "${stderr}" ]]
}

@test "a module that does not parse prints a project error, and the run exits 4" {
  cd "${BATS_TEST_TMPDIR:?set by bats}"
  printf '%s\n' 'def test_one(:' '    pass' >test_paren.py

  run -4 --separate-stderr purba test_paren.py

  diff -u - <(printf '%s\n' "${output}") <<'LINES'
test_paren.py project error: SyntaxError: Expected a parameter or the end of the parameter list
0 cases; 1 project error
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
test_panic.py::test_fails failed
    assert 1 == 2
test_panic.py::test_raises raised: KeyError: 'k'
4 cases: 1 passed, 1 failed, 1 raised, 1 blocked; 1 internal error
LINES
  local expected last
  expected=$(report_lines "${fault_purba}")
  last=$(tail -n 2 <<<"${stderr}")
  diff -u <(printf '%s\n' "${expected}") <(printf '%s\n' "${last}")
}
