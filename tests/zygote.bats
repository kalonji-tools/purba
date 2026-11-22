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
# the zygote. The replay still sends what it read, so the order decides the case,
# and its call raised. The tests after it print as blocked, and one line of the
# module names the end.
@test "a zygote that dies during a replay that sends its verdict blocks each later test" {
  cd "${BATS_TEST_TMPDIR:?set by bats}"
  printf '%s\n' 'import os, signal, threading' 'from pathlib import Path' \
    'def boom():' '    raise ValueError("boom")' \
    'def test_one():' \
    '    threading.Thread(target=threading.Event().wait, daemon=True).start()' \
    '    replay = Path("ran").exists()' '    Path("ran").touch()' \
    '    if replay:' '        os.kill(os.getppid(), signal.SIGKILL)' \
    '    assert 1 == 1' '    assert boom() is True' \
    'def test_two():' '    assert 1 == 1' \
    'def test_three():' '    assert 2 == 2' >test_kill.py

  run -4 --separate-stderr purba test_kill.py

  diff -u - <(printf '%s\n' "${output}") <<'LINES'
test_kill.py::test_one raised: ValueError: boom
    raised in boom() at test_kill.py:4
test_kill.py project error: the module's process died by SIGKILL
test_kill.py::test_two blocked: the module's process died by SIGKILL
test_kill.py::test_three blocked: the module's process died by SIGKILL
3 cases: 1 raised, 2 blocked; 1 project error
LINES
}

# The thread makes the zygote replay the second expectation. The test's process
# kills the zygote at its exit, after it sent the facts, so no replay reads the
# second expectation alone.
@test "a zygote that dies before a replay gives the case the end, and blocks each later test" {
  cd "${BATS_TEST_TMPDIR:?set by bats}"
  printf '%s\n' 'import atexit, os, signal, threading' \
    'def test_one():' \
    '    threading.Thread(target=threading.Event().wait, daemon=True).start()' \
    '    atexit.register(os.kill, os.getppid(), signal.SIGKILL)' \
    '    assert 1 == 1' '    assert 2 == 2' \
    'def test_two():' '    assert 1 == 1' >test_late.py

  run -4 --separate-stderr purba test_late.py

  diff -u - <(printf '%s\n' "${output}") <<'LINES'
test_late.py::test_one project error: the module's process died by SIGKILL
test_late.py::test_two blocked: the module's process died by SIGKILL
2 cases: 2 blocked; 1 project error
LINES
}

# As the case of the replay that kills the zygote, but each expectation holds.
# So the case passes only when purba keeps the verdict that the replay sent
# after the zygote died.
@test "a zygote that dies during a replay keeps the verdict that the replay sends" {
  cd "${BATS_TEST_TMPDIR:?set by bats}"
  printf '%s\n' 'import os, signal, threading' 'from pathlib import Path' \
    'def test_one():' \
    '    threading.Thread(target=threading.Event().wait, daemon=True).start()' \
    '    replay = Path("ran").exists()' '    Path("ran").touch()' \
    '    os.kill(os.getppid(), [0, signal.SIGKILL][replay])' \
    '    assert 1 == 1' '    assert 2 == 2' \
    'def test_two():' '    assert 1 == 1' >test_hold.py

  run -4 --separate-stderr purba test_hold.py

  diff -u - <(printf '%s\n' "${output}") <<'LINES'
test_hold.py::test_one passed
test_hold.py project error: the module's process died by SIGKILL
test_hold.py::test_two blocked: the module's process died by SIGKILL
2 cases: 1 passed, 1 blocked; 1 project error
LINES
}

# The same, with no test after it: nothing is left to block, so no line names
# the end.
@test "a zygote that dies during the replay of a module's last test names no end" {
  cd "${BATS_TEST_TMPDIR:?set by bats}"
  printf '%s\n' 'import os, signal, threading' 'from pathlib import Path' \
    'def test_one():' \
    '    threading.Thread(target=threading.Event().wait, daemon=True).start()' \
    '    replay = Path("ran").exists()' '    Path("ran").touch()' \
    '    os.kill(os.getppid(), [0, signal.SIGKILL][replay])' \
    '    assert 1 == 1' '    assert 2 == 2' >test_last.py

  run -0 --separate-stderr purba test_last.py

  diff -u - <(printf '%s\n' "${output}") <<'LINES'
test_last.py::test_one passed
1 case: 1 passed
LINES
}

# The module's load leaves a thread alive, so each test's process and each
# replay load the module themselves. Only the replay's load raises.
@test "a replay whose own load raises gives the case the error of that load" {
  cd "${BATS_TEST_TMPDIR:?set by bats}"
  printf '%s\n' 'import threading' 'from pathlib import Path' \
    'threading.Thread(target=threading.Event().wait, daemon=True).start()' \
    'if Path("ran").exists():' '    raise ValueError("the second load")' \
    'def test_one():' '    Path("ran").touch()' '    assert 1 == 1' '    assert 2 == 2' \
    >test_reload.py

  run -4 --separate-stderr purba test_reload.py

  diff -u - <(printf '%s\n' "${output}") <<'LINES'
test_reload.py::test_one project error: ValueError: the second load
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
