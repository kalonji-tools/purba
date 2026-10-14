# .github/scripts/post-record-thread.sh: which files it lists as records, and when it posts nothing.
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

@test "a change to the template alone exits 0, and posts no comment" {
  fake gh <<'FAKE'
case "$*" in
  *"/files"*) echo docs/decisions/.template.md ;;
esac
FAKE

  run "${script}"

  [[ "${status}" -eq 0 ]]
  [[ "${output}" == *"no comment is owed"* ]]
  asked=$(calls gh)
  [[ "${asked}" != *"body="* ]]
}

@test "a change to the template and a record posts a comment that lists the record alone" {
  fake gh <<'FAKE'
case "$*" in
  *"/files"*) printf '%s\n' docs/decisions/.template.md docs/decisions/a-record.md ;;
esac
FAKE

  run "${script}"

  [[ "${status}" -eq 0 ]]
  asked=$(calls gh)
  [[ "${asked}" == *"- \`docs/decisions/a-record.md\`"* ]]
  [[ "${asked}" != *".template.md"* ]]
}
