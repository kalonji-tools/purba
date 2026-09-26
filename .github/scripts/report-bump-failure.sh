#!/usr/bin/env bash
# Say that the nightly bump broke, where somebody reads it.
#
#   the caller:    .github/workflows/bump.yml
#   the work:      .github/scripts/bump-nightly.sh
#
#   report-bump-failure.sh <run-url> <issue>
#
#   GH_REPO    the repository `gh` acts on
#   GH_TOKEN   a token that may open and comment on an issue
#
# Exits 0 when the failure is recorded, and 2 when it cannot run.
#
# ⚠️ Nothing else reports a scheduled run that fails.
set -euo pipefail

if [ $# -ne 2 ]; then
  echo "usage: report-bump-failure.sh <run-url> <issue>" >&2
  exit 2
fi

run=$1
issue=$2

# The title is the deduplication key, so it carries no date and no number.
title="The nightly bump failed"

# ⚠️ Never use `--search` here.
existing=$(gh issue list --repo "$GH_REPO" --state open --limit 200 \
  --json number,title --jq "[.[] | select(.title == \"$title\")] | first | .number // empty")

if [ -n "$existing" ]; then
  gh issue comment "$existing" --repo "$GH_REPO" --body "It failed again: $run"
  echo "commented on #$existing rather than opening a second issue"
  exit 0
fi

body=$(cat <<BODY
A scheduled run of \`.github/workflows/bump.yml\` failed before it proposed anything.

| | |
|---|---|
| the run | $run |
| what it does | proposes a newer nightly for a person to sign |
| what its failing means | ⚠️ **the pinned toolchain stops moving, and nothing else says so** |

**This is not a red pull request.** A scheduled run has no pull request, so no gate refuses it and nothing else reports it. That is why this issue exists.

Two other things look identical from outside and are not this:

- the week had no newer nightly, which exits 0 and opens nothing
- a proposal is already open, which stands down on purpose

Later failures are added to this issue as comments rather than opening another. Close it once the bump runs clean.

[#$issue](https://github.com/$GH_REPO/issues/$issue) owns the workflow.
BODY
)

number=$(gh issue create --repo "$GH_REPO" --title "$title" --label bug --body "$body")
echo "opened $number"
