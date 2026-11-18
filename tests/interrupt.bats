#   the decision:  docs/decisions/an-interrupt-stops-the-run-and-a-second-one-exits-at-once.md
#   the decision:  docs/decisions/an-interrupted-run-prints-its-count-and-dies-by-the-signal.md
#   the decision:  docs/decisions/an-interrupt-puts-back-no-more-than-a-normal-end-does.md
#   the decision:
# docs/decisions/a-process-that-a-test-forks-has-the-signal-handlers-it-has-without-purba.md
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
# and gets one SIGINT. With `masked` first, purba starts with SIGINT masked, and
# gets the signals named after it, or no signal. With `alone` first, each signal
# goes to purba alone, and not to its group. With `forwarded`, purba gets one
# SIGINT alone, and 50 ms later each other process of its group gets one, as a
# signal to the group can reach them after purba forwards its own.
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
forwarded = names == ["forwarded"]
alone = names[:1] == ["alone"]
if alone:
    names = names[1:]
masked = names[:1] == ["masked"]
if masked:
    names = names[1:] or ["none"]
if ignored or forwarded or not names:
    names = ["INT"]
disposition = signal.SIG_IGN if ignored else signal.SIG_DFL
def start():
    signal.signal(signal.SIGINT, disposition)
    if masked:
        signal.pthread_sigmask(signal.SIG_BLOCK, {signal.SIGINT})
child = subprocess.Popen(
    [purba, module],
    stdout=subprocess.PIPE,
    stderr=subprocess.PIPE,
    start_new_session=True,
    preexec_fn=start,
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
        send = os.kill if alone or forwarded else os.killpg
        send(child.pid, getattr(signal, "SIG" + name))
        if forwarded:
            time.sleep(0.05)
            group = subprocess.run(["pgrep", "-g", str(child.pid)], capture_output=True, text=True)
            for pid in map(int, group.stdout.split()):
                if pid != child.pid:
                    os.kill(pid, signal.SIGINT)
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
2 cases: 1 raised, 1 unreached
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
1 case: 1 raised
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
2 cases: 1 raised, 1 unreached
LINES
  [[ "${stderr}" == "${SIGINT_LINE}"$'\npurba ended by SIGINT' ]] || false
  [[ ! -e two ]]
}

@test "a Ctrl-C in a test's call makes the test interrupted, and its finally block runs" {
  cd "${BATS_TEST_TMPDIR:?set by bats}"
  printf '%s\n' 'import time' 'from pathlib import Path' \
    'def test_one():' '    assert 1 == 1' \
    'def test_two():' '    try:' '        Path("marker").touch()' '        time.sleep(3)' \
    '    finally:' '        Path("finally").touch()' \
    'def test_three():' '    Path("three").touch()' >test_call.py

  run -0 --separate-stderr interrupt test_call.py 0

  diff -u - <(printf '%s\n' "${output}") <<'LINES'
test_call.py::test_one passed
test_call.py::test_two interrupted
3 cases: 1 passed, 1 interrupted, 1 unreached
LINES
  [[ "${stderr}" == "${SIGINT_LINE}"$'\npurba ended by SIGINT' ]] || false
  [[ -e finally ]] || false
  [[ ! -e three ]]
}

@test "an interrupt sent to purba alone makes the test interrupted, and its finally block runs" {
  cd "${BATS_TEST_TMPDIR:?set by bats}"
  printf '%s\n' 'import time' 'from pathlib import Path' \
    'def test_one():' '    assert 1 == 1' \
    'def test_two():' '    try:' '        Path("marker").touch()' '        time.sleep(3)' \
    '    finally:' '        Path("finally").touch()' \
    'def test_three():' '    Path("three").touch()' >test_call.py

  run -0 --separate-stderr interrupt test_call.py 0 alone

  diff -u - <(printf '%s\n' "${output}") <<'LINES'
test_call.py::test_one passed
test_call.py::test_two interrupted
3 cases: 1 passed, 1 interrupted, 1 unreached
LINES
  [[ "${stderr}" == "${SIGINT_LINE}"$'\npurba ended by SIGINT' ]] || false
  [[ -e finally ]] || false
  [[ ! -e three ]]
}

