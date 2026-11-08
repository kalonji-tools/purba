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
3 tests: 1 passed, 1 failed, 1 unknown
LINES
  [[ ! -e "${PURBA_SENTINEL}" ]]
}

@test "purba with no path exits 3, and says why on stderr" {
  run -3 --separate-stderr purba

  [[ "${stderr}" == "purba: invocation error: usage: purba <path to one .py file>" ]] || false
  [[ -z "${output}" ]]
}

@test "purba with stdout closed exits 3, and says why on stderr" {
  cd "${BATS_TEST_DIRNAME}/skeleton"

  run -3 --separate-stderr bash -c 'purba test_a.py >&-'

  local refused='purba: invocation error: cannot write to stdout: Bad file descriptor (os error 9)'
  [[ "${stderr}" == "${refused}" ]]
}

@test "purba with stdout open for reading exits 3, and says why on stderr" {
  cd "${BATS_TEST_DIRNAME}/skeleton"

  run -3 --separate-stderr bash -c 'purba test_a.py 1</dev/null'

  local refused='purba: invocation error: cannot write to stdout: Bad file descriptor (os error 9)'
  [[ "${stderr}" == "${refused}" ]]
}

@test "purba with no sys.abiflags runs its tests" {
  local site="${BATS_TEST_TMPDIR:?set by bats}/site"
  mkdir "${site}"
  printf '%s\n' 'import sys' '' 'del sys.abiflags' >"${site}/sitecustomize.py"
  cd "${BATS_TEST_DIRNAME}/skeleton"

  run -1 --separate-stderr env PYTHONPATH="${site}" purba test_a.py

  diff -u - <(printf '%s\n' "${output}") <<'LINES'
test_a.py::test_one passed
test_a.py::test_two failed: assert 1 == 2
test_a.py::test_three unknown: no assert ran
3 tests: 1 passed, 1 failed, 1 unknown
LINES
  [[ -z "${stderr}" ]]
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

@test "a test that selection refuses prints above the tests that run, and is never called" {
  cd "${BATS_TEST_TMPDIR:?set by bats}"
  printf '%s\n' \
    'def test_d():' '    assert True' \
    'async def test_a():' '    ...' >test_refused.py

  run -4 --separate-stderr purba test_refused.py

  diff -u - <(printf '%s\n' "${output}") <<'LINES'
test_refused.py::test_a project error: purba cannot run an async test yet
test_refused.py::test_d passed
2 tests: 1 passed, 1 blocked; 1 project error
LINES
  [[ -z "${stderr}" ]]
}

@test "a test under a statement at the module's level prints as refused" {
  cd "${BATS_TEST_TMPDIR:?set by bats}"
  printf '%s\n' \
    'import sys' \
    'if sys.version_info >= (3, 0):' '    def test_inside_if():' '        assert 1 == 2' \
    'try:' '    def test_inside_try():' '        assert 1 == 2' \
    'except Exception:' '    pass' \
    'def test_top():' '    assert True' >test_nested.py

  run -4 --separate-stderr purba test_nested.py

  local unseen='and purba cannot see whether import binds it'
  diff -u - <(printf '%s\n' "${output}") <<LINES
test_nested.py::test_inside_if project error: the def sits under \`if\`, ${unseen}
test_nested.py::test_inside_try project error: the def sits under \`try\`, ${unseen}
test_nested.py::test_top passed
3 tests: 1 passed, 2 blocked; 2 project errors
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
test_load.py project error: ValueError: x
test_load.py::test_one blocked: test_load.py did not load
test_load.py::test_two blocked: test_load.py did not load
2 tests: 2 blocked; 1 project error
LINES
  [[ -z "${stderr}" ]]
}

@test "a thread's exception in a test prints a project error, and the test keeps its outcome" {
  cd "${BATS_TEST_TMPDIR:?set by bats}"
  printf '%s\n' \
    'import threading' \
    'def test_thread():' \
    '    t = threading.Thread(target=lambda: 1/0, name="T"); t.start(); t.join()' \
    '    assert True' >test_thread.py

  run -4 --separate-stderr purba test_thread.py

  diff -u - <(printf '%s\n' "${output}") <<'LINES'
test_thread.py::test_thread passed
test_thread.py::test_thread project error: thread T raised ZeroDivisionError: division by zero
1 test: 1 passed; 1 project error
LINES
  [[ "${stderr}" == *"Traceback"*"ZeroDivisionError: division by zero"* ]] || false
}

@test "a thread that outlives its test prints its error under the test that left it" {
  cd "${BATS_TEST_TMPDIR:?set by bats}"
  printf '%s\n' \
    'import threading' \
    'go = threading.Event()' \
    'left = []' \
    'def late():' '    go.wait()' '    raise ValueError("late")' \
    'def test_a():' \
    '    thread = threading.Thread(target=late, name="T", daemon=True)' \
    '    thread.start()' '    left.append(thread)' '    assert True' \
    'def test_b():' '    go.set()' '    left[0].join()' '    assert True' >test_late.py

  run -4 --separate-stderr purba test_late.py

  diff -u - <(printf '%s\n' "${output}") <<'LINES'
test_late.py::test_a passed
test_late.py::test_b passed
test_late.py::test_a project error: thread T raised ValueError: late
2 tests: 2 passed; 1 project error
LINES
}

@test "a module that does not parse prints its error once, and no test of it" {
  cd "${BATS_TEST_TMPDIR:?set by bats}"
  printf '%s\n' \
    'def test_ok():' '    assert True' \
    'def test_bad1(:' '    pass' \
    'def test_bad2():' '    assert 1 == 1' >test_paren.py

  run -4 --separate-stderr purba test_paren.py

  diff -u - <(printf '%s\n' "${output}") <<'LINES'
test_paren.py project error: SyntaxError: Expected a parameter or the end of the parameter list
0 tests; 1 project error
LINES
  [[ -z "${stderr}" ]]
}
