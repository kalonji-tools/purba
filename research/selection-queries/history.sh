#!/usr/bin/env bash
# Print the history of each runner's selection language, from clones pinned at
# a tag and from GitHub's commit list for Go. Matchers are printed beside each
# count.
# Usage: bash history.sh <pytest clone> <nextest clone> <vitest clone>
# Each clone needs its history: git clone --filter=blob:none is enough.
set -euo pipefail
pytest=$1
nextest=$2
vitest=$3

echo '## pytest 9.2.0: commits on src/_pytest/mark/expression.py'
git -C "$pytest" log --format='%h %ad %s' --date=short 9.2.0 -- src/_pytest/mark/expression.py | tee /dev/stderr | wc -l
echo '## pytest 9.2.0: changelog lines in a "Bug Fixes" section that name -k, -m or an expression'
awk '/^pytest [0-9]+\.[0-9]+\.[0-9]+/{v=$2}
    /^(Bug Fixes|Bug fixes)$/{s="fix";next}
    /^[A-Z][A-Za-z ,]+$/ && !/^(Bug Fixes|Bug fixes)$/{s="other"}
    s=="fix" && /``-k``|``-m``|`-k`|`-m`|keyword expression|marker expression|match expression/ {print v": "substr($0,1,200)}' \
  "$pytest/doc/en/changelog.rst"

echo '## cargo-nextest 0.9.149: commits on nextest-filtering/'
git -C "$nextest" log --oneline cargo-nextest-0.9.149 -- nextest-filtering | wc -l
echo '## cargo-nextest 0.9.149: entries under "### Fixed" in site/src/changelog.md that name a filter or an expression'
awk '/^## \[/{v=$2} /^### /{s=$0}
    s ~ /Fixed/ && tolower($0) ~ /filter|expression/ && $0 !~ /^###/ {print NR": "v" "substr($0,1,200)}' \
  "$nextest/site/src/changelog.md"
echo '## cargo-nextest 0.9.149: entries under "### Fixed" in nextest-filtering/CHANGELOG.md'
awk '/^## \[/{v=$2} /^### /{s=$0;next} /^## /{s="";next} s ~ /Fixed/ && NF && !/^\[#/ {print v" "substr($0,1,200)}' \
  "$nextest/nextest-filtering/CHANGELOG.md"

echo '## Vitest v5.1.1: commits on the tags files, before and after the move of @vitest/runner'
git -C "$vitest" log --format='%h %ad %s' --date=short v5.1.1 -- \
  packages/runner/src/utils/tags.ts packages/vitest/src/runtime/runner/utils/tags.ts packages/vitest/src/node/tags.ts

for p in src/testing/match.go src/go/build/constraint/expr.go; do
  echo "## go1.26.10: commits on $p (GitHub commit list)"
  gh api "repos/golang/go/commits?path=${p}&sha=go1.26.10&per_page=100" --paginate \
    -q '.[] | .sha[:10] + " " + .commit.author.date[:10] + " " + (.commit.message | split("\n")[0])'
done