@test "a Ctrl-C while purba looks a test up makes the test interrupted, and its call never starts" {
  cd "${BATS_TEST_TMPDIR:?set by bats}"
  printf '%s\n' 'import sys, time, types' 'from pathlib import Path' \
    'class Slow(types.ModuleType):' '    def __getattribute__(self, name):' \
    '        if name == "test_two":' \
    '            Path("marker").touch()' '            time.sleep(2)' \
    '        return super().__getattribute__(name)' \
    'sys.modules[__name__].__class__ = Slow' \
    'def test_one():' '    assert 1 == 1' \
    'def test_two():' '    Path("two").touch()' \
    'def test_three():' '    Path("three").touch()' >test_lookup.py

  run -0 --separate-stderr interrupt test_lookup.py 0

  diff -u - <(printf '%s\n' "${output}") <<'LINES'
test_lookup.py::test_one passed
test_lookup.py::test_two interrupted
3 cases: 1 passed, 1 interrupted, 1 unreached
LINES
  [[ "${stderr}" == "${SIGINT_LINE}"$'\npurba ended by SIGINT' ]] || false
  [[ ! -e two ]] || false
  [[ ! -e three ]]
}

@test "a Ctrl-C that reaches a process of the run after purba forwarded it counts once" {
  cd "${BATS_TEST_TMPDIR:?set by bats}"
  printf '%s\n' 'import time' 'from pathlib import Path' \
    'def test_one():' '    try:' '        Path("marker").touch()' '        time.sleep(3)' \
    '    finally:' '        time.sleep(0.5)' '        Path("finally").touch()' \
    'def test_two():' '    Path("two").touch()' >test_running.py
  printf '%s\n' 'from pathlib import Path' \
    'def test_one():' '    Path("marker").touch()' '    raise Exception("x" * 300000)' \
    'def test_two():' '    Path("two").touch()' >test_idle.py

  run -0 --separate-stderr interrupt test_running.py 0 forwarded

  diff -u - <(printf '%s\n' "${output}") <<'LINES'
test_running.py::test_one interrupted
2 cases: 1 interrupted, 1 unreached
LINES
  [[ "${stderr}" == "${SIGINT_LINE}"$'\npurba ended by SIGINT' ]] || false
  [[ -e finally ]] || false

  rm marker
  run -0 --separate-stderr interrupt test_idle.py 2 forwarded

  diff -u - <(printf '%s\n' "${output}") <<LINES
test_idle.py::test_one raised: Exception: ${XS}
2 cases: 1 raised, 1 unreached
LINES
  [[ "${stderr}" == "${SIGINT_LINE}"$'\npurba ended by SIGINT' ]] || false
  [[ ! -e two ]]
}

@test "a Ctrl-C while a module loads leaves each test unreached, and purba dies by SIGINT" {
  cd "${BATS_TEST_TMPDIR:?set by bats}"
  printf '%s\n' 'import time' 'from pathlib import Path' \
    'Path("marker").touch()' 'time.sleep(3)' \
    'def test_one():' '    assert 1 == 1' \
    'def test_two():' '    assert 1 == 1' >test_load.py

  run -0 --separate-stderr interrupt test_load.py 0

  [[ "${output}" == '2 cases: 2 unreached' ]] || false
  [[ "${stderr}" == "${SIGINT_LINE}"$'\npurba ended by SIGINT' ]]
}

@test "a Ctrl-C in a load whose process then exits leaves each test unreached, and no error" {
  cd "${BATS_TEST_TMPDIR:?set by bats}"
  printf '%s\n' 'import os, time' 'from pathlib import Path' \
    'try:' '    Path("marker").touch()' '    time.sleep(3)' 'finally:' '    os._exit(3)' \
    'def test_one():' '    assert 1 == 1' >test_load_exits.py

  run -0 --separate-stderr interrupt test_load_exits.py 0

  [[ "${output}" == '1 case: 1 unreached' ]] || false
  [[ "${stderr}" == "${SIGINT_LINE}"$'\npurba ended by SIGINT' ]]
}

@test "a KeyboardInterrupt with no signal makes the test raised, and the next test runs" {
  cd "${BATS_TEST_TMPDIR:?set by bats}"
  printf '%s\n' 'def test_one():' '    raise KeyboardInterrupt' \
    'def test_two():' '    assert 1 == 1' >test_bare.py

  run -4 --separate-stderr purba test_bare.py

  diff -u - <(printf '%s\n' "${output}") <<'LINES'
test_bare.py::test_one raised: KeyboardInterrupt:
test_bare.py::test_two passed
2 cases: 1 passed, 1 raised
LINES
  [[ -z "${stderr}" ]]
}

