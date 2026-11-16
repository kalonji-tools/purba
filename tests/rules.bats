#   the decision:
# docs/decisions/a-test-returns-nothing-and-the-first-part-that-sees-a-breach-refuses-it.md
#
# Each rule that execution sees while a case runs, in one run for each case.
# Each case matches the exit code and the whole stdout and stderr.
#
# bash 3.2 on macOS fails a test at a `[[ ]]` only on its last line, so `run -N`
# checks each exit code, and `|| false` ends each other check.
# https://bats-core.readthedocs.io/en/stable/gotchas.html#or-did-not-fail-my-test

: "${BATS_TEST_DIRNAME:?set by bats}"
bats_require_minimum_version 1.5.0

@test "a case that breaks a rule prints the line and the rule, and a cleanup that raises fails" {
  cd "${BATS_TEST_DIRNAME}/rules"

  run -1 --separate-stderr purba test_m.py

  local path='the lines before the first expectation take one path'
  local asserts='asserts before the first expectation, and only an expectation asserts'
  local hooks="a test keeps purba's hooks"
  local swallowed='the with at line 39 swallowed a raise'
  diff -u - <(printf '%s\n' "${output}") <<LINES
test_m.py::test_path project error: line 15 did not run, and ${path}
test_m.py::test_branch project error: line 21 went one way only, and ${path}
test_m.py::test_assert project error: line 28 ${asserts}
test_m.py::test_hook project error: line 34 replaced threading.excepthook, and ${hooks}
test_m.py::test_swallow project error: ${swallowed}, and line 41 did not run
test_m.py::test_cleanup failed
    the cleanup of the with at line 46 raised AssertionError: a handle leaked
6 cases: 1 failed, 5 blocked; 5 project errors
LINES
  [[ -z "${stderr}" ]]
}

@test "a with whose entry raises fails the case, ahead of a line that did not run" {
  cd "${BATS_TEST_TMPDIR:?set by bats}"
  printf '%s\n' 'class Broken:' '    def __enter__(self):' \
    '        raise ValueError("no entry")' '' \
    '    def __exit__(self, *raised):' '        return False' '' '' \
    'def test_one():' '    with Broken():' '        total = 1' '    assert total == 1' \
    >test_entry.py

  run -1 --separate-stderr purba test_entry.py

  diff -u - <(printf '%s\n' "${output}") <<'LINES'
test_entry.py::test_one failed
    the entry of the with at line 10 raised ValueError: no entry
1 case: 1 failed
LINES
  [[ -z "${stderr}" ]]
}

@test "an expectation inside a with reads before its cleanup, and one after it reads after" {
  cd "${BATS_TEST_TMPDIR:?set by bats}"
  printf '%s\n' 'import contextlib' '' 'log = []' '' '' \
    '@contextlib.contextmanager' 'def opened():' '    log.append("open")' '    yield' \
    '    log.append("closed")' '' '' \
    'def test_inside():' '    with opened():' '        assert log == ["open"]' '' '' \
    'def test_after():' '    with opened():' '        log.append("used")' \
    '    assert log == ["open", "used", "closed"]' >test_with.py

  run -0 --separate-stderr purba test_with.py

  diff -u - <(printf '%s\n' "${output}") <<'LINES'
test_with.py::test_inside passed
test_with.py::test_after passed
2 cases: 2 passed
LINES
  [[ -z "${stderr}" ]]
}

@test "an assert before the first expectation beats a raise, and a raise keeps raised" {
  cd "${BATS_TEST_TMPDIR:?set by bats}"
  printf '%s\n' 'def test_asserts():' '    total = 1' '    assert total == 1' '    {}["k"]' \
    '    assert total == 1' '' '' \
    'def test_raises():' '    {}["k"]' '    total = 1' '    assert total == 1' >test_order.py

  run -4 --separate-stderr purba test_order.py

  local asserts='asserts before the first expectation, and only an expectation asserts'
  diff -u - <(printf '%s\n' "${output}") <<LINES
test_order.py::test_asserts project error: line 3 ${asserts}
test_order.py::test_raises raised: KeyError: 'k'
2 cases: 1 raised, 1 blocked; 1 project error
LINES
  [[ -z "${stderr}" ]]
}

@test "a helper of the test module takes one path, unless the test calls it both ways" {
  cd "${BATS_TEST_TMPDIR:?set by bats}"
  printf '%s\n' 'def pick(value):' '    if value:' '        return value' '    return 0' '' '' \
    'def test_one_way():' '    total = pick(1)' '    assert total == 1' '' '' \
    'def test_both_ways():' '    total = pick(1) + pick(0)' '    assert total == 1' >test_helper.py

  run -4 --separate-stderr purba test_helper.py

  local path='the lines before the first expectation take one path'
  local one_way="line 8 reaches line 2 of pick, which went one way only, and ${path}"
  local not_run="line 8 reaches line 4 of pick, which did not run, and ${path}"
  diff -u - <(printf '%s\n' "${output}") <<LINES
test_helper.py::test_one_way project error: ${one_way}; ${not_run}
test_helper.py::test_both_ways passed
2 cases: 1 passed, 1 blocked; 1 project error
LINES
  [[ -z "${stderr}" ]]
}

