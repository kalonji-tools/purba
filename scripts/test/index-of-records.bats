# scripts/index-of-records.sh against the records it lists, and the page it writes.
: "${BATS_TEST_DIRNAME:?set by bats}"

setup() {
  # shellcheck source=scripts/test/fixture.sh
  . "${BATS_TEST_DIRNAME}/fixture.sh"
  script="${BATS_TEST_DIRNAME}/../index-of-records.sh"
  make_repo
  stale="docs/README.md is refused when it differs from the page that the area lines give"
  mkdir -p docs/decisions
  cat >LANDSCAPE.md <<'LANDSCAPE'
# purba's landscape

## The parts and their dependencies

- **alpha.** Starts the run.
- **gamma.** Parses the source.

## The cross-cutting concerns

- **the beta seam.** Carries a live object, in every part.
LANDSCAPE
  record b-beta "Beta holds" design
  record a-alpha "Alpha holds" "alpha, tooling"
  record c-seam "The seam holds" "the beta seam"
}

# A record that names its title and its areas, and holds nothing else the
# script reads.
record() {
  cat >"docs/decisions/$1.md" <<RECORD
# $2

## Decision Outcome

**Reach:** \`docs/decisions/*.md\`

**Area:** $3

It is decided.
RECORD
}

@test "the page lists each record under each area it names, in the order of the landscape" {
  run "${script}"

  [[ "${status}" -eq 0 ]]
  cat >expected <<'PAGE'
# The decision records, by area

`mise run fmt:index` writes this page from the `**Area:**` line of each record.

## Product design

- [Beta holds](decisions/b-beta.md)

## One part or concern

### alpha

- [Alpha holds](decisions/a-alpha.md)

### the beta seam

- [The seam holds](decisions/c-seam.md)

## Repository tooling

- [Alpha holds](decisions/a-alpha.md)
PAGE
  diff -u expected docs/README.md
}

@test "the records of one area follow their file names, not their titles" {
  record a-first "Zeta comes first" design

  run "${script}"

  [[ "${status}" -eq 0 ]]
  [[ "$(<docs/README.md)" == *$'- [Zeta comes first](decisions/a-first.md)\n- [Beta holds]'* ]]
}

@test "the template is not listed" {
  record .template "{the proposition}" design

  run "${script}"

  [[ "${status}" -eq 0 ]]
  [[ "$(<docs/README.md)" != *"the proposition"* ]]
}

@test "the check passes on the page the script writes, and changes nothing" {
  "${script}"
  cp docs/README.md written

  run "${script}" --check

  [[ "${status}" -eq 0 ]]
  cmp docs/README.md written
}

@test "the check refuses a page that misses a new record, and writes nothing" {
  "${script}"
  cp docs/README.md written
  record d-delta "Delta holds" workflow

  run "${script}" --check

  [[ "${status}" -eq 1 ]]
  [[ "${output}" == *"${stale}"* ]]
  [[ "${output}" == *"Run mise run fmt:index."* ]]
  [[ "${output}" == *"+- [Delta holds](decisions/d-delta.md)"* ]]
  cmp docs/README.md written
}

@test "the check refuses a page after a record changes its title or its area" {
  "${script}"
  record b-beta "Beta holds, renamed" design

  run "${script}" --check

  [[ "${status}" -eq 1 ]]
  [[ "${output}" == *"${stale}"* ]]

  "${script}"
  record b-beta "Beta holds, renamed" writing

  run "${script}" --check

  [[ "${status}" -eq 1 ]]
  [[ "${output}" == *"+## How a record is written"* ]]
}

@test "the check refuses a missing page" {
  run "${script}" --check

  [[ "${status}" -eq 1 ]]
  [[ "${output}" == *"${stale}"* ]]
}

@test "a record with no area line cannot be listed" {
  cat >docs/decisions/e-bare.md <<'RECORD'
# Bare

## Decision Outcome

**Reach:** `x`

It is decided.
RECORD

  run "${script}"

  [[ "${status}" -eq 2 ]]
  [[ "${output}" == *"names no area"* ]]
  [[ "${output}" == *"mise run lint:records"* ]]
  [[ ! -e docs/README.md ]]
}

@test "an area outside the list cannot be listed" {
  record e-bugs "Bugs hold" bugs

  run "${script}"

  [[ "${status}" -eq 2 ]]
  [[ "${output}" == *"bugs"* ]]
  [[ ! -e docs/README.md ]]
}

@test "an unknown option is refused" {
  run "${script}" --write

  [[ "${status}" -eq 2 ]]
  [[ "${output}" == *"unknown option: --write"* ]]
}
