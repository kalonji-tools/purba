#   the decision:
# docs/decisions/a-test-returns-nothing-and-the-first-part-that-sees-a-breach-refuses-it.md
#
# A check of each item with all() and any(), a check that reads an iterator, and the frame
# of a raise, in the modules of tests/iterable. Each case matches the exit code and the whole
# stdout and stderr.
#
# bash 3.2 on macOS fails a test at a `[[ ]]` only on its last line, so `run -N`
# checks each exit code.
# https://bats-core.readthedocs.io/en/stable/gotchas.html#or-did-not-fail-my-test

: "${BATS_TEST_DIRNAME:?set by bats}"
bats_require_minimum_version 1.5.0

@test "all() names its first broken item, any() each item, and a raise names its frame" {
  cd "${BATS_TEST_DIRNAME}/iterable"

  run -1 --separate-stderr purba iterable.py

  local filtered='[not-one-comparison] line 22 is an all() with an if,'
  filtered+=' and an all() takes one for, no if'
  local empty='[no-item] line 27 checks each item of xs, and xs holds no item'
  local iterator='[iterator-read] line 32 reads it, an iterator that one reading empties;'
  iterator+=' make it a list in the act'
  diff -u - <(printf '%s\n' "${output}") <<LINES
iterable.py::test_filtered project error: ${filtered}
iterable.py::test_each failed
    assert all(len(c) == 2 for c in parts)
      the first item that breaks  len(c) == 2
      [2]  len(c)  1     c  'e'
iterable.py::test_some failed
    assert any(p == "/opt" for p in paths)
      none of 2 items holds  p == "/opt"
      [0]  '/usr/share'
      [1]  '/etc'
iterable.py::test_empty project error: ${empty}
iterable.py::test_iterator project error: ${iterator}
iterable.py::test_made_by_the_check passed
iterable.py::test_generator_raises raised: ValueError: boom
    raised inside the generator gen() at iterable.py:3
iterable.py::test_parse_raises raised: ValueError: invalid literal for int() with base 10: 'x'
    raised in parse() at iterable.py:7
8 cases: 1 passed, 2 failed, 2 raised, 3 blocked; 3 project errors
LINES
  [[ -z "${stderr}" ]]
}

@test "a case that changes its folder names the frame of its raise from the folder of the run" {
  cd "${BATS_TEST_DIRNAME}/iterable"

  run -4 --separate-stderr purba chdir.py

  local raised="chdir.py::test_chdir raised: ValueError:"
  raised+=" invalid literal for int() with base 10: 'x'"
  diff -u - <(printf '%s\n' "${output}") <<LINES
${raised}
    raised in parse() at chdir.py:5
1 case: 1 raised
LINES
  [[ -z "${stderr}" ]]
}
