# scripts/accept-one-commit.sh when it cannot run. `check-replayable.bats` reaches the rest.
: "${BATS_TEST_DIRNAME:?set by bats}"

setup() {
  # shellcheck source=scripts/test/fixture.sh
  . "${BATS_TEST_DIRNAME}/fixture.sh"
  script="${BATS_TEST_DIRNAME}/../accept-one-commit.sh"
  make_repo
}

@test "with no trailer to write it exits 2 and leaves the commit alone" {
  before=$(git rev-parse HEAD)

  run env -u TRAILER "${script}"

  [[ "${status}" -eq 2 ]]
  [[ "${output}" == *"TRAILER reaches this script through the environment"* ]]
  after=$(git rev-parse HEAD)
  [[ "${after}" == "${before}" ]]
}
