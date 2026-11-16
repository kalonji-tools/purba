#!/usr/bin/env bash
# Runs each probe under two installed purbas, plainly and under PYTHONOPTIMIZE.
# Usage: run.sh <purba of main> <purba with optimize=0>
set -u
cd "$(dirname "$0")/probes" || exit 1

first_line() {
  local level=$1 purba=$2 probe=$3 out
  out=$(PYTHONPATH=. PYTHONOPTIMIZE=${level} "${purba}" "${probe}.py" 2>&1) || true
  printf '%s\n' "${out%%$'\n'*}"
}

probes="test_helper test_ctx test_expect test_bare test_cut test_sibling"
probes+=" test_own test_debug test_flag test_doc"
for probe in ${probes}; do
  level=1
  [[ ${probe} == test_doc ]] && level=2
  for arm in "main:$1" "opt0:$2"; do
    name=${arm%%:*}
    purba=${arm#*:}
    plain=$(first_line "" "${purba}" "${probe}")
    optimized=$(first_line "${level}" "${purba}" "${probe}")
    printf '%s\t%s\tplain: %s\n' "${probe}" "${name}" "${plain}"
    printf '%s\t%s\toptimized: %s\n' "${probe}" "${name}" "${optimized}"
  done
done
