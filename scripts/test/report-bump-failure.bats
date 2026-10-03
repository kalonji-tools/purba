# .github/scripts/report-bump-failure.sh when it cannot run.
: "${BATS_TEST_DIRNAME:?set by bats}"

setup() {
  # shellcheck source=scripts/test/fixture.sh
  . "${BATS_TEST_DIRNAME}/fixture.sh"
  script="${BATS_TEST_DIRNAME}/../../.github/scripts/report-bump-failure.sh"
  isolate
  fake gh </dev/null
}

@test "with an empty repository name it exits 2 before it calls the API" {
  export GH_REPO=

  run "${script}" https://example.invalid/run 39

  [[ "${status}" -eq 2 ]]
  [[ "${output}" == *"GH_REPO is set by the workflow env, and it is empty here."* ]]
  asked=$(calls gh)
  [[ -z "${asked}" ]]
}
