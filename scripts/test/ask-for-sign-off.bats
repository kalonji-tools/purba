# .github/scripts/ask-for-sign-off.sh against each step a run takes to ask.
: "${BATS_TEST_DIRNAME:?set by bats}"

setup() {
  # shellcheck source=scripts/test/fixture.sh
  . "${BATS_TEST_DIRNAME}/fixture.sh"
  make_repo
  export GH_REPO=owner/name

  origin="${BATS_TEST_TMPDIR}/origin.git"
  git init --quiet --bare "${origin}"
  git remote add origin "${origin}"

  printf 'one\n' >wanted
  git add wanted
  git commit --quiet --message "chore: wanted"
  start=$(git rev-parse HEAD)

  # A runner has no identity, so the one the helper names is the one that lands.
  unset GIT_AUTHOR_NAME GIT_AUTHOR_EMAIL GIT_COMMITTER_NAME GIT_COMMITTER_EMAIL

  # No pull request is open unless a test says one is.
  fake gh <<'FAKE'
case "$1 $2" in
  "pr list") [[ ! -e "${BATS_TEST_TMPDIR}/open" ]] || cat "${BATS_TEST_TMPDIR}/open" ;;
  *) ;;
esac
FAKE

  change='echo two >wanted'

  # shellcheck source=.github/scripts/ask-for-sign-off.sh
  . "${BATS_TEST_DIRNAME}/../../.github/scripts/ask-for-sign-off.sh"
}

# A script that asks for a sign-off on `wanted`, after it runs `change`.
propose() {
  sign_off_start propose.sh .github/workflows/propose.yml "$@"
  stand_down_while_open "a change"
  eval "${change}"
  ask_for_sign_off "chore: a change (#7)" "a change" one two "**A closing line.**" wanted
}

# One field of the commit on the proposal branch, as `git log --format` names it.
pushed() {
  git --git-dir="${origin}" log -1 --format="$1" propose/next
}

@test "the caller keeps the tree it checked out" {
  run propose propose/next 7

  [[ "${status}" -eq 0 ]]
  head=$(git rev-parse HEAD)
  [[ "${head}" == "${start}" ]]
  dirty=$(git status --porcelain)
  [[ -z "${dirty}" ]]
}

@test "a branch a closed pull request left behind is replaced" {
  git switch --quiet --detach
  git -c user.name=writer -c user.email=writer@example.invalid \
    commit --quiet --allow-empty --message "chore: an older proposal"
  git push --quiet origin HEAD:refs/heads/propose/next
  git switch --quiet main

  run propose propose/next 7

  [[ "${status}" -eq 0 ]]
  landed=$(pushed %s)
  [[ "${landed}" == "chore: a change (#7)" ]]
}

@test "an open pull request makes the run stand down, and says until when" {
  printf '41\n' >"${BATS_TEST_TMPDIR}/open"

  run propose propose/next 7

  [[ "${status}" -eq 0 ]]
  said="pull request #41 is already proposing a change on propose/next, so this run stands down"
  [[ "${output}" == *"${said}"* ]]
  [[ "${output}" == *"nothing is proposed until a person signs that one or closes it"* ]]
  remote=$(git ls-remote origin)
  [[ -z "${remote}" ]]
}

@test "a run that changes a third file is refused, pushes nothing, and keeps the tree" {
  change='echo two >wanted; echo stray >>chorestart'

  run propose propose/next 7

  [[ "${status}" -eq 1 ]]
  [[ "${output}" == *"the run changed files beyond wanted, so it is not proposed"* ]]
  remote=$(git ls-remote origin)
  [[ -z "${remote}" ]]
  asked=$(calls gh)
  [[ "${asked}" != *"pr create"* ]]
  dirty=$(git status --porcelain)
  [[ -z "${dirty}" ]]
}

@test "an issue that is not a number exits 2" {
  for issue in seven 0 "" 7x; do
    run propose propose/next "${issue}"

    [[ "${status}" -eq 2 ]]
    [[ "${output}" == *"propose.sh needs an issue number, and was given '${issue}'."* ]]
  done
  asked=$(calls gh)
  [[ -z "${asked}" ]]
}

@test "with an empty repository name it stops before it calls the API" {
  export GH_REPO=

  run propose propose/next 7

  [[ "${status}" -eq 2 ]]
  [[ "${output}" == *"set by the workflow env"* ]]
  asked=$(calls gh)
  [[ -z "${asked}" ]]
}

@test "the wrong number of arguments exits 2" {
  run propose propose/next

  [[ "${status}" -eq 2 ]]
  [[ "${output}" == *"usage: propose.sh <branch> <issue>"* ]]
}

@test "a push that fails exits 2, and the caller keeps the tree it checked out" {
  git remote set-url origin "${BATS_TEST_TMPDIR}/nowhere.git"

  run propose propose/next 7

  [[ "${status}" -eq 2 ]]
  [[ "${output}" == *"the proposal could not be pushed to propose/next"* ]]
  head=$(git rev-parse HEAD)
  [[ "${head}" == "${start}" ]]
  dirty=$(git status --porcelain)
  [[ -z "${dirty}" ]]
  asked=$(calls gh)
  [[ "${asked}" != *"pr create"* ]]
}

@test "a pull request that cannot be opened exits 2 and says the branch is pushed" {
  fake gh <<'FAKE'
case "$1 $2" in
  "pr create") exit 4 ;;
  *) ;;
esac
FAKE

  run propose propose/next 7

  [[ "${status}" -eq 2 ]]
  [[ "${output}" == *"propose/next is pushed, and its pull request could not be opened"* ]]
  landed=$(pushed %s)
  [[ "${landed}" == "chore: a change (#7)" ]]
}
