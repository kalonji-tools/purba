#!/usr/bin/env bash
# PROTOTYPE, never merged. Is #249 a one-off, or a class that recurs?
#
# Replays MD033 (inline HTML) over the Markdown tree of every commit on `main`
# that touched Markdown, with mado because it is the fastest of the three
# checkers that flag 7 of the template's 8 lines. A finding is keyed by file
# and the text of its line, so one defect carried through many commits counts
# once, with the commit that brought it and the commit that removed it.
set -u
cd "$(git rev-parse --show-toplevel)" || exit 2
# shellcheck source=prototype/lib.sh
. prototype/lib.sh
tmp=$(mktemp -d)
mapfile -t shas < <(git log --reverse --format=%h main -- '*.md')
for sha in "${shas[@]}"; do
  dir="${tmp}/${sha}"
  mkdir -p "${dir}"
  git archive "${sha}" -- $(git ls-tree -r --name-only "${sha}" | grep '\.md$') | tar -x -C "${dir}"
  # shellcheck disable=SC2086
  (cd "${dir}" && mise x ${MADO} -- mado check . 2>/dev/null) |
    sed -nE 's/^([^:]+):([0-9]+):[0-9]+: MD033 .*/\1\t\2/p' | sort -u |
    while IFS=$'\t' read -r file line; do
      printf '%s\t%s\t%s\n' "${sha}" "${file#./}" "$(sed -n "${line}p" "${dir}/${file}" | cut -c1-70)"
    done
done >"${tmp}/hits"
printf 'commits replayed: %s\n\n' "${#shas[@]}"
printf '| file | line text | first seen | last seen | commits carried |\n|---|---|---|---|---:|\n'
awk -F'\t' '{k=$2 FS $3; if (!(k in first)) first[k]=$1; last[k]=$1; n[k]++}
  END {for (k in n) {split(k, a, FS); printf "| %s | `%s` | %s | %s | %d |\n", a[1], a[2], first[k], last[k], n[k]}}' \
  "${tmp}/hits" | sort
rm -rf "${tmp}"
