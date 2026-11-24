#!/usr/bin/env bash
# Probe the ways Jest selects tests. Each case prints the tests that ran, and
# the exit code. Jest has no tag, so "slow" and "db" are parts of names.
# Usage: nix-shell -p nodejs --run "bash run.sh <dir with node_modules/jest> <empty dir>"
set -uo pipefail
mods=$1
work=$2
here=$(cd "$(dirname "$0")" && pwd)
cp "$here"/math.test.js "$here"/store.test.js "$work"/
ln -s "$mods/node_modules" "$work/node_modules"
printf '{"name": "sel", "jest": {"testEnvironment": "node"}}\n' >"$work/package.json"
cd "$work" || exit 1
jt=$work/node_modules/.bin/jest
"$jt" --version
probe() {
  local name=$1
  shift
  local out
  out=$("$jt" --ci --no-cache --json --outputFile=report.json "$@" 2>&1)
  local code=$?
  [ -f report.json ] && out="$out
$(node "$here/passed.js" report.json)" && rm report.json
  printf '### %s: jest' "$name"
  printf ' %q' "$@"
  printf '\nexit %s\n' "$code"
  printf '%s\n' "$out" | grep -E '^ran: |No tests found|Error|Pattern' | head -8
  echo
}
probe all
probe path-pattern math
probe path-ignore --testPathIgnorePatterns store
probe t-substring -t add
probe t-regex-union -t 'slow|db'
probe t-regex-intersection -t '(?=.*slow)(?=.*db)'
probe t-regex-exclusion -t '^(?!.*db).*slow'
probe t-repeated -t slow -t db
probe t-full-name -t '^store migrate db slow$'
probe t-empty -t nosuchname
probe path-empty nosuchfile
probe t-bad-regex -t '('
