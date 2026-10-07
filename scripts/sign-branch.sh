#!/usr/bin/env bash
# The sign-off a person owes for a branch.
#
#   what you owe:  CONTRIBUTING.md
#   the decisions: docs/decisions/liability-is-recorded-from-the-act-that-makes-it-true.md
#                  docs/decisions/purba-records-delegation-and-does-not-prevent-it.md
#
#   sign-branch.sh [<base>]
#
# Exits 1 when nothing was signed. Exits 2 when this script cannot run.
set -euo pipefail

# Both tests are load-bearing. Without this one, a piped answer satisfies the
# question below.
if [[ ! -t 0 ]]; then
  echo "sign-branch.sh ran with no terminal attached, so nothing was signed." >&2
  exit 1
fi

base=${1:-origin/main}

if ! start=$(git merge-base "${base}" HEAD 2>&1); then
  echo "sign-branch.sh cannot find where your branch leaves ${base}." >&2
  echo "${start}" >&2
  exit 2
fi

"$(dirname "$0")/check-replayable.sh" "${base}" HEAD >/dev/null

echo "Measured from ${base}:"
git --no-pager log --reverse --format='  %h  %an  %s' "${start}..HEAD"

read -r -p "Sign these commits? [y/N] " reply || reply=""
if [[ "${reply}" != "y" ]]; then
  echo "nothing was signed." >&2
  exit 1
fi

# The value must hold no newline. git refuses an exec command that contains one,
# and a single-quoted string cannot be wrapped without keeping the backslash.
# editorconfig-checker-disable-next-line
sign_exec='git log -1 --format="%(trailers:key=Signed-off-by)" | grep -q . || git commit --amend --no-edit -s'
git rebase "${start}" --exec "${sign_exec}"
