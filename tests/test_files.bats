#   the decision:
# docs/decisions/purba-picks-a-test-by-its-name-and-the-tests-of-a-run-by-one-query.md
#   the decision:  docs/decisions/purba-reads-configuration-from-six-levels.md
#
# `test-files` under `[tool.purba]`: the patterns of a test file, and the
# errors of the project's file. Each test builds its project in its own
# folder, and git tracks the project.
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
  want=()
}

# A project at `${root}` under git, whose pyproject.toml holds `[tool.purba]`
# and then each line of `$@`.
project() {
  mkdir -p "${root}"
  printf '%s\n' '[tool.purba]' "$@" >"${root}/pyproject.toml"
  git -C "${root}" init --quiet
}

# A module at `$1` with one test, `$2`, that passes.
passing() {
  mkdir -p "$(dirname "$1")"
  printf 'def %s():\n    assert 1 + 1 == 2\n' "$2" >"$1"
}

# Two test files: `tests/test_a.py` and `tests/b_test.py`.
two_files() {
  passing "${root}/tests/test_a.py" test_a
  passing "${root}/tests/b_test.py" test_b
  git -C "${root}" add --all
}

# Whether the run printed each line that the test expects, and then each line
# of `$@`, on stdout, and nothing on stderr.
printed() {
  want+=("$@")
  diff -u <(printf '%s\n' "${want[@]}") <(printf '%s\n' "${output}") || return 1
  [[ -z "${stderr}" ]]
}

# Whether the run printed nothing on stdout, and the invocation error that
# the path `$1` names no test file, for the reason `$2`, on stderr.
refused() {
  [[ -z "${output}" ]] || return 1
  [[ "${stderr}" == "purba: invocation error: $1 names no test file: $2" ]]
}

closed='test_*.py, *_test.py, tests.py and test.py'
dial='test-files under [tool.purba]'

# Expects the error of the item `$1` outside the closed list, whose nearest pattern
# is `$2`.
outside() {
  local line="pyproject.toml project error: ${dial} holds $1, "
  line+="which is none of ${closed}; the nearest is $2"
  want+=("${line}")
}

# Expects the error of a value of the kind `$1` in place of a list.
not_a_list() {
  local line="pyproject.toml project error: ${dial} is $1; "
  line+='write a list, such as ["*_test.py"]'
  want+=("${line}")
}

# Expects the error of an empty list, in the project's file at `$1`.
empty() {
  local line="$1 project error: ${dial} is an empty list, so no file is a test file; "
  line+="list at least one of ${closed}"
  want+=("${line}")
}

# Expects the error of the key `$1`, which no dial owns, and whose nearest dial
# is `$2`.
unknown() {
  local line="pyproject.toml project error: $1 under [tool.purba] is no setting of purba; "
  line+="the nearest is $2"
  want+=("${line}")
}

@test "test-files = [\"*_test.py\"] runs only the files that match *_test.py" {
  project 'test-files = ["*_test.py"]'
  two_files
  cd "${root}"

  run -0 --separate-stderr purba

  printed 'tests/b_test.py::test_b passed' '1 case: 1 passed'
}

@test "a path to a hidden file that matches *_test.py says that the file is hidden" {
  project 'test-files = ["*_test.py"]'
  passing "${root}/tests/.x_test.py" test_x
  git -C "${root}" add --all
  cd "${root}"

  run -3 --separate-stderr purba tests/.x_test.py

  refused tests/.x_test.py 'stage one skips tests/.x_test.py, a hidden file or folder'
}

@test "each pattern of the closed list runs the files it matches, and no other" {
  project 'test-files = ["test_*.py", "*_test.py", "tests.py", "test.py"]'
  passing "${root}/tests/test_a.py" test_a
  passing "${root}/tests/b_test.py" test_b
  passing "${root}/tests/tests.py" test_tests
  passing "${root}/tests/test.py" test_test
  passing "${root}/tests/_test.py" test_bare
  passing "${root}/tests/testing.py" test_testing
  git -C "${root}" add --all
  cd "${root}"

  run -0 --separate-stderr purba

  printed \
    'tests/b_test.py::test_b passed' \
    'tests/test.py::test_test passed' \
    'tests/test_a.py::test_a passed' \
    'tests/tests.py::test_tests passed' \
    '4 cases: 4 passed'
}

