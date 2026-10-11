# .github/scripts/changed.sh against the pull requests it reads.
: "${BATS_TEST_DIRNAME:?set by bats}"
bats_require_minimum_version 1.5.0

setup() {
  # shellcheck source=scripts/test/fixture.sh
  . "${BATS_TEST_DIRNAME}/fixture.sh"
  script="${BATS_TEST_DIRNAME}/../../.github/scripts/changed.sh"
  make_repo
  mkdir -p .config src docs scripts
  printf '%s\n' \
    '.config/gates  build   -scripts' \
    'chorestart     -build  -scripts' \
    'src/**         build   -scripts' \
    'docs/**        -build  -scripts' \
    'scripts/**     -build  scripts' \
    '.gitattributes -build  -scripts' >.config/gates
  echo code >src/lib.rs
  echo note >docs/note.md
  echo tool >scripts/tool.sh
  git add .config src docs scripts
  git commit --quiet --message "chore: a file in each directory"
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

@test "a changed path marked with the gate starts it" {
  echo more >>src/lib.rs
  merged

  run --separate-stderr "${script}" build HEAD^1 HEAD

  [[ "${status}" -eq 0 ]]
  [[ "${output}" == "build=true" ]]
  [[ "${stderr}" == "src/lib.rs is not marked -build, so the gate runs" ]]
}

@test "a pull request whose every path is marked - prints nothing and says so" {
  echo more >>docs/note.md
  echo more >>scripts/tool.sh
  merged

  run --separate-stderr "${script}" build HEAD^1 HEAD

  [[ "${status}" -eq 0 ]]
  [[ -z "${output}" ]]
  [[ "${stderr}" == ".config/gates marks every changed path -build, so the gate does not run" ]]
}

@test "a path the file leaves unmarked starts the gate" {
  echo more >>docs/note.md
  echo 'fn main() {}' >build.rs
  merged

  run --separate-stderr "${script}" build HEAD^1 HEAD

  [[ "${status}" -eq 0 ]]
  [[ "${output}" == "build=true" ]]
  [[ "${stderr}" == "build.rs is not marked -build, so the gate runs" ]]
}

@test "a deleted path marked with the gate starts it" {
  git rm --quiet src/lib.rs
  merged

  run --separate-stderr "${script}" build HEAD^1 HEAD

  [[ "${status}" -eq 0 ]]
  [[ "${output}" == "build=true" ]]
}

# git names a renamed file by its new path, which the file marks -build.
@test "a path moved out of a directory marked with the gate starts it" {
  git mv src/lib.rs docs/lib.rs
  merged

  run --separate-stderr "${script}" build HEAD^1 HEAD

  [[ "${status}" -eq 0 ]]
  [[ "${output}" == "build=true" ]]
}

@test "each gate reads its own attribute from the same file" {
  echo more >>scripts/tool.sh
  merged

  run --separate-stderr "${script}" scripts HEAD^1 HEAD

  [[ "${status}" -eq 0 ]]
  [[ "${output}" == "scripts=true" ]]

  run --separate-stderr "${script}" build HEAD^1 HEAD

  [[ "${status}" -eq 0 ]]
  [[ -z "${output}" ]]
}

@test "the repository's own attributes do not mark a path for the gate" {
  echo 'build.rs  -build' >.gitattributes
  echo 'fn main() {}' >build.rs
  merged

  run --separate-stderr "${script}" build HEAD^1 HEAD

  [[ "${status}" -eq 0 ]]
  [[ "${output}" == "build=true" ]]
}

@test "the wrong number of arguments exits 2" {
  run --separate-stderr "${script}" build HEAD

  [[ "${status}" -eq 2 ]]
  [[ -z "${output}" ]]
  [[ "${stderr}" == *"usage: changed.sh <attr> <base> <head>"* ]]
}

# The commit setup leaves on `main` has a parent, and the first commit has none.
@test "a base that cannot be read exits 2 and prints nothing" {
  root=$(git rev-list --max-parents=0 main)

  run --separate-stderr "${script}" build "${root}^1" "${root}"

  [[ "${status}" -eq 2 ]]
  [[ -z "${output}" ]]
  [[ "${stderr}" == *"could not be read."* ]]
}

# `.config/gates` marks itself `-scripts`, so only the path the head unmarks can
# start the gate.
@test "a head that unmarks its own change still starts the gate" {
  echo more >>scripts/tool.sh
  printf '%s\n' '.config/gates  build   -scripts' 'scripts/**     -build  -scripts' >.config/gates
  merged

  run --separate-stderr "${script}" scripts HEAD^1 HEAD

  [[ "${status}" -eq 0 ]]
  [[ "${output}" == "scripts=true" ]]
  [[ "${stderr}" == "scripts/tool.sh is not marked -scripts, so the gate runs" ]]
}

@test "a base that holds no .config/gates exits 2 and prints nothing" {
  git rm --quiet .config/gates
  git commit --quiet --message "chore: no marks"
  git checkout --quiet HEAD^ -- .config/gates
  git commit --quiet --message "chore: the marks again"

  run --separate-stderr "${script}" build HEAD^1 HEAD

  [[ "${status}" -eq 2 ]]
  [[ -z "${output}" ]]
  [[ "${stderr}" == *".config/gates could not be read"* ]]
}
