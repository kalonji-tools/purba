# .github/scripts/bump-nightly.sh against a quiet week and a week the pin moved.
: "${BATS_TEST_DIRNAME:?set by bats}"

setup() {
  # shellcheck source=scripts/test/fixture.sh
  . "${BATS_TEST_DIRNAME}/fixture.sh"
  script="${BATS_TEST_DIRNAME}/../../.github/scripts/bump-nightly.sh"
  make_repo
  export GH_REPO=owner/name

  origin="${BATS_TEST_TMPDIR}/origin.git"
  git init --quiet --bare "${origin}"
  git remote add origin "${origin}"

  # The real file names a second version above the pin, and the pin must not be
  # read from that line.
  mkdir -p .config
  printf 'min_version = "2026.1.1"\nrust = { version = "nightly-2026-01-01" }\n' >.config/mise.toml
  printf 'version = "nightly-2026-01-01"\n' >.config/mise.lock
  git add .config/mise.toml .config/mise.lock
  git commit --quiet --message "chore: pin"
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
  week ':'

  moved='sed -i "s/2026-01-01/2026-01-08/" .config/mise.toml .config/mise.lock'
  subject="chore: move the nightly to 2026-01-08 (#65)"
}

# What `mise upgrade` does to the tree this week.
week() {
  fake mise <<<"$1"
}

# One field of the commit on the proposal branch, as `git log --format` names it.
pushed() {
  git --git-dir="${origin}" log -1 --format="$1" bump/nightly
}

# A commit a test makes after the identity is gone.
as_writer() {
  git -c user.name=writer -c user.email=writer@example.invalid "$@"
}

@test "a quiet week proposes nothing" {
  run "${script}" bump/nightly 65

  [[ "${status}" -eq 0 ]]
  [[ "${output}" == *"the pinned nightly is the newest mise offers"* ]]
  upgraded=$(calls mise)
  [[ "${upgraded}" == "upgrade --bump rust" ]]
  remote=$(git ls-remote origin)
  [[ -z "${remote}" ]]
  asked=$(calls gh)
  [[ "${asked}" != *"pr create"* ]]
}

@test "a week the pin moved pushes one commit and opens one pull request" {
  week "${moved}"

  run "${script}" bump/nightly 65

  [[ "${status}" -eq 0 ]]
  [[ "${output}" == *"proposing nightly-2026-01-08, which replaces nightly-2026-01-01"* ]]
  landed=$(pushed '%s on %P')
  [[ "${landed}" == "${subject} on ${start}" ]]
  asked=$(calls gh)
  [[ "${asked}" == *"pr create --repo owner/name --base main --head bump/nightly"* ]]
  [[ "${asked}" == *"--title ${subject}"* ]]
  [[ "${asked}" == *"| from | \`nightly-2026-01-01\` |"* ]]
  [[ "${asked}" == *"| to | \`nightly-2026-01-08\` |"* ]]
  [[ "${asked}" == *"https://github.com/owner/name/issues/65"* ]]
  [[ "${asked}" == *"git fetch origin bump/nightly"*"mise run sign-off"* ]]
  [[ "${asked}" == *"git push --force-with-lease origin HEAD:bump/nightly"* ]]
}

@test "the commit it pushes is the bot's, carries no sign-off, and holds the two files" {
  week "${moved}"

  run "${script}" bump/nightly 65

  [[ "${status}" -eq 0 ]]
  author=$(pushed '%an <%ae>')
  [[ "${author}" == "github-actions[bot] <41898282+github-actions[bot]@users.noreply.github.com>" ]]
  signed=$(pushed '%(trailers:key=Signed-off-by)')
  [[ -z "${signed}" ]]
  files=$(git --git-dir="${origin}" show --format= --name-only bump/nightly)
  [[ "${files}" == $'.config/mise.lock\n.config/mise.toml' ]]
}

@test "a file the bump staged is not in the commit it pushes" {
  week "${moved}; echo new >staged-by-the-bump; git add staged-by-the-bump"

  run "${script}" bump/nightly 65

  [[ "${status}" -eq 0 ]]
  files=$(git --git-dir="${origin}" show --format= --name-only bump/nightly)
  [[ "${files}" == $'.config/mise.lock\n.config/mise.toml' ]]
}

