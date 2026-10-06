#!/usr/bin/env bash
# Run each sentence in probes.txt through the Weir rule and the library
# variants, as its own paragraph, and print one table row per sentence.
# Run keys.sh first: it builds the weirpack. Needs harper-cli 2.3.0 on PATH.
set -euo pipefail
here=$(cd "$(dirname "$0")" && pwd)
lib=${here}/libwalk/target/release/libwalk
md=$(mktemp --suffix=.md)
sed 's/$/\n/' "${here}/probes.txt" >"${md}"
{ harper-cli lint --no-color --format json --weirpacks "${here}/out/PerfectTense.weirpack" \
  --only PerfectTense "${md}" 2>/dev/null || true; } |
  jq -r '.[] | .lints[] | "\(.line)"' >"${md}.weir"
for v in det np det+v np+v; do "${lib}" "${v}" "${md}" | sed 's/^[^:]*://' >"${md}.${v}"; done
printf '| sentence | weir | lib-det | lib-np | lib-det+v | lib-np+v |\n|---|---|---|---|---|---|\n'
n=0
while IFS= read -r sentence; do
  n=$((n + 1))
  line=$((2 * n - 1))
  row="| ${sentence} |"
  if grep -qx "${line}" "${md}.weir"; then row+=" refused |"; else row+=" passes |"; fi
  for v in det np det+v np+v; do
    if grep -q "^${line}: " "${md}.${v}"; then row+=" refused |"; else row+=" passes |"; fi
  done
  echo "${row}"
done <"${here}/probes.txt"
rm -f "${md}" "${md}".*
