#!/usr/bin/env bash
# PROTOTYPE #391: throwaway, never merges.
#
# score.sh: counts each sweep run's rows by rule and by the verdict in
# data/verdicts.tsv, and lists each known breach a run missed.
set -euo pipefail
shopt -s nullglob

here=$(cd "$(dirname "$0")" && pwd)

printf 'run\trule\tknown\ttrue\trepeat\tdoubtful\tfalse\n'
for run in "${here}"/runs/sweep-*.tsv "${here}"/runs/allrules-*.tsv; do
  name=$(basename "${run}" .tsv)
  head=$(cut -d- -f2 <<<"${name}")
  awk -F'\t' -v head="${head}" -v name="${name}" '
    FNR == NR {
      if ($1 == head) verdict[$2 "|" $3] = $4
      next
    }
    $2 == "-" { next }
    {
      n = split($2, part, "/")
      key = $1 "|" part[n]
      v = (key in verdict) ? verdict[key] : "UNLABELLED"
      count[$1 "|" v]++
      rules[$1] = 1
    }
    END {
      split("R1 R2 R3 C1", order, " ")
      for (i = 1; i <= 4; i++) {
        r = order[i]
        printf "%s\t%s\t%d\t%d\t%d\t%d\t%d", name, r, count[r "|known"], count[r "|true"], count[r "|repeat"], count[r "|doubtful"], count[r "|false"]
        if (count[r "|UNLABELLED"]) printf "\tUNLABELLED %d", count[r "|UNLABELLED"]
        printf "\n"
      }
    }
  ' "${here}/data/verdicts.tsv" "${run}"
done

printf '\nknown breaches missed\n'
tail -n +2 "${here}/data/known.tsv" | while IFS=$'\t' read -r head rule location _; do
  file=${location##*/}
  for run in "${here}"/runs/sweep-"${head}"-*.tsv "${here}"/runs/allrules-"${head}"-*.tsv; do
    if ! awk -F'\t' -v rule="${rule}" -v loc="${location}" \
      '$1 == rule && $2 == loc { found = 1 } END { exit !found }' "${run}"; then
      printf '%s\t%s\t%s\n' "$(basename "${run}" .tsv)" "${rule}" "${file}"
    fi
  done
done
for run in "${here}"/runs/judge-*.tsv; do
  head=$(basename "${run}" .tsv | cut -d- -f2)
  tail -n +2 "${here}/data/known.tsv" | awk -F'\t' -v head="${head}" '$1 == head' |
    while IFS=$'\t' read -r _ rule location _; do
      line=${location##*:}
      if ! grep -qE "one-manifest-declares-each-package\.md:${line}([^0-9]|$)" "${run}"; then
        printf '%s\t%s\t%s\n' "$(basename "${run}" .tsv)" "${rule}" "${location##*/}"
      fi
    done
done
