#!/usr/bin/env bash
# PROTOTYPE #391: throwaway, never merges.
# shellcheck disable=SC2016
#
# build-bundle.sh <base> <head> <out-dir>
#
# Writes what the two agents read:
#   added.txt    every added line under docs/decisions/*.md, as path:line: text
#                (the reach of the STE record), for the sweep
#   rules.md     the prose rules the sweep takes one at a time
#   diff.patch   the whole change, for the one judgement
#   commits.txt  the commit messages, for the one judgement
#   outcomes.md  the Decision Outcome of each record #388 named, for the one
#                judgement
set -euo pipefail
set -f

base=$1
head=$2
out=$3
here=$(cd "$(dirname "$0")" && pwd)
mkdir -p "${out}"

git diff -U0 --no-color "${base}" "${head}" -- 'docs/decisions/*.md' |
  awk '
    /^\+\+\+ / { path = substr($0, 7); next }
    /^@@ / {
      split($3, range, ",")
      line = substr(range[1], 2) + 0
      next
    }
    /^\+/ { printf "%s:%d: %s\n", path, line, substr($0, 2); line++ }
  ' >"${out}/added.txt"

cp "${here}/rules.md" "${out}/rules.md"

git diff --no-color "${base}" "${head}" >"${out}/diff.patch"
git log --format='--- %h%n%B' "${base}..${head}" >"${out}/commits.txt"

: >"${out}/outcomes.md"
while read -r record; do
  file="docs/decisions/${record}.md"
  {
    printf '# %s\n\n' "${record}"
    git show "${base}:${file}" 2>/dev/null ||
      git show "${head}:${file}"
  } | awk '
      NR == 1 { print; next }
      /^## Decision Outcome/ { keep = 1; print; next }
      /^## / { keep = 0 }
      keep
    ' >>"${out}/outcomes.md"
  printf '\n' >>"${out}/outcomes.md"
done <"${here}/data/named-247.txt"

for name in added.txt rules.md diff.patch commits.txt outcomes.md; do
  wc -lw "${out}/${name}"
done
