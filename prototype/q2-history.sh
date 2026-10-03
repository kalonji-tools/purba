#!/usr/bin/env bash
# PROTOTYPE, never merged. When a person fixed a real typo, would `typos -w`
# have written the same fix?
#
# q2-where.sh measures trees that already ran codespell, so the real typos were
# gone before it looked and only false positives were left. This measures the
# other side: every commit in three histories whose message says it fixes a
# typo or a spelling. Its removed lines are the typo and its added lines are
# the person's fix. `typos -w` runs on the removed lines, by file type:
#   same       a line typos wrote is one the person wrote
#   other      typos rewrote a line, and the person wrote something else
#   missed     the person changed a line typos left alone
set -u
cd "$(dirname "$0")/.." || exit 2
here=$PWD
printf '| tree | commits | lines a person fixed | typos wrote the same | typos wrote something else | typos missed |\n'
printf '|---|---:|---:|---:|---:|---:|\n'
for tree in ../oxitest ../click ../loguru; do
  [[ -d "${tree}/.git" ]] || continue
  tmp=$(mktemp -d)
  mapfile -t shas < <(git -C "${tree}" log --no-merges --format=%h -i -E \
    --grep="(fix|correct)[a-z]* .{0,20}(typo|spelling|misspel)" -- . ":!*.min.js")
  for sha in "${shas[@]}"; do
    git -C "${tree}" show --format= -U0 --no-color "${sha}" -- . ':!*.min.js' |
      awk -v dir="${tmp}" -v sha="${sha}" '
        /^\+\+\+ b\// {f=substr($0, 7); n=split(f, p, "."); ext=(n > 1 ? p[n] : "txt"); next}
        /^--- / {next}
        /^-/ {print substr($0, 2) >> (dir "/" sha "." ext ".removed"); next}
        /^\+/ {print substr($0, 2) >> (dir "/" sha "." ext ".added")}'
  done
  for r in "${tmp}"/*.removed; do
    [[ -e "${r}" ]] || continue
    a=${r%.removed}.added
    [[ -e "${a}" ]] || continue
    ext=${r%.removed} ext=${ext##*.}
    cp "${r}" "${tmp}/w.${ext}"
    (cd "${here}" && mise x -- typos -w "${tmp}/w.${ext}") >/dev/null 2>&1
    # typos -w keeps the line count, so line i of the copy is line i of the
    # removed text.
    awk -v added="${a}" '
      BEGIN {while ((getline l < added) > 0) keep[l] = 1}
      NR == FNR {w[FNR] = $0; next}
      !($0 in keep) {print "fixed"}
      w[FNR] != $0 {print ((w[FNR] in keep) ? "same" : "other"); next}
      !($0 in keep) {print "missed"}' "${tmp}/w.${ext}" "${r}" >>"${tmp}/verdicts"
  done
  same=$(grep -cx same "${tmp}/verdicts" 2>/dev/null || true)
  other=$(grep -cx other "${tmp}/verdicts" 2>/dev/null || true)
  missed=$(grep -cx missed "${tmp}/verdicts" 2>/dev/null || true)
  fixed=$(grep -cx fixed "${tmp}/verdicts" 2>/dev/null || true)
  printf '| %s | %s | %s | %s | %s | %s |\n' "$(basename "${tree}")" "${#shas[@]}" "${fixed}" \
    "${same:-0}" "${other:-0}" "${missed:-0}"
  rm -rf "${tmp}"
done