@test "a signal handler that a test installs lives only for its call" {
  cd "${BATS_TEST_TMPDIR:?set by bats}"
  printf '%s\n' 'import signal, time' 'from pathlib import Path' \
    'def test_one():' '    signal.signal(signal.SIGINT, lambda signum, frame: None)' \
    '    assert 1 == 1' \
    'def test_two():' '    Path("marker").touch()' '    time.sleep(3)' \
    'def test_three():' '    Path("three").touch()' >test_handler.py

  run -0 --separate-stderr interrupt test_handler.py 0

  diff -u - <(printf '%s\n' "${output}") <<'LINES'
test_handler.py::test_one passed
test_handler.py::test_two interrupted
3 cases: 1 passed, 1 interrupted, 1 unreached
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
2 cases: 1 interrupted, 1 unreached
LINES
  [[ "${stderr}" == "${SIGINT_LINE}"$'\npurba ended by SIGINT' ]] || false
  [[ ! -e two ]]
}

@test "a signal handler that a test installs below Python lives only for its call" {
  cd "${BATS_TEST_TMPDIR:?set by bats}"
  printf '%s\n' 'import faulthandler, signal, time' 'from pathlib import Path' \
    'def test_one():' '    faulthandler.register(signal.SIGINT)' '    assert 1 == 1' \
    'def test_two():' '    assert 1 == 1' '    Path("marker").touch()' '    time.sleep(3)' \
    'def test_three():' '    Path("three").touch()' >test_below.py

  run -0 --separate-stderr interrupt test_below.py 0

  diff -u - <(printf '%s\n' "${output}") <<'LINES'
test_below.py::test_one passed
test_below.py::test_two interrupted
3 cases: 1 passed, 1 interrupted, 1 unreached
LINES
  [[ "${stderr}" == "${SIGINT_LINE}"$'\npurba ended by SIGINT' ]] || false
  [[ ! -e three ]]
}

@test "an ignore that a test gives SIGINT below Python lives only for its call" {
  cd "${BATS_TEST_TMPDIR:?set by bats}"
  printf '%s\n' 'import ctypes, signal, time' 'from pathlib import Path' \
    'def test_one():' '    libc = ctypes.CDLL(None)' \
    '    libc.signal.argtypes = (ctypes.c_int, ctypes.c_void_p)' \
    '    libc.signal(signal.SIGINT, signal.SIG_IGN.value)' '    assert 1 == 1' \
    'def test_two():' '    assert 1 == 1' '    Path("marker").touch()' '    time.sleep(3)' \
    'def test_three():' '    Path("three").touch()' >test_ignore_below.py

  run -0 --separate-stderr interrupt test_ignore_below.py 0

  diff -u - <(printf '%s\n' "${output}") <<'LINES'
test_ignore_below.py::test_one passed
test_ignore_below.py::test_two interrupted
3 cases: 1 passed, 1 interrupted, 1 unreached
LINES
  [[ "${stderr}" == "${SIGINT_LINE}"$'\npurba ended by SIGINT' ]] || false
  [[ ! -e three ]]
}

@test "the flag that signal.siginterrupt gives SIGINT lives only for its call" {
  cd "${BATS_TEST_TMPDIR:?set by bats}"
  # The writer starts in a session of its own, so the Ctrl-C does not reach it.
  # It writes 5 seconds after it starts. A read that the signal does not end
  # lasts until then, and the writer writes `wrote` first.
  printf '%s\n' 'import os, signal, subprocess, sys' 'from pathlib import Path' \
    'WRITE = ("import os, sys, time; time.sleep(5); open(\"wrote\", \"w\").close();"' \
    '    " os.write(int(sys.argv[1]), b\"x\")")' \
    'def test_one():' '    signal.siginterrupt(signal.SIGINT, False)' '    assert 1 == 1' \
    'def test_two():' '    assert 1 == 1' '    read, write = os.pipe()' \
    '    subprocess.Popen([sys.executable, "-c", WRITE, str(write)], pass_fds=(write,),' \
    '        stdin=subprocess.DEVNULL, stdout=subprocess.DEVNULL,' \
    '        stderr=subprocess.DEVNULL, start_new_session=True)' \
    '    Path("marker").touch()' '    os.read(read, 1)' \
    'def test_three():' '    Path("three").touch()' >test_siginterrupt.py

  run -0 --separate-stderr interrupt test_siginterrupt.py 0

  diff -u - <(printf '%s\n' "${output}") <<'LINES'
test_siginterrupt.py::test_one passed
test_siginterrupt.py::test_two interrupted
3 cases: 1 passed, 1 interrupted, 1 unreached
LINES
  [[ "${stderr}" == "${SIGINT_LINE}"$'\npurba ended by SIGINT' ]] || false
  [[ ! -e wrote ]] || false
  [[ ! -e three ]]
}

