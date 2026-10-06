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
  compliant >"${record}"
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

**This is decided.**

**Downside:**

- **It costs something.** The cost is named.

## Confirmation

`mise run records` reads this.
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

@test "an em-dash in a table cell is refused" {
  prose $'| a | b |\n|---|---|\n| one — two | three |'

  run "${script}"

  [[ "${status}" -eq 1 ]]
  [[ "${output}" == *"A record carries no em-dash"* ]]
  [[ "${output}" == *"${record}:7"* ]]
}

@test "an em-dash in a code span or a fenced block passes" {
  prose $'It quotes `a — b` here.\n\n```\na — b\n```'

  run "${script}"

  [[ "${status}" -eq 0 ]]
}

@test "bold inside a sentence is refused" {
  prose "Something **really** needs a decision."

  run "${script}"

  [[ "${status}" -eq 1 ]]
  [[ "${output}" == *"Bold opens a sentence and never sits inside one"* ]]
  [[ "${output}" == *"${record}:5"* ]]
}

@test "bold that opens a second sentence passes" {
  prose "Something needs a decision. **This one.** It is taken."

  run "${script}"

  [[ "${status}" -eq 0 ]]
}

@test "two globs on one line are not read as bold" {
  prose "It names .github/** and /scripts/** alike."

  run "${script}"

  [[ "${status}" -eq 0 ]]
}

@test "a sentence of twenty-five words passes and one of twenty-six is refused" {
  words 25
  prose "${filler}end."
  run "${script}"
  [[ "${status}" -eq 0 ]]

  swap "word end." "word word end."
  run "${script}"
  [[ "${status}" -eq 1 ]]
  [[ "${output}" == *"A sentence in descriptive text runs to 25 words, a limit ${borrowed}"* ]]
  [[ "${output}" == *"${record}:5: 26 words"* ]]
}

@test "a link counts as one word, whatever its title holds" {
  words 24
  prose "${filler}[a title that is ten words long all on its own](https://example.invalid) end."

  run "${script}"

  [[ "${status}" -eq 0 ]]
}

@test "a code span counts as one word, and never as none" {
  words 24
  prose "${filler}\`one two three four five\` end."
  run "${script}"
  [[ "${status}" -eq 0 ]]

  swap "word \`one" "word word \`one"
  run "${script}"
  [[ "${status}" -eq 1 ]]
  [[ "${output}" == *"${record}:5: 26 words"* ]]
}

@test "a long sentence in a list item is refused" {
  words 26
  swap "The cost is named." "${filler}end."

  run "${script}"

  [[ "${status}" -eq 1 ]]
  [[ "${output}" == *"${record}:17: "*" words"* ]]
}

@test "a long table cell passes" {
  words 30
  prose $'| a |\n|---|\n| '"${filler}"$'end |'

  run "${script}"

  [[ "${status}" -eq 0 ]]
}

@test "a paragraph of six sentences passes and a seventh is refused once" {
  prose $'One.\nTwo.\nThree.\nFour.\nFive.\nSix.'
  run "${script}"
  [[ "${status}" -eq 0 ]]

  swap $'Six.\n' $'Six.\nSeven.\nEight.\n'
  run "${script}"
  [[ "${status}" -eq 1 ]]
  [[ "${output}" == *"A paragraph runs to six sentences, a limit ${borrowed}"* ]]
  [[ "${output}" == *"${record}:11: 7 sentences"* ]]
  [[ "${output}" != *"${record}:12: 8 sentences"* ]]
}

@test "a blank line splits a paragraph" {
  prose $'One.\nTwo.\nThree.\nFour.\n\nFive.\nSix.\nSeven.\nEight.'

  run "${script}"

  [[ "${status}" -eq 0 ]]
}

@test "seven list items are not a paragraph" {
  swap "- **This.** Chosen." $'- One.\n- Two.\n- Three.\n- Four.\n- Five.\n- Six.\n- Seven.'

  run "${script}"

  [[ "${status}" -eq 0 ]]
}

@test "bold that opens a table cell passes" {
  prose $'| a | b |\n|---|---|\n| one | **two** |'

  run "${script}"

  [[ "${status}" -eq 0 ]]
}

@test "a gerund after a form of be is refused" {
  prose "The gate is running the checks."

  run "${script}"

  [[ "${status}" -eq 1 ]]
  [[ "${output}" == *"is a technical noun here and never a verb, a rule ${borrowed}"* ]]
  [[ "${output}" == *"${record}:5: running"* ]]
}

@test "a gerund after a preposition is refused" {
  prose "A writer finds it by running the gate."

  run "${script}"

  [[ "${status}" -eq 1 ]]
  [[ "${output}" == *"${record}:5: running"* ]]
}

