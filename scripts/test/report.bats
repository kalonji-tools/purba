# scripts/report.sh against a record named in a refusal.
: "${BATS_TEST_DIRNAME:?set by bats}"

setup() {
  # shellcheck source=scripts/test/fixture.sh
  . "${BATS_TEST_DIRNAME}/fixture.sh"
  isolate
  # shellcheck source=scripts/report.sh
  . "${BATS_TEST_DIRNAME}/../report.sh"
}

@test "a summary that names a record stops the check" {
  run report "A rule holds, as docs/decisions/a-rule.md says." "  scripts/a-check.sh"

  [[ "${status}" -eq 2 ]]
  [[ "${output}" == *"a refusal names no record"* ]]
}

@test "a summary that names the template of a record stops the check" {
  run report "A rule holds, as docs/decisions/.template.md says." "  scripts/a-check.sh"

  [[ "${status}" -eq 2 ]]
  [[ "${output}" == *"a refusal names no record"* ]]
}

@test "a summary that names the directory of the records is written" {
  run report "A record outside docs/decisions/ is refused." "  notes/a-rule.md"

  [[ "${status}" -eq 0 ]]
  [[ "${output}" == *"A record outside docs/decisions/ is refused."* ]]
}

@test "a detail that names a record is written" {
  run report "A rule holds, because it has a reason." "  docs/decisions/a-rule.md:3"

  [[ "${status}" -eq 0 ]]
  [[ "${output}" == *"docs/decisions/a-rule.md:3"* ]]
}
