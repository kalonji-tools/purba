#!/usr/bin/env bash
# The sign-off a person owes for a branch.
#
#   what you owe:  CONTRIBUTING.md
#   the decision:  docs/decisions/liability-is-recorded-from-the-act-that-makes-it-true.md
#
#   sign-branch.sh [<base>]
#
# Exits 1 when nothing was signed. Exits 2 when this script cannot run.
set -euo pipefail

# Both tests are load-bearing. Without this one, a piped answer satisfies the
# question below.
if [ ! -t 0 ]; then
  echo "sign-branch.sh ran with no terminal attached, so nothing was signed." >&2
  exit 1
fi

base=${1:-origin/main}

if ! start=$(git merge-base "$base" HEAD 2>&1); then
  echo "sign-branch.sh cannot find where your branch leaves $base." >&2
  echo "$start" >&2
  exit 2
fi

"$(git rev-parse --show-toplevel)/.github/scripts/check-replayable.sh" "$base" HEAD

echo "Measured from $base:"
git --no-pager log --reverse --format='  %h  %an  %s' "$start..HEAD"

read -r -p "Sign these commits? [y/N] " reply || reply=""
if [ "$reply" != "y" ]; then
  echo "nothing was signed." >&2
  exit 1
fi

# Keep this on one line. git refuses an exec command that contains a newline.
git rebase "$start" --exec \
  'git log -1 --format="%(trailers:key=Signed-off-by)" | grep -q . || git commit --amend --no-edit -s'
