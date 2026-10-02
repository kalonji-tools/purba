# .github/scripts/carry-verdicts.sh against the verdicts it will not carry.
: "${BATS_TEST_DIRNAME:?set by bats}"

setup() {
  # shellcheck source=scripts/test/fixture.sh
  . "${BATS_TEST_DIRNAME}/fixture.sh"
  script="${BATS_TEST_DIRNAME}/../../.github/scripts/carry-verdicts.sh"
  make_repo
  export GH_REPO=owner/name
  fake_gh

  # A rewrite that changed a message and no content: two commits, one tree.
  before=$(git rev-parse HEAD)
  git commit --quiet --amend --message "chore: start, accepted"
  after=$(git rev-parse HEAD)
  tree=$(git rev-parse 'HEAD^{tree}')
}

@test "a success is carried onto the rewritten head, under the same name" {
  check_runs Quality=success

  run "${script}" "${before}" "${after}" Quality

  [[ "${status}" -eq 0 ]]
  [[ "${output}" == *"carried Quality=success"* ]]
  asked=$(calls gh)
  [[ "${asked}" == *"api repos/owner/name/commits/${before}/check-runs?per_page=100"* ]]
  [[ "${asked}" == *"--method POST repos/owner/name/check-runs -f name=Quality"* ]]
  [[ "${asked}" == *"-f head_sha=${after} -f status=completed -f conclusion=success"* ]]
  [[ "${asked}" == *"output[title]=Quality, carried"* ]]
  [[ "${asked}" == *"The tree at this commit is \`${tree}\`"*"at \`${before}\`."* ]]
}

@test "a failure is carried as a failure" {
  check_runs Quality=failure

  run "${script}" "${before}" "${after}" Quality

  [[ "${status}" -eq 0 ]]
  [[ "${output}" == *"carried Quality=failure"* ]]
  asked=$(calls gh)
  [[ "${asked}" == *"-f conclusion=failure"* ]]
}

@test "a context with no verdict is not carried" {
  check_runs Build=success

  run "${script}" "${before}" "${after}" Quality

  [[ "${status}" -eq 0 ]]
  [[ "${output}" == *"Quality has no verdict on ${before}, so there is nothing to carry"* ]]
  asked=$(calls gh)
  [[ "${asked}" != *"--method POST"* ]]
}

@test "a conclusion that is neither success nor failure is not carried" {
  for conclusion in neutral cancelled skipped timed_out action_required stale startup_failure; do
    check_runs "Quality=${conclusion}"

    run "${script}" "${before}" "${after}" Quality

    [[ "${status}" -eq 0 ]]
    [[ "${output}" == *"Quality concluded ${conclusion} on ${before}, which is not a verdict"* ]]
  done
  asked=$(calls gh)
  [[ "${asked}" != *"--method POST"* ]]
}

@test "a context that is not carried does not stop the one after it" {
  check_runs Quality=skipped Build=success

  run "${script}" "${before}" "${after}" Docs Quality Build

  [[ "${status}" -eq 0 ]]
  [[ "${output}" == *"Docs has no verdict"* ]]
  [[ "${output}" == *"Quality concluded skipped"* ]]
  [[ "${output}" == *"carried Build=success"* ]]
  asked=$(calls gh)
  [[ "${asked}" == *"-f name=Build -f head_sha=${after}"* ]]
  [[ "${asked}" == *"output[title]=Build, carried"* ]]
  posted=$(grep -c -- '--method POST' <<<"${asked}")
  [[ "${posted}" -eq 1 ]]
}

@test "a rewrite that changed the content carries nothing" {
  check_runs Quality=success
  commit "feat: more content"
  moved=$(git rev-parse HEAD)

  run "${script}" "${before}" "${moved}" Quality

  [[ "${status}" -eq 0 ]]
  [[ "${output}" == *"the content changed, so no verdict is carried"* ]]
  asked=$(calls gh)
  [[ -z "${asked}" ]]
}

@test "a head that did not move carries nothing onto itself" {
  check_runs Quality=success

  run "${script}" "${after}" "${after}" Quality

  [[ "${status}" -eq 0 ]]
  [[ "${output}" == *"the head did not move, so there is nothing to carry"* ]]
  asked=$(calls gh)
  [[ -z "${asked}" ]]
}

@test "with an empty repository name it stops before it calls the API" {
  export GH_REPO=

  run "${script}" "${before}" "${after}" Quality

  [[ "${status}" -eq 2 ]]
  [[ "${output}" == *"set by the workflow env"* ]]
  asked=$(calls gh)
  [[ -z "${asked}" ]]
}

@test "fewer than three arguments exits 2" {
  run "${script}" "${before}" "${after}"

  [[ "${status}" -eq 2 ]]
  [[ "${output}" == *"usage: carry-verdicts.sh <before> <after> <context>..."* ]]
}

@test "a commit git does not hold exits 2" {
  run "${script}" 0000000000000000000000000000000000000000 "${after}" Quality

  [[ "${status}" -eq 2 ]]
  [[ "${output}" == *"could not be read."* ]]
  asked=$(calls gh)
  [[ -z "${asked}" ]]
}

@test "a read the API refuses exits 2" {
  fake gh <<<'echo "gh: not logged in" >&2; exit 4'

  run "${script}" "${before}" "${after}" Quality

  [[ "${status}" -eq 2 ]]
  [[ "${output}" == *"the check runs on ${before} could not be read."* ]]
}

@test "an answer that holds no check runs exits 2" {
  fake gh <<<'echo "{}"'

  run "${script}" "${before}" "${after}" Quality

  [[ "${status}" -eq 2 ]]
  [[ "${output}" == *"the check runs on ${before} could not be read."* ]]
}

@test "a verdict the API refuses to take exits 2" {
  check_runs Quality=success
  fake gh <<'FAKE'
case "$*" in
  *"--method POST"*) exit 4 ;;
  *) cat "${BATS_TEST_TMPDIR}/check-runs" ;;
esac
FAKE

  run "${script}" "${before}" "${after}" Quality

  [[ "${status}" -eq 2 ]]
  [[ "${output}" == *"the check run Quality could not be written onto ${after}."* ]]
  [[ "${output}" != *"carried Quality"* ]]
}
