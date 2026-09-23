#!/usr/bin/env bash
# Carry a contribution from a fork into this repository.
#
#   the decision:  docs/decisions/an-outside-contribution-is-applied-not-merged.md
#   what you owe:  CONTRIBUTING.md
#
#   apply-fork-contribution.sh <pull-request-number>
#
# Exits 1 when it refuses, and 2 when it cannot run.
set -euo pipefail

if [ $# -ne 1 ]; then
  echo "usage: apply-fork-contribution.sh <pull-request-number>" >&2
  exit 2
fi

pr=$1
root=$(git rev-parse --show-toplevel)
me=$(gh api user --jq .login)

if grep -oE '@[A-Za-z0-9-]+' "$root/CODEOWNERS" | tr -d '@' | grep -qxF "$me"; then
  echo "refused: $me is a code owner, so $me cannot approve a pull request that $me opens." >&2
  echo "Run this as the account that opens pull requests here." >&2
  exit 1
fi

git fetch --quiet origin main
git fetch --quiet origin "refs/pull/$pr/head"
base=$(git merge-base origin/main FETCH_HEAD)

"$root/.github/scripts/check-origin.sh" "$base" FETCH_HEAD

if git diff --name-only "$base" FETCH_HEAD | grep -q '^\.github/'; then
  echo "warning: this contribution changes .github/, so its workflows run with this repository's token once it is a branch here, before anyone approves it." >&2
fi

git push --quiet --force origin "FETCH_HEAD:refs/heads/accepted/pr-$pr"

url=$(gh pr list --head "accepted/pr-$pr" --state open --json url --jq '.[0].url // empty')
if [ -z "$url" ]; then
  url=$(gh pr create --base main --head "accepted/pr-$pr" \
    --title "$(gh pr view "$pr" --json title --jq .title)" --body \
"Carries the work proposed in #$pr, unchanged.

purba does not sign a branch that lives on a fork, so this branch is what merges.")

  gh pr comment "$pr" --body \
"Your work is on \`accepted/pr-$pr\` in this repository, and $url is what merges.

A workflow here cannot write to your fork, so purba carries a contribution in rather than merging it from one. Your commits are unchanged: the author field and your \`Signed-off-by:\` trailer stay yours. CONTRIBUTING.md states what purba records on its side."
fi

cat <<EOF

applied #$pr to accepted/pr-$pr
pull request  $url

After $url merges, close the contribution:

  gh pr close $pr --comment "Merged as $url."
EOF
