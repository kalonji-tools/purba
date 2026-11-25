#   the decision:
# docs/decisions/purba-picks-a-test-by-its-name-and-the-tests-of-a-run-by-one-query.md
#   the decision:  docs/decisions/purba-reads-configuration-from-six-levels.md
#
# Stage one: the root, and the test files under it. Each test builds its
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
  one_passed=$(printf '%s\n' 'tests/test_a.py::test_add passed' '1 case: 1 passed')
}

# A folder that a test made unreadable takes its mode back, so that bats can
# remove the folder of the test.
teardown() {
  if [[ -n "${unreadable:-}" ]]; then
    chmod 700 "${unreadable}"
  fi
}

# A project at `$1` whose pyproject.toml holds `[tool.purba]`, under git.
project() {
  mkdir -p "$1"
  printf '[tool.purba]\n' >"$1/pyproject.toml"
  git -C "$1" init --quiet
}

# A module at `$1` with one test, `$2`, that passes.
passing() {
  mkdir -p "$(dirname "$1")"
  printf 'def %s():\n    assert 1 + 1 == 2\n' "$2" >"$1"
}

track() {
  git -C "${root}" add --all
}

@test "purba with no path runs each test file under the root" {
  project "${root}"
  passing "${root}/tests/test_a.py" test_add
  track
  cd "${root}"

  run -0 --separate-stderr purba

  diff -u - <(printf '%s\n' "${output}") <<<"${one_passed}"
  [[ -z "${stderr}" ]]
}

@test "purba with a folder runs each test file under that folder" {
  project "${root}"
  passing "${root}/tests/test_a.py" test_add
  passing "${root}/other/test_b.py" test_other
  track
  cd "${root}"

  run -0 --separate-stderr purba tests

  diff -u - <(printf '%s\n' "${output}") <<<"${one_passed}"
  [[ -z "${stderr}" ]]
}

@test "a folder that holds no test file prints only the count, and exits 2" {
  project "${root}"
  passing "${root}/tests/test_a.py" test_add
  passing "${root}/docs/conf.py" test_conf
  track
  cd "${root}"

  run -2 --separate-stderr purba docs

  [[ "${output}" == "0 cases" ]] || false
  [[ -z "${stderr}" ]]
}

@test "a path ends with a slash, starts with ./, or is ." {
  project "${root}"
  passing "${root}/tests/test_a.py" test_add
  track
  cd "${root}"

  local path
  for path in tests/ ./tests/test_a.py .; do
    run -0 --separate-stderr purba "${path}"

    diff -u - <(printf '%s\n' "${output}") <<<"${one_passed}"
    [[ -z "${stderr}" ]] || false
  done
}

@test "two test files of one name in two folders both run" {
  project "${root}"
  passing "${root}/a/test_x.py" test_a
  passing "${root}/b/test_x.py" test_b
  track
  cd "${root}"

  run -0 --separate-stderr purba

  diff -u - <(printf '%s\n' "${output}") <<'LINES'
a/test_x.py::test_a passed
b/test_x.py::test_b passed
2 cases: 2 passed
LINES
  [[ -z "${stderr}" ]]
}

@test "a test file in a hidden folder does not run" {
  project "${root}"
  passing "${root}/tests/test_a.py" test_add
  passing "${root}/.venv/lib/test_installed.py" test_installed
  track
  cd "${root}"

  run -0 --separate-stderr purba

  diff -u - <(printf '%s\n' "${output}") <<<"${one_passed}"
  [[ -z "${stderr}" ]]
}

@test "the path of each line starts at the root, from a folder below it" {
  project "${root}"
  passing "${root}/tests/test_a.py" test_add
  passing "${root}/test_top.py" test_top
  track
  cd "${root}/tests"

  run -0 --separate-stderr purba

  diff -u - <(printf '%s\n' "${output}") <<'LINES'
test_top.py::test_top passed
tests/test_a.py::test_add passed
2 cases: 2 passed
LINES
  [[ -z "${stderr}" ]]
}

@test "each folder sorts by the name of each part of its path" {
  project "${root}"
  passing "${root}/tests/a/test_x.py" test_x
  passing "${root}/tests/a.b/test_y.py" test_y
  track
  cd "${root}"

  run -0 --separate-stderr purba

  diff -u - <(printf '%s\n' "${output}") <<'LINES'
tests/a/test_x.py::test_x passed
tests/a.b/test_y.py::test_y passed
2 cases: 2 passed
LINES
  [[ -z "${stderr}" ]]
}

