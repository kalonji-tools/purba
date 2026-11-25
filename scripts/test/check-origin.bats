# scripts/check-origin.sh against the branches it refuses.
: "${BATS_TEST_DIRNAME:?set by bats}"
bats_require_minimum_version 1.5.0

setup() {
  # shellcheck source=scripts/test/fixture.sh
  . "${BATS_TEST_DIRNAME}/fixture.sh"
  script="${BATS_TEST_DIRNAME}/../check-origin.sh"
  signed="Signed-off-by: A Person <person@example.invalid>"
  unsigned="A commit is refused when it carries no Signed-off-by trailer, because that trailer \
records who may submit it. CONTRIBUTING.md has the command that adds it."
  merged="A commit is refused when it is a merge commit, because every branch in this repository \
refuses one. Rebase your branch onto its base instead."
  make_repo
  git switch --quiet --create work
}

@test "a branch whose every commit is signed passes" {
  commit "feat: one" "${signed}"
  commit "feat: two" "${signed}"

  run "${script}" main work

  [[ "${status}" -eq 0 ]]
  [[ -z "${output}" ]]
}

@test "a commit with no sign-off is refused, and only that commit is named" {
  commit "feat: signed" "${signed}"
  commit "feat: unsigned"

  run "${script}" main work

  [[ "${status}" -eq 1 ]]
  output_has_line "${unsigned}"
  [[ "${output}" == "${unsigned}"$'\n'* ]]
  [[ "${output}" == *$'\n  '[0-9a-f]*" feat: unsigned"* ]]
  [[ "${output}" != *"feat: signed"* ]]
}

@test "a sign-off written in the body and not as a trailer is refused" {
  commit "feat: prose" "${signed}" "" "A closing paragraph."

  run "${script}" main work

  [[ "${status}" -eq 1 ]]
  [[ "${output}" == *"feat: prose"* ]]
}

# An address sits inside angle brackets and holds an `@`, so each line below
# lacks one half of that or both.
@test "a sign-off whose address cannot reach anyone is refused" {
  commit "feat: no address" "Signed-off-by: A Person"
  commit "feat: no at sign" "Signed-off-by: A Person <person>"
  commit "feat: no brackets" "Signed-off-by: A Person person@example.invalid"

  run "${script}" main work

  [[ "${status}" -eq 1 ]]
  [[ "${output}" == *"feat: no address"* ]]
  [[ "${output}" == *"feat: no at sign"* ]]
  [[ "${output}" == *"feat: no brackets"* ]]
}

@test "a branch that holds no commit of its own passes" {
  run "${script}" main main

  [[ "${status}" -eq 0 ]]
  [[ -z "${output}" ]]
}

@test "an unsigned commit the base gained since the branch left it is not read" {
  commit "feat: one" "${signed}"
  git switch --quiet main
  commit "feat: unsigned and only on the base"

  run "${script}" main work

  [[ "${status}" -eq 0 ]]
}

@test "a signed merge commit is refused, and only that commit is named" {
  git switch --quiet --create side main
  commit "feat: side" "${signed}"
  git switch --quiet work
  commit "feat: mine" "${signed}"
  git merge --quiet --no-ff side --message "chore: merge" --message "${signed}"

  run "${script}" main work

  [[ "${status}" -eq 1 ]]
  [[ "${output}" == "${merged}"$'\n'* ]]
  [[ "${output}" == *$'\n  '[0-9a-f]*" chore: merge"* ]]
  [[ "${output}" != *"feat: side"* ]]
  [[ "${output}" != *"feat: mine"* ]]
}

@test "an unsigned merge commit is named once, under the merge commit" {
  git switch --quiet --create side main
  commit "feat: side" "${signed}"
  git switch --quiet work
  commit "feat: mine" "${signed}"
  git merge --quiet --no-ff side --message "chore: merge"

  run "${script}" main work

  [[ "${status}" -eq 1 ]]
  named=$(git log -1 --format='%h %s' work)
  [[ "${output}" == "${merged}"$'\n  '"${named}" ]]
}