@test "a mask that a test puts on SIGINT lives only for its call" {
  cd "${BATS_TEST_TMPDIR:?set by bats}"
  printf '%s\n' 'import signal, time' 'from pathlib import Path' \
    'def test_one():' '    signal.pthread_sigmask(signal.SIG_BLOCK, {signal.SIGINT})' \
    '    assert 1 == 1' \
    'def test_two():' '    Path("marker").touch()' '    time.sleep(3)' \
    'def test_three():' '    Path("three").touch()' >test_mask.py

  run -0 --separate-stderr interrupt test_mask.py 0

  diff -u - <(printf '%s\n' "${output}") <<'LINES'
test_mask.py::test_one passed
test_mask.py::test_two interrupted
3 cases: 1 passed, 1 interrupted, 1 unreached
LINES
  [[ "${stderr}" == "${SIGINT_LINE}"$'\npurba ended by SIGINT' ]] || false
  [[ ! -e three ]]
}

@test "a mask that a module puts on SIGINT lives only for its load" {
  cd "${BATS_TEST_TMPDIR:?set by bats}"
  printf '%s\n' 'import signal, time' 'from pathlib import Path' \
    'signal.pthread_sigmask(signal.SIG_BLOCK, {signal.SIGINT})' \
    'def test_one():' '    Path("marker").touch()' '    time.sleep(3)' \
    'def test_two():' '    Path("two").touch()' >test_module_mask.py

  run -0 --separate-stderr interrupt test_module_mask.py 0

  diff -u - <(printf '%s\n' "${output}") <<'LINES'
test_module_mask.py::test_one interrupted
2 cases: 1 interrupted, 1 unreached
LINES
  [[ "${stderr}" == "${SIGINT_LINE}"$'\npurba ended by SIGINT' ]] || false
  [[ ! -e two ]]
}

@test "a signal that the parent masks stays masked after a test unmasks it" {
  cd "${BATS_TEST_TMPDIR:?set by bats}"
  printf '%s\n' 'import signal' \
    'def test_one():' '    signal.pthread_sigmask(signal.SIG_UNBLOCK, {signal.SIGINT})' \
    '    assert 1 == 1' \
    'def test_two():' \
    '    assert 2 in signal.pthread_sigmask(signal.SIG_BLOCK, set())' \
    >test_parent_mask.py

  run -0 --separate-stderr interrupt test_parent_mask.py 0 masked

  diff -u - <(printf '%s\n' "${output}") <<'LINES'
test_parent_mask.py::test_one passed
test_parent_mask.py::test_two passed
2 cases: 2 passed
LINES
  [[ "${stderr}" == 'purba exited 0' ]]
}

@test "a Ctrl-C in a test that unmasks a signal the parent masks ends purba by SIGINT" {
  cd "${BATS_TEST_TMPDIR:?set by bats}"
  printf '%s\n' 'import signal, time' 'from pathlib import Path' \
    'def test_one():' '    signal.pthread_sigmask(signal.SIG_UNBLOCK, {signal.SIGINT})' \
    '    Path("marker").touch()' '    time.sleep(3)' \
    'def test_two():' '    Path("two").touch()' >test_unmasked.py

  run -0 --separate-stderr interrupt test_unmasked.py 0 masked INT

  diff -u - <(printf '%s\n' "${output}") <<'LINES'
test_unmasked.py::test_one interrupted
2 cases: 1 interrupted, 1 unreached
LINES
  [[ "${stderr}" == "${SIGINT_LINE}"$'\npurba ended by SIGINT' ]] || false
  [[ ! -e two ]]
}

@test "a Ctrl-C that a test's mask holds stops the run, though the test has its own handler" {
  cd "${BATS_TEST_TMPDIR:?set by bats}"
  printf '%s\n' 'import signal, time' 'from pathlib import Path' \
    'def test_one():' '    signal.signal(signal.SIGINT, lambda signum, frame: None)' \
    '    signal.pthread_sigmask(signal.SIG_BLOCK, {signal.SIGINT})' \
    '    Path("marker").touch()' '    time.sleep(2)' '    assert 1 == 1' \
    'def test_two():' '    Path("two").touch()' >test_held_own.py

  run -0 --separate-stderr interrupt test_held_own.py 0

  diff -u - <(printf '%s\n' "${output}") <<'LINES'
test_held_own.py::test_one passed
2 cases: 1 passed, 1 unreached
LINES
  [[ "${stderr}" == "${SIGINT_LINE}"$'\npurba ended by SIGINT' ]] || false
  [[ ! -e two ]]
}

