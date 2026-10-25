# .github/scripts/standing-approver.sh against the reviews of one pull request.
: "${BATS_TEST_DIRNAME:?set by bats}"
bats_require_minimum_version 1.5.0

setup() {
  # shellcheck source=scripts/test/fixture.sh
  . "${BATS_TEST_DIRNAME}/fixture.sh"
  isolate
  # shellcheck source=.github/scripts/standing-approver.sh
  . "${BATS_TEST_DIRNAME}/../../.github/scripts/standing-approver.sh"
  export GH_REPO=owner/name
  # The permission of each login, as GitHub reports it, read from its prefix.
  fake gh <<'FAKE'
case "$*" in
  "api repos/owner/name/collaborators/reader-"*/permission) echo '{"permission":"read"}' ;;
  "api repos/owner/name/collaborators/writer-"*/permission) echo '{"permission":"write"}' ;;
  "api repos/owner/name/collaborators/blank-"*/permission) echo '{}' ;;
  "api repos/owner/name/collaborators/unreadable-"*/permission)
    echo "HTTP 502" >&2
    exit 1
    ;;
  "api repos/owner/name/collaborators/"*/permission) echo '{"permission":"admin"}' ;;
  *)
    echo "the fake does not answer: $*" >&2
    exit 9
    ;;
esac
FAKE
}

# One review for each `<state>:<login>:<id>`, in the order given.
reviews() {
  local review state login id one all=""
  for review in "$@"; do
    IFS=: read -r state login id <<<"${review}"
    printf -v one '{"state":"%s","user":{"login":"%s","id":%s}}' "${state}" "${login}" "${id}"
    all+="${all:+,}${one}"
  done
  printf '[%s]\n' "${all}"
}

@test "the approval that came last is the one that stands, and a comment is not one" {
  given=$(reviews APPROVED:owner-1:42 APPROVED:owner-2:43 COMMENTED:owner-3:44)

  run standing_approver <<<"${given}"

  [[ "${status}" -eq 0 ]]
  [[ "${output}" == "owner-2 43" ]]
}

@test "an approval its author follows with a change request does not stand" {
  given=$(reviews APPROVED:owner-1:42 CHANGES_REQUESTED:owner-1:42)

  run standing_approver <<<"${given}"

  [[ "${status}" -eq 0 ]]
  [[ -z "${output}" ]]
}

@test "an approval given again after a change request stands, and a comment leaves it" {
  given=$(reviews APPROVED:owner-1:42 CHANGES_REQUESTED:owner-1:42 APPROVED:owner-1:42 \
    COMMENTED:owner-1:42)

  run standing_approver <<<"${given}"

  [[ "${status}" -eq 0 ]]
  [[ "${output}" == "owner-1 42" ]]
}

@test "a change request from another reviewer leaves an approval standing" {
  given=$(reviews APPROVED:owner-1:42 CHANGES_REQUESTED:owner-2:43)

  run standing_approver <<<"${given}"

  [[ "${status}" -eq 0 ]]
  [[ "${output}" == "owner-1 42" ]]
}

@test "an approval its author follows with a dismissed review does not stand" {
  given=$(reviews APPROVED:owner-1:42 DISMISSED:owner-1:42)

  run standing_approver <<<"${given}"

  [[ "${status}" -eq 0 ]]
  [[ -z "${output}" ]]
}

@test "an author who renames their account is still one author" {
  given=$(reviews APPROVED:old-name:42 CHANGES_REQUESTED:new-name:42)

  run standing_approver <<<"${given}"

  [[ "${status}" -eq 0 ]]
  [[ -z "${output}" ]]
}

@test "a login GitHub would not write is refused as text, before its permission is asked" {
  # The login holds a command, and the refusal must print it rather than run it.
  # shellcheck disable=SC2016
  login='own$(id)er'
  given=$(reviews "APPROVED:${login}:42")

  run standing_approver <<<"${given}"

  [[ "${status}" -eq 1 ]]
  summary="A login is written into an Accepted-by trailer only when it holds letters, digits"
  summary+=" and hyphens, because the trailer names the person who accepts the commit, and GitHub"
  summary+=" allows no other character in a person's login. Ask a person to approve the pull"
  summary+=" request."
  [[ "${output}" == *"${summary}"* ]]
  [[ "${output}" == *"  login: ${login}"* ]]
  asked=$(calls gh)
  [[ -z "${asked}" ]]
}

@test "an approval without write access does not stand over an earlier one with it" {
  given=$(reviews APPROVED:owner-1:42 APPROVED:reader-1:50)

  run standing_approver <<<"${given}"

  [[ "${status}" -eq 0 ]]
  [[ "${output}" == "owner-1 42" ]]
}

@test "an approval from an account without write access alone does not stand" {
  given=$(reviews APPROVED:reader-1:50)

  run standing_approver <<<"${given}"

  [[ "${status}" -eq 0 ]]
  [[ -z "${output}" ]]
}

@test "an account whose permission reads write stands, and one that reads read does not" {
  given=$(reviews APPROVED:writer-1:51 APPROVED:reader-1:50)

  run standing_approver <<<"${given}"

  [[ "${status}" -eq 0 ]]
  [[ "${output}" == "writer-1 51" ]]
}

@test "a permission that cannot be read exits 2, and prints no approver" {
  given=$(reviews APPROVED:unreadable-1:52)

  run --separate-stderr standing_approver <<<"${given}"

  [[ "${status}" -eq 2 ]]
  [[ -z "${output}" ]]
}

@test "an approver id that is not a number, or is absent, is refused" {
  summary="An id is written into an Accepted-by trailer only when it is a number, because the"
  summary+=" trailer reaches main, where no commit message is edited, and GitHub gives each"
  summary+=" account a numeric id. Ask a person to approve the pull request."
  for case in '"4x":4x' null:none; do
    given=$(reviews "APPROVED:owner-1:${case%%:*}")

    run standing_approver <<<"${given}"

    [[ "${status}" -eq 1 ]]
    [[ "${output}" == *"${summary}"* ]]
    [[ "${output}" == *"  id: ${case#*:}"* ]]
  done
}

@test "reviews that are not a JSON array exit 2" {
  for given in '{"message": "Not Found"}' 'not json' ''; do
    run standing_approver <<<"${given}"

    [[ "${status}" -eq 2 ]]
  done
}

@test "a reply with no permission exits 2, and prints no approver" {
  given=$(reviews APPROVED:blank-1:53)

  run --separate-stderr standing_approver <<<"${given}"

  [[ "${status}" -eq 2 ]]
  [[ -z "${output}" ]]
}

@test "an approval with no login is passed over" {
  given='[{"state":"APPROVED","user":{"login":"owner-1","id":42}},'
  given+='{"state":"APPROVED","user":{"login":null,"id":7}}]'

  run standing_approver <<<"${given}"

  [[ "${status}" -eq 0 ]]
  [[ "${output}" == "owner-1 42" ]]
}

@test "a call with no GH_REPO exits 2" {
  given=$(reviews APPROVED:owner-1:42)

  GH_REPO='' run --separate-stderr standing_approver <<<"${given}"

  [[ "${status}" -eq 2 ]]
  [[ -z "${output}" ]]
  asked=$(calls gh)
  [[ -z "${asked}" ]]
}

@test "reviews the rule cannot read exit 2, and print no approver" {
  run --separate-stderr standing_approver <<<'[1]'

  [[ "${status}" -eq 2 ]]
  [[ -z "${output}" ]]
}