@test "a path starts at the root, not at the working folder" {
  project "${root}"
  passing "${root}/tests/test_a.py" test_add
  track
  cd "${root}/tests"

  run -0 --separate-stderr purba tests/test_a.py

  diff -u - <(printf '%s\n' "${output}") <<<"${one_passed}"
  [[ -z "${stderr}" ]]
}

@test "an absolute path under the root prints from the root" {
  project "${root}"
  passing "${root}/tests/test_a.py" test_add
  track
  cd "${root}"

  run -0 --separate-stderr purba "${root}/tests/test_a.py"

  diff -u - <(printf '%s\n' "${output}") <<<"${one_passed}"
  [[ -z "${stderr}" ]]
}

@test "a working folder and an absolute path through a link to the root print from the root" {
  project "${root}"
  passing "${root}/tests/test_a.py" test_add
  track
  ln -s "${root}" "${BATS_TEST_TMPDIR}/via"
  cd "${BATS_TEST_TMPDIR}/via"

  run -0 --separate-stderr purba "${BATS_TEST_TMPDIR}/via/tests/test_a.py"

  diff -u - <(printf '%s\n' "${output}") <<<"${one_passed}"
  [[ -z "${stderr}" ]]
}

@test "only test_*.py is a test file" {
  project "${root}"
  passing "${root}/tests/test_a.py" test_add
  passing "${root}/tests/models_test.py" test_model
  passing "${root}/tests/tests.py" test_tests
  passing "${root}/tests/test.py" test_test
  passing "${root}/tests/testing.py" test_testing
  track
  cd "${root}"

  run -0 --separate-stderr purba

  diff -u - <(printf '%s\n' "${output}") <<<"${one_passed}"
  [[ -z "${stderr}" ]]
}

@test "a folder that a .gitignore of the project ignores does not run" {
  project "${root}"
  passing "${root}/tests/test_a.py" test_add
  passing "${root}/gen/test_gen.py" test_gen
  printf 'gen/\n' >"${root}/.gitignore"
  track
  cd "${root}"

  run -0 --separate-stderr purba

  diff -u - <(printf '%s\n' "${output}") <<<"${one_passed}"
  [[ -z "${stderr}" ]]
}

@test "a .gitignore in a folder below the root ignores what it names there" {
  project "${root}"
  passing "${root}/tests/test_a.py" test_add
  passing "${root}/tests/gen/test_gen.py" test_gen
  printf 'gen/\n' >"${root}/tests/.gitignore"
  track
  cd "${root}"

  run -0 --separate-stderr purba

  diff -u - <(printf '%s\n' "${output}") <<<"${one_passed}"
  [[ -z "${stderr}" ]]
}

@test "a .gitignore above the folder that the path names ignores what it names there" {
  project "${root}"
  passing "${root}/tests/test_a.py" test_add
  passing "${root}/tests/gen/test_gen.py" test_gen
  printf 'tests/gen/\n' >"${root}/.gitignore"
  track
  cd "${root}"

  run -0 --separate-stderr purba tests

  diff -u - <(printf '%s\n' "${output}") <<<"${one_passed}"
  [[ -z "${stderr}" ]]
}

@test "a root below the folder that holds .git takes the walk, and the .gitignore between them" {
  local repository="${BATS_TEST_TMPDIR}/repository"
  local sub="${repository}/sub"
  mkdir -p "${sub}"
  git -C "${repository}" init --quiet
  printf '[tool.purba]\n' >"${sub}/pyproject.toml"
  printf 'sub/gen/\n' >"${repository}/.gitignore"
  passing "${sub}/tests/test_a.py" test_add
  passing "${sub}/gen/test_gen.py" test_gen
  cd "${sub}/tests"

  run -0 --separate-stderr purba

  diff -u - <(printf '%s\n' "${output}") <<<"${one_passed}"
  [[ -z "${stderr}" ]]
}

@test "outside git, a .gitignore ignores nothing" {
  mkdir -p "${root}"
  printf '[tool.purba]\n' >"${root}/pyproject.toml"
  passing "${root}/gen/test_gen.py" test_gen
  printf 'gen/\n' >"${root}/.gitignore"
  cd "${root}"

  run -0 --separate-stderr purba

  diff -u - <(printf '%s\n' "${output}") <<'LINES'
gen/test_gen.py::test_gen passed
1 case: 1 passed
LINES
  [[ -z "${stderr}" ]]
}

