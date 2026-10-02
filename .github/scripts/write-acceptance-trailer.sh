#!/usr/bin/env bash
# The acceptance a code owner's approval owes every commit on the branch.
#
#   the decision:  docs/decisions/liability-is-recorded-from-the-act-that-makes-it-true.md
#                  docs/decisions/only-github-runs-what-lives-under-github.md
#   what you owe:  CONTRIBUTING.md
#
# Exits 1 when it refuses, 0 when there is nothing to accept yet, and 2 when it
# cannot write a check run.
set -euo pipefail

# The workflow supplies these, so naming them refuses early rather than at the
# line that first reads one. `GATES` holds the contexts this job waits on.
: "${GH_REPO:?set by the workflow env}"
: "${PR:?set by the workflow env}"
: "${GATES:?set by the workflow env}"
: "${HEAD_REF:?set by the workflow env}"
: "${RUNNER_TEMP:?set by the runner}"
: "${GITHUB_OUTPUT:?set by the runner}"

# shellcheck source=.github/scripts/check-run.sh
. "$(dirname "$0")/check-run.sh"

# Read the approval from the API, not from the event. A push event
# carries no approval, and a push is what this job has to survive.
reviews=$(gh api "repos/${GH_REPO}/pulls/${PR}/reviews" --paginate)
approver=$(jq -r '[.[] | select(.state == "APPROVED")] | last | .user.login // empty' \
  <<<"${reviews}")
approver_id=$(jq -r '[.[] | select(.state == "APPROVED")] | last | .user.id // empty' \
  <<<"${reviews}")

# With no approval, write no check at all. A missing check already
# blocks the merge, so this job stops here and succeeds.
if [[ -z "${approver}" ]]; then
  echo "no approval stands on this head, so there is nothing to accept"
  exit 0
fi

current_head=$(git rev-parse HEAD)
# `GATES` holds one context per word and this script takes one per argument, so
# quoting it would ask for a single context named after all of them.
# shellcheck disable=SC2086
if ! .github/scripts/require-green.sh "${current_head}" ${GATES}; then
  echo "a gate has no verdict on this head, so there is nothing to accept yet"
  exit 0
fi

# These two are written into commit messages that can never be
# edited, so check their shape before writing them. GitHub allows only
# letters, digits and hyphens in a login today.
case "${approver}" in
  *[!A-Za-z0-9-]*)
    echo "::error::the approver login has a character this job will not \
write into a commit: ${approver}"
    exit 1
    ;;
  *) ;;
esac
case "${approver_id}" in
  "" | *[!0-9]*)
    echo "::error::the approver id is not a number: ${approver_id}"
    exit 1
    ;;
  *) ;;
esac

# The numeric id survives a username change. A username-only address
# does not.
export TRAILER="Accepted-by: ${approver} <${approver_id}+${approver}@users.noreply.github.com>"

# The branch this pull request targets, not `main`. Measuring from
# `main` would make the job rewrite commits it does not own. Read it
# from the API: a gate finishing carries no base branch.
base_ref=$(gh api "repos/${GH_REPO}/pulls/${PR}" --jq .base.ref)
git fetch --quiet origin "${base_ref}"
base=$(git merge-base FETCH_HEAD HEAD)
before=$(git rev-parse HEAD)

# Stop before rewriting anything. CONTRIBUTING.md says why this job
# cannot repair a missing trailer.
export PURBA_REPORT="${RUNNER_TEMP}/refusal"

refuse() {
  case "$1" in
    1) title="Refused before anything was rewritten" ;;
    *) title="This check could not run" ;;
  esac

  summary=$(cat "${PURBA_REPORT}" 2>/dev/null || echo "This job stopped without writing \
anything a reader can use. The run log holds what it did.")

  post_check_run "Sign-off" "${before}" failure "${title}" "${summary}"

  exit 1
}

scripts/check-origin.sh "${base}" HEAD || refuse $?

scripts/check-replayable.sh "${base}" HEAD || refuse $?

# A fresh checkout has no git identity, and `git commit` refuses to
# run without one. These two lines only satisfy that. The name does
# not survive: the merge replaces the committer.
git config user.name "github-actions[bot]"
git config user.email "41898282+github-actions[bot]@users.noreply.github.com"

# The acceptance command is one file, copied out before the replay starts.
# `scripts/accept-one-commit.sh` says why the copy is load-bearing, and why
# leaving a commit alone when it already names the approver is what stops this
# repeating forever: the commits stop changing, so the push below stops.
#
# `$TRAILER` reaches the command through the environment, where it
# stays data. Expanding it into a command string instead would let the
# shell paste it in as code, and that was measured running a command
# hidden inside a login.
accept_exec="${RUNNER_TEMP}/accept-one-commit.sh"
cp scripts/accept-one-commit.sh "${accept_exec}"
git rebase "${base}" --exec "${accept_exec}"

head_sha=$(git rev-parse HEAD)
if [[ "${head_sha}" != "${before}" ]]; then
  git push --force origin "HEAD:${HEAD_REF}"
fi

accepted_summary="Every commit on this branch carries an \`Accepted-by:\` trailer \
naming the account that approved it."
post_check_run "Sign-off" "${head_sha}" success "Accepted by @${approver}" \
  "${accepted_summary}"

echo "recorded acceptance by ${approver} on ${head_sha}"

{
  echo "before=${before}"
  echo "after=${head_sha}"
} >>"${GITHUB_OUTPUT}"
