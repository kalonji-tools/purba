# scripts/check-replayable.sh against the branches it refuses.
: "${BATS_TEST_DIRNAME:?set by bats}"

setup() {
  # shellcheck source=scripts/test/fixture.sh
  . "${BATS_TEST_DIRNAME}/fixture.sh"
  script="${BATS_TEST_DIRNAME}/../check-replayable.sh"
  make_repo
  git switch --quiet --create work
}

# A hook in the repository under test that writes one line and refuses.
refusing_hook() {
  printf '#!/usr/bin/env bash\necho "%s" >&2\nexit 1\n' "$2" >".git/hooks/$1"
  chmod +x ".git/hooks/$1"
}

@test "a branch that replays onto its base passes" {
  commit "feat: one"
  commit "feat: two"

  run "${script}" main work

  [[ "${status}" -eq 0 ]]
  [[ -z "${output}" ]]
}

@test "a base that moved on reads the same commits" {
  commit "feat: one"
  git switch --quiet main
  commit "feat: elsewhere"
  git switch --quiet work

  run "${script}" main work

  [[ "${status}" -eq 0 ]]
}

@test "a merge that made a change of its own loses content and is refused" {
  commit "feat: one"
  git switch --quiet --create side main
  commit "feat: side"
  git switch --quiet work
  git merge --quiet --no-commit --no-ff side
  printf 'made in the merge\n' >only-in-the-merge
  git add only-in-the-merge
  git commit --quiet --message "chore: merge and change"

  run "${script}" main work

  [[ "${status}" -eq 1 ]]
  [[ "${output}" == *"the replay of your branch loses content"* ]]
  [[ "${output}" == *"The replay flattens a merge commit"* ]]
  [[ "${output}" == *"only-in-the-merge"* ]]
  [[ "${output}" == *"chore: merge and change"* ]]
}

@test "an empty commit is refused and named" {
  commit "feat: one"
  git commit --quiet --allow-empty --message "feat: nothing"

  run "${script}" main work

  [[ "${status}" -eq 1 ]]
  [[ "${output}" == *"this commit replays empty"* ]]
  [[ "${output}" == *"feat: nothing"* ]]
}

# Both sides add one file with one content, so the flattened replay finds the
# second commit's change on the base already.
@test "a commit the replay empties is refused and named" {
  printf 'same\n' >shared
  git add shared
  git commit --quiet --message "feat: ours"
  git switch --quiet --create side main
  printf 'same\n' >shared
  git add shared
  git commit --quiet --message "feat: theirs"
  git switch --quiet work
  git merge --quiet --no-ff side --message "chore: merge"

  run "${script}" main work

  [[ "${status}" -eq 1 ]]
  [[ "${output}" == *"this commit replays empty"* ]]
  [[ "${output}" == *"feat: theirs"* ]]
}

@test "a merge resolved by hand does not apply and is refused" {
  commit "feat: one"
  # Both sides change one line of one file, and the writer resolves it by hand.
  printf 'ours\n' >shared
  git add shared
  git commit --quiet --message "feat: ours"
  git switch --quiet --create side main
  printf 'theirs\n' >shared
  git add shared
  git commit --quiet --message "feat: theirs"
  git switch --quiet work
  git merge --quiet --no-commit side 2>/dev/null || true
  printf 'resolved\n' >shared
  git add shared
  git commit --quiet --message "chore: merge by hand"

  run "${script}" main work

  [[ "${status}" -eq 1 ]]
  [[ "${output}" == *"this commit does not apply where purba replays your branch"* ]]
  [[ "${output}" == *"feat: theirs"* ]]
}

# The replay stops on its first commit, so the commit it stands on is the base,
# and the base here is a root commit: it has no parent to read.
@test "a replay that stops on a root commit is refused and does not abort" {
  git switch --quiet --orphan unrelated
  printf 'another history\n' >chorestart
  git add chorestart
  git commit --quiet --message "feat: another root"
  git switch --quiet work
  git merge --quiet --no-commit --allow-unrelated-histories unrelated 2>/dev/null || true
  printf 'resolved\n' >chorestart
  git add chorestart
  git commit --quiet --message "chore: merge another history"

  run "${script}" main work

  [[ "${status}" -eq 1 ]]
  [[ "${output}" == *"this commit does not apply where purba replays your branch"* ]]
  [[ "${output}" == *"feat: another root"* ]]
}

