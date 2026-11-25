#   the decision:
# docs/decisions/purba-picks-a-test-by-its-name-and-the-tests-of-a-run-by-one-query.md
#
# A function whose name is near `test` is a project error. Each test builds its
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
  git -C "${root}" init --quiet
}

# `tests/test_a.py` holds `test_add`, which passes, then the text of `$1`.
module() {
  printf 'def test_add():\n    assert 1 + 1 == 2\n\n%s' "$1" >"${root}/tests/test_a.py"
  git -C "${root}" add --all
  cd "${root}" || return
}

# The line of the error of the function `$1`, in `line`.
near() {
  line="tests/test_a.py::$1 project error: [name-near] $1 is not a test,"
  line+=" and its name is near the prefix test"
}

@test "a function whose name is near test is a project error, and the test beside it runs" {
  module $'def tset_sub():\n    assert 2 - 1 == 1\n'

  near tset_sub
  run -4 --separate-stderr purba

  diff -u - <(printf '%s\n' "${output}") <<LINES
${line}
tests/test_a.py::test_add passed
1 case: 1 passed; 1 project error
LINES
  [[ -z "${stderr}" ]]
}

@test "a helper whose first word is far from test gives no error" {
  module $'def format_text():\n    return ""\n'

  run -0 --separate-stderr purba

  diff -u - <(printf '%s\n' "${output}") <<'LINES'
tests/test_a.py::test_add passed
1 case: 1 passed
LINES
  [[ -z "${stderr}" ]]
}

@test "a helper whose first word is one edit from test gives the error" {
  module $'def text_format():\n    return ""\n'

  near text_format
  run -4 --separate-stderr purba

  diff -u - <(printf '%s\n' "${output}") <<LINES
${line}
tests/test_a.py::test_add passed
1 case: 1 passed; 1 project error
LINES
  [[ -z "${stderr}" ]]
}

@test "the line of a name near test sorts in source order among the refused tests" {
  printf '%s\n' \
    'async def test_wait():' '    ...' '' \
    'def tset_sub():' '    pass' '' \
    'def test_mul(x):' '    ...' '' \
    'def test_add():' '    assert 1 + 1 == 2' >"${root}/tests/test_a.py"
  git -C "${root}" add --all
  cd "${root}" || return

  near tset_sub
  local mul="tests/test_a.py::test_mul project error: the parameter x has no default,"
  mul+=" and purba cannot see which fixture builds it"
  run -4 --separate-stderr purba

  diff -u - <(printf '%s\n' "${output}") <<LINES
tests/test_a.py::test_wait project error: purba cannot run an async test yet
${line}
${mul}
tests/test_a.py::test_add passed
3 cases: 1 passed, 2 blocked; 3 project errors
LINES
  [[ -z "${stderr}" ]]
}

@test "a test file that holds only a name near test prints its line, and exits 4" {
  printf 'def tset_sub():\n    assert 2 - 1 == 1\n' >"${root}/tests/test_a.py"
  git -C "${root}" add --all
  cd "${root}" || return

  near tset_sub
  run -4 --separate-stderr purba

  diff -u - <(printf '%s\n' "${output}") <<LINES
${line}
0 cases; 1 project error
LINES
  [[ -z "${stderr}" ]]
}