@test "a .ignore file ignores nothing" {
  project "${root}"
  passing "${root}/gen/test_gen.py" test_gen
  printf 'gen/\n' >"${root}/.ignore"
  track
  cd "${root}"

  run -0 --separate-stderr purba

  diff -u - <(printf '%s\n' "${output}") <<'LINES'
gen/test_gen.py::test_gen passed
1 case: 1 passed
LINES
  [[ -z "${stderr}" ]]
}

@test "a folder that only git's global file or the exclude of a clone ignores runs" {
  project "${root}"
  passing "${root}/local/test_local.py" test_local
  passing "${root}/mine/test_mine.py" test_mine
  printf 'local/\n' >"${HOME}/excludes"
  printf '[core]\n\texcludesFile = %s\n' "${HOME}/excludes" >"${HOME}/gitconfig"
  export GIT_CONFIG_GLOBAL="${HOME}/gitconfig"
  printf 'mine/\n' >>"${root}/.git/info/exclude"
  local unignored
  unignored=$(git -C "${root}" status --porcelain --untracked-files=all -- local mine)
  [[ -z "${unignored}" ]] || false
  cd "${root}"

  run -0 --separate-stderr purba

  diff -u - <(printf '%s\n' "${output}") <<'LINES'
local/test_local.py::test_local passed
mine/test_mine.py::test_mine passed
2 cases: 2 passed
LINES
  [[ -z "${stderr}" ]]
}

@test "a symbolic link to a folder outside the root does not run" {
  project "${root}"
  passing "${root}/tests/test_a.py" test_add
  passing "${BATS_TEST_TMPDIR}/outside/test_far.py" test_far
  ln -s "${BATS_TEST_TMPDIR}/outside" "${root}/tests/link"
  track
  cd "${root}"

  run -0 --separate-stderr purba

  diff -u - <(printf '%s\n' "${output}") <<<"${one_passed}"
  [[ -z "${stderr}" ]]
}

@test "a symbolic link to a folder or a test file inside the root is not followed" {
  project "${root}"
  passing "${root}/other/test_o.py" test_o
  mkdir -p "${root}/tests"
  ln -s ../other "${root}/tests/link"
  ln -s ../other/test_o.py "${root}/tests/test_link.py"
  track
  cd "${root}"

  run -0 --separate-stderr purba

  diff -u - <(printf '%s\n' "${output}") <<'LINES'
other/test_o.py::test_o passed
1 case: 1 passed
LINES
  [[ -z "${stderr}" ]]
}

@test "a module that does not parse stops only itself" {
  project "${root}"
  passing "${root}/tests/test_a.py" test_add
  printf 'def test_broken(:\n' >"${root}/tests/test_b.py"
  track
  cd "${root}"

  run -4 --separate-stderr purba

  diff -u - <(printf '%s\n' "${output}") <<'LINES'
tests/test_b.py project error: SyntaxError: Expected a parameter or the end of the parameter list
tests/test_a.py::test_add passed
1 case: 1 passed; 1 project error
LINES
  [[ -z "${stderr}" ]]
}

@test "a module that raises at its load stops only itself" {
  project "${root}"
  passing "${root}/tests/test_a.py" test_add
  printf 'raise RuntimeError("boom")\n\n\ndef test_b():\n    assert 1 + 1 == 2\n' \
    >"${root}/tests/test_b.py"
  track
  cd "${root}"

  run -4 --separate-stderr purba

  local expected actual
  expected=$(printf '%s\n' 'tests/test_a.py::test_add passed' \
    'tests/test_b.py project error: RuntimeError: boom' \
    'tests/test_b.py::test_b blocked: tests/test_b.py did not load' \
    '2 cases: 1 passed, 1 blocked; 1 project error' | LC_ALL=C sort)
  actual=$(printf '%s\n' "${output}" | LC_ALL=C sort)
  diff -u - <(printf '%s\n' "${actual}") <<<"${expected}"
  [[ -z "${stderr}" ]]
}

@test "a module that purba cannot read stops only itself" {
  local user
  user=$(id -u)
  [[ "${user}" != 0 ]] || skip "root reads a file with no permission"
  project "${root}"
  passing "${root}/tests/test_a.py" test_add
  passing "${root}/tests/test_b.py" test_b
  track
  chmod 000 "${root}/tests/test_b.py"
  cd "${root}"

  run -4 --separate-stderr purba

  [[ "${#lines[@]}" == 3 ]] || false
  [[ "${lines[0]}" == "tests/test_b.py project error: "* ]] || false
  [[ "${lines[1]}" == "tests/test_a.py::test_add passed" ]] || false
  [[ "${lines[2]}" == "1 case: 1 passed; 1 project error" ]] || false
  [[ -z "${stderr}" ]]
}

