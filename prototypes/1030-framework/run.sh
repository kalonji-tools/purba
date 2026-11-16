#!/usr/bin/env bash
# Run each probe module alone under the installed purba; print its output and exit code.
# PURBA names the purba command (default: the one on PATH).
set -u
cd "$(dirname "$0")/probes" || exit 1
for f in test_*.py; do
  echo "=== ${f}"
  env PYTHONDONTWRITEBYTECODE=1 PYTHONPATH=. "${PURBA:-purba}" "${f}" 2>&1
  echo "exit $?"
done
