# .github/scripts/write-acceptance-trailer.sh against the branches it accepts and refuses.
: "${BATS_TEST_DIRNAME:?set by bats}"

setup() {
  # shellcheck source=scripts/test/fixture.sh
  . "${BATS_TEST_DIRNAME}/fixture.sh"
  tree="${BATS_TEST_DIRNAME}/../.."
  script="${tree}/.github/scripts/write-acceptance-trailer.sh"
  make_repo

  mkdir -p .github/scripts scripts
  cp "${tree}/.github/scripts/require-green.sh" "${tree}/.github/scripts/check-run.sh" \
    .github/scripts/
  cp "${tree}/scripts/check-origin.sh" "${tree}/scripts/check-replayable.sh" \
    "${tree}/scripts/accept-one-commit.sh" "${tree}/scripts/report.sh" scripts/
  git add .github scripts
  git commit --quiet --message "chore: hold the scripts the job calls"
  branched=$(git rev-parse HEAD)

  git init --quiet --bare "${BATS_TEST_TMPDIR}/origin.git"
  git remote add origin "${BATS_TEST_TMPDIR}/origin.git"
  git push --quiet origin main
  git switch --quiet --create work
  commit "feat: one" "Signed-off-by: A Person <person@example.invalid>"
  commit "feat: two" "Signed-off-by: A Person <person@example.invalid>"
  git push --quiet origin work
  pushed=$(git rev-parse HEAD)

  export GH_REPO=owner/name PR=7 GATES="Quality Build" HEAD_REF=work
  export RUNNER_TEMP="${BATS_TEST_TMPDIR}/runner" GITHUB_OUTPUT="${BATS_TEST_TMPDIR}/output"
  mkdir -p "${RUNNER_TEMP}"
  touch "${GITHUB_OUTPUT}"

  reviews APPROVED:owner-1:42
  echo main >"${BATS_TEST_TMPDIR}/base"
  check_runs Quality=success Build=success
  fake_signing_gh
}

# One review for each `<state>:<login>:<id>`, in the order given.
reviews() {
  local review state login id one all=""
  for review in "$@"; do
    IFS=: read -r state login id <<<"${review}"
    printf -v one '{"state":"%s","user":{"login":"%s","id":%s}}' "${state}" "${login}" "${id}"
    all+="${all:+,}${one}"
  done
  printf '[%s]\n' "${all}" >"${BATS_TEST_TMPDIR}/reviews"
}

# A file named `refuse-post`, `refuse-read`, `refuse-reviews` or `refuse-base`
# makes that call fail.
fake_signing_gh() {
  fake gh <<'FAKE'
refuse() { [[ ! -e "${BATS_TEST_TMPDIR}/refuse-$1" ]] || exit 1; }
case "$*" in
  *"--method POST"*)
    refuse post
    echo "https://example.invalid/check-run"
    ;;
  *"/check-runs?"*)
    refuse read
    cat "${BATS_TEST_TMPDIR}/check-runs"
    ;;
  *"/pulls/7/reviews --paginate")
    refuse reviews
    cat "${BATS_TEST_TMPDIR}/reviews"
    ;;
  *"/pulls/7 --jq .base.ref")
    refuse base
    cat "${BATS_TEST_TMPDIR}/base"
    ;;
  *)
    echo "the fake does not answer: $*" >&2
    exit 9
    ;;
esac
FAKE
}

origin_holds() {
  local remote
  remote=$(git --git-dir="${BATS_TEST_TMPDIR}/origin.git" rev-parse work)
  [[ "${remote}" == "$1" ]]
}

@test "an approved branch whose gates are green is accepted, pushed and reported" {
  # A runner has no git identity, and the job must not need one.
  unset GIT_COMMITTER_NAME GIT_COMMITTER_EMAIL

  run "${script}"

  [[ "${status}" -eq 0 ]]
  head=$(git rev-parse HEAD)
  [[ "${head}" != "${pushed}" ]]
  origin_holds "${head}"
  [[ "${output}" == *"recorded acceptance by owner-1 on ${head}"* ]]
  owed="Accepted-by: owner-1 <42+owner-1@users.noreply.github.com>"
  trailers=$(git log --format='%(trailers:key=Accepted-by)' main..HEAD)
  carried=$(grep -c -x -F "${owed}" <<<"${trailers}")
  [[ "${carried}" -eq 2 ]]
  committer=$(git log -1 --format='%cn <%ce>')
  bot="github-actions[bot] <41898282+github-actions[bot]@users.noreply.github.com>"
  [[ "${committer}" == "${bot}" ]]
  written=$(cat "${GITHUB_OUTPUT}")
  [[ "${written}" == "before=${pushed}"$'\n'"after=${head}" ]]
  asked=$(calls gh)
  [[ "${asked}" == *"-f name=Sign-off -f head_sha=${head} -f status=completed"* ]]
  [[ "${asked}" == *"-f conclusion=success -f output[title]=Accepted by @owner-1"* ]]
}

