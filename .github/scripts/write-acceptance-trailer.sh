#!/usr/bin/env bash
# The acceptance a code owner's approval owes every commit on the branch.
#
#   the decision:  docs/decisions/liability-is-recorded-from-the-act-that-makes-it-true.md
#                  docs/decisions/only-github-runs-what-lives-under-github.md
#   what you owe:  CONTRIBUTING.md
#
# Exits 1 when it refuses, 0 when there is nothing to accept yet, and 2 when it
# cannot read or write a check run. Any other command that fails exits with its
# own code, which can be 1.
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
# shellcheck source=scripts/report.sh
. "$(dirname "$0")/../../scripts/report.sh"

# Read the approval from the API, not from the event. A push event
# carries no approval, and a push is what this job has to survive.
reviews=$(gh api "repos/${GH_REPO}/pulls/${PR}/reviews" --paginate)
stands=$(jq -c '
  to_entries | map(.value + {order: .key})
  | map(select(.state == "APPROVED" or .state == "CHANGES_REQUESTED" or .state == "DISMISSED"))
  | group_by(.user.id) | map(max_by(.order)) | map(select(.state == "APPROVED"))
  | max_by(.order) // empty' <<<"${reviews}")
approver=$(jq -r '.user.login // empty' <<<"${stands}")
approver_id=$(jq -r '.user.id // empty' <<<"${stands}")

# With no approval, write no check at all. A missing check already
# blocks the merge, so this job stops here and succeeds.
if [[ -z "${approver}" ]]; then
  echo "no approval stands on this head, so there is nothing to accept"
  exit 0
fi

current_head=$(git rev-parse HEAD)
status=0
# `GATES` holds one context per word and this script takes one per argument, so
# quoting it would ask for a single context named after all of them.
# shellcheck disable=SC2086
.github/scripts/require-green.sh "${current_head}" ${GATES} || status=$?
case "${status}" in
  0) ;;
  1)
    echo "a gate is not green on this head, so there is nothing to accept yet"
    exit 0
    ;;
  *)
    cannot "the gates on this head could not be read, so nothing was decided."
    ;;
esac

case "${approver}" in
  *[!A-Za-z0-9-]*)
    refuse "A login is written into an Accepted-by trailer only when it holds letters, digits \
and hyphens, because the trailer names the person who accepts the commit, and GitHub allows \
no other character in a person's login. Ask a person to approve the pull request." \
      "login: ${approver}"
    finish
    ;;
  *) ;;
esac
case "${approver_id}" in
  "" | *[!0-9]*)
    refuse "An id is written into an Accepted-by trailer only when it is a number, because the \
trailer reaches main, where no commit message is edited, and GitHub gives each account a \
numeric id. Ask a person to approve the pull request." \
      "id: ${approver_id:-none}"
    finish
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

post_refusal() {
  case "$1" in
    1) title="Refused before anything was rewritten" ;;
    *) title="This check could not run" ;;
  esac

  summary=$(cat "${PURBA_REPORT}" 2>/dev/null || echo "This job stopped without writing \
anything a reader can use. The run log holds what it did.")

  post_check_run "Sign-off" "${before}" failure "${title}" "${summary}"

  exit 1
}

scripts/check-origin.sh "${base}" HEAD || post_refusal $?

export GIT_COMMITTER_NAME="github-actions[bot]"
export GIT_COMMITTER_EMAIL="41898282+github-actions[bot]@users.noreply.github.com"

# `$TRAILER` reaches the replay through the environment, where it
# stays data. Expanding it into a command string instead would let the
# shell paste it in as code, and that was measured running a command
# hidden inside a login.
status=0
head_sha=$(scripts/check-replayable.sh "${base}" HEAD) || status=$?
if [[ "${status}" -ne 0 ]]; then
  post_refusal "${status}"
fi

# Never a plain --force: it drops a commit pushed after the checkout.
if [[ "${head_sha}" != "${before}" ]]; then
  git push --force-with-lease="refs/heads/${HEAD_REF}:${before}" origin \
    "${head_sha}:refs/heads/${HEAD_REF}"
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
