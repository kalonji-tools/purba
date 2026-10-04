# scripts/check-links.sh against the links and the cited paths it refuses.
: "${BATS_TEST_DIRNAME:?set by bats}"

setup() {
  # shellcheck source=scripts/test/fixture.sh
  . "${BATS_TEST_DIRNAME}/fixture.sh"
  script="${BATS_TEST_DIRNAME}/../check-links.sh"
  make_repo
  # The comments cite paths in purba, and this repository holds none of them.
  grep -v -E '^[[:space:]]*#' "${BATS_TEST_DIRNAME}/../../lychee.toml" >lychee.toml
  write b.md '# Title'
  write a.md '[the title](b.md#title)'
  write docs/decisions/record.md '# A record'
  write tasks.toml '#   the decision:  docs/decisions/record.md'
  settle
  export PURBA_BASE=HEAD
  offline="A link to a file or a heading in this tree is refused"
  online="A link in a file this branch changes is refused"
  cited="A path that a comment cites is refused"
}

# Each file is written whole, one argument to a line.
write() {
  local file=$1
  shift
  mkdir -p "$(dirname "${file}")"
  printf '%s\n' "$@" >"${file}"
}

settle() {
  git add --all
  git commit --quiet --message "docs: settle"
}

# A fake that keeps what each leg was asked to read, and refuses nothing.
fake_lychee() {
  fake lychee <<'FAKE'
leg=online
[[ "$*" != *--offline* ]] || leg=offline
cat >"${BATS_TEST_TMPDIR}/inputs.${leg}"
FAKE
}

@test "a tree whose links and cited paths all resolve passes" {
  run "${script}"

  [[ "${status}" -eq 0 ]]
  [[ -z "${output}" ]]
}

@test "a dead link to a tracked file is refused, though the branch did not change that file" {
  git rm --quiet b.md
  settle

  run "${script}"

  [[ "${status}" -eq 1 ]]
  [[ "${output}" == *"${offline}"* ]]
  [[ "${output}" == *"[a.md]"* ]]
  [[ "${output}" == *"b.md#title"* ]]
}

@test "a link to a heading that does not exist is refused" {
  write a.md '[the title](b.md#nope)'
  settle

  run "${script}"

  [[ "${status}" -eq 1 ]]
  [[ "${output}" == *"${offline}"* ]]
  [[ "${output}" == *"b.md#nope"* ]]
}

@test "a built URL, a fixture and an email address pass the online leg" {
  # shellcheck disable=SC2016 # the script's own expansion, written as it is in the tree
  write scripts/report.sh 'url="https://github.com/${GH_REPO}/issues/${issue}"' \
    'one=https://github.com/o/purba/issues/9' 'two=https://github.com/owner/name/issues/65' \
    'mail=42+owner@users.noreply.github.com'
  git add --all

  run "${script}"

  [[ "${status}" -eq 0 ]]
  [[ -z "${output}" ]]
}

@test "a dead link inside a code span is refused" {
  # Joined at run time, because the gate reads this file too.
  local dead
  dead="file:"///gone/nowhere.md
  write a.md "See \`${dead}\` here."
  settle

  run "${script}"

  [[ "${status}" -eq 1 ]]
  [[ "${output}" == *"${offline}"* ]]
  [[ "${output}" == *"${dead}"* ]]
}

@test "a file .gitattributes marks generated is not read for a link or a cited path" {
  # Joined at run time, because the gate reads this file too.
  local dead
  dead="file:"///gone/nowhere.md
  write .gitattributes 'made.txt linguist-generated'
  write made.txt "source = \"${dead}\"" '# docs/gone.md'
  settle

  run "${script}"

  [[ "${status}" -eq 0 ]]
  [[ -z "${output}" ]]

  cp made.txt read.txt
  git add read.txt

  run "${script}"

  [[ "${status}" -eq 1 ]]
  [[ "${output}" == *"${dead}"* ]]
  [[ "${output}" == *"read.txt:2: docs/gone.md"* ]]
  [[ "${output}" != *"made.txt"* ]]
}

