# .github/scripts/release.sh against each step a run can owe.
: "${BATS_TEST_DIRNAME:?set by bats}"

setup() {
  # shellcheck source=scripts/test/fixture.sh
  . "${BATS_TEST_DIRNAME}/fixture.sh"
  script="${BATS_TEST_DIRNAME}/../../.github/scripts/release.sh"
  make_repo
  export GH_REPO=owner/name

  origin="${BATS_TEST_TMPDIR}/origin.git"
  git init --quiet --bare "${origin}"
  git remote add origin "${origin}"

  # The real register, so a test reads the bump git-cliff will compute.
  cp "${BATS_TEST_DIRNAME}/../../cliff.toml" cliff.toml

  # A second `version` key on each side, which the script must leave alone.
  cat >Cargo.toml <<'TOML'
[package]
name = "purba"
version = "0.0.0"

[dependencies]
pyo3 = { version = "0.27" }
TOML
  cat >Cargo.lock <<'LOCK'
[[package]]
name = "pyo3"
version = "0.27.0"

[[package]]
name = "purba"
version = "0.0.0"
LOCK
  git add cliff.toml Cargo.toml Cargo.lock
  git commit --quiet --message "chore: manifests"
  commit "feat: hold a thing (#1)"
  start=$(git rev-parse HEAD)

  # A runner has no identity, so the one the script names is the one that lands.
  unset GIT_AUTHOR_NAME GIT_AUTHOR_EMAIL GIT_COMMITTER_NAME GIT_COMMITTER_EMAIL

  # No pull request is open unless a test says one is.
  fake gh <<'FAKE'
case "$1 $2" in
  "pr list") [[ ! -e "${BATS_TEST_TMPDIR}/open" ]] || cat "${BATS_TEST_TMPDIR}/open" ;;
  *) ;;
esac
FAKE
}

# One field of the commit on the proposal branch, as `git log --format` names it.
pushed() {
  git --git-dir="${origin}" log -1 --format="$1" release/next
}

# A commit a test makes after the identity is gone.
as_writer() {
  git -c user.name=writer -c user.email=writer@example.invalid "$@"
}

# The fixture's `commit`, made after the identity is gone.
later() {
  GIT_AUTHOR_NAME=writer GIT_AUTHOR_EMAIL=writer@example.invalid \
    GIT_COMMITTER_NAME=writer GIT_COMMITTER_EMAIL=writer@example.invalid commit "$@"
}

# `main` as it reads once a release pull request for <version> has merged.
released() {
  sed -i "s/^version = \"0.0.0\"/version = \"$1\"/" Cargo.toml
  as_writer commit --quiet --all --message "chore(release): cut v$1 (#43)"
}

@test "an owed release pushes one commit and opens one pull request" {
  run "${script}" release/next 43

  [[ "${status}" -eq 0 ]]
  [[ "${output}" == *"proposing v0.1.0"* ]]
  landed=$(pushed '%s on %P')
  [[ "${landed}" == "chore(release): cut v0.1.0 (#43) on ${start}" ]]
  files=$(git --git-dir="${origin}" show --format= --name-only release/next)
  [[ "${files}" == $'CHANGELOG.md\nCargo.lock\nCargo.toml' ]]
  asked=$(calls gh)
  [[ "${asked}" == *"pr create --repo owner/name --base main --head release/next"* ]]
  [[ "${asked}" == *"--title chore(release): cut v0.1.0 (#43)"* ]]
  [[ "${asked}" == *"git fetch origin release/next"*"mise run sign-off"* ]]
  [[ "${asked}" == *"https://github.com/owner/name/issues/43"* ]]
  tags=$(git --git-dir="${origin}" tag)
  [[ -z "${tags}" ]]
}

@test "the pull request says the next run tags the version, whether scheduled or dispatched" {
  run "${script}" release/next 43

  [[ "${status}" -eq 0 ]]
  asked=$(calls gh)
  said="the next run of \`Release\` tags \`v0.1.0\`, whether the schedule or a dispatch starts it."
  [[ "${asked}" == *"${said}"* ]]
  [[ "${asked}" != *"again after it lands"* ]]
}

