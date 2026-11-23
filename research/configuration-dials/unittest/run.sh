#!/usr/bin/env bash
# Probe which unittest dials change a test's outcome.
# usage: run.sh <python 3.13> <empty work dir>
set -u
py=$1
work=$2
here=$(cd "$(dirname "$0")" && pwd)
cp "$here/test_dials.py" "$work/"
cd "$work" || exit 99

run() {
  echo "---- $*"
  "$py" "$@" 2>&1 | grep -E '^(test_|FAIL:|ERROR:|Ran |OK|FAILED|DeprecationWarning|.*unexpected success)' | head -8
  echo "exit: ${PIPESTATUS[0]}"
}

# -I leaves the work dir off sys.path, so discovery puts it there with -t.
d=(discover -s . -t . -p test_dials.py)
run -I -m unittest "${d[@]}" -v
run -I -W error::DeprecationWarning -m unittest "${d[@]}" -v
run -I -m unittest "${d[@]}" -v -f
run -I -m unittest "${d[@]}" -v -k warns
# unittest.main(warnings="error") in code: the module run as a script.
run -I test_dials.py -v
