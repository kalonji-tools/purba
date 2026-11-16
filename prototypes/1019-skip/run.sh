#!/usr/bin/env bash
# PROTOTYPE for #1019, never merge. Run each probe and write each output to runs/.
# $1: a venv with the wheel of main and pytest installed.
# $2: the folder of clones that prototypes/1015-rules-cost/run.sh makes.
set -euo pipefail
here="$(cd "$(dirname "$0")" && pwd)"
venv="${1:?usage: run.sh <venv> <corpus folder>}"
corpus="${2:?usage: run.sh <venv> <corpus folder>}"
cd "${here}/probes"
for f in test_*.py; do
  echo "===== ${f}"
  "${venv}/bin/purba" "${f}" || echo "exit=$?"
done >"${here}/runs/probes.txt" 2>&1
cd "${here}/xfail"
{
  "${venv}/bin/python" -m pytest -q -rxXf -p no:cacheprovider test_xf.py || true
  for t in lax_still_broken lax_now_fixed strict_still_broken strict_now_fixed; do
    rc=0
    "${venv}/bin/python" -m pytest -q -p no:cacheprovider "test_xf.py::test_${t}" \
      >/dev/null 2>&1 || rc=$?
    echo "test_${t} exit=${rc}"
  done
} >"${here}/runs/xfail.txt" 2>&1
"${venv}/bin/python" -I "${here}/find_spec.py" >"${here}/runs/find_spec.txt"
"${venv}/bin/python" -I "${here}/skips.py" "${corpus}" >"${here}/runs/skips.txt"
