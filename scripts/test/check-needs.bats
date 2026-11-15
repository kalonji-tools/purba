# scripts/check-needs.sh against the need lines and the shortlisted crates it refuses.
: "${BATS_TEST_DIRNAME:?set by bats}"

setup() {
  # shellcheck source=scripts/test/fixture.sh
  . "${BATS_TEST_DIRNAME}/fixture.sh"
  script="${BATS_TEST_DIRNAME}/../check-needs.sh"
  make_repo
  part_refused="A line of .config/needs.toml is refused when its part is not in the order of design"
  words_refused="A line of .config/needs.toml is refused when it has no words"
  stated_refused="A line of .config/needs.toml is refused when it has no stated link"
  open_refused="A line of .config/needs.toml is refused when it has no issue, no declined list"
  orphan_refused="An entry of .config/shortlists.toml is refused when no line of .config/needs.toml"

  # gamma is a part of the landscape, and it is not in the order of design.
  cat >LANDSCAPE.md <<'LANDSCAPE'
# purba's landscape

## The parts and their dependencies

- **alpha.** Starts the run.
- **gamma.** Parses the source.

## The order of design

1. **alpha.**
2. **the beta/delta seam.**

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

@test "a need whose part is not in the order of design is refused" {
  need find-files part='"gamma"' declined='[]'
  need parse-query part='"omega"' declined='[]'

  run "${script}"

  [[ "${status}" -eq 1 ]]
  [[ "${output}" == *"${part_refused}"* ]]
  [[ "${output}" == *"  find-files: part gamma"* ]]
  [[ "${output}" == *"  parse-query: part omega"* ]]
  [[ "${output}" != *"  read-flags:"* ]]
}

@test "a need with no words, or an empty list of them, is refused" {
  need find-files words=- declined='[]'
  need parse-query words='[]' declined='[]'

  run "${script}"

  [[ "${status}" -eq 1 ]]
  [[ "${output}" == *"${words_refused}"* ]]
  [[ "${output}" == *"  find-files"* ]]
  [[ "${output}" == *"  parse-query"* ]]
}

@test "a need with no stated link, or an empty one, is refused" {
  need find-files stated=- declined='[]'
  need parse-query stated='""' declined='[]'

  run "${script}"

  [[ "${status}" -eq 1 ]]
  [[ "${output}" == *"${stated_refused}"* ]]
  [[ "${output}" == *"  find-files"* ]]
  [[ "${output}" == *"  parse-query"* ]]
}

@test "a need with no issue, no declined list and no shortlisted crate is refused" {
  need find-files

  run "${script}"

  [[ "${status}" -eq 1 ]]
  [[ "${output}" == *"${open_refused}"* ]]
  [[ "${output}" == *"  find-files"* ]]
  [[ "${output}" != *"  read-flags"* ]]
}

@test "a shortlisted crate whose need has no line is refused" {
  shortlisted find-files walkdir

  run "${script}"

  [[ "${status}" -eq 1 ]]
  [[ "${output}" == *"${orphan_refused}"* ]]
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

@test "a landscape with no order of design stops the check" {
  printf '# purba'"'"'s landscape\n' >LANDSCAPE.md

  run "${script}"

  [[ "${status}" -eq 2 ]]
  [[ "${output}" == *'LANDSCAPE.md has no section named "The order of design".'* ]]
}
