# .github/scripts/write-acceptance-trailer.sh against the branches it accepts and refuses.
: "${BATS_TEST_DIRNAME:?set by bats}"
bats_require_minimum_version 1.5.0

setup() {
  # shellcheck source=scripts/test/fixture.sh
  . "${BATS_TEST_DIRNAME}/fixture.sh"
  tree="${BATS_TEST_DIRNAME}/../.."
  # The job runs the scripts of `main`, from a checkout apart from the branch.
  trusted="${BATS_TEST_TMPDIR}/trusted"
  mkdir -p "${trusted}/.github/scripts" "${trusted}/scripts"
  cp "${tree}/.github/scripts/write-acceptance-trailer.sh" "${tree}/.github/scripts/check-run.sh" \
    "${tree}/.github/scripts/standing-approver.sh" "${tree}/.github/scripts/carry-verdicts.sh" \
    "${trusted}/.github/scripts/"
  cp "${tree}/scripts/check-origin.sh" "${tree}/scripts/check-replayable.sh" \
    "${tree}/scripts/accept-one-commit.sh" "${tree}/scripts/report.sh" "${trusted}/scripts/"
  script="${trusted}/.github/scripts/write-acceptance-trailer.sh"
  make_repo
  branched=$(git rev-parse HEAD)

  git init --quiet --bare "${BATS_TEST_TMPDIR}/origin.git"
  git remote add origin "${BATS_TEST_TMPDIR}/origin.git"
  git push --quiet origin main
  git switch --quiet --create work
  commit "feat: one" "Signed-off-by: A Person <person@example.invalid>"
  commit "feat: two" "Signed-off-by: A Person <person@example.invalid>"
  git push --quiet origin work
  pushed=$(git rev-parse HEAD)

  export GH_REPO=owner/name BASE_REF=main GATES="Quality Build" CARRYABLE="Quality Build" \
    HEAD_REF=work SIGN_TOKEN=sign-token
  export RUNNER_TEMP="${BATS_TEST_TMPDIR}/runner"
  mkdir -p "${RUNNER_TEMP}"

  reviews APPROVED:owner-1:42
  base main
  holders "7:open:main:${pushed}"
  check_runs Quality=success Build=success
  fake_gh
}

# One review for each `<state>:<login>:<id>`, in the order given.
reviews() {
  local review state login id one all=""
  for review in "$@"; do
    IFS=: read -r state login id <<<"${review}"
    printf -v one '{"state":"%s","user":{"login":"%s","id":%s}}' "${state}" "${login}" "${id}"
    all+="${all:+,}${one}"
  done
  gh_answers reviews "[${all}]"
  gh_answers permission '{"permission":"admin"}'
}

# The open pull request from the branch targets <branch>, as the API lists it.
base() {
  gh_answers pulls "[{\"number\":7,\"base\":{\"ref\":\"$1\"}}]"
}

# One pull request that holds the head for each `<number>:<state>:<base>:<sha of its head>`.
holders() {
  local one number state base sha all=""
  for one in "$@"; do
    IFS=: read -r number state base sha <<<"${one}"
    printf -v one '{"number":%s,"state":"%s","base":{"ref":"%s"},"head":{"sha":"%s"}}' \
      "${number}" "${state}" "${base}" "${sha}"
    all+="${all:+,}${one}"
  done
  gh_answers commit-pulls "[${all}]"
}

held() {
  git --git-dir="${BATS_TEST_TMPDIR}/origin.git" rev-parse work
}

origin_holds() {
  local remote
  remote=$(held)
  [[ "${remote}" == "$1" ]]
}

@test "an approved branch whose gates are green is accepted, pushed and reported" {
  # A runner has no git identity, and the job must not need one.
  unset GIT_COMMITTER_NAME GIT_COMMITTER_EMAIL

  run "${script}"

  [[ "${status}" -eq 0 ]]
  head=$(held)
  [[ "${head}" != "${pushed}" ]]
  [[ "${output}" == *"recorded acceptance by owner-1 on ${head}"* ]]
  owed="Accepted-by: owner-1 <42+owner-1@users.noreply.github.com>"
  trailers=$(git log --format='%(trailers:key=Accepted-by)' "main..${head}")
  carried=$(grep -c -x -F "${owed}" <<<"${trailers}")
  [[ "${carried}" -eq 2 ]]
  committer=$(git log -1 --format='%cn <%ce>' "${head}")
  bot="github-actions[bot] <41898282+github-actions[bot]@users.noreply.github.com>"
  [[ "${committer}" == "${bot}" ]]
  asked=$(calls gh)
  [[ "${asked}" == *"api repos/owner/name/pulls/7/reviews --paginate"* ]]
  [[ "${asked}" == *"-f name=Sign-off -f head_sha=${head} -f status=completed"* ]]
  [[ "${asked}" == *"-f conclusion=success -f output[title]=Accepted by @owner-1"* ]]
  [[ "${asked}" == *"-f name=Sign-off"*"-f name=Quality -f head_sha=${head}"* ]]
  [[ "${asked}" == *"-f name=Build -f head_sha=${head}"* ]]
  [[ "${output}" == *"before ${pushed} tree"* ]]
}

