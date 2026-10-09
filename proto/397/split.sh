#!/usr/bin/env bash
# PROTOTYPE: these checks are off because the script is never run outside the replay.
# shellcheck disable=SC2312
# PROTOTYPE for #397 — throwaway, never merges.
#
# Counts the two lists the command prints, for each pull request of each run.
#
#   split.sh <reaches.tsv> <run-dir>...
#
# "every" holds the records whose reach is ** or carries commit message, pull
# request or issue: the command names them on every pull request. "change"
# holds every other record the run named.
set -uf
reaches=$1
shift
every=$(awk -F'\t' '$2 ~ /(^| )\*\*( |$)/ || tolower($3) ~ /commit message|pull request|(^|, )issue(,|$)/ { sub(/\.md$/, "", $1); print $1 }' "${reaches}")
printf 'every-PR list (%s): %s\n' "$(wc -w <<<"${every}")" "$(tr '\n' ' ' <<<"${every}")"
for dir in "$@"; do
  set +f
  files=("${dir}"/pr*.tsv)
  set -f
  for f in "${files[@]}"; do
    pr=$(basename "${f}" .tsv)
    named=$(cut -f1 "${f}")
    n_change=$(grep -vxF -f <(printf '%s\n' "${every}") <<<"${named}" | grep -c .)
    n_every=$(grep -xF -f <(printf '%s\n' "${every}") <<<"${named}" | grep -c .)
    printf '%s\t%s\tchange=%s\tevery=%s\ttotal=%s\n' "$(basename "${dir}")" "${pr}" "${n_change}" "${n_every}" "$(grep -c . <<<"${named}")"
  done
done
