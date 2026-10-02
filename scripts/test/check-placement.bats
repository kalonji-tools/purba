# scripts/check-placement.sh against the trees it refuses.
: "${BATS_TEST_DIRNAME:?set by bats}"

setup() {
  # shellcheck source=scripts/test/fixture.sh
  . "${BATS_TEST_DIRNAME}/fixture.sh"
  script="${BATS_TEST_DIRNAME}/../check-placement.sh"
  make_repo
  stranded="A script is refused when every caller of it lives in the other half"
  unreachable="A script under .github that nothing under .github names is reached by nothing"
}

# A tracked file holding one line.
track() {
  mkdir -p "$(dirname "$1")"
  printf '%s\n' "$2" >"$1"
  git add -- "$1"
}

# One script in each half, each named from its own half.
placed() {
  track scripts/tool.sh 'echo tool'
  track tasks.toml 'run = "scripts/tool.sh"'
  track .github/scripts/job.sh 'echo job'
  track .github/workflows/w.yml 'run: .github/scripts/job.sh'
}

@test "a script named from its own half passes" {
  placed

  run "${script}"

  [[ "${status}" -eq 0 ]]
  [[ -z "${output}" ]]
}

@test "a script under .github named only by a task is refused" {
  placed
  track .github/workflows/w.yml 'run: true'
  track tasks.toml 'run = ".github/scripts/job.sh"'

  run "${script}"

  [[ "${status}" -eq 1 ]]
  [[ "${output}" == *"${stranded}"* ]]
  [[ "${output}" == *".github/scripts/job.sh is named only by tasks.toml"* ]]
  [[ "${output}" != *"${unreachable}"* ]]
}

@test "a script outside .github named only by a workflow is refused" {
  placed
  track tasks.toml 'run = "true"'
  track .github/workflows/w.yml 'run: .github/scripts/job.sh && scripts/tool.sh'

  run "${script}"

  [[ "${status}" -eq 1 ]]
  [[ "${output}" == *"${stranded}"* ]]
  [[ "${output}" == *"scripts/tool.sh is named only by .github/workflows/w.yml"* ]]
  [[ "${output}" != *"${unreachable}"* ]]
}

@test "a script named from both halves passes" {
  placed
  track tasks.toml 'run = "scripts/tool.sh .github/scripts/job.sh"'

  run "${script}"

  [[ "${status}" -eq 0 ]]
}

@test "a script under .github that nothing names is refused" {
  placed
  track .github/scripts/orphan.sh 'echo orphan'

  run "${script}"

  [[ "${status}" -eq 1 ]]
  [[ "${output}" == *"${unreachable}"* ]]
  [[ "${output}" == *".github/scripts/orphan.sh"* ]]
  [[ "${output}" != *"${stranded}"* ]]
}

@test "a script outside .github that nothing names passes, because a person runs it" {
  placed
  track scripts/by-hand.sh 'echo by hand'

  run "${script}"

  [[ "${status}" -eq 0 ]]
}

@test "a name on a comment line is not a caller, indented or not" {
  for comment in '# run: .github/scripts/job.sh' '      # run: .github/scripts/job.sh'; do
    placed
    track .github/workflows/w.yml "${comment}"

    run "${script}"

    [[ "${status}" -eq 1 ]]
    [[ "${output}" == *"${unreachable}"* ]]
  done
}

# The caller is keyed on the name, so the new file borrows the placed one's
# caller and is refused for it. The placed one passes.
@test "a second script of one name, in the other half, is refused" {
  placed
  track scripts/job.sh 'echo a second job'

  run "${script}"

  [[ "${status}" -eq 1 ]]
  [[ "${output}" == *"  scripts/job.sh is named only by .github/workflows/w.yml"* ]]
  [[ "${output}" != *"  .github/scripts/job.sh"* ]]
}

@test "a bare mention of the name, with no slash before it, is not a caller" {
  placed
  track .github/workflows/w.yml 'name: what job.sh does'

  run "${script}"

  [[ "${status}" -eq 1 ]]
  [[ "${output}" == *"${unreachable}"* ]]
}

@test "a script that names itself is not its own caller" {
  placed
  track .github/workflows/w.yml 'run: true'
  track .github/scripts/job.sh 'echo "usage: .github/scripts/job.sh"'

  run "${script}"

  [[ "${status}" -eq 1 ]]
  [[ "${output}" == *"${unreachable}"* ]]
}

@test "a sibling that sources a script is its caller" {
  placed
  track .github/scripts/lib.sh 'helper() { :; }'
  # The line is what a script holds, so nothing in it expands here.
  # shellcheck disable=SC2016
  track .github/scripts/job.sh '. "$(dirname "$0")/lib.sh"'

  run "${script}"

  [[ "${status}" -eq 0 ]]
}

@test "a hook that names a script is its caller" {
  placed
  track tasks.toml 'run = "true"'
  track prek.toml 'entry = "scripts/tool.sh"'
  track .github/workflows/w.yml 'run: .github/scripts/job.sh && scripts/tool.sh'

  run "${script}"

  [[ "${status}" -eq 0 ]]
}

@test "every misplaced script is reported in one run" {
  placed
  track tasks.toml 'run = "true"'
  track .github/workflows/w.yml 'run: scripts/tool.sh'

  run "${script}"

  [[ "${status}" -eq 1 ]]
  [[ "${output}" == *"scripts/tool.sh is named only by .github/workflows/w.yml"* ]]
  [[ "${output}" == *"${unreachable}"* ]]
  [[ "${output}" == *".github/scripts/job.sh"* ]]
}

@test "a script that is not tracked is not read" {
  placed
  mkdir -p .github/scripts
  printf 'echo orphan\n' >.github/scripts/orphan.sh

  run "${script}"

  [[ "${status}" -eq 0 ]]
}

@test "on a runner the refusal is one annotation" {
  placed
  track .github/scripts/orphan.sh 'echo orphan'

  GITHUB_ACTIONS=true run "${script}"

  [[ "${status}" -eq 1 ]]
  [[ "${output}" == "::error::${unreachable}"*"%0A  .github/scripts/orphan.sh" ]]
}

@test "a tree with no tracked script exits 2" {
  run "${script}"

  [[ "${status}" -eq 2 ]]
  [[ "${output}" == *"no script is tracked, so no placement can be decided."* ]]
}

@test "outside a git repository it exits 2" {
  mkdir "${BATS_TEST_TMPDIR}/bare"
  cd "${BATS_TEST_TMPDIR}/bare"

  run "${script}"

  [[ "${status}" -eq 2 ]]
  [[ "${output}" == *"a caller can only be read inside a git repository."* ]]
}
