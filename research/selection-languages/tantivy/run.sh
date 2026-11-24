#!/usr/bin/env bash
# Probe tantivy's query parser, a syntax of the Lucene family. The probe indexes the suite in
# RAM, with the fields id, module and tags, and tags as the default field. Each case prints
# tantivy's grammar tree, the query, and the ids it selects. A nesting case prints how far
# each stage got.
# Usage: bash run.sh <empty dir>
set -uo pipefail
work=$1
here=$(cd "$(dirname "$0")" && pwd)
cp -r "$here/Cargo.toml" "$here/Cargo.lock" "$here/src" "$work/"
cd "$work" || exit 1
cargo --version
cargo build --release --locked -q 2>&1
bin=$work/target/release/p-tantivy

ask() {
  printf -- '--- %s\n' "$1"
  "$bin" "$1" 2>/dev/null
  printf 'exit %s\n' "$?"
}

nest() {
  awk -v n="$1" -v pre="$2" -v mid="$3" -v post="$4" \
    'BEGIN{s="";for(i=0;i<n;i++)s=s pre;s=s mid;for(i=0;i<n;i++)s=s post;printf "%s",s}'
}

deep() {
  local pre=$1 mid=$2 post=$3 n out code
  for n in 100 1000 3000 10000; do
    out=$(nest "$n" "$pre" "$mid" "$post" | "$bin" - 2>&1)
    code=$?
    printf '[%s] x %s: exit %s; %s\n' "$pre" "$n" "$code" \
      "$(printf '%s\n' "$out" | grep -o 'stage: [a-z ]*\|overflowed its stack\|parse error: [A-Za-z ]*' | sort -u | tr '\n' ' ')"
  done
}

echo '== the example, with AND and NOT, and with + and -'
ask 'module:store AND tags:slow AND NOT tags:flaky'
ask '+module:store +slow -flaky'
echo '== precedence: AND before OR selects 5; left to right selects 3'
ask 'db OR slow AND NOT db'
ask 'db OR (slow -db)'
echo '== an id of purba, and a path'
ask 'store::migrate'
ask 'id:store::migrate'
ask 'id:"store::migrate"'
ask 'tests/api'
echo '== an error at a position'
ask 'slow AND'
ask 'slow AND (db'
ask 'nosuchfield:x'
echo '== nesting'
deep '(' 'slow' ')'
deep '(slow AND ' 'slow' ')'