@test "an accepted branch whose carry cannot carry posts a failed Carry, and exits 0" {
  gh_refuses post-check-runs-Quality

  run "${script}"

  [[ "${status}" -eq 0 ]]
  head=$(held)
  asked=$(calls gh)
  [[ "${asked}" == *"-f name=Sign-off -f head_sha=${head}"*"-f conclusion=success"* ]]
  [[ "${asked}" == *"-f name=Carry -f head_sha=${head}"*"-f conclusion=failure"* ]]
}

@test "an accepted branch whose carry cannot post Carry exits 2, after Sign-off" {
  gh_refuses post-check-runs-Quality
  gh_refuses post-check-runs-Carry

  run "${script}"

  [[ "${status}" -eq 2 ]]
  head=$(held)
  asked=$(calls gh)
  [[ "${asked}" == *"-f name=Sign-off -f head_sha=${head}"*"-f conclusion=success"* ]]
  [[ "${output}" == *"the check run Carry could not be written onto ${head}."* ]]
}

@test "a base that moved on after the branch left it is not where the branch lands" {
  git switch --quiet --create later main
  commit "feat: later"
  git push --quiet origin later:main
  git switch --quiet work

  run "${script}"

  [[ "${status}" -eq 0 ]]
  head=$(held)
  kept=$(git rev-parse "${head}~2")
  [[ "${kept}" == "${branched}" ]]
}

@test "the head it pushes is the head the replay check printed" {
  # shellcheck disable=SC2016
  printf '#!/usr/bin/env bash\ncat "${BATS_TEST_TMPDIR}/printed"\n' \
    >"${trusted}/scripts/check-replayable.sh"
  printed=$(git commit-tree -p HEAD -m "feat: as replayed" 'HEAD^{tree}')
  echo "${printed}" >"${BATS_TEST_TMPDIR}/printed"

  run "${script}"

  [[ "${status}" -eq 0 ]]
  origin_holds "${printed}"
  asked=$(calls gh)
  [[ "${asked}" == *"-f name=Sign-off -f head_sha=${printed} -f status=completed"* ]]
}

@test "a rule that the branch holds is not the rule that runs" {
  mkdir -p scripts
  # shellcheck disable=SC2016
  printf '#!/usr/bin/env bash\ntouch "${BATS_TEST_TMPDIR}/the branch ran ${0##*/}"\n' \
    >scripts/check-origin.sh
  cp scripts/check-origin.sh scripts/check-replayable.sh
  chmod +x scripts/check-origin.sh scripts/check-replayable.sh
  git add scripts
  git commit --quiet --message "chore: rules of its own" \
    --message "Signed-off-by: A Person <person@example.invalid>"
  git push --quiet origin work

  run "${script}"

  ran=$(find "${BATS_TEST_TMPDIR}" -maxdepth 1 -name 'the branch ran *')
  [[ -z "${ran}" ]]
  [[ "${status}" -eq 0 ]]
}

@test "a commit pushed after the checkout stays on the branch, and nothing is accepted" {
  git switch --quiet --create elsewhere
  commit "feat: three" "Signed-off-by: A Person <person@example.invalid>"
  git push --quiet origin elsewhere:work
  moved=$(git rev-parse HEAD)
  git switch --quiet work

  run "${script}"

  [[ "${status}" -eq 1 ]]
  origin_holds "${moved}"
  asked=$(calls gh)
  [[ "${asked}" != *"--method POST"* ]]
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
  gh_refuses check-runs

  run "${script}"

  [[ "${status}" -eq 2 ]]
  [[ "${output}" == *"the check runs on ${pushed} could not be read."* ]]
  [[ "${output}" != *"nothing to accept yet"* ]]
  asked=$(calls gh)
  [[ "${asked}" != *"--method POST"* ]]
  origin_holds "${pushed}"
}

@test "reviews that cannot be read exit 2, and nothing is accepted" {
  gh_refuses reviews

  run "${script}"

  [[ "${status}" -eq 2 ]]
  [[ "${output}" == *"the reviews of pull request 7 could not be read, so no approval was read."* ]]
  asked=$(calls gh)
  [[ "${asked}" != *"--method POST"* ]]
  origin_holds "${pushed}"
}

