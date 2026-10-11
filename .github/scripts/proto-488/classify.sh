#!/usr/bin/env bash
# PROTOTYPE for issue 488. Never merge.
#
# Rule 1, the classifier lint: refuse a tracked path that no line of the
# build-reads file reaches. It reads the tree only, so it can run in Quality,
# which runs on every pull request.
#
#   classify.sh <build-reads file>      run at the top of a checkout
#
# Prints `pass` or `refuse: <paths>`.
set -euo pipefail
here=$(dirname "$(realpath "$0")")
unplaced=$(git ls-files | "${here}/attr.sh" "$1" | sed -n 's/^unspecified //p')
if [[ -z "${unplaced}" ]]; then
  echo pass
else
  echo "refuse: $(tr '\n' ' ' <<<"${unplaced}")"
fi
