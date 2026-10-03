#!/usr/bin/env bash
# PROTOTYPE, never merged. Where do typos findings land in real code?
#
# Runs purba's pinned typos over three local trees that never ran it, and sorts
# each fixable finding (one correction) by what `-w` would rewrite:
#   prose    a .md, .rst or .txt file
#   comment  a code line whose text before the typo holds `#` or `//`
#   code     any other code line: an identifier or a string, which `-w` renames
#
# A minified file is left out of the rows: oxitest vendors `mermaid.min.js`, which
# alone holds 1,480 findings, and `-w` would rename its identifiers.
set -u
cd "$(dirname "$0")/.." || exit 2
printf '| tree | findings | one correction | prose | comment | code (identifier or string) |\n'
printf '|---|---:|---:|---:|---:|---:|\n'
for tree in ../oxitest ../click ../loguru; do
  [[ -d "${tree}" ]] || continue
  mise x -- typos --format json "${tree}" 2>/dev/null | jq -r '
    select(.type == "typo")
    | select(.path | test("\\.min\\.js$") | not)
    | [.path, (.line_num|tostring), (.byte_offset|tostring), (.corrections|length|tostring), .typo] | @tsv' |
    while IFS=$'\t' read -r path line offset ncorr typo; do
      kind=code
      case "${path}" in *.md | *.rst | *.txt) kind=prose ;; esac
      if [[ "${kind}" == code ]]; then
        before=$(sed -n "${line}p" "${path}" | head -c "${offset}")
        [[ "${before}" == *'#'* || "${before}" == *'//'* ]] && kind=comment
      fi
      printf '%s\t%s\n' "${ncorr}" "${kind}"
    done | awk -F'\t' -v t="$(basename "${tree}")" '
      {n++} $1 == 1 {one++; k[$2]++}
      END {printf "| %s | %d | %d | %d | %d | %d |\n", t, n, one, k["prose"], k["comment"], k["code"]}'
done
