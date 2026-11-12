# .github/scripts/require-jobs.sh against the needs contexts it reads.
: "${BATS_TEST_DIRNAME:?set by bats}"
bats_require_minimum_version 1.5.0

setup() {
  # shellcheck source=scripts/test/fixture.sh
  . "${BATS_TEST_DIRNAME}/fixture.sh"
  script="${BATS_TEST_DIRNAME}/../../.github/scripts/require-jobs.sh"
  isolate
}

# Sets NEEDS in the shape `toJSON(needs)` writes, with one job for each
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
  summary="A branch is refused unless the job \`changed\` succeeded and every other job"
  summary+=" did what \`changed\` asked. The gate stands for each of those jobs whenever a"
  summary+=" path that starts the gate changes. \`changed\` sets build=true to ask for a"
  summary+=" success, and sets no build to ask for a skip."
  [[ "${output}" == *"${summary}"* ]]
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

@test "every job that skipped passes when changed holds no outputs" {
  NEEDS='{"changed": {"result": "success"}, "rust": {"result": "skipped"}}' \
    run "${script}" changed build

  [[ "${status}" -eq 0 ]]
  [[ "${output}" == *"changed set no build, and every other job skipped"* ]]
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
  [[ "${output}" == *$'\n  changed: success, build=false'* ]]
}

@test "an output set to the boolean false is refused, and named" {
  local needs='{"changed": {"result": "success", "outputs": {"build": false}},'
  needs+=' "rust": {"result": "skipped", "outputs": {}}}'

  NEEDS="${needs}" run "${script}" changed build

  [[ "${status}" -eq 1 ]]
  [[ "${output}" == *$'\n  changed: success, build=false'* ]]
}

@test "an unset NEEDS exits 2" {
  unset NEEDS

  run "${script}" changed build

  [[ "${status}" -eq 2 ]]
  [[ "${output}" == *"NEEDS is unset"* ]]
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

@test "a NEEDS with a job that is not an object exits 2" {
  NEEDS='{"changed": 1, "rust": {"result": "skipped", "outputs": {}}}' run "${script}" changed build

  [[ "${status}" -eq 2 ]]
  [[ "${output}" == *"NEEDS holds a job that is not a JSON object"* ]]
}

@test "a NEEDS with a job whose outputs are not an object exits 2" {
  NEEDS='{"changed": {"result": "success", "outputs": 1}, "rust": {"result": "skipped"}}' \
    run "${script}" changed build

  [[ "${status}" -eq 2 ]]
  [[ "${output}" == *"NEEDS holds a job whose outputs are not a JSON object"* ]]
}

@test "a NEEDS with a job whose outputs are false exits 2" {
  NEEDS='{"changed": {"result": "success", "outputs": false}, "rust": {"result": "skipped"}}' \
    run "${script}" changed build

  [[ "${status}" -eq 2 ]]
  [[ "${output}" == *"NEEDS holds a job whose outputs are not a JSON object"* ]]
}

@test "a NEEDS with a job whose outputs are null exits 2" {
  NEEDS='{"changed": {"result": "success", "outputs": null}, "rust": {"result": "skipped"}}' \
    run "${script}" changed build

  [[ "${status}" -eq 2 ]]
  [[ "${output}" == *"NEEDS holds a job whose outputs are not a JSON object"* ]]
}

@test "a NEEDS without the job that decides exits 2" {
  needs rust=success wheel=success

  run "${script}" changed build

  [[ "${status}" -eq 2 ]]
  [[ "${output}" == *"NEEDS holds no job named \`changed\`"* ]]
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

@test "--success passes when each job it names succeeded" {
  needs quality=success scripts=success changed=success

  run "${script}" --success quality scripts

  [[ "${status}" -eq 0 ]]
  [[ "${output}" == *"quality and scripts succeeded"* ]]
}

@test "--success refuses a job it names that failed, and names each job" {
  needs quality=failure scripts=success

  run "${script}" --success quality scripts

  [[ "${status}" -eq 1 ]]
  [[ "${output}" == *$'\n  quality: failure'* ]]
  [[ "${output}" == *$'\n  scripts: success'* ]]
  summary="A branch is refused unless each job that runs on every push succeeded. A skip or"
  summary+=" a cancel of one is no success, because the job checked nothing."
  [[ "${output}" == *"${summary}"* ]]
}

@test "--success refuses a job it names that skipped" {
  needs quality=success scripts=skipped

  run "${script}" --success quality scripts

  [[ "${status}" -eq 1 ]]
  [[ "${output}" == *$'\n  scripts: skipped'* ]]
}

@test "--success reads no job it does not name" {
  needs quality=success changed=failure rust=skipped

  run "${script}" --success quality

  [[ "${status}" -eq 0 ]]
}

@test "--success with a job that NEEDS does not hold exits 2" {
  needs quality=success

  run "${script}" --success quality lint

  [[ "${status}" -eq 2 ]]
  [[ "${output}" == *"NEEDS holds no job named \`lint\`"* ]]
}

@test "--success with no job exits 2" {
  needs quality=success

  run "${script}" --success

  [[ "${status}" -eq 2 ]]
  [[ "${output}" == *"usage: require-jobs.sh --success <job>..."* ]]
}