@test "a pattern listed twice runs each file once" {
  project 'test-files = ["*_test.py", "*_test.py"]'
  two_files
  cd "${root}"

  run -0 --separate-stderr purba

  printed 'tests/b_test.py::test_b passed' '1 case: 1 passed'
}

@test "a pattern outside the closed list names the nearest, and no test runs" {
  project 'test-files = ["test*.py"]'
  two_files
  cd "${root}"

  run -4 --separate-stderr purba

  outside 'test*.py' 'test_*.py'
  printed '0 cases; 1 project error'
}

@test "a string in place of a list is a project error" {
  project 'test-files = "*_test.py"'
  two_files
  cd "${root}"

  run -4 --separate-stderr purba

  not_a_list 'a string'
  printed '0 cases; 1 project error'
}

@test "a number in place of a list is a project error" {
  project 'test-files = 5'
  two_files
  cd "${root}"

  run -4 --separate-stderr purba

  not_a_list 'a number'
  printed '0 cases; 1 project error'
}

@test "an empty list is a project error" {
  project 'test-files = []'
  two_files
  cd "${root}"

  run -4 --separate-stderr purba

  empty pyproject.toml
  printed '0 cases; 1 project error'
}

@test "each item outside the closed list gives a line, and a number reads as written" {
  project 'test-files = ["tset_*.py", "*_test.py", "tests_*.py", 7]'
  two_files
  cd "${root}"

  run -4 --separate-stderr purba

  outside 'tset_*.py' 'test_*.py'
  outside 'tests_*.py' 'test_*.py'
  outside 7 test.py
  printed '0 cases; 3 project errors'
}

@test "from a folder below the root, the error names the file from the working folder" {
  project 'test-files = []'
  two_files
  cd "${root}/tests"

  run -4 --separate-stderr purba

  empty ../pyproject.toml
  printed '0 cases; 1 project error'
}

@test "a path to a file that matches a pattern of the list runs it" {
  project 'test-files = ["*_test.py"]'
  two_files
  cd "${root}"

  run -0 --separate-stderr purba tests/b_test.py

  printed 'tests/b_test.py::test_b passed' '1 case: 1 passed'
}

@test "a path that matches no pattern of the list is an invocation error" {
  project 'test-files = ["*_test.py"]'
  two_files
  cd "${root}"

  run -3 --separate-stderr purba tests/test_a.py

  refused tests/test_a.py 'its name does not match *_test.py'
}

@test "with two patterns, the invocation error of a path names both" {
  project 'test-files = ["test_*.py", "*_test.py"]'
  passing "${root}/tests/tests.py" test_tests
  git -C "${root}" add --all
  cd "${root}"

  run -3 --separate-stderr purba tests/tests.py

  refused tests/tests.py 'its name matches none of test_*.py and *_test.py'
}

@test "a key under [tool.purba] that no dial owns names the nearest dial" {
  project 'tset-files = ["*_test.py"]'
  two_files
  cd "${root}"

  run -4 --separate-stderr purba

  unknown tset-files
  printed '0 cases; 1 project error'
}

@test "a table under [tool.purba] that no dial owns is a project error" {
  project '' '[tool.purba.platform]' 'linux = "path(tests/linux_only/**)"'
  two_files
  cd "${root}"

  run -4 --separate-stderr purba

  unknown platform test-files
  printed '0 cases; 1 project error'
}

@test "purba names each error of the project's file before it stops" {
  project 'test-files = []' 'workers = 4'
  two_files
  cd "${root}"

  run -4 --separate-stderr purba

  empty pyproject.toml
  unknown workers queries
  printed '0 cases; 2 project errors'
}

@test "a named path with a refused value exits 4, and names the error of the list" {
  project 'test-files = []'
  two_files
  cd "${root}"

  run -4 --separate-stderr purba tests/b_test.py

  empty pyproject.toml
  printed '0 cases; 1 project error'
}

@test "test-files in the user's file or in the environment changes nothing" {
  project
  two_files
  mkdir -p "${XDG_CONFIG_HOME}/purba"
  printf '%s\n' 'test-files = ["*_test.py"]' >"${XDG_CONFIG_HOME}/purba/purba.toml"
  cd "${root}"

  PURBA_TEST_FILES='*_test.py' run -0 --separate-stderr purba

  printed 'tests/test_a.py::test_a passed' '1 case: 1 passed'
}
