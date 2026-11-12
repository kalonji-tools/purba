# .github/scripts/lanes.sh against the lanes and the lints it reads.
: "${BATS_TEST_DIRNAME:?set by bats}"
bats_require_minimum_version 1.5.0

setup() {
  # shellcheck source=scripts/test/fixture.sh
  . "${BATS_TEST_DIRNAME}/fixture.sh"
  script="${BATS_TEST_DIRNAME}/../../.github/scripts/lanes.sh"
  make_repo
  tasks lint:rust lint:manifests lint:shell lint:records lint:links lint:new
  lanes '# a comment' '' \
    'lint:rust       rust' \
    'lint:manifests  rust scripts' \
    'lint:shell      scripts' \
    'lint:records    docs' \
    'lint:links      always'
}

# .config/tasks.toml defines each lint it is given, in that order.
tasks() {
  local task
  mkdir -p .config
  for task in "$@"; do
    printf '["%s"]\nrun = "true"\n\n' "${task}"
  done >.config/tasks.toml
}

# .config/lanes is written whole, one argument to a line.
lanes() {
  mkdir -p .config
  printf '%s\n' "$@" >.config/lanes
}

@test "--names prints each lane but always, once, and no comment" {
  run --separate-stderr "${script}" --names

  [[ "${status}" -eq 0 ]]
  [[ "${output}" == $'docs\nrust\nscripts' ]]
}

@test "--tasks with no lane prints the lints of always and each lint no lane names" {
  run --separate-stderr "${script}" --tasks

  [[ "${status}" -eq 0 ]]
  [[ "${output}" == "lint:links ::: lint:new" ]]
}

@test "--tasks with a lane adds its lints, in the order of .config/tasks.toml" {
  run --separate-stderr "${script}" --tasks scripts

  [[ "${status}" -eq 0 ]]
  [[ "${output}" == "lint:manifests ::: lint:shell ::: lint:links ::: lint:new" ]]
}

@test "a lint in two lanes runs when either lane started" {
  run --separate-stderr "${script}" --tasks rust

  [[ "${status}" -eq 0 ]]
  [[ "${output}" == "lint:rust ::: lint:manifests ::: lint:links ::: lint:new" ]]
}

@test "--tasks with every lane prints every lint, and skips none" {
  run --separate-stderr "${script}" --tasks rust scripts docs

  every="lint:rust ::: lint:manifests ::: lint:shell ::: lint:records ::: lint:links"
  every+=" ::: lint:new"
  [[ "${status}" -eq 0 ]]
  [[ "${output}" == "${every}" ]]
  [[ "${stderr}" == *"lints skipped: none"* ]]
}

@test "standard error names the lanes that started and each lint skipped" {
  run --separate-stderr "${script}" --tasks docs

  [[ "${status}" -eq 0 ]]
  [[ "${stderr}" == *"lanes that started: always docs"* ]]
  [[ "${stderr}" == *"lints skipped: lint:rust lint:manifests lint:shell"* ]]
}

@test "a lane that .config/lanes does not name adds no lint" {
  run --separate-stderr "${script}" --tasks unknown

  [[ "${status}" -eq 0 ]]
  [[ "${output}" == "lint:links ::: lint:new" ]]
}

@test "run below the top, it reads the files of the top" {
  mkdir -p deep/below
  cd deep/below

  run --separate-stderr "${script}" --names

  [[ "${status}" -eq 0 ]]
  [[ "${output}" == $'docs\nrust\nscripts' ]]
}

@test "a missing .config/lanes exits 2 and prints nothing" {
  rm .config/lanes

  run --separate-stderr "${script}" --names

  [[ "${status}" -eq 2 ]]
  [[ -z "${output}" ]]
  [[ "${stderr}" == *".config/lanes cannot be read."* ]]
}

@test "a .config/tasks.toml that defines no lint exits 2 and prints nothing" {
  tasks build test

  run --separate-stderr "${script}" --tasks

  [[ "${status}" -eq 2 ]]
  [[ -z "${output}" ]]
  [[ "${stderr}" == *".config/tasks.toml defines no lint."* ]]
}

@test "no option exits 2 and names the usage" {
  run --separate-stderr "${script}"

  [[ "${status}" -eq 2 ]]
  [[ -z "${output}" ]]
  [[ "${stderr}" == *"usage: lanes.sh --names | --tasks [<lane>...]"* ]]
}
