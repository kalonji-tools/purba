# .github/scripts/report-publish-failure.sh when it cannot run, and the issue it opens.
: "${BATS_TEST_DIRNAME:?set by bats}"

setup() {
  # shellcheck source=scripts/test/fixture.sh
  . "${BATS_TEST_DIRNAME}/fixture.sh"
  script="${BATS_TEST_DIRNAME}/../../.github/scripts/report-publish-failure.sh"
  isolate
  fake gh </dev/null
  export GH_REPO=owner/name
}

@test "with an empty repository name it exits 2 before it calls the API" {
  export GH_REPO=

  run "${script}" https://example.invalid/run 48

  [[ "${status}" -eq 2 ]]
  [[ "${output}" == *"GH_REPO is set by the workflow env, and it is empty here."* ]]
  asked=$(calls gh)
  [[ -z "${asked}" ]]
}

@test "with the wrong arguments it exits 2 before it calls the API" {
  run "${script}" https://example.invalid/run

  [[ "${status}" -eq 2 ]]
  [[ "${output}" == *"usage: report-publish-failure.sh <run-url> <issue>"* ]]
  asked=$(calls gh)
  [[ -z "${asked}" ]]
}

@test "the issue it opens names the upload, how to retry it, and its two labels" {
  run "${script}" https://example.invalid/run 48

  [[ "${status}" -eq 0 ]]
  asked=$(calls gh)
  [[ "${asked}" == *"issue create"*"--title The publish run failed"* ]]
  [[ "${asked}" == *"issue create"*"--label bug"* ]]
  [[ "${asked}" == *"issue create"*"--label wayfinder:task"* ]]
  [[ "${asked}" == *".github/workflows/publish.yml"* ]]
  [[ "${asked}" == *"gh workflow run publish.yml --ref v<version>"* ]]
  [[ "${asked}" == *"https://example.invalid/run"* ]]
}

@test "an open issue with the same title gets a comment, and no second issue opens" {
  fake gh <<'FAKE'
[[ "$1 $2" != "issue list" ]] || echo 7
FAKE

  run "${script}" https://example.invalid/run 48

  [[ "${status}" -eq 0 ]]
  asked=$(calls gh)
  said="issue comment 7 --repo owner/name --body It failed again: https://example.invalid/run"
  [[ "${asked}" == *"${said}"* ]]
  [[ "${asked}" != *"issue create"* ]]
}
