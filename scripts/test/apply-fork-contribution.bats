# scripts/apply-fork-contribution.sh against the code owners it refuses, the diff it reads, and
# each call that fails.
: "${BATS_TEST_DIRNAME:?set by bats}"

setup() {
  # shellcheck source=scripts/test/fixture.sh
  . "${BATS_TEST_DIRNAME}/fixture.sh"
  script="${BATS_TEST_DIRNAME}/../apply-fork-contribution.sh"
  make_repo
  mkdir -p .github docs

  git init --quiet --bare "${BATS_TEST_TMPDIR}/origin.git"
  git remote add origin "${BATS_TEST_TMPDIR}/origin.git"
  git push --quiet origin main
  git switch --quiet --create work
  commit "feat: one" "Signed-off-by: A Person <person@example.invalid>"
  git push --quiet origin work:refs/pull/7/head
  git switch --quiet main

  fake gh <<'FAKE'
case "$*" in
  "api user"*) echo me ;;
  "pr list"*) ;;
  "pr view"*) echo "feat: one" ;;
  "pr create"*) echo "https://example.invalid/pull/8" ;;
esac
FAKE
}

@test "a code owner named in .github/CODEOWNERS is refused" {
  echo "* @me" >.github/CODEOWNERS

  run "${script}" 7

  [[ "${status}" -eq 1 ]]
  [[ "${output}" == "refused: me is a code owner"* ]]
}

# GitHub reads the first of these three files it finds, and this repository
# keeps `.github/CODEOWNERS`. GitHub Docs, "About code owners", the section
# "CODEOWNERS file location".
@test "a code owner named only in CODEOWNERS or docs/CODEOWNERS is not refused" {
  echo "* @someone" >.github/CODEOWNERS
  echo "* @me" >CODEOWNERS
  echo "* @me" >docs/CODEOWNERS

  run "${script}" 7

  [[ "${status}" -eq 0 ]]
  accepted=$(git ls-remote origin refs/heads/accepted/pr-7)
  work=$(git rev-parse work)
  [[ "${accepted}" == "${work}"* ]]
}

# A pipe holds 64 KiB on Linux, so `grep -q` that stops reading at the first
# path leaves `git diff` to die of SIGPIPE on the rest. pipe(7), the section
# "Pipe capacity".
@test "a contribution that changes .github/ and more names than a pipe holds is warned about" {
  git switch --quiet work
  mkdir -p .github/workflows many
  touch .github/workflows/run.yml
  for number in $(seq 2000); do
    touch "many/a-path-name-long-enough-that-two-thousand-of-them-fill-a-pipe-${number}"
  done
  git add .github many
  commit "feat: many" "Signed-off-by: A Person <person@example.invalid>"
  git push --quiet --force origin work:refs/pull/7/head
  git switch --quiet main
  mkdir -p .github
  echo "* @someone" >.github/CODEOWNERS

  run "${script}" 7

  [[ "${status}" -eq 0 ]]
  [[ "${output}" == *"warning: this contribution changes .github/"* ]]
}

@test "a contribution that changes only a .github/ path outside ASCII is warned about" {
  git switch --quiet work
  mkdir -p .github/workflows
  touch .github/workflows/café.yml
  git add .github
  commit "feat: café" "Signed-off-by: A Person <person@example.invalid>"
  git push --quiet --force origin work:refs/pull/7/head
  git switch --quiet main
  mkdir -p .github
  echo "* @someone" >.github/CODEOWNERS

  run "${script}" 7

  [[ "${status}" -eq 0 ]]
  [[ "${output}" == *"warning: this contribution changes .github/"* ]]
}

@test "a contribution whose diff cannot be read exits 2 and pushes nothing" {
  echo "* @someone" >.github/CODEOWNERS
  git=$(command -v git)
  fake git <<FAKE
for argument in "\$@"; do
  [[ "\${argument}" != diff ]] || exit 128
done
exec "${git}" "\$@"
FAKE

  run "${script}" 7

  [[ "${status}" -eq 2 ]]
  accepted=$(git ls-remote origin refs/heads/accepted/pr-7)
  [[ -z "${accepted}" ]]
}

# A gh that fails the call it is given, and answers the others as setup does.
fail_gh() {
  fake gh <<FAKE
case "\$*" in
  "$1"*) echo "HTTP 502" >&2; exit 1 ;;
  "api user"*) echo me ;;
  "pr view"*) echo "feat: one" ;;
  "pr create"*) echo "https://example.invalid/pull/8" ;;
esac
FAKE
}

@test "a failed read of the account gh acts as exits 2 and pushes nothing" {
  echo "* @someone" >.github/CODEOWNERS
  fail_gh "api user"

  run "${script}" 7

  [[ "${status}" -eq 2 ]]
  [[ "${output}" == *"the account gh acts as could not be read, so nothing was pushed."* ]]
  accepted=$(git ls-remote origin refs/heads/accepted/pr-7)
  [[ -z "${accepted}" ]]
}

