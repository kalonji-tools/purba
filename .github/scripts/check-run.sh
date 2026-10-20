# shellcheck shell=bash
# What every caller of the check-runs API reads and writes.
#
#   the decision:  docs/decisions/liability-is-recorded-from-the-act-that-makes-it-true.md
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
#
# A read or a write the API refuses exits 2, because the caller cannot run.
if [[ -z "${GH_REPO:-}" ]]; then
  echo "GH_REPO is set by the workflow env, and it is empty here." >&2
  exit 2
fi

# The shape is read here, once, so `last_conclusion` cannot fail on it.
check_runs_of() {
  local runs
  if ! runs=$(gh api "repos/${GH_REPO}/commits/$1/check-runs?per_page=100") ||
    ! jq -e '.check_runs | type == "array" and all(.[]; type == "object")' \
      >/dev/null <<<"${runs}"; then
    echo "the check runs on $1 could not be read." >&2
    exit 2
  fi
  printf '%s\n' "${runs}"
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
  gh api --method POST "repos/${GH_REPO}/check-runs" \
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
