#!/usr/bin/env bash
# The review thread a pull request owes when it changes a decision record.
#
#   the decision:  docs/decisions/a-record-is-rewritten-not-amended.md
#
# Not runnable outside a workflow, which is why it stays in `.github/scripts/`.
#
# Exits 0 when no thread is owed and when one is already present.
set -euo pipefail

# The workflow supplies these, so naming them refuses early rather than at the
# line that first reads one.
: "${GH_REPO:?set by the workflow env}"
: "${PR:?set by the workflow env}"
: "${HEAD_SHA:?set by the workflow env}"

decisions=$(gh api "repos/${GH_REPO}/pulls/${PR}/files" --paginate --jq '.[].filename' |
  LC_ALL=C sort | grep '^docs/decisions/' || true)
if [[ -z "${decisions}" ]]; then
  echo "the diff does not touch docs/decisions, so no thread is owed"
  exit 0
fi

# Anchoring is narrower than the documentation suggests, and all of
# this was measured against a live pull request rather than read:
# an inline comment is accepted only on a line inside a diff hunk;
# `subject_type` is REJECTED inside a review payload; and GraphQL's
# addPullRequestReviewThread with subjectType FILE returns a null
# thread and creates nothing. A file-level thread exists only through
# the standalone endpoint used below.
marker='<!-- purba:thread:decisions -->'
# `ready_for_review` fires again every time a draft is re-readied, so
# without this check one question would become several required
# resolutions.
existing=$(gh api "repos/${GH_REPO}/pulls/${PR}/comments" --paginate --jq '.[].body')
case "${existing}" in
  *"${marker}"*)
    echo "${marker} is already present, skipping"
    exit 0
    ;;
  *) ;;
esac

first_record=$(head -1 <<<"${decisions}")
# The sed program is single-quoted so the shell hands its `$` and its backticks
# to sed, which is what turns each path into a Markdown list item.
# shellcheck disable=SC2016
record_list=$(sed 's|^|- \`|; s|$|\`|' <<<"${decisions}")

# The URL alone is a hundred characters, so this line cannot be wrapped: inside
# the body below a backslash and a newline would be posted as text, and a
# directive would be posted with them.
# editorconfig-checker-disable-next-line
record_link="[A record is rewritten, not amended](https://github.com/kalonji-tools/purba/blob/main/docs/decisions/a-record-is-rewritten-not-amended.md)"

gh api "repos/${GH_REPO}/pulls/${PR}/comments" \
  -f commit_id="${HEAD_SHA}" -f path="${first_record}" -f subject_type=file \
  -f body="**This pull request changes a decision record.**

${record_list}

A decision record states what this project decided and why, and it is read by people \
who were not in the conversation. ${record_link} names two things about it that no tool \
can check, and you are the only reader who can judge them.

A third has already gone wrong once. **The Decision Outcome is written by a person, or \
the change does not merge.** An agent writing it satisfies the letter and voids the rule.

Resolve this thread when you have judged all three.
${marker}"
echo "posted the decision-record thread"
