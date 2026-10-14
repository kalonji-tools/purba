#!/usr/bin/env bash
# Whether a pull request changes a path that starts a gate. .config/gates marks
# each tracked path for each gate that skips, and a path it leaves unmarked
# starts the gate.
#
#   the callers:  .github/workflows/build.yml
#                 .github/workflows/scripts.yml
#   the paths:    .config/gates
#
#   changed.sh <attr> <base> <head>
#
# Prints `<attr>=true` unless .config/gates at <base> marks every changed path
# `-<attr>`, in the form a workflow appends to its step outputs. What it decided
# goes to standard error.
#
# Exits 0 when it decided, either way, and 2 when it cannot run.
set -euo pipefail

# shellcheck source=scripts/report.sh
. "$(dirname "$0")/../../scripts/report.sh"
# shellcheck source=scripts/attributes.sh
. "$(dirname "$0")/../../scripts/attributes.sh"

if [[ $# -ne 3 ]]; then
  echo "usage: changed.sh <attr> <base> <head>" >&2
  exit 2
fi
attr=$1 base=$2 head=$3

# git writes NUL-separated output, which a shell variable cannot hold, so each
# step writes a file here.
scratch=$(mktemp -d)
trap 'rm -rf "${scratch}"' EXIT

# Written to a file first. Piped straight on, a `git diff` that failed would
# read as a pull request that changes nothing.
#
# A rename is read as a deletion and an addition. git names a renamed file by
# its new path alone, so a file moved out of a marked directory would pass
# unseen.
if ! git diff -z --name-only --no-renames "${base}" "${head}" >"${scratch}/files"; then
  echo "what ${head} changes against ${base} could not be read." >&2
  exit 2
fi

# The marks are read at the base, which the pull request cannot edit. Read at
# the head, a pull request could unmark the paths it changes, and the file
# itself.
if ! git show "${base}:.config/gates" >"${scratch}/gates" 2>/dev/null; then
  echo ".config/gates could not be read at ${base}." >&2
  exit 2
fi

attr_states "${scratch}" "${scratch}/gates" "${attr}" <"${scratch}/files" >"${scratch}/marks"

while IFS= read -r -d '' path && IFS= read -r -d '' _ && IFS= read -r -d '' state; do
  if [[ "${state}" != unset ]]; then
    echo "${attr}=true"
    echo "${path} is not marked -${attr}, so the gate runs" >&2
    exit 0
  fi
done <"${scratch}/marks"

echo ".config/gates marks every changed path -${attr}, so the gate does not run" >&2