@test "the commit is the bot's, carries no sign-off, and writes the version once on each side" {
  run "${script}" release/next 43

  [[ "${status}" -eq 0 ]]
  author=$(pushed '%an <%ae>')
  [[ "${author}" == "github-actions[bot] <41898282+github-actions[bot]@users.noreply.github.com>" ]]
  signed=$(pushed '%(trailers:key=Signed-off-by)')
  [[ -z "${signed}" ]]
  toml=$(git --git-dir="${origin}" show release/next:Cargo.toml)
  [[ "${toml}" == *$'version = "0.1.0"\n'* ]]
  [[ "${toml}" == *'pyo3 = { version = "0.27" }'* ]]
  lock=$(git --git-dir="${origin}" show release/next:Cargo.lock)
  [[ "${lock}" == *$'name = "pyo3"\nversion = "0.27.0"'* ]]
  [[ "${lock}" == *$'name = "purba"\nversion = "0.1.0"'* ]]
}

@test "the changelog holds the features and fixes under the version, and nothing else" {
  later "fix: mend a thing (#2)"
  later "docs: say a thing (#3)"

  run "${script}" release/next 43

  [[ "${status}" -eq 0 ]]
  log=$(git --git-dir="${origin}" show release/next:CHANGELOG.md)
  [[ "${log}" == *"## 0.1.0"* ]]
  [[ "${log}" == *"- hold a thing (#1)"* ]]
  [[ "${log}" == *"- mend a thing (#2)"* ]]
  [[ "${log}" != *"say a thing"* ]]
  [[ "${log}" != *"manifests"* ]]
}

# The register sets both bump switches to false, and unset reads as true.
@test "a feature after a release proposes a patch" {
  released 0.1.0
  git tag v0.1.0
  later "feat: hold another thing (#4)"

  run "${script}" release/next 43

  [[ "${status}" -eq 0 ]]
  landed=$(pushed %s)
  [[ "${landed}" == "chore(release): cut v0.1.1 (#43)" ]]
}

@test "a breaking change after a release proposes a minor" {
  released 0.1.0
  git tag v0.1.0
  later "feat!: drop a thing (#5)"

  run "${script}" release/next 43

  [[ "${status}" -eq 0 ]]
  landed=$(pushed %s)
  [[ "${landed}" == "chore(release): cut v0.2.0 (#43)" ]]
}

@test "nothing to release proposes nothing" {
  released 0.1.0
  git tag v0.1.0
  later "docs: say a thing (#3)"

  run "${script}" release/next 43

  [[ "${status}" -eq 0 ]]
  [[ "${output}" == *"nothing since v0.1.0 is released"* ]]
  remote=$(git ls-remote origin)
  [[ -z "${remote}" ]]
  asked=$(calls gh)
  [[ "${asked}" != *"pr create"* ]]
}

@test "a version no tag carries is tagged on the commit that wrote it" {
  released 0.1.0
  wrote=$(git rev-parse HEAD)
  # A later commit to the manifest that leaves the version alone, and writes
  # the same version line under another table.
  printf '\n[dependencies.serde]\nversion = "0.1.0"\n' >>Cargo.toml
  as_writer commit --quiet --all --message "feat: depend on a thing (#6)"

  run "${script}" release/next 43

  [[ "${status}" -eq 0 ]]
  [[ "${output}" == *"tagging v0.1.0"* ]]
  kind=$(git --git-dir="${origin}" cat-file -t v0.1.0)
  [[ "${kind}" == tag ]]
  tagged=$(git --git-dir="${origin}" rev-parse 'v0.1.0^{commit}')
  [[ "${tagged}" == "${wrote}" ]]
  branches=$(git --git-dir="${origin}" branch)
  [[ -z "${branches}" ]]
  asked=$(calls gh)
  [[ -z "${asked}" ]]
}

@test "an open release pull request makes the run stand down" {
  printf '41\n' >"${BATS_TEST_TMPDIR}/open"

  run "${script}" release/next 43

  [[ "${status}" -eq 0 ]]
  [[ "${output}" == *"pull request #41 is already proposing a release on release/next"* ]]
  remote=$(git ls-remote origin)
  [[ -z "${remote}" ]]
}

