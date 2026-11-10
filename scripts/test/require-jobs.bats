# .github/scripts/require-jobs.sh against the needs contexts it refuses.
: "${BATS_TEST_DIRNAME:?set by bats}"
bats_require_minimum_version 1.5.0

setup() {
  # shellcheck source=scripts/test/fixture.sh
  . "${BATS_TEST_DIRNAME}/fixture.sh"
  script="${BATS_TEST_DIRNAME}/../../.github/scripts/require-jobs.sh"
  isolate
}

# The needs context in the shape `toJSON(needs)` writes, with one job for each
# <name>=<result>. A job given as <name>=<result>/<value> also carries the output
# `build` set to <value>, which may be empty.
needs() {
  local each name result outputs
  local -a jobs=()
  for each in "$@"; do
    name=${each%%=*} result=${each#*=} outputs="{}"
    if [[ "${result}" == */* ]]; then
      outputs="{\"build\": \"${result#*/}\"}"
      result=${result%%/*}
    fi
    jobs+=("\"${name}\": {\"result\": \"${result}\", \"outputs\": ${outputs}}")
  done
  local IFS=,
  NEEDS="{${jobs[*]}}"
  export NEEDS
}

@test "a job that failed after changed asked for the build is refused, and named" {
  needs changed=success/true rust=failure wheel=success

  run "${script}" changed build

  [[ "${status}" -eq 1 ]]
  [[ "${output}" == *$'\n  rust: failure'* ]]
  [[ "${output}" == *$'\n  wheel: success'* ]]
}

@test "every job that succeeded after changed asked for the build passes" {
  needs changed=success/true rust=success wheel=success

  run "${script}" changed build

  [[ "${status}" -eq 0 ]]
}

@test "a job that skipped after changed asked for the build is refused" {
  needs changed=success/true rust=success wheel=skipped

  run "${script}" changed build

  [[ "${status}" -eq 1 ]]
  [[ "${output}" == *$'\n  wheel: skipped'* ]]
}

@test "every job that skipped after changed set no output passes" {
  needs changed=success rust=skipped wheel=skipped

  run "${script}" changed build

  [[ "${status}" -eq 0 ]]
}

@test "every job that skipped after changed set an empty output passes" {
  needs changed=success/ rust=skipped wheel=skipped

  run "${script}" changed build

  [[ "${status}" -eq 0 ]]
}

@test "a job that ran after changed asked for nothing is refused" {
  needs changed=success rust=success wheel=skipped

  run "${script}" changed build

  [[ "${status}" -eq 1 ]]
  [[ "${output}" == *$'\n  rust: success'* ]]
}

@test "a skip after changed failed is refused" {
  needs changed=failure rust=skipped wheel=skipped

  run "${script}" changed build

  [[ "${status}" -eq 1 ]]
  [[ "${output}" == *$'\n  changed: failure'* ]]
}

@test "a job added to needs joins the rule" {
  needs changed=success/true rust=success wheel=success lint=failure

  run "${script}" changed build

  [[ "${status}" -eq 1 ]]
  [[ "${output}" == *$'\n  lint: failure'* ]]
}

@test "an output other than true is refused, even when every job skipped" {
  needs changed=success/false rust=skipped wheel=skipped

  run "${script}" changed build

  [[ "${status}" -eq 1 ]]
}

@test "an unset NEEDS exits 2" {
  unset NEEDS

  run "${script}" changed build

  [[ "${status}" -eq 2 ]]
  [[ "${output}" == *"NEEDS"* ]]
}

@test "a NEEDS that is not a JSON object exits 2" {
  NEEDS='["changed"]' run "${script}" changed build

  [[ "${status}" -eq 2 ]]
  [[ "${output}" == *"not a JSON object"* ]]
}

@test "a NEEDS that is not JSON exits 2" {
  NEEDS='changed=success' run "${script}" changed build

  [[ "${status}" -eq 2 ]]
  [[ "${output}" == *"not a JSON object"* ]]
}

@test "a NEEDS without the job that decides exits 2" {
  needs rust=success wheel=success

  run "${script}" changed build

  [[ "${status}" -eq 2 ]]
  [[ "${output}" == *"changed"* ]]
}

@test "a NEEDS with no job besides the one that decides exits 2" {
  needs changed=success/true

  run "${script}" changed build

  [[ "${status}" -eq 2 ]]
  [[ "${output}" == *"no job besides changed"* ]]
}

@test "a wrong count of arguments exits 2" {
  needs changed=success/true rust=success

  run "${script}" changed

  [[ "${status}" -eq 2 ]]
  [[ "${output}" == *"usage"* ]]
}
