#!/usr/bin/env bash
# PROTOTYPE for purba #537: the arrows a layout's code uses that the diagram does not draw.
#
#   check-wiring.sh <landscape> <layout dir>
#
# Diagram arrows come from the mermaid fence in <landscape>. Code arrows come from
# `cargo metadata` when each part is a crate, and from a grep of `crate::<part>` and
# `purba_core::<part>` when parts are modules. Exits 1 when the code uses an
# undrawn arrow.
set -euo pipefail
landscape=$1
layout=$2
parts='assertion|command|dispatch|execution|fixtures|parsing|reporting|selection'

drawn=$(sed -nE "s/^ +(${parts}) -- \"[^\"]*\" --> (${parts})$/\1 \2/p" "${landscape}" | sort -u)

if [[ -d "${layout}/src/command" ]]; then
  # A crate per part: an arrow is a path dependency between two part crates.
  used=$(cargo metadata --offline --no-deps --format-version 1 --manifest-path "${layout}/Cargo.toml" |
    jq -r '.packages[] | select(.name | startswith("purba-")) | .name[6:] as $from
      | .dependencies[] | select(.name | startswith("purba-")) | "\($from) \(.name[6:])"' | sort -u)
  how='cargo metadata'
else
  # Modules: an arrow is a path naming another part's module.
  used=$(find "${layout}/src" -name '*.rs' | while read -r file; do
    from=$(basename "${file}" .rs)
    [[ "${from}" == lib ]] && from=$(basename "$(dirname "$(dirname "${file}")")")
    [[ "${from}" == cli ]] && from=command
    grep -oE "(crate|purba_core)::(${parts})\b" "${file}" | sed -E "s/.*:://; s/^/${from} /" || true
  done | grep -E "^(${parts}) " | awk '$1 != $2' | sort -u)
  how='grep'
fi

undrawn=$(comm -13 <(printf '%s\n' "${drawn}") <(printf '%s\n' "${used}") | sed '/^$/d')
echo "read by ${how}: $(printf '%s\n' "${used}" | sed '/^$/d' | wc -l) arrows in the code, $(printf '%s\n' "${drawn}" | wc -l) in the diagram"
if [[ -n "${undrawn}" ]]; then
  sed 's/ / → /; s/^/  undrawn: /' <<<"${undrawn}"
  exit 1
fi
echo "  every arrow in the code is drawn"
