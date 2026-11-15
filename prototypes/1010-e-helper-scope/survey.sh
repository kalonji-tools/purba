#!/usr/bin/env bash
# Clone each project at its pinned commit into $1 (one new folder each), survey it,
# and print the table. The clones are read with `ast` only; nothing in them is imported or run.
set -euo pipefail
here="$(cd "$(dirname "$0")" && pwd)"
dest="${1:?usage: survey.sh <folder for the clones> [python]}"
py="${2:-python3}"
mkdir -p "${dest}" "${here}/runs"
names=()
while read -r name url commit; do
  [[ -z "${name}" ]] && continue
  if [[ ! -d "${dest}/${name}" ]]; then
    git init -q "${dest}/${name}"
    git -C "${dest}/${name}" fetch -q --depth 1 "${url}" "${commit}"
    git -C "${dest}/${name}" checkout -q FETCH_HEAD
  fi
  "${py}" -I "${here}/survey.py" "${dest}/${name}" "${name}" >"${here}/runs/${name}.txt"
  names+=("${name}")
done <"${here}/projects.txt"
"${py}" -I "${here}/survey.py" --table "${names[@]}" | tee "${here}/runs/table.md"
