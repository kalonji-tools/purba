# scripts/crate-verdict.sh against the scores and the findings it refuses.
: "${BATS_TEST_DIRNAME:?set by bats}"

setup() {
  # shellcheck source=scripts/test/fixture.sh
  . "${BATS_TEST_DIRNAME}/fixture.sh"
  script="${BATS_TEST_DIRNAME}/../crate-verdict.sh"
  make_repo
  comment="https://github.com/kalonji-tools/purba/issues/960#issuecomment-6874866548"
  missing_refused="A crate that a manifest names is refused when .config/crates.toml holds no entry"
  version_refused="An entry is refused when its version differs from the version the lock holds"
  levels_refused="An entry is refused when a factor has no level from 1 to 3"
  scored_refused="An entry is refused when scored names neither a record under docs/decisions/"
  malformed_refused="A waiver is refused when it names anything except an advisory id"
  excess_refused="A waiver is refused when it names a failure its crate does not have"
  unwaived_refused="A crate is refused when it fails something its waiver does not name"
  out_refused="A crate is refused when a check fails or its relief does not exceed its pain"
  gate_refused="A finding of the advisory or the source check is refused when it names no crate"

  # Each graph answers by its key: `workspace` for the root, and the directory
  # of a cargo script for that script, whose .cargo/<directory>.deny.toml names it.
  fake cargo <<'FAKE'
key=workspace
while [[ $# -gt 0 ]]; do
  [[ "$1" != --manifest-path ]] || { key=${2%/*}; key=${key##*/}; }
  shift
done
cat "${BATS_TEST_TMPDIR}/metadata.${key}"
FAKE
  fake cargo-deny <<'FAKE'
key=workspace
while [[ $# -gt 0 ]]; do
  [[ "$1" != --config ]] || { key=${2##*/}; key=${key%.deny.toml}; }
  shift
done
cat "${BATS_TEST_TMPDIR}/deny.${key}" >&2
grep -q '"severity":"error"' "${BATS_TEST_TMPDIR}/deny.${key}" && exit 1
exit 0
FAKE

  metadata workspace serde@1.0.0
  deny workspace
  entry serde
}

# The `cargo metadata` of one graph, whose one member names each crate given as
# <crate>@<version>, <crate>@<version>@git, or <crate>@path for a crate of purba.
metadata() {
  local graph=$1
  shift
  jq -n --arg graph "${graph}" '
    "registry+https://github.com/rust-lang/crates.io-index" as $index
    | "path+file:///\($graph)#0.1.0" as $member
    | [$ARGS.positional[] | split("@")
      | {name: .[0], version: (.[1] | sub("^path$"; "0.1.0")),
        source: (if .[1] == "path" then null
          elif .[2] == "git" then "git+https://example.invalid/\(.[0])"
          else $index end)}
      | .id = "\(.source // "path")#\(.name)@\(.version)"] as $named
    | {
      workspace_members: [$member],
      packages: ([{id: $member, name: $graph, version: "0.1.0", source: null,
          dependencies: [$named[] | {name, source, kind: null}]}]
        + [$named[] | . + {dependencies: []}]),
      resolve: {nodes: [{id: $member,
        deps: [$named[] | {name: (.name | gsub("-"; "_")), pkg: .id}]}]}
    }' --args "$@" >"${BATS_TEST_TMPDIR}/metadata.${graph}"
}

# What `cargo-deny --format json` writes for one graph: each finding given as
# <crate>@<version>@<id>, with `source` as the id of a source finding, or
# @@<code> for a finding that names no crate. A summary line closes it.
deny() {
  local graph=$1
  shift
  jq -nc '
    ($ARGS.positional[] | split("@")
      | {type: "diagnostic", fields: ({severity: "error", message: "a finding"}
        + if .[2] == "source" then {code: "source-not-allowed"}
          elif .[0] == "" then {code: .[2]}
          else {code: "vulnerability", advisory: {id: .[2]}} end
        + if .[0] == "" then {graphs: []}
          else {graphs: [{Krate: {name: .[0], version: .[1]}}]} end)}),
    {type: "summary", fields: {}}
  ' --args "$@" >"${BATS_TEST_TMPDIR}/deny.${graph}"
}

# Appends the entry of one crate to .config/crates.toml. Each <key>=<value>
# replaces that line, and adds it when the entry holds none. A value of - drops it.
# The entry it writes passes: pain 1 against relief 2.
entry() {
  local crate=$1 line key
  shift
  local -a order=(version owners judged scored blast-area upkeep maintainers upstream-review
    tests unsafe api-churn owner-in-purba relief)
  local -A value=([version]='"1.0.0"' [owners]='["owner (user)"]' [judged]='"2026-11-14"'
    [scored]="\"${comment}\"" ["blast-area"]=1 [upkeep]=1 [maintainers]=1 ["upstream-review"]=1
    [tests]=1 [unsafe]=1 ["api-churn"]=1 ["owner-in-purba"]=1 [relief]=2)
  for line in "$@"; do
    key=${line%%=*}
    [[ -v "value[${key}]" ]] || order+=("${key}")
    value[${key}]=${line#*=}
  done
  mkdir -p .config
  {
    printf '[%s]\n' "${crate}"
    for key in "${order[@]}"; do
      [[ "${value[${key}]}" == - ]] || printf '%s = %s\n' "${key}" "${value[${key}]}"
    done
    printf '\n'
  } >>.config/crates.toml
}

# A tracked cargo script in scripts/<name>/, with its lock beside it and its
# .cargo/<name>.deny.toml.
cargo_script() {
  mkdir -p "scripts/$1"
  printf 'fn main() {}\n' >"scripts/$1/$1.rs"
  : >"scripts/$1/Cargo.lock"
  mkdir -p .cargo
  : >".cargo/$1.deny.toml"
  git add "scripts/$1" ".cargo/$1.deny.toml"
}

@test "a crate whose relief exceeds its pain passes, and prints nothing" {
  run "${script}"

  [[ "${status}" -eq 0 ]]
  [[ -z "${output}" ]]
}

@test "a crate a manifest names with no entry is refused" {
  metadata workspace serde@1.0.0 libc@0.2.0

  run "${script}"

  [[ "${status}" -eq 1 ]]
  [[ "${output}" == *"${missing_refused}"* ]]
  [[ "${output}" == *"  libc 0.2.0, in the workspace"* ]]
  [[ "${output}" != *"serde 1.0.0, in"* ]]
}

@test "a crate of purba needs no entry" {
  metadata workspace serde@1.0.0 purba-parsing@path

  run "${script}"

  [[ "${status}" -eq 0 ]]
  [[ -z "${output}" ]]
}

@test "relief equal to pain is refused, and the pain is named" {
  : >.config/crates.toml
  entry serde blast-area=2 upkeep=2 maintainers=2 upstream-review=2 tests=2 unsafe=2 \
    api-churn=2 owner-in-purba=2

  run "${script}"

  [[ "${status}" -eq 1 ]]
  [[ "${output}" == *"${out_refused}"* ]]
  [[ "${output}" == *"  serde: pain 2.00, relief 2, fails score"* ]]
}

@test "relief above a pain that is not whole passes" {
  : >.config/crates.toml
  entry serde blast-area=3 upkeep=2 relief=3

  run "${script}"

  [[ "${status}" -eq 0 ]]
}

@test "a crate an advisory names is refused, and the advisory is named" {
  deny workspace serde@1.0.0@RUSTSEC-2026-0001

  run "${script}"

  [[ "${status}" -eq 1 ]]
  [[ "${output}" == *"${out_refused}"* ]]
  [[ "${output}" == *"  serde: pain 1.00, relief 2, fails RUSTSEC-2026-0001"* ]]
}

@test "a crate from a git repository is refused" {
  metadata workspace serde@1.0.0@git
  deny workspace serde@1.0.0@source

  run "${script}"

  [[ "${status}" -eq 1 ]]
  [[ "${output}" == *"${out_refused}"* ]]
  [[ "${output}" == *"  serde: pain 1.00, relief 2, fails source"* ]]
}

@test "an advisory on another version of a named crate reaches the gate" {
  deny workspace serde@0.9.0@RUSTSEC-2026-0001

  run "${script}"

  [[ "${status}" -eq 1 ]]
  [[ "${output}" == *"${gate_refused}"* ]]
  [[ "${output}" == *"  serde 0.9.0, in the workspace: RUSTSEC-2026-0001"* ]]
  [[ "${output}" != *"${out_refused}"* ]]
}

@test "a waiver lets in each failure it names" {
  : >.config/crates.toml
  entry serde relief=1 'waiver.fails=["score", "RUSTSEC-2026-0001", "source"]' \
    'waiver.why="nothing else parses it"' \
    'waiver.issue="https://github.com/kalonji-tools/purba/issues/1"'
  deny workspace serde@1.0.0@RUSTSEC-2026-0001 serde@1.0.0@source

  run "${script}"

  [[ "${status}" -eq 0 ]]
  [[ -z "${output}" ]]
}

@test "a waiver that names a failure its crate does not have is refused" {
  : >.config/crates.toml
  entry serde 'waiver.fails=["score"]' 'waiver.why="why"' \
    'waiver.issue="https://github.com/kalonji-tools/purba/issues/1"'

  run "${script}"

  [[ "${status}" -eq 1 ]]
  [[ "${output}" == *"${excess_refused}"* ]]
  [[ "${output}" == *"  serde: score"* ]]
}

@test "a crate that fails what its waiver does not name is refused" {
  : >.config/crates.toml
  entry serde relief=1 'waiver.fails=["score"]' 'waiver.why="why"' \
    'waiver.issue="https://github.com/kalonji-tools/purba/issues/1"'
  deny workspace serde@1.0.0@RUSTSEC-2026-0001

  run "${script}"

  [[ "${status}" -eq 1 ]]
  [[ "${output}" == *"${unwaived_refused}"* ]]
  [[ "${output}" == *"  serde: RUSTSEC-2026-0001"* ]]
  [[ "${output}" != *"${out_refused}"* ]]
}

@test "a waiver that names another kind of failure is refused" {
  : >.config/crates.toml
  entry serde 'waiver.fails=["licence"]' 'waiver.why="why"' \
    'waiver.issue="https://github.com/kalonji-tools/purba/issues/1"'

  run "${script}"

  [[ "${status}" -eq 1 ]]
  [[ "${output}" == *"${malformed_refused}"* ]]
  [[ "${output}" == *"  serde: licence"* ]]
}

@test "a waiver with no why and no issue is refused" {
  : >.config/crates.toml
  entry serde relief=1 'waiver.fails=["score"]'

  run "${script}"

  [[ "${status}" -eq 1 ]]
  [[ "${output}" == *"${malformed_refused}"* ]]
  [[ "${output}" == *"  serde: no why"* ]]
  [[ "${output}" == *"  serde: no issue"* ]]
}

@test "an entry with no scored is refused" {
  : >.config/crates.toml
  entry serde scored=-

  run "${script}"

  [[ "${status}" -eq 1 ]]
  [[ "${output}" == *"${scored_refused}"* ]]
  [[ "${output}" == *"  serde: no scored"* ]]
}

@test "a scored outside docs/decisions/ is refused" {
  : >.config/crates.toml
  entry serde 'scored="docs/serde.md"'
  mkdir docs
  : >docs/serde.md
  git add docs

  run "${script}"

  [[ "${status}" -eq 1 ]]
  [[ "${output}" == *"${scored_refused}"* ]]
  [[ "${output}" == *"  serde: docs/serde.md"* ]]
}

@test "a scored record that git does not track is refused" {
  : >.config/crates.toml
  entry serde 'scored="docs/decisions/serde.md"'

  run "${script}"

  [[ "${status}" -eq 1 ]]
  [[ "${output}" == *"  serde: docs/decisions/serde.md"* ]]
}

@test "a scored record under docs/decisions/ passes" {
  : >.config/crates.toml
  entry serde 'scored="docs/decisions/serde.md"'
  mkdir -p docs/decisions
  : >docs/decisions/serde.md
  git add docs

  run "${script}"

  [[ "${status}" -eq 0 ]]
}

@test "an entry whose version differs from the lock is refused" {
  metadata workspace serde@1.1.0

  run "${script}"

  [[ "${status}" -eq 1 ]]
  [[ "${output}" == *"${version_refused}"* ]]
  [[ "${output}" == *"  serde: the entry judges 1.0.0, and the workspace locks 1.1.0"* ]]
}

@test "a factor with no level from 1 to 3 is refused" {
  : >.config/crates.toml
  entry serde unsafe=4 upkeep=-

  run "${script}"

  [[ "${status}" -eq 1 ]]
  [[ "${output}" == *"${levels_refused}"* ]]
  [[ "${output}" == *"  serde: unsafe is 4"* ]]
  [[ "${output}" == *"  serde: upkeep has no level"* ]]
  [[ "${output}" != *"${out_refused}"* ]]
}

@test "a crate no manifest names that fails a check is refused" {
  deny workspace zip@8.6.0@RUSTSEC-2026-0367

  run "${script}"

  [[ "${status}" -eq 1 ]]
  [[ "${output}" == *"${gate_refused}"* ]]
  [[ "${output}" == *"  zip 8.6.0, in the workspace: RUSTSEC-2026-0367"* ]]
}

@test "a finding that names no crate is refused" {
  deny workspace @@advisory-not-detected

  run "${script}"

  [[ "${status}" -eq 1 ]]
  [[ "${output}" == *"${gate_refused}"* ]]
  [[ "${output}" == *"  the workspace: advisory-not-detected"* ]]
}

@test "a cargo script is judged beside its lock" {
  cargo_script tool
  metadata tool pulldown-cmark@0.13.6
  deny tool

  run "${script}"

  [[ "${status}" -eq 1 ]]
  [[ "${output}" == *"${missing_refused}"* ]]
  [[ "${output}" == *"  pulldown-cmark 0.13.6, in scripts/tool/tool.rs"* ]]
  asked=$(calls cargo)
  [[ "${asked}" == *"-Zscript"*"--manifest-path scripts/tool/tool.rs"* ]]
  top=$(pwd -P)
  [[ "${asked}" == *"resolver.lockfile-path=\"${top}/scripts/tool/Cargo.lock\""* ]]
  asked=$(calls cargo-deny)
  [[ "${asked}" == *"--config .cargo/tool.deny.toml"* ]]
  [[ "${asked%%$'\n'*}" != *"--config"* ]]
}

@test "a cargo-deny that writes no summary stops the check" {
  printf '{"type":"log","fields":{"level":"ERROR","message":"no database"}}\n' \
    >"${BATS_TEST_TMPDIR}/deny.workspace"

  run "${script}"

  [[ "${status}" -eq 2 ]]
  [[ "${output}" == *"no database"* ]]
}
