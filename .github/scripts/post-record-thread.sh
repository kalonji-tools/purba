#!/usr/bin/env bash
# The comment a pull request gets when it changes a decision record.
#
#   the decision:  docs/decisions/a-record-is-rewritten-not-amended.md
#                  docs/decisions/a-standard-a-gate-cannot-decide-does-not-become-a-gate.md
#                  docs/decisions/only-github-runs-what-lives-under-github.md
#
# Posts the comment, edits it to the records the diff changes now, deletes it
# when no record is left, and writes nothing when its body would not change.
# Exits 2 when the files or the comments of the pull request cannot be read.
set -euo pipefail

# shellcheck source=scripts/report.sh
. "$(dirname "$0")/../../scripts/report.sh"
# shellcheck source=scripts/records.sh
. "$(dirname "$0")/../../scripts/records.sh"

# The workflow supplies these, so naming them refuses early rather than at the
# line that first reads one.
: "${GH_REPO:?set by the workflow env}"
: "${PR:?set by the workflow env}"

if ! answer=$(gh api "repos/${GH_REPO}/pulls/${PR}/files" --paginate) ||
  ! files=$(jq -r '.[].filename' <<<"${answer}"); then
  cannot "the files of pull request ${PR} could not be read, so no record was looked for."
fi
decisions=$(LC_ALL=C sort <<<"${files}" | while IFS= read -r path; do
  is_record "${path}"
done)

# An issue comment, never a review thread. The ruleset requires every review
# thread to be resolved, so a thread here would block the merge on a click.
# What this asks for is a judgement, and a click cannot show one was made.
marker='<!-- purba:thread:decisions -->'
# `GITHUB_TOKEN` writes as `github-actions[bot]`, and only its comment counts.
if ! answer=$(gh api "repos/${GH_REPO}/issues/${PR}/comments" --paginate) ||
  ! own=$(jq -cs --arg marker "${marker}" '[.[][] | select(.user.login == "github-actions[bot]"
      and (.body | contains($marker)))] | first // {}' <<<"${answer}"); then
  cannot "the comments of pull request ${PR} could not be read, so no comment was written."
fi
own_id=$(jq -r '.id // empty' <<<"${own}")
own_body=$(jq -r '.body // empty' <<<"${own}")

if [[ -z "${decisions}" ]]; then
  if [[ -n "${own_id}" ]]; then
    gh api --method DELETE "repos/${GH_REPO}/issues/comments/${own_id}"
    echo "the diff no longer changes a decision record, so the comment was deleted"
  else
    echo "the diff changes no decision record, so no comment is owed"
  fi
  exit 0
fi

# The sed program is single-quoted so the shell hands its `$` and its backticks
# to sed, which is what turns each path into a Markdown list item.
# shellcheck disable=SC2016
record_list=$(sed 's|^|- \`|; s|$|\`|' <<<"${decisions}")

# The URL alone is a hundred characters, so this line cannot be wrapped: inside
# the body below a backslash and a newline would be posted as text, and a
# directive would be posted with them.
# editorconfig-checker-disable-next-line
record_link="[A record is rewritten, not amended](https://github.com/kalonji-tools/purba/blob/main/docs/decisions/a-record-is-rewritten-not-amended.md)"

body="**This pull request changes a decision record.**

${record_list}

A decision record states what this project decided and why. It is read by \
people who were not in the conversation.

Two things about it can never be checked by a tool. You are the only reader \
who can judge them:

- does this record state **one** decision?
- is it **still true**?

Nothing blocks on this. ${record_link} is where both come from.
${marker}"

if [[ -z "${own_id}" ]]; then
  gh api "repos/${GH_REPO}/issues/${PR}/comments" -f body="${body}"
  echo "posted the decision-record comment"
elif [[ "${own_body}" == "${body}" ]]; then
  echo "the comment already names these records, so nothing was written"
else
  gh api --method PATCH "repos/${GH_REPO}/issues/comments/${own_id}" -f body="${body}"
  echo "edited the decision-record comment to name the records the diff changes now"
fi
