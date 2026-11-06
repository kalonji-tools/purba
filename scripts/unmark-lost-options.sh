#!/usr/bin/env bash
# Strips the Rejected. mark from each lost option of each record.
#
#   the decision:
# docs/decisions/the-decision-sentence-is-the-winner-and-considered-options-holds-only-what-lost.md
#   the task:      mise run fmt:records
#
#   unmark-lost-options.sh [directory]
#
# Exits 2 when no record is found. Another form of Rejected stays.
set -euo pipefail

# shellcheck source=scripts/report.sh
. "$(dirname "$0")/report.sh"
# shellcheck source=scripts/records.sh
. "$(dirname "$0")/records.sh"

dir=${1:-docs/decisions}

# A leading dot does not match `*`, so the template is left out.
shopt -s nullglob
records=("${dir}"/*.md)
[[ ${#records[@]} -gt 0 ]] || cannot "no record found in ${dir}."

for f in "${records[@]}"; do
  marked=()
  marks=$(option_marks "${f}")
  while read -r number word; do
    [[ "${word}" != Rejected ]] || marked+=("${number}")
  done <<<"${marks}"
  [[ ${#marked[@]} -gt 0 ]] || continue

  # Each line keeps its newline, so the record is written back byte for byte.
  mapfile content <"${f}"
  changed=0
  for number in "${marked[@]}"; do
    line=${content[number - 1]}
    body=${line%$'\n'}
    end=${line:${#body}}
    after=${body#*'**'}
    after=${after#*'**'}
    head=${body%"${after}"}
    after=${after#"${after%%[![:space:]]*}"}
    case "${after}" in
      Rejected.) body=${head} ;;
      'Rejected. '*) body="${head} ${after#'Rejected. '}" ;;
      *) continue ;;
    esac
    content[number - 1]=${body}${end}
    changed=1
  done
  [[ "${changed}" -eq 0 ]] || printf '%s' "${content[@]}" >"${f}"
done
