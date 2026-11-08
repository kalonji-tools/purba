#   the decision:
# docs/decisions/a-test-causes-an-internal-error-through-a-switch-that-only-a-test-build-carries.md
#
# `FAULT_PURBA` names the purba of the wheel built with the feature `fault`.

: "${BATS_TEST_DIRNAME:?set by bats}"
bats_require_minimum_version 1.5.0
load internal_error.sh

@test "the fault wheel panics in the assert hook when PURBA_FAULT is assert" {
  cd "${BATS_TEST_DIRNAME}/fault"

  local fault_purba="${FAULT_PURBA:?set by mise run test:wheel and the wheel job}"
  run -125 --separate-stderr env PURBA_FAULT=assert "${fault_purba}" test_pass.py

  diff -u - <(printf '%s\n' "${output}") <<'LINES'
test_pass.py::test_one internal error: assertion: PURBA_FAULT=assert
1 test: 1 blocked; 1 internal error
LINES
  grep -Fq -- 'panicked at crates/assertion/src/lib.rs:' <<<"${stderr}"
  grep -Fxq -- 'PURBA_FAULT=assert' <<<"${stderr}"
  local expected last
  expected=$(report_lines "${fault_purba}")
  last=$(tail -n 2 <<<"${stderr}")
  diff -u <(printf '%s\n' "${expected}") <(printf '%s\n' "${last}")
}

@test "the fault wheel panics in command when PURBA_FAULT is command" {
  cd "${BATS_TEST_DIRNAME}/fault"

  local fault_purba="${FAULT_PURBA:?set by mise run test:wheel and the wheel job}"
  run -125 --separate-stderr env PURBA_FAULT=command "${fault_purba}" test_pass.py

  [[ -z "${output}" ]] || false
  grep -Fq -- 'panicked at crates/command/src/lib.rs:' <<<"${stderr}"
  local expected last
  expected=$(internal_error_lines "${fault_purba}" command PURBA_FAULT=command)
  last=$(tail -n 3 <<<"${stderr}")
  diff -u <(printf '%s\n' "${expected}") <(printf '%s\n' "${last}")
}

@test "the fault wheel passes the test when PURBA_FAULT is unset" {
  cd "${BATS_TEST_DIRNAME}/fault"

  local fault_purba="${FAULT_PURBA:?set by mise run test:wheel and the wheel job}"
  run -0 --separate-stderr env -u PURBA_FAULT "${fault_purba}" test_pass.py

  diff -u - <(printf '%s\n' "${output}") <<'LINES'
test_pass.py::test_one passed
1 test: 1 passed
LINES
  [[ -z "${stderr}" ]]
}

@test "the wheel without fault passes the test when PURBA_FAULT is assert" {
  cd "${BATS_TEST_DIRNAME}/fault"

  run -0 --separate-stderr env PURBA_FAULT=assert purba test_pass.py

  diff -u - <(printf '%s\n' "${output}") <<'LINES'
test_pass.py::test_one passed
1 test: 1 passed
LINES
  [[ -z "${stderr}" ]]
}

@test "the wheel without fault passes the test when PURBA_FAULT is command" {
  cd "${BATS_TEST_DIRNAME}/fault"

  run -0 --separate-stderr env PURBA_FAULT=command purba test_pass.py

  diff -u - <(printf '%s\n' "${output}") <<'LINES'
test_pass.py::test_one passed
1 test: 1 passed
LINES
  [[ -z "${stderr}" ]]
}

@test "the fault wheel names execution for purba's own PanicException that a test raises" {
  cd "${BATS_TEST_DIRNAME}/fault"

  local fault_purba="${FAULT_PURBA:?set by mise run test:wheel and the wheel job}"
  run -125 --separate-stderr env -u PURBA_FAULT "${fault_purba}" test_panic_exception.py

  diff -u - <(printf '%s\n' "${output}") <<'LINES'
test_panic_exception.py::test_raises_it internal error: execution: raised by the test
1 test: 1 blocked; 1 internal error
LINES
  local expected last
  expected=$(report_lines "${fault_purba}")
  last=$(tail -n 2 <<<"${stderr}")
  diff -u <(printf '%s\n' "${expected}") <(printf '%s\n' "${last}")
}

