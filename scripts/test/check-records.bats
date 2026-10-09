# scripts/check-records.sh against the records it refuses.
: "${BATS_TEST_DIRNAME:?set by bats}"

setup() {
  # shellcheck source=scripts/test/fixture.sh
  . "${BATS_TEST_DIRNAME}/fixture.sh"
  script="${BATS_TEST_DIRNAME}/../check-records.sh"
  make_repo
  mkdir -p docs/decisions
  record=docs/decisions/a-record-holds.md
  borrowed="borrowed from Simplified Technical English"
  unwired="a Confirmation that admits an unwired gate and names no issue"
  unmatched="a pattern in a reach that matches no tracked file"
  reach="**Reach:** \`docs/decisions/*.md\`"
  compliant >"${record}"
  git add "${record}"
}

# A record no rule refuses. Each test changes one thing in it.
compliant() {
  cat <<'RECORD'
# A record holds

## Context and Problem Statement

Something needs a decision.

## Considered Options

- **This.** Chosen.

## Decision Outcome

**Reach:** `docs/decisions/*.md`

**This is decided.**

**Downside:**

- **It costs something.** The cost is named.

## Confirmation

`mise run lint:records` reads this.
RECORD
}

# Replace one piece of a record, and fail when nothing was replaced. A test of
# what a record may hold would otherwise pass on a record it never changed.
#
# The replacement is quoted, because bash reads a bare `&` in one as the match.
swap_in() {
  local body
  body=$(<"$1")
  [[ "${body}" == *"$2"* ]]
  printf '%s\n' "${body/"$2"/"$3"}" >"$1"
}

swap() {
  swap_in "${record}" "$@"
}

# The one sentence of prose the compliant record holds.
prose() {
  swap "Something needs a decision." "$1"
}