@test "a gerund after as is refused" {
  prose "The record names it as owing work."

  run "${script}"

  [[ "${status}" -eq 1 ]]
  [[ "${output}" == *"${record}:5: owing"* ]]
}

@test "a gerund after because is refused" {
  prose "The gate stops, because refusing would close the issue."

  run "${script}"

  [[ "${status}" -eq 1 ]]
  [[ "${output}" == *"${record}:5: refusing"* ]]
}

@test "an adverb between the two does not hide the gerund" {
  prose "The gate is already running the checks."

  run "${script}"

  [[ "${status}" -eq 1 ]]
  [[ "${output}" == *"${record}:5: running"* ]]
}

@test "an adverb ending in ly does not hide the gerund" {
  prose "The gate is quickly running."

  run "${script}"

  [[ "${status}" -eq 1 ]]
  [[ "${output}" == *"${record}:5: running"* ]]
}

@test "two adverbs between the two do not hide the gerund" {
  prose "The gate is not always running the checks."

  run "${script}"

  [[ "${status}" -eq 1 ]]
  [[ "${output}" == *"${record}:5: running"* ]]
}

@test "a technical noun passes where a gerund is refused" {
  prose "A rule is nothing without tooling."

  run "${script}"

  [[ "${status}" -eq 0 ]]
}

@test "a hyphenated compound is not a verb form" {
  prose "The setting is load-bearing."

  run "${script}"

  [[ "${status}" -eq 0 ]]
}

@test "a code span between a preposition and a gerund keeps them apart" {
  prose "It reads over \`the tree\`, checking each line."

  run "${script}"

  [[ "${status}" -eq 0 ]]
}

@test "a three-letter word is not a participle" {
  prose "The run had red beside it."

  run "${script}"

  [[ "${status}" -eq 0 ]]
}

@test "a perfect tense is refused" {
  prose "The gate has refused a record."

  run "${script}"

  [[ "${status}" -eq 1 ]]
  [[ "${output}" == *"A record uses the simple tenses, a rule ${borrowed}"* ]]
  [[ "${output}" == *"${record}:5: has refused"* ]]
}

@test "a modal with the bare verb passes, and so does a bound worth having" {
  prose "The gate must be run, and that is a bound worth having."

  run "${script}"

  [[ "${status}" -eq 0 ]]
}

@test "two adverbs do not hide a perfect tense" {
  prose "The gate has not yet refused a record."

  run "${script}"

  [[ "${status}" -eq 1 ]]
  [[ "${output}" == *"${record}:5: has refused"* ]]
}

@test "two adverbs do not hide a perfect tense after having" {
  prose "Having not yet refused it, the gate waits."

  run "${script}"

  [[ "${status}" -eq 1 ]]
  [[ "${output}" == *"${record}:5: having refused"* ]]
}

@test "adverbs that open a sentence are not a tense" {
  prose "Not yet refused, the record stands."

  run "${script}"

  [[ "${status}" -eq 0 ]]
}

# Each sentence, and the pair the refusal names.
@test "a word between has and its participle does not hide the tense" {
  while IFS='|' read -r sentence pair; do
    compliant >"${record}"
    prose "${sentence}"

    run "${script}" </dev/null

    [[ "${status}" -eq 1 ]]
    [[ "${output}" == *"${record}:5: ${pair}"* ]]
  done <<'CASES'
A person has even moved the date.|has moved
The gate has itself refused the record.|has refused
Has anyone moved the date?|has moved
Has the gate refused the record?|has refused
Have purba's own 92 issues been consistent?|have been
Having itself refused it, the gate waits.|having refused
CASES
}

@test "a participle that no list names is a tense" {
  while IFS='|' read -r sentence pair; do
    compliant >"${record}"
    prose "${sentence}"

    run "${script}" </dev/null

    [[ "${status}" -eq 1 ]]
    [[ "${output}" == *"${record}:5: ${pair}"* ]]
  done <<'CASES'
This project has never had a contributor.|has had
The person has rewritten the record.|has rewritten
CASES
}

@test "a participle in the participle list is a tense" {
  prose "It has grown one workflow at a time."

  run "${script}"

  [[ "${status}" -eq 1 ]]
  [[ "${output}" == *"${record}:5: has grown"* ]]
}

@test "with an empty participle list, has grown is no tense" {
  prose "It has grown one workflow at a time."
  : >"${BATS_TEST_TMPDIR}/participles.txt"

  CHECK_TENSE_PARTICIPLES="${BATS_TEST_TMPDIR}/participles.txt" run "${script}"

  [[ "${status}" -eq 0 ]]
}

@test "a tense wrapped across two lines is refused at its first line" {
  prose "The gate has
refused the record."

  run "${script}"

  [[ "${status}" -eq 1 ]]
  [[ "${output}" == *"${record}:5: has refused"* ]]
}