@test "a test's signal handler lives only for its call, though the test patches signal.signal" {
  cd "${BATS_TEST_TMPDIR:?set by bats}"
  printf '%s\n' 'import signal, time' 'from pathlib import Path' 'from unittest import mock' \
    '# purba: suppress[patched-name] the handler must outlive a patch of signal.signal' \
    '# purba: suppress[mock-without-spec] each mock only blocks the call it replaces' \
    'def test_one():' '    signal.signal(signal.SIGINT, lambda signum, frame: None)' \
    '    mock.patch("signal.signal").start()' '    mock.patch("_signal.signal").start()' \
    '    mock.patch("_signal.getsignal").start()' '    assert 1 == 1' \
    'def test_two():' '    Path("marker").touch()' '    time.sleep(3)' \
    'def test_three():' '    Path("three").touch()' >test_patched.py

  run -0 --separate-stderr interrupt test_patched.py 0

  local argument='and a stand-in enters a test as an argument'
  local spec='makes a mock with no spec, and a mock has the spec of what it stands for'
  diff -u - <(printf '%s\n' "${output}") <<LINES
test_patched.py::test_one passed
    ⚠ suppressed [patched-name] line 8 patches signal.signal, ${argument}; \
line 9 patches _signal.signal, ${argument}; line 10 patches _signal.getsignal, ${argument}
      reason: the handler must outlive a patch of signal.signal
    ⚠ suppressed [mock-without-spec] line 8 ${spec}; line 9 ${spec}; line 10 ${spec}
      reason: each mock only blocks the call it replaces
test_patched.py::test_two interrupted
3 cases: 1 passed, 1 interrupted, 1 unreached; 2 suppressed
LINES
  [[ "${stderr}" == "${SIGINT_LINE}"$'\npurba ended by SIGINT' ]] || false
  [[ ! -e three ]]
}

@test "a test that a Ctrl-C lands in is interrupted, whether it then passes, fails or raises" {
  cd "${BATS_TEST_TMPDIR:?set by bats}"
  printf '%s\n' 'import time' 'from pathlib import Path' \
    'def test_one():' '    try:' '        Path("marker").touch()' '        time.sleep(3)' \
    '    except KeyboardInterrupt:' '        caught = True' '    assert 1 == 1' >test_passes.py
  printf '%s\n' 'import time' 'from pathlib import Path' \
    'def test_one():' '    try:' '        Path("marker").touch()' '        time.sleep(3)' \
    '    finally:' '        assert 1 == 2' >test_fails.py
  printf '%s\n' 'import os, time' 'from pathlib import Path' \
    'def exit_now(code):' '    os._exit(code)' \
    'def test_one():' '    try:' '        Path("marker").touch()' '        time.sleep(3)' \
    '    finally:' '        exit_now(3)' >test_exits.py

  for module in test_passes test_fails test_exits; do
    rm -f marker
    run -0 --separate-stderr interrupt "${module}.py" 0
    [[ "${output}" == "${module}.py::test_one interrupted"$'\n1 case: 1 interrupted' ]] || false
    [[ "${stderr}" == "${SIGINT_LINE}"$'\npurba ended by SIGINT' ]] || false
  done
}

@test "a Ctrl-C that the parent ignores stays ignored, and the run ends normally" {
  cd "${BATS_TEST_TMPDIR:?set by bats}"
  printf '%s\n' 'import time' 'from pathlib import Path' \
    'def test_one():' '    Path("marker").touch()' \
    '    time.sleep(1)' '    assert 1 == 1' >test_ignored.py

  run -0 --separate-stderr interrupt test_ignored.py 0 ignored

  diff -u - <(printf '%s\n' "${output}") <<'LINES'
test_ignored.py::test_one passed
1 case: 1 passed
LINES
  [[ "${stderr}" == 'purba exited 0' ]]
}

@test "a signal handler that a test gives a signal the parent ignores lives only for its call" {
  cd "${BATS_TEST_TMPDIR:?set by bats}"
  printf '%s\n' 'import signal, time' 'from pathlib import Path' \
    'def test_one():' '    signal.signal(signal.SIGINT, signal.default_int_handler)' \
    '    assert 1 == 1' \
    'def test_two():' '    Path("marker").touch()' '    time.sleep(1)' '    assert 1 == 1' \
    'def test_three():' '    assert 1 == 1' >test_ignored_handler.py

  run -0 --separate-stderr interrupt test_ignored_handler.py 0 ignored

  diff -u - <(printf '%s\n' "${output}") <<'LINES'
test_ignored_handler.py::test_one passed
test_ignored_handler.py::test_two passed
test_ignored_handler.py::test_three passed
3 cases: 3 passed
LINES
  [[ "${stderr}" == 'purba exited 0' ]]
}

