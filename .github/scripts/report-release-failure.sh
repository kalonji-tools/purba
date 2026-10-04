#!/usr/bin/env bash
# Say that a release run broke, where somebody reads it.
#
#   the caller:    .github/workflows/release.yml
#   the work:      .github/scripts/release.sh
#
#   report-release-failure.sh <run-url> <issue>
#
#   GH_REPO    the repository `gh` acts on
#   GH_TOKEN   a token that may open and comment on an issue
#
# Exits 0 when the failure is recorded, and 2 when it cannot run.
#
# ⚠️ Nothing else reports a scheduled run that fails.
set -euo pipefail

if [[ $# -ne 2 ]]; then
  echo "usage: report-release-failure.sh <run-url> <issue>" >&2
  exit 2
fi

if [[ -z "${GH_REPO:-}" ]]; then
  echo "GH_REPO is set by the workflow env, and it is empty here." >&2
  exit 2
fi

run=$1
issue=$2

# The title is the deduplication key, so it carries no date and no number.
title="The release run failed"

# ⚠️ Never use `--search` here.
existing=$(gh issue list --repo "${GH_REPO}" --state open --limit 200 \
  --json number,title --jq "[.[] | select(.title == \"${title}\")] | first | .number // empty")

if [[ -n "${existing}" ]]; then
  gh issue comment "${existing}" --repo "${GH_REPO}" --body "It failed again: ${run}"
  echo "commented on #${existing} rather than opening a second issue"
  exit 0
fi

body=$(
  cat <<BODY
A run of \`.github/workflows/release.yml\` failed.

| | |
|---|---|
| the run | ${run} |
| what it does | tags a merged release, or proposes the next one |
| what its failing means | ⚠️ **a merged release stays untagged, or the next one is never \
proposed, and nothing else says so** |

**This is not a red pull request.** A scheduled run has no pull request, so no gate refuses it and \
nothing else reports it. That is why this issue exists.

Two other things look identical from outside and are not this:

- nothing since the tag reaches the changelog, which exits 0 and opens nothing
- a release pull request is already open, which stands down on purpose

Later failures are added to this issue as comments rather than opening another. Close it once a \
release run passes.

[#${issue}](https://github.com/${GH_REPO}/issues/${issue}) owns the workflow.
BODY
)

number=$(gh issue create --repo "${GH_REPO}" --title "${title}" \
  --label bug --label wayfinder:task --body "${body}")
echo "opened ${number}"
