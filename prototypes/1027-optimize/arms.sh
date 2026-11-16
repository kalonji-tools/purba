#!/usr/bin/env bash
# Q1: when does the warning print? Runs each module under PYTHONOPTIMIZE=1 with
# each arm of PURBA_PROTO_WARN, stderr merged into stdout in the order written.
# Usage: arms.sh <purba with the prototype> <pytest for the control>
set -u
cd "$(dirname "$0")/probes" || exit 1
purba=$1
pytest=$2

for arm in a b c; do
  printf '=== arm %s\n' "${arm}"
  for module in "" missing.py test_syntax.py test_none.py test_bare.py \
    test_helper.py test_mix.py; do
    printf -- '--- $ purba %s\n' "${module}"
    PURBA_PROTO_WARN=${arm} PYTHONPATH=. PYTHONOPTIMIZE=1 \
      "${purba}" ${module:+"${module}"} 2>&1
    printf '(exit %s)\n' "$?"
  done
done

printf '=== pytest control, -O\n'
for module in missing.py test_syntax.py test_none.py test_bare.py test_helper.py; do
  printf -- '--- $ pytest %s\n' "${module}"
  PYTHONPATH=. PYTHONOPTIMIZE=1 "${pytest}" -q -p no:cacheprovider "${module}" 2>&1 |
    grep -E 'Warning|assert|passed|failed|error|no tests|ERROR|not found' | head -4
done
