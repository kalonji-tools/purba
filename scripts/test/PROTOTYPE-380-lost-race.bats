# PROTOTYPE for #380 — throwaway, never merged. Runs both lost-race variants and prints the state each leaves.
: "${BATS_TEST_DIRNAME:?set by bats}"
bats_require_minimum_version 1.5.0

setup() {
  # shellcheck source=scripts/test/fixture.sh
  . "${BATS_TEST_DIRNAME}/fixture.sh"
  tree="${BATS_TEST_DIRNAME}/../.."
  dir="${tree}/.github/scripts"
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

held() {
  git --git-dir="${BATS_TEST_TMPDIR}/origin.git" rev-parse work
}

origin_holds() {
  local remote
  remote=$(held)
  [[ "${remote}" == "$1" ]]
}

row() {
  local subjects posted last
  subjects=$(git --git-dir="${BATS_TEST_TMPDIR}/origin.git" log --format=%s work 2>/dev/null | grep '^feat' | paste -sd' ' || true)
  posted=$(calls gh | grep -o 'conclusion=[a-z]*' | paste -sd, || true)
  last=$(grep -v '^ *$' <<<"${output}" | tail -1)
  echo "| $1 | $2 | ${status} | ${subjects:-<branch gone>} | ${posted:-nothing} | ${last} |" >&3
}

moved() {
  git switch --quiet --create elsewhere
  commit "feat: three" "Signed-off-by: A Person <person@example.invalid>"
  git push --quiet origin elsewhere:work
  git switch --quiet work
}

deleted() { git push --quiet origin :work; }

refused_by_remote() {
  printf '#!/usr/bin/env bash\necho "a rule on the remote refuses this push" >&2\nexit 1\n' \
    >"${BATS_TEST_TMPDIR}/origin.git/hooks/pre-receive"
  chmod +x "${BATS_TEST_TMPDIR}/origin.git/hooks/pre-receive"
}

# The next run: the runner checks out what the branch holds now.
next_run() {
  rm -f "${BATS_TEST_TMPDIR}/calls.gh"
  : >"${GITHUB_OUTPUT}"
  git fetch --quiet origin work
  git switch --quiet --force-create work FETCH_HEAD
}

for variant in let-git-fail exit-0; do
  bats_test_function --description "${variant}: no race" -- case_none "${variant}"
  bats_test_function --description "${variant}: branch moved" -- case_moved "${variant}"
  bats_test_function --description "${variant}: branch deleted" -- case_deleted "${variant}"
  bats_test_function --description "${variant}: remote refuses, branch unmoved" -- case_refused "${variant}"
  bats_test_function --description "${variant}: next run after the race" -- case_next "${variant}"
done

case_none() { run "${dir}/PROTOTYPE-380-$1.sh"; row "$1" "no race"; }
case_moved() { moved; run "${dir}/PROTOTYPE-380-$1.sh"; row "$1" "branch moved"; }
case_deleted() { deleted; run "${dir}/PROTOTYPE-380-$1.sh"; row "$1" "branch deleted"; }
case_refused() { refused_by_remote; run "${dir}/PROTOTYPE-380-$1.sh"; row "$1" "remote refuses, unmoved"; }
case_next() {
  moved
  run "${dir}/PROTOTYPE-380-$1.sh"
  next_run
  # The new head has no gate runs yet when its own push event starts the run.
  check_runs Quality= Build=
  run "${dir}/PROTOTYPE-380-$1.sh"
  row "$1" "next run, gates not yet run"
  next_run
  check_runs Quality=success Build=success
  run "${dir}/PROTOTYPE-380-$1.sh"
  row "$1" "next run, gates green"
  head=$(git --git-dir="${BATS_TEST_TMPDIR}/origin.git" rev-parse work)
  accepted=$(git log --format='%(trailers:key=Accepted-by,valueonly)' "main..${head}" | grep -c . || true)
  echo "|  | commits on main..work carrying Accepted-by: ${accepted} of 3 |" >&3
}
