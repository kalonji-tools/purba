#!/usr/bin/env bash
# Probe the ways Vitest selects tests. Each case prints the tests that
# `vitest list` selects, and the exit code.
# Usage: nix-shell -p nodejs --run "bash run.sh <dir with node_modules/vitest> <empty dir>"
set -uo pipefail
mods=$1
work=$2
here=$(cd "$(dirname "$0")" && pwd)
cp "$here"/math.test.mjs "$here"/store.test.mjs "$here"/vitest.config.mjs "$work"/
ln -s "$mods/node_modules" "$work/node_modules"
cd "$work" || exit 1
vt=$work/node_modules/.bin/vitest
"$vt" --version
probe() {
  local name=$1
  shift
  local out
  out=$(CI=1 NO_COLOR=1 "$vt" list "$@" 2>&1)
  local code=$?
  printf '### %s: vitest list' "$name"
  printf ' %q' "$@"
  printf '\nexit %s\n' "$code"
  printf '%s\n' "$out" | grep -E '\.test\.mjs|Error|error|No test' | head -8
  echo
}
probe all
probe file-filter math
probe file-line math.test.mjs:7
probe t-substring -t add
probe t-regex-union -t 'add|query'
probe t-regex-exclusion -t '^(?!.*migrate).*'
probe t-full-name -t '^store > migrate$'
probe tags-union --tagsFilter 'slow || db'
probe tags-intersection --tagsFilter 'slow && db'
probe tags-exclusion --tagsFilter 'slow && !db'
probe tags-nesting --tagsFilter '(slow || db) && !(slow && db)'
probe tags-precedence --tagsFilter 'db || slow && !db'
probe tags-repeated --tagsFilter slow --tagsFilter db
probe tags-and-t --tagsFilter slow -t add
probe tags-unknown --tagsFilter nosuchtag
probe tags-syntax-error --tagsFilter 'slow &&'
probe t-empty -t nosuchname
out=$(CI=1 NO_COLOR=1 "$vt" run -t nosuchname 2>&1)
printf '### t-empty-run: vitest run -t nosuchname\nexit %s\n' "$?"
printf '%s\n' "$out" | grep -E 'No test|Tests' | head -3

for n in 1000 10000; do
  e="$(printf '%*s' "$n" '' | tr ' ' '(')slow$(printf '%*s' "$n" '' | tr ' ' ')')"
  out=$(CI=1 NO_COLOR=1 "$vt" list --tagsFilter "$e" 2>&1)
  code=$?
  printf '### tags-parens-%s: vitest list --tagsFilter <%s parens around slow>\nexit %s\n' "$n" "$n" "$code"
  printf '%s\n' "$out" | grep -E '\.test\.mjs|Error' | head -3
  echo
done