@test "a base that moved on after the branch left it is not where the branch lands" {
  git switch --quiet --create later main
  commit "feat: later"
  git push --quiet origin later:main
  git switch --quiet work

  run "${script}"

  [[ "${status}" -eq 0 ]]
  kept=$(git rev-parse HEAD~2)
  [[ "${kept}" == "${branched}" ]]
}

@test "the approval that came last is the one accepted, and a comment is not one" {
  reviews APPROVED:owner-1:42 APPROVED:owner-2:43 COMMENTED:owner-3:44

  run "${script}"

  [[ "${status}" -eq 0 ]]
  [[ "${output}" == *"recorded acceptance by owner-2"* ]]
  carried=$(git log -1 --format='%(trailers:key=Accepted-by)')
  [[ "${carried}" == "Accepted-by: owner-2 <43+owner-2@users.noreply.github.com>" ]]
}

@test "with no approval it writes nothing and reads no gate" {
  reviews COMMENTED:owner-1:42

  run "${script}"

  [[ "${status}" -eq 0 ]]
  [[ "${output}" == *"no approval stands on this head, so there is nothing to accept"* ]]
  asked=$(calls gh)
  [[ "${asked}" != *"check-runs"* ]]
  origin_holds "${pushed}"
}

@test "a gate that is not green leaves nothing to accept, and the sentence is true" {
  for runs in "Quality=failure Build=success" "Build=success" "Quality=success Build="; do
    # One run for each word, so the list is split on purpose.
    # shellcheck disable=SC2086
    check_runs ${runs}

    run "${script}"

    [[ "${status}" -eq 0 ]]
    [[ "${output}" == *"a gate is not green on this head, so there is nothing to accept yet"* ]]
  done
  asked=$(calls gh)
  [[ "${asked}" != *"--method POST"* ]]
  origin_holds "${pushed}"
}

@test "check runs that cannot be read exit 2, and nothing is accepted" {
  touch "${BATS_TEST_TMPDIR}/refuse-read"

  run "${script}"

  [[ "${status}" -eq 2 ]]
  [[ "${output}" == *"the check runs on ${pushed} could not be read."* ]]
  [[ "${output}" == *"the gates on this head could not be read, so nothing was decided."* ]]
  [[ "${output}" != *"nothing to accept yet"* ]]
  asked=$(calls gh)
  [[ "${asked}" != *"--method POST"* ]]
  origin_holds "${pushed}"
}

@test "a gate read that fails in any other way exits 2 as well" {
  printf '#!/usr/bin/env bash\nexit 5\n' >.github/scripts/require-green.sh

  run "${script}"

  [[ "${status}" -eq 2 ]]
  [[ "${output}" == *"the gates on this head could not be read, so nothing was decided."* ]]
  origin_holds "${pushed}"
}

@test "a pull request that cannot be read exits 1, the code gh gives" {
  for call in reviews base; do
    rm -f "${BATS_TEST_TMPDIR}"/refuse-*
    touch "${BATS_TEST_TMPDIR}/refuse-${call}"

    run "${script}"

    [[ "${status}" -eq 1 ]]
  done
  asked=$(calls gh)
  [[ "${asked}" != *"--method POST"* ]]
  origin_holds "${pushed}"
}

@test "a login GitHub would not write is refused as text, and nothing is pushed" {
  # The login holds a command, and the refusal must print it rather than run it.
  # shellcheck disable=SC2016
  login='own$(id)er'
  reviews "APPROVED:${login}:42"

  run "${script}"

  [[ "${status}" -eq 1 ]]
  [[ "${output}" == *"the approver login has a character this job will not write"* ]]
  [[ "${output}" == *"into a commit: ${login}"* ]]
  origin_holds "${pushed}"
}

@test "an approver id that is not a number, or is absent, is refused" {
  for id in '"4x"' null; do
    reviews "APPROVED:owner-1:${id}"

    run "${script}"

    [[ "${status}" -eq 1 ]]
    [[ "${output}" == *"the approver id is not a number"* ]]
  done
  origin_holds "${pushed}"
}

