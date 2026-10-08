# scripts/report.sh against a record named in a refusal.
: "${BATS_TEST_DIRNAME:?set by bats}"
bats_require_minimum_version 1.5.0

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

@test "two refusals print with a blank line between them, and each finding indented 2 spaces" {
  twice() {
    refuse "A first rule holds, because it has a reason." a.md b.md
    refuse "A second rule holds, because it has a reason." c.md
  }

  run twice

  [[ "${status}" -eq 0 ]]
  [[ "${output}" == "A first rule holds, because it has a reason."$'\n  a.md\n  b.md\n\n'"A second \
rule holds, because it has a reason."$'\n  c.md' ]]
}

@test "each line of a finding is indented, and an empty line stays empty" {
  run refuse "A rule holds, because it has a reason." $'a.md\n\nb.md'

  [[ "${output}" == "A rule holds, because it has a reason."$'\n  a.md\n\n  b.md' ]]
}

@test "finish exits 1 after a refusal, and 0 with none" {
  refuse_and_finish() {
    refuse "A rule holds, because it has a reason." a.md
    finish
  }

  run finish
  [[ "${status}" -eq 0 ]]

  run refuse_and_finish
  [[ "${status}" -eq 1 ]]
}

@test "cannot exits 2, and appends to PURBA_REPORT" {
  export PURBA_REPORT="${BATS_TEST_TMPDIR}/report"
  printf 'before\n' >"${PURBA_REPORT}"

  run cannot "the check could not read a.md." "a.md: no such file"

  [[ "${status}" -eq 2 ]]
  [[ "${output}" == "the check could not read a.md."$'\n'"a.md: no such file" ]]
  [[ "$(<"${PURBA_REPORT}")" == "before"$'\n'"the check could not read a.md."$'\n\n'"a.md: no \
such file" ]]
}

@test "cannot writes a message that names a record" {
  run cannot "docs/decisions/a-rule.md cannot be read."

  [[ "${status}" -eq 2 ]]
  [[ "${output}" == "docs/decisions/a-rule.md cannot be read." ]]
}

@test "on a runner, a refusal leaves stdout empty" {
  GITHUB_ACTIONS=true run --separate-stderr refuse "A rule holds, because it has a reason." a.md

  [[ -z "${output}" ]]
  [[ "${stderr}" == "::error::A rule holds, because it has a reason.%0A  a.md" ]]
}

@test "on a runner, cannot with no detail is one annotation, and stdout stays empty" {
  GITHUB_ACTIONS=true run --separate-stderr cannot "the check could not run."

  [[ "${status}" -eq 2 ]]
  [[ -z "${output}" ]]
  [[ "${stderr}" == "::error::the check could not run." ]]
}

@test "on a runner, a caller's detail is the same after a refusal" {
  export GITHUB_ACTIONS=true
  detail="the caller's own"

  refuse "A rule holds, because it has a reason." a.md >/dev/null
  report "A rule holds, because it has a reason." "  a.md" >/dev/null

  [[ "${detail}" == "the caller's own" ]]
}
