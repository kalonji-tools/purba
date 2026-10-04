# scripts/check-origin.sh against the branches it refuses.
: "${BATS_TEST_DIRNAME:?set by bats}"

setup() {
  # shellcheck source=scripts/test/fixture.sh
  . "${BATS_TEST_DIRNAME}/fixture.sh"
  script="${BATS_TEST_DIRNAME}/../check-origin.sh"
  signed="Signed-off-by: A Person <person@example.invalid>"
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
  [[ "${output}" == "A commit is refused when it carries no Signed-off-by trailer"* ]]
  [[ "${output}" == *"because that trailer records who may submit it"* ]]
  [[ "${output}" == *"feat: unsigned"* ]]
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

@test "a signed merge below an unsigned commit draws no merge sentence" {
  git switch --quiet --create side main
  commit "feat: side" "${signed}"
  git switch --quiet work
  commit "feat: mine" "${signed}"
  git merge --quiet --no-ff side --message "chore: merge" --message "${signed}"
  commit "feat: unsigned"

  run "${script}" main work

  [[ "${status}" -eq 1 ]]
  [[ "${output}" != *"A merge commit below is not one you can sign"* ]]
}

@test "an unsigned merge commit draws the merge sentence" {
  git switch --quiet --create side main
  commit "feat: side" "${signed}"
  git switch --quiet work
  commit "feat: mine" "${signed}"
  git merge --quiet --no-ff side --message "chore: merge"

  run "${script}" main work

  [[ "${status}" -eq 1 ]]
  [[ "${output}" == *"A merge commit below is not one you can sign"* ]]
  [[ "${output}" == *"chore: merge"* ]]
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
}

@test "the wrong number of arguments exits 2" {
  run "${script}" main

  [[ "${status}" -eq 2 ]]
  [[ "${output}" == *"usage: check-origin.sh <base> <head>"* ]]
}
