#!/usr/bin/env bash
# PROTOTYPE for #1015, never merge. Clone each suite at its pinned commit into $1,
# run main's selection over every test file, and write each count to runs/.
# The clones are read only: selection parses them, and `ast` reads them.
set -euo pipefail
here="$(cd "$(dirname "$0")" && pwd)"
dest="${1:?usage: run.sh <folder for the clones>}"
repo="$(cd "${here}/../.." && pwd)"
cargo build -q --release --manifest-path "${repo}/Cargo.toml" -p purba-selection --example tally
tally="${repo}/target/release/examples/tally"
mkdir -p "${dest}"
: >"${here}/runs/tally.tsv"
while read -r name url commit; do
  [[ -z "${name}" ]] && continue
  if [[ ! -d "${dest}/${name}" ]]; then
    git init -q "${dest}/${name}"
    git -C "${dest}/${name}" fetch -q --depth 1 "${url}" "${commit}"
    git -C "${dest}/${name}" checkout -q FETCH_HEAD
  fi
  (cd "${dest}" && find "${name}" \( -name 'test_*.py' -o -name '*_test.py' \) -not -path '*/.*' |
    sort | xargs "${tally}" | sed "s/^/${name}\t/") >>"${here}/runs/tally.tsv"
done <"${here}/projects.txt"
"${here}/reasons.sh" "${here}/runs/tally.tsv" >"${here}/runs/reasons.txt"
for script in origin runtime shapes; do
  python -I "${here}/${script}.py" "${here}/runs/tally.tsv" "${dest}" >"${here}/runs/${script}.txt"
done
python -I "${here}/practices.py" "${dest}" >"${here}/runs/practices.txt"
# Stored gzipped: the text gates of the repository would rewrite the test names.
gzip -nf "${here}/runs/tally.tsv"
