# scripts/unmark-lost-options.sh against the records it rewrites.
: "${BATS_TEST_DIRNAME:?set by bats}"

setup() {
  # shellcheck source=scripts/test/fixture.sh
  . "${BATS_TEST_DIRNAME}/fixture.sh"
  script="${BATS_TEST_DIRNAME}/../unmark-lost-options.sh"
  make_repo
  mkdir -p docs/decisions
  record=docs/decisions/a-record-holds.md
}

# A record whose Considered Options holds <option>, one line or more, and
# whose Decision Outcome holds <outcome>.
holds() {
  cat >"${record}" <<RECORD
# A record holds

## Considered Options

$1

## Decision Outcome

${2:-**This is decided.**}
RECORD
}

@test "Rejected. after the lead-in of a lost option goes, and the reason stays" {
  holds $'- **That.** Rejected. It lost.\n- **Other.** Rejected. It lost too.'

  run "${script}"

  [[ "${status}" -eq 0 ]]
  [[ "$(<"${record}")" == *$'- **That.** It lost.\n- **Other.** It lost too.\n'* ]]
}

@test "a lost option that holds only Rejected. keeps its lead-in" {
  holds "- **That.** Rejected."

  run "${script}"

  [[ "${status}" -eq 0 ]]
  [[ "$(<"${record}")" == *$'\n- **That.**\n'* ]]
}

@test "another form of Rejected stays for a writer to rewrite" {
  holds "- **That.** Rejected on measurement. It lost."

  run "${script}"

  [[ "${status}" -eq 0 ]]
  [[ "$(<"${record}")" == *"- **That.** Rejected on measurement. It lost."* ]]
}

@test "an option inside a fence stays" {
  holds $'```text\n- **That.** Rejected. It lost.\n```'

  run "${script}"

  [[ "${status}" -eq 0 ]]
  [[ "$(<"${record}")" == *"- **That.** Rejected. It lost."* ]]
}

@test "a lead-in outside Considered Options stays" {
  holds "- **That.** It lost." "- **This.** Rejected. It stays."

  run "${script}"

  [[ "${status}" -eq 0 ]]
  [[ "$(<"${record}")" == *"- **This.** Rejected. It stays."* ]]
}

@test "a record with no mark is left byte for byte" {
  holds "- **That.** It lost."
  printf '\n\n' >>"${record}"
  cp "${record}" before

  run "${script}"

  [[ "${status}" -eq 0 ]]
  cmp before "${record}"
}

@test "the template is not rewritten" {
  holds "- **That.** It lost."
  printf -- '- **{option}.** Rejected. {Why it lost.}\n' >docs/decisions/.template.md

  run "${script}"

  [[ "${status}" -eq 0 ]]
  [[ "$(<docs/decisions/.template.md)" == *"Rejected."* ]]
}

@test "the directory is an argument" {
  mkdir -p elsewhere
  holds "- **That.** Rejected. It lost."
  mv "${record}" elsewhere/a-record-holds.md

  run "${script}" elsewhere

  [[ "${status}" -eq 0 ]]
  [[ "$(<elsewhere/a-record-holds.md)" == *"- **That.** It lost."* ]]
}

@test "a directory with no record cannot be rewritten" {
  run "${script}" nowhere

  [[ "${status}" -eq 2 ]]
}
