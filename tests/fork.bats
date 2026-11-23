# PROTO 1154: each fork that purba makes fails once the file `refuse` exists.
# Route `preload` loads tests/fork/refuse-fork.c into the purba of the wheel.
# Route `fault` sets PURBA_FAULT=fork on the purba of the fault wheel.
#
# bash 3.2 on macOS fails a test at a `[[ ]]` only on its last line, so `run -N`
# checks each exit code, and `|| false` ends each other check.

: "${BATS_TEST_DIRNAME:?set by bats}"
bats_require_minimum_version 1.5.0

setup_file() {
  local out="${BATS_FILE_TMPDIR:?set by bats}" os purba
  os=$(uname)
  purba=$(command -v purba)
  export PYTHON="${purba%/*}/python"
  if [[ "${os}" == Darwin ]]; then
    cc -dynamiclib -o "${out}/refuse-fork.dylib" "${BATS_TEST_DIRNAME}/fork/refuse-fork.c"
    export REFUSE_LIB="${out}/refuse-fork.dylib" PRELOAD_VAR=DYLD_INSERT_LIBRARIES
  else
    cc -shared -fPIC -o "${out}/refuse-fork.so" "${BATS_TEST_DIRNAME}/fork/refuse-fork.c" -ldl
    export REFUSE_LIB="${out}/refuse-fork.so" PRELOAD_VAR=LD_PRELOAD
  fi
  # How Rust prints EAGAIN on this platform.
  EAGAIN=$("${PYTHON}" -c \
    'import errno, os; print(f"{os.strerror(errno.EAGAIN)} (os error {errno.EAGAIN})")')
  export EAGAIN
}

# Runs <purba> on <module> with the variables NAME=VALUE before them, in a
# session of its own. It kills that session 30 seconds on, and exits 124.
bounded() {
  "${PYTHON}" - "$@" <<'PY'
import os, signal, subprocess, sys

*pairs, purba, module = sys.argv[1:]
env = dict(os.environ, **dict(pair.split("=", 1) for pair in pairs))
child = subprocess.Popen([purba, module], env=env, start_new_session=True)
try:
    sys.exit(child.wait(timeout=30))
except subprocess.TimeoutExpired:
    os.killpg(child.pid, signal.SIGKILL)
    child.wait()
    sys.exit(124)
PY
}

# Runs purba on <module> by <route>, with fork refused once `refuse` exists.
refusing() {
  local route="$1" module="$2" purba
  case "${route}" in
    preload)
      purba=$(command -v purba)
      bounded REFUSE_FORK_FILE="${PWD}/refuse" "${PRELOAD_VAR}=${REFUSE_LIB}" "${purba}" "${module}"
      ;;
    fault)
      bounded PURBA_FAULT=fork PURBA_FAULT_FILE="${PWD}/refuse" \
        "${FAULT_PURBA:?set by mise run test:wheel and the wheel job}" "${module}"
      ;;
    *) return 2 ;;
  esac
}

arm1() {
  printf '%s\n' 'def test_one():' '    assert 1 == 1' >test_arm.py
  : >refuse
}

arm2() {
  printf '%s\n' 'open("refuse", "w").close()' 'def test_one():' '    assert 1 == 1' >test_arm.py
}

arm3() {
  printf '%s\n' 'import threading' 'def test_one():' \
    '    threading.Thread(target=threading.Event().wait, daemon=True).start()' \
    '    open("refuse", "w").close()' '    assert 1 == 2' '    assert 2 == 2' >test_arm.py
}

# The order puts a raise before a refused replay, and an error before both.
arm3b() {
  printf '%s\n' 'import threading' 'def boom():' '    raise ValueError("boom")' 'def test_one():' \
    '    threading.Thread(target=threading.Event().wait, daemon=True).start()' \
    '    open("refuse", "w").close()' '    assert 1 == 1' '    assert 2 == 2' \
    '    assert boom() is True' >test_arm.py
}

check_arm3b() {
  run -4 --separate-stderr refusing "$1" test_arm.py
  diff -u - <(printf '%s\n' "${output}") <<'LINES'
test_arm.py::test_one raised: ValueError: boom
    raised in boom() at test_arm.py:3
1 case: 1 raised
LINES
}

arm4() {
  printf '%s\n' 'def test_one():' \
    '    open("refuse", "w").close()' '    assert 1 == 2' '    assert 2 == 2' >test_arm.py
}

check_arm1() {
  run -4 --separate-stderr refusing "$1" test_arm.py
  diff -u - <(printf '%s\n' "${output}") <<LINES
test_arm.py project error: purba cannot fork: ${EAGAIN}
test_arm.py::test_one blocked: test_arm.py did not load
1 case: 1 blocked; 1 project error
LINES
}

check_arm2() {
  run -4 --separate-stderr refusing "$1" test_arm.py
  diff -u - <(printf '%s\n' "${output}") <<LINES
test_arm.py::test_one project error: purba cannot fork: ${EAGAIN}
1 case: 1 blocked; 1 project error
LINES
}

check_alone() {
  run -4 --separate-stderr refusing "$1" test_arm.py
  diff -u - <(printf '%s\n' "${output}") <<LINES
test_arm.py::test_one project error: purba cannot fork to read an expectation alone: ${EAGAIN}
1 case: 1 blocked; 1 project error
LINES
}

@test "preload ①: a refused fork of the zygote" {
  cd "${BATS_TEST_TMPDIR:?set by bats}"
  arm1
  check_arm1 preload
}

@test "preload ②: a refused fork of a test" {
  cd "${BATS_TEST_TMPDIR:?set by bats}"
  arm2
  check_arm2 preload
}

@test "preload ③: a refused fork of a replay" {
  cd "${BATS_TEST_TMPDIR:?set by bats}"
  arm3
  check_alone preload
}

@test "preload ④: a refused fork of a reader" {
  cd "${BATS_TEST_TMPDIR:?set by bats}"
  arm4
  check_alone preload
}

@test "fault ①: a refused fork of the zygote" {
  cd "${BATS_TEST_TMPDIR:?set by bats}"
  arm1
  check_arm1 fault
}

@test "fault ②: a refused fork of a test" {
  cd "${BATS_TEST_TMPDIR:?set by bats}"
  arm2
  check_arm2 fault
}

@test "fault ③: a refused fork of a replay" {
  cd "${BATS_TEST_TMPDIR:?set by bats}"
  arm3
  check_alone fault
}

@test "fault ④: a refused fork of a reader" {
  cd "${BATS_TEST_TMPDIR:?set by bats}"
  arm4
  check_alone fault
}

@test "preload ③b: a raise beats a refused replay" {
  cd "${BATS_TEST_TMPDIR:?set by bats}"
  arm3b
  check_arm3b preload
}

@test "fault ③b: a raise beats a refused replay" {
  cd "${BATS_TEST_TMPDIR:?set by bats}"
  arm3b
  check_arm3b fault
}

@test "control: no refuse file, each route passes" {
  cd "${BATS_TEST_TMPDIR:?set by bats}"
  printf '%s\n' 'def test_one():' '    assert 1 == 1' >test_arm.py
  run -0 --separate-stderr refusing preload test_arm.py
  run -0 --separate-stderr refusing fault test_arm.py
}
