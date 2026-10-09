#!/usr/bin/env bash
# PROTOTYPE: these checks are off because the script is never run outside the replay.
# shellcheck disable=SC2312
# PROTOTYPE for #397 — throwaway, never merges.
#
# Replays the wide reaches of #388 and the narrow reaches of #397 through
# proto/388/replay.sh, on the five audited pull requests and the five that
# change no record, with the header links (as #388 measured) and with no links.
#
#   run.sh   (from the repository root)
set -euf
p388=proto/388 p397=proto/397
for pr in 227 234 241 247 271 212 230 254 260 268; do
  git fetch -q origin "refs/pull/${pr}/head:refs/proto388/pr${pr}"
done
nr='212:ef3e87f 230:42b2e7e 254:e4fe7a7 260:3e311d4 268:9e55faf'
for variant in wide:"${p388}/data/reaches.tsv" narrow:"${p397}/data/narrow-reaches.tsv"; do
  name=${variant%%:*} reaches=${variant#*:}
  LINKS=headers "${p388}/replay.sh" "${reaches}" "${p388}/data/breaches.tsv" "${p397}/runs/${name}"
  LINKS=headers PRS=${nr} "${p388}/replay.sh" "${reaches}" "${p388}/data/breaches.tsv" "${p397}/runs/nr-${name}"
  LINKS=none "${p388}/replay.sh" "${reaches}" "${p388}/data/breaches.tsv" "${p397}/runs/${name}-nolinks"
  "${p397}/split.sh" "${reaches}" "${p397}/runs/${name}" "${p397}/runs/nr-${name}"
done
