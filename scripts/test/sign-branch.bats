# scripts/sign-branch.sh, which a person runs in a terminal.
: "${BATS_TEST_DIRNAME:?set by bats}"

setup() {
  local version
  version=$(script --version 2>/dev/null) || true
  [[ "${version}" == *util-linux* ]] ||
    skip "sign-branch.sh needs a terminal, and this test makes one with util-linux script"

  # shellcheck source=scripts/test/fixture.sh
  . "${BATS_TEST_DIRNAME}/fixture.sh"
  sign="${BATS_TEST_DIRNAME}/../sign-branch.sh"
  own="writer <writer@example.invalid>"
  make_repo
  git switch --quiet --create work
}

# The value of each Signed-off-by trailer on the head of `work`, one per line.
sign_offs() {
  git log -1 --format='%(trailers:key=Signed-off-by,valueonly)' work
}

# Answers the question of sign-branch.sh with <reply>, in a terminal.
#   answer <reply> [<base>]
answer() {
  local command
  printf -v command '%q %q' "${sign}" "${2:-main}"
  printf '%s\n' "$1" | script --quiet --return --command "${command}" /dev/null
}

@test "with rebase.updateRefs set, the answer N leaves a stacked branch where it was" {
  commit "feat: one"
  git branch stack
  commit "feat: two"
  before=$(git rev-parse stack)
  git config rebase.updateRefs true

  run answer n

  [[ "${status}" -eq 1 ]]
  [[ "${output}" == *"nothing was signed."* ]]
  after=$(git rev-parse stack)
  [[ "${after}" == "${before}" ]]
}

@test "a base name git calls ambiguous lists the commits, and signs them" {
  commit "feat: one"
  git update-ref refs/remotes/origin/main main
  git branch origin/main main

  run answer y origin/main

  [[ "${status}" -eq 0 ]]
  [[ "${output}" == *"Measured from origin/main:"* ]]
  [[ "${output}" == *"feat: one"* ]]
  carried=$(sign_offs)
  [[ "${carried}" == "${own}" ]]
}

@test "a commit with another person's sign-off gets the committer's own beside it" {
  commit "feat: one" "Signed-off-by: Someone Else <else@example.invalid>"

  run answer y

  [[ "${status}" -eq 0 ]]
  carried=$(sign_offs)
  [[ "${carried}" == "Someone Else <else@example.invalid>"$'\n'"${own}" ]]
}

@test "a commit whose sign-off has no address gets the committer's own" {
  commit "feat: one" "Signed-off-by: Someone Else"

  run answer y

  [[ "${status}" -eq 0 ]]
  carried=$(sign_offs)
  [[ "${carried}" == "Someone Else"$'\n'"${own}" ]]
}

@test "a commit whose own sign-off comes before another trailer gets no second one" {
  commit "feat: one" "Signed-off-by: ${own}" \
    "Accepted-by: owner-1 <42+owner-1@users.noreply.github.com>"

  run answer y

  [[ "${status}" -eq 0 ]]
  carried=$(sign_offs)
  [[ "${carried}" == "${own}" ]]
}

@test "a commit with no sign-off gets the committer's own" {
  commit "feat: one"

  run answer y

  [[ "${status}" -eq 0 ]]
  carried=$(sign_offs)
  [[ "${carried}" == "${own}" ]]
}
