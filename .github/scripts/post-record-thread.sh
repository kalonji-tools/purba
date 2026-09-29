#!/usr/bin/env bash
# The comment a pull request gets when it changes a decision record.
#
#   the decision:  docs/decisions/a-record-is-rewritten-not-amended.md
#                  docs/decisions/a-gate-owns-the-mechanical.md
#                  docs/decisions/only-github-runs-what-lives-under-github.md
#
# Exits 0 when no comment is owed and when one is already present.
set -euo pipefail

# The workflow supplies these, so naming them refuses early rather than at the
# line that first reads one.
: "${GH_REPO:?set by the workflow env}"
: "${PR:?set by the workflow env}"

decisions=$(gh api "repos/${GH_REPO}/pulls/${PR}/files" --paginate --jq '.[].filename' |
  LC_ALL=C sort | grep '^docs/decisions/' || true)
if [[ -z "${decisions}" ]]; then
  echo "the diff does not touch docs/decisions, so no comment is owed"
  exit 0
fi

# An issue comment, never a review thread. The ruleset requires every review
# thread to be resolved, so a thread here would block the merge on a click.
# What this asks for is a judgement, and a click cannot show one was made.
marker='<!-- purba:thread:decisions -->'
# `ready_for_review` fires again every time a draft is re-readied, so without
# this check one comment would become several.
existing=$(gh api "repos/${GH_REPO}/issues/${PR}/comments" --paginate --jq '.[].body')
case "${existing}" in
  *"${marker}"*)
    echo "${marker} is already present, skipping"
    exit 0
    ;;
  *) ;;
esac

# The sed program is single-quoted so the shell hands its `$` and its backticks
# to sed, which is what turns each path into a Markdown list item.
# shellcheck disable=SC2016
record_list=$(sed 's|^|- \`|; s|$|\`|' <<<"${decisions}")

# The URL alone is a hundred characters, so this line cannot be wrapped: inside
# the body below a backslash and a newline would be posted as text, and a
# directive would be posted with them.
# editorconfig-checker-disable-next-line
record_link="[A record is rewritten, not amended](https://github.com/kalonji-tools/purba/blob/main/docs/decisions/a-record-is-rewritten-not-amended.md)"

gh api "repos/${GH_REPO}/issues/${PR}/comments" \
  -f body="**This pull request changes a decision record.**

${record_list}

A decision record states what this project decided and why. It is read by \
people who were not in the conversation.

Two things about it can never be checked by a tool. You are the only reader \
who can judge them:

- does this record state **one** decision?
- is it **still true**?

Nothing blocks on this. ${record_link} is where both come from.
${marker}"
echo "posted the decision-record comment"
