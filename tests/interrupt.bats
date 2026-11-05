#   the decision:  docs/decisions/an-interrupt-stops-the-run-and-a-second-one-exits-at-once.md
#   the decision:  docs/decisions/an-interrupted-run-prints-its-count-and-dies-by-the-signal.md
#   the decision:  docs/decisions/an-interrupt-puts-back-no-more-than-a-normal-end-does.md
#
# A shell reads 130 both for a death by SIGINT and for an exit with the code
# 130. So Python's `subprocess` starts purba and sends the signal. Its
# `returncode` is minus the signal after a death by it.
#
# bash 3.2 on macOS fails a test at a `[[ ]]` only on its last line. So `run -N`
# checks each exit code. `|| false` ends each other check.
# https://bats-core.readthedocs.io/en/stable/gotchas.html#or-did-not-fail-my-test

: "${BATS_TEST_DIRNAME:?set by bats}"
bats_require_minimum_version 1.5.0

# Runs `purba <module>` in a process group of its own. Once the file `marker`
# exists, and half a second later, it sends the first signal to that group. It
# sends each further signal one second after the one before. The signals are
# SIGINT by default, or the names after <hold>, such as `INT TERM`. With
# `none`, it sends no signal. With `ignored`, purba starts with SIGINT ignored,
# and gets one SIGINT.
#
# It reads stdout only <hold> seconds after the last signal. A pipe holds less
# than a line of 300 KB, so purba waits in its write until then. It prints
# purba's stdout and stderr, then a line on stderr that says how purba ended.
# It kills purba when purba lives 30 seconds after that.
interrupt() {
  local purba
  purba=$(command -v purba)
  "${purba%/*}/python" - "${purba}" "$@" <<'PY'
import os, signal, subprocess, sys, time

purba, module, hold, *names = sys.argv[1:]
ignored = names == ["ignored"]
if ignored or not names:
    names = ["INT"]
disposition = signal.SIG_IGN if ignored else signal.SIG_DFL
child = subprocess.Popen(
    [purba, module],
    stdout=subprocess.PIPE,
    stderr=subprocess.PIPE,
    start_new_session=True,
    preexec_fn=lambda: signal.signal(signal.SIGINT, disposition),
)
if names != ["none"]:
    deadline = time.monotonic() + 30
    while not os.path.exists("marker"):
        if time.monotonic() > deadline:
            child.kill()
            sys.exit("the test never wrote the marker")
        time.sleep(0.01)
    for wait, name in zip([0.5] + [1] * len(names), names):
        time.sleep(wait)
        os.killpg(child.pid, getattr(signal, "SIG" + name))
time.sleep(float(hold))
try:
    out, err = child.communicate(timeout=30)
except subprocess.TimeoutExpired:
    os.killpg(child.pid, signal.SIGKILL)
    out, err = child.communicate()
    sys.stderr.write("purba lived 30 seconds after the last signal\n")
sys.stdout.write(out.decode())
sys.stderr.write(err.decode())
code = child.returncode
sys.stderr.write(
    f"purba ended by {signal.Signals(-code).name}\n" if code < 0 else f"purba exited {code}\n"
)
PY
}

# The text of an exception of 300 KB, which a pipe cannot hold, and the line
# on stderr as a signal arrives.
setup() {
  SECOND='; a second interrupt exits at once'
  SIGINT_LINE="purba: interrupt: SIGINT${SECOND}"
  local zeros="${BATS_TEST_TMPDIR:?set by bats}/zeros"
  head -c 300000 /dev/zero >"${zeros}"
  XS=$(tr '\0' x <"${zeros}")
}

@test "a Ctrl-C while purba writes a line starts no later test, and purba dies by SIGINT" {
  cd "${BATS_TEST_TMPDIR:?set by bats}"
  printf '%s\n' 'from pathlib import Path' \
    'def test_one():' '    Path("marker").touch()' '    raise Exception("x" * 300000)' \
    'def test_two():' '    Path("two").touch()' >test_gap.py

  run -0 --separate-stderr interrupt test_gap.py 2

  diff -u - <(printf '%s\n' "${output}") <<LINES
test_gap.py::test_one raised: Exception: ${XS}
2 tests: 1 raised, 1 unreached
LINES
  [[ "${stderr}" == "${SIGINT_LINE}"$'\npurba ended by SIGINT' ]] || false
  [[ ! -e two ]]
}

