# shellcheck shell=bash
# The three shapes every caller of the check-runs API repeats.
#
#   the decision:  docs/decisions/liability-is-recorded-from-the-act-that-makes-it-true.md
#
#   check_runs_of <head>           prints the API payload for that commit
#   last_conclusion <context>      reads that payload on standard input
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
    ! jq -e '.check_runs | type == "array"' >/dev/null <<<"${runs}"; then
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