@test "open pull requests that cannot be read exit 2, and nothing is accepted" {
  for answer in refused '{"message":"Not Found"}' '{"x":{"number":3,"base":{"ref":"main"}}}'; do
    rm -f "${BATS_TEST_TMPDIR}"/gh/pulls.refused
    if [[ "${answer}" == refused ]]; then
      gh_refuses pulls
    else
      gh_answers pulls "${answer}"
    fi

    run "${script}"

    [[ "${status}" -eq 2 ]]
    [[ "${output}" == *"the open pull requests from work into main could not be read."* ]]
  done
  asked=$(calls gh)
  [[ "${asked}" != *"--method POST"* ]]
  origin_holds "${pushed}"
}

@test "a PR from the environment is not the one it signs" {
  gh_answers pulls '[{"number":7,"base":{"ref":"main"}}]'

  PR=8 run "${script}"

  [[ "${status}" -eq 0 ]]
  [[ "${output}" == *"recorded acceptance by owner-1 on "* ]]
  asked=$(calls gh)
  [[ "${asked}" == *"api repos/owner/name/pulls?head=owner:work&base=main&state=open"* ]]
  [[ "${asked}" == *"api repos/owner/name/pulls/7/reviews --paginate"* ]]
  [[ "${asked}" != *"pulls/8"* ]]
}

@test "a login GitHub would not write exits 1, and nothing is pushed" {
  reviews "APPROVED:own;er:42"
  check_runs Quality=failure Build=success

  run "${script}"

  [[ "${status}" -eq 1 ]]
  origin_holds "${pushed}"
}

@test "a commit with no sign-off is refused before anything is rewritten" {
  commit "feat: unsigned"
  git push --quiet origin work
  refused=$(git rev-parse HEAD)

  run "${script}"

  [[ "${status}" -eq 0 ]]
  head=$(git rev-parse HEAD)
  [[ "${head}" == "${refused}" ]]
  origin_holds "${refused}"
  asked=$(calls gh)
  [[ "${asked}" != *"-f name=Quality"* ]]
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

  [[ "${status}" -eq 0 ]]
  head=$(git rev-parse HEAD)
  [[ "${head}" == "${refused}" ]]
  origin_holds "${refused}"
  asked=$(calls gh)
  [[ "${asked}" == *"-f head_sha=${refused} -f status=completed -f conclusion=failure"* ]]
  [[ "${asked}" == *"A commit is refused when it replays empty"* ]]
}

@test "on a runner the refusal of the replay check reaches the log once, on stderr" {
  git commit --quiet --allow-empty --message "feat: nothing" \
    --message "Signed-off-by: A Person <person@example.invalid>"

  GITHUB_ACTIONS=true run --separate-stderr "${script}"

  [[ "${status}" -eq 0 ]]
  [[ "${output}" != *"::error::"* ]]
  refusals=$(grep -cF "::error::A commit is refused when it replays empty" <<<"${stderr}")
  [[ "${refusals}" -eq 1 ]]
}

@test "a branch whose replay changes its content is refused, and nothing is pushed" {
  git switch --quiet --create side main
  commit "feat: side" "Signed-off-by: A Person <person@example.invalid>"
  git switch --quiet work
  git merge --quiet --no-commit --no-ff side
  printf 'made in the merge\n' >only-in-the-merge
  git add only-in-the-merge
  git commit --quiet --message "chore: merge and change" \
    --message "Signed-off-by: A Person <person@example.invalid>"
  git push --quiet origin work
  refused=$(git rev-parse HEAD)

  run "${script}"

  [[ "${status}" -eq 0 ]]
  origin_holds "${refused}"
  asked=$(calls gh)
  [[ "${asked}" != *"-f name=Quality"* ]]
  [[ "${asked}" == *"-f head_sha=${refused} -f status=completed -f conclusion=failure"* ]]
  [[ "${asked}" == *"A branch is refused when its replay changes its content"* ]]
}

@test "a rule that cannot run is reported as a check that could not run" {
  printf '#!/usr/bin/env bash\nexit 2\n' >"${trusted}/scripts/check-origin.sh"

  run "${script}"

  [[ "${status}" -eq 0 ]]
  asked=$(calls gh)
  [[ "${asked}" == *"-f conclusion=failure -f output[title]=This check could not run"* ]]
  [[ "${asked}" == *"This job stopped without writing anything a reader can use."* ]]
  origin_holds "${pushed}"
}

