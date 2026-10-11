#!/usr/bin/env bash
# PROTOTYPE for issue 488. Never merge.
#
#   attr.sh <build-reads file>  < paths, one per line
#
# Prints `<state> <path>` for each path: set, unset or unspecified.
# Resolved in an empty repository, as scripts/check-readers.sh does, so the
# repository's own .gitattributes does not leak in.
set -euo pipefail
list=$(realpath "$1")
empty=$(mktemp -d)
trap 'rm -rf "${empty}"' EXIT
git init --quiet --template= "${empty}"
GIT_ATTR_NOSYSTEM=1 git -C "${empty}" -c core.attributesFile="${list}" \
  check-attr --stdin build | sed -E 's/^(.*): build: (.*)$/\2 \1/'
