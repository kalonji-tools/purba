# scripts/apply-fork-contribution.sh against the code owners it refuses and the diff it reads.
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