@test "a refusal whose check run cannot be written exits 2" {
  commit "feat: unsigned"
  refused=$(git rev-parse HEAD)
  gh_refuses post-check-runs

  run "${script}"

  [[ "${status}" -eq 2 ]]
  [[ "${output}" == *"the check run Sign-off could not be written onto ${refused}."* ]]
}

@test "an acceptance whose check run cannot be written exits 2, after the push" {
  gh_refuses post-check-runs

  run "${script}"

  [[ "${status}" -eq 2 ]]
  head=$(held)
  [[ "${head}" != "${pushed}" ]]
  [[ "${output}" == *"the check run Sign-off could not be written onto ${head}."* ]]
  asked=$(calls gh)
  [[ "${asked}" != *"-f name=Quality"* ]]
  [[ "${output}" != *"recorded acceptance"* ]]
}

@test "a second run on an accepted branch pushes nothing and reports again" {
  "${script}"
  accepted=$(held)
  # The next run checks out the head this one pushed.
  git reset --quiet --hard "${accepted}"
  git switch --quiet --create elsewhere
  commit "feat: three" "Signed-off-by: A Person <person@example.invalid>"
  git push --quiet origin elsewhere:work
  moved=$(git rev-parse HEAD)
  git switch --quiet work

  run "${script}"

  [[ "${status}" -eq 0 ]]
  origin_holds "${moved}"
  [[ "${output}" == *"the head did not move, so there is nothing to carry"* ]]
  asked=$(calls gh)
  last=${asked##*$'\n'}
  [[ "${last}" == *"-f head_sha=${accepted} -f status=completed -f conclusion=success"* ]]
}

@test "an accepted run names the pull request it reads" {
  run "${script}"

  [[ "${status}" -eq 0 ]]
  [[ "${output}" == *"pull request 7 from work into main is the one this run reads"* ]]
}

@test "two open pull requests into the base at one head get a failed Sign-off, and nothing else" {
  holders "7:open:main:${pushed}" "2:open:other:${pushed}" "5:closed:main:${pushed}" \
    "11:open:main:1111111111111111111111111111111111111111" "9:open:main:${pushed}"

  run "${script}"

  [[ "${status}" -eq 0 ]]
  asked=$(calls gh)
  [[ "${asked}" == *"api repos/owner/name/commits/${pushed}/pulls"* ]]
  [[ "${asked}" != *"reviews"* ]]
  [[ "${asked}" == *"-f name=Sign-off -f head_sha=${pushed} -f status=completed"* ]]
  [[ "${asked}" == *"-f conclusion=failure"* ]]
  [[ "${asked}" == *"-f output[title]=More than one pull request into main holds this head"* ]]
  [[ "${asked}" == *"#7, #9 into main"* ]]
  origin_holds "${pushed}"
}

@test "pull requests that hold the head that cannot be read exit 2, and nothing is accepted" {
  for answer in refused '{"message":"Not Found"}'; do
    rm -f "${BATS_TEST_TMPDIR}"/gh/commit-pulls.refused
    if [[ "${answer}" == refused ]]; then
      gh_refuses commit-pulls
    else
      gh_answers commit-pulls "${answer}"
    fi

    run "${script}"

    [[ "${status}" -eq 2 ]]
    [[ "${output}" == *"the pull requests that hold ${pushed} could not be read."* ]]
  done
  asked=$(calls gh)
  [[ "${asked}" != *"--method POST"* ]]
  origin_holds "${pushed}"
}

@test "a pull request into the base whose branch only holds the head leaves the job to sign" {
  holders "7:open:main:${pushed}" "11:open:main:1111111111111111111111111111111111111111"

  run "${script}"

  [[ "${status}" -eq 0 ]]
  [[ "${output}" == *"recorded acceptance by owner-1 on "* ]]
}

@test "a pull request into another branch gets nothing" {
  gh_answers pulls '[]'

  run "${script}"

  [[ "${status}" -eq 0 ]]
  [[ "${output}" == *"no open pull request from work targets main, so there is nothing"* ]]
  asked=$(calls gh)
  [[ "${asked}" != *"reviews"* ]]
  [[ "${asked}" != *"--method POST"* ]]
  origin_holds "${pushed}"
}

@test "a variable the workflow supplies stops it when absent" {
  for name in GH_REPO BASE_REF GATES CARRYABLE HEAD_REF RUNNER_TEMP; do
    run env -u "${name}" "${script}"

    [[ "${status}" -eq 1 ]]
    [[ "${output}" == *"${name}: set by the"* ]]
  done
  asked=$(calls gh)
  [[ -z "${asked}" ]]
}
