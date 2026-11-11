# scripts/check-macos-jobs.sh against the workflows it counts.
: "${BATS_TEST_DIRNAME:?set by bats}"
bats_require_minimum_version 1.5.0

setup() {
  # shellcheck source=scripts/test/fixture.sh
  . "${BATS_TEST_DIRNAME}/fixture.sh"
  script="${BATS_TEST_DIRNAME}/../check-macos-jobs.sh"
  isolate
  file="${BATS_TEST_TMPDIR}/ci.yml"
}

# Writes a workflow of <n> jobs that each name macOS, then the jobs on stdin.
workflow() {
  local n=$1 i
  {
    printf 'name: CI\non: pull_request\njobs:\n'
    for ((i = 1; i <= n; i++)); do
      printf '  plain-%s:\n    runs-on: macos-latest\n    steps: [{run: "true"}]\n' "${i}"
    done
    cat
  } >"${file}"
}

@test "five macOS jobs pass" {
  workflow 3 <<'JOBS'
  linux:
    runs-on: ubuntu-latest
    steps: [{run: "true"}]
  each:
    strategy:
      matrix:
        os: [ubuntu-latest, macos-latest]
    runs-on: ${{ matrix.os }}
    steps: [{run: "true"}]
  last:
    runs-on: [self-hosted, macos]
    steps: [{run: "true"}]
JOBS

  run "${script}" "${file}"

  [[ "${status}" -eq 0 ]]
  [[ "${output}" == *"5 macOS jobs"* ]]
}

@test "a sixth macOS job is refused, and each job is named with its count" {
  workflow 5 <<'JOBS'
  sixth:
    runs-on: macos-latest
    steps: [{run: "true"}]
JOBS

  run "${script}" "${file}"

  [[ "${status}" -eq 1 ]]
  [[ "${output}" == *"6 macOS jobs"* ]]
  [[ "${output}" == *$'\n  sixth: 1'* ]]
  [[ "${output}" == *$'\n  plain-1: 1'* ]]
  summary="A pull request is refused when its workflow asks for more than 5 macOS jobs on"
  summary+=" one push, because the Free plan runs 5 at once, and a sixth waits for a runner"
  summary+=" to come free."
  [[ "${output}" == *"${summary}"* ]]
}

@test "a matrix that crosses macOS with three Pythons counts three" {
  workflow 3 <<'JOBS'
  crossed:
    strategy:
      matrix:
        os: [macos-latest, ubuntu-latest]
        python: ["3.12", "3.13", "3.14"]
    runs-on: ${{ matrix.os }}
    steps: [{run: "true"}]
JOBS

  run "${script}" "${file}"

  [[ "${status}" -eq 1 ]]
  [[ "${output}" == *$'\n  crossed: 3'* ]]
}

@test "an exclude drops the row it matches" {
  workflow 3 <<'JOBS'
  crossed:
    strategy:
      matrix:
        os: [macos-latest, ubuntu-latest]
        python: ["3.12", "3.13", "3.14"]
        exclude:
          - {os: macos-latest, python: "3.12"}
    runs-on: ${{ matrix.os }}
    steps: [{run: "true"}]
JOBS

  run "${script}" "${file}"

  [[ "${status}" -eq 0 ]]
}

@test "an include that matches no row adds a row" {
  workflow 5 <<'JOBS'
  added:
    strategy:
      matrix:
        os: [ubuntu-latest]
        include:
          - {os: macos-latest}
    runs-on: ${{ matrix.os }}
    steps: [{run: "true"}]
JOBS

  run "${script}" "${file}"

  [[ "${status}" -eq 1 ]]
  [[ "${output}" == *$'\n  added: 1'* ]]
}

@test "an include that extends a row adds none" {
  workflow 4 <<'JOBS'
  extended:
    strategy:
      matrix:
        os: [macos-latest]
        include:
          - {os: macos-latest, python: "3.14t"}
          - {python: "3.13"}
    runs-on: ${{ matrix.os }}
    steps: [{run: "true"}]
JOBS

  run "${script}" "${file}"

  [[ "${status}" -eq 0 ]]
}

@test "a matrix of include alone counts each entry" {
  workflow 4 <<'JOBS'
  listed:
    strategy:
      matrix:
        include:
          - {os: macos-latest, python: "3.12"}
          - {os: macos-latest, python: "3.13"}
          - {os: ubuntu-latest, python: "3.14"}
    runs-on: ${{ matrix.os }}
    steps: [{run: "true"}]
JOBS

  run "${script}" "${file}"

  [[ "${status}" -eq 1 ]]
  [[ "${output}" == *$'\n  listed: 2'* ]]
}

@test "with no argument it reads ci.yml of the repository" {
  make_repo
  mkdir -p .github/workflows
  workflow 6 </dev/null
  mv "${file}" .github/workflows/ci.yml

  run "${script}"

  [[ "${status}" -eq 1 ]]
  [[ "${output}" == *"6 macOS jobs"* ]]
}

@test "the ci.yml of the tree passes" {
  run "${script}" "${BATS_TEST_DIRNAME}/../../.github/workflows/ci.yml"

  [[ "${status}" -eq 0 ]]
}

@test "a missing workflow exits 2" {
  run "${script}" "${BATS_TEST_TMPDIR}/nowhere.yml"

  [[ "${status}" -eq 2 ]]
  [[ "${output}" == *"nowhere.yml cannot be read"* ]]
}

@test "a workflow that does not parse exits 2" {
  printf 'jobs: [\n' >"${file}"

  run "${script}" "${file}"

  [[ "${status}" -eq 2 ]]
  [[ "${output}" == *"cannot be parsed"* ]]
}

@test "a workflow with no jobs exits 2" {
  printf 'name: CI\n' >"${file}"

  run "${script}" "${file}"

  [[ "${status}" -eq 2 ]]
  [[ "${output}" == *"names no jobs"* ]]
}
