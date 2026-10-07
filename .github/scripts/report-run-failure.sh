#!/usr/bin/env bash
# Say that a run broke, where somebody reads it.
#
#   the callers:   .github/workflows/bump.yml
#                  .github/workflows/release.yml
#                  .github/workflows/publish.yml
#
#   report-run-failure.sh <run> <run-url> <issue>
#
#   <run>      bump, release or publish, the workflow that failed
#   GH_REPO    the repository `gh` acts on
#   GH_TOKEN   a token that may open and comment on an issue
#
# Exits 0 when the failure is recorded, and 2 when it cannot run.
set -euo pipefail

if [[ $# -ne 3 ]]; then
  echo "usage: report-run-failure.sh <run> <run-url> <issue>" >&2
  exit 2
fi

if [[ -z "${GH_REPO:-}" ]]; then
  echo "GH_REPO is set by the workflow env, and it is empty here." >&2
  exit 2
fi

workflow=$1
run=$2
issue=$3

# The title is the deduplication key, so it carries no date and no number.
case "${workflow}" in
  bump)
    title="The nightly bump failed"
    body=$(
      cat <<BODY
A scheduled run of \`.github/workflows/bump.yml\` failed before it proposed anything.

| | |
|---|---|
| the run | ${run} |
| what it does | proposes a newer nightly for a person to sign |
| what its failing means | ⚠️ **the pinned toolchain stops moving, and nothing else says so** |

**This is not a red pull request.** A scheduled run has no pull request, so no gate refuses it and \
nothing else reports it. That is why this issue exists.

Two other things look identical from outside and are not this:

- the week had no newer nightly, which exits 0 and opens nothing
- a proposal is already open, which stands down on purpose

Later failures are added to this issue as comments rather than opening another. Close it once the \
bump runs clean.

[#${issue}](https://github.com/${GH_REPO}/issues/${issue}) owns the workflow.
BODY
    )
    ;;
  release)
    title="The release run failed"
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
    ;;
  publish)
    title="The publish run failed"
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
    ;;
  *)
    echo "<run> is bump, release or publish, and it is ${workflow} here." >&2
    exit 2
    ;;
esac

# ⚠️ Never use `--search` here.
existing=$(gh issue list --repo "${GH_REPO}" --state open --limit 200 \
  --json number,title --jq "[.[] | select(.title == \"${title}\")] | first | .number // empty")

if [[ -n "${existing}" ]]; then
  gh issue comment "${existing}" --repo "${GH_REPO}" --body "It failed again: ${run}"
  echo "commented on #${existing} rather than opening a second issue"
  exit 0
fi

number=$(gh issue create --repo "${GH_REPO}" --title "${title}" \
  --label bug --label wayfinder:task --body "${body}")
echo "opened ${number}"
