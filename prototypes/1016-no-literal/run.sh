#!/usr/bin/env bash
# Runs each probe under the purba on PATH and prints its output and exit code.
set -u
cd "$(dirname "${0}")/probes" || exit 1
for f in *.py; do
  printf '### %s\n' "${f}"
  purba "${f}" 2>&1
  printf 'exit %s\n\n' "${?}"
done
