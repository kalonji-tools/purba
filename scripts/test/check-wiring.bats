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
  off_seam="Only a part that the line of the Rust/Python seam names may name pyo3"
  concern_unnamed="A part may use the crate of a concern only when the concern's line names the \
part, because no arrow of the parts diagram draws that use. Name the part in the concern's line, \
or remove the dependency."
  concern_uses_part="The crate of a concern uses no part, because each part may use it. Remove \
the dependency."

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

## The cross-cutting concerns

- **the Rust/Python seam.** Carries Python, in alpha, gamma and delta. Beta reads it, in beta.
- **errors.** Keeps each kind of failure apart, in every part.
LANDSCAPE
  } >LANDSCAPE.md
}

# The line of errors, in the parts that $1 names, with the crate purba-errors.
errors_line() {
  sed -i "s/^- \*\*errors\.\*\* .*/- **errors.** Keeps each kind of failure apart, in $1. \
Its crate is \`purba-errors\`./" LANDSCAPE.md
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

@test "a part that the seam's line does not name is refused when it names pyo3" {
  crate beta 'purba-gamma = { path = "../gamma" }' 'pyo3 = "0.29"'
  local why="because the other parts pass data."
  local repair="Name the part in the seam's line, or remove the dependency."

  run "${script}"

  [[ "${status}" -eq 1 ]]
  [[ "${output}" == *"${off_seam}, ${why} ${repair}"* ]]
  [[ "${output}" == *"  purba-beta names pyo3, in crates/beta/Cargo.toml"* ]]
}

@test "pyo3 in the dependencies and the dev-dependencies is refused off the seam" {
  crate beta 'purba-gamma = { path = "../gamma" }' 'pyo3 = "0.29"' '' '[dev-dependencies]' \
    'pyo3 = "0.29"'

  run "${script}"

  [[ "${status}" -eq 1 ]]
  [[ "${output}" == *"  purba-beta names pyo3, in crates/beta/Cargo.toml"* ]]
}

@test "a build dependency on pyo3 is refused off the seam" {
  crate beta 'purba-gamma = { path = "../gamma" }' '' '[build-dependencies]' 'pyo3 = "0.29"'

  run "${script}"

  [[ "${status}" -eq 1 ]]
  [[ "${output}" == *"  purba-beta names pyo3, in crates/beta/Cargo.toml"* ]]
}

@test "a dev-dependency on pyo3 passes off the seam" {
  crate beta 'purba-gamma = { path = "../gamma" }' '' '[dev-dependencies]' 'pyo3 = "0.29"'

  run "${script}"

  [[ "${status}" -eq 0 ]]
  [[ -z "${output}" ]]
}

@test "a part that the seam's line names may name pyo3" {
  crate gamma 'pyo3 = "0.29"'

  run "${script}"

  [[ "${status}" -eq 0 ]]
  [[ -z "${output}" ]]
}

@test "the crate purba may name pyo3" {
  crate purba 'pyo3 = "0.29"'

  run "${script}"

  [[ "${status}" -eq 0 ]]
  [[ -z "${output}" ]]
}

@test "a landscape with no line for the seam cannot be read" {
  sed -i '/^- \*\*the Rust\/Python seam\.\*\*/d' LANDSCAPE.md

  run "${script}"

  [[ "${status}" -eq 2 ]]
  [[ "${output}" == *'LANDSCAPE.md has no line for the Rust/Python seam.'* ]]
}

@test "a landscape with no section of concerns cannot be read" {
  sed -i 's/^## The cross-cutting concerns$/## The concerns/' LANDSCAPE.md

  run "${script}"

  [[ "${status}" -eq 2 ]]
  [[ "${output}" == *'LANDSCAPE.md has no section named "The cross-cutting concerns".'* ]]
}

@test "the first part after the seam's in may name pyo3" {
  crate alpha 'purba-beta = { path = "../beta" }' 'purba-gamma = { path = "../gamma" }' \
    'pyo3 = "0.29"'

  run "${script}"

  [[ "${status}" -eq 0 ]]
  [[ -z "${output}" ]]
}

@test "a part that a concern's line names may use the concern's crate" {
  errors_line 'beta and gamma'
  crate errors
  crate beta 'purba-gamma = { path = "../gamma" }' 'purba-errors = { path = "../errors" }'
  crate gamma 'purba-errors = { path = "../errors" }'

  run "${script}"

  [[ "${status}" -eq 0 ]]
  [[ -z "${output}" ]]
}

@test "each part may use the crate of a concern in every part" {
  errors_line 'every part'
  crate errors
  crate beta 'purba-gamma = { path = "../gamma" }' 'purba-errors = { path = "../errors" }'
  crate gamma 'purba-errors = { path = "../errors" }'

  run "${script}"

  [[ "${status}" -eq 0 ]]
  [[ -z "${output}" ]]
}

@test "a part that a concern's line does not name may not use the concern's crate" {
  errors_line 'beta'
  crate errors
  crate beta 'purba-gamma = { path = "../gamma" }' 'purba-errors = { path = "../errors" }'
  crate gamma 'purba-errors = { path = "../errors" }'

  run "${script}"

  [[ "${status}" -eq 1 ]]
  [[ "${output}" == *"${concern_unnamed}"* ]]
  [[ "${output}" == *"  gamma uses purba-errors, in crates/gamma/Cargo.toml"* ]]
  [[ "${output}" != *"beta uses purba-errors"* ]]
}

@test "a concern's crate that uses a part is refused" {
  errors_line 'every part'
  crate errors 'purba-gamma = { path = "../gamma" }'

  run "${script}"

  [[ "${status}" -eq 1 ]]
  [[ "${output}" == *"${concern_uses_part}"* ]]
  [[ "${output}" == *"  purba-errors uses gamma, in crates/errors/Cargo.toml"* ]]
}

@test "a concern's crate that uses a part in its dev-dependencies is refused" {
  errors_line 'every part'
  crate errors '' '[dev-dependencies]' 'purba-gamma = { path = "../gamma" }'

  run "${script}"

  [[ "${status}" -eq 1 ]]
  [[ "${output}" == *"${concern_uses_part}"* ]]
  [[ "${output}" == *"  purba-errors uses gamma, in crates/errors/Cargo.toml"* ]]
}

@test "a concern's crate that names pyo3 is refused" {
  errors_line 'every part'
  crate errors 'pyo3 = "0.29"'

  run "${script}"

  [[ "${status}" -eq 1 ]]
  [[ "${output}" == *"${off_seam}"* ]]
  [[ "${output}" == *"  purba-errors names pyo3, in crates/errors/Cargo.toml"* ]]
  [[ "${output}" != *"${no_part_crate}"* ]]
}

@test "a concern's line that names no crate leaves a crate of the concern's name refused" {
  crate errors
  crate gamma 'purba-errors = { path = "../errors" }'

  run "${script}"

  [[ "${status}" -eq 1 ]]
  [[ "${output}" == *"${no_part_crate}"* ]]
  [[ "${output}" == *"  purba-errors, in crates/errors/Cargo.toml"* ]]
}

@test "a concern's line that lists three parts with commas names each one" {
  errors_line 'alpha, beta and gamma'
  crate errors
  crate beta 'purba-gamma = { path = "../gamma" }' 'purba-errors = { path = "../errors" }'
  crate gamma 'purba-errors = { path = "../errors" }'

  run "${script}"

  [[ "${status}" -eq 0 ]]
  [[ -z "${output}" ]]
}

@test "a concern's crate that uses a part in its build-dependencies is refused" {
  errors_line 'every part'
  crate errors '' '[build-dependencies]' 'purba-gamma = { path = "../gamma" }'

  run "${script}"

  [[ "${status}" -eq 1 ]]
  [[ "${output}" == *"${concern_uses_part}"* ]]
  [[ "${output}" == *"  purba-errors uses gamma, in crates/errors/Cargo.toml"* ]]
}

@test "a dev-dependency on a concern's crate is a use of it" {
  errors_line 'beta'
  crate errors
  crate gamma '' '[dev-dependencies]' 'purba-errors = { path = "../errors" }'

  run "${script}"

  [[ "${status}" -eq 1 ]]
  [[ "${output}" == *"${concern_unnamed}"* ]]
  [[ "${output}" == *"  gamma uses purba-errors, in crates/gamma/Cargo.toml"* ]]
}

@test "the crate sentence may follow a sentence with a link" {
  sed -i "s/^- \*\*errors\.\*\* .*/- **errors.** Keeps each kind of failure apart, in every part. \
[A failure's kind](docs\/decisions\/a.md#decision-outcome) holds its design in depth. \
Its crate is \`purba-errors\`./" LANDSCAPE.md
  crate errors
  crate gamma 'purba-errors = { path = "../errors" }'

  run "${script}"

  [[ "${status}" -eq 0 ]]
  [[ -z "${output}" ]]
}

@test "the crate of a second concern is read too" {
  errors_line 'every part'
  printf '%s\n' "- **configuration.** States facts, in beta. Its crate is \`purba-config\`." \
    >>LANDSCAPE.md
  crate config
  crate gamma 'purba-config = { path = "../config" }'

  run "${script}"

  [[ "${status}" -eq 1 ]]
  [[ "${output}" == *"${concern_unnamed}"* ]]
  [[ "${output}" == *"  gamma uses purba-config, in crates/gamma/Cargo.toml"* ]]
}
