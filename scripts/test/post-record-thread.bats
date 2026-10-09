# .github/scripts/post-record-thread.sh when the files it reads hold no record, or cannot be read.
: "${BATS_TEST_DIRNAME:?set by bats}"

setup() {
  # shellcheck source=scripts/test/fixture.sh
  . "${BATS_TEST_DIRNAME}/fixture.sh"
  script="${BATS_TEST_DIRNAME}/../../.github/scripts/post-record-thread.sh"
  isolate
  export GH_REPO=owner/name PR=7
}

@test "a failed read of the files exits 2, and does not say that no comment is owed" {
  fake gh <<'FAKE'
echo "HTTP 502" >&2
exit 1
FAKE

  run "${script}"

  [[ "${status}" -eq 2 ]]
  [[ "${output}" != *"no comment is owed"* ]]
}

@test "files that hold no record exit 0, and say that no comment is owed" {
  fake gh <<'FAKE'
echo src/lib.rs
FAKE

  run "${script}"

  [[ "${status}" -eq 0 ]]
  [[ "${output}" == *"no comment is owed"* ]]
}
