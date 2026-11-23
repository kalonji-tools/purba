# scripts/test/fixture.sh against the environment a test must not inherit.
: "${BATS_TEST_DIRNAME:?set by bats}"

setup() {
  # shellcheck source=scripts/test/fixture.sh
  . "${BATS_TEST_DIRNAME}/fixture.sh"
  isolate
}

# What a hook, or a command under `git rebase --exec`, finds in its environment.
@test "a repository git handed the suite is not the one a test commits into" {
  theirs="${BATS_TEST_TMPDIR}/theirs"
  git init --quiet --initial-branch=main "${theirs}"
  export GIT_DIR="${theirs}/.git" GIT_WORK_TREE="${theirs}" GIT_INDEX_FILE="${theirs}/.git/index"

  make_repo

  top=$(git rev-parse --show-toplevel)
  [[ "${top}" -ef "${BATS_TEST_TMPDIR}/repo" ]]
  refs=$(git --git-dir="${theirs}/.git" for-each-ref)
  [[ -z "${refs}" ]]
}

@test "a test that leaves its repository does not find one around it" {
  git init --quiet "${BATS_TEST_TMPDIR}"
  mkdir "${BATS_TEST_TMPDIR}/outside"
  cd "${BATS_TEST_TMPDIR}/outside"

  run git rev-parse --git-dir

  [[ "${status}" -ne 0 ]]
}

@test "a fake that was never run has no call to show" {
  fake gh <<<':'

  asked=$(calls gh)
  [[ -z "${asked}" ]]

  gh pr list --state open
  asked=$(calls gh)
  [[ "${asked}" == "pr list --state open" ]]
}

@test "what a runner or a caller sets does not reach a test" {
  export GITHUB_ACTIONS=true PURBA_REPORT="${BATS_TEST_TMPDIR}/report" FORCE_COLOR=3

  isolate

  [[ -z "${GITHUB_ACTIONS:-}" ]]
  [[ -z "${PURBA_REPORT:-}" ]]
  [[ -z "${FORCE_COLOR:-}" ]]
}

# A refusal is pinned as a whole line, so a word added before it or after it
# fails the match. A refusal holds `**` and `?`, so neither is a pattern. An
# unset variable is empty, so an empty line never matches.
@test "a line of the output matches only when the whole line is given" {
  output=$'the first line\nthe middle line\n\nthe last line'

  output_has_line "the first line"
  output_has_line "the middle line"
  output_has_line "the last line"
  matched=()
  for part in "middle line" "the middle" "the middle line, or not" "and the middle line" "" \
    "the * line" "the?middle line" "the[ ]middle line"; do
    if output_has_line "${part}"; then matched+=("${part}"); fi
  done
  [[ ${#matched[@]} -eq 0 ]]
}

@test "the only line of an output matches, and an empty output matches nothing" {
  output="the only line"

  output_has_line "the only line"
  output=""
  matched=()
  if output_has_line ""; then matched+=(""); fi
  [[ ${#matched[@]} -eq 0 ]]
}