@test "the online leg reads only the files changed since PURBA_BASE, and no deleted file" {
  fake_lychee
  write .gitattributes 'made.txt linguist-generated'
  write made.txt 'changed'
  write c.md 'new'
  git rm --quiet b.md
  git add --all

  run "${script}"

  [[ "${status}" -eq 0 ]]
  asked=$(calls lychee)
  mapfile -t legs <<<"${asked}"
  [[ ${#legs[@]} -eq 2 ]]
  [[ "${legs[0]}" == *"--offline"* ]]
  [[ "${legs[1]}" != *"--offline"* ]]
  online_inputs=$(LC_ALL=C sort "${BATS_TEST_TMPDIR}/inputs.online")
  expected=$(printf '%s\n' .gitattributes c.md)
  [[ "${online_inputs}" == "${expected}" ]]
  offline_inputs=$(LC_ALL=C sort "${BATS_TEST_TMPDIR}/inputs.offline")
  expected=$(printf '%s\n' .gitattributes a.md c.md chorestart docs/decisions/record.md \
    lychee.toml tasks.toml)
  [[ "${offline_inputs}" == "${expected}" ]]
}

@test "on a laptop the base is the merge base with origin/main" {
  fake_lychee
  unset PURBA_BASE
  git update-ref refs/remotes/origin/main HEAD
  write c.md 'new'
  settle

  run "${script}"

  [[ "${status}" -eq 0 ]]
  [[ "$(<"${BATS_TEST_TMPDIR}/inputs.online")" == "c.md" ]]

  git update-ref -d refs/remotes/origin/main

  run "${script}"

  [[ "${status}" -eq 2 ]]
  [[ "${output}" == *"PURBA_BASE"* ]]
}

@test "a base git cannot read stops the check with exit 2" {
  run env PURBA_BASE=no-such-ref "${script}"

  [[ "${status}" -eq 2 ]]
  [[ "${output}" == *"what this branch changes against no-such-ref cannot be read"* ]]
}

@test "a base that looks like an option is not read as one" {
  run env PURBA_BASE="--output=${BATS_TEST_TMPDIR}/written" "${script}"

  [[ "${status}" -eq 2 ]]
  [[ ! -e "${BATS_TEST_TMPDIR}/written" ]]
}

@test "a dead link the online leg finds is refused" {
  fake lychee <<'FAKE'
cat >/dev/null
[[ "$*" != *--offline* ]] || exit 0
printf '[c.md]:\n[404] https://example.invalid/gone (at 1:1) | Rejected status code\n'
exit 2
FAKE
  write c.md 'new'
  git add --all

  run "${script}"

  [[ "${status}" -eq 1 ]]
  [[ "${output}" == *"${online}"* ]]
  [[ "${output}" == *"[404] https://example.invalid/gone"* ]]
  [[ "${output}" != *"${offline}"* ]]
}

@test "a dead link to a file in a changed file is refused once" {
  write c.md '[gone](gone.md)'
  git add --all

  run "${script}"

  [[ "${status}" -eq 1 ]]
  [[ "${output}" == *"${offline}"* ]]
  [[ "${output}" != *"${online}"* ]]
}

@test "a lychee that cannot run stops the check with exit 2" {
  fake lychee <<'FAKE'
cat >/dev/null
printf 'Error while loading config\n'
exit 3
FAKE

  run "${script}"

  [[ "${status}" -eq 2 ]]
  [[ "${output}" == *"lychee could not run"* ]]
  [[ "${output}" == *"Error while loading config"* ]]
}

@test "a comment that cites a missing path is refused, with its file, line and path" {
  write tasks.toml '[tasks]' '#   the decision:  docs/decisions/gone.md'
  settle

  run "${script}"

  [[ "${status}" -eq 1 ]]
  [[ "${output}" == *"${cited}"* ]]
  [[ "${output}" == *"tasks.toml:2: docs/decisions/gone.md"* ]]
}

@test "a cited directory that holds a tracked file passes, and a glob is skipped" {
  write .github/workflows/x.yml '# .github/workflows/ docs/decisions/ .github/** docs/gone/*.md'
  settle

  run "${script}"

  [[ "${status}" -eq 0 ]]
  [[ -z "${output}" ]]
}

@test "a path outside a comment, or in Markdown, is not read" {
  write tasks.toml 'run = "scripts/gone.sh"'
  write README.md '# See docs/gone.md'
  settle

  run "${script}"

  [[ "${status}" -eq 0 ]]
  [[ -z "${output}" ]]
}

@test "a comment in a file with no extension, and a // comment, are read" {
  write CODEOWNERS '# docs/gone.md'
  write src/lib.rs '// scripts/gone.sh'
  settle

  run "${script}"

  [[ "${status}" -eq 1 ]]
  [[ "${output}" == *"CODEOWNERS:1: docs/gone.md"* ]]
  [[ "${output}" == *"src/lib.rs:1: scripts/gone.sh"* ]]
}

@test "a cited path on disk that git does not track is refused" {
  write docs/untracked.md '# Not tracked'
  write tasks.toml '# docs/untracked.md'
  git add tasks.toml
  git commit --quiet --message "docs: cite it"

  run "${script}"

  [[ "${status}" -eq 1 ]]
  [[ "${output}" == *"tasks.toml:1: docs/untracked.md"* ]]
}

@test "every refusal is reported in one run" {
  git rm --quiet b.md
  write tasks.toml '# docs/gone.md'
  settle

  run "${script}"

  [[ "${status}" -eq 1 ]]
  [[ "${output}" == *"${offline}"* ]]
  [[ "${output}" == *"${cited}"* ]]
}

@test "a run from a subdirectory reads the whole tree" {
  write tasks.toml '# docs/gone.md'
  settle
  mkdir -p deep/er

  cd deep/er
  run "${script}"

  [[ "${status}" -eq 1 ]]
  [[ "${output}" == *"tasks.toml:1: docs/gone.md"* ]]
}

@test "outside a git repository the check stops with exit 2" {
  mkdir -p "${BATS_TEST_TMPDIR}/plain"

  cd "${BATS_TEST_TMPDIR}/plain"
  run "${script}"

  [[ "${status}" -eq 2 ]]
  [[ "${output}" == *"git repository"* ]]
}
