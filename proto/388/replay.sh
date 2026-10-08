#!/usr/bin/env bash
# PROTOTYPE: these checks are off because the script is never run outside the replay.
# shellcheck disable=SC2001,SC2012,SC2016,SC2312
# PROTOTYPE for #388 — throwaway, never merges.
#
# Replays name-records.sh on the five audited pull requests.
#
#   replay.sh <reaches.tsv|none> <breaches.tsv> <outdir>
#
# reaches.tsv: <record file> TAB <patterns|-> TAB <glossary words|-> TAB <why>.
# "none" replays the links alone, the control that #374 measured.
# breaches.tsv: <pull request> TAB <record slug> TAB <the breach>.
set -uf # -f: a pattern such as ** must reach the attributes file, not the shell
here=$(cd "$(dirname "$0")" && pwd)
reaches=$1 breaches=$2 out=$3
mkdir -p "${out}"

prs=${PRS:-'227:37607d4 234:0505b4b 241:8fca1c5 247:fcb019a 271:11222de'}

printf 'pr\trecords at base\tnamed\tbreaches\tbreaches named\n' >"${out}/summary.tsv"
for pair in ${prs}; do
  pr=${pair%%:*} base=${pair#*:} head=refs/proto388/pr${pr}
  dir=$(mktemp -d)
  git ls-tree --name-only "${base}" docs/decisions/ | grep -v -e '/\.template' -e '/README' |
    while read -r f; do
      name=${f##*/}
      reach=''
      if [[ "${reaches}" != none ]]; then
        row=$(awk -F'\t' -v n="${name}" '$1 == n' "${reaches}")
        if [[ -n "${row}" ]]; then
          patterns=$(cut -f2 <<<"${row}") words=$(cut -f3 <<<"${row}")
          parts=()
          [[ "${patterns}" == - ]] || for p in ${patterns}; do parts+=("\`${p}\`"); done
          if [[ "${words}" != - ]]; then
            IFS=, read -ra ws <<<"${words}"
            for w in "${ws[@]}"; do
              w=$(sed 's/^ *//; s/ *$//' <<<"${w}")
              slug=$(tr 'A-Z ' 'a-z-' <<<"${w}")
              parts+=("[${w}](../../CONTEXT.md#${slug})")
            done
          fi
          reach="**Reach:** $(
            IFS=,
            echo "${parts[*]}" | sed 's/,/, /g'
          )"
        fi
      fi
      git show "${base}:${f}" |
        awk -v r="${reach}" '{ print } /^## Decision Outcome$/ && r != "" { print ""; print r }' >"${dir}/${name}"
    done
  total=$(ls "${dir}" | wc -l)
  "${here}/name-records.sh" "${base}" "${head}" "${dir}" >"${out}/pr${pr}.tsv" 2>"${out}/pr${pr}.err"
  named=$(wc -l <"${out}/pr${pr}.tsv")
  b=$(awk -F'\t' -v p="${pr}" '$1 == p' "${breaches}" | wc -l)
  hit=$(awk -F'\t' -v p="${pr}" 'NR == FNR { n[$1] = 1; next } $1 == p && ($2 in n)' \
    "${out}/pr${pr}.tsv" "${breaches}" | wc -l)
  printf '%s\t%s\t%s\t%s\t%s\n' "${pr}" "${total}" "${named}" "${b}" "${hit}" >>"${out}/summary.tsv"
  rm -rf "${dir}"
done
column -t -s$'\t' "${out}/summary.tsv"
awk -F'\t' 'NR > 1 { b += $4; h += $5 } END { print "breaches named: " h " of " b }' "${out}/summary.tsv"
