# .github/scripts/check-run.sh against the verdicts it reads.
: "${BATS_TEST_DIRNAME:?set by bats}"

setup() {
  # shellcheck source=scripts/test/fixture.sh
  . "${BATS_TEST_DIRNAME}/fixture.sh"
  module="${BATS_TEST_DIRNAME}/../../.github/scripts/check-run.sh"
  isolate
  export GH_REPO=owner/name
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
  [[ "${asked}" == "api repos/owner/name/commits/abc123/check-runs?per_page=100" ]]
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
