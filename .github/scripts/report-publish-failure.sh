#!/usr/bin/env bash
# Say that a publish run broke, where somebody reads it.
#
#   the caller:    .github/workflows/publish.yml
#
#   report-publish-failure.sh <run-url> <issue>
#
#   GH_REPO    the repository `gh` acts on
#   GH_TOKEN   a token that may open and comment on an issue
#
# Exits 0 when the failure is recorded, and 2 when it cannot run.
#
# ⚠️ Nothing else reports a publish run that a bot dispatches.
set -euo pipefail

if [[ $# -ne 2 ]]; then
  echo "usage: report-publish-failure.sh <run-url> <issue>" >&2
  exit 2
fi

if [[ -z "${GH_REPO:-}" ]]; then
  echo "GH_REPO is set by the workflow env, and it is empty here." >&2
  exit 2
fi

run=$1
issue=$2

# The title is the deduplication key, so it carries no date and no number.
title="The publish run failed"

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
A run of \`.github/workflows/publish.yml\` on a release tag failed.

| | |
|---|---|
| the run | ${run} |
| what it does | builds the wheels of a release tag and uploads them to PyPI |
| what its failing means | ⚠️ **the tag exists, and PyPI holds none of its wheels, or \
only some** |

**This is not a red pull request.** No pull request runs this workflow, and GitHub notifies \
nobody of a run that a bot started. That is why this issue exists.

Re-run the failed jobs of the run, or start a new run on the tag:

\`\`\`sh
gh workflow run publish.yml --ref v<version>
\`\`\`

⚠️ PyPI refuses a file it already has. If the run uploaded some of the wheels, a second upload \
fails on those.

Later failures are added to this issue as comments rather than opening another. Close it once a \
publish run passes.

[#${issue}](https://github.com/${GH_REPO}/issues/${issue}) owns the workflow.
BODY
)

number=$(gh issue create --repo "${GH_REPO}" --title "${title}" \
  --label bug --label wayfinder:task --body "${body}")
echo "opened ${number}"