@test "a Ctrl-C while purba writes the last line prints the count, and purba dies by SIGINT" {
  cd "${BATS_TEST_TMPDIR:?set by bats}"
  printf '%s\n' 'from pathlib import Path' \
    'def test_one():' '    Path("marker").touch()' '    raise Exception("x" * 300000)' >test_last.py

  run -0 --separate-stderr interrupt test_last.py 2

  diff -u - <(printf '%s\n' "${output}") <<LINES
test_last.py::test_one raised: Exception: ${XS}
1 test: 1 raised
LINES
  [[ "${stderr}" == "${SIGINT_LINE}"$'\npurba ended by SIGINT' ]]
}

@test "a Ctrl-C while purba calls __str__ keeps the exception's text, and purba dies by SIGINT" {
  cd "${BATS_TEST_TMPDIR:?set by bats}"
  printf '%s\n' 'import time' 'from pathlib import Path' \
    'class Slow(Exception):' '    def __str__(self):' \
    '        Path("marker").touch()' '        time.sleep(2)' '        return "slow"' \
    'def test_one():' '    raise Slow()' \
    'def test_two():' '    Path("two").touch()' >test_str.py

  run -0 --separate-stderr interrupt test_str.py 0

  diff -u - <(printf '%s\n' "${output}") <<'LINES'
test_str.py::test_one raised: Slow: slow
2 tests: 1 raised, 1 unreached
LINES
  [[ "${stderr}" == "${SIGINT_LINE}"$'\npurba ended by SIGINT' ]] || false
  [[ ! -e two ]]
}

@test "a Ctrl-C in a test's call makes the test interrupted, and its finally block runs" {
  cd "${BATS_TEST_TMPDIR:?set by bats}"
  printf '%s\n' 'import time' 'from pathlib import Path' \
    'def test_one():' '    assert True' \
    'def test_two():' '    try:' '        Path("marker").touch()' '        time.sleep(3)' \
    '    finally:' '        Path("finally").touch()' \
    'def test_three():' '    Path("three").touch()' >test_call.py

  run -0 --separate-stderr interrupt test_call.py 0

  diff -u - <(printf '%s\n' "${output}") <<'LINES'
test_call.py::test_one passed
test_call.py::test_two interrupted
3 tests: 1 passed, 1 interrupted, 1 unreached
LINES
  [[ "${stderr}" == "${SIGINT_LINE}"$'\npurba ended by SIGINT' ]] || false
  [[ -e finally ]] || false
  [[ ! -e three ]]
}

@test "a Ctrl-C while a module loads leaves each test unreached, and purba dies by SIGINT" {
  cd "${BATS_TEST_TMPDIR:?set by bats}"
  printf '%s\n' 'import time' 'from pathlib import Path' \
    'Path("marker").touch()' 'time.sleep(3)' \
    'def test_one():' '    assert True' \
    'def test_two():' '    assert True' >test_load.py

  run -0 --separate-stderr interrupt test_load.py 0

  [[ "${output}" == '2 tests: 2 unreached' ]] || false
  [[ "${stderr}" == "${SIGINT_LINE}"$'\npurba ended by SIGINT' ]]
}

@test "a KeyboardInterrupt with no signal makes the test raised, and the next test runs" {
  cd "${BATS_TEST_TMPDIR:?set by bats}"
  printf '%s\n' 'def test_one():' '    raise KeyboardInterrupt' \
    'def test_two():' '    assert True' >test_bare.py

  run -4 --separate-stderr purba test_bare.py

  diff -u - <(printf '%s\n' "${output}") <<'LINES'
test_bare.py::test_one raised: KeyboardInterrupt:
test_bare.py::test_two passed
2 tests: 1 passed, 1 raised
LINES
  [[ -z "${stderr}" ]]
}

