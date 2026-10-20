# .github/scripts/post-record-thread.sh: which files it lists as records, and when it posts nothing.
: "${BATS_TEST_DIRNAME:?set by bats}"

setup() {
  # shellcheck source=scripts/test/fixture.sh
  . "${BATS_TEST_DIRNAME}/fixture.sh"
  script="${BATS_TEST_DIRNAME}/../../.github/scripts/post-record-thread.sh"
  isolate
  export GH_REPO=owner/name PR=7
  fake_gh
  gh_answers comments '[{"body":"a person said something"}]'
}

@test "a failed read of the files exits 2, and does not say that no comment is owed" {
  gh_refuses files

  run "${script}"

  [[ "${status}" -eq 2 ]]
  [[ "${output}" != *"no comment is owed"* ]]
}

@test "files that hold no record exit 0, and say that no comment is owed" {
  gh_answers files '[{"filename":"src/lib.rs"}]'

  run "${script}"

  [[ "${status}" -eq 0 ]]
  [[ "${output}" == *"no comment is owed"* ]]
}

@test "a change to the template alone exits 0, and posts no comment" {
  gh_answers files '[{"filename":"docs/decisions/.template.md"}]'

  run "${script}"

  [[ "${status}" -eq 0 ]]
  [[ "${output}" == *"no comment is owed"* ]]
  asked=$(calls gh)
  [[ "${asked}" != *"body="* ]]
}

@test "a change to the template and a record posts a comment that lists the record alone" {
  gh_answers files '[{"filename":"docs/decisions/.template.md"},
    {"filename":"docs/decisions/a-record.md"}]'

  run "${script}"

  [[ "${status}" -eq 0 ]]
  [[ "${output}" == *"posted the decision-record comment"* ]]
  asked=$(calls gh)
  [[ "${asked}" == *"- \`docs/decisions/a-record.md\`"* ]]
  [[ "${asked}" != *".template.md"* ]]
}

@test "a pull request that already holds the comment gets no second one" {
  gh_answers files '[{"filename":"docs/decisions/a-record.md"}]'
  gh_answers comments '[{"body":"one"},
    {"body":"<!-- purba:thread:decisions -->"}]'

  run "${script}"

  [[ "${status}" -eq 0 ]]
  [[ "${output}" == *"is already present, skipping"* ]]
  asked=$(calls gh)
  [[ "${asked}" != *"body="* ]]
}

@test "a failed read of the comments exits 2, and posts nothing" {
  gh_answers files '[{"filename":"docs/decisions/a-record.md"}]'
  gh_refuses comments

  run "${script}"

  [[ "${status}" -eq 2 ]]
  [[ "${output}" == *"the comments of pull request 7 could not be read"* ]]
  asked=$(calls gh)
  [[ "${asked}" != *"body="* ]]
}
