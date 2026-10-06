#!/usr/bin/env bash
# Score each variant against the labels in out/. Run corpus.sh first, and keys.sh.
#
# A true tense is any line some variant refuses that out/wild-false.keys does
# not name. A tense no variant refuses is not counted.
set -euo pipefail
cd "$(dirname "$0")/out"
variants=(walk weir union lib-det lib-np lib-det+v lib-np+v)
for v in "${variants[@]}"; do cat "wild-${v}.keys"; done |
  sort -u | comm -23 - wild-false.keys >wild-true.keys
printf '| variant | records: true / missed / false | wild: true / missed / false '
printf '| misses a line the walk catches | refuses a line the walk passes |\n'
printf '|---|---|---|---|---|\n'
for v in "${variants[@]}"; do
  rt=$(comm -12 "rec-${v}.keys" rec-true.keys | wc -l)
  rm=$(comm -13 "rec-${v}.keys" rec-true.keys | wc -l)
  rf=$(comm -23 "rec-${v}.keys" rec-true.keys | wc -l)
  wt=$(comm -12 "wild-${v}.keys" wild-true.keys | wc -l)
  wm=$(comm -13 "wild-${v}.keys" wild-true.keys | wc -l)
  wf=$(comm -12 "wild-${v}.keys" wild-false.keys | wc -l)
  mw=$(comm -12 wild-walk.keys wild-true.keys | comm -23 - "wild-${v}.keys" | paste -sd' ')
  fw=$(comm -12 "wild-${v}.keys" wild-false.keys | comm -23 - wild-walk.keys | paste -sd' ')
  printf '| %s | %s / %s / %s | %s / %s / %s | %s | %s |\n' "${v}" "${rt}" "${rm}" "${rf}" \
    "${wt}" "${wm}" "${wf}" "${mw:-none}" "${fw:-none}"
done
