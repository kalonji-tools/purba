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
  local asserts='an assert ran in the act; one test checks one state; check this state in a test'
  asserts+=' of its own'
  local hooks="replaced threading.excepthook, and a test keeps purba's hooks"
  local swallowed='the with at line 40 swallowed a raise'
  local patched='line 35 patches threading.excepthook, and a stand-in enters a test as an argument'
  diff -u - <(printf '%s\n' "${output}") <<LINES
test_m.py::test_path project error: [second-path] line 15 did not run, and ${path}
test_m.py::test_branch project error: [second-path] line 21 went one way only, and ${path}
test_m.py::test_assert project error: [assert-before-expectation] line 28: ${asserts}
test_m.py::test_hook project error: [hook-replaced] line 35 ${hooks}
    ⚠ suppressed [patched-name] ${patched}
      reason: execution must still refuse a hook that the test replaced
test_m.py::test_swallow project error: [swallowed-raise] ${swallowed}, and line 42 did not run
test_m.py::test_cleanup failed
    the cleanup of the with at line 47 raised AssertionError: a handle leaked
6 cases: 1 failed, 5 blocked; 5 project errors; 1 suppressed
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

  local asserts='an assert ran in the act; one test checks one state; check this state in a test'
  asserts+=' of its own'
  diff -u - <(printf '%s\n' "${output}") <<LINES
test_order.py::test_asserts project error: [assert-before-expectation] line 3: ${asserts}
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
test_helper.py::test_one_way project error: [second-path] ${one_way}; [second-path] ${not_run}
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
test_load_hook.py project error: [hook-replaced] ${replaced}, and a test keeps purba's hooks
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
test_thread.py::test_one project error: [readings-differ] assert seen == [1] ${after}
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
  local reach='line 29 reaches line 12 of check_user, which'
  local opened="reaches line 6 of opened, which went one way only, and ${path}"
  local split='[second-path]'
  diff -u - <(printf '%s\n' "${output}") <<LINES
test_reach.py::test_cm project error: ${split} line 22 ${opened}
test_reach.py::test_reading project error: [assert-in-reading] ${reach} ${reading}
test_reach.py::test_helper project error: ${split} line 33 ${one_way}; ${split} line 33 ${not_run}
test_reach.py::test_two_lines project error: ${split} line 38 ${one_way}; \
${split} line 38 ${not_run}
test_reach.py::test_stack project error: ${split} line 45 ${opened}
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
test_alone.py::test_reader project error: [assert-in-reading] assert check() is True ${alone}
test_alone.py::test_replay project error: [assert-in-reading] assert check() is True ${alone}
2 cases: 2 blocked; 2 project errors
LINES
  [[ -z "${stderr}" ]]
}

@test "a reading alone that reaches an assert gives one line, whether a fork or a replay reads it" {
  cd "${BATS_TEST_TMPDIR:?set by bats}"
  mkdir thread none
  cp "${BATS_TEST_DIRNAME}/rules/test_alone.py" thread/
  grep -v 'threading.Thread(' "${BATS_TEST_DIRNAME}/rules/test_alone.py" >none/test_alone.py

  local refused='project error: [assert-in-reading]'
  local alone='reaches an assert when read alone, and only an expectation asserts'
  local worker
  for worker in thread none; do
    cd "${BATS_TEST_TMPDIR}/${worker}"

    run -4 --separate-stderr purba test_alone.py

    diff -u - <(printf '%s\n' "${output}") <<LINES
test_alone.py::test_raised ${refused} assert check() is True ${alone}
test_alone.py::test_unreached ${refused} assert check() is True ${alone}
test_alone.py::test_raises_in_order ${refused} assert check_raises() is True ${alone}
test_alone.py::test_swallowed ${refused} assert check() is True ${alone}
test_alone.py::test_differs ${refused} assert check() is True ${alone}
test_alone.py::test_suppressed ${refused} assert check() is True ${alone}
6 cases: 6 blocked; 6 project errors
LINES
    [[ -z "${stderr}" ]]
  done
}