@test "a replay that stops for a reason the check cannot name says so" {
  commit "feat: one"
  refusing_hook pre-commit "the hook refuses"

  run "${script}" main work

  [[ "${status}" -eq 1 ]]
  [[ "${output}" == *"the replay of this branch stopped here, and this check cannot say why"* ]]
  [[ "${output}" == *"feat: one"* ]]
  [[ "${output}" == *"the hook refuses"* ]]
}

# A fresh checkout on a runner has no identity, and the replay commits.
@test "a repository with no committer still replays" {
  commit "feat: one"
  git config user.useConfigOnly true
  unset GIT_AUTHOR_NAME GIT_AUTHOR_EMAIL GIT_COMMITTER_NAME GIT_COMMITTER_EMAIL

  run "${script}" main work

  [[ "${status}" -eq 0 ]]
}

@test "on a runner the refusal is one annotation" {
  git commit --quiet --allow-empty --message "feat: nothing"

  GITHUB_ACTIONS=true run "${script}" main work

  [[ "${status}" -eq 1 ]]
  [[ "${output}" == "::error::this commit replays empty"*"%0A"*"feat: nothing" ]]
}

@test "a refusal leaves no worktree and no configuration behind" {
  git commit --quiet --allow-empty --message "feat: nothing"
  before=$(git config --local --list)

  run "${script}" main work

  [[ "${status}" -eq 1 ]]
  worktrees=$(git worktree list)
  [[ "${worktrees}" != *$'\n'* ]]
  after=$(git config --local --list)
  [[ "${after}" == "${before}" ]]
  scratch=$(find "${TMPDIR}" -mindepth 1)
  [[ -z "${scratch}" ]]
}

@test "a head that shares no history with the base exits 2" {
  git switch --quiet --orphan unrelated
  commit "feat: alone"

  run "${script}" main unrelated

  [[ "${status}" -eq 2 ]]
  [[ "${output}" == *"could not find where unrelated leaves main"* ]]
}

@test "the wrong number of arguments exits 2" {
  run "${script}" main

  [[ "${status}" -eq 2 ]]
  [[ "${output}" == *"usage: check-replayable.sh <base> <head>"* ]]
}

@test "a replay git refuses to start says it stopped on no commit" {
  commit "feat: one"
  refusing_hook pre-rebase "the hook refuses the rebase"

  run "${script}" main work

  [[ "${status}" -eq 1 ]]
  [[ "${output}" == *"this check cannot say why"* ]]
  [[ "${output}" == *"the replay left no record of the commit it stopped on."* ]]
  [[ "${output}" == *"the hook refuses the rebase"* ]]
}

@test "a worktree git cannot make exits 2, and what git wrote is kept" {
  commit "feat: one"
  refusing_hook post-checkout "the hook refuses the checkout"

  run "${script}" main work

  [[ "${status}" -eq 2 ]]
  [[ "${output}" == *"the replay check could not make a worktree to replay work in."* ]]
  [[ "${output}" == *"the hook refuses the checkout"* ]]
  scratch=$(find "${TMPDIR}" -mindepth 1)
  [[ -z "${scratch}" ]]
}

# The head commit is empty, and git never reaches it. The empty-commit message
# names a commit the replay stopped on, so it must not be written here.
@test "a replay git refuses to start is not blamed on an empty commit" {
  git commit --quiet --allow-empty --message "feat: nothing"
  refusing_hook pre-rebase "the hook refuses the rebase"

  run "${script}" main work

  [[ "${status}" -eq 1 ]]
  [[ "${output}" == *"this check cannot say why"* ]]
  [[ "${output}" != *"this commit replays empty"* ]]
}

# Inside the replay worktree `HEAD` is the replayed commit, so the script must
# resolve the head it was given before it goes in.
@test "a head given as HEAD is the caller's HEAD, not the replay's" {
  commit "feat: one"
  git switch --quiet --create side main
  commit "feat: side"
  git switch --quiet work
  git merge --quiet --no-commit --no-ff side
  printf 'made in the merge\n' >only-in-the-merge
  git add only-in-the-merge
  git commit --quiet --message "chore: merge and change"

  run "${script}" main HEAD

  [[ "${status}" -eq 1 ]]
  [[ "${output}" == *"only-in-the-merge"* ]]
}
