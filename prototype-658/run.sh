#!/usr/bin/env bash
# PROTOTYPE (#658), never merge. Builds the wheel once, then runs each case
# under each value of PURBA_PROTO_658, and prints purba's lines, its exit code
# and what an atexit hook in the case sees after the failed load.
set -u
out=${1:-target/proto-658}
here=$(cd "$(dirname "$0")" && pwd)
maturin build --quiet --out "${out}/wheel"
python -m venv --clear "${out}/venv"
"${out}/venv/bin/pip" install -q --no-index --find-links "${out}/wheel" purba
purba=$(cd "${out}/venv/bin" && pwd)/purba
cases=(fresh/test_fresh.py onpath/test_onpath.py shadow/abc.py exit/test_exit.py
  builtins/builtins.py)
for case in "${cases[@]}"; do
  for policy in keep remove restore none; do
    echo "### ${case}  PURBA_PROTO_658=${policy}"
    (cd "${here}/cases/$(dirname "${case}")" &&
      PURBA_PROTO_658=${policy} timeout 20 "${purba}" "$(basename "${case}")" 2>&1)
    echo "exit=$?"
  done
done
