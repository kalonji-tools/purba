#!/usr/bin/env bash
# Whether a pull request changes one of purba's own scripts, or a setting a
# script test copies: lychee.toml for the link gate, cliff.toml for the release.
#
#   the caller:    .github/workflows/scripts.yml
#   the decision:  docs/decisions/purba-writes-its-scripts-in-bash-until-purba-can-test-them.md
#
#   changed-scripts.sh <base> <head>
#
# Prints `scripts=true` when one changed and nothing when none did, in the form
# a workflow appends to its step outputs. What it decided goes to standard error.
#
# Exits 0 when it decided, either way, and 2 when it cannot run.
set -euo pipefail

if [[ $# -ne 2 ]]; then
  echo "usage: changed-scripts.sh <base> <head>" >&2
  exit 2
fi

# The list is read into a variable first. Piped straight into `grep`, a
# `git diff` that failed would read as a pull request that changes nothing.
#
# A rename is read as a deletion and an addition. git names a renamed file by
# its new path alone, so a script moved out of either directory would pass
# unseen. A path is printed as it is written, because git quotes one that holds
# a byte outside ASCII, and the quote would hide the directory it starts with.
if ! files=$(git -c core.quotePath=false diff --name-only --no-renames "$1" "$2"); then
  echo "what $2 changes against $1 could not be read." >&2
  exit 2
fi

if grep -qE '^(scripts/|\.github/scripts/|lychee\.toml$|cliff\.toml$)' <<<"${files}"; then
  echo "scripts=true"
  echo "something a script test reads changed, so the script tests run" >&2
else
  echo "nothing a script test reads changed, so the script tests do not run" >&2
fi
