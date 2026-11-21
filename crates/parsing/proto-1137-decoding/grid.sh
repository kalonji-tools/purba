#!/usr/bin/env bash
# PROTOTYPE #1137: runs an installed purba on each module of the corpus, with
# each way to decode, on each Python given as `label=path` arguments.
# Run it from the repository root, after `maturin build` (abi3) and a 3.14t wheel
# in target/wheels-t. It writes target/proto-1137/results.tsv and prints it.
set -euo pipefail

here="$(cd "$(dirname "$0")" && pwd)"
out="$PWD/target/proto-1137"
rm -rf "${out}"
mkdir -p "${out}"
decoders=(utf8 latin1-ascii encoding_rs cpython-codecs cpython)

results="${out}/results.tsv"
printf 'python\tmodule\tway\trefusal\texit\tfirst line\n' >"${results}"

for pair in "$@"; do
  label="${pair%%=*}"
  python="${pair#*=}"
  wheels="$PWD/target/wheels"
  [[ "${label}" == *t ]] && wheels="$PWD/target/wheels-t"
  "${python}" -m venv "${out}/venv-${label}"
  "${out}/venv-${label}/bin/python" -m pip install --quiet --no-index --find-links "${wheels}" purba
  corpus="${out}/corpus-${label}"
  "${python}" -I "${here}/corpus.py" "${corpus}" >/dev/null
  (cd "${corpus}" && "${python}" -I "${here}/cpython.py" test_*.py) |
    while IFS=$'\t' read -r module verdict; do
      printf '%s\t%s\tCPYTHON\t-\t-\t%s\n' "${label}" "${module}" "${verdict}" >>"${results}"
    done
  for module in "${corpus}"/test_*.py; do
    name="$(basename "${module}")"
    for decoder in "${decoders[@]}"; do
      for refusal in invocation project; do
        line="$(cd "${corpus}" && PURBA_DECODER="${decoder}" PURBA_REFUSAL="${refusal}" \
          "${out}/venv-${label}/bin/purba" "${name}" 2>&1 | head -1)" || true
        code="$(cd "${corpus}" && PURBA_DECODER="${decoder}" PURBA_REFUSAL="${refusal}" \
          "${out}/venv-${label}/bin/purba" "${name}" >/dev/null 2>&1; echo $?)" || true
        printf '%s\t%s\t%s\t%s\t%s\t%s\n' "${label}" "${name}" "${decoder}" "${refusal}" "${code}" "${line}" >>"${results}"
      done
    done
  done
done
cat "${results}"