@test "a commit with no sign-off is refused before anything is rewritten" {
  commit "feat: unsigned"
  git push --quiet origin work
  refused=$(git rev-parse HEAD)

  run "${script}"

  [[ "${status}" -eq 1 ]]
  head=$(git rev-parse HEAD)
  [[ "${head}" == "${refused}" ]]
  origin_holds "${refused}"
  [[ ! -s "${GITHUB_OUTPUT}" ]]
  asked=$(calls gh)
  [[ "${asked}" == *"-f name=Sign-off -f head_sha=${refused} -f status=completed"* ]]
  [[ "${asked}" == *"-f conclusion=failure"* ]]
  [[ "${asked}" == *"-f output[title]=Refused before anything was rewritten"* ]]
  [[ "${asked}" == *"A commit is refused when it carries no Signed-off-by trailer"* ]]
}

@test "a commit that replays empty is refused before anything is rewritten" {
  git commit --quiet --allow-empty --message "feat: nothing" \
    --message "Signed-off-by: A Person <person@example.invalid>"
  git push --quiet origin work
  refused=$(git rev-parse HEAD)

  run "${script}"

  [[ "${status}" -eq 1 ]]
  head=$(git rev-parse HEAD)
  [[ "${head}" == "${refused}" ]]
  origin_holds "${refused}"
  asked=$(calls gh)
  [[ "${asked}" == *"-f head_sha=${refused} -f status=completed -f conclusion=failure"* ]]
  [[ "${asked}" == *"A commit is refused when it replays empty"* ]]
}

@test "a rule that cannot run is reported as a check that could not run" {
  printf '#!/usr/bin/env bash\nexit 2\n' >scripts/check-origin.sh

  run "${script}"

  [[ "${status}" -eq 1 ]]
  asked=$(calls gh)
  [[ "${asked}" == *"-f conclusion=failure -f output[title]=This check could not run"* ]]
  [[ "${asked}" == *"This job stopped without writing anything a reader can use."* ]]
  origin_holds "${pushed}"
}

@test "a refusal whose check run cannot be written exits 2" {
  commit "feat: unsigned"
  refused=$(git rev-parse HEAD)
  touch "${BATS_TEST_TMPDIR}/refuse-post"

  run "${script}"

  [[ "${status}" -eq 2 ]]
  [[ "${output}" == *"the check run Sign-off could not be written onto ${refused}."* ]]
}

@test "an acceptance whose check run cannot be written exits 2, after the push" {
  touch "${BATS_TEST_TMPDIR}/refuse-post"

  run "${script}"

  [[ "${status}" -eq 2 ]]
  head=$(git rev-parse HEAD)
  [[ "${head}" != "${pushed}" ]]
  origin_holds "${head}"
  [[ ! -s "${GITHUB_OUTPUT}" ]]
  [[ "${output}" == *"the check run Sign-off could not be written onto ${head}."* ]]
  [[ "${output}" != *"recorded acceptance"* ]]
}

@test "a second run on an accepted branch pushes nothing and reports again" {
  "${script}"
  accepted=$(git rev-parse HEAD)
  : >"${GITHUB_OUTPUT}"
  git switch --quiet --create elsewhere
  commit "feat: three" "Signed-off-by: A Person <person@example.invalid>"
  git push --quiet origin elsewhere:work
  moved=$(git rev-parse HEAD)
  git switch --quiet work

  run "${script}"

  [[ "${status}" -eq 0 ]]
  head=$(git rev-parse HEAD)
  [[ "${head}" == "${accepted}" ]]
  origin_holds "${moved}"
  written=$(cat "${GITHUB_OUTPUT}")
  [[ "${written}" == "before=${accepted}"$'\n'"after=${accepted}" ]]
  asked=$(calls gh)
  last=${asked##*$'\n'}
  [[ "${last}" == *"-f head_sha=${accepted} -f status=completed -f conclusion=success"* ]]
}

@test "a branch onto another branch rewrites only its own commits" {
  git switch --quiet --create below main
  commit "feat: below" "Signed-off-by: A Person <person@example.invalid>"
  git push --quiet origin below
  below=$(git rev-parse HEAD)
  git switch --quiet work
  git rebase --quiet below
  git push --quiet --force origin work
  echo below >"${BATS_TEST_TMPDIR}/base"

  run "${script}"

  [[ "${status}" -eq 0 ]]
  kept=$(git rev-parse HEAD~2)
  [[ "${kept}" == "${below}" ]]
  trailers=$(git log --format='%(trailers:key=Accepted-by)' below..HEAD)
  carried=$(grep -c . <<<"${trailers}")
  [[ "${carried}" -eq 2 ]]
}

@test "a variable the workflow supplies stops it when absent" {
  for name in GH_REPO PR GATES HEAD_REF RUNNER_TEMP GITHUB_OUTPUT; do
    run env -u "${name}" "${script}"

    [[ "${status}" -eq 1 ]]
    [[ "${output}" == *"${name}: set by the"* ]]
  done
  asked=$(calls gh)
  [[ -z "${asked}" ]]
}
