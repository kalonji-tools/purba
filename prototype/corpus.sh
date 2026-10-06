#!/usr/bin/env bash
# Rebuild the two corpora into prototype/corpus/, which git ignores.
#
#   records  the records on `main` at 4d5eb7b
#   wild     the other Markdown at 4d5eb7b, and the bodies of the 150 issues
#            out/wild-issues.txt names
#
# An issue body can change after 2026-10-06, so a rebuilt `wild` can differ.
set -euo pipefail
here=$(cd "$(dirname "$0")" && pwd)
out=${here}/corpus
rm -rf "${out}"
mkdir -p "${out}/records" "${out}/wild" "${out}/tree"
git -C "${here}/.." archive 4d5eb7b | tar -x -C "${out}/tree"
cp "${out}"/tree/docs/decisions/*.md "${out}/records/"
(cd "${out}/tree" && find . -name '*.md' -not -path './docs/decisions/*') |
  while read -r f; do
    name=${f//\//_}
    cp "${out}/tree/${f}" "${out}/wild/repo-${name//./_}.md"
  done
want=$(jq -s . "${here}/out/wild-issues.txt")
gh issue list --repo kalonji-tools/purba --state all --limit 400 --json number,body |
  jq -c --argjson want "${want}" '.[] | select(.number as $n | $want | index($n))' |
  while read -r issue; do
    number=$(jq -r .number <<<"${issue}")
    jq -r .body <<<"${issue}" >"${out}/wild/issue-${number}.md"
  done
find "${out}/records" -name '*.md' | wc -l
find "${out}/wild" -name '*.md' | wc -l
