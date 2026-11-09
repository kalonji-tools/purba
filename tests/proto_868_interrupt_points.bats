#!/usr/bin/env bats
# PROTOTYPE #868, seam "a fault setting raises SIGINT at each point". Never merge.
#
# `FAULT_PURBA` names the purba of the wheel built with the feature `fault`.

: "${BATS_TEST_DIRNAME:?set by bats}"
bats_require_minimum_version 1.5.0

# Runs the fault purba on <module> with PURBA_FAULT=<setting>, prints stdout,
# stderr, then how purba ended: Python's returncode tells a death by a signal
# from an exit with 130.
ended() {
  local fault_purba="${FAULT_PURBA:?set by mise run test:wheel}"
  "${fault_purba%/*}/python" - "${fault_purba}" "$@" <<'PY'
import os, signal, subprocess, sys
purba, setting, module = sys.argv[1:]
env = dict(os.environ, PURBA_FAULT=setting)
child = subprocess.run([purba, module], capture_output=True, env=env, timeout=30)
sys.stdout.write(child.stdout.decode())
sys.stdout.write("--stderr--\n" + child.stderr.decode())
code = child.returncode
print(f"--end-- ended by {signal.Signals(-code).name}" if code < 0 else f"--end-- exited {code}")
PY
}

setup() {
  cd "${BATS_TEST_TMPDIR:?set by bats}"
  printf '%s\n' 'async def test_refused():' '    assert True' \
    'def test_one():' '    assert True' \
    'def test_two():' '    assert True' >test_points.py
}

@test "PROTOTYPE: what the tester sees for an interrupt at each point" {
  local table="${BATS_TEST_DIRNAME}/../proto-868/points.txt"
  : >"${table}"
  for point in command parse selection rewrite assert; do
    printf '=== PURBA_FAULT=%s:interrupt\n' "${point}" >>"${table}"
    ended "${point}:interrupt" test_points.py >>"${table}"
  done
  cat "${table}"
}

@test "an interrupt in the rewrite prints the refused line and the count, and purba dies by SIGINT" {
  run -0 ended rewrite:interrupt test_points.py

  diff -u - <(printf '%s\n' "${output}") <<'LINES'
test_points.py::test_refused project error: purba cannot run an async test yet
3 tests: 1 blocked, 2 unreached; 1 project error
--stderr--
purba: interrupt: SIGINT; a second interrupt exits at once
--end-- ended by SIGINT
LINES
}
