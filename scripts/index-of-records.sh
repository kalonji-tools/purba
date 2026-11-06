#!/usr/bin/env bash
# The index of the records, written from the area line of each record.
#
#   the decision:
# docs/decisions/a-record-names-its-area-and-docs-readme-lists-the-records-by-area.md
#   the tasks:     mise run fmt:index, mise run lint:index
#
#   index-of-records.sh            writes docs/README.md
#   index-of-records.sh --check    changes nothing, and refuses a page that differs
#
# Exits 1 when it refuses, and 2 when a record or the landscape cannot be read.
set -euo pipefail

# shellcheck source=scripts/report.sh
. "$(dirname "$0")/report.sh"
# shellcheck source=scripts/records.sh
. "$(dirname "$0")/records.sh"

check=0
case "${1:-}" in
  --check) check=1 ;;
  '') ;;
  *) cannot "unknown option: $1" ;;
esac

top=$(git rev-parse --show-toplevel 2>/dev/null) ||
  cannot 'the records can only be found inside a git repository.'
cd "${top}"

# The records of one area follow their file names, in the same order on every
# machine. A leading dot does not match `*`, so the template is left out.
export LC_ALL=C
shopt -s nullglob
records=(docs/decisions/*.md)
[[ ${#records[@]} -gt 0 ]] || cannot 'no record found in docs/decisions.'

names=''
[[ ! -r LANDSCAPE.md ]] || names=$(area_names LANDSCAPE.md)
[[ -n "${names}" ]] ||
  cannot 'LANDSCAPE.md names no part or concern, so no area can be listed.'
declare -A rows=()
while IFS= read -r name; do
  rows[${name}]=''
done <<<"${names}"

for f in "${records[@]}"; do
  area_read "${f}" at areas
  [[ -n "${areas}" ]] ||
    cannot "${f} names no area, so the index cannot list it. mise run lint:records names the line."
  title=$(head -n 1 "${f}")
  [[ "${title}" == '# '?* ]] || cannot "${f} holds no title on its first line."
  while IFS= read -r name; do
    [[ -v rows[${name}] ]] ||
      cannot "${f} names ${name}, which is no area. mise run lint:records names the list."
    rows[${name}]+="- [${title#'# '}](${f#docs/})"$'\n'
  done <<<"${areas}"
done

page="# The decision records, by area"$'\n\n'
page+="\`mise run fmt:index\` writes this page from the \`**Area:**\` line of each record."$'\n'
grouped=0
while IFS= read -r name; do
  [[ -n "${rows[${name}]}" ]] || continue
  case "${name}" in
    design) page+=$'\n## Product design\n\n' ;;
    workflow) page+=$'\n## The pull-request workflow\n\n' ;;
    writing) page+=$'\n## How a record is written\n\n' ;;
    tooling) page+=$'\n## Repository tooling\n\n' ;;
    *)
      [[ "${grouped}" -eq 1 ]] || page+=$'\n## One part or concern\n'
      grouped=1
      page+=$'\n'"### ${name}"$'\n\n'
      ;;
  esac
  page+=${rows[${name}]}
done <<<"${names}"

if [[ "${check}" -eq 0 ]]; then
  printf '%s' "${page}" >docs/README.md
  finish
fi

if [[ ! -e docs/README.md ]]; then
  differs='docs/README.md does not exist.'
else
  differs=$(diff -u --label docs/README.md --label 'the page the area lines give' \
    docs/README.md - <<<"${page%$'\n'}") || [[ $? -eq 1 ]] ||
    cannot 'docs/README.md cannot be compared with the page the area lines give.'
fi
[[ -z "${differs}" ]] || refuse \
  "docs/README.md is refused when it differs from the page that the area lines give, because a \
person finds a record there. Run mise run fmt:index." \
  "${differs}"
finish
