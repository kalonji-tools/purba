#!/usr/bin/env bash
# PROTOTYPE, never merged. Does each checker catch the markdown defects purba
# actually shipped?
#
#   control   planted MD018 and MD012. A checker that reports nothing here did
#             not run, and its other rows are VOID.
#   F1        #216: a Downside item run onto the previous line (a42369a^ vs a42369a)
#   F2        #249: nine template placeholders GitHub renders as nothing
#   F3        #41:  a blank line after the last line (df3e6a1^ vs df3e6a1)
#
# For F1 and F3 a catch is a rule that fires more often on the defect than on
# its fix. For F2 a catch is a finding on a line that holds a lost placeholder.
set -u
cd "$(git rev-parse --show-toplevel)" || exit 2
# shellcheck source=prototype/lib.sh
. prototype/lib.sh
fx=prototype/fixtures
f2_lines=' 63 68 72 73 79 83 87 88 '

delta() { # the rules that fire more on $1 than on $2
  join -a1 -e0 -o '0,1.2,2.2' <(cut -f3 "$1" | sort | uniq -c | awk '{print $2, $1}' | sort) \
    <(cut -f3 "$2" | sort | uniq -c | awk '{print $2, $1}' | sort) |
    awk '$2 > $3 {printf "%s%s +%d", sep, $1, $2 - $3; sep=", "}'
}

printf '| checker | control | F1 run-on item | F2 lost placeholders, lines hit of 8 | F3 trailing blank |\n'
printf '|---|---:|---|---|---|\n'
tmp=$(mktemp -d)
for tool in "${CHECKERS[@]}"; do
  for f in control f1-before f1-after f2-template f3-before f3-after; do
    "lint_${tool}" "${fx}/${f}.md" >"${tmp}/${tool}.${f}"
  done
  control=$(wc -l <"${tmp}/${tool}.control")
  f1=$(delta "${tmp}/${tool}.f1-before" "${tmp}/${tool}.f1-after")
  f3=$(delta "${tmp}/${tool}.f3-before" "${tmp}/${tool}.f3-after")
  f2=$(awk -F'\t' -v want="${f2_lines}" 'index(want, " " $2 " ") {print $2 ":" $3}' \
    "${tmp}/${tool}.f2-template" | sort -t: -k1,1n -u)
  f2_hit=$(cut -d: -f1 <<<"${f2}" | sort -u | grep -c .)
  f2_rules=$(cut -d: -f2 <<<"${f2}" | sort -u | paste -sd, -)
  [[ "${control}" -gt 0 ]] || { f1=VOID f2_hit=VOID f3=VOID; }
  printf '| %s | %s | %s | %s %s | %s |\n' "${tool}" "${control}" "${f1:-none}" \
    "${f2_hit}" "${f2_rules:+(${f2_rules})}" "${f3:-none}"
done

# A token rule instead of a linter: a list marker that opens mid-line, right
# after a non-space character. Run over every Markdown file at every commit on
# main, it shows its false positives as well as its catch.
printf '\n### The token rule `[^[:space:]]- \\*\\*` over history\n\n'
printf 'F1 before: %s line(s) · F1 after: %s line(s)\n\n' \
  "$(grep -c -E '[^[:space:]]- \*\*' "${fx}/f1-before.md")" \
  "$(grep -c -E '[^[:space:]]- \*\*' "${fx}/f1-after.md")"
printf "| file:line | text |\n|---|---|\n"
git log --format=%h main -- '*.md' | while read -r sha; do
  git grep -n -E '[^[:space:]]- \*\*' "${sha}" -- '*.md' 2>/dev/null |
    sed -E 's/^[0-9a-f]+://' | while IFS=: read -r file line text; do
    printf '%s\t%s\t%s\n' "${file}" "${line}" "${text:0:60}"
  done
done | sort -u | awk -F'\t' '{printf "| %s | `%s` |\n", $1 ":" $2, $3}' | head -20
rm -rf "${tmp}"