@test "a failed read of the open pull requests exits 2, and says the branch is pushed" {
  echo "* @someone" >.github/CODEOWNERS
  fail_gh "pr list"

  run "${script}" 7

  [[ "${status}" -eq 2 ]]
  said="accepted/pr-7 is pushed, and its open pull requests could not be read."
  [[ "${output}" == *"${said}"* ]]
  accepted=$(git ls-remote origin refs/heads/accepted/pr-7)
  work=$(git rev-parse work)
  [[ "${accepted}" == "${work}"* ]]
}

@test "a failed read of the title exits 2, and says the branch is pushed" {
  echo "* @someone" >.github/CODEOWNERS
  fail_gh "pr view"

  run "${script}" 7

  [[ "${status}" -eq 2 ]]
  [[ "${output}" == *"accepted/pr-7 is pushed, and the title of #7 could not be read."* ]]
  accepted=$(git ls-remote origin refs/heads/accepted/pr-7)
  work=$(git rev-parse work)
  [[ "${accepted}" == "${work}"* ]]
}

@test "a pull request that cannot be opened exits 2, and says the branch is pushed" {
  echo "* @someone" >.github/CODEOWNERS
  fail_gh "pr create"

  run "${script}" 7

  [[ "${status}" -eq 2 ]]
  [[ "${output}" == *"accepted/pr-7 is pushed, and its pull request could not be opened."* ]]
  accepted=$(git ls-remote origin refs/heads/accepted/pr-7)
  work=$(git rev-parse work)
  [[ "${accepted}" == "${work}"* ]]
}

@test "a comment that cannot be posted exits 2, and names the pull request that is open" {
  echo "* @someone" >.github/CODEOWNERS
  fail_gh "pr comment"

  run "${script}" 7

  [[ "${status}" -eq 2 ]]
  said="https://example.invalid/pull/8 is open, and #7 could not be told where its work went."
  [[ "${output}" == *"${said}"* ]]
  accepted=$(git ls-remote origin refs/heads/accepted/pr-7)
  work=$(git rev-parse work)
  [[ "${accepted}" == "${work}"* ]]
}

# A git that exits 1 on any call holding the word it is given, and runs every other.
fail_git() {
  local git
  git=$(command -v git)
  fake git <<FAKE
for argument in "\$@"; do
  [[ "\${argument}" != "$1" ]] || exit 1
done
exec "${git}" "\$@"
FAKE
}

@test "a merge base that cannot be found exits 2 and pushes nothing" {
  echo "* @someone" >.github/CODEOWNERS
  fail_git merge-base

  run "${script}" 7

  [[ "${status}" -eq 2 ]]
  said="the merge base of #7 and main could not be found, so nothing was pushed."
  [[ "${output}" == *"${said}"* ]]
  accepted=$(git ls-remote origin refs/heads/accepted/pr-7)
  [[ -z "${accepted}" ]]
}

@test "a push the remote declines exits 2 and opens no pull request" {
  echo "* @someone" >.github/CODEOWNERS
  fail_git push

  run "${script}" 7

  [[ "${status}" -eq 2 ]]
  [[ "${output}" == *"accepted/pr-7 could not be pushed, so no pull request was opened."* ]]
  asked=$(calls gh)
  [[ "${asked}" != *"pr create"* ]]
}

@test "a work tree whose top cannot be found exits 2 and pushes nothing" {
  echo "* @someone" >.github/CODEOWNERS
  fail_git rev-parse

  run "${script}" 7

  [[ "${status}" -eq 2 ]]
  said="the top of the work tree could not be found, so nothing was pushed."
  [[ "${output}" == *"${said}"* ]]
  accepted=$(git ls-remote origin refs/heads/accepted/pr-7)
  [[ -z "${accepted}" ]]
}

@test "a main that cannot be fetched exits 2 and pushes nothing" {
  echo "* @someone" >.github/CODEOWNERS
  fail_git main

  run "${script}" 7

  [[ "${status}" -eq 2 ]]
  [[ "${output}" == *"main could not be fetched, so nothing was pushed."* ]]
  accepted=$(git ls-remote origin refs/heads/accepted/pr-7)
  [[ -z "${accepted}" ]]
}

@test "a contribution that cannot be fetched exits 2 and pushes nothing" {
  echo "* @someone" >.github/CODEOWNERS
  fail_git refs/pull/7/head

  run "${script}" 7

  [[ "${status}" -eq 2 ]]
  [[ "${output}" == *"#7 could not be fetched, so nothing was pushed."* ]]
  accepted=$(git ls-remote origin refs/heads/accepted/pr-7)
  [[ -z "${accepted}" ]]
}

@test "a .github/CODEOWNERS that cannot be read exits 2 and pushes nothing" {
  run "${script}" 7

  [[ "${status}" -eq 2 ]]
  [[ "${output}" == *".github/CODEOWNERS could not be read, so nothing was pushed."* ]]
  accepted=$(git ls-remote origin refs/heads/accepted/pr-7)
  [[ -z "${accepted}" ]]
}

@test "a .github/CODEOWNERS that is a directory exits 2 and pushes nothing" {
  mkdir .github/CODEOWNERS

  run "${script}" 7

  [[ "${status}" -eq 2 ]]
  [[ "${output}" == *".github/CODEOWNERS could not be read, so nothing was pushed."* ]]
  accepted=$(git ls-remote origin refs/heads/accepted/pr-7)
  [[ -z "${accepted}" ]]
}
