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
# exists, and half a second later, it sends SIGINT to that group. It reads
# stdout only <hold> seconds after that. A pipe holds less than a line of
# 300 KB, so purba waits in its write until then. It prints purba's stdout and
# stderr, then a line on stderr that says how purba ended. With `ignored`,
# purba starts with SIGINT ignored.
interrupt() {
  local purba
  purba=$(command -v purba)
  "${purba%/*}/python" - "${purba}" "$@" <<'PY'
import os, signal, subprocess, sys, time

purba, module, hold = sys.argv[1:4]
disposition = signal.SIG_IGN if sys.argv[4:] == ["ignored"] else signal.SIG_DFL
child = subprocess.Popen(
    [purba, module],
    stdout=subprocess.PIPE,
    stderr=subprocess.PIPE,
    start_new_session=True,
    preexec_fn=lambda: signal.signal(signal.SIGINT, disposition),
)
deadline = time.monotonic() + 30
while not os.path.exists("marker"):
    if time.monotonic() > deadline:
        child.kill()
        sys.exit("the test never wrote the marker")
    time.sleep(0.01)
time.sleep(0.5)
os.killpg(child.pid, signal.SIGINT)
time.sleep(float(hold))
out, err = child.communicate()
sys.stdout.write(out.decode())
sys.stderr.write(err.decode())
code = child.returncode
sys.stderr.write(
    f"purba ended by {signal.Signals(-code).name}\n" if code < 0 else f"purba exited {code}\n"
)
PY
}

# The text of an exception of 300 KB, which a pipe cannot hold.
setup() {
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
  [[ "${stderr}" == $'purba: interrupt: SIGINT\npurba ended by SIGINT' ]] || false
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
  [[ "${stderr}" == $'purba: interrupt: SIGINT\npurba ended by SIGINT' ]]
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
  [[ "${stderr}" == $'purba: interrupt: SIGINT\npurba ended by SIGINT' ]] || false
  [[ ! -e two ]]
}

@test "a Ctrl-C in a test's call starts no later test, and purba dies by SIGINT" {
  cd "${BATS_TEST_TMPDIR:?set by bats}"
  printf '%s\n' 'import time' 'from pathlib import Path' \
    'def test_one():' '    assert True' \
    'def test_two():' '    Path("marker").touch()' '    time.sleep(3)' \
    'def test_three():' '    Path("three").touch()' >test_call.py

  run -0 --separate-stderr interrupt test_call.py 0

  diff -u - <(printf '%s\n' "${output}") <<'LINES'
test_call.py::test_one passed
test_call.py::test_two raised: KeyboardInterrupt:
3 tests: 1 passed, 1 raised, 1 unreached
LINES
  [[ "${stderr}" == $'purba: interrupt: SIGINT\npurba ended by SIGINT' ]] || false
  [[ ! -e three ]]
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
