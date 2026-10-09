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

# shellcheck source=scripts/report.sh
. "$(dirname "$0")/report.sh"

if [[ $# -ne 1 ]]; then
  echo "usage: apply-fork-contribution.sh <pull-request-number>" >&2
  exit 2
fi

pr=$1
if ! root=$(git rev-parse --show-toplevel); then
  cannot "the top of the work tree could not be found, so nothing was pushed."
fi
if ! me=$(gh api user --jq .login); then
  cannot "the account gh acts as could not be read, so nothing was pushed."
fi

if grep -oE '@[A-Za-z0-9-]+' "${root}/.github/CODEOWNERS" | tr -d '@' | grep -qxF "${me}"; then
  echo "refused: ${me} is a code owner, so ${me} cannot approve a pull request that ${me} opens." \
    >&2
  echo "Run this as the account that opens pull requests here." >&2
  exit 1
fi

if ! git fetch --quiet origin main; then
  cannot "main could not be fetched, so nothing was pushed."
fi
if ! git fetch --quiet origin "refs/pull/${pr}/head"; then
  cannot "#${pr} could not be fetched, so nothing was pushed."
fi
if ! base=$(git merge-base origin/main FETCH_HEAD); then
  cannot "the merge base of #${pr} and main could not be found, so nothing was pushed."
fi

"$(dirname "$0")/check-origin.sh" "${base}" FETCH_HEAD

# The list is read into a variable first. Piped straight into `grep -q`, a
# `git diff` that failed would read as a contribution that changes nothing, and
# one longer than a pipe holds dies of SIGPIPE once `grep` stops reading.
#
# A path is printed as it is written, because git quotes one that holds a byte
# outside ASCII, and the quote would hide the directory it starts with.
if ! changed=$(git -c core.quotePath=false diff --name-only "${base}" FETCH_HEAD); then
  echo "what the contribution changes could not be read, so nothing was pushed." >&2
  exit 2
fi
if grep -q '^\.github/' <<<"${changed}"; then
  echo "warning: this contribution changes .github/, so its workflows run with this repository's \
token once it is a branch here, before anyone approves it." >&2
fi

if ! git push --quiet --force origin "FETCH_HEAD:refs/heads/accepted/pr-${pr}"; then
  cannot "accepted/pr-${pr} could not be pushed, so no pull request was opened."
fi

if ! url=$(gh pr list --head "accepted/pr-${pr}" --state open --json url \
  --jq '.[0].url // empty'); then
  cannot "accepted/pr-${pr} is pushed, and its open pull requests could not be read."
fi
if [[ -z "${url}" ]]; then
  if ! title=$(gh pr view "${pr}" --json title --jq .title); then
    cannot "accepted/pr-${pr} is pushed, and the title of #${pr} could not be read."
  fi
  if ! url=$(gh pr create --base main --head "accepted/pr-${pr}" \
    --title "${title}" --body \
    "Carries the work proposed in #${pr}, unchanged.

purba does not sign a branch that lives on a fork, so this branch is what merges."); then
    cannot "accepted/pr-${pr} is pushed, and its pull request could not be opened."
  fi

  if ! gh pr comment "${pr}" --body \
    "Your work is on \`accepted/pr-${pr}\` in this repository, and ${url} is what merges.

A workflow here cannot write to your fork, so purba carries a contribution in rather than merging \
it from one. Your commits are unchanged: the author field and your \`Signed-off-by:\` trailer stay \
yours. CONTRIBUTING.md states what purba records on its side."; then
    cannot "${url} is open, and #${pr} could not be told where its work went."
  fi
fi

cat <<EOF

applied #${pr} to accepted/pr-${pr}
pull request  ${url}

After ${url} merges, close the contribution:

  gh pr close ${pr} --comment "Merged as ${url}."
EOF
