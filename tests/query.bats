#   the decision:
# docs/decisions/purba-picks-a-test-by-its-name-and-the-tests-of-a-run-by-one-query.md
#
# A tester names the tests of a run with one query. Each test builds its
# project in its own folder, and git tracks the project.
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
# `tests/test_a.py` holds `test_add` and `test_sub`, which pass.
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
  mkdir -p "${root}/tests"
  printf '[tool.purba]\n' >"${root}/pyproject.toml"
  printf '%s\n' \
    'def test_add():' '    assert 1 + 1 == 2' '' \
    'def test_sub():' '    assert 2 - 1 == 1' >"${root}/tests/test_a.py"
  git -C "${root}" init --quiet
}

# Tracks each file of the project, and enters it.
track() {
  git -C "${root}" add --all
  cd "${root}" || return
}

@test "the id of a test runs that test alone" {
  track

  run -0 --separate-stderr purba tests/test_a.py::test_add

  diff -u - <(printf '%s\n' "${output}") <<'LINES'
tests/test_a.py::test_add passed
1 case: 1 passed
LINES
  [[ -z "${stderr}" ]]
}

@test "an id that names no test is an invocation error that names the nearest id" {
  track

  run -3 --separate-stderr purba tests/test_a.py::test_ad

  local refused='purba: invocation error: tests/test_a.py::test_ad names no test; '
  refused+='the nearest is tests/test_a.py::test_add'
  [[ "${stderr}" == "${refused}" ]] || false
  [[ -z "${output}" ]]
}

@test "a query nests at most 255 levels, and a deeper one is refused before the parser reads it" {
  track
  local open close
  open=$(printf '(%.0s' {1..255})
  close=$(printf ')%.0s' {1..255})

  run -0 --separate-stderr purba "${open}tests/test_a.py::test_add${close}"

  diff -u - <(printf '%s\n' "${output}") <<'LINES'
tests/test_a.py::test_add passed
1 case: 1 passed
LINES
  [[ -z "${stderr}" ]] || false

  run -3 --separate-stderr purba "(${open}tests/test_a.py::test_add${close})"

  local refused='purba: invocation error: the query nests deeper than 255 levels at column 256'
  [[ "${stderr}" == "${refused}" ]] || false
  [[ -z "${output}" ]]
}

@test "a query that selects no test, while each term names something, prints only the count" {
  track

  run -2 --separate-stderr purba 'tests/test_a.py::test_add & name(sub)'

  diff -u - <(printf '%s\n' "${output}") <<'LINES'
0 cases
LINES
  [[ -z "${stderr}" ]]
}

@test "tag() is no term yet, so a query with it does not parse" {
  track

  run -3 --separate-stderr purba 'tag(slow)'

  local refused='purba: invocation error: column 1: tag( is no term; '
  refused+='a query takes name( and path('
  [[ "${stderr}" == "${refused}" ]] || false
  [[ -z "${output}" ]]
}

@test "stage two parses only the files that the query reaches" {
  printf 'def (:\n' >"${root}/tests/test_b.py"
  track

  run -0 --separate-stderr purba 'path(tests/test_a*.py)'

  diff -u - <(printf '%s\n' "${output}") <<'LINES'
tests/test_a.py::test_add passed
tests/test_a.py::test_sub passed
2 cases: 2 passed
LINES
  [[ -z "${stderr}" ]]
}

@test "a file runs beside a file that the query does not reach and that does not parse" {
  printf 'def (:\n' >"${root}/tests/test_b.py"
  track

  run -0 --separate-stderr purba tests/test_a.py

  diff -u - <(printf '%s\n' "${output}") <<'LINES'
tests/test_a.py::test_add passed
tests/test_a.py::test_sub passed
2 cases: 2 passed
LINES
  [[ -z "${stderr}" ]]
}

@test "& binds tighter than |" {
  printf '%s\n' 'def test_mul():' '    assert 2 * 1 == 2' >"${root}/tests/test_b.py"
  track

  run -0 --separate-stderr purba 'name(mul) | name(add) & name(sub)'

  diff -u - <(printf '%s\n' "${output}") <<'LINES'
tests/test_b.py::test_mul passed
1 case: 1 passed
LINES
  [[ -z "${stderr}" ]]
}

@test "a name that no test of the files the query reaches holds counts the files left out" {
  printf '%s\n' 'def test_mul():' '    assert 2 * 1 == 2' >"${root}/tests/test_b.py"
  track

  run -3 --separate-stderr purba 'tests/test_a.py & name(test_mul)'

  local refused='purba: invocation error: name(test_mul) names no test; '
  refused+='the nearest name is test_sub; the query reaches 1 of 2 test files'
  [[ "${stderr}" == "${refused}" ]] || false
  [[ -z "${output}" ]]
}
