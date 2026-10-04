# .github/scripts/release-owed.sh against each step of a release `main` can owe.
: "${BATS_TEST_DIRNAME:?set by bats}"

setup() {
  # shellcheck source=scripts/test/fixture.sh
  . "${BATS_TEST_DIRNAME}/fixture.sh"
  script="${BATS_TEST_DIRNAME}/../../.github/scripts/release-owed.sh"
  make_repo

  # The real register, so a test reads the bump git-cliff will compute.
  cp "${BATS_TEST_DIRNAME}/../../cliff.toml" cliff.toml

  # A second `version` key, which the script must not read as purba's.
  cat >Cargo.toml <<'TOML'
[dependencies]
pyo3 = { version = "0.27" }

[package]
name = "purba"
version = "0.0.0"
TOML
  git add cliff.toml Cargo.toml
  git commit --quiet --message "chore: manifests"
  commit "feat: hold a thing (#1)"
}

# `main` as it reads once a release pull request for <version> has merged.
released() {
  sed -i "s/^version = \"0.0.0\"/version = \"$1\"/" Cargo.toml
  git commit --quiet --all --message "chore(release): cut v$1 (#43)"
}

@test "a version no release has named owes the first release, and what it holds" {
  commit "docs: say a thing (#2)"

  run "${script}"

  [[ "${status}" -eq 0 ]]
  [[ "${output}" == *"**A release is owed**, from \`0.0.0\` to \`0.1.0\`."* ]]
  [[ "${output}" == *"- hold a thing (#1)"* ]]
  [[ "${output}" != *"say a thing"* ]]
}

@test "a merged release that no tag carries owes the tag, and not a release" {
  released 0.1.0

  run "${script}"

  [[ "${status}" -eq 0 ]]
  [[ "${output}" == *"**A tag is owed.** \`Cargo.toml\` names \`0.1.0\`, and no tag carries it."* ]]
  [[ "${output}" != *"No release is owed"* ]]
}

@test "nothing since the tag owes nothing" {
  released 0.1.0
  git tag --annotate --message v0.1.0 v0.1.0

  run "${script}"

  [[ "${status}" -eq 0 ]]
  [[ "${output}" == *"No release is owed. Nothing since \`v0.1.0\` reaches the changelog."* ]]
}

@test "a fix after the tag owes a patch that holds the fix alone" {
  released 0.1.0
  git tag --annotate --message v0.1.0 v0.1.0
  commit "fix: mend a thing (#3)"
  commit "docs: say a thing (#4)"

  run "${script}"

  [[ "${status}" -eq 0 ]]
  [[ "${output}" == *"**A release is owed**, from \`0.1.0\` to \`0.1.1\`."* ]]
  [[ "${output}" == *"- mend a thing (#3)"* ]]
  [[ "${output}" != *"hold a thing"* ]]
  [[ "${output}" != *"say a thing"* ]]
}

@test "an argument exits 2" {
  run "${script}" extra

  [[ "${status}" -eq 2 ]]
  [[ "${output}" == *"usage: release-owed.sh"* ]]
}

@test "a manifest with no package version exits 2" {
  sed -i '/^version = "0.0.0"$/d' Cargo.toml

  run "${script}"

  [[ "${status}" -eq 2 ]]
  [[ "${output}" == *"Cargo.toml names no package version"* ]]
}

@test "a git-cliff that fails exits 2" {
  fake git-cliff <<<'exit 1'

  run "${script}"

  [[ "${status}" -eq 2 ]]
  [[ "${output}" == *"git-cliff could not compute the next version"* ]]
}

@test "a git-cliff that prints no version exits 2" {
  fake git-cliff <<<'exit 0'

  run "${script}"

  [[ "${status}" -eq 2 ]]
  [[ "${output}" == *"git-cliff could not compute the next version"* ]]
}

@test "a git-cliff that cannot render the release exits 2" {
  fake git-cliff <<'FAKE'
case "$*" in
  --bumped-version) echo v0.1.0 ;;
  *) exit 1 ;;
esac
FAKE

  run "${script}"

  [[ "${status}" -eq 2 ]]
  [[ "${output}" == *"git-cliff could not render what the release holds"* ]]
}
