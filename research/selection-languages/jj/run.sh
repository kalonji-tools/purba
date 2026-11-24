#!/usr/bin/env bash
# Probe Jujutsu's revset parser, in the crate jj-dsl. The functions tag() and module() are
# the probe's own: jj resolves names to commits, and this probe selects tests. The probe holds
# two aliases, as a [revset-aliases] table would. A nesting case prints how far it got.
# Usage: bash run.sh <empty dir>
set -uo pipefail
work=$1
here=$(cd "$(dirname "$0")" && pwd)
cp -r "$here/Cargo.toml" "$here/Cargo.lock" "$here/src" "$work/"
cd "$work" || exit 1
cargo --version
cargo build --release --locked -q 2>&1
bin=$work/target/release/p-jj

ask() {
  printf -- '--- %s\n' "$1"
  "$bin" "$1" 2>/dev/null
  printf 'exit %s\n' "$?"
}

nest() {
  awk -v n="$1" -v pre="$2" -v mid="$3" -v post="$4" \
    'BEGIN{s="";for(i=0;i<n;i++)s=s pre;s=s mid;for(i=0;i<n;i++)s=s post;printf "%s",s}'
}

# limit <before> <middle> <after>: the largest n up to 300 that parses.
limit() {
  local n last=0
  for n in $(seq 1 300); do
    nest "$n" "$1" "$2" "$3" | "$bin" - >/dev/null 2>&1 && last=$n
  done
  printf '[%s]: the largest n that parses is %s\n' "$1" "$last"
}

echo '== the example, and the same query through the aliases slow_store and stable(x)'
ask 'module(store) & tag(slow) ~ tag(flaky)'
ask 'stable(slow_store)'
echo '== precedence: AND before OR selects 5; left to right selects 3'
ask 'tag(db) | tag(slow) & ~tag(db)'
ask 'tag(db) | tag(slow) ~ tag(db)'
echo '== an id of purba, and a path. The probe cannot select by a bare name, and exits 101.'
ask 'store::migrate'
ask 'store::migrate::case'
ask 'tests/api'
echo '== an error at a position'
ask 'tag(slow) &'
ask 'tag(slow) & (tag(db)'
ask 'tag(slow) && tag(db)'
echo '== nesting'
limit '(' 'tag(slow)' ')'
limit '~' 'tag(slow)' ''
limit '(tag(slow) & ' 'tag(slow)' ')'
ask "$(nest 300 '(' 'tag(slow)' ')')"
nest 10000 '(' 'tag(slow)' ')' | "$bin" - 2>&1 | grep -o 'stage: [a-z]*\|overflowed its stack\|= .*'
nest 10000 '(tag(slow) & ' 'tag(slow)' ')' | "$bin" - 2>&1 | grep -o 'stage: [a-z]*\|overflowed its stack\|= .*'
