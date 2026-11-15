#!/usr/bin/env bash
# Run the corpus under replay and under the fork tree (#1010), one folder or file per call.
# Each target writes runs/<name>_<mode>.txt, and runs/<name>_<mode>.acts: one line for each act
# of the code under test, from PIPE_LOG. runs/compare.txt holds one line for each target: the two
# outputs compared as text, with the fork tree's fallback lines left out, and the acts of each.
set -u
here="$(cd "$(dirname "$0")" && pwd)"
p980="${here}/../980-rules-at-run-time"
py="${here}/.venv/bin/python"
out="${here}/runs"
mkdir -p "${out}"
: >"${out}/compare.txt"

run() { # name, folder, target
  local name="$1" dir="$2" target="$3" mode
  for mode in replay tree; do
    local flags=()
    if [[ "${mode}" == tree ]]; then flags=(--tree); fi
    rm -f "${out}/${name}_${mode}.acts"
    touch "${out}/${name}_${mode}.acts"
    (
      cd "${dir}" || exit 1
      PIPE_LOG="${out}/${name}_${mode}.acts" PYTHONPATH="${here}/undertest_lib" \
        "${py}" "${here}/run.py" "${flags[@]}" "${target}"
    ) >"${out}/${name}_${mode}.txt" 2>&1
    grep -v '^   ↺ ' "${out}/${name}_${mode}.txt" >"${out}/.${mode}.cmp"
  done
  local acts_r acts_t same=same last
  acts_r=$(wc -l <"${out}/${name}_replay.acts")
  acts_t=$(wc -l <"${out}/${name}_tree.acts")
  if ! cmp -s "${out}/.replay.cmp" "${out}/.tree.cmp"; then same=DIFFERENT; fi
  rm -f "${out}/.replay.cmp" "${out}/.tree.cmp"
  last=$(tail -n 1 "${out}/${name}_tree.txt")
  printf '%-36s %-9s acts %4s → %4s  %s\n' "${name}" "${same}" "${acts_r}" "${acts_t}" "${last}" |
    tee -a "${out}/compare.txt"
}

for dir in first-attempts/filesink-delay first-attempts/threading \
  first-attempts/add-option-catch regress/filesink-delay regress/add-option-catch \
  regress/threading regress/defaults regress/colorama probes; do
  run "980_${dir//\//_}" "${p980}/${dir}" .
done
run pipeline "${here}/pipeline" test_pipeline.py
# alone: importing it imports test_pipeline.py unrewritten
run pipeline_part "${here}/pipeline" test_part.py
run chains "${here}/chains" .
