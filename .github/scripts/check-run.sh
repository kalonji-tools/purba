# shellcheck shell=bash
# The two shapes every caller of the check-runs API repeats.
#
#   the decision:  docs/decisions/liability-is-recorded-from-the-act-that-makes-it-true.md
#
#   post_check_run <name> <head> <conclusion> <title> <summary>
#   last_conclusion <context>      reads the API payload on standard input
#
#   GH_REPO        the repository the check run is written to
#
# `status` is always `completed`: purba writes a verdict or writes nothing.
post_check_run() {
  : "${GH_REPO:?set by the caller}"
  gh api --method POST "repos/${GH_REPO}/check-runs" \
    -f name="$1" \
    -f head_sha="$2" \
    -f status=completed \
    -f conclusion="$3" \
    -f "output[title]=$4" \
    -f "output[summary]=$5" \
    --jq '.html_url'
}

# The API promises no order, so the run that finished last is read rather than
# the one that happens to be last.
last_conclusion() {
  jq -r --arg c "$1" '
    [.check_runs[] | select(.name == $c) | select(.completed_at != null)]
    | sort_by(.completed_at) | last | .conclusion // empty'
}
