#!/usr/bin/env bash
# PROTOTYPE #868: build the fault wheel and run the point table. Never merge.
# Usage: proto-868/run.sh <label>   (writes proto-868/C-<label>.txt)
set -euo pipefail
label=$1
mise exec -- maturin build --quiet --features extension-module,fault --out target/wheels-fault
mise exec -- python -m venv --clear target/fault-venv
target/fault-venv/bin/python -m pip install --quiet --force-reinstall --no-index \
  --find-links target/wheels-fault purba
FAULT_PURBA="$PWD/target/fault-venv/bin/purba" mise exec -- \
  bats --print-output-on-failure tests/proto_868_interrupt_points.bats >"proto-868/C-${label}.bats.txt" 2>&1 || true
cp proto-868/points.txt "proto-868/C-${label}.txt"
cat "proto-868/C-${label}.bats.txt"