@test "a folder that stage one cannot read is a project error, and the other files run" {
  local user
  user=$(id -u)
  [[ "${user}" != 0 ]] || skip "root reads a folder with no permission"
  project "${root}"
  passing "${root}/tests/test_a.py" test_add
  passing "${root}/secret/test_s.py" test_s
  track
  unreadable="${root}/secret"
  chmod 000 "${unreadable}"
  cd "${root}"

  run -4 --separate-stderr purba

  diff -u - <(printf '%s\n' "${output}") <<'LINES'
secret project error: Permission denied (os error 13)
tests/test_a.py::test_add passed
1 case: 1 passed; 1 project error
LINES
  [[ -z "${stderr}" ]]
}

@test "a test that selection refuses in one file prints before a test of another file runs" {
  project "${root}"
  passing "${root}/tests/test_a.py" test_add
  printf 'async def test_b():\n    assert 1 + 1 == 2\n' >"${root}/tests/test_b.py"
  track
  cd "${root}"

  run -4 --separate-stderr purba

  diff -u - <(printf '%s\n' "${output}") <<'LINES'
tests/test_b.py::test_b project error: purba cannot run an async test yet
tests/test_a.py::test_add passed
2 cases: 1 passed, 1 blocked; 1 project error
LINES
  [[ -z "${stderr}" ]]
}

@test "a message that names a module names it by its path from the root" {
  project "${root}"
  mkdir -p "${root}/tests"
  printf '_purba_ran = 1\n\n\ndef test_x():\n    assert 1 + 1 == 2\n' >"${root}/tests/test_x.py"
  track
  cd "${root}"

  run -4 --separate-stderr purba

  local error='tests/test_x.py project error: NameError: '
  error+="tests/test_x.py binds _purba_ran, the name of purba's assert hook"
  local expected
  expected=$(printf '%s\n' "${error}" \
    'tests/test_x.py::test_x blocked: tests/test_x.py did not load' \
    '1 case: 1 blocked; 1 project error')
  diff -u - <(printf '%s\n' "${output}") <<<"${expected}"
  [[ -z "${stderr}" ]]
}

@test "a root with no test file prints only the count, and exits 2" {
  project "${root}"
  track
  cd "${root}"

  run -2 --separate-stderr purba

  [[ "${output}" == "0 cases" ]] || false
  [[ -z "${stderr}" ]]
}

@test "the walk passes over a pyproject.toml that holds no [tool.purba]" {
  project "${root}"
  mkdir -p "${root}/pkg"
  printf '[project]\nname = "pkg"\n' >"${root}/pkg/pyproject.toml"
  passing "${root}/pkg/test_p.py" test_p
  track
  cd "${root}/pkg"

  run -0 --separate-stderr purba

  diff -u - <(printf '%s\n' "${output}") <<'LINES'
pkg/test_p.py::test_p passed
1 case: 1 passed
LINES
  [[ -z "${stderr}" ]]
}

@test "with no [tool.purba], purba refuses the run and names the pyproject.toml of the project" {
  mkdir -p "${root}/tests"
  printf '[project]\nname = "p"\n' >"${root}/pyproject.toml"
  git -C "${root}" init --quiet
  passing "${root}/tests/test_a.py" test_add
  cd "${root}/tests"

  run -3 --separate-stderr purba

  local refused='purba: invocation error: no pyproject.toml holds [tool.purba]; '
  refused+='add it to ../pyproject.toml'
  [[ "${stderr}" == "${refused}" ]] || false
  [[ -z "${output}" ]]
}

@test "with no [tool.purba] and no pyproject.toml, purba names the folder that holds .git" {
  mkdir -p "${root}/tests"
  git -C "${root}" init --quiet
  passing "${root}/tests/test_a.py" test_add
  cd "${root}/tests"

  run -3 --separate-stderr purba

  local refused='purba: invocation error: no pyproject.toml holds [tool.purba]; '
  refused+='add one with it to ..'
  [[ "${stderr}" == "${refused}" ]] || false
  [[ -z "${output}" ]]
}