@test "a possession, an obligation, a passive and a code span are no tense" {
  for sentence in "The gate has a fixed span." "A check has nothing to run against." \
    "The gate has records which were refused." "The lock has \`rust\` pinned." \
    "Does the record have a fixed span?" "Having a record refused is rare."; do
    compliant >"${record}"
    prose "${sentence}"

    run "${script}"

    [[ "${status}" -eq 0 ]]
  done
}

@test "a tense in a table row, a heading or a link title is not read" {
  for block in "| a | The gate has refused the record. |
|---|---|" "### The gate has refused the record" \
    "See [the gate has refused the record](https://example.com) here."; do
    compliant >"${record}"
    prose "Something needs a decision.

${block}"

    run "${script}"

    [[ "${status}" -eq 0 ]]
  done
}

@test "a Downside label followed by a sentence is refused" {
  swap "**Downside:**" "**Downside:** It costs."

  run "${script}"

  [[ "${status}" -eq 1 ]]
  [[ "${output}" == *"A Downside label stands on its own line"* ]]
  [[ "${output}" != *"A Downside label never counts its costs"* ]]
  [[ "${output}" == *"${record}:15"* ]]
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
  [[ "${output}" == *"${record}:15"* ]]
}

@test "a cost with no bold lead-in is refused" {
  swap "- **It costs something.** The cost is named." "- It costs something."

  run "${script}"

  [[ "${status}" -eq 1 ]]
  [[ "${output}" == *"Bold marks every Downside lead-in"* ]]
  [[ "${output}" == *"${record}:15: 0 lead-ins over 1 costs"* ]]
}

@test "two costs on one line are refused" {
  swap "The cost is named." "The cost is named. - **A second cost.** It does not render."

  run "${script}"

  [[ "${status}" -eq 1 ]]
  [[ "${output}" == *"${record}:15: 2 lead-ins over 1 costs"* ]]
}

# The number is joined to its prefix here and never written beside it. The rule
# reads every tracked file, this one included, and would refuse its own test.
@test "a numbered citation in any tracked file is refused" {
  printf 'oxitest recorded this as ADR-%s.\n' 0019 >NOTES.md
  git add NOTES.md

  run "${script}"

  [[ "${status}" -eq 1 ]]
  [[ "${output}" == *"source cites a record by its proposition"* ]]
  [[ "${output}" == *"NOTES.md:1:"* ]]
}

@test "a Confirmation that admits an unwired gate and names no issue is refused" {
  swap "\`mise run records\` reads this." "The gate is not wired."

  run "${script}"

  [[ "${status}" -eq 1 ]]
  [[ "${output}" == *"names the issue that will wire it"* ]]
  [[ "${output}" == *$'has an owner.\n  '"${record}"* ]]
}

@test "a Confirmation whose table answers no is refused the same way" {
  swap "\`mise run records\` reads this." $'| rule | checked |\n|---|---|\n| one | no |'

  run "${script}"

  [[ "${status}" -eq 1 ]]
  [[ "${output}" == *"names the issue that will wire it"* ]]
}

