# scripts/records.sh against the records and the reaches it reads.
: "${BATS_TEST_DIRNAME:?set by bats}"
bats_require_minimum_version 1.5.0

setup() {
  # shellcheck source=scripts/test/fixture.sh
  . "${BATS_TEST_DIRNAME}/fixture.sh"
  make_repo
  # shellcheck source=scripts/report.sh
  . "${BATS_TEST_DIRNAME}/../report.sh"
  # shellcheck source=scripts/records.sh
  . "${BATS_TEST_DIRNAME}/../records.sh"
  scratch="${BATS_TEST_TMPDIR}/scratch"
  mkdir "${scratch}"
}

# A record at <file> whose Context holds <context>, and whose Decision Outcome
# opens with <opening>. With a one-line context, the opening starts on line 9.
record() {
  printf '# A record\n\n## Context and Problem Statement\n\n%s\n\n' "$2" >"$1"
  printf '## Decision Outcome\n\n%s\n\n**It holds.**\n' "$3" >>"$1"
}

# The paths reach_hits reads, one argument to a path.
paths() {
  printf '%s\0' "$@" >"${BATS_TEST_TMPDIR}/paths"
}

# reach_hits for each <record>, into `got`, one line for each hit, its record,
# pattern and path joined by a space.
hits() {
  reach_hits "${scratch}" "${BATS_TEST_TMPDIR}/paths" "$@" >"${BATS_TEST_TMPDIR}/hits"
  tr '\0' '\n' <"${BATS_TEST_TMPDIR}/hits" >"${BATS_TEST_TMPDIR}/lines"
  got=$(paste -d ' ' - - - <"${BATS_TEST_TMPDIR}/lines")
}

@test "a Markdown file directly under docs/decisions/ is a record, and nothing else is" {
  run is_record docs/decisions/a-record.md

  [[ "${status}" -eq 0 ]]
  [[ "${output}" == docs/decisions/a-record.md ]]

  for path in docs/decisions/.template.md docs/decisions/sub/a-record.md \
    docs/decisions/a-record.txt docs/a-record.md elsewhere/docs/decisions/a-record.md; do
    run is_record "${path}"

    [[ "${status}" -eq 0 ]]
    [[ -z "${output}" ]]
  done
}

@test "the reach is the first paragraph of the Decision Outcome, read whole when it wraps" {
  record r.md "Something." $'**Reach:** `a/**`\n`b/**`'

  run reach_lines r.md

  [[ "${status}" -eq 0 ]]
  [[ "${output}" == "9 10 reach" ]]
}

@test "a Reach paragraph that holds prose beside its locations is not a reach" {
  record r.md "Something." $'**Reach:** `a/**`\nIt has never read the records.'

  run reach_lines r.md

  [[ "${status}" -eq 0 ]]
  [[ "${output}" == "9 10 other" ]]
}

@test "a Reach paragraph that names no location is not a reach" {
  record r.md "Something." "**Reach:** each location where a change makes this record apply"

  run reach_lines r.md

  [[ "${status}" -eq 0 ]]
  [[ "${output}" == "9 9 other" ]]
}

@test "an outcome that opens with a paragraph or a heading has no reach, and gives its first line" {
  record r.md "Something." $'**It holds first.**\n**Reach:** `a/**`'
  record h.md "Something." "### A heading"

  run reach_lines r.md

  [[ "${status}" -eq 0 ]]
  [[ "${output}" == "9 10 other" ]]

  run reach_lines h.md

  [[ "${status}" -eq 0 ]]
  [[ "${output}" == "9 9 other" ]]
}

@test "a record with no Decision Outcome gives nothing" {
  printf "# A record\n\n**Reach:** \`a/**\`\n" >r.md

  run reach_lines r.md

  [[ "${status}" -eq 0 ]]
  [[ -z "${output}" ]]
}

@test "a Reach paragraph before the Decision Outcome is not the reach" {
  record r.md "**Reach:** \`a/**\`" "**It holds first.**"

  run reach_lines r.md

  [[ "${status}" -eq 0 ]]
  [[ "${output}" == "9 9 other" ]]
}

@test "a fenced Decision Outcome line before the heading is not the heading" {
  record r.md $'```markdown\n## Decision Outcome\n```' "**Reach:** \`a/**\`"

  run reach_lines r.md

  [[ "${status}" -eq 0 ]]
  [[ "${output}" == "11 11 reach" ]]
}

@test "a fence closes only on its own character, at least as long as it opened" {
  record r.md $'~~~~\n```\n~~~\n## Decision Outcome\n~~~~' "**Reach:** \`a/**\`"

  run reach_lines r.md

  [[ "${status}" -eq 0 ]]
  [[ "${output}" == "13 13 reach" ]]
}

@test "each code span in a reach is a pattern, in order, and a glossary word is not" {
  record r.md "Something." \
    $'**Reach:** [commit message](../../CONTEXT.md#commit-message) `a/**`\n`b/*.md`'

  run reach_patterns r.md

  [[ "${status}" -eq 0 ]]
  [[ "${output}" == $'a/**\nb/*.md' ]]
}

@test "a paragraph that is not a reach gives no pattern" {
  record r.md "Something." $'**Reach:** `a/**`\nIt has never read the records.'

  run reach_patterns r.md

  [[ "${status}" -eq 0 ]]
  [[ -z "${output}" ]]
}

@test "a reach with ** or a glossary word reaches every pull request, and patterns alone do not" {
  record all.md "Something." "**Reach:** \`a/**\` \`**\`"
  record word.md "Something." "**Reach:** [commit message](../../CONTEXT.md#commit-message)"
  record some.md "Something." "**Reach:** \`a/**\` \`b/*.md\`"

  run reach_is_everywhere all.md
  [[ "${output}" == everywhere ]]
  run reach_is_everywhere word.md
  [[ "${output}" == everywhere ]]
  run reach_is_everywhere some.md
  [[ "${status}" -eq 0 ]]
  [[ -z "${output}" ]]
}

@test "a pattern is matched as gitattributes matches it, not as a pathspec" {
  record r.md "Something." "**Reach:** \`docs/*.md\` \`a-record.md\`"
  paths docs/decisions/a-record.md

  hits r.md

  [[ "${got}" == "r.md a-record.md docs/decisions/a-record.md" ]]
}

@test "a path that ends in a newline is matched like any other" {
  record r.md "Something." "**Reach:** \`t/**\`"
  paths "t/a"$'\n'

  reach_hits "${scratch}" "${BATS_TEST_TMPDIR}/paths" r.md >"${BATS_TEST_TMPDIR}/hits"

  printf 'r.md\0t/**\0t/a\n\0' | cmp - "${BATS_TEST_TMPDIR}/hits"
}

@test "a pattern that two records hold names each of them" {
  record one.md "Something." "**Reach:** \`src/**\`"
  record two.md "Something." "**Reach:** \`src/**\`"
  paths src/a.rs

  hits one.md two.md

  [[ "${got}" == "one.md src/** src/a.rs
two.md src/** src/a.rs" ]]
}
