#!/usr/bin/env bash
# The origin rule.
#
#   the decision:  docs/decisions/liability-is-recorded-from-the-act-that-makes-it-true.md
#   what you owe:  CONTRIBUTING.md
#
#   check-origin.sh <base> <head>
#
# Exits 1 when a commit breaks the rule, and 2 when this script cannot run.
#
# It also writes the failure, so a change to the wording reaches one place.
set -euo pipefail

# shellcheck source=.github/scripts/report.sh
. "$(dirname "$0")/report.sh"

if [ $# -ne 2 ]; then
  echo "usage: check-origin.sh <base> <head>" >&2
  exit 2
fi

if ! commits=$(git rev-list "$1".."$2" 2>&1); then
  report "the origin check could not read the range $1..$2." "$commits"
  exit 2
fi

# Read the trailer, never the message. The decision above says why.
#
# The address must sit inside angle brackets, because CONTRIBUTING.md asks a
# sign-off to reach someone.
missing=$(printf '%s\n' "$commits" | while read -r sha; do
  [ -n "$sha" ] || continue
  if ! git log -1 --format='%(trailers:key=Signed-off-by,valueonly)' "$sha" |
    grep -q '<.*@.*>'; then
    git log -1 --format='%h %s' "$sha"
  fi
done)

if [ -n "$missing" ]; then
  summary="these commits carry no Signed-off-by trailer, so this branch cannot be accepted. CONTRIBUTING.md has the command that adds it."

  # `--no-walk` is load-bearing, and it was measured. Without it this walks the
  # ancestors of each unsigned commit, so a signed merge commit below one of
  # them makes the sentence fire against a list that holds no merge.
  if printf '%s\n' "$missing" | cut -d' ' -f1 |
    git rev-list --no-walk --merges --stdin | grep -q .; then
    summary="$summary A merge commit below is not one you can sign."
  fi

  report "$summary" "$missing"
  exit 1
fi
