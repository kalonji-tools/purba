# .github/scripts/standing-approver.sh against the reviews of one pull request.
: "${BATS_TEST_DIRNAME:?set by bats}"
bats_require_minimum_version 1.5.0

setup() {
  # shellcheck source=scripts/test/fixture.sh
  . "${BATS_TEST_DIRNAME}/fixture.sh"
  isolate
  # shellcheck source=.github/scripts/standing-approver.sh
  . "${BATS_TEST_DIRNAME}/../../.github/scripts/standing-approver.sh"
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

@test "a login GitHub would not write is refused as text" {
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

@test "reviews the rule cannot read exit 2, and print no approver" {
  run --separate-stderr standing_approver <<<'[1]'

  [[ "${status}" -eq 2 ]]
  [[ -z "${output}" ]]
}
