#   the decision:
# docs/decisions/purba-picks-a-test-by-its-name-and-the-tests-of-a-run-by-one-query.md
#
# The project's file saves a query under a name in `[tool.purba.queries]`, as
# an alias. Each test builds its project in its own folder, and git tracks the
# project.
#
# bash 3.2 on macOS fails a test at a `[[ ]]` only on its last line, so `run -N`
# checks each exit code, and `|| false` ends each other check.
# https://bats-core.readthedocs.io/en/stable/gotchas.html#or-did-not-fail-my-test

: "${BATS_TEST_DIRNAME:?set by bats}"
bats_require_minimum_version 1.5.0

# git hands a hook and `git rebase --exec` variables such as GIT_DIR, and a
# test that runs git under them writes into the repository around it. bash 3.2
# has no mapfile, so a loop unsets each name.
#
# `tests/test_a.py` holds `test_add` and `test_sub`, and
# `tests/slow/test_s.py` holds `test_slow`. Each passes.
setup() {
  local names name
  names=$(git rev-parse --local-env-vars)
  while IFS= read -r name; do
    unset "${name}"
  done <<<"${names}"
  export HOME="${BATS_TEST_TMPDIR:?set by bats}/home"
  export XDG_CONFIG_HOME="${HOME}/.config"
  export GIT_CONFIG_GLOBAL=/dev/null GIT_CONFIG_SYSTEM=/dev/null
  export GIT_CEILING_DIRECTORIES="${BATS_TEST_TMPDIR}"
  mkdir -p "${HOME}"
  root="${BATS_TEST_TMPDIR}/project"
  mkdir -p "${root}/tests/slow"
  printf '%s\n' \
    'def test_add():' '    assert 1 + 1 == 2' '' \
    'def test_sub():' '    assert 2 - 1 == 1' >"${root}/tests/test_a.py"
  printf '%s\n' 'def test_slow():' '    assert 3 - 3 == 0' >"${root}/tests/slow/test_s.py"
  git -C "${root}" init --quiet
}

# Writes the project's file with `[tool.purba.queries]` and each line of
# `$@` under it, tracks each file of the project, and enters it.
queries() {
  printf '%s\n' '[tool.purba]' '' '[tool.purba.queries]' "$@" >"${root}/pyproject.toml"
  git -C "${root}" add --all
  cd "${root}" || return
}

@test "an alias runs the tests that its query picks" {
  queries 'fast = "!path(tests/slow/**)"'

  run -0 --separate-stderr purba fast

  diff -u - <(printf '%s\n' "${output}") <<'LINES'
tests/test_a.py::test_add passed
tests/test_a.py::test_sub passed
2 cases: 2 passed
LINES
  [[ -z "${stderr}" ]]
}

@test "an alias that names itself is a project error, even when the query does not call it" {
  queries 'fast = "!path(tests/slow/**)"' 'loop = "!loop"'

  run -4 --separate-stderr purba tests/test_a.py

  diff -u - <(printf '%s\n' "${output}") <<'LINES'
pyproject.toml project error: loop under [tool.purba.queries] names itself
0 cases; 1 project error
LINES
  [[ -z "${stderr}" ]]
}

@test "an alias that names itself through another alias is a project error" {
  queries 'a = "b"' 'b = "a"'

  run -4 --separate-stderr purba

  diff -u - <(printf '%s\n' "${output}") <<'LINES'
pyproject.toml project error: a under [tool.purba.queries] names itself through b
pyproject.toml project error: b under [tool.purba.queries] names itself through a
0 cases; 2 project errors
LINES
  [[ -z "${stderr}" ]]
}

@test "purba names the errors of test-files and of the aliases in one run" {
  printf '%s\n' '[tool.purba]' 'test-files = []' '' '[tool.purba.queries]' 'loop = "loop"' \
    >"${root}/pyproject.toml"
  git -C "${root}" add --all
  cd "${root}" || return

  run -4 --separate-stderr purba

  local empty='pyproject.toml project error: test-files under [tool.purba] is an empty list, '
  empty+='so no file is a test file; list at least one of '
  empty+='test_*.py, *_test.py, tests.py and test.py'
  diff -u <(printf '%s\n' "${empty}" \
    'pyproject.toml project error: loop under [tool.purba.queries] names itself' \
    '0 cases; 2 project errors') <(printf '%s\n' "${output}") || false
  [[ -z "${stderr}" ]]
}

@test "a query that does not parse fails before purba reads the project's file" {
  queries 'loop = "loop"'

  run -3 --separate-stderr purba 'name('

  [[ "${stderr}" == 'purba: invocation error: column 6: expected a name' ]] || false
  [[ -z "${output}" ]]
}

@test "an alias in the user's file changes nothing" {
  queries
  mkdir -p "${XDG_CONFIG_HOME}/purba"
  printf '%s\n' '[queries]' 'fast = "!path(tests/slow/**)"' >"${XDG_CONFIG_HOME}/purba/purba.toml"

  run -3 --separate-stderr purba fast

  [[ "${stderr}" == 'purba: invocation error: fast names no test; the nearest is tests' ]] || false
  [[ -z "${output}" ]]
}

@test "a bare word that names an alias and a folder runs the alias" {
  queries 'tests = "tests/slow"'

  run -0 --separate-stderr purba tests

  diff -u - <(printf '%s\n' "${output}") <<'LINES'
tests/slow/test_s.py::test_slow passed
1 case: 1 passed
LINES
  [[ -z "${stderr}" ]]
}

@test "a call hands its argument to the alias" {
  queries '"stable(x)" = "x & !path(tests/slow/**)"'

  run -0 --separate-stderr purba 'stable(name(test_))'

  diff -u - <(printf '%s\n' "${output}") <<'LINES'
tests/test_a.py::test_add passed
tests/test_a.py::test_sub passed
2 cases: 2 passed
LINES
  [[ -z "${stderr}" ]]
}

@test "a call that names no alias is an invocation error that names the nearest alias" {
  queries '"stable(x)" = "x & !path(tests/slow/**)"'

  run -3 --separate-stderr purba 'stale(tests)'

  local refused='purba: invocation error: column 1: stale( is no term; '
  refused+='the nearest alias is stable(x)'
  [[ "${stderr}" == "${refused}" ]] || false
  [[ -z "${output}" ]]
}

@test "a bare word that names nothing names the nearest alias" {
  queries 'fast = "!path(tests/slow/**)"'

  run -3 --separate-stderr purba fsat

  local refused='purba: invocation error: fsat names no test or alias; '
  refused+='the nearest alias is fast'
  [[ "${stderr}" == "${refused}" ]] || false
  [[ -z "${output}" ]]
}
