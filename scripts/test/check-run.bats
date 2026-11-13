# .github/scripts/check-run.sh against the verdicts it reads.
: "${BATS_TEST_DIRNAME:?set by bats}"

setup() {
  # shellcheck source=scripts/test/fixture.sh
  . "${BATS_TEST_DIRNAME}/fixture.sh"
  module="${BATS_TEST_DIRNAME}/../../.github/scripts/check-run.sh"
  isolate
  export GH_REPO=owner/name HEAD_REF=work PR=7 BASE_REF=main GH_TOKEN=actions-token \
    SIGN_TOKEN=sign-token
  fake_gh
  # shellcheck source=.github/scripts/check-run.sh
  . "${module}"
}

@test "a context that concluded success is green, and the head is what was asked about" {
  check_runs Quality=success

  run require_green abc123 Quality

  [[ "${status}" -eq 0 ]]
  [[ "${output}" == *"Quality is green on abc123"* ]]
  [[ "${output}" == *"every gate named here is green on abc123"* ]]
  asked=$(calls gh)
  [[ "${asked}" == *"api repos/owner/name/commits/abc123/check-runs?per_page=100"* ]]
}

@test "a gate counts only from a run of its own workflow on this pull request" {
  for source in api push branch file; do
    check_runs Quality=failure "Quality=success@2026-01-02~${source}"

    run require_green abc123 Quality

    [[ "${status}" -eq 1 ]]
    [[ "${output}" == *"Quality concluded failure on abc123"* ]]
  done
}

@test "a run that no workflow of the gate made is no verdict" {
  for source in api push branch file; do
    check_runs "Quality=success~${source}"

    run require_green abc123 Quality

    [[ "${status}" -eq 1 ]]
    [[ "${output}" == *"Quality has no verdict on abc123 yet"* ]]
  done
}

@test "a check run in the run of the gate's own file that ran on no runner is no verdict" {
  check_runs Quality=failure "Quality=success@2026-01-02~suite"

  run require_green abc123 Quality

  [[ "${status}" -eq 1 ]]
  [[ "${output}" == *"Quality concluded failure on abc123"* ]]
}

@test "the jobs of each run on this branch are read with filter=latest" {
  check_runs Quality=success Build=success Scripts=success~push

  run verdicts abc123 Quality Build

  [[ "${status}" -eq 0 ]]
  asked=$(calls gh)
  [[ "${asked}" == *"api repos/owner/name/actions/runs/101/jobs?filter=latest&per_page=100"* ]]
  [[ "${asked}" == *"api repos/owner/name/actions/runs/102/jobs?filter=latest&per_page=100"* ]]
  [[ "${asked}" != *"runs/103/jobs"* ]]
}

@test "a read of the jobs the API refuses exits 2" {
  check_runs Quality=success
  gh_refuses jobs

  run require_green abc123 Quality

  [[ "${status}" -eq 2 ]]
  [[ "${output}" == *"the jobs of the workflow run 101 could not be read."* ]]
}

@test "a verdict that purba-sign carried counts" {
  check_runs Quality=success~sign

  run require_green abc123 Quality

  [[ "${status}" -eq 0 ]]
  [[ "${output}" == *"Quality is green on abc123"* ]]
}

@test "a job counts only by a token whose claims name its run, this pull request and its base" {
  for filter in '.ref = "refs/pull/8/merge"' '.base_ref = "evil"' '.run_id = "102"' \
    '.run_attempt = "2"' '.check_run_id = "1002"' '.aud = "sigstore"' \
    '.iss = "https://example.invalid"' '.repository = "owner/other"' \
    '.event_name = "push"'; do
    check_runs Quality=success
    gate_token 101 "${filter}"

    run require_green abc123 Quality

    [[ "${status}" -eq 1 ]]
    [[ "${output}" == *"Quality has no verdict on abc123 yet"* ]]
  done
}

