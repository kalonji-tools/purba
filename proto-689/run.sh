#!/usr/bin/env bash
# PROTOTYPE for "What is a test's interface: what it takes, what it returns, and
# what may change around it?" (#689). Throwaway: this branch never merges.
#
#   CORPUS=<a checkout of click> mise exec -- bash proto-689/run.sh
#
# Asks: can the TEXT of a test show that it is isolated, so that purba runs the
# isolated tests together in one interpreter and gives only the rest a process?
#   bench/       what one test costs in each beaker, on a toy module
#   classify.py  what the text says: L0 signature, L1 + body, L2 + calls
#   leakwatch.py what happened: shared state diffed around each test
#   planted/     a control whose leaks are known in advance
#   reorder.py   whether any leak changes an outcome (reverse, 3 shuffles)
#   forkbeaker.py the price of no beaker, a fork for each test, and the bundles
# Writes every output to proto-689/runs/, except the detector's raw output, which stays
# in the work directory: click's test ids hold mixed-case words that typos refuses.
set -euo pipefail

here=$(cd "$(dirname "$0")" && pwd)
runs="${here}/runs"
work=${PROTO_WORK:-$(mktemp -d)}
corpus=${CORPUS:?set CORPUS to a checkout of click}
mkdir -p "${runs}" "${work}/click"

git -C "${corpus}" archive HEAD | tar -x -C "${work}/click"
git -C "${corpus}" log -1 --format='click at %h %s' >"${runs}/corpus.txt"
python -m venv --clear "${work}/venv"
py="${work}/venv/bin/python"
"${py}" -m pip install --quiet --disable-pip-version-check pytest "${work}/click"
"${py}" -VV >>"${runs}/corpus.txt"
"${py}" -m pytest --version >>"${runs}/corpus.txt" 2>&1
site=$("${py}" -c 'import click, os; print(os.path.dirname(click.__file__))')

echo "== bench: one test in each beaker"
"${py}" -I "${here}/bench/bench.py" "${here}/bench/mods" | tee "${runs}/bench.txt"

echo "== a process for each test, with the module body run once per module"
count="${work}/lab_count.txt"
{
  for round in 1 2; do
    for method in fork:200 forkserver:200 bare:100 spawn:50; do
      LAB_COUNT=${count} "${py}" -I "${here}/bench/forkserver.py" "${here}/bench/mods" \
        "${method%%:*}" "${method##*:}"
    done
  done
  for module in lab_broken lab_import_broken; do
    for method in fork forkserver spawn; do
      line=$(LAB_MODULE=${module} LAB_COUNT=${count} "${py}" -I "${here}/bench/forkserver.py" \
        "${here}/bench/mods" "${method}" 5 2>"${work}/stderr.txt" || echo "${method} exits $?")
      lines=$(wc -l <"${work}/stderr.txt")
      printf '%-18s %s   stderr: %s lines\n' "${module}" "${line}" "${lines}"
    done
  done
} | tee "${runs}/forkserver.txt"

judge() { # name, tests dir, watched roots, pytest args...
  local name=$1 tests=$2 roots=$3
  shift 3
  "${py}" -I "${here}/classify.py" "${tests}" >"${runs}/${name}-static.json"
  PYTHONPATH="${here}" LEAKWATCH_OUT="${work}/${name}-dynamic.json" LEAKWATCH_ROOTS="${roots}" \
    "${py}" -m pytest -q -p no:cacheprovider -p leakwatch "$@" | tail -1
  "${py}" -I "${here}/report.py" "${runs}/${name}-static.json" "${work}/${name}-dynamic.json" \
    "${EACH:-}" | tee "${runs}/${name}-report.txt"
}

echo "== planted control"
cp "${here}/planted/test_zz_planted.py" "${work}/planted.py.tmp"
mkdir -p "${work}/planted"
mv "${work}/planted.py.tmp" "${work}/planted/test_zz_planted.py"
(cd "${work}/planted" && EACH=--each judge planted "${work}/planted" "${site}:${work}/planted" \
  --rootdir="${work}/planted" "${work}/planted")

echo "== click"
(cd "${work}/click" && judge click "${work}/click/tests" "${site}:${work}/click/tests" tests)

echo "== does any leak change an outcome?"
: >"${runs}/orders.txt"
for how in "" reverse 1 2 3; do
  printf '%-8s ' "${how:-forward}" | tee -a "${runs}/orders.txt"
  (cd "${work}/click" && REORDER=${how} PYTHONPATH="${here}" \
    "${py}" -m pytest -q -p no:cacheprovider -p reorder tests | tail -1) |
    tee -a "${runs}/orders.txt"
done

echo "== the price of each beaker, on click"
(cd "${work}/click" && "${py}" -m pytest -q -p no:cacheprovider --durations=0 tests) |
  awk '/^[0-9.]+s (setup|call|teardown)/ { s += $1; n += 1 }
    END { printf "in the parent: %.2f ms an item, setup + call + teardown, from --durations\n",
      1000 * s / 1978 }' |
  tee "${runs}/durations.txt"
"${py}" -I -c '
import json, sys
t = json.load(open(sys.argv[1]))["tests"]
for level in ("L1_body", "L2_calls"):
    with open(f"{sys.argv[2]}/fork-{level}.txt", "w") as out:
        out.write("\n".join(k for k, v in t.items() if not v[level]))
' "${runs}/click-static.json" "${work}"
: >"${runs}/fork.txt"
for round in 1 2 3; do
  for how in "" all "${work}/fork-L1_body.txt" "${work}/fork-L2_calls.txt"; do
    label=$(basename "${how:-none}")
    line=$(cd "${work}/click" && FORK=${how} PYTHONPATH="${here}" \
      "${py}" -m pytest -q -p no:cacheprovider -p forkbeaker tests |
      grep -E 'forkbeaker|passed|failed' | tr '\n' ' ')
    printf 'round %s  %-18s %s\n' "${round}" "${label}" "${line}" | tee -a "${runs}/fork.txt"
  done
done
echo "outputs in ${runs}"
