# scripts/check-wiring.sh against the uses it refuses.
: "${BATS_TEST_DIRNAME:?set by bats}"

setup() {
  # shellcheck source=scripts/test/fixture.sh
  . "${BATS_TEST_DIRNAME}/fixture.sh"
  script="${BATS_TEST_DIRNAME}/../check-wiring.sh"
  make_repo
  undrawn="A part may use only a part that the parts diagram draws an arrow to"
  unused="An arrow between two crates is refused when the manifest does not use it"
  no_part_crate="A crate named purba-<part> is refused when <part> is no part of the diagram"
  call="A call in the sequence of one run must go along an arrow of the parts diagram"
  answer="An answer in the sequence of one run must go back along an arrow of the parts diagram"
  no_part_end="Each end of a message in the sequence of one run must be a part, an actor or _main"

  # Alpha has no crate, so its arrows are not read against a manifest.
  landscape '  beta ->> gamma: a path' '  gamma -->> beta: a tree'
  printf '[workspace]\nmembers = ["crates/*"]\nresolver = "2"\n' >Cargo.toml
  crate purba
  crate beta 'purba-gamma = { path = "../gamma" }'
  crate gamma
}

# The landscape, with each argument as one more message of the sequence.
landscape() {
  {
    cat <<'LANDSCAPE'
# purba's landscape

## The parts and their dependencies

- **alpha.** Starts the run.
- **beta.** Reads the tests.
- **gamma.** Parses the source.

```mermaid
flowchart TD
  alpha -- "a query" --> beta
  alpha -- "a module" --> gamma
  beta -- "an AST" --> gamma
```

```mermaid
sequenceDiagram
  actor tester
  participant main as _main
  participant alpha
  participant beta
  participant gamma
  tester ->> main: purba test_a.py
  main ->> alpha: the arguments
  alpha ->> beta: the query
  beta -->> alpha: the tests
LANDSCAPE
    printf '%s\n' "$@"
    cat <<'LANDSCAPE'
  alpha -->> main: exit code 0
  main -->> tester: the process exits 0
```

## The invariants

- **the diagram is the only wiring.** No part uses an interface the diagram does not draw.
LANDSCAPE
  } >LANDSCAPE.md
}

# A crate under crates/<directory>, its package named purba-<directory>, or purba
# for the directory purba. Each further argument is one line of its manifest.
crate() {
  local directory=$1 name=purba-$1
  [[ "$1" != purba ]] || name=purba
  shift
  mkdir -p "crates/${directory}/src"
  : >"crates/${directory}/src/lib.rs"
  {
    printf '[package]\nname = "%s"\nversion = "0.0.0"\nedition = "2021"\n\n' "${name}"
    printf '[dependencies]\n'
    printf '%s\n' "$@"
  } >"crates/${directory}/Cargo.toml"
}

@test "uses and messages along drawn arrows pass" {
  run "${script}"

  [[ "${status}" -eq 0 ]]
  [[ -z "${output}" ]]
}

@test "a use the diagram does not draw is refused" {
  sed -i '/beta -- "an AST" --> gamma/d' LANDSCAPE.md

  run "${script}"

  [[ "${status}" -eq 1 ]]
  [[ "${output}" == *"${undrawn}"* ]]
  [[ "${output}" != *"each-part-is-a-crate.md"* ]]
  [[ "${output}" == *"  beta uses gamma, in crates/beta/Cargo.toml"* ]]
}

@test "a dev-dependency is a use" {
  sed -i '/beta -- "an AST" --> gamma/d' LANDSCAPE.md
  crate beta '' '[dev-dependencies]' 'purba-gamma = { path = "../gamma" }'

  run "${script}"

  [[ "${status}" -eq 1 ]]
  [[ "${output}" == *"  beta uses gamma, in crates/beta/Cargo.toml"* ]]
}

@test "an arrow between two crates that the manifest does not use is refused" {
  crate beta

  run "${script}"

  [[ "${status}" -eq 1 ]]
  [[ "${output}" == *"${unused}"* ]]
  [[ "${output}" == *"  beta --> gamma, and crates/beta/Cargo.toml names no purba-gamma"* ]]
}

@test "a crate named for no part is refused" {
  crate other

  run "${script}"

  [[ "${status}" -eq 1 ]]
  [[ "${output}" == *"${no_part_crate}"* ]]
  [[ "${output}" == *"  purba-other, in crates/other/Cargo.toml"* ]]
}

@test "the crate purba is not read" {
  crate purba 'purba-beta = { path = "../beta" }'

  run "${script}"

  [[ "${status}" -eq 0 ]]
  [[ -z "${output}" ]]
}

@test "a call along no drawn arrow is refused" {
  landscape '  gamma ->> beta: a tree'

  run "${script}"

  [[ "${status}" -eq 1 ]]
  [[ "${output}" == *"${call}"* ]]
  [[ "${output}" == *"  gamma ->> beta: a tree"* ]]
  [[ "${output}" != *"${answer}"* ]]
}

@test "an answer back along no drawn arrow is refused" {
  landscape '  beta -->> gamma: a path'

  run "${script}"

  [[ "${status}" -eq 1 ]]
  [[ "${output}" == *"${answer}"* ]]
  [[ "${output}" == *"  beta -->> gamma: a path"* ]]
  [[ "${output}" != *"${call}"* ]]
}

@test "a message whose end is no part is refused" {
  landscape '  beta ->> gama: a path'

  run "${script}"

  [[ "${status}" -eq 1 ]]
  [[ "${output}" == *"${no_part_end}"* ]]
  [[ "${output}" == *"  beta ->> gama: a path"* ]]
}

@test "a landscape with no section of parts cannot be read" {
  sed -i 's/^## The parts and their dependencies$/## The parts/' LANDSCAPE.md

  run "${script}"

  [[ "${status}" -eq 2 ]]
  [[ "${output}" == *'LANDSCAPE.md has no section named "The parts and their dependencies".'* ]]
}
