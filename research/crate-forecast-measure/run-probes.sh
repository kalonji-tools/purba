#!/usr/bin/env bash
# run-probes.sh OUT CRATES_TSV
# Runs probe.sh for each crate, one after another, and times each.
# Run it from a purba checkout:  mise exec -- bash run-probes.sh OUT crates.tsv
set -uo pipefail
out=$1
list=$2
here=$(cd "$(dirname "$0")" && pwd)
mkdir -p "${out}"
: >"${out}/probe-times.tsv"
grep -v '^#' "${list}" >"${out}/probe.list"
while IFS=$'\t' read -r c v; do
  s=$(date +%s)
  bash "${here}/probe.sh" "${out}" "${c}" "${v}"
  e=$(date +%s)
  printf '%s\t%s\n' "${c}" "$((e - s))" >>"${out}/probe-times.tsv"
done <"${out}/probe.list"