@test "an open proposal makes the run stand down before it upgrades" {
  printf '41\n' >"${BATS_TEST_TMPDIR}/open"
  week "${moved}"

  run "${script}" bump/nightly 65

  [[ "${status}" -eq 0 ]]
  [[ "${output}" == *"pull request #41 is already proposing a nightly on bump/nightly"* ]]
  asked=$(calls gh)
  [[ "${asked}" == "pr list --repo owner/name --head bump/nightly --state open"* ]]
  upgraded=$(calls mise)
  [[ -z "${upgraded}" ]]
  remote=$(git ls-remote origin)
  [[ -z "${remote}" ]]
}

@test "a pin the script cannot read exits 2 before it upgrades" {
  printf 'rust = "nightly"\n' >.config/mise.toml
  as_writer commit --quiet --all --message "chore: a pin with no version key"

  run "${script}" bump/nightly 65

  [[ "${status}" -eq 2 ]]
  [[ "${output}" == *".config/mise.toml does not name a rust version this script can read"* ]]
  upgraded=$(calls mise)
  [[ -z "${upgraded}" ]]
}

@test "a pin the bump left unreadable exits 2 and pushes nothing" {
  week 'printf "rust = \"nightly\"\n" >.config/mise.toml'

  run "${script}" bump/nightly 65

  [[ "${status}" -eq 2 ]]
  [[ "${output}" == *".config/mise.toml no longer names a rust version this script can read"* ]]
  remote=$(git ls-remote origin)
  [[ -z "${remote}" ]]
}

@test "an upgrade that fails stops the run, and nothing is pushed" {
  week 'echo "mise could not reach the channel" >&2; exit 3'

  run "${script}" bump/nightly 65

  [[ "${status}" -eq 2 ]]
  [[ "${output}" == *"mise could not reach the channel"* ]]
  remote=$(git ls-remote origin)
  [[ -z "${remote}" ]]
  asked=$(calls gh)
  [[ "${asked}" != *"pr create"* ]]
}

# Staged, so the worktree and the index agree and only `HEAD` shows the change.
@test "a tree that is not clean exits 2 before it upgrades" {
  echo stray >>chorestart
  git add chorestart

  run "${script}" bump/nightly 65

  [[ "${status}" -eq 2 ]]
  [[ "${output}" == *"bump-nightly.sh needs a clean tree, because it rewinds to one."* ]]
  upgraded=$(calls mise)
  [[ -z "${upgraded}" ]]
}

@test "open pull requests that cannot be read exit 2 before it upgrades" {
  fake gh <<<'exit 4'

  run "${script}" bump/nightly 65

  [[ "${status}" -eq 2 ]]
  [[ "${output}" == *"the open pull requests on bump/nightly could not be read."* ]]
  upgraded=$(calls mise)
  [[ -z "${upgraded}" ]]
}

@test "a commit git refuses exits 2 and pushes nothing" {
  week "${moved}"
  printf '#!/usr/bin/env bash\nexit 1\n' >.git/hooks/pre-commit
  chmod +x .git/hooks/pre-commit

  run "${script}" bump/nightly 65

  [[ "${status}" -eq 2 ]]
  [[ "${output}" == *"the proposal could not be committed"* ]]
  remote=$(git ls-remote origin)
  [[ -z "${remote}" ]]
}

@test "a bump that rewrites the lock and moves no pin proposes nothing" {
  week 'echo "# a comment" >>.config/mise.toml; echo "another line" >>.config/mise.lock'

  run "${script}" bump/nightly 65

  [[ "${status}" -eq 0 ]]
  [[ "${output}" == *"the pin is still nightly-2026-01-01, so there is nothing to propose"* ]]
  remote=$(git ls-remote origin)
  [[ -z "${remote}" ]]
  asked=$(calls gh)
  [[ "${asked}" != *"pr create"* ]]
  dirty=$(git status --porcelain)
  [[ -z "${dirty}" ]]
}
