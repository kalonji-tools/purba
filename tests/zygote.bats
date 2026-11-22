#   the decision:  docs/decisions/each-test-runs-in-a-process-of-its-own.md
#   the decision:  docs/decisions/a-worker-loads-each-module-once.md
#   the decision:  docs/decisions/bats-tests-the-installed-wheel-from-outside-purba.md
#
# bash 3.2 on macOS fails a test at a `[[ ]]` only on its last line, so `run -N`
# checks each exit code, and `|| false` ends each other check.
# https://bats-core.readthedocs.io/en/stable/gotchas.html#or-did-not-fail-my-test

: "${BATS_TEST_DIRNAME:?set by bats}"
bats_require_minimum_version 1.5.0

@test "a global that a test changes keeps the value of the import in the next test" {
  cd "${BATS_TEST_TMPDIR:?set by bats}"
  printf '%s\n' \
    'value = "import"' \
    'def test_a():' '    global value' '    value = "test_a"' '    assert value == "test_a"' \
    'def test_b():' '    assert value == "import"' >test_global.py

  run -0 --separate-stderr purba test_global.py

  diff -u - <(printf '%s\n' "${output}") <<'LINES'
test_global.py::test_a passed
test_global.py::test_b passed
2 cases: 2 passed
LINES
}

@test "a test whose process exits 0 before its outcome prints raised, and the run exits 4" {
  cd "${BATS_TEST_TMPDIR:?set by bats}"
  printf '%s\n' 'import posix' 'def exit_now(code):' '    posix._exit(code)' \
    'def test_one():' '    exit_now(0)' \
    'def test_two():' '    assert 1 == 1' >test_exit.py

  run -4 --separate-stderr purba test_exit.py

  diff -u - <(printf '%s\n' "${output}") <<'LINES'
test_exit.py::test_one raised: its process exited 0 before its outcome
test_exit.py::test_two passed
2 cases: 1 passed, 1 raised
LINES
}

@test "a module body runs once for all its tests" {
  cd "${BATS_TEST_TMPDIR:?set by bats}"
  printf '%s\n' 'from pathlib import Path' \
    'with Path("body.log").open("a") as log:' '    log.write("body\n")' \
    'def test_one():' '    assert 1 == 1' \
    'def test_two():' '    assert 1 == 1' >test_body.py

  run -0 --separate-stderr purba test_body.py

  local lines
  lines=$(wc -l <body.log)
  [[ "${lines}" -eq 1 ]]
}

# The zygote loads the module once, and finds the thread. Each test then loads
# the module again in its own process.
@test "a module body that leaves a thread alive runs again for each test" {
  cd "${BATS_TEST_TMPDIR:?set by bats}"
  printf '%s\n' 'import threading' 'from pathlib import Path' \
    'with Path("body.log").open("a") as log:' '    log.write("body\n")' \
    'threading.Thread(target=threading.Event().wait, daemon=True).start()' \
    'def test_one():' '    assert 1 == 1' \
    'def test_two():' '    assert 1 == 1' >test_body.py

  run -0 --separate-stderr purba test_body.py

  [[ "${output}" == *$'\n2 cases: 2 passed' ]] || false
  local lines
  lines=$(wc -l <body.log)
  [[ "${lines}" -eq 3 ]]
}

# The thread makes the zygote replay the second expectation, and the replay kills
# the zygote. The case raises, so an end of the zygote that read as a replay it
# could not fork would print raised.
@test "a zygote that dies during a replay gives the case a project error, and the run exits 4" {
  cd "${BATS_TEST_TMPDIR:?set by bats}"
  printf '%s\n' 'import os, signal, threading' 'from pathlib import Path' \
    'def boom():' '    raise ValueError("boom")' \
    'def test_one():' \
    '    threading.Thread(target=threading.Event().wait, daemon=True).start()' \
    '    replay = Path("ran").exists()' '    Path("ran").touch()' \
    '    if replay:' '        os.kill(os.getppid(), signal.SIGKILL)' \
    '    assert 1 == 1' '    assert boom() is True' >test_kill.py

  run -4 --separate-stderr purba test_kill.py

  diff -u - <(printf '%s\n' "${output}") <<'LINES'
test_kill.py::test_one project error: the module's process died by SIGKILL
1 case: 1 blocked; 1 project error
LINES
}

@test "a module whose body reads the proxies passes a test that reads them again" {
  cd "${BATS_TEST_TMPDIR:?set by bats}"
  printf '%s\n' 'import urllib.request' \
    'urllib.request.getproxies()' \
    'def test_one():' \
    '    assert isinstance(urllib.request.getproxies(), dict) is True' >test_proxies.py

  run -0 --separate-stderr purba test_proxies.py

  diff -u - <(printf '%s\n' "${output}") <<'LINES'
test_proxies.py::test_one passed
1 case: 1 passed
LINES
}
