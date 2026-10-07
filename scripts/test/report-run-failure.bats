# .github/scripts/report-run-failure.sh when it cannot run, and the issue it opens for each run.
: "${BATS_TEST_DIRNAME:?set by bats}"

setup() {
  # shellcheck source=scripts/test/fixture.sh
  . "${BATS_TEST_DIRNAME}/fixture.sh"
  script="${BATS_TEST_DIRNAME}/../../.github/scripts/report-run-failure.sh"
  isolate
  fake gh </dev/null
  export GH_REPO=owner/name
}

runs=(
  "bump=The nightly bump failed"
  "release=The release run failed"
  "publish=The publish run failed"
)

@test "each run opens its issue with its own title, and carries bug and wayfinder:task" {
  for pair in "${runs[@]}"; do
    each=${pair%%=*}
    rm -f "${BATS_TEST_TMPDIR}/calls.gh"

    run "${script}" "${each}" https://example.invalid/run 43

    [[ "${status}" -eq 0 ]]
    asked=$(calls gh)
    [[ "${asked}" == *"issue create"*"--title ${pair#*=}"* ]]
    [[ "${asked}" == *"issue create"*"--label bug"* ]]
    [[ "${asked}" == *"issue create"*"--label wayfinder:task"* ]]
    [[ "${asked}" == *".github/workflows/${each}.yml"* ]]
    [[ "${asked}" == *"https://example.invalid/run"* ]]
  done
}

@test "for each run, an open issue with the same title gets a comment, and no second issue opens" {
  fake gh <<'FAKE'
[[ "$1 $2" != "issue list" ]] || echo 7
FAKE

  for pair in "${runs[@]}"; do
    each=${pair%%=*}
    rm -f "${BATS_TEST_TMPDIR}/calls.gh"

    run "${script}" "${each}" https://example.invalid/run 43

    [[ "${status}" -eq 0 ]]
    asked=$(calls gh)
    [[ "${asked}" == *"issue list"*"${pair#*=}"* ]]
    said="issue comment 7 --repo owner/name --body It failed again: https://example.invalid/run"
    [[ "${asked}" == *"${said}"* ]]
    [[ "${asked}" != *"issue create"* ]]
  done
}

@test "for each run, the wrong arguments exit 2 before it calls the API" {
  for each in "${runs[@]%%=*}"; do
    run "${script}" "${each}" https://example.invalid/run

    [[ "${status}" -eq 2 ]]
    [[ "${output}" == *"usage: report-run-failure.sh <run> <run-url> <issue>"* ]]
  done
  asked=$(calls gh)
  [[ -z "${asked}" ]]
}

@test "for each run, an empty repository name exits 2 before it calls the API" {
  export GH_REPO=

  for each in "${runs[@]%%=*}"; do
    run "${script}" "${each}" https://example.invalid/run 43

    [[ "${status}" -eq 2 ]]
    [[ "${output}" == *"GH_REPO is set by the workflow env, and it is empty here."* ]]
  done
  asked=$(calls gh)
  [[ -z "${asked}" ]]
}

@test "a run it does not know exits 2 before it calls the API" {
  run "${script}" nightly https://example.invalid/run 43

  [[ "${status}" -eq 2 ]]
  [[ "${output}" == *"<run> is bump, release or publish, and it is nightly here."* ]]
  asked=$(calls gh)
  [[ -z "${asked}" ]]
}