@test "a token that a key of the set did not sign by RS256 is no verdict" {
  # The key `other` signs under the name `gate`, and the alg HS256 heads a
  # signature that `gate` made by RS256.
  for signer in "other RS256" "short RS256" "gate none" "other RS256 gate" "gate HS256"; do
    check_runs Quality=success
    # shellcheck disable=SC2086 # a key, an alg and a name, one per word
    gate_token 101 . ${signer}

    run require_green abc123 Quality

    [[ "${status}" -eq 1 ]]
    [[ "${output}" == *"Quality has no verdict on abc123 yet"* ]]
  done
}

@test "a job of a rerun counts only by the token of its own attempt" {
  check_runs Quality=success
  jq -c '.jobs[0].run_attempt = 2' "${BATS_TEST_TMPDIR}/gh/jobs.json" >"${BATS_TEST_TMPDIR}/jobs"
  gh_answers jobs "$(<"${BATS_TEST_TMPDIR}/jobs")"

  run require_green abc123 Quality

  [[ "${status}" -eq 1 ]]
  [[ "${output}" == *"Quality has no verdict on abc123 yet"* ]]
}

@test "a run with no token is no verdict, and neither is one whose artifact expired" {
  for artifacts in '[]' '[{"id":5101,"name":"gate-token-1","expired":true}]'; do
    check_runs Quality=success
    gh_answers artifacts-101 "{\"artifacts\":${artifacts}}"

    run require_green abc123 Quality

    [[ "${status}" -eq 1 ]]
    [[ "${output}" == *"Quality has no verdict on abc123 yet"* ]]
  done
}

@test "the token is read from the artifact of the job's attempt, and the key set from GitHub" {
  check_runs Quality=success

  run require_green abc123 Quality

  [[ "${status}" -eq 0 ]]
  asked=$(calls gh)
  read_token="api repos/owner/name/actions/runs/101/artifacts?name=gate-token-1"
  [[ "${asked}" == *"${read_token}&per_page=100"* ]]
  [[ "${asked}" == *"api repos/owner/name/actions/artifacts/5101/zip"* ]]
  fetched=$(calls curl)
  [[ "${fetched}" == *"https://token.actions.githubusercontent.com/.well-known/jwks"* ]]
}

@test "a read of the token or of the key set that fails exits 2" {
  for refused in artifacts zip; do
    check_runs Quality=success
    gh_refuses "${refused}"

    run require_green abc123 Quality

    [[ "${status}" -eq 2 ]]
    [[ "${output}" == *"the token of the workflow run 101 could not be read."* ]]
    rm "${BATS_TEST_TMPDIR}/gh/${refused}.refused"
  done

  check_runs Quality=success
  fake curl <<<'echo "curl: (22) The requested URL returned error: 503" >&2; exit 22'

  run require_green abc123 Quality

  [[ "${status}" -eq 2 ]]
  [[ "${output}" == *"the key set of GitHub could not be read."* ]]
}

@test "with no pull request it exits 2, and names the script that finds it" {
  check_runs Quality=success
  export PR=

  run require_green abc123 Quality

  [[ "${status}" -eq 2 ]]
  [[ "${output}" == *"PR is set by write-acceptance-trailer.sh, and it is empty here."* ]]
}

@test "with no base it exits 2, because no token can be matched to it" {
  check_runs Quality=success
  export BASE_REF=

  run require_green abc123 Quality

  [[ "${status}" -eq 2 ]]
  [[ "${output}" == *"BASE_REF is set by the workflow env, and it is empty here."* ]]
}

@test "a gate is a job of the workflow file named for it, up to its first dash" {
  gh_answers check-runs '{"check_runs":[{"id":9,"name":"Quality-macOS","conclusion":"success",
    "completed_at":"2026-01-01T00:00:00Z","app":{"slug":"github-actions"},
    "check_suite":{"id":5}}]}'
  gh_answers runs '{"workflow_runs":[{"id":7,"check_suite_id":5,
    "path":".github/workflows/quality.yml","event":"pull_request","head_branch":"work"}]}'
  gh_answers jobs '{"jobs":[{"id":9,"run_id":7,"run_attempt":1,"name":"Quality-macOS",
    "conclusion":"success","completed_at":"2026-01-01T00:00:00Z","runner_id":1}]}'
  gate_token_for 7 9

  run require_green abc123 Quality-macOS

  [[ "${status}" -eq 0 ]]
}