@test "a test that catches the panic still gets its internal error, and the next test its own" {
  cd "${BATS_TEST_DIRNAME}/fault"

  local fault_purba="${FAULT_PURBA:?set by mise run test:wheel and the wheel job}"
  run -125 --separate-stderr env PURBA_FAULT=assert "${fault_purba}" test_stale_note.py

  diff -u - <(printf '%s\n' "${output}") <<'LINES'
test_stale_note.py::test_catches internal error: assertion: PURBA_FAULT=assert
test_stale_note.py::test_raises internal error: execution: raised by the second test
2 tests: 2 blocked; 2 internal errors
LINES
  local expected last
  expected=$(report_lines "${fault_purba}")
  last=$(tail -n 2 <<<"${stderr}")
  diff -u <(printf '%s\n' "${expected}") <(printf '%s\n' "${last}")
}

@test "a module that catches the panic while it loads gets its internal error, and no test runs" {
  cd "${BATS_TEST_DIRNAME}/fault"

  local fault_purba="${FAULT_PURBA:?set by mise run test:wheel and the wheel job}"
  run -125 --separate-stderr env -u PURBA_FAULT "${fault_purba}" test_load_catches_panic.py

  diff -u - <(printf '%s\n' "${output}") <<'LINES'
test_load_catches_panic.py internal error: assertion: PURBA_FAULT=assert
test_load_catches_panic.py::test_one blocked: test_load_catches_panic.py did not load
test_load_catches_panic.py::test_two blocked: test_load_catches_panic.py did not load
2 tests: 2 blocked; 1 internal error
LINES
  local expected last
  expected=$(report_lines "${fault_purba}")
  last=$(tail -n 2 <<<"${stderr}")
  diff -u <(printf '%s\n' "${expected}") <(printf '%s\n' "${last}")
}

@test "the fault wheel gives the module an internal error when PURBA_FAULT is selection" {
  cd "${BATS_TEST_DIRNAME}/fault"

  local fault_purba="${FAULT_PURBA:?set by mise run test:wheel and the wheel job}"
  run -125 --separate-stderr env PURBA_FAULT=selection "${fault_purba}" test_pass.py

  diff -u - <(printf '%s\n' "${output}") <<'LINES'
test_pass.py internal error: selection: PURBA_FAULT=selection
0 tests; 1 internal error
LINES
  grep -Fq -- 'panicked at crates/selection/src/lib.rs:' <<<"${stderr}"
  local expected last
  expected=$(report_lines "${fault_purba}")
  last=$(tail -n 2 <<<"${stderr}")
  diff -u <(printf '%s\n' "${expected}") <(printf '%s\n' "${last}")
}

@test "the wheel without fault passes the test when PURBA_FAULT is selection" {
  cd "${BATS_TEST_DIRNAME}/fault"

  run -0 --separate-stderr env PURBA_FAULT=selection purba test_pass.py

  diff -u - <(printf '%s\n' "${output}") <<'LINES'
test_pass.py::test_one passed
1 test: 1 passed
LINES
  [[ -z "${stderr}" ]]
}

@test "the fault wheel blocks the selected tests when PURBA_FAULT is rewrite" {
  cd "${BATS_TEST_DIRNAME}/fault"

  local fault_purba="${FAULT_PURBA:?set by mise run test:wheel and the wheel job}"
  run -125 --separate-stderr env PURBA_FAULT=rewrite "${fault_purba}" test_rewrite_panic.py

  diff -u - <(printf '%s\n' "${output}") <<'LINES'
test_rewrite_panic.py::test_refused project error: purba cannot run an async test yet
test_rewrite_panic.py internal error: assertion: PURBA_FAULT=rewrite
test_rewrite_panic.py::test_one blocked: test_rewrite_panic.py did not load
2 tests: 2 blocked; 1 project error, 1 internal error
LINES
  grep -Fq -- 'panicked at crates/assertion/src/lib.rs:' <<<"${stderr}"
  local expected last
  expected=$(report_lines "${fault_purba}")
  last=$(tail -n 2 <<<"${stderr}")
  diff -u <(printf '%s\n' "${expected}") <(printf '%s\n' "${last}")
}

