# scripts/attributes.sh against the patterns it matches.
: "${BATS_TEST_DIRNAME:?set by bats}"
bats_require_minimum_version 1.5.0

setup() {
  # shellcheck source=scripts/test/fixture.sh
  . "${BATS_TEST_DIRNAME}/fixture.sh"
  make_repo
  # shellcheck source=scripts/report.sh
  . "${BATS_TEST_DIRNAME}/../report.sh"
  # shellcheck source=scripts/attributes.sh
  . "${BATS_TEST_DIRNAME}/../attributes.sh"
  scratch="${BATS_TEST_TMPDIR}/scratch"
  attributes="${BATS_TEST_TMPDIR}/attributes"
  mkdir "${scratch}"
}

# The attributes file is written whole, one argument to a line.
patterns() {
  printf '%s\n' "$@" >"${attributes}"
}

# The paths attr_states reads, one argument to a path.
paths() {
  printf '%s\0' "$@" >"${BATS_TEST_TMPDIR}/paths"
}

# attr_states for each <attr>, into `got`, one line for each path, attribute and
# state, the three joined by a space.
states() {
  attr_states "${scratch}" "${attributes}" "$@" <"${BATS_TEST_TMPDIR}/paths" \
    >"${BATS_TEST_TMPDIR}/states"
  tr '\0' '\n' <"${BATS_TEST_TMPDIR}/states" >"${BATS_TEST_TMPDIR}/lines"
  got=$(paste -d ' ' - - - <"${BATS_TEST_TMPDIR}/lines")
}

@test "each path gets each named attribute and its state" {
  patterns '*.rs  build -scripts lang=rust'
  paths src/lib.rs README.md

  states build scripts lang docs

  [[ "${got}" == "src/lib.rs build set
src/lib.rs scripts unset
src/lib.rs lang rust
src/lib.rs docs unspecified
README.md build unspecified
README.md scripts unspecified
README.md lang unspecified
README.md docs unspecified" ]]
}

@test "with no attribute named, a path gets only the attributes the file gives it" {
  patterns 'src/**  build' '*.md  -build'
  paths src/lib.rs README.md notes.txt

  states

  [[ "${got}" == "src/lib.rs build set
README.md build unset" ]]
}

@test "the repository's own attributes do not change a match" {
  patterns 'chorestart  -build'
  printf '* build docs\n' >.git/info/attributes
  printf '* build docs\n' >.gitattributes
  git add .gitattributes
  paths chorestart

  states build docs

  [[ "${got}" == "chorestart build unset
chorestart docs unspecified" ]]
}

@test "the attributes of a template do not change a match" {
  patterns 'chorestart  build'
  mkdir -p "${BATS_TEST_TMPDIR}/template/info"
  printf '* -build docs\n' >"${BATS_TEST_TMPDIR}/template/info/attributes"
  export GIT_TEMPLATE_DIR="${BATS_TEST_TMPDIR}/template"
  paths chorestart

  states build docs

  [[ "${got}" == "chorestart build set
chorestart docs unspecified" ]]
}

@test "each name after a pattern or a macro is printed once, with no sign and no value" {
  patterns '# a comment names nothing' '' '[attr]group  owner reader' \
    '*.rs  build -scripts !lang owner=me' 'src/**  build'

  run attr_names "${attributes}"

  [[ "${status}" -eq 0 ]]
  [[ "${output}" == "build
lang
owner
reader
scripts" ]]
}

@test "a git that cannot make the empty repository exits 2" {
  patterns 'chorestart  build'
  paths chorestart
  git_fails_on init

  run --separate-stderr attr_states "${scratch}" "${attributes}" build \
    <"${BATS_TEST_TMPDIR}/paths"

  [[ "${status}" -eq 2 ]]
  [[ "${stderr}" == *"git cannot make the empty repository a pattern is matched in."* ]]
}

@test "a git that cannot match exits 2, and names the file" {
  patterns 'chorestart  build'
  paths chorestart
  git_fails_on check-attr

  run --separate-stderr attr_states "${scratch}" "${attributes}" build \
    <"${BATS_TEST_TMPDIR}/paths"

  [[ "${status}" -eq 2 ]]
  [[ "${stderr}" == *"git cannot match the patterns in ${attributes}."* ]]
}
