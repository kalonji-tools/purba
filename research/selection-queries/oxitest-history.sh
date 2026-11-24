#!/usr/bin/env bash
# Print the history of oxitest's query language: commits on its files, the
# commits whose subject starts with "fix", and the issues that name it.
# Usage:
#   gh issue list --repo kalonji-tools/oxitest --state all --limit 4000 \
#     --json number,title,state,labels,body > issues.json
#   bash oxitest-history.sh <oxitest checkout> <python> issues.json
# Matchers are printed beside each count.
set -euo pipefail
repo=$1
py=$2
issues=$3
rev=731c2b3f
core="src/query/dsl.rs src/query/ast.rs src/query/compile.rs src/query/eval.rs src/query/resource.rs"
all="src/query src/config/cli/query.rs python/oxitest/_bridge/query_bridge.py src/marker.rs"
printf 'commits on the core (%s): %s\n' "$core" "$(git -C "$repo" log --oneline "$rev" -- $core | wc -l)"
printf 'commits on all query files (%s): %s\n' "$all" "$(git -C "$repo" log --oneline "$rev" -- $all | wc -l)"
echo 'commits on all query files whose subject starts with "fix":'
git -C "$repo" log --format='%h %ad %s' --date=short "$rev" -- $all | grep -E '^\S+ \S+ fix' || true
echo 'commits on the core:'
git -C "$repo" log --format='%h %ad %s' --date=short "$rev" -- $core
"$py" -I - "$issues" <<'PY'
import json, re, sys
issues = json.load(open(sys.argv[1]))
title = re.compile(r'\bdsl\b|\bquery\b|-E\b|\bfilter|node.?id|\bfzf\b|\bpredicate|\bselect', re.I)
body = re.compile(r'query DSL|\bDSL\b|-E [\'"]|\bname\(|\bmark\(|src/query/(ast|compile|eval)')
t = {i['number'] for i in issues if title.search(i['title'])}
b = {i['number'] for i in issues if body.search(i['body'] or '')}
print(f'issues: {len(issues)}; title matcher {title.pattern!r}: {len(t)}; '
      f'body matcher {body.pattern!r}: {len(b)}; either: {len(t | b)}')
for i in sorted(issues, key=lambda x: x['number']):
    if i['number'] in t | b:
        labels = ','.join(l['name'] for l in i['labels'])
        print(f"#{i['number']}\t{i['state']}\t{labels}\t{i['title']}")
PY
