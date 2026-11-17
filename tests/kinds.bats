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
      expected  2
      reading   1
1 case: 1 failed
LINES
  [[ -z "${stderr}" ]]
}

@test "an expectation inside a checking context fails, and an assert in a try is refused" {
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

  local act='[assert-before-expectation] line 21: an assert ran in the act; one test checks one'
  act+=' state; check this state in a test of its own'
  diff -u - <(printf '%s\n' "${output}") <<LINES
test_cleanup_assert.py::test_inside_a_checking_context failed
    assert 1 + 1 == 3
      expected  3
      reading   2
test_cleanup_assert.py::test_with_a_finally_check project error: ${act}
2 cases: 1 failed, 1 blocked; 1 project error
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
test_none.py::test_one project error: [no-expectation] no expectation stated
1 case: 1 blocked; 1 project error
LINES
  [[ -z "${stderr}" ]]
}

@test "a test whose expectation never runs prints a project error, and the run exits 4" {
  cd "${BATS_TEST_TMPDIR:?set by bats}"
  printf '%s\n' 'def test_one():' '    return' '    assert 1 == 1' >test_early.py

  run -4 --separate-stderr purba test_early.py

  diff -u - <(printf '%s\n' "${output}") <<'LINES'
test_early.py::test_one project error: [no-expectation] no expectation ran
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
      expected  1
      reading   2
    assert b == 2
      expected  2
      reading   3
1 case: 1 failed
LINES
  [[ -z "${stderr}" ]]
}

@test "a failed expectation shows its expected value and its reading, and the run exits 1" {
  cd "${BATS_TEST_TMPDIR:?set by bats}"
  cat >reading.py <<'PY'
class One:
    def __repr__(self):
        return "1"


class NoRepr:
    def __repr__(self):
        raise ValueError("no repr")


def test_eq():
    got = [x for x in [1, -3, 2] if x <= 0]
    assert got == []


def test_lt():
    got = 3
    assert got < 2


def test_contains():
    got = "ab"
    assert "z" in got


def test_none():
    got = [-1]
    assert got is None


def test_same_text():
    got = One()
    assert got == 1


def test_no_repr():
    got = NoRepr()
    assert got == 1


def test_two():
    got = [-1]
    assert got == []
    assert len(got) == 0


def test_long():
    got = list(range(-30, 0))
    assert got == []
PY

  run -1 --separate-stderr purba reading.py

  local long=-30 n
  for ((n = -29; n <= -1; n++)); do long+=", ${n}"; done
  diff -u - <(printf '%s\n' "${output}") <<LINES
reading.py::test_eq failed
    assert got == []
      expected  []
      reading   [-3]
reading.py::test_lt failed
    assert got < 2
      expected  < 2
      reading   3
reading.py::test_contains failed
    assert "z" in got
      expected  to contain 'z'
      reading   'ab'
reading.py::test_none failed
    assert got is None
      expected  is None
      reading   [-1]
reading.py::test_same_text failed
    assert got == 1
      expected  1 (int)
      reading   1 (One)
reading.py::test_no_repr failed
    assert got == 1
      expected  1
      reading   (repr raised ValueError: no repr)
reading.py::test_two failed
    assert got == []
      expected  []
      reading   [-1]
    assert len(got) == 0
      expected  0
      reading   1
reading.py::test_long failed
    assert got == []
      expected  []
      reading   [${long}]
8 cases: 8 failed
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
test_reading.py::test_holds_alone project error: [readings-differ] assert seen == [] ${alone}
test_reading.py::test_fails_alone project error: [readings-differ] assert seen == [1] ${after}
2 cases: 2 blocked; 2 project errors
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
    'def test_passes():' '    assert 1 == 1' '' '' \
    '# purba: suppress[patched-name] the test sets the switch that makes purba panic' \
    'def test_panics():' \
    '    os.environ["PURBA_FAULT"] = "assert"' \
    '    try:' '        assert 1 == 1' \
    '    finally:' '        del os.environ["PURBA_FAULT"]' '' '' \
    'def test_fails():' '    assert 1 == 2' '' '' \
    'def test_raises():' '    raise KeyError("k")' >test_panic.py

  local fault_purba="${FAULT_PURBA:?set by mise run test:wheel and the wheel job}"
  run -125 --separate-stderr env -u PURBA_FAULT "${fault_purba}" test_panic.py

  local argument='and a stand-in enters a test as an argument'
  local switch='patches os.environ["PURBA_FAULT"]'
  diff -u - <(printf '%s\n' "${output}") <<LINES
test_panic.py::test_passes passed
test_panic.py::test_panics internal error: assertion: PURBA_FAULT=assert
    ⚠ suppressed [patched-name] line 10 ${switch}, ${argument}; line 14 ${switch}, ${argument}
      reason: the test sets the switch that makes purba panic
test_panic.py::test_fails failed
    assert 1 == 2
      expected  2
      reading   1
test_panic.py::test_raises raised: KeyError: 'k'
4 cases: 1 passed, 1 failed, 1 raised, 1 blocked; 1 internal error; 1 suppressed
LINES
  local expected last
  expected=$(report_lines "${fault_purba}")
  last=$(tail -n 2 <<<"${stderr}")
  diff -u <(printf '%s\n' "${expected}") <(printf '%s\n' "${last}")
}