@test "a signal handler that a test installs lives only for its call" {
  cd "${BATS_TEST_TMPDIR:?set by bats}"
  printf '%s\n' 'import signal, time' 'from pathlib import Path' \
    'def test_one():' '    signal.signal(signal.SIGINT, lambda signum, frame: None)' \
    '    assert True' \
    'def test_two():' '    Path("marker").touch()' '    time.sleep(3)' \
    'def test_three():' '    Path("three").touch()' >test_handler.py

  run -0 --separate-stderr interrupt test_handler.py 0

  diff -u - <(printf '%s\n' "${output}") <<'LINES'
test_handler.py::test_one passed
test_handler.py::test_two interrupted
3 tests: 1 passed, 1 interrupted, 1 unreached
LINES
  [[ "${stderr}" == "${SIGINT_LINE}"$'\npurba ended by SIGINT' ]] || false
  [[ ! -e three ]]
}

@test "a signal handler that a module installs lives only for its load" {
  cd "${BATS_TEST_TMPDIR:?set by bats}"
  printf '%s\n' 'import signal, time' 'from pathlib import Path' \
    'signal.signal(signal.SIGINT, lambda signum, frame: None)' \
    'def test_one():' '    Path("marker").touch()' '    time.sleep(3)' \
    'def test_two():' '    Path("two").touch()' >test_module_handler.py

  run -0 --separate-stderr interrupt test_module_handler.py 0

  diff -u - <(printf '%s\n' "${output}") <<'LINES'
test_module_handler.py::test_one interrupted
2 tests: 1 interrupted, 1 unreached
LINES
  [[ "${stderr}" == "${SIGINT_LINE}"$'\npurba ended by SIGINT' ]] || false
  [[ ! -e two ]]
}

@test "a test that a Ctrl-C lands in is interrupted, whether it then passes, fails or raises" {
  cd "${BATS_TEST_TMPDIR:?set by bats}"
  printf '%s\n' 'import time' 'from pathlib import Path' \
    'def test_one():' '    try:' '        Path("marker").touch()' '        time.sleep(3)' \
    '    except KeyboardInterrupt:' '        pass' '    assert True' >test_passes.py
  printf '%s\n' 'import time' 'from pathlib import Path' \
    'def test_one():' '    try:' '        Path("marker").touch()' '        time.sleep(3)' \
    '    finally:' '        assert 1 == 2' >test_fails.py
  printf '%s\n' 'import os, time' 'from pathlib import Path' \
    'def test_one():' '    try:' '        Path("marker").touch()' '        time.sleep(3)' \
    '    finally:' '        os._exit(3)' >test_exits.py

  for module in test_passes test_fails test_exits; do
    rm -f marker
    run -0 --separate-stderr interrupt "${module}.py" 0
    [[ "${output}" == "${module}.py::test_one interrupted"$'\n1 test: 1 interrupted' ]] || false
    [[ "${stderr}" == "${SIGINT_LINE}"$'\npurba ended by SIGINT' ]] || false
  done
}

@test "a Ctrl-C that the parent ignores stays ignored, and the run ends normally" {
  cd "${BATS_TEST_TMPDIR:?set by bats}"
  printf '%s\n' 'import time' 'from pathlib import Path' \
    'def test_one():' '    Path("marker").touch()' \
    '    time.sleep(1)' '    assert True' >test_ignored.py

  run -0 --separate-stderr interrupt test_ignored.py 0 ignored

  diff -u - <(printf '%s\n' "${output}") <<'LINES'
test_ignored.py::test_one passed
1 test: 1 passed
LINES
  [[ "${stderr}" == 'purba exited 0' ]]
}

@test "SIGTERM and SIGHUP make the test they land in interrupted, and purba dies by that signal" {
  cd "${BATS_TEST_TMPDIR:?set by bats}"
  printf '%s\n' 'import time' 'from pathlib import Path' \
    'def test_one():' '    assert True' \
    'def test_two():' '    try:' '        Path("marker").touch()' '        time.sleep(3)' \
    '    finally:' '        Path("finally").touch()' \
    'def test_three():' '    Path("three").touch()' >test_term.py

  for name in TERM HUP; do
    rm -f marker finally
    run -0 --separate-stderr interrupt test_term.py 0 "${name}"
    diff -u - <(printf '%s\n' "${output}") <<'LINES'
test_term.py::test_one passed
test_term.py::test_two interrupted
3 tests: 1 passed, 1 interrupted, 1 unreached
LINES
    local ended="purba ended by SIG${name}"
    [[ "${stderr}" == "purba: interrupt: SIG${name}${SECOND}"$'\n'"${ended}" ]] || false
    [[ -e finally ]] || false
    [[ ! -e three ]] || false
  done
}

