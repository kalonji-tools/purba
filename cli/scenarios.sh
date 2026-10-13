#!/usr/bin/env bash
# PROTOTYPE for purba #536 Q2a: run both shapes of the command through the same scenarios.
# S is a scratch directory whose wheels/ holds the three wheels; PY312 and PY314 are
# a 3.12 interpreter and a free-threaded 3.14 interpreter.
set -u
S=${S:?set S to a scratch directory holding wheels/}
TREE=$(cd "$(dirname "$0")/.." && pwd)
PY312=${PY312:?}
PY314=${PY314:?}
# A base python3 with no purba installed is first on PATH.
SYS_PATH=/run/current-system/sw/bin:$(dirname "${PY312}")

rm -rf "${S}/A" "${S}/B" "${S}/tools"
"${PY312}" -m venv "${S}/A" # the tester's project venv, both packages installed
"${PY314}" -m venv "${S}/B" # a second project venv, on a free-threaded python
"${S}/A/bin/pip" -q install --no-index \
  "${S}"/wheels/purba-0.0.0-cp312-abi3-*.whl "${S}"/wheels/purba_cli-*.whl
"${S}/B/bin/pip" -q install --no-index "${S}"/wheels/purba-0.0.0-cp314-cp314t-*.whl
# One global copy of the executable, as `uv tool` or `cargo install` gives.
mkdir -p "${S}/tools/bin" && cp "${S}/A/bin/purba-exe" "${S}/tools/bin/"

cd "${TREE}/tests/skeleton" || exit 1
show() { # $1 title, rest: command
  local title=$1
  shift
  echo "━━ ${title}"
  echo "\$ $*"
  "$@"
  echo "exit: $?"
  echo
}

echo "######## S1: installed in the project venv, venv not activated"
show "crate shape" env PATH="${SYS_PATH}" "${S}/A/bin/purba" test_a.py
show "executable shape" env PATH="${SYS_PATH}" "${S}/A/bin/purba-exe" test_a.py

echo "######## S2: one global executable, the tester's venv B activated"
B_ENV=(env PATH="${S}/B/bin:${SYS_PATH}" VIRTUAL_ENV="${S}/B")
show "executable shape, global copy" "${B_ENV[@]}" "${S}/tools/bin/purba-exe" test_a.py
show "crate shape: venv B's own console script" "${B_ENV[@]}" purba test_a.py

echo "######## S3: one global executable, no venv activated"
show "executable shape, global copy" env PATH="${SYS_PATH}" "${S}/tools/bin/purba-exe" test_a.py
show "crate shape, nothing on PATH" env PATH="${SYS_PATH}" bash -c 'purba test_a.py'

echo "######## S4: time for 20 runs, S1 setup"
for cmd in "${S}/A/bin/purba" "${S}/A/bin/purba-exe"; do
  start=$(date +%s%N)
  for _ in $(seq 20); do env PATH="${SYS_PATH}" "${cmd}" test_a.py >/dev/null 2>&1; done
  end=$(date +%s%N)
  name=$(basename "${cmd}")
  echo "${name}: $(((end - start) / 20000000)) ms per run"
done