# One word short of <count>, in `filler`, so a test adds the last one itself.
words() {
  printf -v filler '%*s' "$(($1 - 1))" ''
  filler=${filler// /word }
}

@test "a record that breaks no rule passes" {
  run "${script}"

  [[ "${status}" -eq 0 ]]
}

@test "the directory is an argument" {
  mkdir elsewhere
  swap "## Considered Options" "## Options"
  mv "${record}" elsewhere/a-record-holds.md

  run "${script}" elsewhere

  [[ "${status}" -eq 1 ]]
  [[ "${output}" == *$'the confirmation.\n  elsewhere/a-record-holds.md'* ]]
}

@test "a record missing a section is refused and named" {
  swap $'## Considered Options\n' ""

  run "${script}"

  [[ "${status}" -eq 1 ]]
  [[ "${output}" == *"A record carries four sections, in order"* ]]
  [[ "${output}" == *"a public Markdown convention cut to its minimum"* ]]
  [[ "${output}" == *$'the confirmation.\n  '"${record}"* ]]
}

@test "sections out of order are refused" {
  swap "## Considered Options" "## Decision Outcome"
  swap $'**This.** Chosen.\n\n## Decision Outcome' $'**This.** Chosen.\n\n## Considered Options'

  run "${script}"

  [[ "${status}" -eq 1 ]]
  [[ "${output}" == *"A record carries four sections, in order"* ]]
}

@test "Consequences between the outcome and the confirmation passes" {
  swap "## Confirmation" $'## Consequences\n\nIt follows.\n\n## Confirmation'

  run "${script}"

  [[ "${status}" -eq 0 ]]
}

@test "Consequences anywhere else is refused" {
  swap "## Considered Options" $'## Consequences\n\nIt follows.\n\n## Considered Options'

  run "${script}"

  [[ "${status}" -eq 1 ]]
  [[ "${output}" == *"A record carries four sections, in order"* ]]
}

@test "a comment addressed to a reviewer is refused" {
  prose "<!-- check this -->"

  run "${script}"

  [[ "${status}" -eq 1 ]]
  [[ "${output}" == *"it carries no comment addressed to a reviewer"* ]]
  [[ "${output}" == *"${record}:5:"* ]]
}

@test "the template is not read, because a leading dot keeps it out" {
  printf '<!-- how to write a record, with #45 in it -->\n' >docs/decisions/.template.md

  run "${script}"

  [[ "${status}" -eq 0 ]]
}

@test "a bare issue number is refused" {
  prose "Something needs a decision, as #45 said."

  run "${script}"

  [[ "${status}" -eq 1 ]]
  [[ "${output}" == *"record prose carries no issue number"* ]]
  [[ "${output}" == *"${record}:5:"* ]]
}

@test "an em-dash in prose is refused" {
  prose "Something needs a decision — soon."

  run "${script}"

  [[ "${status}" -eq 1 ]]
  [[ "${output}" == *"A record carries no em-dash"* ]]
  [[ "${output}" == *"because an em-dash joins two ideas in one sentence"* ]]
  [[ "${output}" == *"${record}:5"* ]]
}

@test "bold inside a sentence is refused" {
  prose "Something **really** needs a decision."

  run "${script}"

  [[ "${status}" -eq 1 ]]
  [[ "${output}" == *"Bold opens a sentence and never sits inside one"* ]]
  [[ "${output}" == *"${record}:5"* ]]
}

@test "a sentence wrapped across two lines is refused where it breaks" {
  prose $'The gate runs\non each record.'

  run "${script}"

  [[ "${status}" -eq 1 ]]
  [[ "${output}" == *"A sentence stays on one line."* ]]
  [[ "${output}" == *"${record}:5"* ]]
}

@test "a sentence of twenty-six words is refused" {
  words 26
  prose "${filler}end."

  run "${script}"

  [[ "${status}" -eq 1 ]]
  [[ "${output}" == *"A sentence in descriptive text runs to 25 words, a limit ${borrowed}"* ]]
  [[ "${output}" == *"${record}:5: 26 words"* ]]
}

@test "a seventh sentence in a paragraph is refused once" {
  prose $'One.\nTwo.\nThree.\nFour.\nFive.\nSix.\nSeven.\nEight.'

  run "${script}"

  [[ "${status}" -eq 1 ]]
  [[ "${output}" == *"A paragraph runs to six sentences, a limit ${borrowed}"* ]]
  [[ "${output}" == *"${record}:11: 7 sentences"* ]]
  [[ "${output}" != *"${record}:12: 8 sentences"* ]]
}

@test "a gerund after a form of be is refused" {
  prose "The gate is running the checks."

  run "${script}"

  [[ "${status}" -eq 1 ]]
  [[ "${output}" == *"is a technical noun here and never a verb, a rule ${borrowed}"* ]]
  [[ "${output}" == *"${record}:5: running"* ]]
}

@test "a perfect tense is refused" {
  prose "The gate has refused a record."

  run "${script}"

  [[ "${status}" -eq 1 ]]
  [[ "${output}" == *"A record uses the simple tenses, a rule ${borrowed}"* ]]
  [[ "${output}" == *"${record}:5: has refused"* ]]
}

@test "a participle in the participle list is a tense" {
  prose "It has grown one workflow at a time."

  run "${script}"

  [[ "${status}" -eq 1 ]]
  [[ "${output}" == *"${record}:5: has grown"* ]]
}

@test "a Downside label followed by a sentence is refused" {
  swap "**Downside:**" "**Downside:** It costs."

  run "${script}"

  [[ "${status}" -eq 1 ]]
  [[ "${output}" == *"A Downside label stands on its own line"* ]]
  [[ "${output}" != *"A Downside label never counts its costs"* ]]
  [[ "${output}" == *"${record}:17"* ]]
}

@test "a Downside label that counts its costs is refused for both" {
  swap "**Downside:**" "**Downside:** Three costs."

  run "${script}"

  [[ "${status}" -eq 1 ]]
  [[ "${output}" == *"A Downside label stands on its own line"* ]]
  [[ "${output}" == *"A Downside label never counts its costs"* ]]
}

@test "a record with no Downside label is refused" {
  swap $'**Downside:**\n\n' ""

  run "${script}"

  [[ "${status}" -eq 1 ]]
  [[ "${output}" == *"A record states its costs as a list under a Downside label"* ]]
  [[ "${output}" == *"label, because a decision without its cost is advocacy, not a record"* ]]
  [[ "${output}" == *"${record}: no Downside label"* ]]
}

@test "a Downside label named inside a table is not the label" {
  swap $'**Downside:**\n\n' $'| the label |\n|---|\n| **Downside:** |\n\n'

  run "${script}"

  [[ "${status}" -eq 1 ]]
  [[ "${output}" == *"${record}: no Downside label"* ]]
}

@test "a Downside with no list is refused" {
  swap "- **It costs something.** The cost is named." "It costs something."

  run "${script}"

  [[ "${status}" -eq 1 ]]
  [[ "${output}" == *"A record states its costs as a list under a Downside label"* ]]
  [[ "${output}" == *"${record}:17"* ]]
}

@test "a cost with no bold lead-in is refused" {
  swap "- **It costs something.** The cost is named." "- It costs something."

  run "${script}"

  [[ "${status}" -eq 1 ]]
  [[ "${output}" == *"Bold marks every Downside lead-in"* ]]
  [[ "${output}" == *"${record}:17: 0 lead-ins over 1 costs"* ]]
}

@test "two costs on one line are refused" {
  swap "The cost is named." "The cost is named. - **A second cost.** It does not render."

  run "${script}"

  [[ "${status}" -eq 1 ]]
  [[ "${output}" == *"${record}:17: 2 lead-ins over 1 costs"* ]]
}

@test "a numbered citation is refused in a record and passes outside one" {
  citing=docs/decisions/a-second-record-holds.md
  compliant >"${citing}"
  swap_in "${citing}" "Something needs a decision." "Something needs a decision, as ADR-0019 says."
  printf 'oxitest recorded this as ADR-0019.\n' >NOTES.md
  git add NOTES.md

  run "${script}"

  [[ "${status}" -eq 1 ]]
  [[ "${output}" == *"so a record cites another by its proposition"* ]]
  [[ "${output}" == *"${citing}:5:"* ]]
  [[ "${output}" != *"NOTES.md"* ]]
}

@test "a Confirmation that admits an unwired gate and names no issue is reported, not refused" {
  swap "\`mise run lint:records\` reads this." "The gate is not wired."

  run "${script}"

  [[ "${status}" -eq 0 ]]
  [[ "${output}" == *"${unwired}"$'\n\n  '"${record}"* ]]
}

@test "a Confirmation whose table answers no is reported the same way" {
  swap "\`mise run lint:records\` reads this." $'| rule | checked |\n|---|---|\n| one | no |'

  run "${script}"

  [[ "${status}" -eq 0 ]]
  [[ "${output}" == *"${unwired}"$'\n\n  '"${record}"* ]]
}

@test "on a runner an unwired gate writes no annotation" {
  swap "\`mise run lint:records\` reads this." "The gate is not wired."

  GITHUB_ACTIONS=true run "${script}"

  [[ "${status}" -eq 0 ]]
  [[ "${output}" != *"::error::"* ]]
  [[ "${output}" == *"${unwired}"* ]]
}

@test "a Confirmation that names the issue passes" {
  swap "\`mise run lint:records\` reads this." \
    "The gate is not wired. [Wire it](https://github.com/o/purba/issues/9) owns it."

  run "${script}"

  [[ "${status}" -eq 0 ]]
  [[ "${output}" != *"${unwired}"* ]]
}

@test "every rule a record breaks is reported in one run" {
  prose "Something — as #45 said — **really** needs it."

  run "${script}"

  [[ "${status}" -eq 1 ]]
  [[ "${output}" == *"record prose carries no issue number"* ]]
  [[ "${output}" == *"A record carries no em-dash"* ]]
  [[ "${output}" == *"Bold opens a sentence and never sits inside one"* ]]
}

@test "on a runner each rule a record breaks is one annotation" {
  prose "Something — as #45 said — needs it."

  GITHUB_ACTIONS=true run "${script}"

  [[ "${status}" -eq 1 ]]
  annotations=$(grep -c '^::error::' <<<"${output}" || true)
  [[ "${annotations}" -eq 2 ]]
  [[ "${output}" == *"::error::A record states what is true"*"%0A  ${record}:5:"* ]]
  [[ "${output}" == *"::error::A record carries no em-dash"*"%0A  ${record}:5"* ]]
}

@test "the passive voice and a formal word are reported and never refused" {
  prose "The gate is refused by nothing. We utilize it."

  run "${script}"

  [[ "${status}" -eq 0 ]]
  [[ "${output}" == *"the borrowed rules a command reports and never refuses"* ]]
  [[ "${output}" == *"in the passive voice             3 of     9 sentences  33%"* ]]
  [[ "${output}" == *"the formal words found"*"utilize"* ]]
}

@test "evidence density is reported for each record and for all of them" {
  run "${script}"

  [[ "${status}" -eq 0 ]]
  [[ "${output}" == *" 75.0%    3 of   4  ${record}"* ]]
  [[ "${output}" == *"all records  75.0%  3 of 4"* ]]
}

@test "a record with no prose line is refused, and the report does not abort" {
  printf '# Nothing\n' >"${record}"

  run "${script}"

  [[ "${status}" -eq 1 ]]
  [[ "${output}" == *"all records  no prose line to read"* ]]
  [[ "${output}" != *"division by 0"* ]]
}

@test "a directory holding no record exits 2" {
  run "${script}" docs/nowhere

  [[ "${status}" -eq 2 ]]
  [[ "${output}" == *"no record found in docs/nowhere."* ]]
}

@test "outside a git repository it reads the records" {
  swap "${reach}" "${reach} \`nowhere/**\`"
  mkdir -p "${BATS_TEST_TMPDIR}/bare/docs/decisions"
  cp "${record}" "${BATS_TEST_TMPDIR}/bare/docs/decisions/"
  cd "${BATS_TEST_TMPDIR}/bare"

  run "${script}"

  [[ "${status}" -eq 0 ]]
  [[ "${output}" != *"${unmatched}"* ]]
}

@test "each phrase that admits an unwired gate is reported when no issue is named" {
  for admission in "It is run by hand." "The check is not written yet." "It does not exist yet."; do
    compliant >"${record}"
    swap "\`mise run lint:records\` reads this." "${admission}"

    run "${script}"

    [[ "${status}" -eq 0 ]]
    [[ "${output}" == *"${unwired}"$'\n\n  '"${record}"* ]]
  done
}

@test "an admission outside the Confirmation needs no issue" {
  prose "The gate is not wired."

  run "${script}"

  [[ "${status}" -eq 0 ]]
  [[ "${output}" != *"${unwired}"* ]]
}

@test "a Downside label that counts its costs in digits is refused" {
  swap "**Downside:**" "**Downside:** 3 costs."

  run "${script}"

  [[ "${status}" -eq 1 ]]
  [[ "${output}" == *"A Downside label never counts its costs"* ]]
}

@test "spaces after the Downside label are not a preamble" {
  swap "**Downside:**" "**Downside:**  "

  run "${script}"

  [[ "${status}" -eq 0 ]]
}

@test "a list under a later heading is not a Downside cost" {
  swap "\`mise run lint:records\` reads this." "- \`mise run lint:records\` reads this."

  run "${script}"

  [[ "${status}" -eq 0 ]]
}

@test "a nested cost is a cost, and a warning sign may precede its lead-in" {
  swap "The cost is named." $'The cost is named.\n  - ⚠️ **A nested cost.** It counts.'

  run "${script}"

  [[ "${status}" -eq 0 ]]
}

@test "evidence density lists the barest record first" {
  second=docs/decisions/b-second-record.md
  compliant >"${second}"
  swap_in "${second}" "Something needs a decision." "\`This\` needs a decision."
  swap_in "${second}" "**This is decided.**" "\`This\` is decided."

  run "${script}"

  [[ "${status}" -eq 0 ]]
  [[ "${output}" == *" 75.0%    3 of   4  ${record}"*" 25.0%    1 of   4  ${second}"* ]]
}

@test "a fifth section after the Confirmation is refused" {
  printf '\n## Notes\n\nOne more thing.\n' >>"${record}"

  run "${script}"

  [[ "${status}" -eq 1 ]]
  [[ "${output}" == *$'the confirmation.\n  '"${record}"* ]]
}

@test "an issue number with no space before it is refused" {
  prose "Something needs a decision(#45)."

  run "${script}"

  [[ "${status}" -eq 1 ]]
  [[ "${output}" == *"record prose carries no issue number"* ]]
}

@test "a Downside label that counts its costs in a word is refused" {
  swap "**Downside:**" "**Downside:** two costs."

  run "${script}"

  [[ "${status}" -eq 1 ]]
  [[ "${output}" == *"A Downside label never counts its costs"* ]]
}

@test "an admission is reported unless it links an issue of this repository" {
  swap "\`mise run lint:records\` reads this." \
    "The gate is not wired. [Wire it](https://github.com/o/other/issues/9) owns it."

  run "${script}"

  [[ "${status}" -eq 0 ]]
  [[ "${output}" == *"${unwired}"$'\n\n  '"${record}"* ]]
}

@test "the report counts the articles and the words it read" {
  run "${script}"

  [[ "${status}" -eq 0 ]]
  [[ "${output}" == *"articles                         2 in    20 words"* ]]
  [[ "${output}" == *"formal words                     0 in    20 words"* ]]
}

@test "a Decision Outcome that does not open with a reach is refused" {
  swap "${reach}"$'\n\n' ""

  run "${script}"

  [[ "${status}" -eq 1 ]]
  [[ "${output}" == *"A Decision Outcome opens with a reach"* ]]
  [[ "${output}" == *"${record}:13"* ]]
}

@test "a reach below the first line of the Decision Outcome is refused" {
  swap "${reach}"$'\n\n**This is decided.**' $'**This is decided.**\n\n'"${reach}"

  run "${script}"

  [[ "${status}" -eq 1 ]]
  [[ "${output}" == *"A Decision Outcome opens with a reach"* ]]
  [[ "${output}" == *"${record}:13"* ]]
}

@test "a pattern that matches no tracked file is listed, and refused nowhere" {
  swap "${reach}" "${reach} \`nowhere/**\`"
  mkdir nowhere
  printf 'untracked\n' >nowhere/file

  run "${script}"

  [[ "${status}" -eq 0 ]]
  [[ "${output}" == *"${unmatched}"$'\n\n  '"${record}: nowhere/**"* ]]
  [[ "${output}" != *": docs/decisions/*.md"* ]]
}

@test "a pattern is matched as gitattributes matches it, not as a pathspec" {
  swap "${reach}" "**Reach:** \`docs/*.md\` \`a-record-holds.md\`"

  run "${script}"

  [[ "${status}" -eq 0 ]]
  [[ "${output}" == *"${unmatched}"$'\n\n  '"${record}: docs/*.md"* ]]
  [[ "${output}" != *": a-record-holds.md"* ]]
}

@test "a glossary word in a reach is not a pattern" {
  swap "${reach}" "**Reach:** [commit message](../../CONTEXT.md#commit-message)"

  run "${script}"

  [[ "${status}" -eq 0 ]]
  [[ "${output}" != *"${unmatched}"* ]]
}

@test "on a runner a pattern that matches nothing writes no annotation" {
  swap "${reach}" "${reach} \`nowhere/**\`"

  GITHUB_ACTIONS=true run "${script}"

  [[ "${status}" -eq 0 ]]
  [[ "${output}" != *"::error::"* ]]
  [[ "${output}" == *"${unmatched}"* ]]
}

@test "a reach that names no location is refused" {
  swap "${reach}" "**Reach:** each location where a change makes this record apply"

  run "${script}"

  [[ "${status}" -eq 1 ]]
  [[ "${output}" == *"its glossary word, linked to its CONTEXT.md entry"* ]]
  [[ "${output}" == *"A Decision Outcome opens with a reach"* ]]
  [[ "${output}" == *"${record}:13"* ]]
}

@test "a reach is not prose, so its length and its words are not counted" {
  printf -v many "\`p%d/**\` " {1..30}
  swap "${reach}" "${reach} ${many% }"

  run "${script}"

  [[ "${status}" -eq 0 ]]
  [[ "${output}" != *"runs to 25 words"* ]]
  [[ "${output}" == *"articles                         2 in    20 words"* ]]
}

@test "a reach that wraps is read whole, so a pattern on its second line is listed" {
  swap "${reach}" "${reach}"$'\n'"\`nowhere/**\`"

  run "${script}"

  [[ "${status}" -eq 0 ]]
  [[ "${output}" == *"${unmatched}"$'\n\n  '"${record}: nowhere/**"* ]]
}

@test "a reach quoted before the Decision Outcome is not the reach" {
  prose "Something needs a decision."$'\n\n```\n'"**Reach:** \`nowhere/**\`"$'\n```'

  run "${script}"

  [[ "${status}" -eq 0 ]]
  [[ "${output}" != *"nowhere/**"* ]]
}

@test "a paragraph that opens with Reach outside the Decision Outcome is prose" {
  words 26
  prose "**Reach:** ${filler}end."

  run "${script}"

  [[ "${status}" -eq 1 ]]
  [[ "${output}" == *"runs to 25 words"* ]]
}

@test "a reach that holds prose beside its locations is refused" {
  swap "${reach}" "${reach}"$'\n'"It has never read the records."

  run "${script}"

  [[ "${status}" -eq 1 ]]
  [[ "${output}" == *"A Decision Outcome opens with a reach"*"and holds nothing else"* ]]
  [[ "${output}" == *"${record}:13"* ]]
}

@test "a reach of words and patterns across two lines passes" {
  swap "${reach}" "${reach} [commit message](../../CONTEXT.md#commit-message)"$'\n'"\`chorestart\`"

  run "${script}"

  [[ "${status}" -eq 0 ]]
}

@test "a tracked path that ends in a newline is matched like any other" {
  swap "${reach}" "${reach} \`t/**\`"
  mkdir t
  printf 'file\n' >"t/a"$'\n'
  git add --all

  run "${script}"

  [[ "${status}" -eq 0 ]]
  [[ "${output}" != *"${unmatched}"* ]]
}
