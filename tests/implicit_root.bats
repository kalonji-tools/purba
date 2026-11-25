#   the decision:  docs/decisions/purba-reads-configuration-from-six-levels.md
#
# `PURBA_IMPLICIT_ROOT`: the steps to a root when no pyproject.toml holds
# `[tool.purba]`, the line that names that root, and the errors of the
# environment. Each test builds its project in its own folder.
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
  root="${HOME}/p"
  because='because PURBA_IMPLICIT_ROOT is set and no pyproject.toml holds [tool.purba]'
  refused='purba: invocation error: no pyproject.toml holds [tool.purba]; '
  near='purba: invocation error: PURBA_IMLPICIT_ROOT is no variable of purba; '
  near+='the nearest is PURBA_IMPLICIT_ROOT'
}

# A module at `$1` with one test, `$2`, that passes.
passing() {
  mkdir -p "$(dirname "$1")"
  printf 'def %s():\n    assert 1 + 1 == 2\n' "$2" >"$1"
}

# `tests/test_a.py` under the folder `$1`, by default `${root}`.
one_test() {
  passing "${1:-${root}}/tests/test_a.py" test_add
}

# Whether the run printed `$1` as its first line, then the line of
# `tests/test_a.py::test_add` from the root, then the count, and nothing on
# stderr.
printed_root() {
  diff -u - <(printf '%s\n' "${output}") <<LINES || return 1
$1
tests/test_a.py::test_add passed
1 case: 1 passed
LINES
  [[ -z "${stderr}" ]]
}

@test "with PURBA_IMPLICIT_ROOT=1, the root is the folder of the pyproject.toml with [project]" {
  mkdir -p "${root}"
  printf '[project]\nname = "p"\n' >"${root}/pyproject.toml"
  git -C "${root}" init --quiet
  one_test
  local line="⚠ root ~/p is the folder of the pyproject.toml that holds [project], ${because}"

  cd "${root}"
  run -0 --separate-stderr env PURBA_IMPLICIT_ROOT=1 purba
  printed_root "${line}"

  cd "${root}/tests"
  run -0 --separate-stderr env PURBA_IMPLICIT_ROOT=1 purba
  printed_root "${line}"
}

@test "with PURBA_IMPLICIT_ROOT=1 and no [project], the root is the folder that holds .git" {
  mkdir -p "${root}"
  printf '[tool.black]\n' >"${root}/pyproject.toml"
  git -C "${root}" init --quiet
  one_test
  local line="⚠ root ~/p is the folder that holds .git, ${because}"

  cd "${root}"
  run -0 --separate-stderr env PURBA_IMPLICIT_ROOT=1 purba
  printed_root "${line}"

  cd "${root}/tests"
  run -0 --separate-stderr env PURBA_IMPLICIT_ROOT=1 purba
  printed_root "${line}"
}

@test "with PURBA_IMPLICIT_ROOT=1 and neither, the root and each id move with the working folder" {
  one_test
  local moves='the id of each test starts here, and moves with the working folder'

  cd "${root}"
  run -0 --separate-stderr env PURBA_IMPLICIT_ROOT=1 purba
  printed_root "⚠ root ~/p is the working folder, ${because}; ${moves}"

  cd "${root}/tests"
  run -0 --separate-stderr env PURBA_IMPLICIT_ROOT=1 purba
  diff -u - <(printf '%s\n' "${output}") <<LINES
⚠ root ~/p/tests is the working folder, ${because}; ${moves}
test_a.py::test_add passed
1 case: 1 passed
LINES
  [[ -z "${stderr}" ]]
}

@test "a root outside the home folder prints in full" {
  local outside="${BATS_TEST_TMPDIR}/project" full
  mkdir -p "${outside}"
  git -C "${outside}" init --quiet
  one_test "${outside}"
  full=$(cd "${outside}" && pwd -P)
  cd "${outside}"

  run -0 --separate-stderr env PURBA_IMPLICIT_ROOT=1 purba

  printed_root "⚠ root ${full} is the folder that holds .git, ${because}"
}

@test "a root that [tool.purba] gives prints no line with PURBA_IMPLICIT_ROOT=1" {
  mkdir -p "${root}"
  printf '[tool.purba]\n' >"${root}/pyproject.toml"
  git -C "${root}" init --quiet
  one_test
  cd "${root}"

  run -0 --separate-stderr env PURBA_IMPLICIT_ROOT=1 purba

  diff -u - <(printf '%s\n' "${output}") <<'LINES'
tests/test_a.py::test_add passed
1 case: 1 passed
LINES
  [[ -z "${stderr}" ]]
}

@test "PURBA_IMPLICIT_ROOT=0 and an empty value refuse the run as an unset variable does" {
  mkdir -p "${root}"
  printf '[project]\nname = "p"\n' >"${root}/pyproject.toml"
  git -C "${root}" init --quiet
  one_test
  cd "${root}"

  run -3 --separate-stderr env PURBA_IMPLICIT_ROOT=0 purba
  [[ "${stderr}" == "${refused}add it to pyproject.toml" ]] || false
  [[ -z "${output}" ]] || false

  run -3 --separate-stderr env PURBA_IMPLICIT_ROOT= purba
  [[ "${stderr}" == "${refused}add it to pyproject.toml" ]] || false
  [[ -z "${output}" ]]
}

@test "any other value of PURBA_IMPLICIT_ROOT is an invocation error, with [tool.purba] too" {
  mkdir -p "${root}"
  git -C "${root}" init --quiet
  one_test
  cd "${root}"
  local error='purba: invocation error: PURBA_IMPLICIT_ROOT is yes; set 1 or 0'

  run -3 --separate-stderr env PURBA_IMPLICIT_ROOT=yes purba
  [[ "${stderr}" == "${error}" ]] || false
  [[ -z "${output}" ]] || false

  printf '[tool.purba]\n' >"${root}/pyproject.toml"
  run -3 --separate-stderr env PURBA_IMPLICIT_ROOT=yes purba
  [[ "${stderr}" == "${error}" ]] || false
  [[ -z "${output}" ]]
}

@test "a PURBA_ variable near PURBA_IMPLICIT_ROOT is an invocation error, and a far one is not" {
  mkdir -p "${root}"
  printf '[tool.purba]\n' >"${root}/pyproject.toml"
  git -C "${root}" init --quiet
  one_test
  cd "${root}"

  run -3 --separate-stderr env PURBA_IMLPICIT_ROOT=1 purba
  [[ "${stderr}" == "${near}" ]] || false
  [[ -z "${output}" ]] || false

  run -0 --separate-stderr env PURBA_SENTINEL=1 purba
  diff -u - <(printf '%s\n' "${output}") <<'LINES'
tests/test_a.py::test_add passed
1 case: 1 passed
LINES
  [[ -z "${stderr}" ]]
}

@test "each error of the environment prints on its own line, before the project's file is read" {
  mkdir -p "${root}"
  printf '[tool.purba]\nworkers = 4\n' >"${root}/pyproject.toml"
  git -C "${root}" init --quiet
  one_test
  cd "${root}"

  run -3 --separate-stderr env PURBA_IMPLICIT_ROOT=yes PURBA_IMLPICIT_ROOT=1 purba

  diff -u - <(printf '%s\n' "${stderr}") <<LINES
${near}
purba: invocation error: PURBA_IMPLICIT_ROOT is yes; set 1 or 0
LINES
  [[ -z "${output}" ]]
}
