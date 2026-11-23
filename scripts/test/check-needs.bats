# scripts/check-needs.sh against the need lines and the shortlisted crates it refuses.
: "${BATS_TEST_DIRNAME:?set by bats}"

setup() {
  # shellcheck source=scripts/test/fixture.sh
  . "${BATS_TEST_DIRNAME}/fixture.sh"
  script="${BATS_TEST_DIRNAME}/../check-needs.sh"
  make_repo
  part_refused="A line of .config/needs.toml is refused when its part is not a part or concern \
of the landscape, because a line states a need of a part of purba. Name the part as the \
landscape names it, or remove the line."
  words_refused="A line of .config/needs.toml is refused when it has no words, because the \
forecast finds its crates on crates.io by each of them. Add a keyword that crates.io holds."
  stated_refused="A line of .config/needs.toml is refused when it has no stated link, because \
the line that states the need holds why purba has it. Link that line."
  open_refused="A line of .config/needs.toml is refused when it has no issue, no declined list \
and no shortlisted crate, because nothing would then vet its crates. Open the issue that vets \
them, and write its link in the issue field."
  orphan_refused="An entry of .config/shortlists.toml is refused when no line of \
.config/needs.toml names its need, because an entry answers its need, and an entry with no need \
answers nothing. Remove the entry, or restore the line."

  # gamma is a part of the landscape, and epsilon a concern, and neither is in the
  # order of design. zeta is in the order and in the invariants, and is neither.
  cat >LANDSCAPE.md <<'LANDSCAPE'
# purba's landscape

## The parts and their dependencies

- **alpha.** Starts the run.
- **gamma.** Parses the source.

## The cross-cutting concerns

- **the beta/delta seam.** Carries a value across.
- **epsilon.** States facts about the project.

## The order of design

1. **alpha.**
2. **the beta/delta seam.**
3. **zeta.**

## The invariants

- **zeta.** Holds in every part.

## What belongs in this file
LANDSCAPE

  mkdir -p .config
  : >.config/needs.toml
  : >.config/shortlists.toml
  need read-flags
  shortlisted read-flags clap
}

# A line of .config/needs.toml for the need <key>. Each <field>=<value> replaces
# that field, and adds it when the line holds none. A value of - drops it. The
# line it writes passes once a crate is shortlisted for it.
need() {
  local key=$1 line field joined
  shift
  local -a order=(part need words stated) fields=()
  local -A value=([part]='"alpha"' [need]='"read the flags"' [words]='["cli"]'
    [stated]='"https://example.invalid/stated"')
  for line in "$@"; do
    field=${line%%=*}
    [[ -v "value[${field}]" ]] || order+=("${field}")
    value[${field}]=${line#*=}
  done
  for field in "${order[@]}"; do
    [[ "${value[${field}]}" == - ]] || fields+=("${field} = ${value[${field}]}")
  done
  joined=$(printf '%s, ' "${fields[@]}")
  printf '%s = { %s }\n' "${key}" "${joined%, }" >>.config/needs.toml
}

# A table of .config/shortlists.toml for <crate>, under the key of <need>.
shortlisted() {
  printf '[%s.%s]\nversion = "1.0.0"\n\n' "$1" "$2" >>.config/shortlists.toml
}

@test "a need with a shortlisted crate passes, and prints nothing" {
  run "${script}"

  [[ "${status}" -eq 0 ]]
  [[ -z "${output}" ]]
}

@test "a need whose issue declined each crate passes" {
  need find-files declined='[]'

  run "${script}"

  [[ "${status}" -eq 0 ]]
}

@test "a need with an issue passes" {
  need find-files issue='"https://example.invalid/issues/1"'

  run "${script}"

  [[ "${status}" -eq 0 ]]
}

@test "a part whose name holds spaces and a slash passes" {
  need find-files part='"the beta/delta seam"' declined='[]'

  run "${script}"

  [[ "${status}" -eq 0 ]]
}

@test "a need whose part the landscape names, outside the order of design, passes" {
  need find-files part='"gamma"' declined='[]'

  run "${script}"

  [[ "${status}" -eq 0 ]]
}

@test "a need whose concern the landscape names, outside the order of design, passes" {
  need find-files part='"epsilon"' declined='[]'

  run "${script}"

  [[ "${status}" -eq 0 ]]
}

@test "a need whose part the landscape does not name is refused" {
  need parse-query part='"omega"' declined='[]'
  need sort-items part='"zeta"' declined='[]'

  run "${script}"

  [[ "${status}" -eq 1 ]]
  output_has_line "${part_refused}"
  [[ "${output}" == *"  parse-query: part omega"* ]]
  [[ "${output}" == *"  sort-items: part zeta"* ]]
  [[ "${output}" != *"  read-flags:"* ]]
}

@test "a need with no words, or an empty list of them, is refused" {
  need find-files words=- declined='[]'
  need parse-query words='[]' declined='[]'

  run "${script}"

  [[ "${status}" -eq 1 ]]
  output_has_line "${words_refused}"
  [[ "${output}" == *"  find-files"* ]]
  [[ "${output}" == *"  parse-query"* ]]
}

@test "a need with no stated link, or an empty one, is refused" {
  need find-files stated=- declined='[]'
  need parse-query stated='""' declined='[]'

  run "${script}"

  [[ "${status}" -eq 1 ]]
  output_has_line "${stated_refused}"
  [[ "${output}" == *"  find-files"* ]]
  [[ "${output}" == *"  parse-query"* ]]
}

@test "a need with no issue, no declined list and no shortlisted crate is refused" {
  need find-files

  run "${script}"

  [[ "${status}" -eq 1 ]]
  output_has_line "${open_refused}"
  [[ "${output}" == *"  find-files"* ]]
  [[ "${output}" != *"  read-flags"* ]]
}

@test "a shortlisted crate whose need has no line is refused" {
  shortlisted find-files walkdir

  run "${script}"

  [[ "${status}" -eq 1 ]]
  output_has_line "${orphan_refused}"
  [[ "${output}" == *"  find-files.walkdir"* ]]
  [[ "${output}" != *"read-flags.clap"* ]]
}

@test "no .config/needs.toml stops the check" {
  rm .config/needs.toml

  run "${script}"

  [[ "${status}" -eq 2 ]]
  [[ "${output}" == *".config/needs.toml cannot be read."* ]]
}

@test "no .config/shortlists.toml stops the check" {
  rm .config/shortlists.toml

  run "${script}"

  [[ "${status}" -eq 2 ]]
  [[ "${output}" == *".config/shortlists.toml cannot be read."* ]]
}

@test "a landscape with no parts and no concerns stops the check" {
  printf '# purba'"'"'s landscape\n' >LANDSCAPE.md

  run "${script}"

  [[ "${status}" -eq 2 ]]
  local stopped='LANDSCAPE.md has neither a section named "The parts and their dependencies"'
  stopped+=' nor one named "The cross-cutting concerns".'
  [[ "${output}" == *"${stopped}"* ]]
}

@test "a landscape with parts, and no concerns and no order of design, passes" {
  cat >LANDSCAPE.md <<'LANDSCAPE'
# purba's landscape

## The parts and their dependencies

- **alpha.** Starts the run.
LANDSCAPE

  run "${script}"

  [[ "${status}" -eq 0 ]]
}

@test "a landscape with concerns, and no parts and no order of design, passes" {
  cat >LANDSCAPE.md <<'LANDSCAPE'
# purba's landscape

## The cross-cutting concerns

- **alpha.** Carries a value across.
LANDSCAPE

  run "${script}"

  [[ "${status}" -eq 0 ]]
}