@test "a try of one statement and one except takes one path, and another try names that shape" {
  cd "${BATS_TEST_DIRNAME}/rules"

  run -1 --separate-stderr purba test_try.py

  local path='the lines before the first expectation take one path'
  local shape='a try of one path holds one statement and one except SomeError:'
  shape+=', with no else and no finally'
  diff -u - <(printf '%s\n' "${output}") <<LINES
test_try.py::test_raises passed
test_try.py::test_finally passed
test_try.py::test_else project error: [second-path] line 23 did not run, and ${path}; \
[second-path] line 24 did not run, and ${path}; ${shape}
test_try.py::test_named failed
    the try at line 32 expects ValueError, and the statement did not raise it
test_try.py::test_other_raise raised: KeyError: 'k'
5 cases: 2 passed, 1 failed, 1 raised, 1 blocked; 1 project error
LINES
  [[ -z "${stderr}" ]]
}

@test "a suppressed fault in another try gives no shape to the line of a with" {
  cd "${BATS_TEST_DIRNAME}/rules"

  run -4 --separate-stderr purba test_hint_suppressed.py

  local path='the lines before the first expectation take one path'
  local shape='a try of one path holds one statement and one except SomeError:'
  shape+=', with no else and no finally'
  diff -u - <(printf '%s\n' "${output}") <<LINES
test_hint_suppressed.py::test_suppressed project error: [swallowed-raise] the with at line 12 \
swallowed a raise, and line 14 did not run
    ⚠ suppressed [second-path] line 9 did not run, and ${path}
      reason: the loop once read names
test_hint_suppressed.py::test_unsuppressed project error: [second-path] line 22 did not run, \
and ${path}; [swallowed-raise] the with at line 25 swallowed a raise, and line 27 did not run; \
${shape}
test_hint_suppressed.py::test_try_ran project error: [swallowed-raise] the with at line 37 \
swallowed a raise, and line 39 did not run
3 cases: 3 blocked; 3 project errors; 1 suppressed
LINES
  [[ -z "${stderr}" ]]
}

@test "a with that skips the lines of another try names the shape of a try of one path" {
  cd "${BATS_TEST_DIRNAME}/rules"

  run -4 --separate-stderr purba test_hint_skipped.py

  local shape='a try of one path holds one statement and one except SomeError:'
  shape+=', with no else and no finally'
  diff -u - <(printf '%s\n' "${output}") <<LINES
test_hint_skipped.py::test_try_in_with project error: [swallowed-raise] the with at line 6 \
swallowed a raise, and line 8 did not run; ${shape}
test_hint_skipped.py::test_with_in_try project error: [swallowed-raise] the with at line 18 \
swallowed a raise, and line 20 did not run; ${shape}
test_hint_skipped.py::test_no_try project error: [swallowed-raise] the with at line 28 \
swallowed a raise, and line 30 did not run
3 cases: 3 blocked; 3 project errors
LINES
  [[ -z "${stderr}" ]]
}

