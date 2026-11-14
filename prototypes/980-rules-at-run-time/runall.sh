#!/usr/bin/env bash
# Run each folder of first attempts and each regression folder under the #979 runner and this one (#980).
# One folder per call: each holds its own loguru_lab.py, and Python keeps the first module of a name.
set -u
here="$(cd "$(dirname "$0")" && pwd)"
py="$here/.venv/bin/python"
mkdir -p "$here/runs"
for dir in first-attempts/filesink-delay first-attempts/threading first-attempts/add-option-catch \
           regress/filesink-delay regress/add-option-catch regress/threading regress/defaults regress/colorama; do
  name="${dir//\//_}"
  for runner in 979 980; do
    if [ "$runner" = 979 ]; then script="$here/../979-setup-ends-before-checks/run.py"; else script="$here/run.py"; fi
    (cd "$here/$dir" && "$py" "$script" .) > "$here/runs/${name}_${runner}.txt" 2>&1
    printf '%-34s %s  %s\n' "$dir" "$runner" "$(tail -n 1 "$here/runs/${name}_${runner}.txt")"
  done
done
(cd "$here/probes" && "$py" ../run.py .) > "$here/runs/probes_980.txt" 2>&1
printf '%-34s %s  %s\n' probes 980 "$(tail -n 1 "$here/runs/probes_980.txt")"