@test "the walk starts at the working folder, never at the path" {
  project "${root}"
  passing "${root}/tests/test_a.py" test_add
  track
  cd "${BATS_TEST_TMPDIR}"

  run -3 --separate-stderr purba "${root}/tests/test_a.py"

  local refused='purba: invocation error: no pyproject.toml holds [tool.purba]; '
  refused+='add one with it to .'
  [[ "${stderr}" == "${refused}" ]] || false
  [[ -z "${output}" ]]
}

@test "the walk stops at a .git file, as in a worktree, and refuses a [tool.purba] above it" {
  printf '[tool.purba]\n' >"${BATS_TEST_TMPDIR}/pyproject.toml"
  mkdir -p "${root}"
  printf 'gitdir: %s\n' "${BATS_TEST_TMPDIR}/elsewhere" >"${root}/.git"
  passing "${root}/test_a.py" test_add
  cd "${root}"

  run -3 --separate-stderr purba

  local refused='purba: invocation error: no pyproject.toml holds [tool.purba]; '
  refused+='add one with it to .'
  [[ "${stderr}" == "${refused}" ]] || false
  [[ -z "${output}" ]]
}

@test "a pyproject.toml on the walk that is not TOML is a project error" {
  mkdir -p "${root}"
  printf '[tool.purba\n' >"${root}/pyproject.toml"
  git -C "${root}" init --quiet
  passing "${root}/test_a.py" test_add
  cd "${root}"

  run -4 --separate-stderr purba

  diff -u - <(printf '%s\n' "${output}") <<'LINES'
pyproject.toml project error: TOML parse error at line 1, column 12: unclosed table, expected `]`
0 cases; 1 project error
LINES
  [[ -z "${stderr}" ]]
}

@test "a path to a file whose name is no test file's is an invocation error" {
  project "${root}"
  passing "${root}/tests/models_test.py" test_model
  track
  cd "${root}"

  run -3 --separate-stderr purba tests/models_test.py

  local refused='purba: invocation error: tests/models_test.py names no test file: '
  refused+='its name does not match test_*.py'
  [[ "${stderr}" == "${refused}" ]] || false
  [[ -z "${output}" ]]
}

@test "a path in a folder or to a folder that a .gitignore ignores is an invocation error" {
  project "${root}"
  passing "${root}/gen/test_gen.py" test_gen
  printf 'gen/\n' >"${root}/.gitignore"
  track
  cd "${root}"

  local path
  for path in gen/test_gen.py gen; do
    run -3 --separate-stderr purba "${path}"

    local refused="purba: invocation error: ${path} names no test file: "
    refused+='stage one skips gen, which a .gitignore ignores'
    [[ "${stderr}" == "${refused}" ]] || false
    [[ -z "${output}" ]] || false
  done
}

@test "a path in a hidden folder or to a hidden folder is an invocation error" {
  project "${root}"
  passing "${root}/.venv/lib/test_installed.py" test_installed
  track
  cd "${root}"

  local path
  for path in .venv/lib/test_installed.py .venv; do
    run -3 --separate-stderr purba "${path}"

    local refused="purba: invocation error: ${path} names no test file: "
    refused+='stage one skips .venv, a hidden folder'
    [[ "${stderr}" == "${refused}" ]] || false
    [[ -z "${output}" ]] || false
  done
}

@test "a path that names nothing is an invocation error" {
  project "${root}"
  track
  cd "${root}"

  run -3 --separate-stderr purba nowhere

  local refused='purba: invocation error: nowhere names no test file: '
  refused+='no file or folder has that path'
  [[ "${stderr}" == "${refused}" ]] || false
  [[ -z "${output}" ]]
}

@test "an absolute path outside the root is an invocation error" {
  project "${root}"
  passing "${BATS_TEST_TMPDIR}/outside/test_far.py" test_far
  track
  cd "${root}"

  run -3 --separate-stderr purba "${BATS_TEST_TMPDIR}/outside/test_far.py"

  local refused="purba: invocation error: ${BATS_TEST_TMPDIR}/outside/test_far.py "
  refused+='names no test file: it is outside the root'
  [[ "${stderr}" == "${refused}" ]] || false
  [[ -z "${output}" ]]
}

@test "two paths give the usage error" {
  project "${root}"
  passing "${root}/test_a.py" test_add
  passing "${root}/test_b.py" test_add
  track
  cd "${root}"

  run -3 --separate-stderr purba test_a.py test_b.py

  [[ "${stderr}" == "purba: invocation error: usage: purba [<path>]" ]] || false
  [[ -z "${output}" ]]
}
