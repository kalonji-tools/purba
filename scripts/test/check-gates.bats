# scripts/check-gates.sh against the marks it refuses.
: "${BATS_TEST_DIRNAME:?set by bats}"

setup() {
  # shellcheck source=scripts/test/fixture.sh
  . "${BATS_TEST_DIRNAME}/fixture.sh"
  script="${BATS_TEST_DIRNAME}/../check-gates.sh"
  make_repo
  unmarked="A tracked path is refused when .config/gates does not mark it set or unset for \
each gate the file names"
  why="because a gate that skips must know whether the path starts it"
  gates '.config/gates  -build' 'chorestart  build'
}

# .config/gates is written whole, one argument to a line.
gates() {
  mkdir -p .config
  printf '%s\n' "$@" >.config/gates
  git add .config/gates
}

track() {
  local path
  for path in "$@"; do
    mkdir -p "$(dirname "${path}")"
    : >"${path}"
    git add -- "${path}"
  done
}

@test "a tree whose every path is marked passes" {
  run "${script}"

  [[ "${status}" -eq 0 ]]
  [[ -z "${output}" ]]
}

@test "a tracked path no line marks is refused, and named with the gate" {
  track build.rs

  run "${script}"

  [[ "${status}" -eq 1 ]]
  [[ "${output}" == *"${unmarked}, ${why}"* ]]
  [[ "${output}" == *"  build: build.rs"* ]]
  [[ "${output}" != *"chorestart"* ]]
}

@test "a new file in a directory whose files are marked one by one is refused" {
  gates '.config/gates  -build' 'chorestart  build' '.config/mise.toml  build'
  track .config/mise.toml .config/mise/conf.d/x.toml

  run "${script}"

  [[ "${status}" -eq 1 ]]
  [[ "${output}" == *"  build: .config/mise/conf.d/x.toml"* ]]
  [[ "${output}" != *"  build: .config/mise.toml"* ]]
}

@test "a later line wins over an earlier one" {
  gates '.config/gates  -build' 'chorestart  build' 'docs/**  -build' 'docs/x.md  build'
  track docs/x.md docs/y.md

  run "${script}"

  [[ "${status}" -eq 0 ]]
  [[ -z "${output}" ]]
}

@test "a path whose mark is removed with ! is refused" {
  gates '.config/gates  -build' 'chorestart  build' 'chorestart  !build'

  run "${script}"

  [[ "${status}" -eq 1 ]]
  [[ "${output}" == *"  build: chorestart"* ]]
}

@test "a path marked with a value is refused" {
  gates '.config/gates  -build' 'chorestart  build=yes'

  run "${script}"

  [[ "${status}" -eq 1 ]]
  [[ "${output}" == *"  build: chorestart"* ]]
}

@test "each gate the file names is checked, so a new gate needs no change here" {
  gates '.config/gates  -build -scripts' 'chorestart  build'

  run "${script}"

  [[ "${status}" -eq 1 ]]
  [[ "${output}" == *"  scripts: chorestart"* ]]
  [[ "${output}" != *"  build: chorestart"* ]]
}

@test "a gate is read without the prefix and the value that mark it" {
  gates '.config/gates  -build -scripts' 'chorestart  build=yes scripts' 'chorestart  build'

  run "${script}"

  [[ "${status}" -eq 0 ]]
  [[ -z "${output}" ]]
}

@test "a comment and a blank line name no gate" {
  gates '# the gates: build, quality' '' '.config/gates  -build' 'chorestart  build'

  run "${script}"

  [[ "${status}" -eq 0 ]]
  [[ -z "${output}" ]]
}

@test "an untracked file is not checked" {
  : >build.rs

  run "${script}"

  [[ "${status}" -eq 0 ]]
}

@test "the repository's own attributes do not change what .config/gates resolves" {
  printf '* -build\n' >.git/info/attributes
  printf '* -build\n' >.gitattributes
  git add .gitattributes

  run "${script}"

  [[ "${status}" -eq 1 ]]
  [[ "${output}" == *"  build: .gitattributes"* ]]
}

@test "a missing .config/gates is an error, not a refusal" {
  git rm --quiet --cached .config/gates
  rm .config/gates

  run "${script}"

  [[ "${status}" -eq 2 ]]
  [[ "${output}" == *".config/gates cannot be read"* ]]
}

@test "a .config/gates that names no gate is an error, not a refusal" {
  gates '# nothing here'

  run "${script}"

  [[ "${status}" -eq 2 ]]
  [[ "${output}" == *".config/gates names no gate"* ]]
}