@test "a Confirmation that names the issue passes" {
  swap "\`mise run records\` reads this." \
    "The gate is not wired. [Wire it](https://github.com/o/purba/issues/9) owns it."

  run "${script}"

  [[ "${status}" -eq 0 ]]
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

@test "two adverbs do not hide the passive voice" {
  prose "The gate is not yet refused by nothing. We utilize it."

  run "${script}"

  [[ "${status}" -eq 0 ]]
  [[ "${output}" == *"in the passive voice             3 of     9 sentences  33%"* ]]
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

@test "outside a git repository it exits 2" {
  mkdir -p "${BATS_TEST_TMPDIR}/bare/docs/decisions"
  cp "${record}" "${BATS_TEST_TMPDIR}/bare/docs/decisions/"
  cd "${BATS_TEST_TMPDIR}/bare"

  run "${script}"

  [[ "${status}" -eq 2 ]]
  [[ "${output}" == *"can only be read inside a git repository"* ]]
}

@test "each phrase that admits an unwired gate needs an issue" {
  for admission in "It is run by hand." "The check is not written yet." "It does not exist yet."; do
    compliant >"${record}"
    swap "\`mise run records\` reads this." "${admission}"

    run "${script}"

    [[ "${status}" -eq 1 ]]
    [[ "${output}" == *$'has an owner.\n  '"${record}"* ]]
  done
}

@test "an admission outside the Confirmation needs no issue" {
  prose "The gate is not wired."

  run "${script}"

  [[ "${status}" -eq 0 ]]
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
  swap "\`mise run records\` reads this." "- \`mise run records\` reads this."

  run "${script}"

  [[ "${status}" -eq 0 ]]
}

@test "a nested cost is a cost, and a warning sign may precede its lead-in" {
  swap "The cost is named." $'The cost is named.\n  - ⚠️ **A nested cost.** It counts.'

  run "${script}"

  [[ "${status}" -eq 0 ]]
}

@test "a fence nested under a list item opens a block" {
  swap "- **This.** Chosen." $'- **This.** Chosen.\n\n  ```\n  a — b\n  ```'

  run "${script}"

  [[ "${status}" -eq 0 ]]
}

# The first record ends inside a fence it never closes. A reader that carried
# that state into the second would skip everything the second holds.
@test "a fence one record leaves open does not hide the next record" {
  swap "\`mise run records\` reads this." $'\`mise run records\` reads this.\n\n```\nunclosed'
  second=docs/decisions/b-second-record.md
  compliant >"${second}"
  swap_in "${second}" "Something needs a decision." "Something — else."

  run "${script}"

  [[ "${status}" -eq 1 ]]
  [[ "${output}" == *"A record carries no em-dash"* ]]
  [[ "${output}" == *"${second}:5"* ]]
}

@test "each line that is not prose closes the paragraph above it" {
  for closer in '### A heading' '> Quoted.' '1. Numbered.' '  Indented.' '* Starred.' '+ Plus.'; do
    compliant >"${record}"
    prose $'One.\nTwo.\nThree.\nFour.\n'"${closer}"$'\nFive.\nSix.\nSeven.\nEight.'

    run "${script}"

    [[ "${status}" -eq 0 ]]
  done
}

@test "an exclamation mark and a question mark each end a sentence" {
  words 14
  prose "${filler}end! ${filler}end? ${filler}end."

  run "${script}"

  [[ "${status}" -eq 0 ]]
}

@test "a perfect tense is refused in each of its forms" {
  for perfect in "They have refused it." "It had refused it." "It has written it." \
    "Having refused it, the gate stops."; do
    compliant >"${record}"
    prose "${perfect}"

    run "${script}"

    [[ "${status}" -eq 1 ]]
    [[ "${output}" == *"A record uses the simple tenses"* ]]
  done
}

@test "an adverb ending in ly does not hide a perfect tense" {
  prose "A person has recently moved the date."

  run "${script}"

  [[ "${status}" -eq 1 ]]
  [[ "${output}" == *"${record}:5: has moved"* ]]
}

@test "since, ever and twice do not hide a perfect tense" {
  for perfect in "A person has since moved the date." "No person has ever moved the date." \
    "A person has twice moved the date."; do
    compliant >"${record}"
    prose "${perfect}"

    run "${script}"

    [[ "${status}" -eq 1 ]]
    [[ "${output}" == *"${record}:5: has moved"* ]]
  done
}

@test "a word of four letters that ends in ing is not a gerund" {
  prose "A host answers by ping."

  run "${script}"

  [[ "${status}" -eq 0 ]]
}

@test "bold after a code span sits inside the sentence and is refused" {
  prose "\`mise\` **really** runs it."

  run "${script}"

  [[ "${status}" -eq 1 ]]
  [[ "${output}" == *"Bold opens a sentence and never sits inside one"* ]]
}

@test "a mark with no letter and no digit in it is not a word" {
  words 25
  prose "${filler}/ end."

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

@test "one bold letter inside a sentence is refused" {
  prose "Something needs **a** decision."

  run "${script}"

  [[ "${status}" -eq 1 ]]
  [[ "${output}" == *"Bold opens a sentence and never sits inside one"* ]]
}

@test "a Downside label that counts its costs in a word is refused" {
  swap "**Downside:**" "**Downside:** two costs."

  run "${script}"

  [[ "${status}" -eq 1 ]]
  [[ "${output}" == *"A Downside label never counts its costs"* ]]
}

@test "an admission needs a link to an issue of this repository" {
  swap "\`mise run records\` reads this." \
    "The gate is not wired. [Wire it](https://github.com/o/other/issues/9) owns it."

  run "${script}"

  [[ "${status}" -eq 1 ]]
  [[ "${output}" == *$'has an owner.\n  '"${record}"* ]]
}

@test "the report counts the articles and the words it read" {
  run "${script}"

  [[ "${status}" -eq 0 ]]
  [[ "${output}" == *"articles                         2 in    20 words"* ]]
  [[ "${output}" == *"formal words                     0 in    20 words"* ]]
}

@test "a prose line that holds a link is not a bare line" {
  prose "Something needs [a decision](https://example.invalid)."

  run "${script}"

  [[ "${status}" -eq 0 ]]
  [[ "${output}" == *" 50.0%    2 of   4  ${record}"* ]]
}
