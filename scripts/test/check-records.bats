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
  area="**Area:** tooling"
  refused_area="A record names each area it decides something in, in an \`**Area:**\` paragraph"
  landscape >LANDSCAPE.md
  template=docs/decisions/.template.md
  register >"${template}"
  compliant >"${record}"
  git add "${record}"
}

# A template whose comment no rule refuses. The comment names one record, and
# a path and the glossary, which are no record names. Its comment ends on line
# 5, and the form below it is never read.
register() {
  cat <<'TEMPLATE'
<!--
Copy this file to docs/decisions/{the-proposition-as-a-slug}.md.
a-record-holds.md says why, and CONTEXT.md names each word.
docs/agents/a-guide.md is no record.
-->

# {the proposition}
TEMPLATE
}

# A record no rule refuses. Each test changes one thing in it.
compliant() {
  cat <<'RECORD'
# A record holds

## Context and Problem Statement

Something needs a decision.

## Considered Options

- **That.** It lost.

## Decision Outcome

**Reach:** `docs/decisions/*.md`

**Area:** tooling

**This is decided.**

**Downside:**

- **It costs something.** The cost is named.

## Confirmation

`mise run lint:records` reads this.
RECORD
}

# The landscape the area rule reads, two folders above the records. It names one
# part, one invariant and one concern.
landscape() {
  cat <<'LANDSCAPE'
# purba's landscape

## The parts and their dependencies

- **alpha.** Starts the run.

## The invariants

- **one parser.** No part but alpha parses the source.

## The cross-cutting concerns

- **the beta seam.** Carries a live object, in every part.
LANDSCAPE
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
  mkdir -p elsewhere/decisions
  swap "## Considered Options" "## Options"
  mv "${record}" elsewhere/decisions/a-record-holds.md
  cp "${template}" elsewhere/decisions/

  run "${script}" elsewhere/decisions

  [[ "${status}" -eq 1 ]]
  [[ "${output}" == *$'the confirmation.\n  elsewhere/decisions/a-record-holds.md'* ]]
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
  swap $'**That.** It lost.\n\n## Decision Outcome' $'**That.** It lost.\n\n## Considered Options'

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

@test "the template's comment obeys the prose rules, refused at the template's own line" {
  swap_in "${template}" "says why," "says why, as #45 said,"

  run "${script}"

  [[ "${status}" -eq 1 ]]
  [[ "${output}" == *"record prose carries no issue number"* ]]
  [[ "${output}" == *"${template}:3:"* ]]
}

@test "only the template's comment is read, and never as a record" {
  printf '\nAs #45 said.\n' >>"${template}"

  run "${script}"

  [[ "${status}" -eq 0 ]]
  [[ "${output}" != *".template.md"* ]]
}

@test "a record that the template's comment names and no file holds is refused" {
  swap_in "${template}" "a-record-holds.md" "a-gone-record.md"

  run "${script}"

  [[ "${status}" -eq 1 ]]
  [[ "${output}" == *"A record that the template's comment names"* ]]
  [[ "${output}" == *"${template}:3: a-gone-record.md"* ]]
}

@test "a name with an underscore before it, or a letter after .md, is no record name" {
  swap_in "${template}" "is no record." "is no record, nor note_gone.md, nor a-gone-record.mdx."

  run "${script}"

  [[ "${status}" -eq 0 ]]
}

@test "a name after another name and a slash is part of a path, and no record name" {
  swap_in "${template}" "is no record." "is no record, nor a-record-holds.md/a-gone-record.md."

  run "${script}"

  [[ "${status}" -eq 0 ]]
}

@test "a name with a second extension is no record name" {
  swap_in "${template}" "is no record." "is no record, nor a-gone-record.md.bak."

  run "${script}"

  [[ "${status}" -eq 0 ]]
}

@test "a name before a full stop is a record name" {
  swap_in "${template}" "is no record." "is no record, and neither is a-gone-record.md."

  run "${script}"

  [[ "${status}" -eq 1 ]]
  [[ "${output}" == *"${template}:4: a-gone-record.md"* ]]
}

@test "a missing name written twice on one line is refused once" {
  swap_in "${template}" "is no record." "is no record, nor a-gone.md, nor a-gone.md again."

  run "${script}"

  [[ "${status}" -eq 1 ]]
  found=$(grep -c "${template}:4: a-gone.md" <<<"${output}")
  [[ "${found}" -eq 1 ]]
}

@test "the first part of a path is no record name, though no file holds it" {
  swap_in "${template}" "is no record." "is no record, nor a-gone-record.md/a-record-holds.md."

  run "${script}"

  [[ "${status}" -eq 0 ]]
}

@test "a missing record name is refused once, though a longer name before it holds it" {
  swap_in "${template}" "is no record." "is no record, nor a-gone-record.mdx, nor a-gone-record.md"

  run "${script}"

  [[ "${status}" -eq 1 ]]
  found=$(grep -c "${template}:4: a-gone-record.md" <<<"${output}")
  [[ "${found}" -eq 1 ]]
}

@test "a missing template cannot be decided" {
  rm "${template}"

  run "${script}"

  [[ "${status}" -eq 2 ]]
  [[ "${output}" == *"the comment of ${template} cannot be read"* ]]
}

@test "a missing template says the file is missing, and nothing of the two markers" {
  rm "${template}"

  run "${script}"

  [[ "${status}" -eq 2 ]]
  [[ "${output}" == *"is missing"* ]]
  [[ "${output}" != *"stand alone"* ]]
}

@test "a template that exists and cannot be read says so, and not that it is missing" {
  chmod 000 "${template}"

  run "${script}"

  chmod 644 "${template}"
  [[ "${status}" -eq 2 ]]
  [[ "${output}" == *"cannot be opened"* ]]
  [[ "${output}" != *"is missing"* ]]
}

@test "a template that does not open with its comment cannot be decided" {
  printf '# {the proposition}\n' >"${template}"

  run "${script}"

  [[ "${status}" -eq 2 ]]
  [[ "${output}" == *"the comment of ${template} cannot be read"* ]]
}

@test "a first line that holds more than the marker cannot be decided, and the message says why" {
  swap_in "${template}" $'<!--\nCopy' '<!-- Copy'

  run "${script}"

  [[ "${status}" -eq 2 ]]
  [[ "${output}" == *"each stand alone on a line"* ]]
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

@test "a sentence of twenty-one words in a numbered list is refused" {
  words 21
  prose "1. ${filler}end."

  run "${script}"

  [[ "${status}" -eq 1 ]]
  [[ "${output}" == *"A sentence in a numbered list runs to 20 words, a limit ${borrowed}"* ]]
  [[ "${output}" == *"${record}:5: 21 words"* ]]
}

@test "a sentence of twenty-one words in a bulleted list passes" {
  words 21
  prose "- ${filler}end."

  run "${script}"

  [[ "${status}" -eq 0 ]]
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
  [[ "${output}" == *"${record}:19"* ]]
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

@test "a cost with no bold lead-in is refused" {
  swap "- **It costs something.** The cost is named." "- It costs something."

  run "${script}"

  [[ "${status}" -eq 1 ]]
  [[ "${output}" == *"Bold marks every Downside lead-in"* ]]
  [[ "${output}" == *"${record}:19: 0 lead-ins over 1 costs"* ]]
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

@test "on a runner an unwired gate writes no annotation" {
  swap "\`mise run lint:records\` reads this." "The gate is not wired."

  GITHUB_ACTIONS=true run "${script}"

  [[ "${status}" -eq 0 ]]
  [[ "${output}" != *"::error::"* ]]
  [[ "${output}" == *"${unwired}"* ]]
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
  [[ "${output}" == *"in the passive voice             3 of    10 sentences  30%"* ]]
  [[ "${output}" == *"the formal words found"*"utilize"* ]]
}

@test "evidence density is reported for each record and for all of them" {
  run "${script}"

  [[ "${status}" -eq 0 ]]
  [[ "${output}" == *" 80.0%    4 of   5  ${record}"* ]]
  [[ "${output}" == *"all records  80.0%  4 of 5"* ]]
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

@test "a tag that check-records.sh does not know exits 2" {
  unset PURBA_CHECK_PROSE # it fakes cargo, which a built binary skips
  fake cargo <<'FAKE'
printf 'unheard\tdocs/decisions/a-record-holds.md:1\n'
FAKE

  run "${script}"

  [[ "${status}" -eq 2 ]]
  [[ "${output}" == *"unheard"* ]]
}

@test "outside a git repository it reads the records" {
  swap "${reach}" "${reach} \`nowhere/**\`"
  mkdir -p "${BATS_TEST_TMPDIR}/bare/docs/decisions"
  cp "${record}" "${template}" "${BATS_TEST_TMPDIR}/bare/docs/decisions/"
  cp LANDSCAPE.md "${BATS_TEST_TMPDIR}/bare/"
  cd "${BATS_TEST_TMPDIR}/bare"

  run "${script}"

  [[ "${status}" -eq 0 ]]
  [[ "${output}" != *"${unmatched}"* ]]
}

@test "evidence density lists the barest record first" {
  second=docs/decisions/b-second-record.md
  compliant >"${second}"
  swap_in "${second}" "Something needs a decision." "\`This\` needs a decision."
  swap_in "${second}" "**This is decided.**" "**\`This\` is decided.**"

  run "${script}"

  [[ "${status}" -eq 0 ]]
  [[ "${output}" == *" 80.0%    4 of   5  ${record}"*" 40.0%    2 of   5  ${second}"* ]]
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

@test "the report counts the articles and the words it read" {
  run "${script}"

  [[ "${status}" -eq 0 ]]
  [[ "${output}" == *"articles                         2 in    23 words"* ]]
  [[ "${output}" == *"formal words                     0 in    23 words"* ]]
}

@test "a Decision Outcome that does not open with a reach is refused" {
  swap "${reach}"$'\n\n' ""

  run "${script}"

  [[ "${status}" -eq 1 ]]
  [[ "${output}" == *"A Decision Outcome opens with a reach"* ]]
  [[ "${output}" == *"${record}:13"* ]]
}

@test "a reach below the first line of the Decision Outcome is refused" {
  swap "${reach}"$'\n\n'"${area}"$'\n\n**This is decided.**' \
    $'**This is decided.**\n\n'"${reach}"$'\n\n'"${area}"

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
  [[ "${output}" == *"articles                         2 in    23 words"* ]]
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

@test "an area that the landscape names passes" {
  swap "${area}" "**Area:** the beta seam"

  run "${script}"

  [[ "${status}" -eq 0 ]]
}

@test "the four areas outside the landscape pass, a comma between two" {
  swap "${area}" "**Area:** design, workflow, writing, tooling, alpha"

  run "${script}"

  [[ "${status}" -eq 0 ]]
}

@test "a record with no area line is refused where the line belongs" {
  swap "${area}"$'\n\n' ""

  run "${script}"

  [[ "${status}" -eq 1 ]]
  [[ "${output}" == *"${refused_area} after its reach."* ]]
  [[ "${output}" == *"An area is design, workflow, writing, tooling, or the name of a part"* ]]
  [[ "${output}" == *"${record}:15"* ]]
}

@test "an area line below the decision is refused" {
  swap "${area}"$'\n\n**This is decided.**' $'**This is decided.**\n\n'"${area}"

  run "${script}"

  [[ "${status}" -eq 1 ]]
  [[ "${output}" == *"${refused_area}"* ]]
  [[ "${output}" == *"${record}:15"* ]]
}

@test "an area line with no area is refused" {
  swap "${area}" "**Area:**"

  run "${script}"

  [[ "${status}" -eq 1 ]]
  [[ "${output}" == *"${refused_area}"* ]]
  [[ "${output}" == *"${record}:15"* ]]
}

@test "an area outside the list is refused and named" {
  swap "${area}" "**Area:** bugs, tooling"

  run "${script}"

  [[ "${status}" -eq 1 ]]
  [[ "${output}" == *"${refused_area}"* ]]
  [[ "${output}" == *"${record}:15: bugs"* ]]
  [[ "${output}" != *": tooling"* ]]
}

@test "an invariant of the landscape is no area" {
  swap "${area}" "**Area:** one parser"

  run "${script}"

  [[ "${status}" -eq 1 ]]
  [[ "${output}" == *"${record}:15: one parser"* ]]
}

@test "a landscape that names no part cannot be decided" {
  printf '# purba\x27s landscape\n' >LANDSCAPE.md

  run "${script}"

  [[ "${status}" -eq 2 ]]
  [[ "${output}" == *"LANDSCAPE.md names no part"* ]]
}

@test "math outside a math fence is refused" {
  prose "Something needs a decision at \$x\$ now."

  run "${script}"

  [[ "${status}" -eq 1 ]]
  [[ "${output}" == *"A formula in a record is a"* ]]
  [[ "${output}" == *"A record writes each kind of figure in one form"* ]]
  [[ "${output}" == *"${record}:5"* ]]
}

@test "an icon in a mermaid fence is refused" {
  prose $'Something needs a decision.\n\n```mermaid\nflowchart LR\n  A[fa:fa-twitter Tweet]\n```'

  run "${script}"

  [[ "${status}" -eq 1 ]]
  [[ "${output}" == *"draws no icon"* ]]
  [[ "${output}" == *"${record}:9"* ]]
}

@test "a record whose fence holds a ## line passes" {
  prose $'Something needs a decision.\n\n```sh\n## not a heading\n```'

  run "${script}"

  [[ "${status}" -eq 0 ]]
}

@test "a record whose fence holds a <!-- passes" {
  prose $'Something needs a decision.\n\n```html\n<!-- quoted -->\n```'

  run "${script}"

  [[ "${status}" -eq 0 ]]
}

@test "a record whose fence holds a #12 passes" {
  prose $'Something needs a decision.\n\n```sh\ngh issue view #12\n```'

  run "${script}"

  [[ "${status}" -eq 0 ]]
}

@test "a record whose fence holds an ADR-12 passes" {
  prose $'Something needs a decision.\n\n```text\nADR-12\n```'

  run "${script}"

  [[ "${status}" -eq 0 ]]
}

@test "a #12 on the line after a closing fence is refused" {
  prose $'Something needs a decision.\n\n```sh\ngh issue view\n```\nIt waits on #12.'

  run "${script}"

  [[ "${status}" -eq 1 ]]
  [[ "${output}" == *"record prose carries no issue number"* ]]
  [[ "${output}" == *"${record}:10:"* ]]
}

@test "a paragraph line followed by --- is refused for its sections" {
  prose $'Something needs a decision.\n---'

  run "${script}"

  [[ "${status}" -eq 1 ]]
  [[ "${output}" == *$'the confirmation.\n  '"${record}"* ]]
}

@test "a winner marked Chosen in Considered Options is refused where it stands" {
  swap "- **That.** It lost." "- **That.** Chosen. It won."

  run "${script}"

  [[ "${status}" -eq 1 ]]
  [[ "${output}" == *"Considered Options holds only what lost"* ]]
  [[ "${output}" == *"${record}:9"* ]]
}

@test "a lost option that opens with Rejected. is refused, and the refusal names the fixer" {
  swap "- **That.** It lost." "- **That.** Rejected. It lost."

  run "${script}"

  [[ "${status}" -eq 1 ]]
  [[ "${output}" == *"A lost option states why it lost, with no mark"* ]]
  [[ "${output}" == *"mise run fmt:records"* ]]
  [[ "${output}" == *"${record}:9"* ]]
}

@test "another form of Rejected that opens a lost option is refused" {
  swap "- **That.** It lost." "- **That.** Rejected on measurement. It lost."

  run "${script}"

  [[ "${status}" -eq 1 ]]
  [[ "${output}" == *"A lost option states why it lost, with no mark"* ]]
  [[ "${output}" == *"${record}:9"* ]]
}

@test "Rejected later in the reason of a lost option passes" {
  swap "- **That.** It lost." "- **That.** It lost. A review Rejected it."

  run "${script}"

  [[ "${status}" -eq 0 ]]
}

@test "an option marked Chosen inside a fence in Considered Options passes" {
  swap "- **That.** It lost." $'- **That.** It lost.\n\n```text\n- **This.** Chosen.\n```'

  run "${script}"

  [[ "${status}" -eq 0 ]]
}

@test "a lead-in with Chosen after it outside Considered Options passes" {
  swap "**This is decided.**" $'**This is decided.**\n\n- **This.** Chosen. Rejected.'

  run "${script}"

  [[ "${status}" -eq 0 ]]
}

@test "an option marked Chosen inside a tilde fence in Considered Options passes" {
  swap "- **That.** It lost." $'- **That.** It lost.\n\n~~~text\n- **This.** Chosen.\n~~~'

  run "${script}"

  [[ "${status}" -eq 0 ]]
}

@test "a plain decision sentence after the area is refused where it stands" {
  swap "**This is decided.**" "This is decided."

  run "${script}"

  [[ "${status}" -eq 1 ]]
  [[ "${output}" == *"after its area, with the decision in one"* ]]
  [[ "${output}" == *"${record}:17"* ]]
}

@test "a decision sentence bold in part is refused where it stands" {
  swap "**This is decided.**" "**This is decided**, for now."

  run "${script}"

  [[ "${status}" -eq 1 ]]
  [[ "${output}" == *"after its area, with the decision in one"* ]]
  [[ "${output}" == *"${record}:17"* ]]
}

@test "a decision that opens with a list item is refused" {
  swap "**This is decided.**" "- **This is decided.**"

  run "${script}"

  [[ "${status}" -eq 1 ]]
  [[ "${output}" == *"after its area, with the decision in one"* ]]
  [[ "${output}" == *"${record}:17"* ]]
}

@test "a decision that opens with a fence is refused" {
  swap "**This is decided.**" $'```text\nThis is decided.\n```'

  run "${script}"

  [[ "${status}" -eq 1 ]]
  [[ "${output}" == *"after its area, with the decision in one"* ]]
  [[ "${output}" == *"${record}:17"* ]]
}

@test "a gerund after beside is refused" {
  prose "The gate sits beside checking the records."

  run "${script}"

  [[ "${status}" -eq 1 ]]
  [[ "${output}" == *"${record}:5: checking"* ]]
}

@test "a bold label straight after the area is refused where it stands" {
  swap "**This is decided.**" "**Note:**"

  run "${script}"

  [[ "${status}" -eq 1 ]]
  [[ "${output}" == *"after its area, with the decision in one"* ]]
  [[ "${output}" == *"${record}:17"* ]]
}

@test "a bold line that ends no sentence is refused where it stands" {
  swap "**This is decided.**" "**This is decided (for now)**"

  run "${script}"

  [[ "${status}" -eq 1 ]]
  [[ "${output}" == *"after its area, with the decision in one"* ]]
  [[ "${output}" == *"${record}:17"* ]]
}

@test "a decision sentence that holds ** in a code span passes" {
  swap "**This is decided.**" "**The reach \`**\` is decided.**"

  run "${script}"

  [[ "${status}" -eq 0 ]]
}

@test "a decision sentence with a space after it passes" {
  swap "**This is decided.**" "**This is decided.** "

  run "${script}"

  [[ "${status}" -eq 0 ]]
}

@test "a decision sentence that ends in an emphasis passes" {
  swap "**This is decided.**" "**This is decided *for now*.**"

  run "${script}"

  [[ "${status}" -eq 0 ]]
}

@test "a decision sentence that ends in a quote passes" {
  swap "**This is decided.**" "**The word is \"done.\"**"

  run "${script}"

  [[ "${status}" -eq 0 ]]
}

@test "a decision sentence that ends in a parenthesis passes" {
  swap "**This is decided.**" "**This is decided (for now.)**"

  run "${script}"

  [[ "${status}" -eq 0 ]]
}

@test "a decision sentence that ends in a single quote passes" {
  swap "**This is decided.**" "**The word is 'done.'**"

  run "${script}"

  [[ "${status}" -eq 0 ]]
}

@test "a built binary that PURBA_CHECK_PROSE names reads the prose, and no cargo starts" {
  fake cargo <<'FAKE'
exit 1
FAKE
  fake check-prose </dev/null
  export PURBA_CHECK_PROSE="${BATS_TEST_TMPDIR}/bin/check-prose"

  run "${script}"

  started=$(calls cargo)
  ran=$(calls check-prose)
  [[ -z "${started}" ]]
  [[ "${ran}" == *"participles.txt ${record}"* ]]
}