@test "an ignore of the parent comes back after a test changes it below Python" {
  cd "${BATS_TEST_TMPDIR:?set by bats}"
  printf '%s\n' 'import ctypes, signal, time' 'from pathlib import Path' \
    'def test_one():' '    libc = ctypes.CDLL(None)' \
    '    libc.signal.argtypes = (ctypes.c_int, ctypes.c_void_p)' \
    '    libc.signal(signal.SIGINT, signal.SIG_DFL.value)' '    assert 1 == 1' \
    'def test_two():' '    Path("marker").touch()' '    time.sleep(1)' '    assert 1 == 1' \
    'def test_three():' '    assert 1 == 1' >test_ignored_below.py

  run -0 --separate-stderr interrupt test_ignored_below.py 0 ignored

  diff -u - <(printf '%s\n' "${output}") <<'LINES'
test_ignored_below.py::test_one passed
test_ignored_below.py::test_two passed
test_ignored_below.py::test_three passed
3 cases: 3 passed
LINES
  [[ "${stderr}" == 'purba exited 0' ]]
}

@test "a Ctrl-C that a test's mask holds stops the run when the call returns" {
  cd "${BATS_TEST_TMPDIR:?set by bats}"
  printf '%s\n' 'import signal, time' 'from pathlib import Path' \
    'def test_one():' '    signal.pthread_sigmask(signal.SIG_BLOCK, {signal.SIGINT})' \
    '    Path("marker").touch()' '    time.sleep(2)' '    assert 1 == 1' \
    'def test_two():' '    Path("two").touch()' >test_held.py

  run -0 --separate-stderr interrupt test_held.py 0

  diff -u - <(printf '%s\n' "${output}") <<'LINES'
test_held.py::test_one passed
2 cases: 1 passed, 1 unreached
LINES
  [[ "${stderr}" == "${SIGINT_LINE}"$'\npurba ended by SIGINT' ]] || false
  [[ ! -e two ]]
}

@test "SIGTERM and SIGHUP make the test they land in interrupted, and purba dies by that signal" {
  cd "${BATS_TEST_TMPDIR:?set by bats}"
  printf '%s\n' 'import time' 'from pathlib import Path' \
    'def test_one():' '    assert 1 == 1' \
    'def test_two():' '    try:' '        Path("marker").touch()' '        time.sleep(3)' \
    '    finally:' '        Path("finally").touch()' \
    'def test_three():' '    Path("three").touch()' >test_term.py

  for name in TERM HUP; do
    rm -f marker finally
    run -0 --separate-stderr interrupt test_term.py 0 "${name}"
    diff -u - <(printf '%s\n' "${output}") <<'LINES'
test_term.py::test_one passed
test_term.py::test_two interrupted
3 cases: 1 passed, 1 interrupted, 1 unreached
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
    'def test_one():' '    assert 1 == 1' \
    'def test_two():' '    try:' '        Path("marker").touch()' '        time.sleep(3)' \
    '    finally:' '        sum(range(10**10))' '        Path("finally").touch()' >test_second.py

  run -0 --separate-stderr interrupt test_second.py 0 INT INT

  [[ "${output}" == 'test_second.py::test_one passed' ]] || false
  [[ "${stderr}" == "${SIGINT_LINE}"$'\npurba ended by SIGINT' ]] || false
  [[ ! -e finally ]]
}