@test "a function that C code calls back is free, and one that the test calls by name is not" {
  cd "${BATS_TEST_TMPDIR:?set by bats}"
  printf '%s\n' 'def key(x):' '    if x < 0:' '        return -x' '    return x' '' '' \
    'def check(x):' '    assert x == 1' '    return x' '' '' \
    'class Box:' '    def __init__(self, x):' '        if x:' '            self.x = x' '' '' \
    'def test_by_name():' '    total = key(1)' '    assert total == 1' '' '' \
    'def test_sorted():' '    total = sorted([3, 1, 2], key=key)' '    assert total == [1, 2, 3]' \
    '' '' 'def test_map():' '    total = list(map(key, [1]))' '    assert total == [1]' '' '' \
    'def test_class():' '    box = Box(1)' '    assert box.x == 1' '' '' \
    'def test_assert_in_callback():' '    total = sorted([1], key=check)' \
    '    assert total == [1]' >test_call.py

  run -4 --separate-stderr purba test_call.py

  local path='the lines before the first expectation take one path'
  local act='ran an assert in the act; one test checks one state; check this state in a test'
  act+=' of its own'
  diff -u - <(printf '%s\n' "${output}") <<LINES
test_call.py::test_by_name project error: [second-path] line 19 reaches line 2 of key, \
which went one way only, and ${path}; [second-path] line 19 reaches line 3 of key, \
which did not run, and ${path}
test_call.py::test_sorted passed
test_call.py::test_map passed
test_call.py::test_class project error: [second-path] line 34 reaches line 14 of __init__, \
which went one way only, and ${path}
test_call.py::test_assert_in_callback project error: [assert-before-expectation] line 39 \
reaches line 8 of check, which ${act}
5 cases: 2 passed, 3 blocked; 3 project errors
LINES
  [[ -z "${stderr}" ]]
}

@test "a test that changes the warning filters is refused, and so is a module that does" {
  cd "${BATS_TEST_TMPDIR:?set by bats}"
  printf '%s\n' 'import warnings' '' '' 'def test_filter():' \
    '    warnings.filterwarnings("ignore", category=DeprecationWarning)' '    assert 1 == 1' \
    '' '' 'def test_record():' '    with warnings.catch_warnings(record=True) as seen:' \
    '        warnings.warn("old", UserWarning)' '    assert len(seen) == 1' >test_filters.py
  printf '%s\n' 'import warnings' '' 'warnings.simplefilter("error")' '' '' \
    'def test_one():' '    assert 1 == 1' >test_load_filters.py

  run -4 --separate-stderr purba test_filters.py

  local kept='a test keeps the filters that the run began with'
  diff -u - <(printf '%s\n' "${output}") <<LINES
test_filters.py::test_filter project error: [hook-replaced] line 5 replaced the warning \
filters, and ${kept}
test_filters.py::test_record passed
2 cases: 1 passed, 1 blocked; 1 project error
LINES
  [[ -z "${stderr}" ]] || false

  run -4 --separate-stderr purba test_load_filters.py

  diff -u - <(printf '%s\n' "${output}") <<LINES
test_load_filters.py project error: [hook-replaced] the module replaced the warning filters \
as it loaded, and ${kept}
test_load_filters.py::test_one blocked: test_load_filters.py did not load
1 case: 1 blocked; 1 project error
LINES
  [[ -z "${stderr}" ]]
}

# The module of each test of the tool ids of `sys.monitoring`: `pick` goes one
# way when the test calls it, so the watch refuses the case.
pick=('' '' 'def pick(x):' '    if x > 0:' '        return "pos"' '    return "neg"'
  '' '' 'def test_one_way():' '    got = pick(1)' '    assert got == "pos"')

@test "a module that takes the tool ids 3 and 4 as it loads is refused" {
  cd "${BATS_TEST_TMPDIR:?set by bats}"
  printf '%s\n' 'import sys' '' 'sys.monitoring.use_tool_id(3, "other")' \
    'sys.monitoring.use_tool_id(4, "other")' "${pick[@]}" >test_ids.py

  run -4 --separate-stderr purba test_ids.py

  local took='the module took the tool ids 3 and 4 of sys.monitoring as it loaded'
  diff -u - <(printf '%s\n' "${output}") <<LINES
test_ids.py project error: [hook-replaced] ${took}, and a test keeps purba's hooks
test_ids.py::test_one_way blocked: test_ids.py did not load
1 case: 1 blocked; 1 project error
LINES
  [[ -z "${stderr}" ]]
}