@test "the fault wheel gives the module an internal error when PURBA_FAULT is parse" {
  cd "${BATS_TEST_DIRNAME}/fault"

  local fault_purba="${FAULT_PURBA:?set by mise run test:wheel and the wheel job}"
  run -125 --separate-stderr env PURBA_FAULT=parse "${fault_purba}" test_rewrite_panic.py

  diff -u - <(printf '%s\n' "${output}") <<'LINES'
test_rewrite_panic.py internal error: parsing: PURBA_FAULT=parse
0 tests; 1 internal error
LINES
  grep -Fq -- 'panicked at crates/parsing/src/lib.rs:' <<<"${stderr}"
  local expected last
  expected=$(report_lines "${fault_purba}")
  last=$(tail -n 2 <<<"${stderr}")
  diff -u <(printf '%s\n' "${expected}") <(printf '%s\n' "${last}")
}

@test "the wheel without fault runs the module when PURBA_FAULT is rewrite or parse" {
  cd "${BATS_TEST_DIRNAME}/fault"

  local setting
  for setting in rewrite parse; do
    run -4 --separate-stderr env PURBA_FAULT="${setting}" purba test_rewrite_panic.py

    diff -u - <(printf '%s\n' "${output}") <<'LINES'
test_rewrite_panic.py::test_refused project error: purba cannot run an async test yet
test_rewrite_panic.py::test_one passed
2 tests: 1 passed, 1 blocked; 1 project error
LINES
    [[ -z "${stderr}" ]]
  done
}

@test "a run that met an internal error exits 125 when stdout then refuses a write" {
  cd "${BATS_TEST_DIRNAME}/fault"

  local fault_purba="${FAULT_PURBA:?set by mise run test:wheel and the wheel job}"
  # shellcheck disable=SC2016 # $1 is the inner shell's.
  run -125 --separate-stderr bash -c \
    'PURBA_FAULT=assert "$1" test_probe.py | head -n 1; exit "${PIPESTATUS[0]}"' _ "${fault_purba}"

  diff -u - <(printf '%s\n' "${output}") <<'LINES'
test_probe.py::test_one internal error: assertion: PURBA_FAULT=assert
LINES
  grep -Fq -- 'purba: invocation error: cannot write to stdout: ' <<<"${stderr}"
  local expected last
  expected=$(report_lines "${fault_purba}")
  last=$(tail -n 2 <<<"${stderr}")
  diff -u <(printf '%s\n' "${expected}") <(printf '%s\n' "${last}")
}

@test "the report of an internal error names the flag when sys.abiflags is missing" {
  local site="${BATS_TEST_TMPDIR:?set by bats}/site"
  mkdir "${site}"
  printf '%s\n' 'import sys' '' 'del sys.abiflags' >"${site}/sitecustomize.py"
  cd "${BATS_TEST_DIRNAME}/fault"

  local fault_purba="${FAULT_PURBA:?set by mise run test:wheel and the wheel job}"
  run -125 --separate-stderr \
    env PYTHONPATH="${site}" PURBA_FAULT=assert "${fault_purba}" test_pass.py

  diff -u - <(printf '%s\n' "${output}") <<'LINES'
test_pass.py::test_one internal error: assertion: PURBA_FAULT=assert
1 test: 1 blocked; 1 internal error
LINES
  local expected last
  expected=$(report_lines "${fault_purba}")
  last=$(tail -n 2 <<<"${stderr}")
  diff -u <(printf '%s\n' "${expected}") <(printf '%s\n' "${last}")
}

@test "the report of an internal error ignores a test that changed sys.abiflags" {
  cd "${BATS_TEST_DIRNAME}/fault"

  local fault_purba="${FAULT_PURBA:?set by mise run test:wheel and the wheel job}"
  run -125 --separate-stderr env PURBA_FAULT=assert "${fault_purba}" test_changes_abiflags.py

  diff -u - <(printf '%s\n' "${output}") <<'LINES'
test_changes_abiflags.py::test_one internal error: assertion: PURBA_FAULT=assert
1 test: 1 blocked; 1 internal error
LINES
  local expected last
  expected=$(report_lines "${fault_purba}")
  last=$(tail -n 2 <<<"${stderr}")
  diff -u <(printf '%s\n' "${expected}") <(printf '%s\n' "${last}")
}
