#!/usr/bin/env bash
# Carry a verdict across a rewrite that changed no content.
#
#   the decision:  docs/decisions/liability-is-recorded-from-the-act-that-makes-it-true.md
#   what you owe:  CONTRIBUTING.md
#
#   carry-verdicts.sh <before> <after> <context>...
#
# Never name a context here that reads a commit message. The rewrite edits the
# thing it reads, so its answer is genuinely different and carrying it lies.
#
# Exits 0 whether or not anything was carried, and 2 when it cannot run. The
# caller must not let a failure here fail the signing job: a branch that is
# signed and uncarried is recovered by restarting the run, and one that is
# unsigned is not.
set -euo pipefail

if [ $# -lt 3 ]; then
  echo "usage: carry-verdicts.sh <before> <after> <context>..." >&2
  exit 2
fi

before=$1
after=$2
shift 2

# Nothing moved, so the verdicts are already where they belong. Without this
# the loop below would read a commit's own verdict and post it back onto it.
if [ "$before" = "$after" ]; then
  echo "the head did not move, so there is nothing to carry"
  exit 0
fi

before_tree=$(git rev-parse "$before^{tree}")
after_tree=$(git rev-parse "$after^{tree}")

echo "before $before tree $before_tree"
echo "after  $after tree $after_tree"

if [ "$before_tree" != "$after_tree" ]; then
  echo "the content changed, so no verdict is carried"
  exit 0
fi

for context in "$@"; do
  # The API does not promise this array is in any order, so read the one that
  # finished last rather than the one that happens to be last.
  conclusion=$(gh api "repos/$GH_REPO/commits/$before/check-runs?per_page=100" |
    jq -r --arg c "$context" '
      [.check_runs[] | select(.name == $c) | select(.completed_at != null)]
      | sort_by(.completed_at) | last | .conclusion // empty')

  # Absent reads the same as refused here, on purpose. A missing verdict
  # leaves the context missing, which blocks the merge until somebody
  # restarts the run.
  case "$conclusion" in
    # Never widen this list.
    success | failure) ;;
    "") echo "$context has no verdict on $before, so there is nothing to carry"; continue ;;
    *) echo "$context concluded $conclusion on $before, which is not a verdict, so there is nothing to carry"; continue ;;
  esac

  # The conclusion goes across as it stands. A refusal stays a refusal.
  gh api --method POST "repos/$GH_REPO/check-runs" \
    -f name="$context" \
    -f head_sha="$after" \
    -f status=completed \
    -f conclusion="$conclusion" \
    -f "output[title]=$context, carried" \
    -f "output[summary]=The tree at this commit is \`$after_tree\`, which is the tree this check answered for at \`$before\`." \
    --jq '.html_url'

  echo "carried $context=$conclusion"
done