@test "a module that takes the tool id 4 where a tool holds 3 is refused for the id 4" {
  cd "${BATS_TEST_TMPDIR:?set by bats}"
  mkdir site
  printf '%s\n' 'import sys' '' 'sys.monitoring.use_tool_id(3, "coverage")' \
    >site/sitecustomize.py
  printf '%s\n' 'import sys' '' 'sys.monitoring.use_tool_id(4, "other")' "${pick[@]}" \
    >test_four.py

  run -4 --separate-stderr env PYTHONPATH="${PWD}/site" purba test_four.py

  local took='the module took the tool id 4 of sys.monitoring as it loaded'
  diff -u - <(printf '%s\n' "${output}") <<LINES
test_four.py project error: [hook-replaced] ${took}, and a test keeps purba's hooks
test_four.py::test_one_way blocked: test_four.py did not load
1 case: 1 blocked; 1 project error
LINES
  [[ -z "${stderr}" ]]
}

@test "a module under a tool that holds the tool ids 3 and 4 is refused, and does not run" {
  cd "${BATS_TEST_TMPDIR:?set by bats}"
  mkdir site
  printf '%s\n' 'import sys' '' 'sys.monitoring.use_tool_id(3, "coverage")' \
    'sys.monitoring.use_tool_id(4, "profiler")' >site/sitecustomize.py
  printf '%s\n' 'open("loaded.txt", "w").close()' "${pick[@]}" >test_held.py

  run -4 --separate-stderr env PYTHONPATH="${PWD}/site" purba test_held.py

  local held='sys.monitoring holds the tool ids 3 and 4 for coverage and profiler'
  diff -u - <(printf '%s\n' "${output}") <<LINES
test_held.py project error: ${held}, and execution needs one of them to watch
test_held.py::test_one_way blocked: test_held.py did not load
1 case: 1 blocked; 1 project error
LINES
  [[ -z "${stderr}" ]] || false
  [[ ! -e loaded.txt ]]
}

@test "a module or a tool that takes the tool id 3 alone leaves execution the id 4" {
  cd "${BATS_TEST_TMPDIR:?set by bats}"
  printf '%s\n' 'import sys' '' 'sys.monitoring.use_tool_id(3, "other")' "${pick[@]}" \
    >test_three.py
  mkdir site
  printf '%s\n' 'import sys' '' 'sys.monitoring.use_tool_id(3, "coverage")' \
    >site/sitecustomize.py
  printf '%s\n' "${pick[@]}" >test_watched.py

  run -4 --separate-stderr purba test_three.py

  local path='the lines before the first expectation take one path'
  diff -u - <(printf '%s\n' "${output}") <<LINES
test_three.py::test_one_way project error: [second-path] line 13 reaches line 7 of pick, \
which went one way only, and ${path}; [second-path] line 13 reaches line 9 of pick, which \
did not run, and ${path}
1 case: 1 blocked; 1 project error
LINES
  [[ -z "${stderr}" ]] || false

  run -4 --separate-stderr env PYTHONPATH="${PWD}/site" purba test_watched.py

  diff -u - <(printf '%s\n' "${output}") <<LINES
test_watched.py::test_one_way project error: [second-path] line 10 reaches line 4 of pick, \
which went one way only, and ${path}; [second-path] line 10 reaches line 6 of pick, which \
did not run, and ${path}
1 case: 1 blocked; 1 project error
LINES
  [[ -z "${stderr}" ]]
}

@test "a test that changes the events that execution watches with is refused" {
  cd "${BATS_TEST_TMPDIR:?set by bats}"
  printf '%s\n' 'import sys' '' '' 'def pick(x):' '    if x > 0:' '        return "pos"' \
    '    return "neg"' '' '' 'def test_one_way():' \
    '    sys.monitoring.set_local_events(3, pick.__code__, 0)' '    got = pick(1)' \
    '    assert got == "pos"' >test_events.py

  run -4 --separate-stderr purba test_events.py

  local replaced='line 11 replaced the events of sys.monitoring that purba watches with'
  diff -u - <(printf '%s\n' "${output}") <<LINES
test_events.py::test_one_way project error: [hook-replaced] ${replaced}, and a test keeps \
purba's hooks
1 case: 1 blocked; 1 project error
LINES
  [[ -z "${stderr}" ]]
}

