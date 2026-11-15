#!/usr/bin/env bash
# The injected controls (#1010), not corpus evidence. The coin runs 40 times in each mode.
# The open file runs once in each mode, and once more under the fork tree with its fallback
# for an open descriptor switched off.
set -u
here="$(cd "$(dirname "$0")" && pwd)"
py="${here}/.venv/bin/python"
cd "${here}/controls" || exit 1
out="${here}/runs"
: >"${out}/controls.txt"

runner() { # mode, test file, output; FORM_FDS passes through
  local flags=()
  if [[ "$1" == tree ]]; then flags=(--tree); fi
  PYTHONPATH="${here}/undertest_lib" "${py}" "${here}/run.py" "${flags[@]}" "$2" >"$3" 2>&1
}

for mode in replay tree; do
  tally="${out}/.coin_${mode}"
  : >"${tally}"
  for _ in {1..40}; do
    last="${out}/.coin_last"
    runner "${mode}" test_coin.py "${last}"
    own=fail later=blocked why=-
    if grep -q '✓ tossed ›' "${last}"; then own=pass; fi
    if grep -q '✓ called ›' "${last}"; then later=pass; fi
    if grep -q 'diverged on replay' "${last}"; then why=diverged; fi
    echo "${mode} tossed=${own} called=${later} ${why}" >>"${tally}"
    rm -f "${last}"
  done
  sort "${tally}" >"${tally}.sorted"
  uniq -c "${tally}.sorted" >"${tally}.counted"
  sed 's/^/coin  /' "${tally}.counted" >>"${out}/controls.txt"
  rm -f "${tally}" "${tally}.sorted" "${tally}.counted"
  runner "${mode}" test_handle.py "${out}/handle_${mode}.txt"
  last=$(tail -n 1 "${out}/handle_${mode}.txt")
  echo "handle ${mode}: ${last}" >>"${out}/controls.txt"
done
FORM_FDS=off runner tree test_handle.py "${out}/handle_tree_fds-off.txt"
last=$(tail -n 1 "${out}/handle_tree_fds-off.txt")
echo "handle tree, FORM_FDS=off: ${last}" >>"${out}/controls.txt"
cat "${out}/controls.txt"
