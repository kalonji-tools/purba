#!/usr/bin/env bash
# Measure oxitest's query language at one commit: lines per file, split at the
# first `#[cfg(test)]` line into production and test lines.
# Usage: bash oxitest-size.sh <oxitest checkout> [commit]
# Matcher: a production line is any line before the first line that is exactly
# `#[cfg(test)]`; every line from there on is a test line. Blank lines and
# comments count. A file with no such line is all production.
set -euo pipefail
repo=$1
rev=${2:-731c2b3f}
printf 'file\ttotal\tproduction\ttest\n'
files=$(git -C "$repo" ls-tree -r --name-only "$rev" src/query src/filter.rs src/config/cli/query.rs | grep '\.rs$')
for f in $files; do
  body=$(git -C "$repo" show "${rev}:${f}")
  total=$(printf '%s\n' "$body" | wc -l)
  cut=$(printf '%s\n' "$body" | grep -n -x '#\[cfg(test)\]' | head -1 | cut -d: -f1 || true)
  if [ -z "$cut" ]; then prod=$total; else prod=$((cut - 1)); fi
  printf '%s\t%s\t%s\t%s\n' "$f" "$total" "$prod" "$((total - prod))"
done
for f in python/oxitest/_bridge/query_bridge.py python/tests/integration/test_query.py python/tests/integration/test_query_color.py; do
  total=$(git -C "$repo" show "${rev}:${f}" | wc -l)
  case $f in python/tests/*) printf '%s\t%s\t0\t%s\n' "$f" "$total" "$total" ;; *) printf '%s\t%s\t%s\t0\n' "$f" "$total" "$total" ;; esac
done
