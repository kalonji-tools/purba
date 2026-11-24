#!/usr/bin/env bash
# Probe the crate nextest-filtering, the parser of cargo-nextest's filtersets. Each case runs
# two parses: ParsedExpr::parse, which needs no package graph, and, with --strict,
# Filterset::parse, the parse cargo-nextest runs, against a graph of this probe's own project.
# A nesting case prints how far each stage got.
# Usage: bash run.sh <empty dir>
set -uo pipefail
work=$1
here=$(cd "$(dirname "$0")" && pwd)
cp -r "$here/Cargo.toml" "$here/Cargo.lock" "$here/src" "$work/"
cd "$work" || exit 1
cargo --version
cargo build --release --locked -q 2>&1
bin=$work/target/release/p-nextest

ask() {
  printf -- '--- %s\n' "$1"
  "$bin" "$1" 2>/dev/null
  printf 'exit %s\n' "$?"
  "$bin" --strict "$1" 2>/dev/null
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
      "$(printf '%s\n' "$out" | grep -o 'stage: [a-z]*\|overflowed its stack' | sort -u | tr '\n' ' ')"
  done
}

echo '== the example. nextest has no tag, so each name carries its tags.'
ask 'test(store::) & test(slow) - test(flaky)'
echo '== precedence: AND before OR selects 5; left to right selects 3'
ask 'test(db) | test(slow) & not test(db)'
echo '== an id of purba, a path, and a predicate nextest does not have'
ask 'store::migrate'
ask 'tests/api'
ask 'tag(slow)'
echo '== an error at a position'
ask 'test(slow) &'
ask 'test(slow) & (test(db)'
echo '== nesting'
deep '(' 'test(slow)' ')'
deep 'not ' 'test(slow)' ''
deep '(test(slow) & ' 'test(slow)' ')'