@test "the workflow runs are read for this head, its pull request event and its branch" {
  check_runs Quality=success

  run verdicts abc123 Quality

  [[ "${status}" -eq 0 ]]
  asked=$(calls gh)
  read_runs="api repos/owner/name/actions/runs?head_sha=abc123&event=pull_request&branch=work"
  [[ "${asked}" == *"${read_runs}&per_page=100"* ]]
}

@test "a read of the workflow runs the API refuses exits 2" {
  check_runs Quality=success
  gh_refuses runs

  run require_green abc123 Quality

  [[ "${status}" -eq 2 ]]
  [[ "${output}" == *"the workflow runs on abc123 could not be read."* ]]
}

@test "with no branch it exits 2, because no run can be matched to it" {
  check_runs Quality=success
  export HEAD_REF=

  run require_green abc123 Quality

  [[ "${status}" -eq 2 ]]
  [[ "${output}" == *"HEAD_REF is set by the workflow env, and it is empty here."* ]]
}

@test "a check run is posted with the token of purba-sign" {
  fake gh <<'FAKE'
printf '%s\n' "${GH_TOKEN}" >>"${BATS_TEST_TMPDIR}/token"
echo https://example.invalid/run
FAKE

  run post_check_run Sign-off abc123 success title summary

  [[ "${status}" -eq 0 ]]
  token=$(<"${BATS_TEST_TMPDIR}/token")
  [[ "${token}" == sign-token ]]
}

@test "with no token of purba-sign it posts nothing and exits 2" {
  export SIGN_TOKEN=

  run post_check_run Sign-off abc123 success title summary

  [[ "${status}" -eq 2 ]]
  [[ "${output}" == *"SIGN_TOKEN is set by the workflow env, and it is empty here."* ]]
  asked=$(calls gh)
  [[ -z "${asked}" ]]
}

@test "a context with no run at all is refused" {
  check_runs Build=success

  run require_green abc123 Quality

  [[ "${status}" -eq 1 ]]
  [[ "${output}" == *"Quality has no verdict on abc123 yet"* ]]
}

@test "a run still going is no verdict" {
  check_runs Quality=

  run require_green abc123 Quality

  [[ "${status}" -eq 1 ]]
  [[ "${output}" == *"Quality has no verdict on abc123 yet"* ]]
}

@test "a context that concluded anything but success is refused and named" {
  for conclusion in failure neutral cancelled skipped timed_out action_required stale \
    startup_failure; do
    check_runs "Quality=${conclusion}"

    run require_green abc123 Quality

    [[ "${status}" -eq 1 ]]
    [[ "${output}" == *"Quality concluded ${conclusion} on abc123"* ]]
  done
}

@test "the run that finished last decides, whatever order the API returns" {
  check_runs Quality=failure@2026-01-02 Quality=success@2026-01-01

  run require_green abc123 Quality

  [[ "${status}" -eq 1 ]]
  [[ "${output}" == *"Quality concluded failure on abc123"* ]]
}

@test "a rerun still going does not hide the verdict before it" {
  check_runs Quality=success Quality=

  run require_green abc123 Quality

  [[ "${status}" -eq 0 ]]
}

@test "every context is read, and the first one that is not green stops it" {
  check_runs Quality=success Build=failure

  run require_green abc123 Quality Build Docs

  [[ "${status}" -eq 1 ]]
  [[ "${output}" == *"Quality is green on abc123"* ]]
  [[ "${output}" == *"Build concluded failure on abc123"* ]]
  [[ "${output}" != *"Docs"* ]]
}

@test "with no context named it exits 2, because no list is green by being empty" {
  check_runs Quality=success

  run require_green abc123

  [[ "${status}" -eq 2 ]]
  [[ "${output}" == *"usage: require_green <head> <context>..."* ]]
  [[ "${output}" != *"green on abc123"* ]]
}