@test "two merge commits are each named" {
  git switch --quiet --create side main
  commit "feat: side" "${signed}"
  git switch --quiet --create other main
  commit "feat: other" "${signed}"
  git switch --quiet work
  git merge --quiet --no-ff side --message "chore: merge side" --message "${signed}"
  git merge --quiet --no-ff other --message "chore: merge other" --message "${signed}"

  run "${script}" main work

  [[ "${status}" -eq 1 ]]
  [[ "${output}" == "${merged}"$'\n  '*" chore: merge other"$'\n  '*" chore: merge side" ]]
}

@test "a merge commit on the base, below where the branch leaves it, is not read" {
  git switch --quiet --create side main
  commit "feat: side" "${signed}"
  git switch --quiet main
  git merge --quiet --no-ff side --message "chore: merge on the base" --message "${signed}"
  git switch --quiet --force-create work main
  commit "feat: mine" "${signed}"

  run "${script}" main work

  [[ "${status}" -eq 0 ]]
  [[ -z "${output}" ]]
}

@test "a merge commit and an unsigned commit are each named under their own refusal" {
  git switch --quiet --create side main
  commit "feat: side" "${signed}"
  git switch --quiet work
  git merge --quiet --no-ff side --message "chore: merge" --message "${signed}"
  commit "feat: unsigned"

  run "${script}" main work

  [[ "${status}" -eq 1 ]]
  first="${unsigned}"$'\n  '$(git log -1 --format='%h %s' work)
  second="${merged}"$'\n  '$(git log -1 --format='%h %s' work^)
  [[ "${output}" == "${first}"$'\n\n'"${second}" ]]
}

@test "on a runner the refusal is one annotation, and each commit keeps its line" {
  commit "feat: reach 100%"
  commit "feat: second unsigned"

  GITHUB_ACTIONS=true run "${script}" main work

  [[ "${status}" -eq 1 ]]
  [[ "${#lines[@]}" -eq 1 ]]
  [[ "${output}" == "::error::A commit is refused when it carries no Signed-off-by trailer"* ]]
  [[ "${output}" == *"%0A"*"feat: second unsigned%0A"*"feat: reach 100%25" ]]
}

@test "a refusal is appended to the file the caller names" {
  commit "feat: unsigned"

  PURBA_REPORT="${BATS_TEST_TMPDIR}/refusal" run "${script}" main work

  [[ "${status}" -eq 1 ]]
  grep -q "refused when it carries no Signed-off-by trailer" "${BATS_TEST_TMPDIR}/refusal"
  grep -q "feat: unsigned" "${BATS_TEST_TMPDIR}/refusal"
}

@test "a range that cannot be read exits 2" {
  run "${script}" main no-such-branch

  [[ "${status}" -eq 2 ]]
  [[ "${output}" == *"the origin check could not read the range main..no-such-branch"* ]]
  [[ "${output}" == *"unknown revision"* ]]
}

@test "a base name git calls ambiguous is read, and git's warning reaches stderr" {
  commit "feat: one" "${signed}"
  git update-ref refs/remotes/origin/main main
  git branch origin/main main

  run --separate-stderr "${script}" origin/main work

  [[ "${status}" -eq 0 ]]
  [[ "${stderr}" == *"refname 'origin/main' is ambiguous"* ]]
}

@test "the wrong number of arguments exits 2" {
  run "${script}" main

  [[ "${status}" -eq 2 ]]
  [[ "${output}" == *"usage: check-origin.sh <base> <head>"* ]]
}

# A git that exits 128 on any call whose arguments hold the text it is given, and
# runs every other.
fail_git() {
  local git
  git=$(command -v git)
  fake git <<FAKE
for argument in "\$@"; do
  [[ "\${argument}" != *"$1"* ]] || { echo "fatal: bad object" >&2; exit 128; }
done
exec "${git}" "\$@"
FAKE
}

@test "a trailer that cannot be read exits 2, and refuses no commit" {
  commit "feat: one" "${signed}"
  fail_git "trailers"

  run "${script}" main work

  [[ "${status}" -eq 2 ]]
  [[ "${output}" == *"the origin check could not read the trailers of "* ]]
  [[ "${output}" != *"is refused"* ]]
}

@test "a subject that cannot be read exits 2, and does not pass the commit" {
  commit "feat: signed" "${signed}"
  commit "feat: unsigned"
  fail_git "%h %s"

  run "${script}" main work

  [[ "${status}" -eq 2 ]]
  [[ "${output}" == *"the origin check could not read the subject of "* ]]
}
