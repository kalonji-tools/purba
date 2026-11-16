#!/usr/bin/env bash
# Q2: which test arm goes red when a part of the change is reverted?
#   arm a: tests/optimize.bats alone, as the run sets PYTHONOPTIMIZE per case
#   arm b: every bats file with PYTHONOPTIMIZE=1 exported (arm a's file included)
# Mutations, as switches of the prototype wheel:
#   none     the change whole        (PURBA_PROTO_WARN=a)
#   interp   optimize=0 reverted     (PURBA_PROTO_MUT=interp)
#   nowarn   the warning dropped     (PURBA_PROTO_WARN unset)
# Usage: mutations.sh <venv bin of the wheel> <purba of the fault wheel> <bats>
set -u
cd "$(dirname "$0")/../.." || exit 1
bin=$1
fault=$2
bats=$3

summary() { grep -E '^(not ok|[0-9]+ tests?,)' | sed 's/^/    /'; }

for mutation in none interp nowarn; do
  warn=a mut=
  [[ ${mutation} == interp ]] && mut=interp
  [[ ${mutation} == nowarn ]] && warn=
  printf '=== mutation %s\n' "${mutation}"
  printf -- '--- arm a: tests/optimize.bats\n'
  start=${SECONDS}
  PATH="${bin}:${PATH}" FAULT_PURBA=${fault} PURBA_PROTO_WARN=${warn} PURBA_PROTO_MUT=${mut} \
    "${bats}" tests/optimize.bats 2>&1 | summary
  printf '    (%ss)\n' "$((SECONDS - start))"
  printf -- '--- arm b: every bats file, PYTHONOPTIMIZE=1\n'
  start=${SECONDS}
  PATH="${bin}:${PATH}" FAULT_PURBA=${fault} PURBA_PROTO_WARN=${warn} PURBA_PROTO_MUT=${mut} \
    PYTHONOPTIMIZE=1 "${bats}" tests 2>&1 | summary
  printf '    (%ss)\n' "$((SECONDS - start))"
done

printf '=== plain baseline: every bats file, no -O, the change whole\n'
start=${SECONDS}
PATH="${bin}:${PATH}" FAULT_PURBA=${fault} PURBA_PROTO_WARN=a "${bats}" tests 2>&1 | summary
printf '    (%ss)\n' "$((SECONDS - start))"
