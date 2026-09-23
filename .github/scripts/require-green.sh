#!/usr/bin/env bash
# Refuse to go on until every gate this workflow does not post has a verdict.
#
#   the decision:  docs/decisions/liability-is-recorded-from-the-act-that-makes-it-true.md
#   what you owe:  CONTRIBUTING.md
#
#   require-green.sh <head> <context>...
#
# Exits 0 when every context is green, 1 when one is not, 2 when it cannot run.
set -euo pipefail

if [ $# -lt 2 ]; then
  echo "usage: require-green.sh <head> <context>..." >&2
  exit 2
fi

head=$1
shift

runs=$(gh api "repos/$GH_REPO/commits/$head/check-runs?per_page=100")

for context in "$@"; do
  # Not ordered by the API, so read the one that finished last.
  conclusion=$(jq -r --arg c "$context" '
    [.check_runs[] | select(.name == $c) | select(.completed_at != null)]
    | sort_by(.completed_at) | last | .conclusion // empty' <<<"$runs")

  case "$conclusion" in
    success) echo "$context is green on $head" ;;
    "") echo "$context has no verdict on $head yet"; exit 1 ;;
    *) echo "$context concluded $conclusion on $head"; exit 1 ;;
  esac
done

echo "every gate named here is green on $head"
