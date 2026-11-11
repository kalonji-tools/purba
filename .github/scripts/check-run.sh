# shellcheck shell=bash
# What every caller of the check-runs API reads and writes.
#
#   the decision:  docs/decisions/liability-is-recorded-from-the-act-that-makes-it-true.md
#   who reports:   docs/decisions/a-required-check-of-main-counts-only-from-github-actions.md
#
#   verdicts <head> <context>...   prints `<context> <conclusion>` for each, in
#                                  the order given, and `none` for no conclusion
#   require_green <head> <context>...
#                                  returns 0 when every context is green, and 1
#                                  when one is not. With no context it exits 2
#   post_check_run <name> <head> <conclusion> <title> <summary>
#
#   GH_REPO        the repository the check runs belong to. A caller that
#                  sources this file without it exits 2
#   HEAD_REF       the branch of the pull request. A read with none exits 2
#   SIGN_TOKEN     the token of `purba-sign`, which posts each check run. A
#                  write with none exits 2
#
# A read or a write the API refuses exits 2, because the caller cannot run.
if [[ -z "${GH_REPO:-}" ]]; then
  echo "GH_REPO is set by the workflow env, and it is empty here." >&2
  exit 2
fi

# The check runs of a head hold only the newest run of each name, so a later
# post would hide a job there. So this function reads the verdict from the
# job itself.
#
# The shape is read here, once, so `last_conclusion` cannot fail on it.
check_runs_of() {
  local runs made jobs='[]' ids one id query
  if [[ -z "${HEAD_REF:-}" ]]; then
    echo "HEAD_REF is set by the workflow env, and it is empty here." >&2
    exit 2
  fi
  if ! runs=$(gh api "repos/${GH_REPO}/commits/$1/check-runs?per_page=100") ||
    ! jq -e '.check_runs | type == "array" and all(.[]; type == "object")' \
      >/dev/null <<<"${runs}"; then
    echo "the check runs on $1 could not be read." >&2
    exit 2
  fi
  query="head_sha=$1&event=pull_request&branch=${HEAD_REF}&per_page=100"
  if ! made=$(gh api "repos/${GH_REPO}/actions/runs?${query}") ||
    ! jq -e '.workflow_runs | type == "array" and all(.[]; type == "object")' \
      >/dev/null <<<"${made}"; then
    echo "the workflow runs on $1 could not be read." >&2
    exit 2
  fi
  ids=$(jq -r --arg branch "${HEAD_REF}" '.workflow_runs[]
    | select(.event == "pull_request" and .head_branch == $branch) | .id' <<<"${made}")
  # Each payload reaches jq on standard input, never as an argument: Linux
  # refuses one argument longer than 131072 bytes, and the check runs of a head
  # with many jobs grow past that.
  # https://man7.org/linux/man-pages/man2/execve.2.html
  # `filter=latest` reads the last attempt of a run, so a rerun replaces it.
  while read -r id; do
    [[ -n "${id}" ]] || continue
    if ! one=$(gh api "repos/${GH_REPO}/actions/runs/${id}/jobs?filter=latest&per_page=100") ||
      ! jq -e '.jobs | type == "array" and all(.[]; type == "object")' >/dev/null <<<"${one}"; then
      echo "the jobs of the workflow run ${id} could not be read." >&2
      exit 2
    fi
    jobs=$(printf '%s\n%s' "${jobs}" "${one}" | jq -sc '.[0] + .[1].jobs')
  done <<<"${ids}"
  printf '%s\n%s\n%s' "${runs}" "${made}" "${jobs}" | jq -s --arg branch "${HEAD_REF}" '
    .[0] as $runs | .[1] as $made | .[2] as $jobs
    | [$made.workflow_runs[]
      | select(.event == "pull_request" and .head_branch == $branch)
      | {key: (.id | tostring), value: .path}]
    | from_entries as $file
    | {check_runs: ([$runs.check_runs[] | select(.app.slug == "purba-sign")]
        + [$jobs[]
          | select(.runner_id != null
              and $file[(.run_id | tostring)]
                == ".github/workflows/\(.name | split("-")[0] | ascii_downcase).yml")])}'
}

# The API promises no order, so the run that finished last is read rather than
# the one that happens to be last.
last_conclusion() {
  jq -r --arg c "$1" '
    [.check_runs[] | select(.name == $c)]
    | sort_by(.completed_at) | last | .conclusion // empty'
}

# A caller behind `||` or `if` turns `set -e` off inside a function, so each
# read that fails exits here, or the function would go on with no payload.
verdicts() {
  local runs context conclusion
  runs=$(check_runs_of "$1") || exit 2
  shift
  for context in "$@"; do
    conclusion=$(last_conclusion "${context}" <<<"${runs}")
    echo "${context} ${conclusion:-none}"
  done
}

require_green() {
  local head=$1 lines context conclusion
  # With no context the loop below reads one empty line and returns 1, as if a
  # gate had failed. A list that names nothing is the caller's mistake.
  if [[ $# -lt 2 ]]; then
    echo "usage: require_green <head> <context>..." >&2
    exit 2
  fi
  shift
  lines=$(verdicts "${head}" "$@") || exit 2

  while read -r context conclusion; do
    case "${conclusion}" in
      success) echo "${context} is green on ${head}" ;;
      none)
        echo "${context} has no verdict on ${head} yet"
        return 1
        ;;
      *)
        echo "${context} concluded ${conclusion} on ${head}"
        return 1
        ;;
    esac
  done <<<"${lines}"

  echo "every gate named here is green on ${head}"
}

# `status` is always `completed`: purba writes a verdict or writes nothing.
post_check_run() {
  if [[ -z "${SIGN_TOKEN:-}" ]]; then
    echo "SIGN_TOKEN is set by the workflow env, and it is empty here." >&2
    exit 2
  fi
  GH_TOKEN="${SIGN_TOKEN}" gh api --method POST "repos/${GH_REPO}/check-runs" \
    -f name="$1" \
    -f head_sha="$2" \
    -f status=completed \
    -f conclusion="$3" \
    -f "output[title]=$4" \
    -f "output[summary]=$5" \
    --jq '.html_url' || {
    echo "the check run $1 could not be written onto $2." >&2
    exit 2
  }
}
