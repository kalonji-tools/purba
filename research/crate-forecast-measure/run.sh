#!/usr/bin/env bash
# run.sh OUT UNTIL
# Measures blast area and the seven risks of each crate in crates.tsv, then compares the levels
# with baseline.tsv. Run it from a purba checkout, with OUT in a scratch directory:
#   GH_TOKEN=$(gh auth token) bash research/crate-forecast-measure/run.sh "$OUT" "$UNTIL"
# UNTIL cuts the crates.io versions and the commit history. 2026-11-14T10:18:47Z is the commit
# time of research/future-crates.md (8607015). Use the current time for a forecast.
# The probes build each crate, so the build scripts and proc macros of each crate run.
set -euo pipefail
out=$1
until=$2
here=$(cd "$(dirname "$0")" && pwd)
purba=$(git -C "${here}" rev-parse --show-toplevel)
mkdir -p "${out}"
grep -v '^#' "${here}/crates.tsv" | cut -f1 >"${out}/crates.names"
mapfile -t crates <"${out}/crates.names"
bash "${here}/fetch-cratesio.sh" "${out}/cratesio" "${crates[@]}"
(cd "${purba}" && mise exec -- bash "${here}/run-probes.sh" "${out}" "${here}/crates.tsv")
bash "${here}/run-github.sh" "${out}" "${until}"
bash "${here}/levels.sh" "${out}" "${until}" "${purba}" >"${out}/levels.tsv"
bash "${here}/compare.sh" "${out}/levels.tsv"