# A process that a kill ends can live on for a moment as it exits, so the check
# waits up to two seconds. A test's process that outlived purba would sleep on
# for five.
@test "a second Ctrl-C in a test that catches the first leaves no process of the run alive" {
  cd "${BATS_TEST_TMPDIR:?set by bats}"
  printf '%s\n' 'import os, time' 'from pathlib import Path' \
    'def test_one():' '    Path("group").write_text(str(os.getpgrp()))' \
    '    Path("marker").touch()' '    try:' '        time.sleep(5)' \
    '    except KeyboardInterrupt:' '        time.sleep(5)' >test_stubborn.py

  run -0 --separate-stderr interrupt test_stubborn.py 0 INT INT

  [[ "${stderr}" == "${SIGINT_LINE}"$'\npurba ended by SIGINT' ]] || false
  local group tries=0
  group=$(<group)
  while pgrep -g "${group}" >/dev/null && ((tries++ < 20)); do
    sleep 0.1
  done
  run -1 pgrep -g "${group}"
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

@test "a second Ctrl-C ends purba at once by SIGINT, when the first lands in a test's C code" {
  cd "${BATS_TEST_TMPDIR:?set by bats}"
  printf '%s\n' 'from pathlib import Path' \
    'def test_one():' '    Path("marker").touch()' '    sum(range(10**10))' \
    'def test_two():' '    Path("two").touch()' >test_c_first.py

  run -0 --separate-stderr interrupt test_c_first.py 0 INT INT

  [[ -z "${output}" ]] || false
  [[ "${stderr}" == "${SIGINT_LINE}"$'\npurba ended by SIGINT' ]] || false
  [[ ! -e two ]]
}

@test "SIGINT then SIGTERM end purba at once by SIGTERM, when the first lands in a test's C code" {
  cd "${BATS_TEST_TMPDIR:?set by bats}"
  printf '%s\n' 'from pathlib import Path' \
    'def test_one():' '    Path("marker").touch()' '    sum(range(10**10))' \
    'def test_two():' '    Path("two").touch()' >test_c_then_term.py

  run -0 --separate-stderr interrupt test_c_then_term.py 0 INT TERM

  [[ -z "${output}" ]] || false
  [[ "${stderr}" == "${SIGINT_LINE}"$'\npurba ended by SIGTERM' ]] || false
  [[ ! -e two ]]
}

@test "two signals that wait at once in a test's process count as one interrupt" {
  cd "${BATS_TEST_TMPDIR:?set by bats}"
  printf '%s\n' 'import os, signal' \
    'def test_one():' '    both = {signal.SIGINT, signal.SIGTERM}' \
    '    signal.pthread_sigmask(signal.SIG_BLOCK, both)' \
    '    os.kill(os.getpid(), signal.SIGINT)' '    os.kill(os.getpid(), signal.SIGTERM)' \
    '    signal.pthread_sigmask(signal.SIG_UNBLOCK, both)' \
    'def test_two():' '    assert 1 == 1' >test_waiting.py

  run -0 --separate-stderr interrupt test_waiting.py 0 none

  diff -u - <(printf '%s\n' "${output}") <<'LINES'
test_waiting.py::test_one interrupted
2 cases: 1 interrupted, 1 unreached
LINES
  [[ "${stderr##*$'\n'}" =~ ^purba\ ended\ by\ SIG(INT|TERM)$ ]]
}

@test "a Ctrl-C ends the wait for a thread that a test left alive, and purba dies by SIGINT" {
  cd "${BATS_TEST_TMPDIR:?set by bats}"
  printf '%s\n' 'import threading, time' 'from pathlib import Path' \
    'def linger():' '    time.sleep(10)' '    Path("thread").touch()' \
    'def test_one():' '    threading.Thread(target=linger).start()' \
    '    Path("marker").touch()' '    assert 1 == 1' >test_thread.py

  run -0 --separate-stderr interrupt test_thread.py 0

  diff -u - <(printf '%s\n' "${output}") <<'LINES'
test_thread.py::test_one passed
1 case: 1 passed
LINES
  [[ "${stderr%%$'\n'*}" == "${SIGINT_LINE}" ]] || false
  [[ "${stderr##*$'\n'}" == 'purba ended by SIGINT' ]] || false
  [[ ! -e thread ]]
}

@test "the error of a thread in the test that a Ctrl-C lands in prints after that test's line" {
  cd "${BATS_TEST_TMPDIR:?set by bats}"
  printf '%s\n' 'import threading, time' 'from pathlib import Path' \
    'go = threading.Event()' \
    'def late():' '    go.wait()' '    raise ValueError("late")' \
    'def test_one():' '    assert 1 == 1' \
    'def test_two():' '    thread = threading.Thread(target=late, name="T")' '    thread.start()' \
    '    try:' '        Path("marker").touch()' '        time.sleep(3)' \
    '    finally:' '        go.set()' '        thread.join()' \
    'def test_three():' '    Path("three").touch()' >test_thread_error.py

  run -0 --separate-stderr interrupt test_thread_error.py 0

  diff -u - <(printf '%s\n' "${output}") <<'LINES'
test_thread_error.py::test_one passed
test_thread_error.py::test_two interrupted
test_thread_error.py::test_two project error: thread T raised ValueError: late
3 cases: 1 passed, 1 interrupted, 1 unreached; 1 project error
LINES
  [[ $'\n'"${stderr}" == *$'\n'"${SIGINT_LINE}"$'\n'* ]] || false
  [[ "${stderr##*$'\n'}" == 'purba ended by SIGINT' ]] || false
  [[ ! -e three ]]
}

@test "a Ctrl-C in a test's atexit function lets the function finish, and purba dies by SIGINT" {
  cd "${BATS_TEST_TMPDIR:?set by bats}"
  printf '%s\n' 'import atexit, time' 'from pathlib import Path' \
    'def linger():' '    Path("marker").touch()' '    time.sleep(2)' '    Path("finally").touch()' \
    'def test_one():' '    atexit.register(linger)' '    assert 1 == 1' >test_atexit.py

  run -0 --separate-stderr interrupt test_atexit.py 0

  diff -u - <(printf '%s\n' "${output}") <<'LINES'
test_atexit.py::test_one passed
1 case: 1 passed
LINES
  [[ "${stderr}" == "${SIGINT_LINE}"$'\npurba ended by SIGINT' ]] || false
  [[ -e finally ]]
}

@test "a child that a test forks dies by SIGTERM, and prints no report of its own" {
  cd "${BATS_TEST_TMPDIR:?set by bats}"
  printf '%s\n' 'import os, signal, time' \
    'def exit_now(code):' '    os._exit(code)' \
    'def child():' '    time.sleep(30)' '    exit_now(0)' \
    'def test_child_dies_by_sigterm():' '    pid = os.fork()' '    {0: child}.get(pid, int)()' \
    '    time.sleep(0.5)' \
    '    os.kill(pid, signal.SIGTERM)' '    _, status = os.waitpid(pid, 0)' \
    '    assert os.waitstatus_to_exitcode(status) == -15' \
    'def test_two():' '    assert 1 == 1' >test_fork_term.py

  run -0 --separate-stderr purba test_fork_term.py

  diff -u - <(printf '%s\n' "${output}") <<'LINES'
test_fork_term.py::test_child_dies_by_sigterm passed
test_fork_term.py::test_two passed
2 cases: 2 passed
LINES
  [[ -z "${stderr}" ]]
}

@test "a child that multiprocessing forks dies by SIGTERM on terminate" {
  cd "${BATS_TEST_TMPDIR:?set by bats}"
  printf '%s\n' 'import multiprocessing, signal, time' \
    'def sleep():' '    time.sleep(30)' \
    'def test_terminate():' \
    '    child = multiprocessing.get_context("fork").Process(target=sleep)' \
    '    child.start()' '    time.sleep(0.5)' '    child.terminate()' '    child.join()' \
    '    assert child.exitcode == -15' >test_fork_terminate.py

  run -0 --separate-stderr purba test_fork_terminate.py

  diff -u - <(printf '%s\n' "${output}") <<'LINES'
test_fork_terminate.py::test_terminate passed
1 case: 1 passed
LINES
  [[ -z "${stderr}" ]]
}

@test "a child that a test forks has the signal handlers that Python has without purba" {
  cd "${BATS_TEST_TMPDIR:?set by bats}"
  printf '%s\n' 'import os, signal, sys' \
    'def exit_now(code):' '    os._exit(code)' \
    'READ = ("import signal as s; "' \
    '        "print([s.getsignal(n) for n in (s.SIGINT, s.SIGTERM, s.SIGHUP)])")' \
    'def in_child(write):' '    os.dup2(write, 1)' '    exec(READ)' \
    '    sys.stdout.flush()' '    exit_now(0)' \
    'def test_handlers():' \
    '    read, write = os.pipe()' '    pid = os.fork()' '    {0: in_child}.get(pid, int)(write)' \
    '    os.close(write)' '    os.waitpid(pid, 0)' \
    '    with os.fdopen(read) as child:' '        got = child.read()' \
    '    assert got == ("[<built-in function default_int_handler>, "' \
    '                   "<Handlers.SIG_DFL: 0>, <Handlers.SIG_DFL: 0>]\n")' >test_fork_handlers.py

  run -0 --separate-stderr purba test_fork_handlers.py

  diff -u - <(printf '%s\n' "${output}") <<'LINES'
test_fork_handlers.py::test_handlers passed
1 case: 1 passed
LINES
  [[ -z "${stderr}" ]]
}

@test "a signal handler that a test gives SIGTERM before it forks runs in the child" {
  cd "${BATS_TEST_TMPDIR:?set by bats}"
  printf '%s\n' 'import os, signal, time' \
    'def exit_now(code):' '    os._exit(code)' \
    'def child():' '    time.sleep(30)' '    exit_now(0)' \
    'def test_own_handler():' \
    '    signal.signal(signal.SIGTERM, lambda signum, frame: exit_now(9))' \
    '    pid = os.fork()' '    {0: child}.get(pid, int)()' \
    '    time.sleep(0.5)' '    os.kill(pid, signal.SIGTERM)' '    _, status = os.waitpid(pid, 0)' \
    '    assert os.waitstatus_to_exitcode(status) == 9' >test_fork_own_handler.py

  run -0 --separate-stderr purba test_fork_own_handler.py

  diff -u - <(printf '%s\n' "${output}") <<'LINES'
test_fork_own_handler.py::test_own_handler passed
1 case: 1 passed
LINES
  [[ -z "${stderr}" ]]
}