@test "the caller keeps the tree it checked out" {
  run "${script}" release/next 43

  [[ "${status}" -eq 0 ]]
  head=$(git rev-parse HEAD)
  [[ "${head}" == "${start}" ]]
  dirty=$(git status --porcelain)
  [[ -z "${dirty}" ]]
}

@test "a branch a closed pull request left behind is replaced" {
  git switch --quiet --detach
  as_writer commit --quiet --allow-empty --message "chore: an older proposal"
  git push --quiet origin HEAD:refs/heads/release/next
  git switch --quiet main

  run "${script}" release/next 43

  [[ "${status}" -eq 0 ]]
  landed=$(pushed %s)
  [[ "${landed}" == "chore(release): cut v0.1.0 (#43)" ]]
}

@test "a commit git refuses exits 2, pushes nothing, and leaves no changelog" {
  printf '#!/usr/bin/env bash\nexit 1\n' >.git/hooks/pre-commit
  chmod +x .git/hooks/pre-commit

  run "${script}" release/next 43

  [[ "${status}" -eq 2 ]]
  [[ "${output}" == *"the release could not be committed"* ]]
  remote=$(git ls-remote origin)
  [[ -z "${remote}" ]]
  dirty=$(git status --porcelain)
  [[ -z "${dirty}" ]]
}

@test "a register git-cliff refuses exits 2 and shows why" {
  echo 'not toml' >>cliff.toml
  as_writer commit --quiet --all --message "chore: break the register"

  run "${script}" release/next 43

  [[ "${status}" -eq 2 ]]
  [[ "${output}" == *"git-cliff could not compute the next version"* ]]
  [[ "${output}" == *"TOML"* ]]
}

@test "a tree that is not clean exits 2 before it computes anything" {
  echo stray >>chorestart
  git add chorestart

  run "${script}" release/next 43

  [[ "${status}" -eq 2 ]]
  [[ "${output}" == *"release.sh needs a clean tree, because it rewinds to one."* ]]
  asked=$(calls gh)
  [[ -z "${asked}" ]]
}

@test "an issue that is not a number exits 2" {
  for issue in forty-three 0 "" 4x; do
    run "${script}" release/next "${issue}"

    [[ "${status}" -eq 2 ]]
    [[ "${output}" == *"release.sh needs an issue number, and was given '${issue}'."* ]]
  done
}

@test "with an empty repository name it stops before it calls the API" {
  export GH_REPO=

  run "${script}" release/next 43

  [[ "${status}" -eq 2 ]]
  [[ "${output}" == *"set by the workflow env"* ]]
  asked=$(calls gh)
  [[ -z "${asked}" ]]
}

@test "the wrong number of arguments exits 2" {
  run "${script}" release/next

  [[ "${status}" -eq 2 ]]
  [[ "${output}" == *"usage: release.sh <branch> <issue>"* ]]
}

@test "a push that fails exits 2, and the caller keeps the tree it checked out" {
  git remote set-url origin "${BATS_TEST_TMPDIR}/nowhere.git"

  run "${script}" release/next 43

  [[ "${status}" -eq 2 ]]
  [[ "${output}" == *"the proposal could not be pushed to release/next"* ]]
  head=$(git rev-parse HEAD)
  [[ "${head}" == "${start}" ]]
  dirty=$(git status --porcelain)
  [[ -z "${dirty}" ]]
  asked=$(calls gh)
  [[ "${asked}" != *"pr create"* ]]
}

@test "a tag that cannot be pushed exits 2" {
  released 0.1.0
  git remote set-url origin "${BATS_TEST_TMPDIR}/nowhere.git"

  run "${script}" release/next 43

  [[ "${status}" -eq 2 ]]
  [[ "${output}" == *"v0.1.0 could not be pushed"* ]]
}

@test "a pull request that cannot be opened exits 2 and says the branch is pushed" {
  fake gh <<'FAKE'
case "$1 $2" in
  "pr create") exit 4 ;;
  *) ;;
esac
FAKE

  run "${script}" release/next 43

  [[ "${status}" -eq 2 ]]
  [[ "${output}" == *"release/next is pushed, and its pull request could not be opened"* ]]
  landed=$(pushed %s)
  [[ "${landed}" == "chore(release): cut v0.1.0 (#43)" ]]
}
