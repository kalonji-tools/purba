#!/usr/bin/env bash
# Whether a pull request changes a path that starts a gate. .config/gates marks
# each tracked path for each gate that skips, and a path it leaves unmarked
# starts the gate.
#
#   the caller:  .github/workflows/build.yml
#   the paths:   .config/gates
#
#   changed.sh <attr> <base> <head>
#
# Prints `<attr>=true` unless .config/gates at <head> marks every changed path
# `-<attr>`, in the form a workflow appends to its step outputs. What it decided
# goes to standard error.
#
# Exits 0 when it decided, either way, and 2 when it cannot run.
set -euo pipefail

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

if ! git show "${head}:.config/gates" >"${scratch}/gates" 2>/dev/null; then
  echo ".config/gates could not be read at ${head}." >&2
  exit 2
fi

# The repository's own attributes would answer for an attribute they happen to
# name, so the marks are resolved in an empty repository. A template can carry
# an info/attributes file, so the empty repository takes none.
git init --quiet --template= "${scratch}/empty"
if ! GIT_ATTR_NOSYSTEM=1 git -C "${scratch}/empty" -c core.attributesFile="${scratch}/gates" \
  check-attr -z --stdin "${attr}" <"${scratch}/files" >"${scratch}/marks"; then
  echo "git cannot resolve .config/gates at ${head}." >&2
  exit 2
fi

while IFS= read -r -d '' path && IFS= read -r -d '' _ && IFS= read -r -d '' state; do
  if [[ "${state}" != unset ]]; then
    echo "${attr}=true"
    echo "${path} is not marked -${attr}, so the gate runs" >&2
    exit 0
  fi
done <"${scratch}/marks"

echo ".config/gates marks every changed path -${attr}, so the gate does not run" >&2
