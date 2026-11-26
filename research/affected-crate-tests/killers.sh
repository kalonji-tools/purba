#!/usr/bin/env bash
# Print each test target that failed in the log of one
# `cargo test --workspace --no-fail-fast`, one per line, as cargo names it:
#   -p purba-reporting --lib
#   -p purba-selection --test select
#   -p purba-errors --doc
# cargo prints this list after "error: N target(s) failed:" when the run goes on
# past a failure. A build that fails prints no list, so the script prints
# "build-failed" when the log holds "could not compile".
# usage: killers.sh <log file>
set -u
log=$1
if grep -q 'could not compile' "$log"; then
  echo build-failed
  exit 0
fi
sed -n '/^error: [0-9]* targets\{0,1\} failed:/,/^[^ ]/p' "$log" |
  sed -n 's/^ *`\(-p [^`]*\)`.*$/\1/p'
