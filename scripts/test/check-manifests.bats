# scripts/check-manifests.sh against the manifests it refuses.
: "${BATS_TEST_DIRNAME:?set by bats}"

setup() {
  # shellcheck source=scripts/test/fixture.sh
  . "${BATS_TEST_DIRNAME}/fixture.sh"
  script="${BATS_TEST_DIRNAME}/../check-manifests.sh"
  make_repo
  conflict="A package is refused when two manifests declare it"
  python="A package in pyproject.toml is refused until purba chooses its Python manager"
  mise_toml 'jq = "1"'
  cargo_toml 'serde = "1"'
  pyproject_toml '[build-system]' 'requires = []' '[project]' 'name = "fixture"'
}

# Each manifest is written whole, one argument to a line.
mise_toml() {
  {
    printf '[tools]\n'
    printf '%s\n' "$@"
  } >mise.toml
}

cargo_toml() {
  mkdir -p src
  : >src/lib.rs
  {
    printf '[package]\nname = "fixture"\nversion = "0.1.0"\nedition = "2021"\n\n'
    printf '[dependencies]\n'
    printf '%s\n' "$@"
  } >Cargo.toml
}

pyproject_toml() {
  printf '%s\n' "$@" >pyproject.toml
}

@test "manifests that declare different packages pass" {
  run "${script}"

  [[ "${status}" -eq 0 ]]
  [[ -z "${output}" ]]
}

@test "a package in mise.toml and Cargo.toml is refused" {
  cargo_toml 'jq = "1"'

  run "${script}"

  [[ "${status}" -eq 1 ]]
  [[ "${output}" == *"${conflict}"* ]]
  [[ "${output}" != *"one-manifest-declares-each-package.md"* ]]
  [[ "${output}" == *"  jq: Cargo.toml as jq, mise.toml as jq"* ]]
  [[ "${output}" != *"${python}"* ]]
}

@test "case and a run of separators do not tell two names apart" {
  mise_toml 'editorconfig-checker = "3"'
  cargo_toml 'Editorconfig_Checker = "3"'

  run "${script}"

  [[ "${status}" -eq 1 ]]
  [[ "${output}" == *"  editorconfig-checker: Cargo.toml as Editorconfig_Checker, mise.toml as"* ]]
}

@test "a mise key is read without its backend and its owner" {
  mise_toml '"aqua:jqlang/jq" = "1"' '"pipx:black" = "24"'
  cargo_toml 'jq = "1"' 'black = "1"'

  run "${script}"

  [[ "${status}" -eq 1 ]]
  [[ "${output}" == *"  jq: Cargo.toml as jq, mise.toml as aqua:jqlang/jq"* ]]
  [[ "${output}" == *"  black: Cargo.toml as black, mise.toml as pipx:black"* ]]
}

@test "a renamed crate is read by its package name" {
  cargo_toml 'json = { package = "jq", version = "1" }'

  run "${script}"

  [[ "${status}" -eq 1 ]]
  [[ "${output}" == *"  jq: Cargo.toml as jq, mise.toml as jq"* ]]
}

@test "a package that one manifest declares twice passes" {
  cargo_toml 'jq = "1"' '[dev-dependencies]' 'jq = "1"'
  mise_toml 'shfmt = "3"'

  run "${script}"

  [[ "${status}" -eq 0 ]]
  [[ -z "${output}" ]]
}

@test "a mise.toml above the repository declares nothing" {
  printf '[tools]\nshellcheck = "0"\n' >../mise.toml
  cargo_toml 'shellcheck = "0"'

  run "${script}"

  [[ "${status}" -eq 0 ]]
  [[ -z "${output}" ]]
}

@test "a Cargo.toml above the repository is not read in place of a missing one" {
  cargo_toml 'jq = "1"'
  mv Cargo.toml src ..

  run "${script}"

  [[ "${status}" -eq 2 ]]
  [[ "${output}" == *"Cargo.toml cannot be read"* ]]
}

@test "a run from a subdirectory reads the manifests at the top" {
  cargo_toml 'jq = "1"'
  mkdir -p deep/er

  cd deep/er
  run "${script}"

  [[ "${status}" -eq 1 ]]
  [[ "${output}" == *"  jq: Cargo.toml as jq, mise.toml as jq"* ]]
}

@test "a package in pyproject.toml is refused, wherever it is declared" {
  local declared
  for declared in \
    "build-system.requires|[build-system]|requires = [\"maturin\"]" \
    "project.dependencies|[project]|dependencies = [\"pytest\"]" \
    "project.optional-dependencies|[project.optional-dependencies]|cli = [\"click\"]" \
    "dependency-groups|[dependency-groups]|dev = [\"pytest\"]"; do
    IFS='|' read -r key table line <<<"${declared}"
    pyproject_toml "${table}" "${line}"

    run "${script}"

    [[ "${status}" -eq 1 ]]
    [[ "${output}" == *"${python}"* ]]
    [[ "${output}" == *"  ${key}"* ]]
    [[ "${output}" != *"${conflict}"* ]]
  done
}

@test "a pyproject.toml whose lists are empty, or absent, declares nothing" {
  local shape
  for shape in \
    '[build-system]|requires = [ ]|[project]|dependencies = []' \
    '[project]|name = "fixture"' \
    ''; do
    IFS='|' read -r -a lines <<<"${shape}"
    pyproject_toml "${lines[@]}"

    run "${script}"

    [[ "${status}" -eq 0 ]]
    [[ -z "${output}" ]]
  done
}

@test "a manifest that cannot be read stops the check" {
  local manifest
  for manifest in mise.toml Cargo.toml pyproject.toml; do
    cp "${manifest}" "${manifest}.kept"
    printf '[broken\n' >"${manifest}"

    run "${script}"

    [[ "${status}" -eq 2 ]]
    [[ "${output}" == *"${manifest} cannot be read"* ]]
    mv "${manifest}.kept" "${manifest}"
  done
}

@test "every refusal is reported in one run" {
  mise_toml 'jq = "1"' 'shfmt = "3"'
  cargo_toml 'jq = "1"' 'shfmt = "3"'
  pyproject_toml '[project]' 'name = "fixture"' 'dependencies = ["pytest"]'

  run "${script}"

  [[ "${status}" -eq 1 ]]
  [[ "${output}" == *"  jq: Cargo.toml as jq, mise.toml as jq"* ]]
  [[ "${output}" == *"  shfmt: Cargo.toml as shfmt, mise.toml as shfmt"* ]]
  [[ "${output}" == *"${python}"* ]]
  [[ "${output}" == *"  project.dependencies"* ]]
}

@test "a crate in a cargo script and Cargo.toml is refused" {
  mkdir -p scripts/check
  printf '%s\n' '---cargo' '[dependencies]' 'serde = "1"' '---' '' 'fn main() {}' \
    >scripts/check/check.rs
  git add scripts/check/check.rs

  run "${script}"

  [[ "${status}" -eq 1 ]]
  [[ "${output}" == *"${conflict}"* ]]
  [[ "${output}" == *"  serde: Cargo.toml as serde, scripts/check/check.rs as serde"* ]]
}