@test "each context gets its conclusion, or none, in the order asked" {
  check_runs Quality=success Build=failure Scripts=

  run verdicts abc123 Docs Build Quality Scripts

  [[ "${status}" -eq 0 ]]
  [[ "${output}" == "Docs none"$'\n'"Build failure"$'\n'"Quality success"$'\n'"Scripts none" ]]
}

@test "three contexts read the check runs once" {
  check_runs Quality=success Build=success Scripts=success

  run verdicts abc123 Quality Build Scripts

  [[ "${status}" -eq 0 ]]
  asked=$(calls gh)
  read=$(grep -c 'check-runs?per_page' <<<"${asked}")
  [[ "${read}" -eq 1 ]]
}

@test "with an empty repository name it stops before it calls the API" {
  export GH_REPO=

  run bash -c '. "$1"; echo sourced' _ "${module}"

  [[ "${status}" -eq 2 ]]
  [[ "${output}" == *"set by the workflow env"* ]]
  [[ "${output}" != *"sourced"* ]]
  asked=$(calls gh)
  [[ -z "${asked}" ]]
}

@test "a read the API refuses exits 2, whichever function reads" {
  fake gh <<<'echo "gh: not logged in" >&2; exit 4'

  for reader in require_green verdicts; do
    run "${reader}" abc123 Quality

    [[ "${status}" -eq 2 ]]
    [[ "${output}" == *"the check runs on abc123 could not be read."* ]]
  done
}

@test "an answer that holds no check runs exits 2" {
  fake gh <<<'echo "{}"'

  run require_green abc123 Quality

  [[ "${status}" -eq 2 ]]
  [[ "${output}" == *"the check runs on abc123 could not be read."* ]]
}

@test "an answer whose check runs are not objects exits 2" {
  fake gh <<<'echo "{\"check_runs\":[1]}"'

  run require_green abc123 Quality

  [[ "${status}" -eq 2 ]]
  [[ "${output}" == *"the check runs on abc123 could not be read."* ]]
}

# Pads the answer to <call> past 131071 bytes, the most one argument may hold on
# Linux with pages of 4 KiB, with a field no rule reads. <filter> says where the
# field goes. The padding goes through a file, because an argument that long is
# the very thing under test.
#
#   pad_answer <call> [filter]
pad_answer() {
  local answer="${BATS_TEST_TMPDIR}/gh/$1.json" padding="${BATS_TEST_TMPDIR}/padding" pad=x
  # shellcheck disable=SC2016 # jq expands $pad, not the shell
  local filter=${2:-'. + {pad: $pad}'}
  while ((${#pad} < 140000)); do pad+=${pad}; done
  printf '%s' "${pad}" >"${padding}"
  jq -c --rawfile pad "${padding}" "${filter}" "${answer}" >"${answer}.padded"
  mv "${answer}.padded" "${answer}"
}

@test "check runs larger than one argument may hold still give the verdict" {
  check_runs Quality=success
  pad_answer check-runs

  run require_green abc123 Quality

  [[ "${status}" -eq 0 ]]
  [[ "${output}" == *"Quality is green on abc123"* ]]
}

@test "workflow runs larger than one argument may hold still give the verdict" {
  check_runs Quality=success
  pad_answer runs

  run require_green abc123 Quality

  [[ "${status}" -eq 0 ]]
  [[ "${output}" == *"Quality is green on abc123"* ]]
}

@test "the jobs of a run larger than one argument may hold still give the verdict" {
  check_runs Quality=success
  # Inside a job, so the list of every job grows past the limit as well.
  # shellcheck disable=SC2016 # jq expands $pad, not the shell
  pad_answer jobs '.jobs[0] += {pad: $pad}'

  run require_green abc123 Quality

  [[ "${status}" -eq 0 ]]
  [[ "${output}" == *"Quality is green on abc123"* ]]
}
