# scripts/check-readers.sh against the bindings it refuses.
: "${BATS_TEST_DIRNAME:?set by bats}"

setup() {
  # shellcheck source=scripts/test/fixture.sh
  . "${BATS_TEST_DIRNAME}/fixture.sh"
  script="${BATS_TEST_DIRNAME}/../check-readers.sh"
  make_repo
  unbound="A tracked file is refused when it reaches no actor"
  off_roster="A name in .readers is refused when it is neither an actor nor a macro"
  shadow="A macro is refused when its name is an actor's"
  roster architect coder toolsmith technical-writer
  readers '.readers  architect' 'docs/**  architect' 'chorestart  toolsmith'
}

# The roster as the actor record writes it, one row for each slug.
roster() {
  local slug
  mkdir -p docs/decisions
  {
    printf '| actor | mindset |\n|---|---|\n'
    for slug in "$@"; do
      printf '| [**%s**](../../CONTEXT.md#%s) | a mindset |\n' "${slug//-/ }" "${slug}"
    done
  } >docs/decisions/an-actor-is-what-it-does-not-what-it-is.md
  git add docs/decisions
}

# .readers is written whole, one argument to a line.
readers() {
  printf '%s\n' "$@" >.readers
  git add .readers
}

track() {
  local path
  for path in "$@"; do
    mkdir -p "$(dirname "${path}")"
    : >"${path}"
    git add -- "${path}"
  done
}

@test "a tree whose every file reaches an actor passes" {
  run "${script}"

  [[ "${status}" -eq 0 ]]
  [[ -z "${output}" ]]
}

@test "a tracked file that reaches no actor is refused, and named" {
  track src/lib.rs

  run "${script}"

  [[ "${status}" -eq 1 ]]
  [[ "${output}" == *"${unbound}, because every artifact has a named reader"* ]]
  [[ "${output}" != *"a-location-inherits-its-readers.md"* ]]
  [[ "${output}" == *"  src/lib.rs"* ]]
  [[ "${output}" != *"  chorestart"* ]]
}

@test "a directory that binds nothing passes its ancestor's actors down" {
  readers '.readers  architect' 'docs/**  architect' 'chorestart  toolsmith' 'src/**  coder'
  track src/config/deep/pyproject.toml

  run "${script}"

  [[ "${status}" -eq 0 ]]
}

@test "a file reached only through a macro passes" {
  readers '[attr]builders  coder toolsmith' '.readers  architect' 'docs/**  architect' \
    'chorestart  builders'

  run "${script}"

  [[ "${status}" -eq 0 ]]
  [[ -z "${output}" ]]
}

@test "an actor unset below its binding no longer reaches the file" {
  readers '.readers  architect' 'docs/**  architect' 'chorestart  toolsmith' 'src/**  coder' \
    'src/vendor/**  -coder'
  track src/lib.rs src/vendor/lib.rs

  run "${script}"

  [[ "${status}" -eq 1 ]]
  [[ "${output}" == *"  src/vendor/lib.rs"* ]]
  [[ "${output}" != *"  src/lib.rs"* ]]
}

@test "a name off the roster is refused, even on a pattern that matches nothing" {
  readers '.readers  architect' 'docs/**  architect' 'chorestart  toolsmith' \
    'nowhere/**  architekt'

  run "${script}"

  [[ "${status}" -eq 1 ]]
  [[ "${output}" == *"${off_roster}"* ]]
  [[ "${output}" == *"because a reader is an actor and the roster names every actor"* ]]
  [[ "${output}" != *"an-actor-is-what-it-does-not-what-it-is.md"* ]]
  [[ "${output}" == *"  architekt"* ]]
}

@test "a member of a macro off the roster is refused" {
  readers '[attr]builders  coder toolsmyth' '.readers  architect' 'docs/**  architect' \
    'chorestart  builders'

  run "${script}"

  [[ "${status}" -eq 1 ]]
  [[ "${output}" == *"${off_roster}"* ]]
  [[ "${output}" == *"  toolsmyth"* ]]
}

@test "a name is read without the prefix that unsets it" {
  readers '.readers  architect' 'docs/**  architect' 'chorestart  toolsmith coder' \
    'chorestart  -coder !technical-writer'

  run "${script}"

  [[ "${status}" -eq 0 ]]
  [[ -z "${output}" ]]
}

@test "a macro named after an actor is refused" {
  readers '[attr]coder  architect' '.readers  architect' 'docs/**  architect' \
    'chorestart  toolsmith'

  run "${script}"

  [[ "${status}" -eq 1 ]]
  [[ "${output}" == *"${shadow}"* ]]
  [[ "${output}" == *"  coder"* ]]
}

@test "the roster is read from the record, so a new actor needs no change here" {
  roster architect coder toolsmith technical-writer packager
  readers '.readers  architect' 'docs/**  architect' 'chorestart  packager'

  run "${script}"

  [[ "${status}" -eq 0 ]]
}

@test "a linked name outside the roster's table is not an actor" {
  printf '\n| actor | what it cost |\n|---|---|\n%s\n' \
    '| [**intruder**](../../CONTEXT.md#intruder) | a lot |' \
    >>docs/decisions/an-actor-is-what-it-does-not-what-it-is.md
  readers '.readers  architect' 'docs/**  architect' 'chorestart  intruder'

  run "${script}"

  [[ "${status}" -eq 1 ]]
  [[ "${output}" == *"${off_roster}"* ]]
  [[ "${output}" == *"  intruder"* ]]
}

@test "an untracked file is not checked" {
  : >scratch.txt

  run "${script}"

  [[ "${status}" -eq 0 ]]
}

@test "the repository's own attributes do not change what .readers resolves" {
  printf '* -toolsmith -architect\n' >.git/info/attributes
  printf '* -toolsmith -architect\n' >.gitattributes
  git add .gitattributes
  readers '.readers  architect' 'docs/**  architect' 'chorestart  toolsmith' \
    '.gitattributes  toolsmith'

  run "${script}"

  [[ "${status}" -eq 0 ]]
  [[ -z "${output}" ]]
}

@test "a missing .readers is an error, not a refusal" {
  git rm --quiet --cached .readers
  rm .readers

  run "${script}"

  [[ "${status}" -eq 2 ]]
  [[ "${output}" == *".readers cannot be read"* ]]
}

@test "a roster with no actor is an error, not a refusal" {
  printf 'no table here\n' >docs/decisions/an-actor-is-what-it-does-not-what-it-is.md

  run "${script}"

  [[ "${status}" -eq 2 ]]
  [[ "${output}" == *"the roster cannot be read"* ]]
}
