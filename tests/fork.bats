#   the decision:
# docs/decisions/a-test-of-the-wheel-makes-each-fork-fail-through-a-library-it-preloads.md
#
# bash 3.2 on macOS fails a test at a `[[ ]]` only on its last line. So `run -N`
# checks each exit code.
# https://bats-core.readthedocs.io/en/stable/gotchas.html#or-did-not-fail-my-test

: "${BATS_TEST_DIRNAME:?set by bats}"
bats_require_minimum_version 1.5.0

# A test that builds its module in its own folder runs it from that folder, the root.
setup() {
  printf '[tool.purba]\n' >"${BATS_TEST_TMPDIR:?set by bats}/pyproject.toml"
}

setup_file() {
  local out="${BATS_FILE_TMPDIR:?set by bats}" os purba
  os=$(uname)
  purba=$(command -v purba)
  export PURBA="${purba}" PYTHON="${purba%/*}/python"
  if [[ "${os}" == Darwin ]]; then
    cc -dynamiclib -o "${out}/refuse-fork.dylib" "${BATS_TEST_DIRNAME}/fork/refuse-fork.c"
    export REFUSE_LIB="${out}/refuse-fork.dylib" PRELOAD_VAR=DYLD_INSERT_LIBRARIES
  else
    cc -shared -fPIC -o "${out}/refuse-fork.so" "${BATS_TEST_DIRNAME}/fork/refuse-fork.c" -ldl
    export REFUSE_LIB="${out}/refuse-fork.so" PRELOAD_VAR=LD_PRELOAD
  fi
  EAGAIN=$("${PYTHON}" -c \
    'import errno, os; print(f"{os.strerror(errno.EAGAIN)} (os error {errno.EAGAIN})")')
  export EAGAIN
}

# Exits 124 when purba runs 30 seconds. macOS has no `timeout`.
refusing() {
  "${PYTHON}" - "${PRELOAD_VAR}=${REFUSE_LIB}" "REFUSE_FORK_FILE=${PWD}/refuse" <<'PY'
import os, signal, subprocess, sys

env = dict(os.environ, **dict(pair.split("=", 1) for pair in sys.argv[1:]))
child = subprocess.Popen([os.environ["PURBA"], "test_arm.py"], env=env, start_new_session=True)
try:
    code = child.wait(timeout=30)
except subprocess.TimeoutExpired:
    code = 124
try:
    os.killpg(child.pid, signal.SIGKILL)
except ProcessLookupError:
    pass
child.wait()
sys.exit(code)
PY
}

check() {
  diff -u - <(printf '%s\n' "${output}") || return 1
  [[ -z "${stderr}" ]]
}

@test "a refused fork of the zygote blocks each test of the module" {
  cd "${BATS_TEST_TMPDIR:?set by bats}"
  printf '%s\n' 'def test_one():' '    assert 1 == 1' \
    'def test_two():' '    assert 2 == 2' >test_arm.py
  : >refuse

  run -4 --separate-stderr refusing
  check <<LINES
test_arm.py project error: purba cannot fork: ${EAGAIN}
test_arm.py::test_one blocked: test_arm.py did not load
test_arm.py::test_two blocked: test_arm.py did not load
2 cases: 2 blocked; 1 project error
LINES
}

@test "a refused fork of a test is a project error of its case" {
  cd "${BATS_TEST_TMPDIR:?set by bats}"
  printf '%s\n' 'open("refuse", "w").close()' 'def test_one():' '    assert 1 == 1' >test_arm.py

  run -4 --separate-stderr refusing
  check <<LINES
test_arm.py::test_one project error: purba cannot fork: ${EAGAIN}
1 case: 1 blocked; 1 project error
LINES
}

@test "a refused fork of a replay, while a thread is alive, is a project error" {
  cd "${BATS_TEST_TMPDIR:?set by bats}"
  printf '%s\n' 'import threading' 'def test_one():' \
    '    threading.Thread(target=threading.Event().wait, daemon=True).start()' \
    '    open("refuse", "w").close()' '    assert 1 == 2' '    assert 2 == 2' >test_arm.py

  run -4 --separate-stderr refusing
  check <<LINES
test_arm.py::test_one project error: purba cannot fork to read an expectation alone: ${EAGAIN}
1 case: 1 blocked; 1 project error
LINES
}

@test "a raise decides the case before a refused fork of a replay" {
  cd "${BATS_TEST_TMPDIR:?set by bats}"
  printf '%s\n' 'import threading' 'def boom():' '    raise ValueError("boom")' 'def test_one():' \
    '    threading.Thread(target=threading.Event().wait, daemon=True).start()' \
    '    open("refuse", "w").close()' '    assert 1 == 1' '    assert 2 == 2' \
    '    assert boom() is True' >test_arm.py

  run -4 --separate-stderr refusing
  check <<'LINES'
test_arm.py::test_one raised: ValueError: boom
    raised in boom() at test_arm.py:3
1 case: 1 raised
LINES
}

@test "a refused fork of a reader, with no thread, is a project error" {
  cd "${BATS_TEST_TMPDIR:?set by bats}"
  printf '%s\n' 'def test_one():' \
    '    open("refuse", "w").close()' '    assert 1 == 2' '    assert 2 == 2' >test_arm.py

  run -4 --separate-stderr refusing
  check <<LINES
test_arm.py::test_one project error: purba cannot fork to read an expectation alone: ${EAGAIN}
1 case: 1 blocked; 1 project error
LINES
}

@test "the library refuses no fork while the file refuse is absent" {
  cd "${BATS_TEST_TMPDIR:?set by bats}"
  printf '%s\n' 'import threading' 'def test_thread():' \
    '    threading.Thread(target=threading.Event().wait, daemon=True).start()' \
    '    assert 1 == 2' '    assert 2 == 2' \
    'def test_plain():' '    assert 1 == 2' '    assert 2 == 2' >test_arm.py

  run -1 --separate-stderr refusing
  check <<'LINES'
test_arm.py::test_thread failed
    assert 1 == 2
      expected  2
      reading   1
test_arm.py::test_plain failed
    assert 1 == 2
      expected  2
      reading   1
2 cases: 2 failed
LINES
}