@test "a second Ctrl-C ends purba at once by SIGINT, even while a finally block runs C code" {
  cd "${BATS_TEST_TMPDIR:?set by bats}"
  printf '%s\n' 'import time' 'from pathlib import Path' \
    'def test_one():' '    assert True' \
    'def test_two():' '    try:' '        Path("marker").touch()' '        time.sleep(3)' \
    '    finally:' '        sum(range(10**10))' '        Path("finally").touch()' >test_second.py

  run -0 --separate-stderr interrupt test_second.py 0 INT INT

  [[ "${output}" == 'test_second.py::test_one passed' ]] || false
  [[ "${stderr}" == "${SIGINT_LINE}"$'\npurba ended by SIGINT' ]] || false
  [[ ! -e finally ]]
}

@test "SIGINT and then SIGTERM make a second interrupt, and purba dies by SIGTERM" {
  cd "${BATS_TEST_TMPDIR:?set by bats}"
  printf '%s\n' 'import time' 'from pathlib import Path' \
    'def test_one():' '    try:' '        Path("marker").touch()' '        time.sleep(3)' \
    '    finally:' '        time.sleep(5)' '        Path("finally").touch()' >test_then_term.py

  run -0 --separate-stderr interrupt test_then_term.py 0 INT TERM

  [[ -z "${output}" ]] || false
  [[ "${stderr}" == "${SIGINT_LINE}"$'\npurba ended by SIGTERM' ]] || false
  [[ ! -e finally ]]
}

@test "two signals that wait at once make a second interrupt, and purba dies by the second" {
  cd "${BATS_TEST_TMPDIR:?set by bats}"
  printf '%s\n' 'import os, signal' \
    'def test_one():' '    both = {signal.SIGINT, signal.SIGTERM}' \
    '    signal.pthread_sigmask(signal.SIG_BLOCK, both)' \
    '    os.kill(os.getpid(), signal.SIGINT)' '    os.kill(os.getpid(), signal.SIGTERM)' \
    '    signal.pthread_sigmask(signal.SIG_UNBLOCK, both)' \
    'def test_two():' '    assert True' >test_waiting.py

  run -0 --separate-stderr interrupt test_waiting.py 0 none

  [[ "${stderr##*$'\n'}" == 'purba ended by SIGTERM' ]]
}

@test "a Ctrl-C ends Python's wait for a thread that a test left alive, and purba dies by SIGINT" {
  cd "${BATS_TEST_TMPDIR:?set by bats}"
  printf '%s\n' 'import threading, time' 'from pathlib import Path' \
    'def linger():' '    time.sleep(10)' '    Path("thread").touch()' \
    'def test_one():' '    threading.Thread(target=linger).start()' \
    '    Path("marker").touch()' '    assert True' >test_thread.py

  run -0 --separate-stderr interrupt test_thread.py 0

  diff -u - <(printf '%s\n' "${output}") <<'LINES'
test_thread.py::test_one passed
1 test: 1 passed
LINES
  [[ "${stderr%%$'\n'*}" == "${SIGINT_LINE}" ]] || false
  [[ "${stderr##*$'\n'}" == 'purba ended by SIGINT' ]] || false
  [[ ! -e thread ]]
}

@test "a Ctrl-C in a test's atexit function lets the function finish, and purba dies by SIGINT" {
  cd "${BATS_TEST_TMPDIR:?set by bats}"
  printf '%s\n' 'import atexit, time' 'from pathlib import Path' \
    'def linger():' '    Path("marker").touch()' '    time.sleep(2)' '    Path("finally").touch()' \
    'def test_one():' '    atexit.register(linger)' '    assert True' >test_atexit.py

  run -0 --separate-stderr interrupt test_atexit.py 0

  diff -u - <(printf '%s\n' "${output}") <<'LINES'
test_atexit.py::test_one passed
1 test: 1 passed
LINES
  [[ "${stderr}" == "${SIGINT_LINE}"$'\npurba ended by SIGINT' ]] || false
  [[ -e finally ]]
}