@test "a library's callback, an imported module, and a raise at the end of a with are free" {
  cd "${BATS_TEST_TMPDIR:?set by bats}"
  printf '%s\n' 'def pick(value):' '    if value:' '        return value' '    return 0' >picks.py
  printf '%s\n' 'import contextlib' '' 'import picks' '' 'seen = []' '' '' \
    'def keep(value):' '    if value:' '        seen.append(value)' '' '' \
    'def test_callback():' '    total = picks.pick(1)' '    with contextlib.ExitStack() as stack:' \
    '        stack.callback(keep, total)' '    assert seen == [1]' '' '' \
    'def test_raise_at_the_end():' '    with contextlib.suppress(KeyError):' '        total = 1' \
    '        {}["k"]' '    assert total == 1' >test_free.py

  run -0 --separate-stderr env PYTHONPATH=. purba test_free.py

  diff -u - <(printf '%s\n' "${output}") <<'LINES'
test_free.py::test_callback passed
test_free.py::test_raise_at_the_end passed
2 cases: 2 passed
LINES
  [[ -z "${stderr}" ]]
}

@test "a hook that a helper replaces and puts back inside one call is free" {
  cd "${BATS_TEST_TMPDIR:?set by bats}"
  printf '%s\n' 'import sys' '' '' 'def quiet():' '    saved = sys.unraisablehook' \
    '    sys.unraisablehook = print' '    sys.unraisablehook = saved' '' '' \
    'def test_one():' '    quiet()' '    assert 1 == 1' >test_restores.py

  run -0 --separate-stderr purba test_restores.py

  diff -u - <(printf '%s\n' "${output}") <<'LINES'
test_restores.py::test_one passed
1 case: 1 passed
LINES
  [[ -z "${stderr}" ]]
}

@test "a module that replaces a hook as it loads is refused, and blocks each of its tests" {
  cd "${BATS_TEST_TMPDIR:?set by bats}"
  printf '%s\n' 'import sys' '' 'sys.unraisablehook = print' '' '' \
    'def test_one():' '    assert 1 == 1' >test_load_hook.py

  run -4 --separate-stderr purba test_load_hook.py

  local replaced='the module replaced sys.unraisablehook as it loaded'
  diff -u - <(printf '%s\n' "${output}") <<LINES
test_load_hook.py project error: ${replaced}, and a test keeps purba's hooks
test_load_hook.py::test_one blocked: test_load_hook.py did not load
1 case: 1 blocked; 1 project error
LINES
  [[ -z "${stderr}" ]]
}

@test "a thread alive at the first expectation gives a replay for each later expectation" {
  cd "${BATS_TEST_TMPDIR:?set by bats}"
  printf '%s\n' 'import threading' '' '' 'def test_one():' \
    '    threading.Thread(target=threading.Event().wait, daemon=True).start()' \
    '    with open("acts.txt", "a") as acts:' '        acts.write("act\n")' \
    '    assert 1 == 1' '    assert 2 == 2' '    assert 3 == 3' >test_replay.py

  run -0 --separate-stderr purba test_replay.py

  diff -u - <(printf '%s\n' "${output}") <<'LINES'
test_replay.py::test_one passed
1 case: 1 passed
LINES
  [[ -z "${stderr}" ]] || false
  diff -u - acts.txt <<'LINES'
act
act
act
LINES
}

@test "a replay that reads an expectation alone refuses one that a reading before it changed" {
  cd "${BATS_TEST_TMPDIR:?set by bats}"
  printf '%s\n' 'import threading' '' 'seen = []' '' '' 'def test_one():' \
    '    threading.Thread(target=threading.Event().wait, daemon=True).start()' \
    '    assert seen.append(1) is None' '    assert seen == [1]' >test_thread.py

  run -4 --separate-stderr purba test_thread.py

  local after='holds after the expectations before it, and not when read alone'
  diff -u - <(printf '%s\n' "${output}") <<LINES
test_thread.py::test_one project error: assert seen == [1] ${after}
1 case: 1 blocked; 1 project error
LINES
  [[ -z "${stderr}" ]]
}

@test "a helper that breaks a rule names the line of the test that reaches it first" {
  cd "${BATS_TEST_DIRNAME}/rules"

  run -4 --separate-stderr purba test_reach.py

  local path='the lines before the first expectation take one path'
  local one_way="reaches line 16 of make, which went one way only, and ${path}"
  local not_run="reaches line 18 of make, which did not run, and ${path}"
  local reading='asserts inside a reading, and only an expectation asserts'
  local opened="reaches line 6 of opened, which went one way only, and ${path}"
  diff -u - <(printf '%s\n' "${output}") <<LINES
test_reach.py::test_cm project error: line 22 ${opened}
test_reach.py::test_reading project error: line 29 reaches line 12 of check_user, which ${reading}
test_reach.py::test_helper project error: line 33 ${one_way}; line 33 ${not_run}
test_reach.py::test_two_lines project error: line 38 ${one_way}; line 38 ${not_run}
test_reach.py::test_stack project error: line 45 ${opened}
test_reach.py::test_kept passed
test_reach.py::test_inside passed
7 cases: 2 passed, 5 blocked; 5 project errors
LINES
  [[ -z "${stderr}" ]]
}

@test "execution refuses an assert that only a fork or a replay reaches" {
  cd "${BATS_TEST_TMPDIR:?set by bats}"
  printf '%s\n' 'import threading' '' 'seen = []' '' '' 'def check():' '    if not seen:' \
    '        assert seen == []' '    return True' '' '' \
    'def test_reader():' '    assert seen.append(1) is None' '    assert check() is True' '' '' \
    'def test_replay():' \
    '    threading.Thread(target=threading.Event().wait, daemon=True).start()' \
    '    assert seen.append(1) is None' '    assert check() is True' >test_alone.py

  run -4 --separate-stderr purba test_alone.py

  local alone='reaches an assert when read alone, and only an expectation asserts'
  diff -u - <(printf '%s\n' "${output}") <<LINES
test_alone.py::test_reader project error: assert check() is True ${alone}
test_alone.py::test_replay project error: assert check() is True ${alone}
2 cases: 2 blocked; 2 project errors
LINES
  [[ -z "${stderr}" ]]
}
