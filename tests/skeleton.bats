#   the decision:  docs/decisions/bats-tests-the-installed-wheel-from-outside-purba.md
#
# bash 3.2 on macOS fails a test at a `[[ ]]` only on its last line, so `run -N`
# checks each exit code, and `|| false` ends each other check.
# https://bats-core.readthedocs.io/en/stable/gotchas.html#or-did-not-fail-my-test

: "${BATS_TEST_DIRNAME:?set by bats}"
bats_require_minimum_version 1.5.0

@test "purba runs test_a.py through every part, and never imports test_b.py" {
  export PURBA_SENTINEL="${BATS_TEST_TMPDIR:?set by bats}/test_b.imported"
  cd "${BATS_TEST_DIRNAME}/skeleton"

  run -1 --separate-stderr purba test_a.py

  diff -u - <(printf '%s\n' "${output}") <<'LINES'
test_a.py::test_one passed
test_a.py::test_two failed: assert 1 == 2
test_a.py::test_three unknown: no assert ran
3 tests: 1 passed, 1 failed, 0 raised, 1 unknown, 0 blocked; 0 errors
LINES
  [[ ! -e "${PURBA_SENTINEL}" ]]
}

@test "purba with no path exits 3, and says why on stderr" {
  run -3 --separate-stderr purba

  [[ "${stderr}" == "purba: usage: purba <path to one .py file>" ]] || false
  [[ -z "${output}" ]]
}

@test "purba with stdout closed exits 3, and says why on stderr" {
  cd "${BATS_TEST_DIRNAME}/skeleton"

  run -3 --separate-stderr bash -c 'purba test_a.py >&-'

  [[ "${stderr}" == "purba: cannot write to stdout: Bad file descriptor (os error 9)" ]]
}

@test "purba with stdout open for reading exits 3, and says why on stderr" {
  cd "${BATS_TEST_DIRNAME}/skeleton"

  run -3 --separate-stderr bash -c 'purba test_a.py 1</dev/null'

  [[ "${stderr}" == "purba: cannot write to stdout: Bad file descriptor (os error 9)" ]]
}

@test "a test that aborts purba leaves the line of each test before it" {
  cd "${BATS_TEST_TMPDIR:?set by bats}"
  printf '%s\n' \
    'import os' \
    'def test_one():' '    assert 1 == 1' \
    'def test_two():' '    os.abort()' \
    'def test_three():' '    assert 1 == 1' >test_abort.py

  run -134 --separate-stderr purba test_abort.py

  [[ "${output}" == "test_abort.py::test_one passed" ]]
}

@test "a test that selection refuses prints before any test runs, and is never called" {
  cd "${BATS_TEST_TMPDIR:?set by bats}"
  printf '%s\n' \
    'def test_d():' '    assert True' \
    'async def test_a():' '    ...' \
    'def test_b():' '    yield' \
    'def test_c(db: int):' '    ...' \
    'def test_e():' '    assert True' '    return False' \
    'def keep(function):' '    return function' \
    '@keep' 'def test_f():' '    assert False' \
    'def test_g():' '    assert False' \
    'test_g = keep(test_g)' >test_refused.py

  run -4 --separate-stderr purba test_refused.py

  local no_default='the parameter db has no default, and purba cannot see which fixture builds it'
  local rebound='line 19 binds test_g again, and purba cannot see what the name then holds'
  diff -u - <(printf '%s\n' "${output}") <<LINES
test_refused.py::test_a error: purba cannot run an async test yet
test_refused.py::test_b error: the test is a generator, and purba does not iterate it
test_refused.py::test_c error: ${no_default}
test_refused.py::test_e error: the test returns a value, and purba does not read it
test_refused.py::test_f error: the decorator keep is not purba's, and purba cannot see what it does
test_refused.py::test_g error: ${rebound}
test_refused.py::test_d passed
7 tests: 1 passed, 0 failed, 0 raised, 0 unknown, 6 blocked; 6 errors
LINES
  [[ -z "${stderr}" ]]
}

@test "a module that did not load prints its error once, and each of its tests as blocked" {
  cd "${BATS_TEST_TMPDIR:?set by bats}"
  printf '%s\n' \
    'raise ValueError("x")' \
    'def test_one():' '    assert 1 == 1' \
    'def test_two():' '    assert 1 == 2' >test_load.py

  run -4 --separate-stderr purba test_load.py

  diff -u - <(printf '%s\n' "${output}") <<'LINES'
test_load.py error: ValueError: x
test_load.py::test_one blocked: test_load.py did not load
test_load.py::test_two blocked: test_load.py did not load
2 tests: 0 passed, 0 failed, 0 raised, 0 unknown, 2 blocked; 1 error
LINES
  [[ -z "${stderr}" ]]
}

@test "a module that does not parse prints its error once, and no test of it" {
  cd "${BATS_TEST_TMPDIR:?set by bats}"
  printf '%s\n' \
    'def test_ok():' '    assert True' \
    'def test_bad1(:' '    pass' \
    'def test_bad2():' '    assert 1 == 1' >test_paren.py

  run -4 --separate-stderr purba test_paren.py

  diff -u - <(printf '%s\n' "${output}") <<'LINES'
test_paren.py error: SyntaxError: Expected a parameter or the end of the parameter list
0 tests: 0 passed, 0 failed, 0 raised, 0 unknown, 0 blocked; 1 error
LINES
  [[ -z "${stderr}" ]]
}
