#!/usr/bin/env bash
# PROTOTYPE for issue 488. Never merge.
#
# The detector all three rules share: Build runs when the pull request changes
# a path whose state is `set`. Deleted and renamed paths count by both names.
#
#   detect.sh <build-reads file> <base> <head>      prints run or skip
set -euo pipefail
here=$(dirname "$(realpath "$0")")
files=$(git -c core.quotePath=false diff --name-only --no-renames "$2" "$3")
if "${here}/attr.sh" "$1" <<<"${files}" | grep -q '^set '; then
  echo run
else
  echo skip
fi