@test "a test that changes the events and puts them back inside one statement is refused" {
  cd "${BATS_TEST_TMPDIR:?set by bats}"
  printf '%s\n' 'import sys' '' '' 'def pick(x):' '    if x > 0:' '        return "pos"' \
    '    return "neg"' '' '' 'def quiet(x):' \
    '    saved = sys.monitoring.get_local_events(3, pick.__code__)' \
    '    sys.monitoring.set_local_events(3, pick.__code__, 0)' '    got = pick(x)' \
    '    sys.monitoring.set_local_events(3, pick.__code__, saved)' '    return got' '' '' \
    'def test_one_way():' '    got = quiet(1)' '    assert got == "pos"' >test_restores.py

  run -4 --separate-stderr purba test_restores.py

  local replaced='line 19 replaced the events of sys.monitoring that purba watches with'
  diff -u - <(printf '%s\n' "${output}") <<LINES
test_restores.py::test_one_way project error: [hook-replaced] ${replaced}, and a test keeps \
purba's hooks
1 case: 1 blocked; 1 project error
LINES
  [[ -z "${stderr}" ]]
}

@test "a test that sets the events to their own value is free" {
  cd "${BATS_TEST_TMPDIR:?set by bats}"
  printf '%s\n' 'import sys' '' '' 'def pick(x):' '    if x > 0:' '        return "pos"' \
    '    return "neg"' '' '' 'def test_one_way():' \
    '    events = sys.monitoring.get_local_events(3, pick.__code__)' \
    '    sys.monitoring.set_local_events(3, pick.__code__, events)' '    got = pick(1)' \
    '    assert got == "pos"' >test_same.py

  run -4 --separate-stderr purba test_same.py

  local path='the lines before the first expectation take one path'
  diff -u - <(printf '%s\n' "${output}") <<LINES
test_same.py::test_one_way project error: [second-path] line 13 reaches line 5 of pick, \
which went one way only, and ${path}; [second-path] line 13 reaches line 7 of pick, which \
did not run, and ${path}
1 case: 1 blocked; 1 project error
LINES
  [[ -z "${stderr}" ]]
}

@test "a test that changes the events through a name bound as its module loaded is refused" {
  cd "${BATS_TEST_TMPDIR:?set by bats}"
  printf '%s\n' 'import sys' '' 'bound = sys.monitoring.set_local_events' '' '' 'def pick(x):' \
    '    if x > 0:' '        return "pos"' '    return "neg"' '' '' 'def test_one_way():' \
    '    bound(3, pick.__code__, 0)' '    got = pick(1)' '    assert got == "pos"' >test_bound.py

  run -4 --separate-stderr purba test_bound.py

  local replaced='line 13 replaced the events of sys.monitoring that purba watches with'
  diff -u - <(printf '%s\n' "${output}") <<LINES
test_bound.py::test_one_way project error: [hook-replaced] ${replaced}, and a test keeps \
purba's hooks
1 case: 1 blocked; 1 project error
LINES
  [[ -z "${stderr}" ]]
}

@test "a module that changes the events of a tool id that no tool holds does not load" {
  cd "${BATS_TEST_TMPDIR:?set by bats}"
  printf '%s\n' 'import sys' '' 'sys.monitoring.set_local_events(3, (lambda: 0).__code__, 1)' \
    "${pick[@]}" >test_free_id.py

  run -4 --separate-stderr purba test_free_id.py

  diff -u - <(printf '%s\n' "${output}") <<LINES
test_free_id.py project error: ValueError: tool 3 is not in use
test_free_id.py::test_one_way blocked: test_free_id.py did not load
1 case: 1 blocked; 1 project error
LINES
  [[ -z "${stderr}" ]]
}
