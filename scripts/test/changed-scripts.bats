# .github/scripts/changed-scripts.sh against the pull requests it reads.
: "${BATS_TEST_DIRNAME:?set by bats}"
bats_require_minimum_version 1.5.0

setup() {
  # shellcheck source=scripts/test/fixture.sh
  . "${BATS_TEST_DIRNAME}/fixture.sh"
  script="${BATS_TEST_DIRNAME}/../../.github/scripts/changed-scripts.sh"
  make_repo
  mkdir -p scripts .github/scripts docs
  echo tool >scripts/tool.sh
  echo job >.github/scripts/job.sh
  echo note >docs/note.md
  git add scripts .github docs
  git commit --quiet --message "chore: a script in each half, and a note"
  git switch --quiet --create pr
}

# The merge commit a runner checks out. Its first parent is the base, and its
# second is the branch a test has just changed.
merged() {
  git add --all
  git commit --quiet --message "feat: a change"
  git switch --quiet --detach main
  git merge --quiet --no-ff pr --message "chore: merge"
}

@test "a script that changed is seen" {
  echo more >>scripts/tool.sh
  merged

  run --separate-stderr "${script}" HEAD^1 HEAD

  [[ "${status}" -eq 0 ]]
  [[ "${output}" == "scripts=true" ]]
  [[ "${stderr}" == "something a script test reads changed, so the script tests run" ]]
}

@test "a change to lychee.toml is seen" {
  echo 'exclude = []' >lychee.toml
  merged

  run --separate-stderr "${script}" HEAD^1 HEAD

  [[ "${status}" -eq 0 ]]
  [[ "${output}" == "scripts=true" ]]
  [[ "${stderr}" == "something a script test reads changed, so the script tests run" ]]
}

@test "a change to cliff.toml is seen" {
  echo "[bump]" >cliff.toml
  merged

  run --separate-stderr "${script}" HEAD^1 HEAD

  [[ "${status}" -eq 0 ]]
  [[ "${output}" == "scripts=true" ]]
}

@test "a change to pyproject.toml is seen" {
  echo "[project]" >pyproject.toml
  merged

  run --separate-stderr "${script}" HEAD^1 HEAD

  [[ "${status}" -eq 0 ]]
  [[ "${output}" == "scripts=true" ]]
}

@test "a script under .github that changed is seen" {
  echo more >>.github/scripts/job.sh
  merged

  run --separate-stderr "${script}" HEAD^1 HEAD

  [[ "${status}" -eq 0 ]]
  [[ "${output}" == "scripts=true" ]]
}

@test "a new test of a script is seen among other changes" {
  echo more >>docs/note.md
  mkdir scripts/test
  echo test >scripts/test/tool.bats
  merged

  run --separate-stderr "${script}" HEAD^1 HEAD

  [[ "${status}" -eq 0 ]]
  [[ "${output}" == "scripts=true" ]]
}

@test "a deleted script is seen" {
  git rm --quiet scripts/tool.sh
  merged

  run --separate-stderr "${script}" HEAD^1 HEAD

  [[ "${status}" -eq 0 ]]
  [[ "${output}" == "scripts=true" ]]
}

# git names a renamed file by its new path, which starts with neither directory.
@test "a script moved out of its directory is seen" {
  mkdir tools
  git mv scripts/tool.sh tools/tool.sh
  merged

  run --separate-stderr "${script}" HEAD^1 HEAD

  [[ "${status}" -eq 0 ]]
  [[ "${output}" == "scripts=true" ]]
}

@test "a file moved into a script directory is seen" {
  git mv docs/note.md .github/scripts/note.md
  merged

  run --separate-stderr "${script}" HEAD^1 HEAD

  [[ "${status}" -eq 0 ]]
  [[ "${output}" == "scripts=true" ]]
}

# git quotes such a path unless told otherwise, and the quote comes first.
@test "a script whose name holds a letter outside ASCII is seen" {
  echo tool >scripts/hé.sh
  merged

  run --separate-stderr "${script}" HEAD^1 HEAD

  [[ "${status}" -eq 0 ]]
  [[ "${output}" == "scripts=true" ]]
}

@test "a pull request that changes no script prints nothing and says so" {
  echo more >>docs/note.md
  echo task >tasks.toml
  echo lock >mise.lock
  merged

  run --separate-stderr "${script}" HEAD^1 HEAD

  [[ "${status}" -eq 0 ]]
  [[ -z "${output}" ]]
  [[ "${stderr}" == "nothing a script test reads changed, so the script tests do not run" ]]
}

@test "a path that only holds the word is not a script" {
  mkdir -p myscripts docs/scripts docs/.github/scripts
  echo tool >myscripts/tool.sh
  echo note >docs/scripts/note.md
  echo note >docs/.github/scripts/note.md
  echo 'exclude = []' >docs/lychee.toml
  echo "[bump]" >docs/cliff.toml
  echo "[project]" >docs/pyproject.toml
  merged

  run --separate-stderr "${script}" HEAD^1 HEAD

  [[ "${status}" -eq 0 ]]
  [[ -z "${output}" ]]
}

# The commit setup leaves on `main` has a parent, and the first commit has none.
@test "a base that cannot be read exits 2 and prints nothing" {
  root=$(git rev-list --max-parents=0 main)

  run --separate-stderr "${script}" "${root}^1" "${root}"

  [[ "${status}" -eq 2 ]]
  [[ -z "${output}" ]]
  [[ "${stderr}" == *"could not be read."* ]]
}

@test "the wrong number of arguments exits 2" {
  run --separate-stderr "${script}" HEAD

  [[ "${status}" -eq 2 ]]
  [[ "${stderr}" == *"usage: changed-scripts.sh <base> <head>"* ]]
}
